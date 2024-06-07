use anyhow::Context;
use indicatif::MultiProgress;
use itertools::Itertools;
use regex::Regex;
use sea_query::Alias;
use sea_query::{Expr, NullOrdering, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde_json::{json, Value};
use sqlx::PgConnection;
use sqlx::{Executor, Pool, Postgres};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::task::JoinSet;
use tracing::{instrument, warn};
use walkdir::WalkDir;

use crate::debug_sqlx_query;
use crate::filter_builder::FilterBuilder;
use crate::pipeline::FieldAction;
use crate::rag_query_builder::build_rag_query;
use crate::search_query_builder::build_search_query;
use crate::types::GeneralJsonAsyncIterator;
use crate::vector_search_query_builder::build_vector_search_query;
use crate::{
    get_or_initialize_pool, models, order_by_builder,
    pipeline::Pipeline,
    queries, query_builder,
    query_builder::QueryBuilder,
    splitter::Splitter,
    types::{DateTime, IntoTableNameAndSchema, Json, SIden, TryToNumeric},
    utils,
};

#[cfg(feature = "rust_bridge")]
use rust_bridge::{alias, alias_methods};

#[cfg(feature = "c")]
use crate::languages::c::GeneralJsonAsyncIteratorC;

#[cfg(feature = "python")]
use crate::{
    pipeline::PipelinePython,
    query_builder::QueryBuilderPython,
    types::{GeneralJsonAsyncIteratorPython, JsonPython},
};

/// A RAGStream Struct
#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[allow(dead_code)]
pub struct RAGStream {
    general_json_async_iterator: Option<GeneralJsonAsyncIterator>,
    sources: Json,
}

// Required that we implement clone for our rust-bridge macros but it will not be used
impl Clone for RAGStream {
    fn clone(&self) -> Self {
        panic!("Cannot clone RAGStream")
    }
}

#[cfg_attr(feature = "rust_bridge", alias_methods(stream, sources))]
impl RAGStream {
    pub fn stream(&mut self) -> anyhow::Result<GeneralJsonAsyncIterator> {
        self.general_json_async_iterator
            .take()
            .context("Cannot call stream method more than once")
    }

    pub fn sources(&self) -> anyhow::Result<Json> {
        panic!("Cannot get sources yet for RAG streaming")
    }
}

#[cfg(feature = "c")]
use crate::{languages::c::JsonC, pipeline::PipelineC, query_builder::QueryBuilderC};

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
#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[derive(Debug, Clone)]
pub struct Collection {
    pub(crate) name: String,
    pub(crate) database_url: Option<String>,
    pub(crate) pipelines_table_name: String,
    pub(crate) documents_table_name: String,
    pub(crate) database_data: Option<CollectionDatabaseData>,
}

#[cfg_attr(
    feature = "rust_bridge",
    alias_methods(
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
        search,
        add_search_event,
        vector_search,
        query,
        rag,
        rag_stream,
        exists,
        archive,
        upsert_directory,
        upsert_file,
        generate_er_diagram,
        get_pipeline_status
    )
)]
impl Collection {
    /// Creates a new [Collection]
    ///
    /// # Arguments
    /// * `name` - The name of the collection.
    /// * `database_url` - An optional database_url. If passed, this url will be used instead of
    /// the `PGML_DATABASE_URL` environment variable.
    ///
    /// # Errors
    /// * If the `name` is not composed of alphanumeric characters, whitespace, or '-' and '_'
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use anyhow::Result;
    /// async fn doc() -> Result<()> {
    ///     let mut collection = Collection::new("my_collection", None)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new(name: &str, database_url: Option<String>) -> anyhow::Result<Self> {
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_')
        {
            anyhow::bail!(
                "Name must only consist of letters, numebers, white space, and '-' or '_'"
            )
        }
        let (pipelines_table_name, documents_table_name) = Self::generate_table_names(name);
        Ok(Self {
            name: name.to_string(),
            database_url,
            pipelines_table_name,
            documents_table_name,
            database_data: None,
        })
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
                Pipeline::create_pipelines_table(
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
    /// * `pipeline` - The [Pipeline] to add to the [Collection]
    ///
    /// # Errors
    /// * If the [Pipeline] does not have schema
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use anyhow::Result;
    /// use serde_json::json;
    /// async fn doc() -> Result<()> {
    ///     let mut collection = Collection::new("my_collection", None)?;
    ///     let mut pipeline = Pipeline::new("my_pipeline", Some(json!({}).into()))?;
    ///     collection.add_pipeline(&mut pipeline).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn add_pipeline(&mut self, pipeline: &mut Pipeline) -> anyhow::Result<()> {
        // The flow for this function:
        // 1. Create collection if it does not exists
        // 2. Create the pipeline if it does not exist and add it to the collection.pipelines table with ACTIVE = TRUE
        // 3. Sync the pipeline - this will delete all previous chunks, embeddings, and tsvectors
        self.verify_in_database(false).await?;
        let project_info = &self
            .database_data
            .as_ref()
            .context("Database data must be set to add a pipeline to a collection")?
            .project_info;

        // Let's check if we already have it enabled
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let pipelines_table_name = format!("{}.pipelines", project_info.name);
        let exists: bool = sqlx::query_scalar(&query_builder!(
            "SELECT EXISTS (SELECT id FROM %s WHERE name = $1 AND active = TRUE)",
            pipelines_table_name
        ))
        .bind(&pipeline.name)
        .fetch_one(&pool)
        .await?;

        if exists {
            warn!("Pipeline {} already exists not adding", pipeline.name);
        } else {
            // We want to intentially throw an error if they have already added this pipeline
            // as we don't want to casually resync
            pipeline
                .verify_in_database(project_info, true, &pool)
                .await?;

            let mp = MultiProgress::new();
            mp.println(format!("Added Pipeline {}, Now Syncing...", pipeline.name))?;

            // TODO: Revisit this. If the pipeline is added but fails to sync, then it will be "out of sync" with the documents in the table
            // This is rare, but could happen
            pipeline
                .resync(project_info, pool.acquire().await?.as_mut())
                .await?;
            mp.println(format!("Done Syncing {}\n", pipeline.name))?;
        }
        Ok(())
    }

    /// Removes a [Pipeline] from the [Collection]
    ///
    /// # Arguments
    /// * `pipeline` - The [Pipeline] to remove from the [Collection]
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use anyhow::Result;
    /// use serde_json::json;
    /// async fn doc() -> Result<()> {
    ///     let mut collection = Collection::new("my_collection", None)?;
    ///     let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///     collection.remove_pipeline(&mut pipeline).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn remove_pipeline(&mut self, pipeline: &Pipeline) -> anyhow::Result<()> {
        // The flow for this function:
        // 1. Create collection if it does not exist
        // 2. Begin a transaction
        // 3. Drop the collection_pipeline schema
        // 4. Delete the pipeline from the collection.pipelines table
        // 5. Commit the transaction
        self.verify_in_database(false).await?;
        let project_info = &self.database_data.as_ref().unwrap().project_info;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let pipeline_schema = format!("{}_{}", project_info.name, pipeline.name);

        let mut transaction = pool.begin().await?;
        transaction
            .execute(query_builder!("DROP SCHEMA IF EXISTS %s CASCADE", pipeline_schema).as_str())
            .await?;
        sqlx::query(&query_builder!(
            "DELETE FROM %s WHERE name = $1",
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
    /// * `pipeline` - The [Pipeline] to enable
    ///
    /// # Errors
    /// * If the pipeline has not already been added to the [Collection]
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use anyhow::Result;
    /// use serde_json::json;
    /// async fn doc() -> Result<()> {
    ///     let mut collection = Collection::new("my_collection", None)?;
    ///     let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///     collection.enable_pipeline(&mut pipeline).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn enable_pipeline(&mut self, pipeline: &mut Pipeline) -> anyhow::Result<()> {
        // The flow for this function:
        // 1. Set ACTIVE = TRUE for the pipeline in collection.pipelines
        // 2. Resync the pipeline
        // TODO: Review this pattern
        self.verify_in_database(false).await?;
        let project_info = &self.database_data.as_ref().unwrap().project_info;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        sqlx::query(&query_builder!(
            "UPDATE %s SET active = TRUE WHERE name = $1",
            self.pipelines_table_name
        ))
        .bind(&pipeline.name)
        .execute(&pool)
        .await?;
        pipeline
            .resync(project_info, pool.acquire().await?.as_mut())
            .await
    }

    /// Disables a [Pipeline] on the [Collection]
    ///
    /// # Arguments
    /// * `pipeline` - The [Pipeline] to remove
    ///
    /// # Errors
    /// * If the pipeline has not already been added to the [Collection]
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use anyhow::Result;
    /// use serde_json::json;
    /// async fn doc() -> Result<()> {
    ///     let mut collection = Collection::new("my_collection", None)?;
    ///     let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///     collection.disable_pipeline(&pipeline).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn disable_pipeline(&self, pipeline: &Pipeline) -> anyhow::Result<()> {
        // The flow for this function:
        // 1. Set ACTIVE = FALSE for the pipeline in collection.pipelines
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

    /// Upserts documents into [Collection]
    ///
    /// # Arguments
    /// * `documents` - A vector of [Json] documents to upsert
    /// * `args` - A [Json] object containing arguments for the upsert
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use anyhow::Result;
    /// use serde_json::json;
    /// async fn doc() -> Result<()> {
    ///     let mut collection = Collection::new("my_collection", None)?;
    ///     collection.upsert_documents(vec![json!({"id": "1", "name": "one"}).into()], None).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self, documents))]
    pub async fn upsert_documents(
        &mut self,
        documents: Vec<Json>,
        args: Option<Json>,
    ) -> anyhow::Result<()> {
        // The flow for this function
        // 1. Create the collection if it does not exist
        // 2. Get all pipelines where ACTIVE = TRUE
        // -> Foreach pipeline get the parsed schema
        // 4. Foreach n documents
        // -> Begin a transaction returning the old document if it existed
        // -> Insert the document
        // -> Foreach pipeline check if we need to resync the document and if so sync the document
        // -> Commit the transaction
        let mut args = args.unwrap_or_default();
        let args = args.as_object_mut().context("args must be a JSON object")?;

        self.verify_in_database(false).await?;
        let mut pipelines = self.get_pipelines().await?;

        let pool = get_or_initialize_pool(&self.database_url).await?;

        let project_info = &self.database_data.as_ref().unwrap().project_info;
        let mut parsed_schemas = vec![];
        for pipeline in &mut pipelines {
            let parsed_schema = pipeline
                .get_parsed_schema(project_info, &pool)
                .await
                .expect("Error getting parsed schema for pipeline");
            parsed_schemas.push(parsed_schema);
        }
        let pipelines: Vec<(Pipeline, HashMap<String, FieldAction>)> =
            pipelines.into_iter().zip(parsed_schemas).collect();

        let batch_size = args
            .remove("batch_size")
            .map(|x| x.try_to_u64())
            .unwrap_or(Ok(100))?;

        let parallel_batches = args
            .get("parallel_batches")
            .map(|x| x.try_to_u64())
            .unwrap_or(Ok(1))? as usize;

        let progress_bar = utils::default_progress_bar(documents.len() as u64);
        progress_bar.println("Upserting Documents...");

        let mut set = JoinSet::new();
        for batch in documents.chunks(batch_size as usize) {
            if set.len() >= parallel_batches {
                set.join_next().await.unwrap()??;
                progress_bar.inc(batch_size);
            }

            let local_self = self.clone();
            let local_batch = batch.to_owned();
            let local_args = args.clone();
            let local_pipelines = pipelines.clone();
            let local_pool = pool.clone();
            set.spawn(async move {
                local_self
                    ._upsert_documents(local_batch, local_args, local_pipelines, local_pool)
                    .await
            });
        }

        while let Some(res) = set.join_next().await {
            res??;
            progress_bar.inc(batch_size);
        }

        progress_bar.println("Done Upserting Documents\n");
        progress_bar.finish();

        Ok(())
    }

    async fn _upsert_documents(
        self,
        batch: Vec<Json>,
        args: serde_json::Map<String, Value>,
        mut pipelines: Vec<(Pipeline, HashMap<String, FieldAction>)>,
        pool: Pool<Postgres>,
    ) -> anyhow::Result<()> {
        let project_info = &self.database_data.as_ref().unwrap().project_info;

        let query = if args
            .get("merge")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false)
        {
            query_builder!(
                queries::UPSERT_DOCUMENT_AND_MERGE_METADATA,
                self.documents_table_name,
                self.documents_table_name,
                self.documents_table_name,
                self.documents_table_name
            )
        } else {
            query_builder!(
                queries::UPSERT_DOCUMENT,
                self.documents_table_name,
                self.documents_table_name,
                self.documents_table_name
            )
        };

        let mut transaction = pool.begin().await?;

        let mut query_values = String::new();
        let mut binding_parameter_counter = 1;
        for _ in 0..batch.len() {
            query_values = format!(
                "{query_values}, (${}, ${}, ${})",
                binding_parameter_counter,
                binding_parameter_counter + 1,
                binding_parameter_counter + 2
            );
            binding_parameter_counter += 3;
        }

        let query = query.replace(
            "{values_parameters}",
            &query_values.chars().skip(1).collect::<String>(),
        );
        let query = query.replace(
            "{binding_parameter}",
            &format!("${binding_parameter_counter}"),
        );

        let mut query = sqlx::query_as(&query);

        let mut source_uuids = vec![];
        for document in &batch {
            let id = document
                .get("id")
                .context("`id` must be a key in document")?
                .to_string();
            let md5_digest = md5::compute(id.as_bytes());
            let source_uuid = uuid::Uuid::from_slice(&md5_digest.0)?;
            source_uuids.push(source_uuid);

            let start = SystemTime::now();
            let timestamp = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            let versions: HashMap<String, serde_json::Value> = document
                .as_object()
                .context("document must be an object")?
                .iter()
                .try_fold(HashMap::new(), |mut acc, (key, value)| {
                    let md5_digest = md5::compute(serde_json::to_string(value)?.as_bytes());
                    let md5_digest = format!("{md5_digest:x}");
                    acc.insert(
                        key.to_owned(),
                        serde_json::json!({
                            "last_updated": timestamp,
                            "md5": md5_digest
                        }),
                    );
                    anyhow::Ok(acc)
                })?;
            let versions = serde_json::to_value(versions)?;

            query = query.bind(source_uuid).bind(document).bind(versions);
        }

        let results: Vec<(i64, Option<Json>)> = query
            .bind(source_uuids)
            .fetch_all(&mut *transaction)
            .await?;

        let dp: Vec<(i64, Json, Option<Json>)> = results
            .into_iter()
            .zip(batch)
            .map(|((id, previous_document), document)| (id, document.to_owned(), previous_document))
            .collect();

        for (pipeline, parsed_schema) in &mut pipelines {
            let ids_to_run_on: Vec<i64> = dp
                .iter()
                .filter(|(_, document, previous_document)| match previous_document {
                    Some(previous_document) => parsed_schema
                        .iter()
                        .any(|(key, _)| document[key] != previous_document[key]),
                    None => true,
                })
                .map(|(document_id, _, _)| *document_id)
                .collect();
            if !ids_to_run_on.is_empty() {
                pipeline
                    .sync_documents(ids_to_run_on, project_info, &mut transaction)
                    .await
                    .expect("Failed to execute pipeline");
            }
        }

        transaction.commit().await?;
        Ok(())
    }

    /// Gets the documents on a [Collection]
    ///
    /// # Arguments
    ///
    /// * `args` - A JSON object containing the following keys:
    ///   * `limit` - The maximum number of documents to return. Defaults to 1000.
    ///   * `order_by` - A JSON array of objects that specify the order of the documents to return.
    ///     Each object must have a `field` key with the name of the field to order by, and a `direction`
    ///     key with the value `asc` or `desc`.
    ///   * `last_row_id` - The id of the last document returned
    ///   * `offset` - The number of documents to skip before returning results
    ///   * `filter` - A JSON object specifying the filter to apply to the documents
    ///   * `keys` - a JSON array specifying the document keys to return
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///     let collection = Collection::new("my_collection", None)?;
    ///     let documents = collection.get_documents(Some(json!({
    ///         "limit": 2,
    ///     }).into()));
    ///     Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn get_documents(&self, args: Option<Json>) -> anyhow::Result<Vec<Json>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let mut args = args.unwrap_or_default();
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
            .columns([
                SIden::Str("id"),
                SIden::Str("created_at"),
                SIden::Str("source_uuid"),
                SIden::Str("version"),
            ])
            .limit(limit);

        if let Some(keys) = args.remove("keys") {
            let document_queries = keys
                .as_array()
                .context("`keys` must be an array")?
                .iter()
                .map(|d| {
                    let key = d.as_str().context("`key` value must be a string")?;
                    anyhow::Ok(format!("'{key}', document #> '{{{key}}}'"))
                })
                .collect::<anyhow::Result<Vec<String>>>()?
                .join(",");
            query.expr_as(
                Expr::cust(format!("jsonb_build_object({document_queries})")),
                Alias::new("document"),
            );
        } else {
            query.column(SIden::Str("document"));
        }

        if let Some(order_by) = args.remove("order_by") {
            let order_by_builder =
                order_by_builder::OrderByBuilder::new(order_by, "documents", "document").build()?;
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

        if let Some(filter) = args.remove("filter") {
            let filter = FilterBuilder::new(filter, "documents", "document").build()?;
            query.cond_where(filter);
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
    /// * `filter` - A JSON object specifying the filter to apply to the documents.
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let collection = Collection::new("my_collection", None)?;
    ///    collection.delete_documents(json!({
    ///        "id": {
    ///            "$eq": 1
    ///        }
    ///    }).into());
    ///    Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn delete_documents(&self, filter: Json) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let mut query = Query::delete();
        query.from_table(self.documents_table_name.to_table_tuple());

        let filter = FilterBuilder::new(filter.0, "documents", "document").build()?;
        query.cond_where(filter);

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).fetch_all(&pool).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    /// Performs search over the documents in a [Collection]
    ///
    /// # Arguments
    ///
    /// * `query` - A JSON object specifying the query to perform.
    /// * `pipeline` - The [Pipeline] to use for the search.
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///    let results = collection.search(json!({
    ///        "query": {
    ///            "semantic_search": {
    ///                "title": {
    ///                    "query": "This is a an example query string",
    ///                },
    ///            }
    ///        }
    ///    }).into(), &mut pipeline).await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn search(&mut self, query: Json, pipeline: &mut Pipeline) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let (built_query, values) = build_search_query(self, query.clone(), pipeline).await?;
        let results: Result<(Json,), _> = sqlx::query_as_with(&built_query, values)
            .fetch_one(&pool)
            .await;

        match results {
            Ok(r) => {
                let mut results = r.0;
                if results["results"].is_null() {
                    results["results"] = json!([]);
                }
                Ok(results)
            }
            Err(e) => match e.as_database_error() {
                Some(d) => {
                    if d.code() == Some(Cow::from("XX000")) {
                        self.verify_in_database(false).await?;
                        let project_info = &self.database_data.as_ref().unwrap().project_info;
                        pipeline
                            .verify_in_database(project_info, false, &pool)
                            .await?;
                        let (built_query, values) =
                            build_search_query(self, query, pipeline).await?;
                        let results: (Json,) = sqlx::query_as_with(&built_query, values)
                            .fetch_one(&pool)
                            .await?;
                        let mut results = results.0;
                        if results["results"].is_null() {
                            results["results"] = json!([]);
                        }
                        Ok(results)
                    } else {
                        Err(anyhow::anyhow!(e))
                    }
                }
                None => Err(anyhow::anyhow!(e)),
            },
        }
    }

    #[instrument(skip(self))]
    /// Same as search but the [Collection] is not mutable. This will not work with [Pipeline]s that use remote embeddings
    pub async fn search_local(&self, query: Json, pipeline: &Pipeline) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let (built_query, values) = build_search_query(self, query.clone(), pipeline).await?;
        let results: (Json,) = sqlx::query_as_with(&built_query, values)
            .fetch_one(&pool)
            .await?;
        let mut results = results.0;
        if results["results"].is_null() {
            results["results"] = json!([]);
        }
        Ok(results)
    }

    /// Adds a search event to the database
    ///
    /// # Arguments
    ///
    /// * `search_id` - The id of the search
    /// * `search_result` - The index of the search result
    /// * `event` - The event to add
    /// * `pipeline` - The [Pipeline] used for the search
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///    collection.add_search_event(1, 1, json!({
    ///        "event": "click",
    ///    }).into(), &mut pipeline).await?;
    ///    Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn add_search_event(
        &self,
        search_id: i64,
        search_result: i64,
        event: Json,
        pipeline: &Pipeline,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let search_events_table = format!("{}_{}.search_events", self.name, pipeline.name);
        let search_results_table = format!("{}_{}.search_results", self.name, pipeline.name);

        let query = query_builder!(
            queries::INSERT_SEARCH_EVENT,
            search_events_table,
            search_results_table
        );
        debug_sqlx_query!(
            INSERT_SEARCH_EVENT,
            query,
            search_id,
            search_result,
            event.0
        );
        sqlx::query(&query)
            .bind(search_id)
            .bind(search_result)
            .bind(event.0)
            .execute(&pool)
            .await?;
        Ok(())
    }

    /// Performs vector search on the [Collection]
    ///
    /// # Arguments
    /// * `query` - The query to search for
    /// * `pipeline` - The [Pipeline] to use for the search
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///    let results = collection.vector_search(json!({
    ///        "query": {
    ///            "fields": {
    ///                "title": {
    ///                    "query": "This is an example query string"
    ///                }
    ///             }
    ///        }
    ///    }).into(), &mut pipeline).await?;
    ///    Ok(())
    /// }
    #[allow(clippy::type_complexity)]
    #[instrument(skip(self))]
    pub async fn vector_search(
        &mut self,
        query: Json,
        pipeline: &mut Pipeline,
    ) -> anyhow::Result<Vec<Json>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let (built_query, values) =
            build_vector_search_query(query.clone(), self, pipeline).await?;
        let results: Result<Vec<(Json, String, f64, Option<f64>)>, _> =
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
                        "score": v.2,
                        "rerank_score": v.3
                    })
                    .into()
                })
                .collect()),
            Err(e) => match e.as_database_error() {
                Some(d) => {
                    if d.code() == Some(Cow::from("XX000")) {
                        self.verify_in_database(false).await?;
                        let project_info = &self.database_data.as_ref().unwrap().project_info;
                        pipeline
                            .verify_in_database(project_info, false, &pool)
                            .await?;
                        let (built_query, values) =
                            build_vector_search_query(query, self, pipeline).await?;
                        let results: Vec<(Json, String, f64, Option<f64>)> =
                            sqlx::query_as_with(&built_query, values)
                                .fetch_all(&pool)
                                .await?;
                        Ok(results
                            .into_iter()
                            .map(|v| {
                                serde_json::json!({
                                    "document": v.0,
                                    "chunk": v.1,
                                    "score": v.2,
                                    "rerank_score": v.3
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

    /// Same as vector_search but assumes embeddings are done locally
    #[instrument(skip(self))]
    pub async fn vector_search_local(
        &self,
        query: Json,
        pipeline: &Pipeline,
    ) -> anyhow::Result<Vec<Json>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let (built_query, values) =
            build_vector_search_query(query.clone(), self, pipeline).await?;
        let results: Vec<(Json, String, f64, Option<f64>)> =
            sqlx::query_as_with(&built_query, values)
                .fetch_all(&pool)
                .await?;
        Ok(results
            .into_iter()
            .map(|v| {
                serde_json::json!({
                    "document": v.0,
                    "chunk": v.1,
                    "score": v.2,
                    "rerank_score": v.3
                })
                .into()
            })
            .collect())
    }

    /// Performs rag on the [Collection]
    ///
    /// # Arguments
    /// * `query` - The query to search for
    /// * `pipeline` - The [Pipeline] to use for the search
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///    let results = collection.rag(json!({
    ///       "CONTEXT": {
    ///           "vector_search": {
    ///               "query": {
    ///                   "fields": {
    ///                       "body": {
    ///                           "query": "Test document: 2",
    ///                           "parameters": {
    ///                               "prompt": "query: "
    ///                           }
    ///                       },
    ///                   },
    ///               },
    ///               "document": {
    ///                   "keys": [
    ///                       "id"
    ///                   ]
    ///               },
    ///               "limit": 2
    ///           },
    ///           "aggregate": {
    ///             "join": "\n"
    ///           }
    ///       },
    ///       "CUSTOM": {
    ///           "sql": "SELECT 'test'"
    ///       },
    ///       "chat": {
    ///           "model": "meta-llama/Meta-Llama-3-8B-Instruct",
    ///           "messages": [
    ///               {
    ///                   "role": "system",
    ///                   "content": "You are a friendly and helpful chatbot"
    ///               },
    ///               {
    ///                   "role": "user",
    ///                   "content": "Some text with {CONTEXT} - {CUSTOM}",
    ///               }
    ///           ],
    ///           "max_tokens": 10
    ///       }
    ///    }).into(), &mut pipeline).await?;
    ///    Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn rag(&self, query: Json, pipeline: &mut Pipeline) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let (built_query, values) = build_rag_query(query.clone(), self, pipeline, false).await?;
        let mut results: Vec<(Json,)> = sqlx::query_as_with(&built_query, values)
            .fetch_all(&pool)
            .await?;
        Ok(std::mem::take(&mut results[0].0))
    }

    /// Same as rag buit returns a stream of results
    #[instrument(skip(self))]
    pub async fn rag_stream(
        &self,
        query: Json,
        pipeline: &mut Pipeline,
    ) -> anyhow::Result<RAGStream> {
        let pool = get_or_initialize_pool(&self.database_url).await?;

        let (built_query, values) = build_rag_query(query.clone(), self, pipeline, true).await?;

        let mut transaction = pool.begin().await?;

        sqlx::query_with(&built_query, values)
            .execute(&mut *transaction)
            .await?;

        let s = futures::stream::try_unfold(transaction, move |mut transaction| async move {
            let mut res: Vec<Json> = sqlx::query_scalar("FETCH 1 FROM c")
                .fetch_all(&mut *transaction)
                .await?;
            if !res.is_empty() {
                Ok(Some((std::mem::take(&mut res[0]), transaction)))
            } else {
                transaction.commit().await?;
                Ok(None)
            }
        });

        Ok(RAGStream {
            general_json_async_iterator: Some(GeneralJsonAsyncIterator(Box::pin(s))),
            sources: serde_json::json!({}).into(),
        })
    }

    /// Archives a [Collection]
    /// This will free up the name to be reused. It does not delete it.
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    collection.archive().await?;
    ///    Ok(())
    /// }
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

    /// A legacy query builder.
    #[deprecated(since = "1.0.0", note = "please use `vector_search` instead")]
    #[instrument(skip(self))]
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(self.clone())
    }

    /// Gets all pipelines for the [Collection]
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let pipelines = collection.get_pipelines().await?;
    ///    Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn get_pipelines(&mut self) -> anyhow::Result<Vec<Pipeline>> {
        self.verify_in_database(false).await?;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let pipelines: Vec<models::Pipeline> = sqlx::query_as(&query_builder!(
            "SELECT * FROM %s WHERE active = TRUE",
            self.pipelines_table_name
        ))
        .fetch_all(&pool)
        .await?;
        pipelines.into_iter().map(|p| p.try_into()).collect()
    }

    /// Gets a [Pipeline] by name
    ///
    /// # Arguments
    /// * `name` - The name of the [Pipeline]
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let pipeline = collection.get_pipeline("my_pipeline").await?;
    ///    Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn get_pipeline(&mut self, name: &str) -> anyhow::Result<Pipeline> {
        self.verify_in_database(false).await?;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let pipeline: models::Pipeline = sqlx::query_as(&query_builder!(
            "SELECT * FROM %s WHERE name = $1 AND active = TRUE LIMIT 1",
            self.pipelines_table_name
        ))
        .bind(name)
        .fetch_one(&pool)
        .await?;
        pipeline.try_into()
    }

    /// Check if the [Collection] exists in the database
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let exists = collection.exists().await?;
    ///    Ok(())
    /// }
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

    /// Upsert all files in a directory that match the file_types
    ///
    /// # Arguments
    /// * `path` - The path to the directory to upsert
    /// * `args` - A [Json](serde_json::Value) object with the following keys:
    ///   * `file_types` - An array of file extensions to match. E.G. ['md', 'txt']
    ///   * `file_batch_size` - The number of files to upsert at a time. Defaults to 10.
    ///   * `follow_links` - Whether to follow symlinks. Defaults to false.
    ///   * `ignore_paths` - An array of regexes to ignore. E.G. ['.*ignore.*']
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use serde_json::json;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    collection.upsert_directory("/path/to/my/files", json!({
    ///        "file_types": ["md", "txt"]
    ///    }).into()).await?;
    ///    Ok(())
    /// }
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

    /// Gets the sync status of a [Pipeline]
    ///
    /// # Arguments
    /// * `pipeline` - The [Pipeline] to get the sync status of
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///    let status = collection.get_pipeline_status(&mut pipeline).await?;
    ///    Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn get_pipeline_status(&mut self, pipeline: &mut Pipeline) -> anyhow::Result<Json> {
        self.verify_in_database(false).await?;
        let project_info = &self.database_data.as_ref().unwrap().project_info;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        pipeline.get_status(project_info, &pool).await
    }

    #[instrument(skip(self))]
    /// Generates a PlantUML ER Diagram for a [Collection] and [Pipeline] tables
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use pgml::Pipeline;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    let mut pipeline = Pipeline::new("my_pipeline", None)?;
    ///    let er_diagram = collection.generate_er_diagram(&mut pipeline).await?;
    ///    Ok(())
    /// }
    #[instrument(skip(self))]
    pub async fn generate_er_diagram(&mut self, pipeline: &mut Pipeline) -> anyhow::Result<String> {
        self.verify_in_database(false).await?;
        let project_info = &self.database_data.as_ref().unwrap().project_info;
        let pool = get_or_initialize_pool(&self.database_url).await?;
        pipeline
            .verify_in_database(project_info, false, &pool)
            .await?;

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

            if let Some(_embed_action) = &field_action.semantic_search {
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

    /// Upserts a file into a [Collection]
    ///
    /// # Arguments
    /// * `path` - The path to the file to upsert
    ///
    /// # Example
    /// ```
    /// use pgml::Collection;
    /// use anyhow::Result;
    /// async fn run() -> anyhow::Result<()> {
    ///    let mut collection = Collection::new("my_collection", None)?;
    ///    collection.upsert_file("my_file.txt").await?;
    ///    Ok(())
    /// }
    #[instrument(skip(self))]
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

    fn generate_table_names(name: &str) -> (String, String) {
        [".pipelines", ".documents"]
            .into_iter()
            .map(|s| format!("{}{}", name, s))
            .collect_tuple()
            .unwrap()
    }
}
