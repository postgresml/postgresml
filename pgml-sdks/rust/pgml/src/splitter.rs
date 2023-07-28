use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::PgPool;

use crate::{
    collection::ProjectInfo,
    models, queries,
    types::{DateTime, Json},
};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

#[cfg(feature = "python")]
use crate::languages::CustomInto;

#[derive(Debug, Clone)]
pub struct SplitterDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
}

#[derive(custom_derive, Debug, Clone)]
pub struct Splitter {
    pub name: String,
    pub parameters: Json,
    pub project_info: Option<ProjectInfo>,
    pub database_data: Option<SplitterDatabaseData>,
}

#[custom_methods(new)]
impl Splitter {
    pub fn new(
        name: Option<String>,
        parameters: Option<Json>
    ) -> Self {
        let name = name.unwrap_or("recursive_character".to_string());
        let parameters = parameters.unwrap_or(Json(serde_json::json!({})));
        Self {
            name,
            parameters,
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
                    .expect("Cannot verify splitter without project info");

                let result: Result<Option<models::Splitter>, _> = sqlx::query_as(
                    "SELECT * FROM pgml.sdk_splitters WHERE project_id = $1 AND name = $2 and parameters = $3",
                )
                .bind(project_info.id)
                .bind(&self.name)
                .bind(&self.parameters)
                .fetch_optional(pool)
                .await;

                let splitter: Option<models::Splitter> = match result {
                    Ok(s) => anyhow::Ok(s),
                    Err(e) => {
                        match e.as_database_error() {
                            Some(db_e) => {
                                // Error 42P01 is "undefined_table"
                                if db_e.code() == Some(std::borrow::Cow::from("42P01")) {
                                    Self::create_splitters_table(pool).await?;
                                    Ok(None)
                                } else {
                                    Err(e.into())
                                }
                            }
                            None => Err(e.into()),
                        }
                    }
                }?;

                let splitter = if let Some(s) = splitter {
                    if throw_if_exists {
                        anyhow::bail!("Splitter already exists in database")
                    }
                    s
                } else {
                    sqlx::query_as(
                        "INSERT INTO pgml.sdk_splitters (project_id, name, parameters) VALUES ($1, $2, $3) RETURNING *",
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
                Ok(())
            }
        }
    }

    pub async fn create_splitters_table(pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query(queries::CREATE_SPLITTERS_TABLE)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub fn set_project_info(&mut self, project_info: ProjectInfo) {
        self.project_info = Some(project_info)
    }
}
