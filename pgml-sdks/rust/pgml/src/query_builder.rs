use itertools::Itertools;
use pgml_macros::{custom_derive, custom_methods};
use sea_query::{
    all, extension::postgres::PgExpr, query::SelectStatement, Alias, CommonTableExpression, Expr,
    Func, Iden, JoinType, Order, PostgresQueryBuilder, Query, QueryStatementWriter, WithClause,
};
use sea_query_binder::SqlxBinder;

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

#[custom_methods(limit, filter, vector_recall, run)]
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

    pub fn filter(mut self, filter: Json) -> Self {
        self.query.and_where(
            Expr::col((SIden::Str("documents"), SIden::Str("metadata"))).contains(filter.0),
        );
        self
    }

    pub async fn vector_recall(
        mut self,
        query: String,
        query_params: Option<Json>,
        model_id: Option<i64>,
        splitter_id: Option<i64>,
    ) -> anyhow::Result<Self> {
        let query_params = match query_params {
            Some(params) => params.0,
            None => serde_json::json!({}),
        };
        let model_id = model_id.unwrap_or(1);
        let splitter_id = splitter_id.unwrap_or(1);

        let embeddings_table_name = self
            .collection
            .get_embeddings_table_name(model_id, splitter_id)?;

        let model_name = self
            .collection
            .get_model_name(model_id)
            .await?
            .unwrap_or_else(|| panic!("Model with id: {} does not exist", model_id));

        let mut query_cte = Query::select();
        query_cte.expr_as(
            Func::cust(SIden::Str("pgml.embed")).args([
                Expr::cust_with_values("transformer => ($1)", [model_name]),
                Expr::cust_with_values("text => $1", [query]),
                Expr::cust_with_values("kwargs => $1", [query_params]),
            ]),
            Alias::new("query_embedding"),
        );
        let mut query_cte = CommonTableExpression::from_select(query_cte);
        query_cte.table_name(Alias::new("query_cte"));

        let mut cte = Query::select();
        cte.from_as(embeddings_table_name.to_table_tuple(), SIden::Str("embedding"))
            .inner_join(Alias::new("query_cte"), all![]) // NOTE: This is a hack to make the query work - sea_query does not support postgres cross join correctly
            .columns([models::EmbeddingIden::ChunkId])
            .expr(Expr::cust(
                "1 - (embedding.embedding <=> query_cte.query_embedding :: float8[] :: vector) as score",
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

        Ok(self)
    }

    pub async fn run(self) -> anyhow::Result<Vec<(f64, String, Json)>> {
        let (sql, values) = self.query.with(self.with).build_sqlx(PostgresQueryBuilder);
        let results: Vec<(f64, String, Json)> = sqlx::query_as_with(&sql, values)
            .fetch_all(&self.collection.pool)
            .await?;
        Ok(results)
    }

    pub fn debug(&self) {
        let query = self.query.clone().with(self.with.clone());
        println!("{}", query.to_string(PostgresQueryBuilder));
    }
}
