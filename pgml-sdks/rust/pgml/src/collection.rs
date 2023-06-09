use anyhow::anyhow;
use itertools::Itertools;
use log::warn;
use pgml_macros::{custom_derive, custom_methods};
use pyo3::prelude::*;
use sqlx::postgres::PgPool;
use sqlx::types::Json;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

use crate::models;
use crate::queries;
use crate::query_builder;

#[derive(custom_derive, Debug, Clone)]
pub struct Collection {
    name: String,
    pool: Arc<PgPool>,
    documents_table_name: String,
    splitters_table_name: String,
    models_table_name: String,
    transforms_table_name: String,
    chunks_table_name: String,
    #[allow(unused)]
    active: bool,
}

#[custom_methods(
    get_name,
    get_name_result,
    upsert_documents,
    register_text_splitter,
    get_text_splitters,
    generate_chunks,
    register_model,
    get_models,
    generate_embeddings,
    vector_search,
    get_name
)]
impl Collection {
    pub async fn new(name: String, pool: Arc<PgPool>) -> anyhow::Result<Self> {
        let (
            documents_table_name,
            splitters_table_name,
            models_table_name,
            transforms_table_name,
            chunks_table_name,
        ) = Self::generate_table_names(&name);
        let collection = Self {
            name,
            pool,
            documents_table_name,
            splitters_table_name,
            models_table_name,
            transforms_table_name,
            chunks_table_name,
            active: true,
        };
        collection.create_documents_table().await?;
        collection.create_splitter_table().await?;
        collection.create_models_table().await?;
        collection.create_transforms_table().await?;
        collection.create_chunks_table().await?;
        collection.register_text_splitter(None, None).await?;
        collection.register_model(None, None, None).await?;
        Ok(collection)
    }

    async fn create_documents_table(&self) -> anyhow::Result<()> {
        sqlx::query(&query_builder!("CREATE SCHEMA IF NOT EXISTS %s", self.name))
            .execute(self.pool.borrow())
            .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_DOCUMENTS_TABLE,
            self.documents_table_name
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "created_at_index",
            self.documents_table_name,
            "created_at"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX_USING_GIN,
            "metadata_index",
            self.documents_table_name,
            "metadata jsonb_path_ops"
        ))
        .execute(self.pool.borrow())
        .await?;
        Ok(())
    }

    async fn create_splitter_table(&self) -> anyhow::Result<()> {
        sqlx::query(&query_builder!(
            queries::CREATE_SPLITTERS_TABLE,
            self.splitters_table_name
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "created_at_index",
            self.splitters_table_name,
            "created_at"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "name_index",
            self.splitters_table_name,
            "name"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX_USING_GIN,
            "parameters_index",
            self.splitters_table_name,
            "parameters jsonb_path_ops"
        ))
        .execute(self.pool.borrow())
        .await?;
        Ok(())
    }

    async fn create_models_table(&self) -> anyhow::Result<()> {
        sqlx::query(&query_builder!(
            queries::CREATE_MODELS_TABLE,
            self.models_table_name
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "created_at_index",
            self.models_table_name,
            "created_at"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "task_index",
            self.models_table_name,
            "task"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "name_index",
            self.models_table_name,
            "name"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX_USING_GIN,
            "parameters_index",
            self.models_table_name,
            "parameters jsonb_path_ops"
        ))
        .execute(self.pool.borrow())
        .await?;
        Ok(())
    }

    async fn create_transforms_table(&self) -> anyhow::Result<()> {
        sqlx::query(&query_builder!(
            queries::CREATE_TRANSFORMS_TABLE,
            self.transforms_table_name,
            self.splitters_table_name,
            self.models_table_name
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "created_at_index",
            self.transforms_table_name,
            "created_at"
        ))
        .execute(self.pool.borrow())
        .await?;
        Ok(())
    }

    async fn create_chunks_table(&self) -> anyhow::Result<()> {
        sqlx::query(&query_builder!(
            queries::CREATE_CHUNKS_TABLE,
            self.chunks_table_name,
            self.documents_table_name,
            self.splitters_table_name
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "created_at_index",
            self.chunks_table_name,
            "created_at"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "document_id_index",
            self.chunks_table_name,
            "document_id"
        ))
        .execute(self.pool.borrow())
        .await?;
        sqlx::query(&query_builder!(
            queries::CREATE_INDEX,
            "splitter_id_index",
            self.chunks_table_name,
            "splitter_id"
        ))
        .execute(self.pool.borrow())
        .await?;
        Ok(())
    }

    pub async fn upsert_documents(
        &self,
        documents: Vec<HashMap<String, String>>,
        text_key: Option<String>,
        id_key: Option<String>,
    ) -> anyhow::Result<()> {
        let text_key = text_key.unwrap_or("text".to_string());
        let id_key = id_key.unwrap_or("id".to_string());
        for mut document in documents {
            let text = match document.remove(&text_key) {
                Some(t) => t,
                None => {
                    warn!("{} is not a key in document", text_key);
                    continue;
                }
            };

            let document_json = serde_json::to_value(&document)?;

            let md5_digest = match document.get(&id_key) {
                Some(k) => md5::compute(k.as_bytes()),
                None => md5::compute(format!("{}{}", text, document_json).as_bytes()),
            };
            let source_uuid = uuid::Uuid::from_slice(&md5_digest.0)?;

            sqlx::query(&query_builder!(
                    "INSERT INTO %s (text, source_uuid, metadata) VALUES ($1, $2, $3) ON CONFLICT (source_uuid) DO UPDATE SET text = $4, metadata = $5",
                    self.documents_table_name
                ))
                .bind(&text)
                .bind(source_uuid)
                .bind(&document_json)
                .bind(&text)
                .bind(&document_json)
            .execute(self.pool.borrow())
            .await?;
        }
        Ok(())
    }

    pub async fn register_text_splitter(
        &self,
        splitter_name: Option<String>,
        splitter_params: Option<HashMap<String, String>>,
    ) -> anyhow::Result<()> {
        let splitter_name = splitter_name.unwrap_or("recursive_character".to_string());

        let splitter_params = match splitter_params {
            Some(params) => serde_json::to_value(params)?,
            None => serde_json::json!({}),
        };

        let current_splitter: Option<models::Splitter> = sqlx::query_as(&query_builder!(
            "SELECT * from %s where name = $1 and parameters = $2;",
            self.splitters_table_name
        ))
        .bind(&splitter_name)
        .bind(&splitter_params)
        .fetch_optional(self.pool.borrow())
        .await?;

        match current_splitter {
            Some(_splitter) => {
                warn!(
                    "Text splitter with name: {} and parameters: {:?} already exists",
                    splitter_name, splitter_params
                );
            }
            None => {
                sqlx::query(&query_builder!(
                    "INSERT INTO %s (name, parameters) VALUES ($1, $2)",
                    self.splitters_table_name
                ))
                .bind(splitter_name)
                .bind(splitter_params)
                .execute(self.pool.borrow())
                .await?;
            }
        }
        Ok(())
    }

    pub async fn get_text_splitters(&self) -> anyhow::Result<Vec<models::Splitter>> {
        Ok(sqlx::query_as(&query_builder!(
            "SELECT * from %s",
            self.splitters_table_name
        ))
        .fetch_all(self.pool.borrow())
        .await?)
    }

    pub async fn generate_chunks(&self, splitter_id: Option<i64>) -> anyhow::Result<()> {
        let splitter_id = splitter_id.unwrap_or(1);
        sqlx::query(&query_builder!(
            queries::GENERATE_CHUNKS,
            self.splitters_table_name,
            self.chunks_table_name,
            self.documents_table_name,
            self.chunks_table_name
        ))
        .bind(splitter_id)
        .execute(self.pool.borrow())
        .await?;
        Ok(())
    }

    pub async fn register_model(
        &self,
        task: Option<String>,
        model_name: Option<String>,
        model_params: Option<HashMap<String, String>>,
    ) -> anyhow::Result<()> {
        let task = task.unwrap_or("embedding".to_string());
        let model_name = model_name.unwrap_or("intfloat/e5-small".to_string());
        let model_params = match model_params {
            Some(params) => serde_json::to_value(params)?,
            None => serde_json::json!({}),
        };

        let current_model: Option<models::Model> = sqlx::query_as(&query_builder!(
            "SELECT * from %s where task = $1 and name = $2 and parameters = $3;",
            self.models_table_name
        ))
        .bind(&task)
        .bind(&model_name)
        .bind(&model_params)
        .fetch_optional(self.pool.borrow())
        .await?;

        match current_model {
            Some(_model) => {
                warn!(
                    "Model with name: {} and parameters: {:?} already exists",
                    model_name, model_params
                );
            }
            None => {
                sqlx::query(&query_builder!(
                    "INSERT INTO %s (task, name, parameters) VALUES ($1, $2, $3)",
                    self.models_table_name
                ))
                .bind(task)
                .bind(model_name)
                .bind(model_params)
                .execute(self.pool.borrow())
                .await?;
            }
        }

        Ok(())
    }

    pub async fn get_models(&self) -> anyhow::Result<Vec<models::Model>> {
        Ok(
            sqlx::query_as(&query_builder!("SELECT * from %s", self.models_table_name))
                .fetch_all(self.pool.borrow())
                .await?,
        )
    }

    async fn create_or_get_embeddings_table(
        &self,
        model_id: i64,
        splitter_id: i64,
    ) -> anyhow::Result<String> {
        let table_name: Option<(String,)> = sqlx::query_as(&query_builder!(
                "SELECT table_name from %s WHERE task = 'embedding' AND model_id = $1 and splitter_id = $2", 
                self.transforms_table_name))
            .bind(model_id)
            .bind(splitter_id)
            .fetch_optional(self.pool.borrow()).await?;
        match table_name {
            Some((name,)) => Ok(name),
            None => {
                let table_name = format!(
                    "{}.embeddings_{}",
                    self.name,
                    &uuid::Uuid::new_v4().to_string()[0..6]
                );
                let (embedding,): (Vec<f32>,) = sqlx::query_as(&query_builder!(
                    "SELECT embedding from pgml.embed(transformer => (SELECT name from %s where id = $1), text => 'Hello, World!', kwargs => '{}') as embedding", 
                    self.models_table_name))
                    .bind(model_id)
                .fetch_one(self.pool.borrow())
                .await?;
                let embedding_length = embedding.len() as i64;
                sqlx::query(&query_builder!(
                    queries::CREATE_EMBEDDINGS_TABLE,
                    table_name,
                    self.chunks_table_name,
                    embedding_length.to_string()
                ))
                .execute(self.pool.borrow())
                .await?;
                sqlx::query(&query_builder!(
                    "INSERT INTO %s (table_name, task, model_id, splitter_id) VALUES ($1, 'embedding', $2, $3)",
                    self.transforms_table_name))
                    .bind(&table_name)
                    .bind(model_id)
                    .bind(splitter_id)
                .execute(self.pool.borrow())
                .await?;
                sqlx::query(&query_builder!(
                    queries::CREATE_INDEX,
                    "created_at_index",
                    table_name,
                    "created_at"
                ))
                .execute(self.pool.borrow())
                .await?;
                sqlx::query(&query_builder!(
                    queries::CREATE_INDEX,
                    "chunk_id_index",
                    table_name,
                    "chunk_id"
                ))
                .execute(self.pool.borrow())
                .await?;
                sqlx::query(&query_builder!(
                    queries::CREATE_INDEX_USING_IVFFLAT,
                    "vector_index",
                    table_name,
                    "embedding vector_cosine_ops"
                ))
                .execute(self.pool.borrow())
                .await?;
                Ok(table_name)
            }
        }
    }

    pub async fn generate_embeddings(
        &self,
        model_id: Option<i64>,
        splitter_id: Option<i64>,
    ) -> anyhow::Result<()> {
        let model_id = model_id.unwrap_or(1);
        let splitter_id = splitter_id.unwrap_or(1);

        let embeddings_table_name = self
            .create_or_get_embeddings_table(model_id, splitter_id)
            .await?;

        sqlx::query(&query_builder!(
            queries::GENERATE_EMBEDDINGS,
            self.models_table_name,
            embeddings_table_name,
            self.chunks_table_name,
            embeddings_table_name
        ))
        .bind(model_id)
        .bind(splitter_id)
        .execute(self.pool.borrow())
        .await?;

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub async fn vector_search(
        &self,
        query: &str,
        query_parameters: Option<HashMap<String, String>>,
        top_k: Option<i64>,
        model_id: Option<i64>,
        splitter_id: Option<i64>,
    ) -> anyhow::Result<Vec<(f64, String, HashMap<String, String>)>> {
        //embedding_table_name as (SELECT table_name from %s WHERE task = 'embedding' AND model_id = %s and splitter_id = %s) \
        let query_parameters = match query_parameters {
            Some(params) => serde_json::to_value(params)?,
            None => serde_json::json!({}),
        };

        let top_k = top_k.unwrap_or(5);
        let model_id = model_id.unwrap_or(1);
        let splitter_id = splitter_id.unwrap_or(1);

        // TODO: Talk with lev about turning this into some kind of CTE and merging with the query
        // below
        let embeddings_table_name: Option<(String,)> = sqlx::query_as(&query_builder!(
                "SELECT table_name from %s WHERE task = 'embedding' AND model_id = $1 and splitter_id = $2", 
                self.transforms_table_name
            ))
            .bind(model_id)
            .bind(splitter_id)
        .fetch_optional(self.pool.borrow())
        .await?;
        let embeddings_table_name = match embeddings_table_name {
            Some((table_name,)) => table_name,
            None => {
                return Err(anyhow!(format!("Embeddings table does not exist for task: embedding model_id: {} and splitter_id: {}", model_id, splitter_id)))
            }
        };

        let results: Vec<(f64, String, Json<HashMap<String, String>>)> =
            sqlx::query_as(&query_builder!(
                queries::VECTOR_SEARCH,
                self.models_table_name,
                embeddings_table_name,
                embeddings_table_name,
                self.chunks_table_name,
                self.documents_table_name
            ))
            .bind(model_id)
            .bind(query)
            .bind(query_parameters)
            .bind(top_k)
            .fetch_all(self.pool.borrow())
            .await?;
        let results: Vec<(f64, String, HashMap<String, String>)> =
            results.into_iter().map(|r| (r.0, r.1, r.2 .0)).collect();
        Ok(results)
    }

    pub fn from_model_and_pool(collection_model: models::Collection, pool: Arc<PgPool>) -> Self {
        let (
            documents_table_name,
            splitters_table_name,
            models_table_name,
            transforms_table_name,
            chunks_table_name,
        ) = Self::generate_table_names(&collection_model.name);
        Self {
            name: collection_model.name,
            pool,
            documents_table_name,
            splitters_table_name,
            models_table_name,
            transforms_table_name,
            chunks_table_name,
            active: collection_model.active,
        }
    }

    fn generate_table_names(name: &str) -> (String, String, String, String, String) {
        [
            ".documents",
            ".splitters",
            ".models",
            ".transforms",
            ".chunks",
        ]
        .into_iter()
        .map(|s| format!("{}{}", name, s))
        .collect_tuple()
        .unwrap()
    }
}
