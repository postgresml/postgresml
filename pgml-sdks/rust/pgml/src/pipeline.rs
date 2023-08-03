use anyhow::Context;
use pgml_macros::{custom_derive, custom_methods};
use sqlx::{Executor, PgConnection, PgPool};
use tracing::instrument;

use crate::{
    collection::ProjectInfo,
    get_or_initialize_pool,
    model::{Model, ModelRuntime},
    models, queries, query_builder,
    remote_embeddings::build_remote_embeddings,
    splitter::Splitter,
    types::{DateTime, Json},
};

#[cfg(feature = "python")]
use crate::{languages::CustomInto, model::ModelPython, splitter::SplitterPython};

#[derive(Debug, Clone)]
pub enum PipelineSyncStatus {
    NA,
    NotSynced,
    Synced,
    Syncing,
    Failed,
}

impl From<&str> for PipelineSyncStatus {
    fn from(s: &str) -> Self {
        match s {
            "not_synced" => Self::NotSynced,
            "synced" => Self::Synced,
            "syncing" => Self::Syncing,
            "failed" => Self::Failed,
            _ => Self::NA,
        }
    }
}

impl From<&PipelineSyncStatus> for &str {
    fn from(s: &PipelineSyncStatus) -> Self {
        match s {
            PipelineSyncStatus::NotSynced => "not_synced",
            PipelineSyncStatus::Synced => "synced",
            PipelineSyncStatus::Syncing => "syncing",
            PipelineSyncStatus::Failed => "failed",
            _ => "na",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineSyncData {
    pub chunks_status: (i64, i64),
    pub embeddings_status: (i64, i64),
    pub tsvectors_status: (i64, i64),
}

#[derive(Debug, Clone)]
pub struct PipelineDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
    pub model_id: i64,
    pub splitter_id: i64,
}

#[derive(custom_derive, Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub model: Option<Model>,
    pub splitter: Option<Splitter>,
    pub project_info: Option<ProjectInfo>,
    pub database_data: Option<PipelineDatabaseData>,
    pub parameters: Option<Json>,
}

#[custom_methods(new, to_dict)]
impl Pipeline {
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
            project_info: None,
            database_data: None,
            parameters,
        }
    }

    #[instrument(skip(self))]
    pub async fn get_status(&mut self) -> anyhow::Result<PipelineSyncData> {
        let database_url = &self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to get status")?
            .database_url;

        let pool = get_or_initialize_pool(database_url).await?;
        self.verify_in_database(&pool, false).await?;

        let embeddings_table_name = self.create_or_get_embeddings_table(&pool).await?;

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
            "SELECT chunks.m, documents.m FROM (SELECT MAX(document_id) AS m FROM %s WHERE splitter_id = $1) chunks, (SELECT MAX(id) AS m FROM %s) documents",
            format!("{}.chunks", project_name),
            format!("{}.documents", project_name)
        ))
        .bind(database_data.splitter_id)
        .fetch_one(&pool).await?;
        let chunks_status = (chunks_status.0.unwrap_or(0), chunks_status.1.unwrap_or(0));

        let embeddings_status: (Option<i64>, Option<i64>) = sqlx::query_as(&query_builder!(
            "SELECT embeddings.m, chunks.m FROM (SELECT MAX(id) AS m FROM %s) embeddings, (SELECT MAX(id) AS m FROM %s WHERE splitter_id = $1) chunks",
            embeddings_table_name,
            format!("{}.chunks", project_name)
        ))
        .bind(database_data.splitter_id)
        .fetch_one(&pool).await?;
        let embeddings_status = (
            embeddings_status.0.unwrap_or(0),
            embeddings_status.1.unwrap_or(0),
        );

        let tsvectors_status = if parameters["full_text_search"]["active"]
            == serde_json::Value::Bool(true)
        {
            sqlx::query_as(&query_builder!(
                "SELECT tsvectors.m, documents.m FROM (SELECT MAX(id) AS m FROM %s WHERE configuration = $1) tsvectors, (SELECT MAX(id) AS m FROM %s) documents",
                format!("{}.documents_tsvectors", project_name),
                format!("{}.documents", project_name)
            ))
            .bind(parameters["full_text_search"]["configuration"].as_str())
            .fetch_one(&pool).await?
        } else {
            (Some(0), Some(0))
        };
        let tsvectors_status = (
            tsvectors_status.0.unwrap_or(0),
            tsvectors_status.1.unwrap_or(0),
        );

        Ok(PipelineSyncData {
            chunks_status,
            embeddings_status,
            tsvectors_status,
        })
    }

    #[instrument(skip(self, pool))]
    pub async fn verify_in_database(
        &mut self,
        pool: &PgPool,
        throw_if_exists: bool,
    ) -> anyhow::Result<()> {
        if self.database_data.is_none() {
            let project_info = self
                .project_info
                .as_ref()
                .expect("Cannot verify pipeline without project info");

            let pipeline: Option<models::Pipeline> = sqlx::query_as(&query_builder!(
                "SELECT * FROM %s WHERE name = $1",
                format!("{}.pipelines", project_info.name)
            ))
            .bind(&self.name)
            .fetch_optional(pool)
            .await?;

            let pipeline = if let Some(p) = pipeline {
                if throw_if_exists {
                    anyhow::bail!("Pipeline {} already exists", p.name);
                }
                p
            } else {
                let model = self
                    .model
                    .as_mut()
                    .expect("Cannot save pipeline without model");
                model.set_project_info(project_info.clone());
                model.verify_in_database(pool, false).await?;

                let splitter = self
                    .splitter
                    .as_mut()
                    .expect("Cannot save pipeline without splitter");
                splitter.set_project_info(project_info.clone());
                splitter.verify_in_database(pool, false).await?;

                sqlx::query_as(&query_builder!(
                        "INSERT INTO %s (name, model_id, splitter_id, parameters) VALUES ($1, $2, $3, $4) RETURNING *",
                        format!("{}.pipelines", project_info.name)
                    ))
                    .bind(&self.name)
                    .bind(
                        model
                            .database_data
                            .as_ref()
                            .expect("Cannot save pipeline without model")
                            .id,
                    )
                    .bind(
                        splitter
                            .database_data
                            .as_ref()
                            .expect("Cannot save pipeline without splitter")
                            .id,
                    )
                    .bind(&self.parameters)
                    .fetch_one(pool)
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

    #[instrument(skip(self, pool))]
    pub async fn get_and_set_model(&mut self, pool: &PgPool) -> anyhow::Result<&mut Model> {
        self.verify_in_database(pool, false).await?;
        if self.model.is_none() {
            let model: models::Model = sqlx::query_as(
                "SELECT id, created_at, runtime::TEXT, hyperparams FROM pgml.models WHERE id = $1",
            )
            .bind(
                self.database_data
                    .as_ref()
                    .context("Pipeline must be verified to set model")?
                    .model_id,
            )
            .fetch_one(pool)
            .await?;
            let project_info = self
                .project_info
                .as_ref()
                .expect("Cannot set model without project info");
            let mut model: Model = model.into();
            model.set_project_info(project_info.clone());
            self.model = Some(model);
        }
        Ok(self.model.as_mut().unwrap())
    }

    async fn get_and_set_splitter(&mut self, pool: &PgPool) -> anyhow::Result<&mut Splitter> {
        self.verify_in_database(pool, false).await?;
        if self.splitter.is_none() {
            let splitter: models::Splitter =
                sqlx::query_as("SELECT * FROM pgml.sdk_splitters WHERE id = $1")
                    .bind(
                        self.database_data
                            .as_ref()
                            .context("Pipeline must be verified to set splitter")?
                            .splitter_id,
                    )
                    .fetch_one(pool)
                    .await?;
            let project_info = self
                .project_info
                .as_ref()
                .expect("Cannot set model without project info");
            let mut splitter: Splitter = splitter.into();
            splitter.set_project_info(project_info.clone());
            self.splitter = Some(splitter);
        }
        Ok(self.splitter.as_mut().unwrap())
    }

    #[instrument(skip(self, pool))]
    pub async fn execute(
        &mut self,
        document_ids: &Option<Vec<i64>>,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        // Verify in the database once before calling other functions that all rely on it
        self.verify_in_database(pool, false).await?;

        // TODO: Chunk document_ids if there are too many
        let chunk_ids = self.sync_chunks(document_ids, pool).await?;
        self.sync_embeddings(chunk_ids, pool).await?;
        self.sync_tsvectors(document_ids, pool).await?;
        Ok(())
    }

    #[instrument(skip(self, pool))]
    async fn sync_chunks(
        &mut self,
        document_ids: &Option<Vec<i64>>,
        pool: &PgPool,
    ) -> anyhow::Result<Option<Vec<i64>>> {
        let database_data = self
            .database_data
            .as_mut()
            .context("Pipeline must be verified to generate chunks")?;

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to generate chunks")?;

        // This part is a bit tricky
        // We want to return the ids for all chunks we inserted OR would have inserted if they didn't already exist
        // The query is structured in such a way to not insert any chunks that already exist so we
        // can't rely on the data returned from the inset queries, we need to query the chunks table
        // It is important we return the ids for chunks we would have inserted if they didn't already exist so we are robust to random crashes
        if document_ids.is_some() {
            sqlx::query(&query_builder!(
                queries::GENERATE_CHUNKS_FOR_DOCUMENT_IDS,
                &format!("{}.chunks", project_info.name),
                &format!("{}.documents", project_info.name),
                &format!("{}.chunks", project_info.name)
            ))
            .bind(database_data.splitter_id)
            .bind(document_ids)
            .execute(pool)
            .await?;
            let chunk_ids: Vec<i64> = sqlx::query_scalar(&query_builder!(
                "SELECT id FROM %s WHERE document_id = ANY($1)",
                &format!("{}.chunks", project_info.name)
            ))
            .bind(document_ids)
            .fetch_all(pool)
            .await?;
            Ok(Some(chunk_ids))
        } else {
            sqlx::query(&query_builder!(
                queries::GENERATE_CHUNKS,
                &format!("{}.chunks", project_info.name),
                &format!("{}.documents", project_info.name),
                &format!("{}.chunks", project_info.name)
            ))
            .bind(database_data.splitter_id)
            .execute(pool)
            .await?;
            Ok(None)
        }
    }

    #[instrument(skip(self, pool))]
    async fn sync_embeddings(
        &mut self,
        chunk_ids: Option<Vec<i64>>,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        self.get_and_set_model(pool).await?;
        let embeddings_table_name = self.create_or_get_embeddings_table(pool).await?;

        let model = self
            .model
            .as_ref()
            .context("Pipeline must have model to generate embeddings")?;

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

        match model.runtime {
            ModelRuntime::Python => {
                if chunk_ids.is_some() {
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
                    .execute(pool)
                    .await?;
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
                    .execute(pool)
                    .await?;
                }
            }
            r => {
                let remote_embeddings = build_remote_embeddings(r, &model.name, &parameters)?;
                remote_embeddings
                    .generate_embeddings(
                        &embeddings_table_name,
                        &format!("{}.chunks", project_info.name),
                        database_data.splitter_id,
                        chunk_ids,
                        pool,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    #[instrument(skip(self, pool))]
    async fn sync_tsvectors(
        &mut self,
        document_ids: &Option<Vec<i64>>,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
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

        if document_ids.is_some() {
            sqlx::query(&query_builder!(
                queries::GENERATE_TSVECTORS_FOR_DOCUMENT_IDS,
                format!("{}.documents_tsvectors", project_info.name),
                parameters["full_text_search"]["configuration"],
                parameters["full_text_search"]["configuration"],
                format!("{}.documents", project_info.name)
            ))
            .bind(document_ids)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(&query_builder!(
                queries::GENERATE_TSVECTORS,
                format!("{}.documents_tsvectors", project_info.name),
                parameters["full_text_search"]["configuration"]
                    .as_str()
                    .context("Full text search configuration must be a string")?,
                parameters["full_text_search"]["configuration"]
                    .as_str()
                    .context("Full text search configuration must be a string")?,
                format!("{}.documents", project_info.name)
            ))
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    #[instrument(skip(self, pool))]
    async fn create_or_get_embeddings_table(&mut self, pool: &PgPool) -> anyhow::Result<String> {
        self.verify_in_database(pool, false).await?;

        let embeddings_table_name = format!(
            "{}.{}_embeddings",
            &self
                .project_info
                .as_ref()
                .context("Pipeline must have project info to get the embeddings table name")?
                .name,
            self.name
        );

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
            .bind(&embeddings_table_name).fetch_one(pool).await?;

        if !exists {
            self.get_and_set_model(pool).await?;
            let model = self
                .model
                .as_ref()
                .context("Cannot create embeddings table without having the model")?;

            // Remove the stored name from the parameters
            let mut parameters = model.parameters.clone();
            parameters
                .as_object_mut()
                .context("Model parameters must be an object")?
                .remove("name");

            let embedding_length = match &model.runtime {
                ModelRuntime::Python => {
                    let embedding: (Vec<f32>,) = sqlx::query_as(
                            "SELECT embedding from pgml.embed(transformer => $1, text => 'Hello, World!', kwargs => $2) as embedding")
                            .bind(&model.name)
                            .bind(parameters)
                            .fetch_one(pool).await?;
                    embedding.0.len() as i64
                }
                t => {
                    let remote_embeddings =
                        build_remote_embeddings(t.to_owned(), &model.name, &model.parameters)?;
                    remote_embeddings.get_embedding_size().await?
                }
            };

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
            .execute(pool)
            .await?;
            pool.execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "",
                    "created_at_index",
                    &embeddings_table_name,
                    "created_at"
                )
                .as_str(),
            )
            .await?;
            pool.execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "",
                    "chunk_id_index",
                    &embeddings_table_name,
                    "chunk_id"
                )
                .as_str(),
            )
            .await?;
            pool.execute(
                query_builder!(
                    queries::CREATE_INDEX_USING_IVFFLAT,
                    "",
                    "vector_index",
                    &embeddings_table_name,
                    "embedding vector_cosine_ops"
                )
                .as_str(),
            )
            .await?;
        }

        Ok(embeddings_table_name)
    }

    #[instrument(skip(self))]
    pub fn set_project_info(&mut self, project_info: ProjectInfo) {
        self.project_info = Some(project_info);
    }

    #[instrument(skip(self))]
    pub async fn to_dict(&mut self) -> anyhow::Result<Json> {
        let database_url = &self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to call to_dict. Are you calling to_dict on a pipeline that has not been added to a collection?")?
            .database_url;
        let pool = get_or_initialize_pool(database_url).await?;
        self.verify_in_database(&pool, false).await?;

        let model = self.get_and_set_model(&pool).await?;
        let model_dict = model.to_dict().await?;

        let splitter = self.get_and_set_splitter(&pool).await?;
        let splitter_dict = splitter.to_dict().await?;

        let database_data = self
            .database_data
            .as_ref()
            .context("Pipeline must be verified to call to_dict")?;

        Ok(serde_json::json!({
            "id": database_data.id,
            "name": self.name,
            "model": *model_dict,
            "splitter": *splitter_dict
        })
        .into())
    }

    // We want to be able to call this function from anywhere
    pub async fn create_pipelines_table(
        project_info: &ProjectInfo,
        conn: &mut PgConnection,
    ) -> anyhow::Result<()> {
        sqlx::query(&query_builder!(
            queries::CREATE_PIPELINES_TABLE,
            &format!("{}.pipelines", project_info.name)
        ))
        .execute(conn)
        .await?;
        Ok(())
    }
}

impl From<models::Pipeline> for Pipeline {
    fn from(pipeline: models::Pipeline) -> Self {
        Self {
            name: pipeline.name,
            model: None,
            splitter: None,
            project_info: None,
            database_data: Some(PipelineDatabaseData {
                id: pipeline.id,
                created_at: pipeline.created_at,
                model_id: pipeline.model_id,
                splitter_id: pipeline.splitter_id,
            }),
            parameters: Some(pipeline.parameters),
        }
    }
}
