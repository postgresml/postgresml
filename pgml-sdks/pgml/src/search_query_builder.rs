use anyhow::Context;
use sea_query::{
    Alias, CommonTableExpression, Expr, Func, JoinType, Order, PostgresQueryBuilder, Query,
    SimpleExpr, WithClause,
};
use sea_query_binder::{SqlxBinder, SqlxValues};
use serde::Deserialize;
use serde_with::{serde_as, FromInto};
use std::collections::HashMap;

use crate::{
    collection::Collection,
    debug_sea_query,
    filter_builder::FilterBuilder,
    model::ModelRuntime,
    models,
    pipeline::Pipeline,
    remote_embeddings::build_remote_embeddings,
    types::{CustomU64Convertor, IntoTableNameAndSchema, Json, SIden},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidSemanticSearchAction {
    query: String,
    parameters: Option<Json>,
    boost: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidFullTextSearchAction {
    query: String,
    boost: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidQueryActions {
    full_text_search: Option<HashMap<String, ValidFullTextSearchAction>>,
    semantic_search: Option<HashMap<String, ValidSemanticSearchAction>>,
    filter: Option<Json>,
}

const fn default_limit() -> u64 {
    10
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidQuery {
    query: ValidQueryActions,
    // Need this when coming from JavaScript as everything is an f64 from JS
    #[serde(default = "default_limit")]
    #[serde_as(as = "FromInto<CustomU64Convertor>")]
    limit: u64,
}

pub async fn build_search_query(
    collection: &Collection,
    query: Json,
    pipeline: &Pipeline,
) -> anyhow::Result<(String, SqlxValues)> {
    let valid_query: ValidQuery = serde_json::from_value(query.0.clone())?;
    let limit = valid_query.limit;

    let pipeline_table = format!("{}.pipelines", collection.name);
    let documents_table = format!("{}.documents", collection.name);

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
                        .semantic_search
                        .as_ref()
                        .context(format!(
                            "Bad query - {key} does not have any directive to semantic_search"
                        ))?
                        .model
                        .runtime,
                )
            })
            .transpose()?
            .unwrap_or(ModelRuntime::Python);

        // Build the CTE we actually use later
        let embeddings_table = format!("{}_{}.{}_embeddings", collection.name, pipeline.name, key);
        let chunks_table = format!("{}_{}.{}_chunks", collection.name, pipeline.name, key);
        let cte_name = format!("{key}_embedding_score");
        let boost = vsa.boost.unwrap_or(1.);
        let mut score_cte_non_recursive = Query::select();
        let mut score_cte_recurisive = Query::select();
        match model_runtime {
            ModelRuntime::Python => {
                // Build the embedding CTE
                let mut embedding_cte = Query::select();
                embedding_cte.expr_as(
                    Func::cust(SIden::Str("pgml.embed")).args([
                        Expr::cust(format!(
                            "transformer => (SELECT schema #>> '{{{key},semantic_search,model}}' FROM pipeline)",
                        )),
                        Expr::cust_with_values("text => $1", [&vsa.query]),
                        Expr::cust_with_values("kwargs => $1", [vsa.parameters.unwrap_or_default().0]),
                    ]),
                    Alias::new("embedding"),
                );
                let mut embedding_cte = CommonTableExpression::from_select(embedding_cte);
                embedding_cte.table_name(Alias::new(format!("{key}_embedding")));
                with_clause.cte(embedding_cte);

                score_cte_non_recursive
                    .from_as(embeddings_table.to_table_tuple(), Alias::new("embeddings"))
                    .column((SIden::Str("documents"), SIden::Str("id")))
                    .join_as(
                        JoinType::InnerJoin,
                        chunks_table.to_table_tuple(),
                        Alias::new("chunks"),
                        Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                            .equals((SIden::Str("embeddings"), SIden::Str("chunk_id"))),
                    )
                    .join_as(
                        JoinType::InnerJoin,
                        documents_table.to_table_tuple(),
                        Alias::new("documents"),
                        Expr::col((SIden::Str("documents"), SIden::Str("id")))
                            .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
                    )
                    .expr(Expr::cust(r#"ARRAY[documents.id] as previous_document_ids"#))
                    .expr(Expr::cust(format!(
                        r#"(1 - (embeddings.embedding <=> (SELECT embedding FROM "{key}_embedding")::vector)) * {boost} AS score"#
                    )))
                    .order_by_expr(Expr::cust(format!(
                        r#"embeddings.embedding <=> (SELECT embedding FROM "{key}_embedding")::vector"#
                    )), Order::Asc )
                    .limit(1);

                score_cte_recurisive
                    .from_as(embeddings_table.to_table_tuple(), Alias::new("embeddings"))
                    .column((SIden::Str("documents"), SIden::Str("id")))
                    .expr(Expr::cust(format!(r#""{cte_name}".previous_document_ids || documents.id"#)))
                    .expr(Expr::cust(format!(
                        r#"(1 - (embeddings.embedding <=> (SELECT embedding FROM "{key}_embedding")::vector)) * {boost} AS score"#
                    )))
                    .and_where(Expr::cust(format!(r#"NOT documents.id = ANY("{cte_name}".previous_document_ids)"#)))
                    .join(
                        JoinType::Join,
                        SIden::String(cte_name.clone()),
                        Expr::cust("1 = 1"),
                    )
                    .join_as(
                        JoinType::InnerJoin,
                        chunks_table.to_table_tuple(),
                        Alias::new("chunks"),
                        Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                            .equals((SIden::Str("embeddings"), SIden::Str("chunk_id"))),
                    )
                    .join_as(
                        JoinType::InnerJoin,
                        documents_table.to_table_tuple(),
                        Alias::new("documents"),
                        Expr::col((SIden::Str("documents"), SIden::Str("id")))
                            .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
                    )
                    .order_by_expr(Expr::cust(format!(
                        r#"embeddings.embedding <=> (SELECT embedding FROM "{key}_embedding")::vector"#
                    )), Order::Asc )
                    .limit(1);
            }
            ModelRuntime::OpenAI => {
                // We can unwrap here as we know this is all set from above
                let model = &pipeline
                    .parsed_schema
                    .as_ref()
                    .unwrap()
                    .get(&key)
                    .unwrap()
                    .semantic_search
                    .as_ref()
                    .unwrap()
                    .model;

                // Get the remote embedding
                let embedding = {
                    let remote_embeddings = build_remote_embeddings(
                        model.runtime,
                        &model.name,
                        vsa.parameters.as_ref(),
                    )?;
                    let mut embeddings = remote_embeddings.embed(vec![vsa.query]).await?;
                    std::mem::take(&mut embeddings[0])
                };

                score_cte_non_recursive
                    .from_as(embeddings_table.to_table_tuple(), Alias::new("embeddings"))
                    .column((SIden::Str("documents"), SIden::Str("id")))
                    .expr(Expr::cust("ARRAY[documents.id] as previous_document_ids"))
                    .expr(Expr::cust_with_values(
                        format!("(1 - (embeddings.embedding <=> $1::vector)) * {boost} AS score"),
                        [embedding.clone()],
                    ))
                    .join_as(
                        JoinType::InnerJoin,
                        chunks_table.to_table_tuple(),
                        Alias::new("chunks"),
                        Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                            .equals((SIden::Str("embeddings"), SIden::Str("chunk_id"))),
                    )
                    .join_as(
                        JoinType::InnerJoin,
                        documents_table.to_table_tuple(),
                        Alias::new("documents"),
                        Expr::col((SIden::Str("documents"), SIden::Str("id")))
                            .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
                    )
                    .order_by_expr(
                        Expr::cust_with_values(
                            "embeddings.embedding <=> $1::vector",
                            [embedding.clone()],
                        ),
                        Order::Asc,
                    )
                    .limit(1);

                score_cte_recurisive
                    .from_as(embeddings_table.to_table_tuple(), Alias::new("embeddings"))
                    .join(
                        JoinType::Join,
                        SIden::String(cte_name.clone()),
                        Expr::cust("1 = 1"),
                    )
                    .column((SIden::Str("documents"), SIden::Str("id")))
                    .expr(Expr::cust(format!(
                        r#""{cte_name}".previous_document_ids || documents.id"#
                    )))
                    .expr(Expr::cust_with_values(
                        format!("(1 - (embeddings.embedding <=> $1::vector)) * {boost} AS score"),
                        [embedding.clone()],
                    ))
                    .and_where(Expr::cust(format!(
                        r#"NOT documents.id = ANY("{cte_name}".previous_document_ids)"#
                    )))
                    .join_as(
                        JoinType::InnerJoin,
                        chunks_table.to_table_tuple(),
                        Alias::new("chunks"),
                        Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                            .equals((SIden::Str("embeddings"), SIden::Str("chunk_id"))),
                    )
                    .join_as(
                        JoinType::InnerJoin,
                        documents_table.to_table_tuple(),
                        Alias::new("documents"),
                        Expr::col((SIden::Str("documents"), SIden::Str("id")))
                            .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
                    )
                    .order_by_expr(
                        Expr::cust_with_values(
                            "embeddings.embedding <=> $1::vector",
                            [embedding.clone()],
                        ),
                        Order::Asc,
                    )
                    .limit(1);
            }
        }

        if let Some(filter) = &valid_query.query.filter {
            let filter = FilterBuilder::new(filter.clone().0, "documents", "document").build()?;
            score_cte_non_recursive.cond_where(filter.clone());
            score_cte_recurisive.cond_where(filter);
        }

        let score_cte = Query::select()
            .expr(Expr::cust("*"))
            .from_subquery(score_cte_non_recursive, Alias::new("non_recursive"))
            .union(sea_query::UnionType::All, score_cte_recurisive)
            .to_owned();

        let mut score_cte = CommonTableExpression::from_select(score_cte);
        score_cte.table_name(Alias::new(&cte_name));
        with_clause.cte(score_cte);

        // Add to the sum expression
        sum_expression = if let Some(expr) = sum_expression {
            Some(expr.add(Expr::cust(format!(r#"COALESCE("{cte_name}".score, 0.0)"#))))
        } else {
            Some(Expr::cust(format!(r#"COALESCE("{cte_name}".score, 0.0)"#)))
        };
        score_table_names.push(cte_name);
    }

    for (key, vma) in valid_query.query.full_text_search.unwrap_or_default() {
        let full_text_table = format!("{}_{}.{}_tsvectors", collection.name, pipeline.name, key);
        let chunks_table = format!("{}_{}.{}_chunks", collection.name, pipeline.name, key);
        let boost = vma.boost.unwrap_or(1.0);

        // Build the score CTE
        let cte_name = format!("{key}_tsvectors_score");

        let mut score_cte_non_recursive = Query::select()
            .column((SIden::Str("documents"), SIden::Str("id")))
            .expr_as(
                Expr::cust_with_values(
                    format!(
                        r#"ts_rank(tsvectors.ts, plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1), 32) * {boost}"#,
                    ),
                    [&vma.query],
                ),
                Alias::new("score")
            )
            .expr(Expr::cust(
                "ARRAY[documents.id] as previous_document_ids",
            ))
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
            .join_as(
                JoinType::InnerJoin,
                chunks_table.to_table_tuple(),
                Alias::new("chunks"),
                Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                    .equals((SIden::Str("tsvectors"), SIden::Str("chunk_id"))),
            )
            .join_as(
                JoinType::InnerJoin,
                documents_table.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
            )
            .order_by(SIden::Str("score"), Order::Desc)
            .limit(1).
            to_owned();

        let mut score_cte_recursive = Query::select()
            .column((SIden::Str("documents"), SIden::Str("id")))
            .expr_as(
                Expr::cust_with_values(
                    format!(
                        r#"ts_rank(tsvectors.ts, plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1), 32) * {boost}"#,
                    ),
                    [&vma.query],
                ),
                Alias::new("score")
            )
            .expr(Expr::cust(format!(
                r#""{cte_name}".previous_document_ids || documents.id"#
            )))
            .from_as(
                full_text_table.to_table_tuple(),
                Alias::new("tsvectors"),
            )
            .join(
                JoinType::Join,
                SIden::String(cte_name.clone()),
                Expr::cust("1 = 1"),
            )
            .and_where(Expr::cust(format!(
                r#"NOT documents.id = ANY("{cte_name}".previous_document_ids)"#
            )))
            .and_where(Expr::cust_with_values(
                format!(
                    r#"tsvectors.ts @@ plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{{key},full_text_search,configuration}}' FROM pipeline)), $1)"#,
                ),
                [&vma.query],
            ))
            .join_as(
                JoinType::InnerJoin,
                chunks_table.to_table_tuple(),
                Alias::new("chunks"),
                Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                    .equals((SIden::Str("tsvectors"), SIden::Str("chunk_id"))),
            )
            .join_as(
                JoinType::InnerJoin,
                documents_table.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
            )
            .order_by(SIden::Str("score"), Order::Desc)
            .limit(1)
            .to_owned();

        if let Some(filter) = &valid_query.query.filter {
            let filter = FilterBuilder::new(filter.clone().0, "documents", "document").build()?;
            score_cte_recursive.cond_where(filter.clone());
            score_cte_non_recursive.cond_where(filter);
        }

        let score_cte = Query::select()
            .expr(Expr::cust("*"))
            .from_subquery(score_cte_non_recursive, Alias::new("non_recursive"))
            .union(sea_query::UnionType::All, score_cte_recursive)
            .to_owned();

        let mut score_cte = CommonTableExpression::from_select(score_cte);
        score_cte.table_name(Alias::new(&cte_name));
        with_clause.cte(score_cte);

        // Add to the sum expression
        sum_expression = if let Some(expr) = sum_expression {
            Some(expr.add(Expr::cust(format!(r#"COALESCE("{cte_name}".score, 0.0)"#))))
        } else {
            Some(Expr::cust(format!(r#"COALESCE("{cte_name}".score, 0.0)"#)))
        };
        score_table_names.push(cte_name);
    }

    let query = if let Some(select_from) = score_table_names.first() {
        let score_table_names_e: Vec<SimpleExpr> = score_table_names
            .clone()
            .into_iter()
            .map(|t| Expr::col((SIden::String(t), SIden::Str("id"))).into())
            .collect();
        let mut main_query = Query::select();
        for i in 1..score_table_names_e.len() {
            main_query.full_outer_join(
                SIden::String(score_table_names[i].to_string()),
                Expr::col((
                    SIden::String(score_table_names[i].to_string()),
                    SIden::Str("id"),
                ))
                .eq(Func::coalesce(score_table_names_e[0..i].to_vec())),
            );
        }
        let id_select_expression = Func::coalesce(score_table_names_e);

        let sum_expression = sum_expression
            .context("query requires some scoring through full_text_search or semantic_search")?;
        main_query
            .expr_as(Expr::expr(id_select_expression.clone()), Alias::new("id"))
            .expr_as(sum_expression, Alias::new("score"))
            .column(SIden::Str("document"))
            .from(SIden::String(select_from.to_string()))
            .join_as(
                JoinType::InnerJoin,
                documents_table.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id"))).eq(id_select_expression),
            )
            .order_by(SIden::Str("score"), Order::Desc)
            .limit(limit);

        let mut main_query = CommonTableExpression::from_select(main_query);
        main_query.table_name(Alias::new("main"));
        with_clause.cte(main_query);

        // Insert into searches table
        let searches_table = format!("{}_{}.searches", collection.name, pipeline.name);
        let searches_insert_query = Query::insert()
            .into_table(searches_table.to_table_tuple())
            .columns([SIden::Str("query")])
            .values([query.0.into()])?
            .returning_col(SIden::Str("id"))
            .to_owned();
        let mut searches_insert_query = CommonTableExpression::new()
            .query(searches_insert_query)
            .to_owned();
        searches_insert_query.table_name(Alias::new("searches_insert"));
        with_clause.cte(searches_insert_query);

        // Insert into search_results table
        let search_results_table = format!("{}_{}.search_results", collection.name, pipeline.name);
        let jsonb_builder = score_table_names.iter().fold(String::new(), |acc, t| {
            format!("{acc}, '{t}', (SELECT score FROM {t} WHERE {t}.id = main.id)")
        });
        let jsonb_builder = format!("JSONB_BUILD_OBJECT('total', score{jsonb_builder})");
        let search_results_insert_query = Query::insert()
            .into_table(search_results_table.to_table_tuple())
            .columns([
                SIden::Str("search_id"),
                SIden::Str("document_id"),
                SIden::Str("scores"),
                SIden::Str("rank"),
            ])
            .select_from(
                Query::select()
                    .expr(Expr::cust("(SELECT id FROM searches_insert)"))
                    .column(SIden::Str("id"))
                    .expr(Expr::cust(jsonb_builder))
                    .expr(Expr::cust("row_number() over()"))
                    .from(SIden::Str("main"))
                    .to_owned(),
            )?
            .to_owned();
        let mut search_results_insert_query = CommonTableExpression::new()
            .query(search_results_insert_query)
            .to_owned();
        search_results_insert_query.table_name(Alias::new("search_results_insert"));
        with_clause.cte(search_results_insert_query);

        Query::select()
            .expr(Expr::cust(
                "JSONB_BUILD_OBJECT('search_id', (SELECT id FROM searches_insert), 'results', JSON_AGG(main.*))",
            ))
            .from(SIden::Str("main"))
            .to_owned()
    } else {
        // TODO: Maybe let users filter documents only here?
        anyhow::bail!("If you are only looking to filter documents checkout the `get_documents` method on the Collection")
    };

    // For whatever reason, sea query does not like multiple ctes if the cte is recursive
    let (sql, values) = query.with(with_clause).build_sqlx(PostgresQueryBuilder);
    let sql = sql.replace("WITH ", "WITH RECURSIVE ");
    debug_sea_query!(DOCUMENT_SEARCH, sql, values);
    Ok((sql, values))
}
