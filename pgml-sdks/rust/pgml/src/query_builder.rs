use itertools::Itertools;
use pgml_macros::{custom_derive, custom_methods};
use sea_query::{
    extension::postgres::PgExpr, query::SelectStatement, Alias, CommonTableExpression, Expr, Func,
    Iden, Query, QueryStatementWriter, WithClause, all
};

use crate::{models, types::Json, Collection};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

#[derive(Clone)]
enum SIden<'a> {
    Str(&'a str),
    String(String),
}

impl Iden for SIden<'_> {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                SIden::Str(s) => s,
                SIden::String(s) => s.as_str(),
            }
        )
        .unwrap();
    }
}

#[derive(custom_derive, Clone, Debug)]
pub struct QueryBuilder {
    query: SelectStatement,
    collection: Collection,
}

#[custom_methods(limit)]
impl QueryBuilder {
    pub fn new(collection: Collection) -> Self {
        Self {
            query: SelectStatement::new(),
            collection,
        }
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.query.limit(limit);
        self
    }

    pub fn filter(mut self, filter: Json) -> Self {
        let documents_table_name = self.collection.documents_table_name.clone();
        let documents_table_parts: (SIden, SIden) = documents_table_name
            .split(".")
            .map(|s| SIden::String(s.to_string()))
            .collect_tuple()
            .expect("Malformed documents table name in vector_recall");
        self.query.from(documents_table_parts);
        self.query
            .and_where(Expr::col(models::DocumentIden::Metadata).contains(filter.0.to_string()));
        self
    }

    pub async fn vector_recall(
        mut self,
        query: String,
        query_params: Option<Json>,
        top_k: Option<i64>,
        model_id: Option<i64>,
        splitter_id: Option<i64>,
    ) -> anyhow::Result<Self> {
        let query_params = match query_params {
            Some(params) => params.0,
            None => serde_json::json!({}),
        };
        let top_k = top_k.unwrap_or(5);
        let model_id = model_id.unwrap_or(1);
        let splitter_id = splitter_id.unwrap_or(1);

        let embeddings_table_name = self
            .collection
            .get_embeddings_table_name(model_id, splitter_id)
            .await?;
        let embeddings_table_name = embeddings_table_name.expect(&format!(
            "Embeddings table does not exist for task: embedding model_id: {} and splitter_id: {}",
            model_id, splitter_id
        ));
        let embeddings_table_parts: (SIden, SIden) = embeddings_table_name
            .split(".")
            .map(|s| SIden::String(s.to_string()))
            .collect_tuple()
            .expect("Malformed embeddings table name in vector_recall");

        let model_name = self.collection.get_model_name(model_id).await?;
        let model_name = model_name.expect(&format!("Model with id: {} does not exist", model_id));

        let mut query_cte = Query::select();
        query_cte
            .from(embeddings_table_parts.clone())
            .expr(Func::cust(SIden::Str("Embed")).arg(Expr::cust_with_values(
                "transformer=$1, text=$2, parameters=$3",
                [model_name, query, query_params.to_string()],
            )));
        let mut query_cte = CommonTableExpression::from_select(query_cte);
        query_cte.table_name(Alias::new("query_cte"));

        let mut cte = Query::select();
        cte.from_as(embeddings_table_parts, SIden::Str("embedding"))
            .cross_join(Alias::new("query_cte"), all![])
            .columns([models::EmbeddingIden::ChunkId])
            .expr(Func::cust(SIden::Str("1 - CosineDistance")).arg(Expr::cust("embedding.embedding, Cast(query_cte.embedding, \"vector\")")));
        let mut cte = CommonTableExpression::from_select(cte);
        cte.table_name(Alias::new("cte"));


        let mut with_clause = WithClause::new();
        let with_clause = with_clause.cte(query_cte).cte(cte).to_owned();

        let mut query = Query::select();
        query.columns([(SIden::Str("cte"), SIden::Str("score"))]);
        let query = query.with(with_clause.to_owned());
        // .from_as(embeddings_table_parts, SIden::Str("embedding"))
        // .columns([models::EmbeddingIden::ChunkId])
        // .expr(Func::cust(SIden::Str("CosineDistance")).arg(Expr::cust("embedding.embedding")))
        // .order_by(models::EmbeddingIden::ChunkId, false)
        // .limit(top_k);

        println!(
            "query: {}",
            query.to_string(sea_query::PostgresQueryBuilder)
        );

        // let mut cte = Query::select();
        // cte.from_as(embeddings_table_parts, SIden::Str("embedding"))
        //     .columns([models::EmbeddingIden::ChunkId])
        //     .expr(Func::cust(SIden::Str("CosineDistance")).arg(Expr::cust("embedding.embedding")));
        // println!("cte: {}", cte.to_string(sea_query::PostgresQueryBuilder));
        // let common_table_expression = CommonTableExpression::new().query(query)

        Ok(self)
    }
}
