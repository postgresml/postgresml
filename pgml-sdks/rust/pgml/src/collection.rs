use anyhow::anyhow;
use itertools::Itertools;
use log::warn;
use neon::prelude::*;
use pgml_macros::{custom_derive, custom_methods};
use pyo3::prelude::*;
use sqlx::postgres::PgPool;
use sqlx::types::Json;
use sqlx::Executor;
use std::borrow::Borrow;
use std::collections::HashMap;

use crate::languages::javascript::*;
use crate::models;
use crate::queries;
use crate::query_builder;

/// A collection of documents
#[derive(custom_derive, Debug, Clone)]
pub struct Collection {
    name: String,
    pool: PgPool,
    documents_table_name: String,
    splitters_table_name: String,
    models_table_name: String,
    transforms_table_name: String,
    chunks_table_name: String,
    #[allow(unused)]
    active: bool,
}

#[custom_methods(
    upsert_documents,
    register_text_splitter,
    get_text_splitters,
    generate_chunks,
    register_model,
    get_models,
    generate_embeddings,
    vector_search
)]
impl Collection {
    /// Creates a new collection
    ///
    /// This should not be called directly. Use [crate::Database::create_or_get_collection] instead.
    ///
    /// Note that a default text splitter and model are created automatically.
    pub async fn new(name: String, pool: PgPool) -> anyhow::Result<Self> {
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
        let pool = self.pool.borrow();
        pool.execute(query_builder!("CREATE SCHEMA IF NOT EXISTS %s", self.name).as_str())
            .await?;
        pool.execute(
            query_builder!(queries::CREATE_DOCUMENTS_TABLE, self.documents_table_name).as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "created_at_index",
                self.documents_table_name,
                "created_at"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX_USING_GIN,
                "metadata_index",
                self.documents_table_name,
                "metadata jsonb_path_ops"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    async fn create_splitter_table(&self) -> anyhow::Result<()> {
        let pool = self.pool.borrow();
        pool.execute(
            query_builder!(queries::CREATE_SPLITTERS_TABLE, self.splitters_table_name).as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "created_at_index",
                self.splitters_table_name,
                "created_at"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "name_index",
                self.splitters_table_name,
                "name"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX_USING_GIN,
                "parameters_index",
                self.splitters_table_name,
                "parameters jsonb_path_ops"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    async fn create_models_table(&self) -> anyhow::Result<()> {
        let pool = self.pool.borrow();
        pool.execute(query_builder!(queries::CREATE_MODELS_TABLE, self.models_table_name).as_str())
            .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "created_at_index",
                self.models_table_name,
                "created_at"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "task_index",
                self.models_table_name,
                "task"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "name_index",
                self.models_table_name,
                "name"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX_USING_GIN,
                "parameters_index",
                self.models_table_name,
                "parameters jsonb_path_ops"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    async fn create_transforms_table(&self) -> anyhow::Result<()> {
        let pool = self.pool.borrow();
        pool.execute(
            query_builder!(
                queries::CREATE_TRANSFORMS_TABLE,
                self.transforms_table_name,
                self.splitters_table_name,
                self.models_table_name
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "created_at_index",
                self.transforms_table_name,
                "created_at"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    async fn create_chunks_table(&self) -> anyhow::Result<()> {
        let pool = self.pool.borrow();
        pool.execute(
            query_builder!(
                queries::CREATE_CHUNKS_TABLE,
                self.chunks_table_name,
                self.documents_table_name,
                self.splitters_table_name
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "created_at_index",
                self.chunks_table_name,
                "created_at"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "document_id_index",
                self.chunks_table_name,
                "document_id"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "splitter_id_index",
                self.chunks_table_name,
                "splitter_id"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }

    /// Upserts documents into the database
    ///
    /// # Arguments
    ///
    /// * `documents` - A vector of documents to upsert.
    /// * `text_key` - The key in the document that contains the text.
    /// * `id_key` - The key in the document that contains the id.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents = vec![HashMap::from([
    ///        ("id".to_string(), "1".to_string()),
    ///        ("text".to_string(), "This is a document".to_string()),
    ///    ])];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    Ok(())
    /// }
    /// ```
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
                .execute(self.pool.borrow()).await?;
        }
        Ok(())
    }

    /// Registers new text splitters
    ///
    /// # Arguments
    ///
    /// * `splitter_name` - The name of the text splitter.
    /// * `splitter_params` - A [std::collections::HashMap] of parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    collection.register_text_splitter(None, None).await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn register_text_splitter(
        &self,
        splitter_name: Option<String>,
        splitter_params: Option<HashMap<String, String>>,
    ) -> anyhow::Result<i64> {
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
            Some(splitter) => {
                warn!(
                    "Text splitter with name: {} and parameters: {:?} already exists",
                    splitter_name, splitter_params
                );
		Ok(splitter.id)
            }
            None => {
                let splitter_id: (i64,) = sqlx::query_as(&query_builder!(
                    "INSERT INTO %s (name, parameters) VALUES ($1, $2) RETURNING id",
                    self.splitters_table_name
                ))
                .bind(splitter_name)
                .bind(splitter_params)
                .fetch_one(self.pool.borrow())
                .await?;
		Ok(splitter_id.0)
            }
        }
    }

    /// Gets all registered text [models::Splitter]s
    pub async fn get_text_splitters(&self) -> anyhow::Result<Vec<models::Splitter>> {
        let splitters: Vec<models::Splitter> = sqlx::query_as(&query_builder!(
            "SELECT * from %s",
            self.splitters_table_name
        ))
        .fetch_all(self.pool.borrow())
        .await?;
        Ok(splitters)
    }

    /// Generates chunks for the collection
    ///
    /// # Arguments
    ///
    /// * `splitter_id` - The id of the splitter to chunk with.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents = vec![HashMap::from([
    ///        ("id".to_string(), "1".to_string()),
    ///        ("text".to_string(), "This is a document".to_string()),
    ///    ])];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    collection.generate_chunks(None).await?;
    ///    Ok(())
    /// }
    /// ```
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

    /// Registers new models for specific tasks
    ///
    /// # Arguments
    ///
    /// * `task` - The name of the task.
    /// * `model_name` - The name of the [models::Model].
    /// * `model_params` - A [std::collections::HashMap] of parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    collection.register_model(None, None, None).await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn register_model(
        &self,
        task: Option<String>,
        model_name: Option<String>,
        model_params: Option<HashMap<String, String>>,
    ) -> anyhow::Result<i64> {
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
            Some(model) => {
                warn!(
                    "Model with name: {} and parameters: {:?} already exists",
                    model_name, model_params
                );
                Ok(model.id)
            }
            None => {
                let id: (i64,) = sqlx::query_as(&query_builder!(
                    "INSERT INTO %s (task, name, parameters) VALUES ($1, $2, $3) RETURNING id",
                    self.models_table_name
                ))
                .bind(task)
                .bind(model_name)
                .bind(model_params)
                .fetch_one(self.pool.borrow())
                .await?;
                Ok(id.0)
            }
        }
    }

    /// Gets all registered [models::Model]s
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
        let pool = self.pool.borrow();
        let table_name: Option<(String,)> =
            sqlx::query_as(&query_builder!(
                "SELECT table_name from %s WHERE task = 'embedding' AND model_id = $1 and splitter_id = $2", 
                self.transforms_table_name))
            .bind(model_id)
            .bind(splitter_id)
            .fetch_optional(pool).await?;
        match table_name {
            Some((name,)) => Ok(name),
            None => {
                let table_name = format!(
                    "{}.embeddings_{}",
                    self.name,
                    &uuid::Uuid::new_v4().to_string()[0..6]
                );
                let embedding: (Vec<f32>,) = sqlx::query_as(&query_builder!(
                    "WITH model as (SELECT name, parameters from %s where id = $1) SELECT embedding from pgml.embed(transformer => (SELECT name FROM model), text => 'Hello, World!', kwargs => (SELECT parameters FROM model)) as embedding", 
                    self.models_table_name))
                    .bind(model_id)
                    .fetch_one(pool).await?;
                let embedding = embedding.0;
                let embedding_length = embedding.len() as i64;
                pool.execute(
                    query_builder!(
                        queries::CREATE_EMBEDDINGS_TABLE,
                        table_name,
                        self.chunks_table_name,
                        embedding_length.to_string()
                    )
                    .as_str(),
                )
                .await?;
                sqlx::query(&query_builder!(
                    "INSERT INTO %s (table_name, task, model_id, splitter_id) VALUES ($1, 'embedding', $2, $3)",
                    self.transforms_table_name))
                    .bind(&table_name)
                    .bind(model_id)
                    .bind(splitter_id)
                    .execute(pool).await?;
                pool.execute(
                    query_builder!(
                        queries::CREATE_INDEX,
                        "created_at_index",
                        table_name,
                        "created_at"
                    )
                    .as_str(),
                )
                .await?;
                pool.execute(
                    query_builder!(
                        queries::CREATE_INDEX,
                        "chunk_id_index",
                        table_name,
                        "chunk_id"
                    )
                    .as_str(),
                )
                .await?;
                pool.execute(
                    query_builder!(
                        queries::CREATE_INDEX_USING_IVFFLAT,
                        "vector_index",
                        table_name,
                        "embedding vector_cosine_ops"
                    )
                    .as_str(),
                )
                .await?;
                Ok(table_name)
            }
        }
    }

    /// Generates embeddings for the collection
    ///
    /// # Arguments
    ///
    /// * `model_id` - The id of the model.
    /// * `splitter_id` - The id of the splitter.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents = vec![HashMap::from([
    ///        ("id".to_string(), "1".to_string()),
    ///        ("text".to_string(), "This is a document".to_string()),
    ///    ])];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    collection.generate_chunks(None).await?;
    ///    collection.generate_embeddings(None, None).await?;
    ///    Ok(())
    /// }
    /// ```
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

    /// Performs vector search on the [Collection]
    ///
    /// # Arguments
    ///
    /// * `query` - The query to search for.
    /// * `query_params` - A [std::collections::HashMap] of parameters for the model used in the
    /// query.
    /// * `top_k` - How many results to limit on.
    /// * `model_id` - The id of the [models::Model] to use.
    /// * `splitter_id` - The id of the [models::Splitter] to use.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents = vec![HashMap::from([
    ///        ("id".to_string(), "1".to_string()),
    ///        ("text".to_string(), "This is a document".to_string()),
    ///    ])];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    collection.generate_chunks(None).await?;
    ///    collection.generate_embeddings(None, None).await?;
    ///    let results = collection
    ///        .vector_search("Here is a test", None, None, None, None)
    ///        .await
    ///        .unwrap();
    ///    println!("The results are: {:?}", results);
    ///    Ok(())
    /// }
    /// ```
    #[allow(clippy::type_complexity)]
    pub async fn vector_search(
        &self,
        query: &str,
        query_params: Option<HashMap<String, String>>,
        top_k: Option<i64>,
        model_id: Option<i64>,
        splitter_id: Option<i64>,
    ) -> anyhow::Result<Vec<(f64, String, HashMap<String, String>)>> {
        //embedding_table_name as (SELECT table_name from %s WHERE task = 'embedding' AND model_id = %s and splitter_id = %s) \
        let query_params = match query_params {
            Some(params) => serde_json::to_value(params)?,
            None => serde_json::json!({}),
        };

        let top_k = top_k.unwrap_or(5);
        let model_id = model_id.unwrap_or(1);
        let splitter_id = splitter_id.unwrap_or(1);

        let embeddings_table_name: Option<(String,)> = sqlx::query_as(&query_builder!(
                "SELECT table_name from %s WHERE task = 'embedding' AND model_id = $1 and splitter_id = $2", 
                self.transforms_table_name))
            .bind(model_id)
            .bind(splitter_id)
            .fetch_optional(self.pool.borrow()).await?;

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
            .bind(query_params)
            .bind(top_k)
            .fetch_all(self.pool.borrow())
            .await?;
        let results: Vec<(f64, String, HashMap<String, String>)> =
            results.into_iter().map(|r| (r.0, r.1, r.2 .0)).collect();
        Ok(results)
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
