use sqlx::{postgres::PgConnection, Pool, Postgres};
use tracing::instrument;

use crate::{
    collection::ProjectInfo,
    models, queries,
    types::{DateTime, Json},
};

#[cfg(feature = "python")]
use crate::types::JsonPython;

#[cfg(feature = "c")]
use crate::languages::c::JsonC;

#[cfg(feature = "rust_bridge")]
use rust_bridge::{alias, alias_methods};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct SplitterDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
}

/// A text splitter
#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[derive(Debug, Clone)]
pub struct Splitter {
    pub(crate) name: String,
    pub(crate) parameters: Json,
    pub(crate) database_data: Option<SplitterDatabaseData>,
}

impl Default for Splitter {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[cfg_attr(feature = "rust_bridge", alias_methods(new))]
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
            let splitter: Option<models::Splitter> = sqlx::query_as(
                    "SELECT * FROM pgml.splitters WHERE project_id = $1 AND name = $2 and parameters = $3",
                )
                .bind(project_info.id)
                .bind(&self.name)
                .bind(&self.parameters)
                .fetch_optional(pool)
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
                    .fetch_one(pool)
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
}

impl From<models::Splitter> for Splitter {
    fn from(splitter: models::Splitter) -> Self {
        Self {
            name: splitter.name,
            parameters: splitter.parameters,
            database_data: Some(SplitterDatabaseData {
                id: splitter.id,
                created_at: splitter.created_at,
            }),
        }
    }
}
