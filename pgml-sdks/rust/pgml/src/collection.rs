use anyhow::Context;
use indicatif::MultiProgress;
use itertools::Itertools;
use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::PgPool;
use sqlx::Executor;
use sqlx::PgConnection;
use std::borrow::Cow;
use std::time::SystemTime;
use tracing::{instrument, warn};

use crate::get_or_initialize_pool;
use crate::model::ModelRuntime;
use crate::models;
use crate::pipeline::Pipeline;
use crate::queries;
use crate::query_builder;
use crate::query_builder::QueryBuilder;
use crate::remote_embeddings::build_remote_embeddings;
use crate::splitter::Splitter;
use crate::types::DateTime;
use crate::types::Json;
use crate::utils;

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

#[cfg(feature = "python")]
use crate::{languages::CustomInto, pipeline::PipelinePython, query_builder::QueryBuilderPython};

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

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub id: i64,
    pub name: String,
    pub task: ProjectTask,
    pub database_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CollectionDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
    pub project_info: ProjectInfo,
}

/// A collection of documents
#[derive(custom_derive, Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub database_url: Option<String>,
    pub pipelines_table_name: String,
    pub documents_table_name: String,
    pub transforms_table_name: String,
    pub chunks_table_name: String,
    pub documents_tsvectors_table_name: String,
    pub database_data: Option<CollectionDatabaseData>,
}

#[custom_methods(
    new,
    upsert_documents,
    get_documents,
    add_pipeline,
    remove_pipeline,
    enable_pipeline,
    disable_pipeline,
    vector_search,
    query,
    exists,
    archive
)]
impl Collection {
    /// Creates a new collection
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
    pub async fn verify_in_database(&mut self, throw_if_exists: bool) -> anyhow::Result<()> {
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
                    .fetch_one(&mut transaction)
                    .await?;

                transaction
                    .execute(query_builder!("CREATE SCHEMA IF NOT EXISTS %s", self.name).as_str())
                    .await?;

                let c: models::Collection = sqlx::query_as("INSERT INTO pgml.collections (name, project_id) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING RETURNING *")
                        .bind(&self.name)
                        .bind(project_id)
                        .fetch_one(&mut transaction)
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
            .execute(&pool)
            .await?;

        // Drop the embeddings table
        sqlx::query(&query_builder!(
            "DROP TABLE IF EXISTS %s",
            embeddings_table_name
        ))
        .execute(&mut transaction)
        .await?;

        // Need to delete from the tsvectors table only if no other pipelines use the
        // same tsvector configuration
        sqlx::query(&query_builder!(
                    "DELETE FROM %s WHERE configuration = $1 AND NOT EXISTS (SELECT 1 FROM %s WHERE parameters->'full_text_search'->>'configuration' = $1 AND id != $2)", 
                    self.documents_tsvectors_table_name,
                    self.pipelines_table_name))
                .bind(parameters["full_text_search"]["configuration"].as_str())
                .bind(database_data.id)
                .execute(&mut transaction)
                .await?;

        sqlx::query(&query_builder!(
            "DELETE FROM %s WHERE id = $1",
            self.pipelines_table_name
        ))
        .bind(database_data.id)
        .execute(&mut transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

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
    /// * `documents` - A vector of documents to upsert.
    /// * `text_key` - The key in the document that contains the text.
    /// * `id_key` - The key in the document that contains the id.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use pgml::Database;
    /// use pgml::types::Json;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents: Vec<Json> = vec![
    ///        serde_json::json!( {
    ///            "id": 1,
    ///            "text": "This is a document"
    ///        })
    ///        .into()
    ///    ];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    Ok(())
    /// }
    /// ```
    ///
    // We skip documents because it may be very large
    #[instrument(skip(self, documents))]
    pub async fn upsert_documents(
        &mut self,
        documents: Vec<Json>,
        strict: Option<bool>,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(false).await?;

        let strict = strict.unwrap_or(true);

        let progress_bar = utils::default_progress_bar(documents.len() as u64);
        progress_bar.println("Upserting Documents...");

        let documents: anyhow::Result<Vec<_>> = documents.into_iter().map(|mut document| {
            let document = document
                .as_object_mut()
                .expect("Documents must be a vector of objects");
            let text = match document.remove("text") {
                Some(t) => t,
                None => {
                    if strict {
                        anyhow::bail!("`text` is not a key in document, throwing error. To supress this error, pass strict: false");
                    } else {
                        warn!("`text` is not a key in document, skipping document. To throw an error instead, pass strict: true");
                    }
                    return Ok(None)
                }
            };
            let text = text.as_str().context("`text` must be a string")?.to_string();

            // We don't want the text included in the document metadata, but everything else
            // should be in there
            let metadata = serde_json::to_value(&document)?.into();

            let md5_digest = match document.get("id") {
                Some(k) => md5::compute(k.to_string().as_bytes()),
                None => {
                    if strict {
                        anyhow::bail!("`id` is not a key in document, throwing error. To supress this error, pass strict: false");
                    } else {
                        warn!("`id` is not a key in document, skipping document. To throw an error instead, pass strict: true");
                    }
                    return Ok(None)
                }
            };
            let source_uuid = uuid::Uuid::from_slice(&md5_digest.0)?;

            Ok(Some((source_uuid, text, metadata)))
        }).collect();

        // We could continue chaining the above iterators but types become super annoying to
        // deal with, especially because we are dealing with async functions. This is much easier to read
        // Also, we may want to use a variant of chunks that is owned, I'm not 100% sure of what
        // cloning happens when passing values into sqlx bind. itertools variants will not work as
        // it is not thread safe and pyo3 will get upset
        let mut document_ids = Vec::new();
        for chunk in documents?.chunks(10) {
            // We want the length before we filter out any None values
            let chunk_len = chunk.len();
            // Filter out the None values
            let chunk: Vec<&(uuid::Uuid, String, Json)> =
                chunk.iter().filter_map(|x| x.as_ref()).collect();

            // Make sure we didn't filter everything out
            if chunk.is_empty() {
                progress_bar.inc(chunk_len as u64);
                continue;
            }

            let mut transaction = pool.begin().await?;
            // First delete any documents that already have the same UUID then insert the new ones.
            // We are essentially upserting in two steps
            sqlx::query(&query_builder!(
                "DELETE FROM %s WHERE source_uuid IN (SELECT source_uuid FROM %s WHERE source_uuid = ANY($1::uuid[]))",
                self.documents_table_name,
                self.documents_table_name
            )).
                bind(&chunk.iter().map(|(source_uuid, _, _)| *source_uuid).collect::<Vec<_>>()).
                execute(&mut transaction).await?;
            let query_string_values = (0..chunk.len())
                .map(|i| format!("(${}, ${}, ${})", i * 3 + 1, i * 3 + 2, i * 3 + 3))
                .collect::<Vec<String>>()
                .join(",");
            let query_string = format!(
                "INSERT INTO %s (source_uuid, text, metadata) VALUES {} ON CONFLICT (source_uuid) DO UPDATE SET text = $2, metadata = $3 RETURNING id",
                query_string_values
            );
            let query = query_builder!(query_string, self.documents_table_name);
            let mut query = sqlx::query_scalar(&query);
            for (source_uuid, text, metadata) in chunk.into_iter() {
                query = query.bind(source_uuid).bind(text).bind(metadata);
            }

            let ids: Vec<i64> = query.fetch_all(&mut transaction).await?;
            document_ids.extend(ids);
            progress_bar.inc(chunk_len as u64);
            transaction.commit().await?;
        }
        progress_bar.finish();
        eprintln!("Done Upserting Documents\n");

        self.sync_pipelines(Some(document_ids)).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_documents(
        &mut self,
        last_id: Option<i64>,
        limit: Option<i64>,
    ) -> anyhow::Result<Vec<Json>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let documents: Vec<models::Document> = sqlx::query_as(&query_builder!(
            "SELECT * FROM %s WHERE id > $1 ORDER BY id ASC LIMIT $2",
            self.documents_table_name
        ))
        .bind(last_id.unwrap_or(0))
        .bind(limit.unwrap_or(100))
        .fetch_all(&pool)
        .await?;
        documents
            .into_iter()
            .map(|d| {
                serde_json::to_value(d)
                    .map(|t| t.into())
                    .map_err(|e| anyhow::anyhow!(e))
            })
            .collect()
    }

    #[instrument(skip(self))]
    pub async fn sync_pipelines(&mut self, document_ids: Option<Vec<i64>>) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let pipelines = self.get_pipelines().await?;
        if !pipelines.is_empty() {
            let mp = MultiProgress::new();
            mp.println("Syncing Pipelines...")?;
            for mut pipeline in pipelines {
                pipeline.execute(&document_ids, mp.clone()).await?;
            }
            eprintln!("Done Syncing Pipelines\n");
        }
        Ok(())
    }

    /// Performs vector search on the [Collection]
    ///
    /// # Arguments
    ///
    /// * `query` - The query to search for.
    /// * `query_params` - A [std::collections::HashMap] of parameters for the model used in the
    /// query.
    /// * `top_k` - How many results to limit on.
    /// * `model_id` - The id of the [models::Model] to use.
    /// * `splitter_id` - The id of the [models::Splitter] to use.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use pgml::Database;
    /// use pgml::types::Json;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents: Vec<Json> = vec![
    ///        serde_json::json!( {
    ///            "id": 1,
    ///            "text": "This is a document"
    ///        })
    ///        .into()
    ///    ];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    collection.generate_chunks(None).await?;
    ///    collection.generate_embeddings(None, None).await?;
    ///    let results = collection
    ///        .vector_search("Here is a test", None, None, None, None)
    ///        .await
    ///        .unwrap();
    ///    Ok(())
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
        sqlx::query("UPDATE pgml.collections SET name = $1, active = FALSE where name = $2")
            .bind(&archive_table_name)
            .bind(&self.name)
            .execute(&pool)
            .await?;
        sqlx::query(&query_builder!(
            "ALTER SCHEMA %s RENAME TO %s",
            &self.name,
            archive_table_name
        ))
        .execute(&pool)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(self.clone())
    }

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
              INNER JOIN pgml.sdk_splitters s ON p.splitter_id = s.id 
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

    #[instrument(skip(self))]
    pub async fn get_project_info(&mut self) -> anyhow::Result<ProjectInfo> {
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
    /// ```
    /// async fn example() -> anyhow::Result<()> {
    ///   let builtins = Builtins::new(None);
    ///   let collection_exists = builtins.does_collection_exist("collection number 1").await?;
    ///   // Do stuff with your new found information
    ///   Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;

    #[sqlx::test]
    async fn can_upsert_documents() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cud_2", None);

        // Test basic upsert
        let documents = vec![
            serde_json::json!({"id": 1, "text": "hello world"}).into(),
            serde_json::json!({"text": "hello world"}).into(),
        ];
        collection
            .upsert_documents(documents.clone(), Some(false))
            .await?;
        let document = &collection.get_documents(None, Some(1)).await?[0];
        assert_eq!(document["text"], "hello world");

        // Test strictness
        assert!(collection
            .upsert_documents(documents, Some(true))
            .await
            .is_err());

        // Test upsert
        let documents = vec![
            serde_json::json!({"id": 1, "text": "hello world 2"}).into(),
            serde_json::json!({"text": "hello world"}).into(),
        ];
        collection
            .upsert_documents(documents.clone(), Some(false))
            .await?;
        let document = &collection.get_documents(None, Some(1)).await?[0];
        assert_eq!(document["text"], "hello world 2");
        collection.archive().await?;
        Ok(())
    }
}
