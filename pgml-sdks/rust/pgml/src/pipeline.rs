use anyhow::Context;
use pgml_macros::{custom_derive, custom_methods};
use sqlx::PgPool;

use crate::{
    collection::ProjectInfo, model::Model, models, queries, query_builder, splitter::Splitter,
    types::DateTime,
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
    pub chunks_status: PipelineSyncStatus,
    pub embeddings_status: PipelineSyncStatus,
    pub tsvectors_status: PipelineSyncStatus,
}

#[derive(Debug, Clone)]
pub struct PipelineDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
    pub model_id: i64,
    pub splitter_id: i64,
    pub sync_data: PipelineSyncData,
}

#[derive(custom_derive, Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub model: Option<Model>,
    pub splitter: Option<Splitter>,
    pub project_info: Option<ProjectInfo>,
    pub database_data: Option<PipelineDatabaseData>,
}

#[custom_methods(new)]
impl Pipeline {
    pub fn new(name: &str, model: Option<Model>, splitter: Option<Splitter>) -> Self {
        Self {
            name: name.to_string(),
            model,
            splitter,
            project_info: None,
            database_data: None,
        }
    }

    pub async fn verify_in_database(
        &mut self,
        pool: &PgPool,
        throw_if_exists: bool,
    ) -> anyhow::Result<()> {
        match &self.database_data {
            Some(_) => Ok(()),
            None => {
                let project_info = self
                    .project_info
                    .as_ref()
                    .expect("Cannot verify pipeline without project info");

                let result: Result<Option<models::Pipeline>, _> = sqlx::query_as(&query_builder!(
                    "SELECT * FROM %s WHERE name = $1",
                    format!("{}.pipelines", project_info.name)
                ))
                .bind(&self.name)
                .fetch_optional(pool)
                .await;

                let pipeline: Option<models::Pipeline> = match result {
                    Ok(s) => anyhow::Ok(s),
                    Err(e) => {
                        match e.as_database_error() {
                            Some(db_e) => {
                                // Error 42P01 is "undefined_table"
                                if db_e.code() == Some(std::borrow::Cow::from("42P01")) {
                                    // Must create splitters_sdk table here first if it does not exist
                                    // or we cannot create the pipeline table
                                    Splitter::create_splitters_table(pool).await?;
                                    sqlx::query(&query_builder!(
                                        queries::CREATE_PIPELINES_TABLE,
                                        &format!("{}.pipelines", project_info.name)
                                    ))
                                    .execute(pool)
                                    .await?;
                                    Ok(None)
                                } else {
                                    Err(e.into())
                                }
                            }
                            None => Err(e.into()),
                        }
                    }
                }?;

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
                        "INSERT INTO %s (name, model_id, splitter_id, chunks_status, embeddings_status, tsvectors_status) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
                        format!("{}.pipelines", project_info.name)
                    ))
                    .bind(&self.name)
                    .bind(
                        &model
                            .database_data
                            .as_ref()
                            .expect("Cannot save pipeline without model")
                            .id,
                    )
                    .bind(
                        &splitter
                            .database_data
                            .as_ref()
                            .expect("Cannot save pipeline without splitter")
                            .id,
                    )
                    .bind(Into::<&str>::into(&PipelineSyncStatus::NotSynced))
                    .bind(Into::<&str>::into(&PipelineSyncStatus::NotSynced))
                    .bind(Into::<&str>::into(&PipelineSyncStatus::NotSynced))
                    .fetch_one(pool)
                    .await?
                };

                self.database_data = Some(PipelineDatabaseData {
                    id: pipeline.id,
                    created_at: pipeline.created_at,
                    model_id: pipeline.model_id,
                    splitter_id: pipeline.splitter_id,
                    sync_data: PipelineSyncData {
                        chunks_status: pipeline.chunks_status.as_str().into(),
                        embeddings_status: pipeline.embeddings_status.as_str().into(),
                        tsvectors_status: pipeline.tsvectors_status.as_str().into(),
                    },
                });
                Ok(())
            }
        }
    }

    pub async fn execute(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        self.sync_chunks(pool).await?;
        Ok(())
    }

    async fn sync_chunks(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        self.verify_in_database(pool, false).await?;
        let database_data = self
            .database_data
            .as_ref()
            .context("Pipeline must be verified to generate chunks")?;

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to generate chunks")?;

        match database_data.sync_data.chunks_status {
            PipelineSyncStatus::NotSynced => {
                sqlx::query(&query_builder!(
                    queries::GENERATE_CHUNKS,
                    &format!("{}.chunks", project_info.name),
                    &format!("{}.documents", project_info.name),
                    &format!("{}.chunks", project_info.name)
                ))
                .bind(database_data.splitter_id)
                .execute(pool)
                .await?;

                self.database_data.as_mut().unwrap().sync_data.chunks_status =
                    PipelineSyncStatus::Synced;
            }
            PipelineSyncStatus::Synced => {
                println!("Pipeline {} chunks are already synced", self.name);
            }
            _ => (),
        }
        Ok(())
    }

    pub fn set_project_info(&mut self, project_info: ProjectInfo) {
        self.project_info = Some(project_info);
    }
}
