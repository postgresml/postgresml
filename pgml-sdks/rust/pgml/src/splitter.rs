use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::PgPool;

use crate::{
    get_or_initialize_pool, models,
    types::{DateTime, Json},
};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

#[derive(custom_derive, Debug, Clone)]
pub struct Splitter {
    pub id: Option<i64>,
    pub created_at: Option<DateTime>,
    pub name: String,
    pub parameters: Json,
    pub database_url: Option<String>,
    pub verified_in_database: bool,
}

#[custom_methods(new, get_id, get_created_at, get_name, get_parameters, get_verified_in_database)]
impl Splitter {
    pub fn new(
        name: Option<String>,
        parameters: Option<Json>,
        database_url: Option<String>,
    ) -> Self {
        let name = name.unwrap_or("recursive_character".to_string());
        let parameters = parameters.unwrap_or(Json(serde_json::json!({})));
        Self {
            id: None,
            created_at: None,
            name,
            parameters,
            database_url,
            verified_in_database: false,
        }
    }

    async fn verify_in_database(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        if !self.verified_in_database {
            let splitter: Option<models::Splitter> = sqlx::query_as(
                "SELECT * FROM pgml.sdk_splitters WHERE name = $1 and parameters = $2",
            )
            .bind(&self.name)
            .bind(&self.parameters)
            .fetch_optional(pool)
            .await?;
            let splitter = if let Some(m) = splitter {
                m
            } else {
                let splitter: models::Splitter = sqlx::query_as(
                    "INSERT INTO pgml.sdk_splitters (name, parameters) VALUES ($1, $2) RETURNING *",
                )
                .bind(&self.name)
                .bind(&self.parameters)
                .fetch_one(pool)
                .await?;
                splitter
            };
            self.id = Some(splitter.id);
            self.created_at = Some(splitter.created_at);
            self.verified_in_database = true;
        }
        Ok(())
    }

    pub async fn get_id(&mut self) -> anyhow::Result<i64> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(&pool).await?;
        Ok(self.id.expect("Model id should be set"))
    }

    pub async fn get_created_at(&mut self) -> anyhow::Result<DateTime> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(&pool).await?;
        Ok(self
            .created_at
            .clone()
            .expect("Model created_at should be set"))
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_parameters(&self) -> Json {
        self.parameters.clone()
    }

    pub fn get_verified_in_database(&self) -> bool {
        self.verified_in_database
    }

    // pub async fn delete(&mut self) -> anyhow::Result<()> {
    //     let pool = get_or_initialize_pool(&self.database_url).await?;
    //     self.verify_in_database(&pool).await?;
    //     sqlx::query("DELETE FROM pgml.sdk_splitters WHERE id = $1")
    //         .bind(&self.id)
    //         .execute(&pool)
    //         .await?;
    //     self.id = None;
    //     self.created_at = None;
    //     self.verified_in_database = false;
    //     Ok(())
    // }
}
