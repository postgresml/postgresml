use itertools::Itertools;
use pgml_macros::{custom_derive, custom_methods};
use sea_query::{
    query::SelectStatement, Alias, CommonTableExpression, Expr, Func, Iden, JoinType, Order,
    PostgresQueryBuilder, Query, QueryStatementWriter, WithClause,
};
use sea_query_binder::SqlxBinder;

use crate::{filter_builder, model::Model, models, splitter::Splitter, types::Json, Collection};

#[cfg(feature = "javascript")]
use crate::{languages::javascript::*, model::ModelJavascript, splitter::SplitterJavascript};

#[cfg(feature = "python")]
use crate::{model::ModelPython, splitter::SplitterPython};

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

trait IntoTableNameAndSchema {
    fn to_table_tuple<'b>(&self) -> (SIden<'b>, SIden<'b>);
}

impl IntoTableNameAndSchema for String {
    fn to_table_tuple<'b>(&self) -> (SIden<'b>, SIden<'b>) {
        self.split('.')
            .map(|s| SIden::String(s.to_string()))
            .collect_tuple()
            .expect("Malformed table name in IntoTableNameAndSchema")
    }
}

#[derive(custom_derive, Clone, Debug)]
pub struct QueryBuilder {
    query: SelectStatement,
    with: WithClause,
    collection: Collection,
}

#[custom_methods(limit, filter, vector_recall, to_full_string, run)]
impl QueryBuilder {
    pub fn new(collection: Collection) -> Self {
        Self {
            query: SelectStatement::new(),
            with: WithClause::new(),
            collection,
        }
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.query.limit(limit);
        self
    }

    pub fn filter(mut self, mut filter: Json) -> Self {
        let filter = filter
            .0
            .as_object_mut()
            .expect("Filter must be a Json object");
        if let Some(f) = filter.remove("metadata") {
            self = self.filter_metadata(f);
        }
        if let Some(f) = filter.remove("full_text") {
            self = self.filter_full_text(f);
        }
        self
    }

    fn filter_metadata(mut self, filter: serde_json::Value) -> Self {
        let filter = filter_builder::FilterBuilder::new(filter, "documents", "metadata").build();
        self.query.cond_where(filter);
        self
    }

    fn filter_full_text(mut self, mut filter: serde_json::Value) -> Self {
        let filter = filter
            .as_object_mut()
            .expect("Full text filter must be a Json object");
        let configuration = match filter.get("configuration") {
            Some(config) => config.as_str().expect("Configuration must be a string"),
            None => "english",
        };
        let filter_text = filter
            .get("text")
            .expect("Filter must contain a text field")
            .as_str()
            .expect("Text must be a string");
        self.query
            .join_as(
                JoinType::InnerJoin,
                self.collection
                    .documents_tsvectors_table_name
                    .to_table_tuple(),
                Alias::new("documents_tsvectors"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .equals((SIden::Str("documents_tsvectors"), SIden::Str("document_id"))),
            )
            .and_where(
                Expr::col((
                    SIden::Str("documents_tsvectors"),
                    SIden::Str("configuration"),
                ))
                .eq(configuration),
            )
            .and_where(Expr::cust_with_values(
                &format!(
                    "documents_tsvectors.ts @@ plainto_tsquery('{}', $1)",
                    configuration
                ),
                [filter_text],
            ));
        self
    }

    pub fn vector_recall(
        mut self,
        query: String,
        model: &Model,
        splitter: &Splitter,
        query_params: Option<Json>,
    ) -> Self {
        let query_params = match query_params {
            Some(params) => params.0,
            None => serde_json::json!({}),
        };

        let embeddings_table_name = self
            .collection
            .get_embeddings_table_name(model.id, splitter.id)
            .expect("Error getting embeddings table name in vector_recall");

        let mut query_cte = Query::select();
        query_cte
            .expr_as(
                Func::cast_as(
                    Func::cust(SIden::Str("pgml.embed")).args([
                        Expr::cust("transformer => models.name"),
                        Expr::cust_with_values("text => $1", [query]),
                        Expr::cust_with_values("kwargs => $1", [query_params]),
                    ]),
                    Alias::new("vector"),
                ),
                Alias::new("query_embedding"),
            )
            .from_as(
                (SIden::Str("pgml"), SIden::Str("models")),
                SIden::Str("models"),
            )
            .and_where(Expr::col((SIden::Str("models"), SIden::Str("id"))).eq(model.id));
        let mut query_cte = CommonTableExpression::from_select(query_cte);
        query_cte.table_name(Alias::new("query_cte"));

        let mut cte = Query::select();
        cte.from_as(
            embeddings_table_name.to_table_tuple(),
            SIden::Str("embedding"),
        )
        .columns([models::EmbeddingIden::ChunkId])
        .expr(Expr::cust(
            "1 - (embedding.embedding <=> (select query_embedding from query_cte)) as score",
        ));
        let mut cte = CommonTableExpression::from_select(cte);
        cte.table_name(Alias::new("cte"));

        let mut with_clause = WithClause::new();
        self.with = with_clause.cte(query_cte).cte(cte).to_owned();

        self.query
            .columns([
                (SIden::Str("cte"), SIden::Str("score")),
                (SIden::Str("chunks"), SIden::Str("chunk")),
                (SIden::Str("documents"), SIden::Str("metadata")),
            ])
            .from(SIden::Str("cte"))
            .join_as(
                JoinType::InnerJoin,
                self.collection.chunks_table_name.to_table_tuple(),
                Alias::new("chunks"),
                Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                    .equals((SIden::Str("cte"), SIden::Str("chunk_id"))),
            )
            .join_as(
                JoinType::InnerJoin,
                self.collection.documents_table_name.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
            )
            .order_by((SIden::Str("cte"), SIden::Str("score")), Order::Desc);

        self
    }

    pub async fn run(self) -> anyhow::Result<Vec<(f64, String, Json)>> {
        let (sql, values) = self.query.with(self.with).build_sqlx(PostgresQueryBuilder);
        let results: Vec<(f64, String, Json)> = sqlx::query_as_with(&sql, values)
            .fetch_all(&self.collection.pool)
            .await?;
        Ok(results)
    }

    // This is mostly so our SDKs in other languages have some way to debug
    pub fn to_full_string(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for QueryBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let query = self.query.clone().with(self.with.clone());
        write!(f, "{}", query.to_string(PostgresQueryBuilder))
    }
}
