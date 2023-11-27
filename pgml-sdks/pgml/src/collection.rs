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
use std::time::SystemTime;
use tracing::{instrument, warn};
use walkdir::WalkDir;

use crate::{
    filter_builder, get_or_initialize_pool,
    model::ModelRuntime,
    models, order_by_builder,
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
    pub transforms_table_name: String,
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
            transforms_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
        ) = Self::generate_table_names(name);
        Self {
            name: name.to_string(),
            database_url,
            pipelines_table_name,
            documents_table_name,
            transforms_table_name,
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

                Splitter::create_splitters_table(&mut transaction).await?;
                Pipeline::create_pipelines_table(
                    &collection_database_data.project_info,
                    &mut transaction,
                )
                .await?;
                self.create_documents_table(&mut transaction).await?;
                self.create_chunks_table(&mut transaction).await?;
                self.create_documents_tsvectors_table(&mut transaction)
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
    pub async fn add_pipeline(&mut self, pipeline: &mut Pipeline) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        pipeline.set_project_info(self.database_data.as_ref().unwrap().project_info.clone());
        let mp = MultiProgress::new();
        mp.println(format!("Added Pipeline {}, Now Syncing...", pipeline.name))?;
        pipeline.execute(&None, mp).await?;
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
    pub async fn remove_pipeline(&mut self, pipeline: &mut Pipeline) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(false).await?;
        pipeline.set_project_info(self.database_data.as_ref().unwrap().project_info.clone());
        pipeline.verify_in_database(false).await?;

        let database_data = pipeline
            .database_data
            .as_ref()
            .context("Pipeline must be verified to remove it")?;

        let embeddings_table_name = format!("{}.{}_embeddings", self.name, pipeline.name);

        let parameters = pipeline
            .parameters
            .as_ref()
            .context("Pipeline must be verified to remove it")?;

        let mut transaction = pool.begin().await?;

        // Need to delete from chunks table only if no other pipelines use the same splitter
        sqlx::query(&query_builder!(
                "DELETE FROM %s WHERE splitter_id = $1 AND NOT EXISTS (SELECT 1 FROM %s WHERE splitter_id = $1 AND id != $2)",
                self.chunks_table_name,
                self.pipelines_table_name
            ))
            .bind(database_data.splitter_id)
            .bind(database_data.id)
            .execute(&mut *transaction)
            .await?;

        // Drop the embeddings table
        sqlx::query(&query_builder!(
            "DROP TABLE IF EXISTS %s",
            embeddings_table_name
        ))
        .execute(&mut *transaction)
        .await?;

        // Need to delete from the tsvectors table only if no other pipelines use the
        // same tsvector configuration
        sqlx::query(&query_builder!(
                    "DELETE FROM %s WHERE configuration = $1 AND NOT EXISTS (SELECT 1 FROM %s WHERE parameters->'full_text_search'->>'configuration' = $1 AND id != $2)", 
                    self.documents_tsvectors_table_name,
                    self.pipelines_table_name))
                .bind(parameters["full_text_search"]["configuration"].as_str())
                .bind(database_data.id)
                .execute(&mut *transaction)
                .await?;

        sqlx::query(&query_builder!(
            "DELETE FROM %s WHERE id = $1",
            self.pipelines_table_name
        ))
        .bind(database_data.id)
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
                queries::CREATE_INDEX,
                "",
                "created_at_index",
                self.documents_table_name,
                "created_at"
            )
            .as_str(),
        )
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX_USING_GIN,
                "",
                "metadata_index",
                self.documents_table_name,
                "metadata jsonb_path_ops"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    #[instrument(skip(self, conn))]
    async fn create_chunks_table(&mut self, conn: &mut PgConnection) -> anyhow::Result<()> {
        conn.execute(
            query_builder!(
                queries::CREATE_CHUNKS_TABLE,
                self.chunks_table_name,
                self.documents_table_name
            )
            .as_str(),
        )
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "",
                "created_at_index",
                self.chunks_table_name,
                "created_at"
            )
            .as_str(),
        )
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "",
                "document_id_index",
                self.chunks_table_name,
                "document_id"
            )
            .as_str(),
        )
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "",
                "splitter_id_index",
                self.chunks_table_name,
                "splitter_id"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    #[instrument(skip(self, conn))]
    async fn create_documents_tsvectors_table(
        &mut self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<()> {
        conn.execute(
            query_builder!(
                queries::CREATE_DOCUMENTS_TSVECTORS_TABLE,
                self.documents_tsvectors_table_name,
                self.documents_table_name
            )
            .as_str(),
        )
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "",
                "configuration_index",
                self.documents_tsvectors_table_name,
                "configuration"
            )
            .as_str(),
        )
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX_USING_GIN,
                "",
                "tsvector_index",
                self.documents_tsvectors_table_name,
                "ts"
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
    #[instrument(skip(self, documents))]
    pub async fn upsert_documents(
        &mut self,
        documents: Vec<Json>,
        args: Option<Json>,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(false).await?;

        let args = args.unwrap_or_default();

        let progress_bar = utils::default_progress_bar(documents.len() as u64);
        progress_bar.println("Upserting Documents...");

        let documents: anyhow::Result<Vec<_>> = documents
            .into_iter()
            .map(|mut document| {
                let document = document
                    .as_object_mut()
                    .context("Documents must be a vector of objects")?;

                // We don't want the text included in the document metadata, but everything else
                // should be in there
                let text = document.remove("text").map(|t| {
                    t.as_str()
                        .expect("`text` must be a string in document")
                        .to_string()
                });
                let metadata = serde_json::to_value(&document)?.into();

                let id = document
                    .get("id")
                    .context("`id` must be a key in document")?
                    .to_string();
                let md5_digest = md5::compute(id.as_bytes());
                let source_uuid = uuid::Uuid::from_slice(&md5_digest.0)?;

                Ok((source_uuid, text, metadata))
            })
            .collect();

        // We could continue chaining the above iterators but types become super annoying to
        // deal with, especially because we are dealing with async functions. This is much easier to read
        // Also, we may want to use a variant of chunks that is owned, I'm not 100% sure of what
        // cloning happens when passing values into sqlx bind. itertools variants will not work as
        // it is not thread safe and pyo3 will get upset
        let mut document_ids = Vec::new();
        for chunk in documents?.chunks(10) {
            // Need to make it a vec to partition it and must include explicit typing here
            let mut chunk: Vec<&(uuid::Uuid, Option<String>, Json)> = chunk.iter().collect();

            // Split the chunk into two groups, one with text, and one with just metadata
            let split_index = itertools::partition(&mut chunk, |(_, text, _)| text.is_some());
            let (text_chunk, metadata_chunk) = chunk.split_at(split_index);

            // Start the transaction
            let mut transaction = pool.begin().await?;

            if !metadata_chunk.is_empty() {
                // Update the metadata
                // Merge the metadata if the user has specified to do so otherwise replace it
                if args["metadata"]["merge"].as_bool().unwrap_or(false) {
                    sqlx::query(query_builder!(
                    "UPDATE %s d SET metadata = d.metadata || v.metadata FROM (SELECT UNNEST($1) source_uuid, UNNEST($2) metadata) v WHERE d.source_uuid = v.source_uuid",
                    self.documents_table_name
                ).as_str()).bind(metadata_chunk.iter().map(|(source_uuid, _, _)| *source_uuid).collect::<Vec<_>>())
                    .bind(metadata_chunk.iter().map(|(_, _, metadata)| metadata.0.clone()).collect::<Vec<_>>())
                    .execute(&mut *transaction).await?;
                } else {
                    sqlx::query(query_builder!(
                "UPDATE %s d SET metadata = v.metadata FROM (SELECT UNNEST($1) source_uuid, UNNEST($2) metadata) v WHERE d.source_uuid = v.source_uuid",
                self.documents_table_name
            ).as_str()).bind(metadata_chunk.iter().map(|(source_uuid, _, _)| *source_uuid).collect::<Vec<_>>())
                .bind(metadata_chunk.iter().map(|(_, _, metadata)| metadata.0.clone()).collect::<Vec<_>>())
                .execute(&mut *transaction).await?;
                }
            }

            if !text_chunk.is_empty() {
                // First delete any documents that already have the same UUID as documents in
                // text_chunk, then insert the new ones.
                // We are essentially upserting in two steps
                sqlx::query(&query_builder!(
                "DELETE FROM %s WHERE source_uuid IN (SELECT source_uuid FROM %s WHERE source_uuid = ANY($1::uuid[]))",
                self.documents_table_name,
                self.documents_table_name
            )).
                bind(&text_chunk.iter().map(|(source_uuid, _, _)| *source_uuid).collect::<Vec<_>>()).
                execute(&mut *transaction).await?;
                let query_string_values = (0..text_chunk.len())
                    .map(|i| format!("(${}, ${}, ${})", i * 3 + 1, i * 3 + 2, i * 3 + 3))
                    .collect::<Vec<String>>()
                    .join(",");
                let query_string = format!(
                "INSERT INTO %s (source_uuid, text, metadata) VALUES {} ON CONFLICT (source_uuid) DO UPDATE SET text = $2, metadata = $3 RETURNING id",
                query_string_values
            );
                let query = query_builder!(query_string, self.documents_table_name);
                let mut query = sqlx::query_scalar(&query);
                for (source_uuid, text, metadata) in text_chunk.iter() {
                    query = query.bind(source_uuid).bind(text).bind(metadata);
                }
                let ids: Vec<i64> = query.fetch_all(&mut *transaction).await?;
                document_ids.extend(ids);
                progress_bar.inc(chunk.len() as u64);
            }

            transaction.commit().await?;
        }
        progress_bar.finish();
        eprintln!("Done Upserting Documents\n");

        self.sync_pipelines(Some(document_ids)).await?;
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
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let mut args = args.unwrap_or_default().0;
        let args = args.as_object_mut().context("args must be an object")?;

        // Get limit or set it to 1000
        let limit = args
            .remove("limit")
            .map(|l| l.try_to_u64())
            .unwrap_or(Ok(1000))?;

        let mut query = Query::select();
        query
            .from_as(
                self.documents_table_name.to_table_tuple(),
                SIden::Str("documents"),
            )
            .expr(Expr::cust("*")) // Adds the * in SELECT * FROM
            .limit(limit);

        if let Some(order_by) = args.remove("order_by") {
            let order_by_builder =
                order_by_builder::OrderByBuilder::new(order_by, "documents", "metadata").build()?;
            for (order_by, order) in order_by_builder {
                query.order_by_expr_with_nulls(order_by, order, NullOrdering::Last);
            }
        }
        query.order_by((SIden::Str("documents"), SIden::Str("id")), Order::Asc);

        // TODO: Make keyset based pagination work with custom order by
        if let Some(last_row_id) = args.remove("last_row_id") {
            let last_row_id = last_row_id
                .try_to_u64()
                .context("last_row_id must be an integer")?;
            query.and_where(Expr::col((SIden::Str("documents"), SIden::Str("id"))).gt(last_row_id));
        }

        if let Some(offset) = args.remove("offset") {
            let offset = offset.try_to_u64().context("offset must be an integer")?;
            query.offset(offset);
        }

        if let Some(mut filter) = args.remove("filter") {
            let filter = filter
                .as_object_mut()
                .context("filter must be a Json object")?;

            if let Some(f) = filter.remove("metadata") {
                query.cond_where(
                    filter_builder::FilterBuilder::new(f, "documents", "metadata").build(),
                );
            }
            if let Some(f) = filter.remove("full_text_search") {
                let f = f
                    .as_object()
                    .context("Full text filter must be a Json object")?;
                let configuration = f
                    .get("configuration")
                    .context("In full_text_search `configuration` is required")?
                    .as_str()
                    .context("In full_text_search `configuration` must be a string")?;
                let filter_text = f
                    .get("text")
                    .context("In full_text_search `text` is required")?
                    .as_str()
                    .context("In full_text_search `text` must be a string")?;
                query
                    .join_as(
                        JoinType::InnerJoin,
                        self.documents_tsvectors_table_name.to_table_tuple(),
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
            }
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let documents: Vec<models::Document> =
            sqlx::query_as_with(&sql, values).fetch_all(&pool).await?;
        Ok(documents
            .into_iter()
            .map(|d| d.into_user_friendly_json())
            .collect())
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
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let mut query = Query::delete();
        query.from_table(self.documents_table_name.to_table_tuple());

        let filter = filter
            .as_object_mut()
            .context("filter must be a Json object")?;

        if let Some(f) = filter.remove("metadata") {
            query
                .cond_where(filter_builder::FilterBuilder::new(f, "documents", "metadata").build());
        }

        if let Some(mut f) = filter.remove("full_text_search") {
            let f = f
                .as_object_mut()
                .context("Full text filter must be a Json object")?;
            let configuration = f
                .get("configuration")
                .context("In full_text_search `configuration` is required")?
                .as_str()
                .context("In full_text_search `configuration` must be a string")?;
            let filter_text = f
                .get("text")
                .context("In full_text_search `text` is required")?
                .as_str()
                .context("In full_text_search `text` must be a string")?;
            let mut inner_select_query = Query::select();
            inner_select_query
                .from_as(
                    self.documents_tsvectors_table_name.to_table_tuple(),
                    SIden::Str("documents_tsvectors"),
                )
                .column(SIden::Str("document_id"))
                .and_where(Expr::cust_with_values(
                    format!(
                        "documents_tsvectors.ts @@ plainto_tsquery('{}', $1)",
                        configuration
                    ),
                    [filter_text],
                ))
                .and_where(
                    Expr::col((
                        SIden::Str("documents_tsvectors"),
                        SIden::Str("configuration"),
                    ))
                    .eq(configuration),
                );
            query.and_where(
                Expr::col((SIden::Str("documents"), SIden::Str("id")))
                    .in_subquery(inner_select_query),
            );
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).fetch_all(&pool).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) async fn sync_pipelines(
        &mut self,
        document_ids: Option<Vec<i64>>,
    ) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let pipelines = self.get_pipelines().await?;
        if !pipelines.is_empty() {
            let mp = MultiProgress::new();
            mp.println("Syncing Pipelines...")?;
            use futures::stream::StreamExt;
            futures::stream::iter(pipelines)
                // Need this map to get around moving the document_ids and mp
                .map(|pipeline| (pipeline, document_ids.clone(), mp.clone()))
                .for_each_concurrent(10, |(mut pipeline, document_ids, mp)| async move {
                    pipeline
                        .execute(&document_ids, mp)
                        .await
                        .expect("Failed to execute pipeline");
                })
                .await;
            eprintln!("Done Syncing Pipelines\n");
        }
        Ok(())
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
        query: &str,
        pipeline: &mut Pipeline,
        query_parameters: Option<Json>,
        top_k: Option<i64>,
    ) -> anyhow::Result<Vec<(f64, String, Json)>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let query_parameters = query_parameters.unwrap_or_default();
        let top_k = top_k.unwrap_or(5);

        // With this system, we only do the wrong type of vector search once
        let runtime = if pipeline.model.is_some() {
            pipeline.model.as_ref().unwrap().runtime
        } else {
            ModelRuntime::Python
        };
        match runtime {
            ModelRuntime::Python => {
                let embeddings_table_name = format!("{}.{}_embeddings", self.name, pipeline.name);

                let result = sqlx::query_as(&query_builder!(
                    queries::EMBED_AND_VECTOR_SEARCH,
                    self.pipelines_table_name,
                    embeddings_table_name,
                    self.chunks_table_name,
                    self.documents_table_name
                ))
                .bind(&pipeline.name)
                .bind(query)
                .bind(&query_parameters)
                .bind(top_k)
                .fetch_all(&pool)
                .await;

                match result {
                    Ok(r) => Ok(r),
                    Err(e) => match e.as_database_error() {
                        Some(d) => {
                            if d.code() == Some(Cow::from("XX000")) {
                                self.vector_search_with_remote_embeddings(
                                    query,
                                    pipeline,
                                    query_parameters,
                                    top_k,
                                    &pool,
                                )
                                .await
                            } else {
                                Err(anyhow::anyhow!(e))
                            }
                        }
                        None => Err(anyhow::anyhow!(e)),
                    },
                }
            }
            _ => {
                self.vector_search_with_remote_embeddings(
                    query,
                    pipeline,
                    query_parameters,
                    top_k,
                    &pool,
                )
                .await
            }
        }
        .map(|r| {
            r.into_iter()
                .map(|(score, id, metadata)| (1. - score, id, metadata))
                .collect()
        })
    }

    #[instrument(skip(self, pool))]
    #[allow(clippy::type_complexity)]
    async fn vector_search_with_remote_embeddings(
        &mut self,
        query: &str,
        pipeline: &mut Pipeline,
        query_parameters: Json,
        top_k: i64,
        pool: &PgPool,
    ) -> anyhow::Result<Vec<(f64, String, Json)>> {
        self.verify_in_database(false).await?;

        // Have to set the project info before we can get and set the model
        pipeline.set_project_info(
            self.database_data
                .as_ref()
                .context(
                    "Collection must be verified to perform vector search with remote embeddings",
                )?
                .project_info
                .clone(),
        );
        // Verify to get and set the model if we don't have it set on the pipeline yet
        pipeline.verify_in_database(false).await?;
        let model = pipeline
            .model
            .as_ref()
            .context("Pipeline must be verified to perform vector search with remote embeddings")?;

        // We need to make sure we are not mutably and immutably borrowing the same things
        let embedding = {
            let remote_embeddings =
                build_remote_embeddings(model.runtime, &model.name, &query_parameters)?;
            let mut embeddings = remote_embeddings.embed(vec![query.to_string()]).await?;
            std::mem::take(&mut embeddings[0])
        };

        let embeddings_table_name = format!("{}.{}_embeddings", self.name, pipeline.name);
        sqlx::query_as(&query_builder!(
            queries::VECTOR_SEARCH,
            embeddings_table_name,
            self.chunks_table_name,
            self.documents_table_name
        ))
        .bind(embedding)
        .bind(top_k)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))
    }

    #[instrument(skip(self))]
    pub async fn archive(&mut self) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Error getting system time")
            .as_secs();
        let archive_table_name = format!("{}_archive_{}", &self.name, timestamp);
        let mut transaciton = pool.begin().await?;
        sqlx::query("UPDATE pgml.collections SET name = $1, active = FALSE where name = $2")
            .bind(&archive_table_name)
            .bind(&self.name)
            .execute(&mut *transaciton)
            .await?;
        sqlx::query(&query_builder!(
            "ALTER SCHEMA %s RENAME TO %s",
            &self.name,
            archive_table_name
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
    pub async fn get_pipelines(&mut self) -> anyhow::Result<Vec<Pipeline>> {
        self.verify_in_database(false).await?;
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let pipelines_with_models_and_splitters: Vec<models::PipelineWithModelAndSplitter> =
            sqlx::query_as(&query_builder!(
                r#"SELECT 
              p.id as pipeline_id, 
              p.name as pipeline_name, 
              p.created_at as pipeline_created_at, 
              p.active as pipeline_active, 
              p.parameters as pipeline_parameters, 
              m.id as model_id, 
              m.created_at as model_created_at, 
              m.runtime::TEXT as model_runtime, 
              m.hyperparams as model_hyperparams, 
              s.id as splitter_id, 
              s.created_at as splitter_created_at, 
              s.name as splitter_name, 
              s.parameters as splitter_parameters 
            FROM 
              %s p 
              INNER JOIN pgml.models m ON p.model_id = m.id 
              INNER JOIN pgml.splitters s ON p.splitter_id = s.id 
            WHERE 
              p.active = TRUE
            "#,
                self.pipelines_table_name
            ))
            .fetch_all(&pool)
            .await?;

        let pipelines: Vec<Pipeline> = pipelines_with_models_and_splitters
            .into_iter()
            .map(|p| {
                let mut pipeline: Pipeline = p.into();
                pipeline.set_project_info(
                    self.database_data
                        .as_ref()
                        .expect("Collection must be verified to get all pipelines")
                        .project_info
                        .clone(),
                );
                pipeline
            })
            .collect();
        Ok(pipelines)
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
    pub async fn get_pipeline(&mut self, name: &str) -> anyhow::Result<Pipeline> {
        self.verify_in_database(false).await?;
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let pipeline_with_model_and_splitter: models::PipelineWithModelAndSplitter =
            sqlx::query_as(&query_builder!(
                r#"SELECT 
              p.id as pipeline_id, 
              p.name as pipeline_name, 
              p.created_at as pipeline_created_at, 
              p.active as pipeline_active, 
              p.parameters as pipeline_parameters, 
              m.id as model_id, 
              m.created_at as model_created_at, 
              m.runtime::TEXT as model_runtime, 
              m.hyperparams as model_hyperparams, 
              s.id as splitter_id, 
              s.created_at as splitter_created_at, 
              s.name as splitter_name, 
              s.parameters as splitter_parameters 
            FROM 
              %s p 
              INNER JOIN pgml.models m ON p.model_id = m.id 
              INNER JOIN pgml.splitters s ON p.splitter_id = s.id 
            WHERE 
              p.active = TRUE
              AND p.name = $1
            "#,
                self.pipelines_table_name
            ))
            .bind(name)
            .fetch_one(&pool)
            .await?;

        let mut pipeline: Pipeline = pipeline_with_model_and_splitter.into();
        pipeline.set_project_info(self.database_data.as_ref().unwrap().project_info.clone());
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

    fn generate_table_names(name: &str) -> (String, String, String, String, String) {
        [
            ".pipelines",
            ".documents",
            ".transforms",
            ".chunks",
            ".documents_tsvectors",
        ]
        .into_iter()
        .map(|s| format!("{}{}", name, s))
        .collect_tuple()
        .unwrap()
    }
}
