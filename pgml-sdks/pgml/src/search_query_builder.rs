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
struct ValidQueryAction {
    full_text_search: Option<HashMap<String, ValidMatchAction>>,
    semantic_search: Option<HashMap<String, ValidSemanticSearchAction>>,
}

#[derive(Debug, Deserialize)]
struct ValidQuery {
    query: ValidQueryAction,
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

    let mut with_clause = WithClause::new();
    let mut sub_query = Query::select();
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

                // Add to the sum expression
                let boost = vsa.boost.unwrap_or(1.);
                sum_expression = if let Some(expr) = sum_expression {
                    Some(expr.add(Expr::cust(format!(
                        // r#"((1 - MIN("{key}_embeddings".embedding <=> (SELECT embedding FROM "{key}_embedding")::vector)) * {boost})"#
                        r#"(MIN("{key}_embeddings".embedding <=> (SELECT embedding FROM "{key}_embedding")::vector))"#
                    ))))
                } else {
                    Some(Expr::cust(format!(
                        // r#"((1 - MIN("{key}_embeddings".embedding <=> (SELECT embedding FROM "{key}_embedding")::vector)) * {boost})"#
                        r#"(MIN("{key}_embeddings".embedding <=> (SELECT embedding FROM "{key}_embedding")::vector))"#
                    )))
                };
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

                // Add to the sum expression
                let boost = vsa.boost.unwrap_or(1.);
                sum_expression = if let Some(expr) = sum_expression {
                    Some(expr.add(Expr::cust_with_values(
                        format!(
                            // r#"((1 - MIN("{key}_embeddings".embedding <=> $1::vector)) * {boost})"#,
                            r#"(MIN("{key}_embeddings".embedding <=> $1::vector))"#,
                        ),
                        [embedding],
                    )))
                } else {
                    Some(Expr::cust_with_values(
                        format!(
                            r#"(MIN("{key}_embeddings".embedding <=> $1::vector))"# // r#"((1 - MIN("{key}_embeddings".embedding <=> $1::vector)) * {boost})"#
                        ),
                        [embedding],
                    ))
                };
            }
        }

        // Do the proper inner joins
        let chunks_table = format!("{}_{}.{}_chunks", collection.name, pipeline.name, key);
        let embeddings_table = format!("{}_{}.{}_embeddings", collection.name, pipeline.name, key);
        sub_query.join_as(
            JoinType::InnerJoin,
            chunks_table.to_table_tuple(),
            Alias::new(format!("{key}_chunks")),
            Expr::col((
                SIden::String(format!("{key}_chunks")),
                SIden::Str("document_id"),
            ))
            .equals((SIden::Str("documents"), SIden::Str("id"))),
        );
        sub_query.join_as(
            JoinType::InnerJoin,
            embeddings_table.to_table_tuple(),
            Alias::new(format!("{key}_embeddings")),
            Expr::col((
                SIden::String(format!("{key}_embeddings")),
                SIden::Str("chunk_id"),
            ))
            .equals((SIden::String(format!("{key}_chunks")), SIden::Str("id"))),
        );
    }

    for (key, vma) in valid_query.query.full_text_search.unwrap_or_default() {
        let full_text_table = format!("{}_{}.{}_tsvectors", collection.name, pipeline.name, key);

        // Inner join the tsvectors table
        sub_query.join_as(
            JoinType::InnerJoin,
            full_text_table.to_table_tuple(),
            Alias::new(format!("{key}_tsvectors")),
            Expr::col((
                SIden::String(format!("{key}_tsvectors")),
                SIden::Str("document_id"),
            ))
            .equals((SIden::Str("documents"), SIden::Str("id"))),
        );

        // TODO: Maybe add this??
        // Do the proper where statement
        // sub_query.and_where(Expr::cust_with_values(
        //     format!(
        //         r#""{key}_tsvectors".ts @@ plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1)"#,
        //     ),
        //     [&vma.query],
        // ));

        // Add to the sum expression
        let boost = vma.boost.unwrap_or(1.);
        sum_expression = if let Some(expr) = sum_expression {
            Some(expr.add(Expr::cust_with_values(format!(
                r#"(MAX(ts_rank("{key}_tsvectors".ts, plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1), 32)) * {boost})"#,
            ),
            [vma.query]
            )))
        } else {
            Some(Expr::cust_with_values(
                format!(
                    r#"(MAX(ts_rank("{key}_tsvectors".ts, plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1), 32)) * {boost})"#,
                ),
                [vma.query],
            ))
        };
    }

    // Finalize the sub query
    sub_query
        .column((SIden::Str("documents"), SIden::Str("document")))
        .expr_as(sum_expression.unwrap(), Alias::new("score"))
        .from_as(documents_table.to_table_tuple(), Alias::new("documents"))
        .group_by_col((SIden::Str("documents"), SIden::Str("id")))
        .order_by(SIden::Str("score"), Order::Desc)
        .limit(limit);

    // Combine to make the real query
    let mut sql_query = Query::select();
    sql_query
        .expr(Expr::cust("json_array_elements(json_agg(q))"))
        .from_subquery(sub_query, Alias::new("q"));

    let query_string = sql_query
        .clone()
        .with(with_clause.clone())
        .to_string(PostgresQueryBuilder);
    println!("{}", query_string);

    let (sql, values) = sql_query.with(with_clause).build_sqlx(PostgresQueryBuilder);
    Ok((sql, values))
}
