use anyhow::Context;
use rust_bridge::{alias, alias_methods};
use sea_query::{
    query::SelectStatement, Alias, CommonTableExpression, Expr, Func, JoinType, Order,
    PostgresQueryBuilder, Query, QueryStatementWriter, WithClause,
};
use sea_query_binder::SqlxBinder;
use std::borrow::Cow;
use tracing::instrument;

use crate::{
    filter_builder, get_or_initialize_pool,
    model::ModelRuntime,
    models,
    pipeline::Pipeline,
    query_builder,
    remote_embeddings::build_remote_embeddings,
    types::{IntoTableNameAndSchema, Json, SIden, TryToNumeric},
    Collection,
};

#[cfg(feature = "python")]
use crate::{pipeline::PipelinePython, types::JsonPython};

#[derive(Clone, Debug)]
struct QueryBuilderState {}

#[derive(alias, Clone, Debug)]
pub struct QueryBuilder {
    query: SelectStatement,
    with: WithClause,
    collection: Collection,
    query_string: Option<String>,
    pipeline: Option<Pipeline>,
    query_parameters: Option<Json>,
}

#[alias_methods(limit, filter, vector_recall, to_full_string, fetch_all)]
impl QueryBuilder {
    pub fn new(collection: Collection) -> Self {
        Self {
            query: SelectStatement::new(),
            with: WithClause::new(),
            collection,
            query_string: None,
            pipeline: None,
            query_parameters: None,
        }
    }

    #[instrument(skip(self))]
    pub fn limit(mut self, limit: u64) -> Self {
        self.query.limit(limit);
        self
    }

    #[instrument(skip(self))]
    pub fn filter(mut self, mut filter: Json) -> Self {
        let filter = filter
            .0
            .as_object_mut()
            .expect("Filter must be a Json object");
        if let Some(f) = filter.remove("metadata") {
            self = self.filter_metadata(f);
        }
        if let Some(f) = filter.remove("full_text_search") {
            self = self.filter_full_text(f);
        }
        self
    }

    #[instrument(skip(self))]
    fn filter_metadata(mut self, filter: serde_json::Value) -> Self {
        let filter = filter_builder::FilterBuilder::new(filter, "documents", "metadata").build();
        self.query.cond_where(filter);
        self
    }

    #[instrument(skip(self))]
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
                format!(
                    "documents_tsvectors.ts @@ plainto_tsquery('{}', $1)",
                    configuration
                ),
                [filter_text],
            ));
        self
    }

    #[instrument(skip(self))]
    pub fn vector_recall(
        mut self,
        query: &str,
        pipeline: &Pipeline,
        query_parameters: Option<Json>,
    ) -> Self {
        // Save these in case of failure
        self.pipeline = Some(pipeline.clone());
        self.query_string = Some(query.to_owned());
        self.query_parameters = query_parameters.clone();

        let mut query_parameters = query_parameters.unwrap_or_default().0;
        // If they did set hnsw, remove it before we pass it to the model
        query_parameters
            .as_object_mut()
            .expect("Query parameters must be a Json object")
            .remove("hnsw");
        let embeddings_table_name =
            format!("{}.{}_embeddings", self.collection.name, pipeline.name);

        // Build the pipeline CTE
        let mut pipeline_cte = Query::select();
        pipeline_cte
            .from_as(
                self.collection.pipelines_table_name.to_table_tuple(),
                SIden::Str("pipeline"),
            )
            .columns([models::PipelineIden::ModelId])
            .and_where(Expr::col(models::PipelineIden::Name).eq(&pipeline.name));
        let mut pipeline_cte = CommonTableExpression::from_select(pipeline_cte);
        pipeline_cte.table_name(Alias::new("pipeline"));

        // Build the model CTE
        let mut model_cte = Query::select();
        model_cte
            .from_as(
                (SIden::Str("pgml"), SIden::Str("models")),
                SIden::Str("model"),
            )
            .columns([models::ModelIden::Hyperparams])
            .and_where(Expr::cust("id = (SELECT model_id FROM pipeline)"));
        let mut model_cte = CommonTableExpression::from_select(model_cte);
        model_cte.table_name(Alias::new("model"));

        // Build the embedding CTE
        let mut embedding_cte = Query::select();
        embedding_cte.expr_as(
            Func::cast_as(
                Func::cust(SIden::Str("pgml.embed")).args([
                    Expr::cust("transformer => (SELECT hyperparams->>'name' FROM model)"),
                    Expr::cust_with_values("text => $1", [query]),
                    Expr::cust_with_values("kwargs => $1", [query_parameters]),
                ]),
                Alias::new("vector"),
            ),
            Alias::new("embedding"),
        );
        let mut embedding_cte = CommonTableExpression::from_select(embedding_cte);
        embedding_cte.table_name(Alias::new("embedding"));

        // Build the where clause
        let mut with_clause = WithClause::new();
        self.with = with_clause
            .cte(pipeline_cte)
            .cte(model_cte)
            .cte(embedding_cte)
            .to_owned();

        // Build the query
        self.query
            .expr(Expr::cust(
                "(embeddings.embedding <=> (SELECT embedding from embedding)) score",
            ))
            .columns([
                (SIden::Str("chunks"), SIden::Str("chunk")),
                (SIden::Str("documents"), SIden::Str("metadata")),
            ])
            .from_as(
                embeddings_table_name.to_table_tuple(),
                SIden::Str("embeddings"),
            )
            .join_as(
                JoinType::InnerJoin,
                self.collection.chunks_table_name.to_table_tuple(),
                Alias::new("chunks"),
                Expr::col((SIden::Str("chunks"), SIden::Str("id")))
                    .equals((SIden::Str("embeddings"), SIden::Str("chunk_id"))),
            )
            .join_as(
                JoinType::InnerJoin,
                self.collection.documents_table_name.to_table_tuple(),
                Alias::new("documents"),
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .equals((SIden::Str("chunks"), SIden::Str("document_id"))),
            )
            .order_by(SIden::Str("score"), Order::Asc);

        self
    }

    #[instrument(skip(self))]
    pub async fn fetch_all(mut self) -> anyhow::Result<Vec<(f64, String, Json)>> {
        let pool = get_or_initialize_pool(&self.collection.database_url).await?;

        let mut query_parameters = self.query_parameters.unwrap_or_default();

        let (sql, values) = self
            .query
            .clone()
            .with(self.with.clone())
            .build_sqlx(PostgresQueryBuilder);

        let result: Result<Vec<(f64, String, Json)>, _> =
            if !query_parameters["hnsw"]["ef_search"].is_null() {
                let mut transaction = pool.begin().await?;
                let ef_search = query_parameters["hnsw"]["ef_search"]
                    .try_to_i64()
                    .context("ef_search must be an integer")?;
                sqlx::query(&query_builder!("SET LOCAL hnsw.ef_search = %d", ef_search))
                    .execute(&mut *transaction)
                    .await?;
                let results = sqlx::query_as_with(&sql, values)
                    .fetch_all(&mut *transaction)
                    .await;
                transaction.commit().await?;
                results
            } else {
                sqlx::query_as_with(&sql, values).fetch_all(&pool).await
            };

        match result {
            Ok(r) => Ok(r),
            Err(e) => match e.as_database_error() {
                Some(d) => {
                    if d.code() == Some(Cow::from("XX000")) {
                        // Explicitly get and set the model
                        let project_info = self.collection.get_project_info().await?;
                        let pipeline = self
                            .pipeline
                            .as_mut()
                            .context("Need pipeline to call fetch_all on query builder with remote embeddings")?;
                        pipeline.set_project_info(project_info);
                        pipeline.verify_in_database(false).await?;
                        let model = pipeline
                            .model
                            .as_ref()
                            .context("Pipeline must be verified to perform vector search with remote embeddings")?;

                        // If the model runtime is python, the error was not caused by an unsupported runtime
                        if model.runtime == ModelRuntime::Python {
                            return Err(anyhow::anyhow!(e));
                        }

                        let hnsw_parameters = query_parameters
                            .as_object_mut()
                            .context("Query parameters must be a Json object")?
                            .remove("hnsw");

                        let remote_embeddings =
                            build_remote_embeddings(model.runtime, &model.name, &query_parameters)?;
                        let mut embeddings = remote_embeddings
                            .embed(vec![self
                                .query_string
                                .to_owned()
                                .context("Must have query_string to call fetch_all on query_builder with remote embeddings")?])
                            .await?;
                        let embedding = std::mem::take(&mut embeddings[0]);

                        let mut embedding_cte = Query::select();
                        embedding_cte
                            .expr(Expr::cust_with_values("$1::vector embedding", [embedding]));

                        let mut embedding_cte = CommonTableExpression::from_select(embedding_cte);
                        embedding_cte.table_name(Alias::new("embedding"));
                        let mut with_clause = WithClause::new();
                        with_clause.cte(embedding_cte);

                        let (sql, values) = self
                            .query
                            .clone()
                            .with(with_clause)
                            .build_sqlx(PostgresQueryBuilder);

                        if let Some(parameters) = hnsw_parameters {
                            let mut transaction = pool.begin().await?;
                            let ef_search = parameters["ef_search"]
                                .try_to_i64()
                                .context("ef_search must be an integer")?;
                            sqlx::query(&query_builder!(
                                "SET LOCAL hnsw.ef_search = %d",
                                ef_search
                            ))
                            .execute(&mut *transaction)
                            .await?;
                            let results = sqlx::query_as_with(&sql, values)
                                .fetch_all(&mut *transaction)
                                .await;
                            transaction.commit().await?;
                            results
                        } else {
                            sqlx::query_as_with(&sql, values).fetch_all(&pool).await
                        }
                        .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!(e))
                    }
                }
                None => Err(anyhow::anyhow!(e)),
            },
        }.map(|r| r.into_iter().map(|(score, id, metadata)| (1. - score, id, metadata)).collect())
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
