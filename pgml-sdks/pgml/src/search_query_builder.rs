use serde::Deserialize;
use std::collections::HashMap;

use crate::{
    collection::Collection, multi_field_pipeline::MultiFieldPipeline, query_builder, types::Json,
};

#[derive(Debug, Deserialize)]
struct ValidSemanticSearchAction {
    query: String,
    boost: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ValidMatchAction {
    query: String,
    boost: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ValidQueryAction {
    // exists: Option<
    #[serde(alias = "match")]
    mtch: Option<HashMap<String, ValidMatchAction>>,
    semantic_search: Option<HashMap<String, ValidSemanticSearchAction>>,
}

#[derive(Debug, Deserialize)]
struct ValidQuery {
    query: ValidQueryAction,
}

pub fn build_search_query(
    collection: &Collection,
    query: Json,
    pipeline: &MultiFieldPipeline,
) -> anyhow::Result<String> {
    let valid_query: ValidQuery = serde_json::from_value(query.0)?;

    let pipeline_table = format!("{}.pipelines", collection.name);
    let documents_table = format!("{}.documents", collection.name);

    let mut ctes = "".to_string();
    let mut sums = vec![];
    let mut inner_joins = "".to_string();
    let mut wheres = vec![];

    for (key, vsa) in valid_query.query.semantic_search.unwrap_or_default() {
        ctes.push_str(&format!(
            r#" ,
                "{key}_embedding" AS (
                    SELECT
                        pgml.embed(
                            transformer => (SELECT schema #>> '{{{key},embed,model}}' FROM pipeline),
                            text => '{}',
                            kwargs => COALESCE((SELECT schema #> '{{{key},embed,model_parameters}}' FROM pipeline), '{{}}'::jsonb)
                        )::vector as embedding
                )
            "#,
            vsa.query
        ));

        sums.push(format!(
            r#"("{key}_embeddings".embedding <=> (SELECT embedding FROM "{key}_embedding"))"#
        ));

        let chunks_table = format!("{}_{}.{}_chunks", collection.name, pipeline.name, key);
        let embeddings_table = format!("{}_{}.{}_embeddings", collection.name, pipeline.name, key);
        // Use the query_builder! here to handle the formatting of table names correctly
        inner_joins.push_str(&query_builder!(
            r#"
                INNER JOIN %s "%d_chunks" on "%d_chunks".document_id = documents.id
                INNER JOIN %s "%d_embeddings" on "%d_embeddings".chunk_id = "%d_chunks".id
            "#,
            chunks_table,
            key,
            key,
            embeddings_table,
            key,
            key,
            key
        ));
    }

    for (key, vma) in valid_query.query.mtch.unwrap_or_default() {
        let full_text_table = format!("{}_{}.{}_tsvectors", collection.name, pipeline.name, key);
        inner_joins.push_str(&query_builder!(
            "\nINNER JOIN %s on %s.document_id = documents.id\n",
            full_text_table,
            full_text_table
        ));
        sums.push(query_builder!(
            "(ts_rank(%s.ts, plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{%d,full_text_search,configuration}}' FROM pipeline)), '%d'), 32))", 
            full_text_table, 
            key,
            vma.query 
        ));
        wheres.push(query_builder!(
            "%s.ts @@ plainto_tsquery((SELECT oid FROM pg_ts_config WHERE cfgname = (SELECT schema #>> '{{%d,full_text_search,configuration}}' FROM pipeline)), '%d')", 
            full_text_table, 
            key,
            vma.query 
        ));
    }

    let wheres = if wheres.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", wheres.join(" AND "))
    };
    let sums = sums.join(" + ");
    Ok(query_builder!(
        r#"
        WITH pipeline AS (
            SELECT
              schema
            FROM
              %s
            WHERE
              name = 'test_r_p_cs_6'
        )
        %d
        SELECT json_agg(q)
        FROM (
            SELECT 
                documents.id,
                SUM(%d) as score
            FROM
                %s documents
                %d
            %d
            GROUP BY documents.id
            ORDER BY score ASC
            LIMIT 10
        ) q
    "#,
        pipeline_table,
        ctes,
        sums,
        documents_table,
        inner_joins,
        wheres
    ))
}
