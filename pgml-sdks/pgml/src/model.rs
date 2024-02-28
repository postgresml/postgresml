use rust_bridge::{alias, alias_methods};
use sqlx::{Pool, Postgres};
use tracing::instrument;

use crate::{
    collection::ProjectInfo,
    models,
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

#[allow(dead_code)]
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
            database_data: None,
        }
    }

    #[instrument(skip(self))]
    pub(crate) async fn verify_in_database(
        &mut self,
        project_info: &ProjectInfo,
        throw_if_exists: bool,
        pool: &Pool<Postgres>,
    ) -> anyhow::Result<()> {
        if self.database_data.is_none() {
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
                .fetch_optional(pool)
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
                    .fetch_one(pool)
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
}

impl From<models::Model> for Model {
    fn from(model: models::Model) -> Self {
        Self {
            name: model.hyperparams["name"].as_str().unwrap().to_string(),
            runtime: model.runtime.as_str().into(),
            parameters: model.hyperparams,
            database_data: Some(ModelDatabaseData {
                id: model.id,
                created_at: model.created_at,
            }),
        }
    }
}
