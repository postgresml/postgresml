use anyhow::Context;
use indicatif::MultiProgress;
use rust_bridge::{alias, alias_manual, alias_methods};
use sqlx::{Executor, PgConnection, PgPool};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use tokio::join;
use tracing::instrument;

use crate::{
    collection::ProjectInfo,
    get_or_initialize_pool,
    model::{Model, ModelRuntime},
    models, queries, query_builder,
    remote_embeddings::build_remote_embeddings,
    splitter::Splitter,
    types::{DateTime, Json, TryToNumeric},
    utils,
};

#[cfg(feature = "python")]
use crate::{model::ModelPython, splitter::SplitterPython, types::JsonPython};

#[derive(Debug, Clone)]
pub struct InvividualSyncStatus {
    pub synced: i64,
    pub not_synced: i64,
    pub total: i64,
}

impl From<InvividualSyncStatus> for Json {
    fn from(value: InvividualSyncStatus) -> Self {
        serde_json::json!({
            "synced": value.synced,
            "not_synced": value.not_synced,
            "total": value.total,
        })
        .into()
    }
}

impl From<Json> for InvividualSyncStatus {
    fn from(value: Json) -> Self {
        Self {
            synced: value["synced"]
                .as_i64()
                .expect("The synced field is not an integer"),
            not_synced: value["not_synced"]
                .as_i64()
                .expect("The not_synced field is not an integer"),
            total: value["total"]
                .as_i64()
                .expect("The total field is not an integer"),
        }
    }
}

#[derive(alias_manual, Debug, Clone)]
pub struct PipelineSyncData {
    pub chunks_status: InvividualSyncStatus,
    pub embeddings_status: InvividualSyncStatus,
    pub tsvectors_status: InvividualSyncStatus,
}

impl From<PipelineSyncData> for Json {
    fn from(value: PipelineSyncData) -> Self {
        serde_json::json!({
            "chunks_status": *Json::from(value.chunks_status),
            "embeddings_status": *Json::from(value.embeddings_status),
            "tsvectors_status": *Json::from(value.tsvectors_status),
        })
        .into()
    }
}

impl From<Json> for PipelineSyncData {
    fn from(mut value: Json) -> Self {
        Self {
            chunks_status: Json::from(std::mem::take(&mut value["chunks_status"])).into(),
            embeddings_status: Json::from(std::mem::take(&mut value["embeddings_status"])).into(),
            tsvectors_status: Json::from(std::mem::take(&mut value["tsvectors_status"])).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
    pub model_id: i64,
    pub splitter_id: i64,
}

/// A pipeline that processes documents
#[derive(alias, Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub model: Option<Model>,
    pub splitter: Option<Splitter>,
    pub parameters: Option<Json>,
    project_info: Option<ProjectInfo>,
    pub(crate) database_data: Option<PipelineDatabaseData>,
}

#[alias_methods(new, get_status, to_dict)]
impl Pipeline {
    /// Creates a new [Pipeline]
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the pipeline
    /// * `model` - The pipeline [Model]
    /// * `splitter` - The pipeline [Splitter]
    /// * `parameters` - The parameters to the pipeline. Defaults to None
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Pipeline, Model, Splitter};
    /// let model = Model::new(None, None, None);
    /// let splitter = Splitter::new(None, None);
    /// let pipeline = Pipeline::new("my_splitter", Some(model), Some(splitter), None);
    /// ```
    pub fn new(
        name: &str,
        model: Option<Model>,
        splitter: Option<Splitter>,
        parameters: Option<Json>,
    ) -> Self {
        let parameters = Some(parameters.unwrap_or_default());
        Self {
            name: name.to_string(),
            model,
            splitter,
            parameters,
            project_info: None,
            database_data: None,
        }
    }

    /// Gets the status of the [Pipeline]
    /// This includes the status of the chunks, embeddings, and tsvectors
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let mut collection = Collection::new("my_collection", None);
    ///     let mut pipeline = collection.get_pipeline("my_pipeline").await?;
    ///     let status = pipeline.get_status().await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_status(&mut self) -> anyhow::Result<PipelineSyncData> {
        let pool = self.get_pool().await?;

        self.verify_in_database(false).await?;
        let embeddings_table_name = self.create_or_get_embeddings_table().await?;

        let database_data = self
            .database_data
            .as_ref()
            .context("Pipeline must be verified to get status")?;

        let parameters = self
            .parameters
            .as_ref()
            .context("Pipeline must be verified to get status")?;

        let project_name = &self.project_info.as_ref().unwrap().name;

        // TODO: Maybe combine all of these into one query so it is faster
        let chunks_status: (Option<i64>, Option<i64>) = sqlx::query_as(&query_builder!(
            "SELECT (SELECT COUNT(DISTINCT document_id) FROM %s WHERE splitter_id = $1), COUNT(id) FROM %s",
            format!("{}.chunks", project_name),
            format!("{}.documents", project_name)
        ))
        .bind(database_data.splitter_id)
        .fetch_one(&pool).await?;
        let chunks_status = InvividualSyncStatus {
            synced: chunks_status.0.unwrap_or(0),
            not_synced: chunks_status.1.unwrap_or(0) - chunks_status.0.unwrap_or(0),
            total: chunks_status.1.unwrap_or(0),
        };

        let embeddings_status: (Option<i64>, Option<i64>) = sqlx::query_as(&query_builder!(
            "SELECT (SELECT count(*) FROM %s), (SELECT count(*) FROM %s WHERE splitter_id = $1)",
            embeddings_table_name,
            format!("{}.chunks", project_name)
        ))
        .bind(database_data.splitter_id)
        .fetch_one(&pool)
        .await?;
        let embeddings_status = InvividualSyncStatus {
            synced: embeddings_status.0.unwrap_or(0),
            not_synced: embeddings_status.1.unwrap_or(0) - embeddings_status.0.unwrap_or(0),
            total: embeddings_status.1.unwrap_or(0),
        };

        let tsvectors_status = if parameters["full_text_search"]["active"]
            == serde_json::Value::Bool(true)
        {
            sqlx::query_as(&query_builder!(
                "SELECT (SELECT COUNT(*) FROM %s WHERE configuration = $1), (SELECT COUNT(*) FROM %s)",
                format!("{}.documents_tsvectors", project_name),
                format!("{}.documents", project_name)
            ))
            .bind(parameters["full_text_search"]["configuration"].as_str())
            .fetch_one(&pool).await?
        } else {
            (Some(0), Some(0))
        };
        let tsvectors_status = InvividualSyncStatus {
            synced: tsvectors_status.0.unwrap_or(0),
            not_synced: tsvectors_status.1.unwrap_or(0) - tsvectors_status.0.unwrap_or(0),
            total: tsvectors_status.1.unwrap_or(0),
        };

        Ok(PipelineSyncData {
            chunks_status,
            embeddings_status,
            tsvectors_status,
        })
    }

    #[instrument(skip(self))]
    pub(crate) async fn verify_in_database(&mut self, throw_if_exists: bool) -> anyhow::Result<()> {
        if self.database_data.is_none() {
            let pool = self.get_pool().await?;

            let project_info = self
                .project_info
                .as_ref()
                .expect("Cannot verify pipeline without project info");

            let pipeline: Option<models::Pipeline> = sqlx::query_as(&query_builder!(
                "SELECT * FROM %s WHERE name = $1",
                format!("{}.pipelines", project_info.name)
            ))
            .bind(&self.name)
            .fetch_optional(&pool)
            .await?;

            let pipeline = if let Some(p) = pipeline {
                if throw_if_exists {
                    anyhow::bail!("Pipeline {} already exists", p.name);
                }
                let model: models::Model = sqlx::query_as(
                    "SELECT id, created_at, runtime::TEXT, hyperparams FROM pgml.models WHERE id = $1",
                )
                .bind(p.model_id)
                .fetch_one(&pool)
                .await?;
                let mut model: Model = model.into();
                model.set_project_info(project_info.clone());
                self.model = Some(model);

                let splitter: models::Splitter =
                    sqlx::query_as("SELECT * FROM pgml.splitters WHERE id = $1")
                        .bind(p.splitter_id)
                        .fetch_one(&pool)
                        .await?;
                let mut splitter: Splitter = splitter.into();
                splitter.set_project_info(project_info.clone());
                self.splitter = Some(splitter);

                p
            } else {
                let model = self
                    .model
                    .as_mut()
                    .expect("Cannot save pipeline without model");
                model.set_project_info(project_info.clone());
                model.verify_in_database(false).await?;

                let splitter = self
                    .splitter
                    .as_mut()
                    .expect("Cannot save pipeline without splitter");
                splitter.set_project_info(project_info.clone());
                splitter.verify_in_database(false).await?;

                sqlx::query_as(&query_builder!(
                        "INSERT INTO %s (name, model_id, splitter_id, parameters) VALUES ($1, $2, $3, $4) RETURNING *",
                        format!("{}.pipelines", project_info.name)
                    ))
                    .bind(&self.name)
                    .bind(
                        model
                            .database_data
                            .as_ref()
                            .context("Cannot save pipeline without model")?
                            .id,
                    )
                    .bind(
                        splitter
                            .database_data
                            .as_ref()
                            .context("Cannot save pipeline without splitter")?
                            .id,
                    )
                    .bind(&self.parameters)
                    .fetch_one(&pool)
                    .await?
            };

            self.database_data = Some(PipelineDatabaseData {
                id: pipeline.id,
                created_at: pipeline.created_at,
                model_id: pipeline.model_id,
                splitter_id: pipeline.splitter_id,
            });
            self.parameters = Some(pipeline.parameters);
        }
        Ok(())
    }

    #[instrument(skip(self, mp))]
    pub(crate) async fn execute(
        &mut self,
        document_ids: &Option<Vec<i64>>,
        mp: MultiProgress,
    ) -> anyhow::Result<()> {
        // TODO: Chunk document_ids if there are too many

        // A couple notes on the following methods
        // - Atomic bools are required to work nicely with pyo3 otherwise we would use cells
        // - We use green threads because they are cheap, but we want to be super careful to not
        // return an error before stopping the green thread. To meet that end, we map errors and
        // return types often
        let chunk_ids = self.sync_chunks(document_ids, &mp).await?;
        self.sync_embeddings(chunk_ids, &mp).await?;
        self.sync_tsvectors(document_ids, &mp).await?;
        Ok(())
    }

    #[instrument(skip(self, mp))]
    async fn sync_chunks(
        &mut self,
        document_ids: &Option<Vec<i64>>,
        mp: &MultiProgress,
    ) -> anyhow::Result<Option<Vec<i64>>> {
        self.verify_in_database(false).await?;
        let pool = self.get_pool().await?;

        let database_data = self
            .database_data
            .as_mut()
            .context("Pipeline must be verified to generate chunks")?;

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to generate chunks")?;

        let progress_bar = mp
            .add(utils::default_progress_spinner(1))
            .with_prefix(self.name.clone())
            .with_message("generating chunks");

        // This part is a bit tricky
        // We want to return the ids for all chunks we inserted OR would have inserted if they didn't already exist
        // The query is structured in such a way to not insert any chunks that already exist so we
        // can't rely on the data returned from the inset queries, we need to query the chunks table
        // It is important we return the ids for chunks we would have inserted if they didn't already exist so we are robust to random crashes
        let is_done = AtomicBool::new(false);
        let work = async {
            let chunk_ids: Result<Option<Vec<i64>>, _> = if document_ids.is_some() {
                sqlx::query(&query_builder!(
                    queries::GENERATE_CHUNKS_FOR_DOCUMENT_IDS,
                    &format!("{}.chunks", project_info.name),
                    &format!("{}.documents", project_info.name),
                    &format!("{}.chunks", project_info.name)
                ))
                .bind(database_data.splitter_id)
                .bind(document_ids)
                .execute(&pool)
                .await
                .map_err(|e| {
                    is_done.store(true, Relaxed);
                    e
                })?;
                sqlx::query_scalar(&query_builder!(
                    "SELECT id FROM %s WHERE document_id = ANY($1)",
                    &format!("{}.chunks", project_info.name)
                ))
                .bind(document_ids)
                .fetch_all(&pool)
                .await
                .map(Some)
            } else {
                sqlx::query(&query_builder!(
                    queries::GENERATE_CHUNKS,
                    &format!("{}.chunks", project_info.name),
                    &format!("{}.documents", project_info.name),
                    &format!("{}.chunks", project_info.name)
                ))
                .bind(database_data.splitter_id)
                .execute(&pool)
                .await
                .map(|_t| None)
            };
            is_done.store(true, Relaxed);
            chunk_ids
        };
        let progress_work = async {
            while !is_done.load(Relaxed) {
                progress_bar.inc(1);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        };
        let (chunk_ids, _) = join!(work, progress_work);
        progress_bar.set_message("done generating chunks");
        progress_bar.finish();
        Ok(chunk_ids?)
    }

    #[instrument(skip(self, mp))]
    async fn sync_embeddings(
        &mut self,
        chunk_ids: Option<Vec<i64>>,
        mp: &MultiProgress,
    ) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let pool = self.get_pool().await?;

        let embeddings_table_name = self.create_or_get_embeddings_table().await?;

        let model = self
            .model
            .as_ref()
            .context("Pipeline must be verified to generate embeddings")?;

        let database_data = self
            .database_data
            .as_mut()
            .context("Pipeline must be verified to generate embeddings")?;

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to generate embeddings")?;

        // Remove the stored name from the parameters
        let mut parameters = model.parameters.clone();
        parameters
            .as_object_mut()
            .context("Model parameters must be an object")?
            .remove("name");

        let progress_bar = mp
            .add(utils::default_progress_spinner(1))
            .with_prefix(self.name.clone())
            .with_message("generating emmbeddings");

        let is_done = AtomicBool::new(false);
        // We need to be careful about how we handle errors here. We do not want to return an error
        // from the async block before setting is_done to true. If we do, the progress bar will
        // will load forever. We also want to make sure to propogate any errors we have
        let work = async {
            let res = match model.runtime {
                ModelRuntime::Python => if chunk_ids.is_some() {
                    sqlx::query(&query_builder!(
                        queries::GENERATE_EMBEDDINGS_FOR_CHUNK_IDS,
                        embeddings_table_name,
                        &format!("{}.chunks", project_info.name),
                        embeddings_table_name
                    ))
                    .bind(&model.name)
                    .bind(&parameters)
                    .bind(database_data.splitter_id)
                    .bind(chunk_ids)
                    .execute(&pool)
                    .await
                } else {
                    sqlx::query(&query_builder!(
                        queries::GENERATE_EMBEDDINGS,
                        embeddings_table_name,
                        &format!("{}.chunks", project_info.name),
                        embeddings_table_name
                    ))
                    .bind(&model.name)
                    .bind(&parameters)
                    .bind(database_data.splitter_id)
                    .execute(&pool)
                    .await
                }
                .map_err(|e| anyhow::anyhow!(e))
                .map(|_t| ()),
                r => {
                    let remote_embeddings = build_remote_embeddings(r, &model.name, &parameters)?;
                    remote_embeddings
                        .generate_embeddings(
                            &embeddings_table_name,
                            &format!("{}.chunks", project_info.name),
                            database_data.splitter_id,
                            chunk_ids,
                            &pool,
                        )
                        .await
                        .map(|_t| ())
                }
            };
            is_done.store(true, Relaxed);
            res
        };
        let progress_work = async {
            while !is_done.load(Relaxed) {
                progress_bar.inc(1);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        };
        let (res, _) = join!(work, progress_work);
        progress_bar.set_message("done generating embeddings");
        progress_bar.finish();
        res
    }

    #[instrument(skip(self))]
    async fn sync_tsvectors(
        &mut self,
        document_ids: &Option<Vec<i64>>,
        mp: &MultiProgress,
    ) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let pool = self.get_pool().await?;

        let parameters = self
            .parameters
            .as_ref()
            .context("Pipeline must be verified to generate tsvectors")?;

        if parameters["full_text_search"]["active"] != serde_json::Value::Bool(true) {
            return Ok(());
        }

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to generate tsvectors")?;

        let progress_bar = mp
            .add(utils::default_progress_spinner(1))
            .with_prefix(self.name.clone())
            .with_message("generating tsvectors for full text search");

        let configuration = parameters["full_text_search"]["configuration"]
            .as_str()
            .context("Full text search configuration must be a string")?;

        let is_done = AtomicBool::new(false);
        let work = async {
            let res = if document_ids.is_some() {
                sqlx::query(&query_builder!(
                    queries::GENERATE_TSVECTORS_FOR_DOCUMENT_IDS,
                    format!("{}.documents_tsvectors", project_info.name),
                    configuration,
                    configuration,
                    format!("{}.documents", project_info.name)
                ))
                .bind(document_ids)
                .execute(&pool)
                .await
            } else {
                sqlx::query(&query_builder!(
                    queries::GENERATE_TSVECTORS,
                    format!("{}.documents_tsvectors", project_info.name),
                    configuration,
                    configuration,
                    format!("{}.documents", project_info.name)
                ))
                .execute(&pool)
                .await
            };
            is_done.store(true, Relaxed);
            res.map(|_t| ()).map_err(|e| anyhow::anyhow!(e))
        };
        let progress_work = async {
            while !is_done.load(Relaxed) {
                progress_bar.inc(1);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        };
        let (res, _) = join!(work, progress_work);
        progress_bar.set_message("done generating tsvectors for full text search");
        progress_bar.finish();
        res
    }

    #[instrument(skip(self))]
    pub(crate) async fn create_or_get_embeddings_table(&mut self) -> anyhow::Result<String> {
        self.verify_in_database(false).await?;
        let pool = self.get_pool().await?;

        let collection_name = &self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to get the embeddings table name")?
            .name;
        let embeddings_table_name = format!("{}.{}_embeddings", collection_name, self.name);

        // Notice that we actually check for existence of the table in the database instead of
        // blindly creating it with `CREATE TABLE IF NOT EXISTS`. This is because we want to avoid
        // generating embeddings just to get the length if we don't need to
        let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = $1 AND table_name = $2)"
            )
            .bind(&self
                .project_info
                .as_ref()
                .context("Pipeline must have project info to get the embeddings table name")?.name)
            .bind(format!("{}_embeddings", self.name)).fetch_one(&pool).await?;

        if !exists {
            let model = self
                .model
                .as_ref()
                .context("Pipeline must be verified to create embeddings table")?;

            // Remove the stored name from the model parameters
            let mut model_parameters = model.parameters.clone();
            model_parameters
                .as_object_mut()
                .context("Model parameters must be an object")?
                .remove("name");

            let embedding_length = match &model.runtime {
                ModelRuntime::Python => {
                    let embedding: (Vec<f32>,) = sqlx::query_as(
                            "SELECT embedding from pgml.embed(transformer => $1, text => 'Hello, World!', kwargs => $2) as embedding")
                            .bind(&model.name)
                            .bind(model_parameters)
                            .fetch_one(&pool).await?;
                    embedding.0.len() as i64
                }
                t => {
                    let remote_embeddings =
                        build_remote_embeddings(t.to_owned(), &model.name, &model_parameters)?;
                    remote_embeddings.get_embedding_size().await?
                }
            };

            let mut transaction = pool.begin().await?;
            sqlx::query(&query_builder!(
                queries::CREATE_EMBEDDINGS_TABLE,
                &embeddings_table_name,
                &format!(
                    "{}.chunks",
                    self.project_info
                        .as_ref()
                        .context("Pipeline must have project info to create the embeddings table")?
                        .name
                ),
                embedding_length
            ))
            .execute(&mut *transaction)
            .await?;
            let index_name = format!("{}_pipeline_created_at_index", self.name);
            transaction
                .execute(
                    query_builder!(
                        queries::CREATE_INDEX,
                        "",
                        index_name,
                        &embeddings_table_name,
                        "created_at"
                    )
                    .as_str(),
                )
                .await?;
            let index_name = format!("{}_pipeline_chunk_id_index", self.name);
            transaction
                .execute(
                    query_builder!(
                        queries::CREATE_INDEX,
                        "",
                        index_name,
                        &embeddings_table_name,
                        "chunk_id"
                    )
                    .as_str(),
                )
                .await?;
            // See: https://github.com/pgvector/pgvector
            let (m, ef_construction) = match &self.parameters {
                Some(p) => {
                    let m = if !p["hnsw"]["m"].is_null() {
                        p["hnsw"]["m"]
                            .try_to_u64()
                            .context("hnsw.m must be an integer")?
                    } else {
                        16
                    };
                    let ef_construction = if !p["hnsw"]["ef_construction"].is_null() {
                        p["hnsw"]["ef_construction"]
                            .try_to_u64()
                            .context("hnsw.ef_construction must be an integer")?
                    } else {
                        64
                    };
                    (m, ef_construction)
                }
                None => (16, 64),
            };
            let index_with_parameters =
                format!("WITH (m = {}, ef_construction = {})", m, ef_construction);
            let index_name = format!("{}_pipeline_hnsw_vector_index", self.name);
            transaction
                .execute(
                    query_builder!(
                        queries::CREATE_INDEX_USING_HNSW,
                        "",
                        index_name,
                        &embeddings_table_name,
                        "embedding vector_cosine_ops",
                        index_with_parameters
                    )
                    .as_str(),
                )
                .await?;
            transaction.commit().await?;
        }

        Ok(embeddings_table_name)
    }

    #[instrument(skip(self))]
    pub(crate) fn set_project_info(&mut self, project_info: ProjectInfo) {
        if self.model.is_some() {
            self.model
                .as_mut()
                .unwrap()
                .set_project_info(project_info.clone());
        }
        if self.splitter.is_some() {
            self.splitter
                .as_mut()
                .unwrap()
                .set_project_info(project_info.clone());
        }
        self.project_info = Some(project_info);
    }

    /// Convert the [Pipeline] to [Json]
    ///
    /// # Example:
    ///
    /// ```
    /// use pgml::Collection;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let mut collection = Collection::new("my_collection", None);
    ///     let mut pipeline = collection.get_pipeline("my_pipeline").await?;
    ///     let pipeline_dict = pipeline.to_dict().await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn to_dict(&mut self) -> anyhow::Result<Json> {
        self.verify_in_database(false).await?;

        let status = self.get_status().await?;

        let model_dict = self
            .model
            .as_mut()
            .context("Pipeline must be verified to call to_dict")?
            .to_dict()
            .await?;

        let splitter_dict = self
            .splitter
            .as_mut()
            .context("Pipeline must be verified to call to_dict")?
            .to_dict()
            .await?;

        let database_data = self
            .database_data
            .as_ref()
            .context("Pipeline must be verified to call to_dict")?;

        let parameters = self
            .parameters
            .as_ref()
            .context("Pipeline must be verified to call to_dict")?;

        Ok(serde_json::json!({
            "id": database_data.id,
            "name": self.name,
            "model": *model_dict,
            "splitter": *splitter_dict,
            "parameters": *parameters,
            "status": *Json::from(status),
        })
        .into())
    }

    async fn get_pool(&self) -> anyhow::Result<PgPool> {
        let database_url = &self
            .project_info
            .as_ref()
            .context("Project info required to call method pipeline.get_pool()")?
            .database_url;
        get_or_initialize_pool(database_url).await
    }

    pub(crate) async fn create_pipelines_table(
        project_info: &ProjectInfo,
        conn: &mut PgConnection,
    ) -> anyhow::Result<()> {
        let pipelines_table_name = format!("{}.pipelines", project_info.name);
        sqlx::query(&query_builder!(
            queries::CREATE_PIPELINES_TABLE,
            pipelines_table_name
        ))
        .execute(&mut *conn)
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "",
                "pipeline_name_index",
                pipelines_table_name,
                "name"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }
}

impl From<models::PipelineWithModelAndSplitter> for Pipeline {
    fn from(x: models::PipelineWithModelAndSplitter) -> Self {
        Self {
            model: Some(x.clone().into()),
            splitter: Some(x.clone().into()),
            name: x.pipeline_name,
            project_info: None,
            database_data: Some(PipelineDatabaseData {
                id: x.pipeline_id,
                created_at: x.pipeline_created_at,
                model_id: x.model_id,
                splitter_id: x.splitter_id,
            }),
            parameters: Some(x.pipeline_parameters),
        }
    }
}
