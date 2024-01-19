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
    multi_field_pipeline::MultiFieldPipeline,
    queries, query_builder,
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
    ) -> MultiFieldPipeline {
        // let schema = serde_json::json!({
        //     "text": {
        //         "embed": {
        //             "model": model.na
        // });
        let schema = if let Some(model) = model {
            Some(serde_json::json!({
                "text": {
                    "embed": {
                        "model": model.name
                    }
                }
            }))
        } else {
            None
        };
        MultiFieldPipeline::new(name, schema.map(|v| v.into()))
            .expect("Error conerting pipeline into new multifield pipeline")

        // let parameters = Some(parameters.unwrap_or_default());
        // Self {
        //     name: name.to_string(),
        //     model,
        //     splitter,
        //     parameters,
        //     project_info: None,
        //     database_data: None,
        // }
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
        unimplemented!()
        // let pool = self.get_pool().await?;

        // self.verify_in_database(false).await?;
        // let embeddings_table_name = self.create_or_get_embeddings_table().await?;

        // let database_data = self
        //     .database_data
        //     .as_ref()
        //     .context("Pipeline must be verified to get status")?;

        // let parameters = self
        //     .parameters
        //     .as_ref()
        //     .context("Pipeline must be verified to get status")?;

        // let project_name = &self.project_info.as_ref().unwrap().name;

        // // TODO: Maybe combine all of these into one query so it is faster
        // let chunks_status: (Option<i64>, Option<i64>) = sqlx::query_as(&query_builder!(
        //     "SELECT (SELECT COUNT(DISTINCT document_id) FROM %s WHERE splitter_id = $1), COUNT(id) FROM %s",
        //     format!("{}.chunks", project_name),
        //     format!("{}.documents", project_name)
        // ))
        // .bind(database_data.splitter_id)
        // .fetch_one(&pool).await?;
        // let chunks_status = InvividualSyncStatus {
        //     synced: chunks_status.0.unwrap_or(0),
        //     not_synced: chunks_status.1.unwrap_or(0) - chunks_status.0.unwrap_or(0),
        //     total: chunks_status.1.unwrap_or(0),
        // };

        // let embeddings_status: (Option<i64>, Option<i64>) = sqlx::query_as(&query_builder!(
        //     "SELECT (SELECT count(*) FROM %s), (SELECT count(*) FROM %s WHERE splitter_id = $1)",
        //     embeddings_table_name,
        //     format!("{}.chunks", project_name)
        // ))
        // .bind(database_data.splitter_id)
        // .fetch_one(&pool)
        // .await?;
        // let embeddings_status = InvividualSyncStatus {
        //     synced: embeddings_status.0.unwrap_or(0),
        //     not_synced: embeddings_status.1.unwrap_or(0) - embeddings_status.0.unwrap_or(0),
        //     total: embeddings_status.1.unwrap_or(0),
        // };

        // let tsvectors_status = if parameters["full_text_search"]["active"]
        //     == serde_json::Value::Bool(true)
        // {
        //     sqlx::query_as(&query_builder!(
        //         "SELECT (SELECT COUNT(*) FROM %s WHERE configuration = $1), (SELECT COUNT(*) FROM %s)",
        //         format!("{}.documents_tsvectors", project_name),
        //         format!("{}.documents", project_name)
        //     ))
        //     .bind(parameters["full_text_search"]["configuration"].as_str())
        //     .fetch_one(&pool).await?
        // } else {
        //     (Some(0), Some(0))
        // };
        // let tsvectors_status = InvividualSyncStatus {
        //     synced: tsvectors_status.0.unwrap_or(0),
        //     not_synced: tsvectors_status.1.unwrap_or(0) - tsvectors_status.0.unwrap_or(0),
        //     total: tsvectors_status.1.unwrap_or(0),
        // };

        // Ok(PipelineSyncData {
        //     chunks_status,
        //     embeddings_status,
        //     tsvectors_status,
        // })
    }
}
