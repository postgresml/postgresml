use anyhow::Context;
use indicatif::MultiProgress;
use itertools::Itertools;
use regex::Regex;
use rust_bridge::{alias, alias_methods};
use sea_query::{Alias, Expr, JoinType, NullOrdering, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Executor;
use sqlx::PgConnection;
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tracing::{instrument, warn};
use walkdir::WalkDir;

use crate::search_query_builder::build_search_query;
use crate::vector_search_query_builder::build_vector_search_query;
use crate::{
    filter_builder, get_or_initialize_pool,
    model::ModelRuntime,
    models,
    multi_field_pipeline::MultiFieldPipeline,
    order_by_builder,
    pipeline::Pipeline,
    queries, query_builder,
    query_builder::QueryBuilder,
    remote_embeddings::build_remote_embeddings,
    splitter::Splitter,
    types::{DateTime, IntoTableNameAndSchema, Json, SIden, TryToNumeric},
    utils,
};

#[cfg(feature = "python")]
use crate::{pipeline::PipelinePython, query_builder::QueryBuilderPython, types::JsonPython};

/// Our project tasks
#[derive(Debug, Clone)]
pub enum ProjectTask {
    Regression,
    Classification,
    QuestionAnswering,
    Summarization,
    Translation,
    TextClassification,
    TextGeneration,
    Text2text,
    Embedding,
}

impl From<&str> for ProjectTask {
    fn from(s: &str) -> Self {
        match s {
            "regression" => Self::Regression,
            "classification" => Self::Classification,
            "question_answering" => Self::QuestionAnswering,
            "summarization" => Self::Summarization,
            "translation" => Self::Translation,
            "text_classification" => Self::TextClassification,
            "text_generation" => Self::TextGeneration,
            "text2text" => Self::Text2text,
            "embedding" => Self::Embedding,
            _ => panic!("Unknown project task: {}", s),
        }
    }
}

impl From<&ProjectTask> for &'static str {
    fn from(m: &ProjectTask) -> Self {
        match m {
            ProjectTask::Regression => "regression",
            ProjectTask::Classification => "classification",
            ProjectTask::QuestionAnswering => "question_answering",
            ProjectTask::Summarization => "summarization",
            ProjectTask::Translation => "translation",
            ProjectTask::TextClassification => "text_classification",
            ProjectTask::TextGeneration => "text_generation",
            ProjectTask::Text2text => "text2text",
            ProjectTask::Embedding => "embedding",
        }
    }
}

// Contains information a Collection, Model, or Splitter needs to know about the project
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ProjectInfo {
    pub id: i64,
    pub name: String,
    pub task: ProjectTask,
    pub database_url: Option<String>,
}

// Data that is stored in the database about a collection
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct CollectionDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
    pub project_info: ProjectInfo,
}

/// A collection of documents
#[derive(alias, Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub database_url: Option<String>,
    pub pipelines_table_name: String,
    pub documents_table_name: String,
    pub chunks_table_name: String,
    pub documents_tsvectors_table_name: String,
    pub(crate) database_data: Option<CollectionDatabaseData>,
}

#[alias_methods(
    new,
    upsert_documents,
    get_documents,
    delete_documents,
    get_pipelines,
    get_pipeline,
    add_pipeline,
    remove_pipeline,
    enable_pipeline,
    disable_pipeline,
    vector_search,
    query,
    exists,
    archive,
    upsert_directory,
    upsert_file
)]
impl Collection {
    /// Creates a new [Collection]
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the collection.
    /// * `database_url` - An optional database_url. If passed, this url will be used instead of
    /// the `DATABASE_URL` environment variable.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    /// let collection = Collection::new("my_collection", None);
    /// ```
    pub fn new(name: &str, database_url: Option<String>) -> Self {
        let (
            pipelines_table_name,
            documents_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
        ) = Self::generate_table_names(name);
        Self {
            name: name.to_string(),
            database_url,
            pipelines_table_name,
            documents_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
            database_data: None,
        }
    }

    #[instrument(skip(self))]
    pub(crate) async fn verify_in_database(&mut self, throw_if_exists: bool) -> anyhow::Result<()> {
        if self.database_data.is_none() {
            let pool = get_or_initialize_pool(&self.database_url).await?;

            let result: Result<Option<models::Collection>, _> =
                sqlx::query_as("SELECT * FROM pgml.collections WHERE name = $1")
                    .bind(&self.name)
                    .fetch_optional(&pool)
                    .await;
            let collection: Option<models::Collection> = match result {
                Ok(s) => anyhow::Ok(s),
                Err(e) => match e.as_database_error() {
                    Some(db_e) => {
                        // Error 42P01 is "undefined_table"
                        if db_e.code() == Some(std::borrow::Cow::from("42P01")) {
                            sqlx::query(queries::CREATE_COLLECTIONS_TABLE)
                                .execute(&pool)
                                .await?;
                            Ok(None)
                        } else {
                            Err(e.into())
                        }
                    }
                    None => Err(e.into()),
                },
            }?;

            self.database_data = if let Some(c) = collection {
                anyhow::ensure!(!throw_if_exists, "Collection {} already exists", self.name);
                Some(CollectionDatabaseData {
                    id: c.id,
                    created_at: c.created_at,
                    project_info: ProjectInfo {
                        id: c.project_id,
                        name: self.name.clone(),
                        task: "embedding".into(),
                        database_url: self.database_url.clone(),
                    },
                })
            } else {
                let mut transaction = pool.begin().await?;

                let project_id: i64 = sqlx::query_scalar("INSERT INTO pgml.projects (name, task) VALUES ($1, 'embedding'::pgml.task) ON CONFLICT (name) DO UPDATE SET task = EXCLUDED.task RETURNING id, task::TEXT")
                    .bind(&self.name)
                    .fetch_one(&mut *transaction)
                    .await?;

                transaction
                    .execute(query_builder!("CREATE SCHEMA IF NOT EXISTS %s", self.name).as_str())
                    .await?;

                let c: models::Collection = sqlx::query_as("INSERT INTO pgml.collections (name, project_id, sdk_version) VALUES ($1, $2, $3) ON CONFLICT (name) DO NOTHING RETURNING *")
                        .bind(&self.name)
                        .bind(project_id)
                        .bind(crate::SDK_VERSION)
                        .fetch_one(&mut *transaction)
                        .await?;

                let collection_database_data = CollectionDatabaseData {
                    id: c.id,
                    created_at: c.created_at,
                    project_info: ProjectInfo {
                        id: c.project_id,
                        name: self.name.clone(),
                        task: "embedding".into(),
                        database_url: self.database_url.clone(),
                    },
                };

                // Splitters table is not unique to a collection or pipeline. It exists in the pgml schema
                Splitter::create_splitters_table(&mut transaction).await?;
                self.create_documents_table(&mut transaction).await?;
                MultiFieldPipeline::create_multi_field_pipelines_table(
                    &collection_database_data.project_info,
                    &mut transaction,
                )
                .await?;

                transaction.commit().await?;
                Some(collection_database_data)
            };
        }
        Ok(())
    }

    /// Adds a new  [Pipeline] to the [Collection]
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The [Pipeline] to add.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Collection, Pipeline, Model, Splitter};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let model = Model::new(None, None, None);
    ///     let splitter = Splitter::new(None, None);
    ///     let mut pipeline = Pipeline::new("my_pipeline", None, None, None);
    ///     let mut collection = Collection::new("my_collection", None);
    ///     collection.add_pipeline(&mut pipeline).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn add_pipeline(&mut self, pipeline: &mut MultiFieldPipeline) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let project_info = &self
            .database_data
            .as_ref()
            .context("Database data must be set to add a pipeline to a collection")?
            .project_info;
        pipeline.set_project_info(project_info.clone());
        pipeline.verify_in_database(true).await?;
        let mp = MultiProgress::new();
        mp.println(format!("Added Pipeline {}, Now Syncing...", pipeline.name))?;
        self.sync_pipeline(pipeline).await?;
        eprintln!("Done Syncing {}\n", pipeline.name);
        Ok(())
    }

    /// Removes a [Pipeline] from the [Collection]
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The [Pipeline] to remove.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Collection, Pipeline};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let mut pipeline = Pipeline::new("my_pipeline", None, None, None);
    ///    let mut collection = Collection::new("my_collection", None);
    ///    collection.remove_pipeline(&mut pipeline).await?;
    ///    Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn remove_pipeline(
        &mut self,
        pipeline: &mut MultiFieldPipeline,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(false).await?;
        let project_info = &self
            .database_data
            .as_ref()
            .context("Database data must be set to remove pipeline from collection")?
            .project_info;
        pipeline.set_project_info(project_info.clone());
        pipeline.verify_in_database(false).await?;

        let pipeline_schema = format!("{}_{}", project_info.name, pipeline.name);

        let mut transaction = pool.begin().await?;
        transaction
            .execute(query_builder!("DROP SCHEMA IF EXISTS %s CASCADE", pipeline_schema).as_str())
            .await?;
        sqlx::query(&query_builder!(
            "UPDATE %s SET active = FALSE WHERE name = $1",
            self.pipelines_table_name
        ))
        .bind(&pipeline.name)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;

        Ok(())
    }

    /// Enables a [Pipeline] on the [Collection]
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The [Pipeline] to remove.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Collection, Pipeline};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let pipeline = Pipeline::new("my_pipeline", None, None, None);
    ///    let collection = Collection::new("my_collection", None);
    ///    collection.enable_pipeline(&pipeline).await?;
    ///    Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn enable_pipeline(&self, pipeline: &Pipeline) -> anyhow::Result<()> {
        sqlx::query(&query_builder!(
            "UPDATE %s SET active = TRUE WHERE name = $1",
            self.pipelines_table_name
        ))
        .bind(&pipeline.name)
        .execute(&get_or_initialize_pool(&self.database_url).await?)
        .await?;
        Ok(())
    }

    /// Disables a [Pipeline] on the [Collection]
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The [Pipeline] to remove.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Collection, Pipeline};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let pipeline = Pipeline::new("my_pipeline", None, None, None);
    ///    let collection = Collection::new("my_collection", None);
    ///    collection.disable_pipeline(&pipeline).await?;
    ///    Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn disable_pipeline(&self, pipeline: &Pipeline) -> anyhow::Result<()> {
        sqlx::query(&query_builder!(
            "UPDATE %s SET active = FALSE WHERE name = $1",
            self.pipelines_table_name
        ))
        .bind(&pipeline.name)
        .execute(&get_or_initialize_pool(&self.database_url).await?)
        .await?;
        Ok(())
    }

    #[instrument(skip(self, conn))]
    async fn create_documents_table(&mut self, conn: &mut PgConnection) -> anyhow::Result<()> {
        conn.execute(
            query_builder!(queries::CREATE_DOCUMENTS_TABLE, self.documents_table_name).as_str(),
        )
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX_USING_GIN,
                "",
                "documents_document_index",
                self.documents_table_name,
                "document jsonb_path_ops"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    /// Upserts documents into the database
    ///
    /// # Arguments
    ///
    /// * `documents` - A vector of documents to upsert
    /// * `strict` - Whether to throw an error if keys: `id` or `text` are missing from any documents
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None);
    ///    let documents = vec![
    ///        serde_json::json!({"id": 1, "text": "hello world"}).into(),
    ///        serde_json::json!({"id": 2, "text": "hello world"}).into(),
    ///    ];
    ///    collection.upsert_documents(documents, None).await?;
    ///    Ok(())
    /// }
    /// ```
    // TODO: Make it so if we upload the same documen twice it doesn't do anything
    #[instrument(skip(self, documents))]
    pub async fn upsert_documents(
        &mut self,
        documents: Vec<Json>,
        _args: Option<Json>,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(false).await?;
        let mut pipelines = self.get_pipelines().await?;
        for pipeline in &mut pipelines {
            pipeline.create_tables().await?;
        }

        let progress_bar = utils::default_progress_bar(documents.len() as u64);
        progress_bar.println("Upserting Documents...");

        for document in documents {
            let mut transaction = pool.begin().await?;
            let id = document
                .get("id")
                .context("`id` must be a key in document")?
                .to_string();
            let md5_digest = md5::compute(id.as_bytes());
            let source_uuid = uuid::Uuid::from_slice(&md5_digest.0)?;

            let document_id: i64 = sqlx::query_scalar(&query_builder!("INSERT INTO %s (source_uuid, document) VALUES ($1, $2) ON CONFLICT (source_uuid) DO UPDATE SET document = $2 RETURNING id", self.documents_table_name)).bind(source_uuid).bind(document).fetch_one(&mut *transaction).await?;

            let transaction = Arc::new(Mutex::new(transaction));
            if !pipelines.is_empty() {
                use futures::stream::StreamExt;
                futures::stream::iter(&mut pipelines)
                    // Need this map to get around moving the transaction
                    .map(|pipeline| (pipeline, transaction.clone()))
                    .for_each_concurrent(10, |(pipeline, transaction)| async move {
                        pipeline
                            .execute(Some(document_id), transaction)
                            .await
                            .expect("Failed to execute pipeline");
                    })
                    .await;
            }

            Arc::into_inner(transaction)
                .context("Error transaction dangling")?
                .into_inner()
                .commit()
                .await?;
        }

        progress_bar.println("Done Upserting Documents\n");
        progress_bar.finish();
        Ok(())
    }

    /// Gets the documents on a [Collection]
    ///
    /// # Arguments
    ///
    /// * `args` - The filters and options to apply to the query
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let mut collection = Collection::new("my_collection", None);
    ///     let documents = collection.get_documents(None).await?;
    ///     Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn get_documents(&self, args: Option<Json>) -> anyhow::Result<Vec<Json>> {
        // TODO: If we want to filter on full text this needs to be part of a pipeline
        unimplemented!()

        // let pool = get_or_initialize_pool(&self.database_url).await?;

        // let mut args = args.unwrap_or_default().0;
        // let args = args.as_object_mut().context("args must be an object")?;

        // // Get limit or set it to 1000
        // let limit = args
        //     .remove("limit")
        //     .map(|l| l.try_to_u64())
        //     .unwrap_or(Ok(1000))?;

        // let mut query = Query::select();
        // query
        //     .from_as(
        //         self.documents_table_name.to_table_tuple(),
        //         SIden::Str("documents"),
        //     )
        //     .expr(Expr::cust("*")) // Adds the * in SELECT * FROM
        //     .limit(limit);

        // if let Some(order_by) = args.remove("order_by") {
        //     let order_by_builder =
        //         order_by_builder::OrderByBuilder::new(order_by, "documents", "metadata").build()?;
        //     for (order_by, order) in order_by_builder {
        //         query.order_by_expr_with_nulls(order_by, order, NullOrdering::Last);
        //     }
        // }
        // query.order_by((SIden::Str("documents"), SIden::Str("id")), Order::Asc);

        // // TODO: Make keyset based pagination work with custom order by
        // if let Some(last_row_id) = args.remove("last_row_id") {
        //     let last_row_id = last_row_id
        //         .try_to_u64()
        //         .context("last_row_id must be an integer")?;
        //     query.and_where(Expr::col((SIden::Str("documents"), SIden::Str("id"))).gt(last_row_id));
        // }

        // if let Some(offset) = args.remove("offset") {
        //     let offset = offset.try_to_u64().context("offset must be an integer")?;
        //     query.offset(offset);
        // }

        // if let Some(mut filter) = args.remove("filter") {
        //     let filter = filter
        //         .as_object_mut()
        //         .context("filter must be a Json object")?;

        //     if let Some(f) = filter.remove("metadata") {
        //         query.cond_where(
        //             filter_builder::FilterBuilder::new(f, "documents", "metadata").build(),
        //         );
        //     }
        //     if let Some(f) = filter.remove("full_text_search") {
        //         let f = f
        //             .as_object()
        //             .context("Full text filter must be a Json object")?;
        //         let configuration = f
        //             .get("configuration")
        //             .context("In full_text_search `configuration` is required")?
        //             .as_str()
        //             .context("In full_text_search `configuration` must be a string")?;
        //         let filter_text = f
        //             .get("text")
        //             .context("In full_text_search `text` is required")?
        //             .as_str()
        //             .context("In full_text_search `text` must be a string")?;
        //         query
        //             .join_as(
        //                 JoinType::InnerJoin,
        //                 self.documents_tsvectors_table_name.to_table_tuple(),
        //                 Alias::new("documents_tsvectors"),
        //                 Expr::col((SIden::Str("documents"), SIden::Str("id")))
        //                     .equals((SIden::Str("documents_tsvectors"), SIden::Str("document_id"))),
        //             )
        //             .and_where(
        //                 Expr::col((
        //                     SIden::Str("documents_tsvectors"),
        //                     SIden::Str("configuration"),
        //                 ))
        //                 .eq(configuration),
        //             )
        //             .and_where(Expr::cust_with_values(
        //                 format!(
        //                     "documents_tsvectors.ts @@ plainto_tsquery('{}', $1)",
        //                     configuration
        //                 ),
        //                 [filter_text],
        //             ));
        //     }
        // }

        // let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        // let documents: Vec<models::Document> =
        //     sqlx::query_as_with(&sql, values).fetch_all(&pool).await?;
        // Ok(documents
        //     .into_iter()
        //     .map(|d| d.into_user_friendly_json())
        //     .collect())
    }

    /// Deletes documents in a [Collection]
    ///
    /// # Arguments
    ///
    /// * `filter` - The filters to apply
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let mut collection = Collection::new("my_collection", None);
    ///     let documents = collection.delete_documents(serde_json::json!({
    ///         "metadata": {
    ///             "id": {
    ///                 "eq": 1
    ///             }
    ///         }
    ///     }).into()).await?;
    ///     Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn delete_documents(&self, mut filter: Json) -> anyhow::Result<()> {
        // TODO: If we want to filter on full text this needs to be part of a pipeline
        unimplemented!()

        // let pool = get_or_initialize_pool(&self.database_url).await?;

        // let mut query = Query::delete();
        // query.from_table(self.documents_table_name.to_table_tuple());

        // let filter = filter
        //     .as_object_mut()
        //     .context("filter must be a Json object")?;

        // if let Some(f) = filter.remove("metadata") {
        //     query
        //         .cond_where(filter_builder::FilterBuilder::new(f, "documents", "metadata").build());
        // }

        // if let Some(mut f) = filter.remove("full_text_search") {
        //     let f = f
        //         .as_object_mut()
        //         .context("Full text filter must be a Json object")?;
        //     let configuration = f
        //         .get("configuration")
        //         .context("In full_text_search `configuration` is required")?
        //         .as_str()
        //         .context("In full_text_search `configuration` must be a string")?;
        //     let filter_text = f
        //         .get("text")
        //         .context("In full_text_search `text` is required")?
        //         .as_str()
        //         .context("In full_text_search `text` must be a string")?;
        //     let mut inner_select_query = Query::select();
        //     inner_select_query
        //         .from_as(
        //             self.documents_tsvectors_table_name.to_table_tuple(),
        //             SIden::Str("documents_tsvectors"),
        //         )
        //         .column(SIden::Str("document_id"))
        //         .and_where(Expr::cust_with_values(
        //             format!(
        //                 "documents_tsvectors.ts @@ plainto_tsquery('{}', $1)",
        //                 configuration
        //             ),
        //             [filter_text],
        //         ))
        //         .and_where(
        //             Expr::col((
        //                 SIden::Str("documents_tsvectors"),
        //                 SIden::Str("configuration"),
        //             ))
        //             .eq(configuration),
        //         );
        //     query.and_where(
        //         Expr::col((SIden::Str("documents"), SIden::Str("id")))
        //             .in_subquery(inner_select_query),
        //     );
        // }

        // let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        // sqlx::query_with(&sql, values).fetch_all(&pool).await?;
        // Ok(())
    }

    #[instrument(skip(self))]
    async fn sync_pipeline(&mut self, pipeline: &mut MultiFieldPipeline) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let project_info = &self
            .database_data
            .as_ref()
            .context("Database data must be set to get collection pipelines")?
            .project_info;
        pipeline.set_project_info(project_info.clone());
        pipeline.create_tables().await?;

        let pool = get_or_initialize_pool(&self.database_url).await?;
        let transaction = pool.begin().await?;
        let transaction = Arc::new(Mutex::new(transaction));
        pipeline.execute(None, transaction.clone()).await?;

        Arc::into_inner(transaction)
            .context("Error transaction dangling")?
            .into_inner()
            .commit()
            .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn search(
        &mut self,
        query: Json,
        pipeline: &mut MultiFieldPipeline,
    ) -> anyhow::Result<Vec<Json>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let (built_query, values) = build_search_query(self, query.clone(), pipeline).await?;
        let results: Result<Vec<(Json,)>, _> = sqlx::query_as_with(&built_query, values)
            .fetch_all(&pool)
            .await;

        match results {
            Ok(r) => Ok(r.into_iter().map(|r| r.0).collect()),
            Err(e) => match e.as_database_error() {
                Some(d) => {
                    if d.code() == Some(Cow::from("XX000")) {
                        self.verify_in_database(false).await?;
                        let project_info = &self
                            .database_data
                            .as_ref()
                            .context("Database data must be set to do remote embeddings search")?
                            .project_info;
                        pipeline.set_project_info(project_info.to_owned());
                        pipeline.verify_in_database(false).await?;
                        let (built_query, values) =
                            build_search_query(self, query, pipeline).await?;
                        let results: Vec<(Json,)> = sqlx::query_as_with(&built_query, values)
                            .fetch_all(&pool)
                            .await?;
                        Ok(results.into_iter().map(|r| r.0).collect())
                    } else {
                        Err(anyhow::anyhow!(e))
                    }
                }
                None => Err(anyhow::anyhow!(e)),
            },
        }
    }

    /// Performs vector search on the [Collection]
    ///
    /// # Arguments
    ///
    /// * `query` - The query to search for
    /// * `pipeline` - The [Pipeline] used for the search
    /// * `query_paramaters` - The query parameters passed to the model for search
    /// * `top_k` - How many results to limit on.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Collection, Pipeline};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let mut collection = Collection::new("my_collection", None);
    ///     let mut pipeline = Pipeline::new("my_pipeline", None, None, None);
    ///     let results = collection.vector_search("Query", &mut pipeline, None, None).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    #[allow(clippy::type_complexity)]
    pub async fn vector_search(
        &mut self,
        query: Json,
        pipeline: &mut MultiFieldPipeline,
        top_k: Option<i64>,
    ) -> anyhow::Result<Vec<Json>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let (built_query, values) =
            build_vector_search_query(query.clone(), self, pipeline).await?;
        let results: Result<Vec<(Json, String, f64)>, _> =
            sqlx::query_as_with(&built_query, values)
                .fetch_all(&pool)
                .await;
        match results {
            Ok(r) => Ok(r
                .into_iter()
                .map(|v| {
                    serde_json::json!({
                        "document": v.0,
                        "chunk": v.1,
                        "score": v.2
                    })
                    .into()
                })
                .collect()),
            Err(e) => match e.as_database_error() {
                Some(d) => {
                    if d.code() == Some(Cow::from("XX000")) {
                        self.verify_in_database(false).await?;
                        let project_info = &self
                            .database_data
                            .as_ref()
                            .context("Database data must be set to do remote embeddings search")?
                            .project_info;
                        pipeline.set_project_info(project_info.to_owned());
                        pipeline.verify_in_database(false).await?;
                        let (built_query, values) =
                            build_vector_search_query(query, self, pipeline).await?;
                        let results: Vec<(Json, String, f64)> =
                            sqlx::query_as_with(&built_query, values)
                                .fetch_all(&pool)
                                .await?;
                        Ok(results
                            .into_iter()
                            .map(|v| {
                                serde_json::json!({
                                    "document": v.0,
                                    "chunk": v.1,
                                    "score": v.2
                                })
                                .into()
                            })
                            .collect())
                    } else {
                        Err(anyhow::anyhow!(e))
                    }
                }
                None => Err(anyhow::anyhow!(e)),
            },
        }
    }

    #[instrument(skip(self))]
    pub async fn archive(&mut self) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let pipelines = self.get_pipelines().await?;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Error getting system time")
            .as_secs();
        let collection_archive_name = format!("{}_archive_{}", &self.name, timestamp);
        let mut transaciton = pool.begin().await?;
        // Change name in pgml.collections
        sqlx::query("UPDATE pgml.collections SET name = $1, active = FALSE where name = $2")
            .bind(&collection_archive_name)
            .bind(&self.name)
            .execute(&mut *transaciton)
            .await?;
        // Change collection_pipeline schema
        for pipeline in pipelines {
            sqlx::query(&query_builder!(
                "ALTER SCHEMA %s RENAME TO %s",
                format!("{}_{}", self.name, pipeline.name),
                format!("{}_{}", collection_archive_name, pipeline.name)
            ))
            .execute(&mut *transaciton)
            .await?;
        }
        // Change collection schema
        sqlx::query(&query_builder!(
            "ALTER SCHEMA %s RENAME TO %s",
            &self.name,
            collection_archive_name
        ))
        .execute(&mut *transaciton)
        .await?;
        transaciton.commit().await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(self.clone())
    }

    /// Gets all pipelines for the [Collection]
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let mut collection = Collection::new("my_collection", None);
    ///     let pipelines = collection.get_pipelines().await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_pipelines(&mut self) -> anyhow::Result<Vec<MultiFieldPipeline>> {
        self.verify_in_database(false).await?;
        let project_info = &self
            .database_data
            .as_ref()
            .context("Database data must be set to get collection pipelines")?
            .project_info;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let pipelines: Vec<models::Pipeline> = sqlx::query_as(&query_builder!(
            "SELECT * FROM %s WHERE active = TRUE",
            self.pipelines_table_name
        ))
        .fetch_all(&pool)
        .await?;

        pipelines
            .into_iter()
            .map(|p| {
                let mut p: MultiFieldPipeline = p.try_into()?;
                p.set_project_info(project_info.clone());
                Ok(p)
            })
            .collect()
    }

    /// Gets a [Pipeline] by name
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let mut collection = Collection::new("my_collection", None);
    ///     let pipeline = collection.get_pipeline("my_pipeline").await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_pipeline(&mut self, name: &str) -> anyhow::Result<MultiFieldPipeline> {
        self.verify_in_database(false).await?;
        let project_info = &self
            .database_data
            .as_ref()
            .context("Database data must be set to get collection pipelines")?
            .project_info;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let pipeline: models::Pipeline = sqlx::query_as(&query_builder!(
            "SELECT * FROM %s WHERE name = $1 AND active = TRUE LIMIT 1",
            self.pipelines_table_name
        ))
        .bind(name)
        .fetch_one(&pool)
        .await?;
        let mut pipeline: MultiFieldPipeline = pipeline.try_into()?;
        pipeline.set_project_info(project_info.clone());
        Ok(pipeline)
    }

    #[instrument(skip(self))]
    pub(crate) async fn get_project_info(&mut self) -> anyhow::Result<ProjectInfo> {
        self.verify_in_database(false).await?;
        Ok(self
            .database_data
            .as_ref()
            .context("Collection must be verified to get project info")?
            .project_info
            .clone())
    }

    /// Check if the [Collection] exists in the database
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let collection = Collection::new("my_collection", None);
    ///     let exists = collection.exists().await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn exists(&self) -> anyhow::Result<bool> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let collection: Option<models::Collection> = sqlx::query_as::<_, models::Collection>(
            "SELECT * FROM pgml.collections WHERE name = $1 AND active = TRUE;",
        )
        .bind(&self.name)
        .fetch_optional(&pool)
        .await?;
        Ok(collection.is_some())
    }

    #[instrument(skip(self))]
    pub async fn upsert_directory(&mut self, path: &str, args: Json) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let mut documents: Vec<Json> = Vec::new();

        let file_types: Vec<&str> = args["file_types"]
            .as_array()
            .context("file_types must be an array of valid file types. E.G. ['md', 'txt']")?
            .iter()
            .map(|v| {
                let v = v.as_str().with_context(|| {
                    format!("file_types must be an array of valid file types. E.G. ['md', 'txt']. Found: {}", v)
                })?;
                Ok(v)
            })
            .collect::<anyhow::Result<Vec<&str>>>()?;

        let file_batch_size: usize = args["file_batch_size"]
            .as_u64()
            .map(|v| v as usize)
            .unwrap_or(10);

        let follow_links: bool = args["follow_links"].as_bool().unwrap_or(false);

        let ignore_paths: Vec<Regex> =
            args["ignore_paths"]
                .as_array()
                .map_or(Ok(Vec::new()), |v| {
                    v.iter()
                        .map(|v| {
                            let v = v.as_str().with_context(|| {
                                "ignore_paths must be an array of valid regexes".to_string()
                            })?;
                            Regex::new(v).with_context(|| format!("Invalid regex: {}", v))
                        })
                        .collect()
                })?;

        for entry in WalkDir::new(path).follow_links(follow_links) {
            let entry = entry.context("Error reading directory")?;
            if !entry.path().is_file() {
                continue;
            }
            if let Some(extension) = entry.path().extension() {
                let nice_path = entry.path().to_str().context("Path is not valid UTF-8")?;
                let extension = extension
                    .to_str()
                    .with_context(|| format!("Extension is not valid UTF-8: {}", nice_path))?;
                if !file_types.contains(&extension)
                    || ignore_paths.iter().any(|r| r.is_match(nice_path))
                {
                    continue;
                }

                let contents = utils::get_file_contents(entry.path())?;
                documents.push(
                    json!({
                        "id": nice_path,
                        "file_type": extension,
                        "text": contents
                    })
                    .into(),
                );
                if documents.len() == file_batch_size {
                    self.upsert_documents(documents, None).await?;
                    documents = Vec::new();
                }
            }
        }
        if !documents.is_empty() {
            self.upsert_documents(documents, None).await?;
        }
        Ok(())
    }

    pub async fn generate_er_diagram(
        &mut self,
        pipeline: &mut MultiFieldPipeline,
    ) -> anyhow::Result<String> {
        self.verify_in_database(false).await?;
        pipeline.verify_in_database(false).await?;

        let parsed_schema = pipeline
            .parsed_schema
            .as_ref()
            .context("Pipeline must have schema to generate er diagram")?;

        let mut uml_entites = format!(
            r#"
@startuml
' hide the spot
' hide circle

' avoid problems with angled crows feet
skinparam linetype ortho

entity "pgml.collections" as pgmlc {{
    id : bigint
    --
    created_at : timestamp without time zone                
    name : text
    active : boolean
    project_id : bigint
    sdk_version : text
}}

entity "{}.documents" as documents {{
    id : bigint              
    --
    created_at : timestamp without time zone
    source_uuid : uuid
    document : jsonb
}}

entity "{}.pipelines" as pipelines {{
    id : bigint
    --
    created_at : timestamp without time zone
    name : text
    active : boolean
    schema : jsonb
}}
        "#,
            self.name, self.name
        );

        let schema = format!("{}_{}", self.name, pipeline.name);

        let mut uml_relations = r#"
pgmlc ||..|| pipelines
        "#
        .to_string();

        for (key, field_action) in parsed_schema.iter() {
            let nice_name_key = key.replace(' ', "_");

            let relations = format!(
                r#"
documents ||..|{{ {nice_name_key}_chunks
{nice_name_key}_chunks ||.|| {nice_name_key}_embeddings
                    "#
            );
            uml_relations.push_str(&relations);

            if let Some(_embed_action) = &field_action.embed {
                let entites = format!(
                    r#"
entity "{schema}.{key}_chunks" as {nice_name_key}_chunks {{
    id : bigint
    --
    created_at : timestamp without time zone
    document_id : bigint
    chunk_index : bigint
    chunk : text
}}

entity "{schema}.{key}_embeddings" as {nice_name_key}_embeddings {{
    id : bigint
    --
    created_at : timestamp without time zone
    chunk_id : bigint
    document_id : bigint
    embedding : vector
}}
                        "#
                );
                uml_entites.push_str(&entites);
            }

            if let Some(_full_text_search_action) = &field_action.full_text_search {
                let entites = format!(
                    r#"
entity "{schema}.{key}_tsvectors" as {nice_name_key}_tsvectors {{
    id : bigint
    --
    created_at : timestamp without time zone
    chunk_id : bigint
    document_id : bigint
    tsvectors : tsvector
}}
                        "#
                );
                uml_entites.push_str(&entites);

                let relations = format!(
                    r#"
{nice_name_key}_chunks ||..|| {nice_name_key}_tsvectors
                    "#
                );
                uml_relations.push_str(&relations);
            }
        }

        uml_entites.push_str(&uml_relations);
        Ok(uml_entites)
    }

    pub async fn upsert_file(&mut self, path: &str) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let path = Path::new(path);
        let contents = utils::get_file_contents(path)?;
        let document = json!({
            "id": path,
            "text": contents
        });
        self.upsert_documents(vec![document.into()], None).await
    }

    fn generate_table_names(name: &str) -> (String, String, String, String) {
        [
            ".pipelines",
            ".documents",
            ".chunks",
            ".documents_tsvectors",
        ]
        .into_iter()
        .map(|s| format!("{}{}", name, s))
        .collect_tuple()
        .unwrap()
    }
}
