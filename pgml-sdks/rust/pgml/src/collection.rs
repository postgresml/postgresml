use itertools::Itertools;
use log::warn;
use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::PgPool;
use sqlx::Executor;
use std::borrow::Borrow;

use crate::models;
use crate::queries;
use crate::query_builder;
use crate::query_builder::QueryBuilder;
use crate::remote_embeddings::build_remote_embeddings;
use crate::splitter::Splitter;
use crate::types::Json;

#[cfg(feature = "javascript")]
use crate::{languages::javascript::*, splitter::SplitterJavascript};

#[cfg(feature = "python")]
use crate::{query_builder::QueryBuilderPython, splitter::SplitterPython};

/// A collection of documents
#[derive(custom_derive, Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub pool: PgPool,
    pub documents_table_name: String,
    pub splitters_table_name: String,
    pub models_table_name: String,
    pub transforms_table_name: String,
    pub chunks_table_name: String,
    pub documents_tsvectors_table_name: String,
}

#[custom_methods(
    upsert_documents,
    get_text_splitters,
    generate_chunks,
    get_models,
    generate_embeddings,
    generate_tsvectors,
    vector_search,
    query
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
            documents_tsvectors_table_name,
        ) = Self::generate_table_names(&name);
        let collection = Self {
            name,
            pool,
            documents_table_name,
            splitters_table_name,
            models_table_name,
            transforms_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
        };
        sqlx::query("INSERT INTO pgml.collections (name, active) VALUES ($1, FALSE) ON CONFLICT (name) DO NOTHING")
            .bind(&collection.name)
            .execute(&collection.pool)
            .await?;
        collection.create_documents_table().await?;
        // collection.create_splitter_table().await?;
        // collection.create_models_table().await?;
        collection.create_transforms_table().await?;
        collection.create_chunks_table().await?;
        collection.create_documents_tsvectors_table().await?;
        sqlx::query("UPDATE pgml.collections SET active = TRUE WHERE name = $1")
            .bind(&collection.name)
            .execute(&collection.pool)
            .await?;
        Ok(collection)
    }

    async fn create_documents_table(&self) -> anyhow::Result<()> {
        self.pool
            .execute(query_builder!("CREATE SCHEMA IF NOT EXISTS %s", self.name).as_str())
            .await?;
        self.pool
            .execute(
                query_builder!(queries::CREATE_DOCUMENTS_TABLE, self.documents_table_name).as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "created_at_index",
                    self.documents_table_name,
                    "created_at"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
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
        self.pool
            .execute(
                query_builder!(queries::CREATE_SPLITTERS_TABLE, self.splitters_table_name).as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "created_at_index",
                    self.splitters_table_name,
                    "created_at"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "name_index",
                    self.splitters_table_name,
                    "name"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
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
        self.pool
            .execute(query_builder!(queries::CREATE_MODELS_TABLE, self.models_table_name).as_str())
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "created_at_index",
                    self.models_table_name,
                    "created_at"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "task_index",
                    self.models_table_name,
                    "task"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "name_index",
                    self.models_table_name,
                    "name"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
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
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_TRANSFORMS_TABLE,
                    self.transforms_table_name,
                    self.splitters_table_name,
                    self.models_table_name
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
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
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_CHUNKS_TABLE,
                    self.chunks_table_name,
                    self.documents_table_name,
                    self.splitters_table_name
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "created_at_index",
                    self.chunks_table_name,
                    "created_at"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "document_id_index",
                    self.chunks_table_name,
                    "document_id"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
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

    async fn create_documents_tsvectors_table(&self) -> anyhow::Result<()> {
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_DOCUMENTS_TSVECTORS_TABLE,
                    self.documents_tsvectors_table_name,
                    self.documents_table_name
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX,
                    "configuration_index",
                    self.documents_tsvectors_table_name,
                    "configuration"
                )
                .as_str(),
            )
            .await?;
        self.pool
            .execute(
                query_builder!(
                    queries::CREATE_INDEX_USING_GIN,
                    "tsvector_index",
                    self.documents_tsvectors_table_name,
                    "ts"
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
    /// use pgml::types::Json;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents: Vec<Json> = vec![
    ///        serde_json::json!( {
    ///            "id": 1,
    ///            "text": "This is a document"
    ///        })
    ///        .into()
    ///    ];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn upsert_documents(
        &self,
        documents: Vec<Json>,
        text_key: Option<String>,
        id_key: Option<String>,
    ) -> anyhow::Result<()> {
        let text_key = text_key.unwrap_or("text".to_string());
        let id_key = id_key.unwrap_or("id".to_string());

        for mut document in documents {
            let document = document
                .0
                .as_object_mut()
                .expect("Documents must be a vector of objects");

            let text = match document.remove(&text_key) {
                Some(t) => t,
                None => {
                    warn!("{} is not a key in document", text_key);
                    continue;
                }
            };

            let document_json = serde_json::to_value(&document)?;

            let md5_digest = match document.get(&id_key) {
                Some(k) => md5::compute(k.to_string().as_bytes()),
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

    pub async fn generate_tsvectors(&self, configuration: Option<String>) -> anyhow::Result<()> {
        let (count,): (i64,) = sqlx::query_as(&query_builder!(
            "SELECT count(*) FROM (SELECT 1 FROM %s LIMIT 1) AS t",
            self.documents_table_name
        ))
        .fetch_one(&self.pool)
        .await?;

        if count == 0 {
            anyhow::bail!("No documents in the documents table. Make sure to upsert documents before generating tsvectors")
        }

        let configuration = configuration.unwrap_or("english".to_string());
        sqlx::query(&query_builder!(
            queries::GENERATE_TSVECTORS,
            self.documents_tsvectors_table_name,
            configuration,
            configuration,
            self.documents_table_name
        ))
        .execute(&self.pool)
        .await?;
        Ok(())
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
    /// use pgml::types::Json;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents: Vec<Json> = vec![
    ///        serde_json::json!( {
    ///            "id": 1,
    ///            "text": "This is a document"
    ///        })
    ///        .into()
    ///    ];
    ///    collection
    ///        .upsert_documents(documents, None, None)
    ///        .await?;
    ///    collection.generate_chunks(None).await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn generate_chunks(&self, splitter: Splitter) -> anyhow::Result<()> {
        let (count,): (i64,) = sqlx::query_as(&query_builder!(
            "SELECT count(*) FROM (SELECT 1 FROM %s LIMIT 1) AS t",
            self.documents_table_name
        ))
        .fetch_one(&self.pool)
        .await?;

        if count == 0 {
            anyhow::bail!("No documents in the documents table. Make sure to upsert documents before generating chunks")
        }

        let splitter_id = splitter.id;
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

    async fn create_or_get_embeddings_table(
        &self,
        model: &models::Model,
        splitter_id: i64,
    ) -> anyhow::Result<String> {
        let table_name = self.get_embeddings_table_name(model.id, splitter_id)?;
        let exists: Option<(String,)> = sqlx::query_as(&query_builder!(
            "SELECT table_name from %s WHERE table_name = $1",
            self.transforms_table_name
        ))
        .bind(&table_name)
        .fetch_optional(self.pool.borrow())
        .await?;

        match exists {
            Some(_e) => Ok(table_name),
            None => {
                let embedding_length = match model.source.as_str() {
                    "huggingface" => {
                        let embedding: (Vec<f32>,) = sqlx::query_as(&query_builder!(
                        "WITH model as (SELECT name, parameters from %s where id = $1) SELECT embedding from pgml.embed(transformer => (SELECT name FROM model), text => 'Hello, World!', kwargs => (SELECT parameters FROM model)) as embedding", 
                        self.models_table_name))
                        .bind(model.id)
                        .fetch_one(&self.pool).await?;
                        embedding.0.len() as i64
                    }
                    t @ _ => {
                        let remote_embeddings = build_remote_embeddings(t, &model.name)?;
                        remote_embeddings.get_embedding_size().await?
                    }
                };
                self.pool
                    .execute(
                        query_builder!(
                            queries::CREATE_EMBEDDINGS_TABLE,
                            table_name,
                            self.chunks_table_name,
                            embedding_length
                        )
                        .as_str(),
                    )
                    .await?;
                sqlx::query(&query_builder!(
                    "INSERT INTO %s (table_name, task, model_id, splitter_id) VALUES ($1, 'embedding', $2, $3)",
                    self.transforms_table_name))
                    .bind(&table_name)
                    .bind(model.id)
                    .bind(splitter_id)
                    .execute(&self.pool).await?;
                self.pool
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX,
                            "created_at_index",
                            table_name,
                            "created_at"
                        )
                        .as_str(),
                    )
                    .await?;
                self.pool
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX,
                            "chunk_id_index",
                            table_name,
                            "chunk_id"
                        )
                        .as_str(),
                    )
                    .await?;
                self.pool
                    .execute(
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
    /// use pgml::types::Json;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents: Vec<Json> = vec![
    ///        serde_json::json!( {
    ///            "id": 1,
    ///            "text": "This is a document"
    ///        })
    ///        .into()
    ///    ];
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

        let (count,): (i64,) = sqlx::query_as(&query_builder!(
            "SELECT count(*) FROM (SELECT 1 FROM %s WHERE splitter_id = $1 LIMIT 1) AS t",
            self.chunks_table_name
        ))
        .bind(splitter_id)
        .fetch_one(&self.pool)
        .await?;

        if count == 0 {
            anyhow::bail!("No chunks in the chunks table with the associated splitter_id. Make sure to generate chunks with the correct splitter_id before generating embeddings")
        }

        let model: models::Model = sqlx::query_as(&query_builder!(
            "SELECT * from %s where id = $1",
            self.models_table_name
        ))
        .bind(model_id)
        .fetch_optional(&self.pool)
        .await?
        .expect("Model not found. Please double check your model_id is correct");

        let embeddings_table_name = self
            .create_or_get_embeddings_table(&model, splitter_id)
            .await?;

        match model.source.as_str() {
            "huggingface" => {
                sqlx::query(&query_builder!(
                    queries::GENERATE_EMBEDDINGS,
                    embeddings_table_name,
                    self.chunks_table_name,
                    embeddings_table_name
                ))
                .bind(model.name)
                .bind(model.parameters)
                .bind(splitter_id)
                .execute(self.pool.borrow())
                .await?;
            }
            t @ _ => {
                let remote_embeddings = build_remote_embeddings(t, &model.name)?;
                remote_embeddings
                    .generate_embeddings(
                        &embeddings_table_name,
                        &self.chunks_table_name,
                        splitter_id,
                        &self.pool,
                    )
                    .await?;
            }
        }

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
    /// use pgml::types::Json;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    let documents: Vec<Json> = vec![
    ///        serde_json::json!( {
    ///            "id": 1,
    ///            "text": "This is a document"
    ///        })
    ///        .into()
    ///    ];
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
        query_params: Option<Json>,
        top_k: Option<i64>,
        model_id: Option<i64>,
        splitter_id: Option<i64>,
    ) -> anyhow::Result<Vec<(f64, String, Json)>> {
        let query_params = match query_params {
            Some(params) => params.0,
            None => serde_json::json!({}),
        };
        let top_k = top_k.unwrap_or(5);
        let model_id = model_id.unwrap_or(1);
        let splitter_id = splitter_id.unwrap_or(1);

        let embeddings_table_name = self.get_embeddings_table_name(model_id, splitter_id)?;

        let results: Vec<(f64, String, Json)> = sqlx::query_as(&query_builder!(
            queries::VECTOR_SEARCH,
            self.models_table_name,
            embeddings_table_name,
            embeddings_table_name,
            self.chunks_table_name,
            self.documents_table_name
        ))
        .bind(query)
        .bind(query_params)
        .bind(model_id)
        .bind(top_k)
        .fetch_all(self.pool.borrow())
        .await?;
        Ok(results)
    }

    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(self.clone())
    }

    // We will probably want to add a task parameter to this function
    pub fn get_embeddings_table_name(
        &self,
        model_id: i64,
        splitter_id: i64,
    ) -> anyhow::Result<String> {
        let model_splitter_hash = md5::compute(format!("{}_{}", model_id, splitter_id).as_bytes());
        Ok(format!(
            "{}.embeddings_{}",
            self.name,
            &uuid::Uuid::from_slice(&model_splitter_hash.0)?
        ))
    }

    pub fn from_model_and_pool(model: models::Collection, pool: PgPool) -> Self {
        let (
            documents_table_name,
            splitters_table_name,
            models_table_name,
            transforms_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
        ) = Self::generate_table_names(&model.name);
        Self {
            name: model.name,
            documents_table_name,
            splitters_table_name,
            models_table_name,
            transforms_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
            pool,
        }
    }

    fn generate_table_names(name: &str) -> (String, String, String, String, String, String) {
        [
            ".documents",
            ".splitters",
            ".models",
            ".transforms",
            ".chunks",
            ".documents_tsvectors",
        ]
        .into_iter()
        .map(|s| format!("{}{}", name, s))
        .collect_tuple()
        .unwrap()
    }
}
