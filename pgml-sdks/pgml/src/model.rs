use anyhow::Context;
use rust_bridge::{alias, alias_methods};
use sqlx::postgres::PgPool;
use tracing::instrument;

use crate::{
    collection::ProjectInfo,
    get_or_initialize_pool, models,
    types::{DateTime, Json},
};

#[cfg(feature = "python")]
use crate::types::JsonPython;

/// A few notes on the following enums:
/// - Sqlx does provide type derivation for enums, but it's not very good
/// - Queries using these enums require a number of additional queries to get their oids and
/// other information
/// - Because of the added overhead, we just cast back and forth to strings, which is kind of
/// annoying, but with the traits implimented below is a breeze and can be done just using .into

/// Our model runtimes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRuntime {
    Python,
    OpenAI,
}

impl From<&str> for ModelRuntime {
    fn from(s: &str) -> Self {
        match s {
            "pgml" | "python" => Self::Python,
            "openai" => Self::OpenAI,
            _ => panic!("Unknown model runtime: {}", s),
        }
    }
}

impl From<&ModelRuntime> for &'static str {
    fn from(m: &ModelRuntime) -> Self {
        match m {
            ModelRuntime::Python => "python",
            ModelRuntime::OpenAI => "openai",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ModelDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
}

/// A model used for embedding, inference, etc...
#[derive(alias, Debug, Clone)]
pub struct Model {
    pub name: String,
    pub runtime: ModelRuntime,
    pub parameters: Json,
    project_info: Option<ProjectInfo>,
    pub(crate) database_data: Option<ModelDatabaseData>,
}

impl Default for Model {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

#[alias_methods(new, transform)]
impl Model {
    /// Creates a new [Model]
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the model.
    /// * `source` - The source of the model. Defaults to `pgml`, but can be set to providers like `openai`.
    /// * `parameters` - The parameters to the model. Defaults to None
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Model;
    /// let model = Model::new(Some("intfloat/e5-small".to_string()), None, None, None);
    /// ```
    pub fn new(name: Option<String>, source: Option<String>, parameters: Option<Json>) -> Self {
        let name = name.unwrap_or("intfloat/e5-small".to_string());
        let parameters = parameters.unwrap_or(Json(serde_json::json!({})));
        let source = source.unwrap_or("pgml".to_string());
        let runtime: ModelRuntime = source.as_str().into();

        Self {
            name,
            runtime,
            parameters,
            project_info: None,
            database_data: None,
        }
    }

    #[instrument(skip(self))]
    pub(crate) async fn verify_in_database(&mut self, throw_if_exists: bool) -> anyhow::Result<()> {
        if self.database_data.is_none() {
            let pool = self.get_pool().await?;

            let project_info = self
                .project_info
                .as_ref()
                .expect("Cannot verify model without project info");

            let mut parameters = self.parameters.clone();
            parameters
                .as_object_mut()
                .expect("Parameters for model should be an object")
                .insert("name".to_string(), serde_json::json!(self.name));

            let model: Option<models::Model> = sqlx::query_as(
                    "SELECT id, created_at, runtime::TEXT, hyperparams FROM pgml.models WHERE project_id = $1 AND runtime = $2::pgml.runtime AND hyperparams = $3",
                )
                .bind(project_info.id)
                .bind(Into::<&str>::into(&self.runtime))
                .bind(&parameters)
                .fetch_optional(&pool)
                .await?;

            let model = if let Some(m) = model {
                anyhow::ensure!(!throw_if_exists, "Model already exists in database");
                m
            } else {
                let model: models::Model = sqlx::query_as("INSERT INTO pgml.models (project_id, num_features, algorithm, runtime, hyperparams, status, search_params, search_args) VALUES ($1, $2, $3, $4::pgml.runtime, $5, $6, $7, $8) RETURNING id, created_at, runtime::TEXT, hyperparams")
                    .bind(project_info.id)
                    .bind(1)
                    .bind("transformers")
                    .bind(Into::<&str>::into(&self.runtime))
                    .bind(parameters)
                    .bind("successful")
                    .bind(serde_json::json!({}))
                    .bind(serde_json::json!({}))
                    .fetch_one(&pool)
                    .await?;
                model
            };

            self.database_data = Some(ModelDatabaseData {
                id: model.id,
                created_at: model.created_at,
            });
        }
        Ok(())
    }

    pub(crate) fn set_project_info(&mut self, project_info: ProjectInfo) {
        self.project_info = Some(project_info);
    }

    #[instrument(skip(self))]
    pub(crate) async fn to_dict(&mut self) -> anyhow::Result<Json> {
        self.verify_in_database(false).await?;

        let database_data = self
            .database_data
            .as_ref()
            .context("Model must be verified to call to_dict")?;

        Ok(serde_json::json!({
            "id": database_data.id,
            "created_at": database_data.created_at,
            "name": self.name,
            "runtime": Into::<&str>::into(&self.runtime),
            "parameters": *self.parameters,
        })
        .into())
    }

    async fn get_pool(&self) -> anyhow::Result<PgPool> {
        let database_url = &self
            .project_info
            .as_ref()
            .context("Project info required to call method model.get_pool()")?
            .database_url;
        get_or_initialize_pool(database_url).await
    }
}

impl From<models::PipelineWithModelAndSplitter> for Model {
    fn from(x: models::PipelineWithModelAndSplitter) -> Self {
        Self {
            name: x.model_hyperparams["name"].as_str().unwrap().to_string(),
            runtime: x.model_runtime.as_str().into(),
            parameters: x.model_hyperparams,
            project_info: None,
            database_data: Some(ModelDatabaseData {
                id: x.model_id,
                created_at: x.model_created_at,
            }),
        }
    }
}

impl From<models::Model> for Model {
    fn from(model: models::Model) -> Self {
        Self {
            name: model.hyperparams["name"].as_str().unwrap().to_string(),
            runtime: model.runtime.as_str().into(),
            parameters: model.hyperparams,
            project_info: None,
            database_data: Some(ModelDatabaseData {
                id: model.id,
                created_at: model.created_at,
            }),
        }
    }
}
