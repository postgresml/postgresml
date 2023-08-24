use anyhow::Context;
use rust_bridge::{alias, alias_methods};
use sqlx::postgres::{PgConnection, PgPool};
use tracing::instrument;

use crate::{
    collection::ProjectInfo,
    get_or_initialize_pool, models, queries,
    types::{DateTime, Json},
};

#[cfg(feature = "python")]
use crate::types::JsonPython;

#[derive(Debug, Clone)]
pub(crate) struct SplitterDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
}

/// A text splitter
#[derive(alias, Debug, Clone)]
pub struct Splitter {
    pub name: String,
    pub parameters: Json,
    project_info: Option<ProjectInfo>,
    pub(crate) database_data: Option<SplitterDatabaseData>,
}

impl Default for Splitter {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[alias_methods(new)]
impl Splitter {
    /// Creates a new [Splitter]
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the splitter.
    /// * `parameters` - The parameters to the splitter. Defaults to None
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Splitter;
    /// let splitter = Splitter::new(Some("recursive_character".to_string()), None);
    /// ```
    pub fn new(name: Option<String>, parameters: Option<Json>) -> Self {
        let name = name.unwrap_or("recursive_character".to_string());
        let parameters = parameters.unwrap_or(Json(serde_json::json!({})));
        Self {
            name,
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
                .expect("Cannot verify splitter without project info");

            let splitter: Option<models::Splitter> = sqlx::query_as(
                    "SELECT * FROM pgml.splitters WHERE project_id = $1 AND name = $2 and parameters = $3",
                )
                .bind(project_info.id)
                .bind(&self.name)
                .bind(&self.parameters)
                .fetch_optional(&pool)
                .await?;

            let splitter = if let Some(s) = splitter {
                anyhow::ensure!(!throw_if_exists, "Splitter already exists in database");
                s
            } else {
                sqlx::query_as(
                        "INSERT INTO pgml.splitters (project_id, name, parameters) VALUES ($1, $2, $3) RETURNING *",
                    )
                    .bind(project_info.id)
                    .bind(&self.name)
                    .bind(&self.parameters)
                    .fetch_one(&pool)
                    .await?
            };

            self.database_data = Some(SplitterDatabaseData {
                id: splitter.id,
                created_at: splitter.created_at,
            });
        }
        Ok(())
    }

    pub(crate) async fn create_splitters_table(conn: &mut PgConnection) -> anyhow::Result<()> {
        sqlx::query(queries::CREATE_SPLITTERS_TABLE)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub(crate) fn set_project_info(&mut self, project_info: ProjectInfo) {
        self.project_info = Some(project_info)
    }

    #[instrument(skip(self))]
    pub(crate) async fn to_dict(&mut self) -> anyhow::Result<Json> {
        self.verify_in_database(false).await?;

        let database_data = self
            .database_data
            .as_ref()
            .context("Splitter must be verified to call to_dict")?;

        Ok(serde_json::json!({
            "id": database_data.id,
            "created_at": database_data.created_at,
            "name": self.name,
            "parameters": *self.parameters,
        })
        .into())
    }

    async fn get_pool(&self) -> anyhow::Result<PgPool> {
        let database_url = &self
            .project_info
            .as_ref()
            .context("Project info required to call method splitter.get_pool()")?
            .database_url;
        get_or_initialize_pool(database_url).await
    }
}

impl From<models::PipelineWithModelAndSplitter> for Splitter {
    fn from(x: models::PipelineWithModelAndSplitter) -> Self {
        Self {
            name: x.splitter_name,
            parameters: x.splitter_parameters,
            project_info: None,
            database_data: Some(SplitterDatabaseData {
                id: x.splitter_id,
                created_at: x.splitter_created_at,
            }),
        }
    }
}

impl From<models::Splitter> for Splitter {
    fn from(splitter: models::Splitter) -> Self {
        Self {
            name: splitter.name,
            parameters: splitter.parameters,
            project_info: None,
            database_data: Some(SplitterDatabaseData {
                id: splitter.id,
                created_at: splitter.created_at,
            }),
        }
    }
}
