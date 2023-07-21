use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::PgPool;

use crate::{
    get_or_initialize_pool, models,
    types::{DateTime, Json},
};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

#[derive(custom_derive, Debug, Clone)]
pub struct Model {
    pub id: Option<i64>,
    pub created_at: Option<DateTime>,
    pub name: String,
    pub task: String,
    pub source: String,
    pub parameters: Json,
    pub database_url: Option<String>,
    pub verified_in_database: bool,
}

#[custom_methods(new, get_id, get_created_at, get_task, get_name, get_source, get_parameters, get_verified_in_database)]
impl Model {
    pub fn new(
        name: Option<String>,
        task: Option<String>,
        source: Option<String>,
        parameters: Option<Json>,
        database_url: Option<String>,
    ) -> Self {
        let name = name.unwrap_or("intfloat/e5-small".to_string());
        let task = task.unwrap_or("embedding".to_string());
        let parameters = parameters.unwrap_or(Json(serde_json::json!({})));
        let source = source.unwrap_or("pgml".to_string());

        Self {
            id: None,
            created_at: None,
            name,
            task,
            source,
            parameters,
            database_url,
            verified_in_database: false,
        }
    }

    async fn verify_in_database(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        if !self.verified_in_database {
            let model: Option<models::Model> = sqlx::query_as("SELECT * FROM pgml.sdk_models WHERE name = $1 AND task = $2 AND source = $3 and parameters = $4")         
                .bind(&self.name)
                .bind(&self.task)
                .bind(&self.source)
                .bind(&self.parameters)
                .fetch_optional(pool)
                .await?;
            let model = if let Some(m) = model {
                m
            } else {
                let model: models::Model = sqlx::query_as("INSERT INTO pgml.sdk_models (name, task, source, parameters) VALUES ($1, $2, $3, $4) RETURNING *")
                    .bind(&self.name)
                    .bind(&self.task)
                    .bind(&self.source)
                    .bind(&self.parameters)
                    .fetch_one(pool)
                    .await?;
                model
            };
            self.id = Some(model.id);
            self.created_at = Some(model.created_at);
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

    pub fn get_task(&self) -> String {
        self.task.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_source(&self) -> String {
        self.source.clone()
    }

    pub fn get_parameters(&self) -> Json {
        self.parameters.clone()
    }

    pub fn get_verified_in_database(&self) -> bool {
        self.verified_in_database
    }

    pub async fn register(&mut self) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(&pool).await?;
        Ok(())
    }

    // pub async fn delete(&mut self) -> anyhow::Result<()> {
    //     let pool = get_or_initialize_pool(&self.database_url).await?;
    //     self.verify_in_database(&pool).await?;
    //     sqlx::query("DELETE FROM pgml.sdk_models WHERE id = $1")
    //         .bind(&self.id)
    //         .execute(&pool)
    //         .await?;
    //     self.id = None;
    //     self.created_at = None;
    //     self.verified_in_database = false;
    //     Ok(())
    // }

    pub fn from_model_and_database_url(m: models::Model, database_url: Option<String>) -> Self {
        Self {
            id: Some(m.id),
            created_at: Some(m.created_at),
            task: m.task,
            name: m.name,
            source: m.source,
            parameters: m.parameters,
            database_url,
            verified_in_database: true,
        }
    }
}
