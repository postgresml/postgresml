use pgml_macros::{custom_derive, custom_methods};
use pyo3::prelude::*;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::borrow::Borrow;
use std::sync::Arc;
use std::time::SystemTime;

use crate::collection::*;
use crate::get_or_set_runtime;
use crate::models;
use crate::queries;
use crate::query_builder;

#[derive(custom_derive, Clone, Debug)]
pub struct Database {
    pub pool: Arc<PgPool>,
}

#[custom_methods(new, create_or_get_collection, archive_collection)]
impl Database {
    pub async fn new(connection_string: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await?;
        sqlx::query(queries::CREATE_COLLECTIONS_TABLE)
            .execute(&pool)
            .await?;
        let pool = Arc::new(pool);
        Ok(Self { pool })
    }

    pub async fn create_or_get_collection(&self, name: &str) -> anyhow::Result<Collection> {
        let collection: Option<models::Collection> =
            sqlx::query_as("SELECT * from pgml.collections where name = $1;")
                .bind(name)
                .fetch_optional(self.pool.borrow())
                .await?;
        match collection {
            Some(c) => Ok(Collection::from_model_and_pool(c, self.pool.clone())),
            None => {
                sqlx::query("INSERT INTO pgml.collections (name) VALUES ($1)")
                    .bind(name)
                    .execute(self.pool.borrow())
                    .await?;
                Ok(Collection::new(name.to_string(), self.pool.clone()).await?)
            }
        }
    }

    pub async fn archive_collection(&self, name: &str) -> anyhow::Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Error getting system time")
            .as_secs();
        let archive_table_name = format!("{}_archive_{}", name, timestamp);
        sqlx::query(&query_builder!(
            "ALTER SCHEMA %s RENAME TO %s",
            name,
            archive_table_name
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query("UPDATE pgml.collections SET name = $1, active = FALSE where name = $2")
            .bind(archive_table_name)
            .bind(name)
            .execute(self.pool.borrow())
            .await?;
        Ok(())
    }
}
