use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;

use sea_query::{
    Alias, CommonTableExpression, Expr, Func, JoinType, Order, PostgresQueryBuilder, Query,
    QueryStatementWriter, SimpleExpr, WithClause,
};
use sea_query_binder::{SqlxBinder, SqlxValues};

use crate::{
    collection::Collection,
    filter_builder::FilterBuilder,
    model::ModelRuntime,
    models,
    multi_field_pipeline::MultiFieldPipeline,
    remote_embeddings::build_remote_embeddings,
    types::{IntoTableNameAndSchema, Json, SIden},
};

#[derive(Debug, Deserialize)]
struct ValidSemanticSearchAction {
    query: String,
    model_parameters: Option<Json>,
    boost: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ValidMatchAction {
    query: String,
    boost: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ValidQueryActions {
    full_text_search: Option<HashMap<String, ValidMatchAction>>,
    semantic_search: Option<HashMap<String, ValidSemanticSearchAction>>,
    filter: Option<Json>,
}

#[derive(Debug, Deserialize)]
struct ValidQuery {
    query: ValidQueryActions,
    limit: Option<u64>,
}

pub async fn build_search_query(
    collection: &Collection,
    query: Json,
    pipeline: &MultiFieldPipeline,
) -> anyhow::Result<(String, SqlxValues)> {
    let valid_query: ValidQuery = serde_json::from_value(query.0)?;
    let limit = valid_query.limit.unwrap_or(10);

    let pipeline_table = format!("{}.pipelines", collection.name);
    let documents_table = format!("{}.documents", collection.name);

    let mut query = Query::select();
    let mut score_table_names = Vec::new();
    let mut with_clause = WithClause::new();
    let mut sum_expression: Option<SimpleExpr> = None;

    let mut pipeline_cte = Query::select();
    pipeline_cte
        .from(pipeline_table.to_table_tuple())
        .columns([models::PipelineIden::Schema])
        .and_where(Expr::col(models::PipelineIden::Name).eq(&pipeline.name));
    let mut pipeline_cte = CommonTableExpression::from_select(pipeline_cte);
    pipeline_cte.table_name(Alias::new("pipeline"));
    with_clause.cte(pipeline_cte);

    for (key, vsa) in valid_query.query.semantic_search.unwrap_or_default() {
        let model_runtime = pipeline
            .parsed_schema
            .as_ref()
            .map(|s| {
                // Any of these errors means they have a malformed query
                anyhow::Ok(
                    s.get(&key)
                        .as_ref()
                        .context(format!("Bad query - {key} does not exist in schema"))?
                        .embed
                        .as_ref()
                        .context(format!(
                            "Bad query - {key} does not have any directive to embed"
                        ))?
                        .model
                        .runtime,
                )
            })
            .transpose()?
            .unwrap_or(ModelRuntime::Python);

        // Build the CTE we actually use later
        let embeddings_table = format!("{}_{}.{}_embeddings", collection.name, pipeline.name, key);
        let cte_name = format!("{key}_embedding_score");
        let mut score_cte = Query::select();
        match model_runtime {
            ModelRuntime::Python => {
                // Build the embedding CTE
                let mut embedding_cte = Query::select();
                embedding_cte.expr_as(
                    Func::cust(SIden::Str("pgml.embed")).args([
                        Expr::cust(format!(
                            "transformer => (SELECT schema #>> '{{{key},embed,model}}' FROM pipeline)",
                        )),
                        Expr::cust_with_values("text => $1", [&vsa.query]),
                        Expr::cust(format!("kwargs => COALESCE((SELECT schema #> '{{{key},embed,model_parameters}}' FROM pipeline), '{{}}'::jsonb)")),
                    ]),
                    Alias::new("embedding"),
                );
                let mut embedding_cte = CommonTableExpression::from_select(embedding_cte);
                embedding_cte.table_name(Alias::new(format!("{key}_embedding")));
                with_clause.cte(embedding_cte);

                // Build the score CTE
                score_cte
                    .column((SIden::Str("embeddings"), SIden::Str("document_id")))
                    .expr(Expr::cust(format!(
                        r#"MIN(embeddings.embedding <=> (SELECT embedding FROM "{key}_embedding")::vector) AS score"#
                    )))
                    .order_by_expr(Expr::cust(format!(
                        r#"embeddings.embedding <=> (SELECT embedding FROM "{key}_embedding")::vector"#
                    )), Order::Asc )
            }
            ModelRuntime::OpenAI => {
                // We can unwrap here as we know this is all set from above
                let model = &pipeline
                    .parsed_schema
                    .as_ref()
                    .unwrap()
                    .get(&key)
                    .unwrap()
                    .embed
                    .as_ref()
                    .unwrap()
                    .model;

                // Get the remote embedding
                let embedding = {
                    let remote_embeddings = build_remote_embeddings(
                        model.runtime,
                        &model.name,
                        vsa.model_parameters.as_ref(),
                    )?;
                    let mut embeddings = remote_embeddings.embed(vec![vsa.query]).await?;
                    std::mem::take(&mut embeddings[0])
                };

                // Build the score CTE
                score_cte
                    .column((SIden::Str("embeddings"), SIden::Str("document_id")))
                    .expr(Expr::cust_with_values(
                        r#"MIN(embeddings.embedding <=> $1::vector) AS score"#,
                        [embedding.clone()],
                    ))
                    .order_by_expr(
                        Expr::cust_with_values(
                            r#"embeddings.embedding <=> $1::vector"#,
                            [embedding],
                        ),
                        Order::Asc,
                    )
            }
        };

        score_cte
            .from_as(embeddings_table.to_table_tuple(), Alias::new("embeddings"))
            .group_by_col((SIden::Str("embeddings"), SIden::Str("id")))
            .limit(limit);

        if let Some(filter) = &valid_query.query.filter {
            let filter = FilterBuilder::new(filter.clone().0, "documents", "document").build()?;
            score_cte.cond_where(filter);
            score_cte.join_as(
                JoinType::InnerJoin,
                documents_table.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .equals((SIden::Str("embeddings"), SIden::Str("document_id"))),
            );
        }

        let mut score_cte = CommonTableExpression::from_select(score_cte);
        score_cte.table_name(Alias::new(&cte_name));
        with_clause.cte(score_cte);

        // Add to the sum expression
        let boost = vsa.boost.unwrap_or(1.);
        sum_expression = if let Some(expr) = sum_expression {
            Some(expr.add(Expr::cust(format!(
                r#"COALESCE((1 - "{cte_name}".score) * {boost}, 0.0)"#
            ))))
        } else {
            Some(Expr::cust(format!(
                r#"COALESCE((1 - "{cte_name}".score) * {boost}, 0.0)"#
            )))
        };
        score_table_names.push(cte_name);
    }

    for (key, vma) in valid_query.query.full_text_search.unwrap_or_default() {
        let full_text_table = format!("{}_{}.{}_tsvectors", collection.name, pipeline.name, key);

        // Build the score CTE
        let cte_name = format!("{key}_tsvectors_score");
        let mut score_cte = Query::select();
        score_cte
            .column(SIden::Str("document_id"))
            .expr_as(
                Expr::cust_with_values(
                    format!(
                        r#"MAX(ts_rank(tsvectors.ts, plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1), 32))"#,
                    ),
                    [&vma.query],
                ),
                Alias::new("score")
            )
            .from_as(
                full_text_table.to_table_tuple(),
                Alias::new("tsvectors"),
            )
            .and_where(Expr::cust_with_values(
                format!(
                    r#"tsvectors.ts @@ plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1)"#,
                ),
                [&vma.query],
            ))
            .group_by_col(SIden::Str("document_id"))
            .order_by(SIden::Str("score"), Order::Desc)
            .limit(limit);

        if let Some(filter) = &valid_query.query.filter {
            let filter = FilterBuilder::new(filter.clone().0, "documents", "document").build()?;
            score_cte.cond_where(filter);
            score_cte.join_as(
                JoinType::InnerJoin,
                documents_table.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .equals((SIden::Str("tsvectors"), SIden::Str("document_id"))),
            );
        }

        let mut score_cte = CommonTableExpression::from_select(score_cte);
        score_cte.table_name(Alias::new(&cte_name));
        with_clause.cte(score_cte);

        // Add to the sum expression
        let boost = vma.boost.unwrap_or(1.0);
        sum_expression = if let Some(expr) = sum_expression {
            Some(expr.add(Expr::cust(format!(
                r#"COALESCE("{cte_name}".score * {boost}, 0.0)"#
            ))))
        } else {
            Some(Expr::cust(format!(
                r#"COALESCE("{cte_name}".score * {boost}, 0.0)"#
            )))
        };
        score_table_names.push(cte_name);
    }

    let query = if let Some(select_from) = score_table_names.first() {
        let score_table_names_e: Vec<SimpleExpr> = score_table_names
            .clone()
            .into_iter()
            .map(|t| Expr::col((SIden::String(t), SIden::Str("document_id"))).into())
            .collect();
        for i in 1..score_table_names_e.len() {
            query.full_outer_join(
                SIden::String(score_table_names[i].to_string()),
                Expr::col((
                    SIden::String(score_table_names[i].to_string()),
                    SIden::Str("document_id"),
                ))
                .eq(Func::coalesce(score_table_names_e[0..i].to_vec())),
            );
        }
        let id_select_expression = Func::coalesce(score_table_names_e);

        let sum_expression = sum_expression
            .context("query requires some scoring through full_text_search or semantic_search")?;
        query
            // .expr_as(id_select_expression.clone(), Alias::new("id"))
            .expr(Expr::cust_with_expr(
                "DISTINCT ON ($1) $1 as id",
                id_select_expression.clone(),
            ))
            .expr_as(sum_expression, Alias::new("score"))
            .column(SIden::Str("document"))
            .from(SIden::String(select_from.to_string()))
            .join_as(
                JoinType::InnerJoin,
                documents_table.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .eq(id_select_expression.clone()),
            )
            .order_by_expr(
                Expr::cust_with_expr("$1, score", id_select_expression),
                Order::Desc,
            );
        // .order_by(SIden::Str("score"), Order::Desc);

        let mut re_ordered_query = Query::select();
        re_ordered_query
            .expr(Expr::cust("*"))
            .from_subquery(query, Alias::new("q1"))
            .order_by(SIden::Str("score"), Order::Desc)
            .limit(limit);

        let mut combined_query = Query::select();
        combined_query
            .expr(Expr::cust("json_array_elements(json_agg(q2))"))
            .from_subquery(re_ordered_query, Alias::new("q2"));
        combined_query
    } else {
        // TODO: Maybe let users filter documents only here?
        anyhow::bail!("If you are only looking to filter documents checkout the `get_documents` method on the Collection")
    };

    // TODO: Remove this
    let query_string = query
        .clone()
        .with(with_clause.clone())
        .to_string(PostgresQueryBuilder);
    println!("\nTHE QUERY: \n{query_string}\n");

    let (sql, values) = query.with(with_clause).build_sqlx(PostgresQueryBuilder);
    Ok((sql, values))
}
