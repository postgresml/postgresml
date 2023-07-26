use itertools::Itertools;
use log::warn;
use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::PgPool;
use sqlx::Executor;
use std::time::SystemTime;

use crate::get_or_initialize_pool;
use crate::model::Model;
use crate::queries;
use crate::query_builder;
use crate::query_builder::QueryBuilder;
use crate::remote_embeddings::build_remote_embeddings;
use crate::splitter::Splitter;
use crate::types::Json;

#[cfg(feature = "javascript")]
use crate::{languages::javascript::*, model::ModelJavascript, splitter::SplitterJavascript};

#[cfg(feature = "python")]
use crate::{model::ModelPython, query_builder::QueryBuilderPython, splitter::SplitterPython};

/// A collection of documents
#[derive(custom_derive, Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub database_url: Option<String>,
    pub documents_table_name: String,
    pub transforms_table_name: String,
    pub chunks_table_name: String,
    pub documents_tsvectors_table_name: String,
    pub verified_in_database: bool,
}

#[custom_methods(
    new,
    upsert_documents,
    generate_chunks,
    generate_embeddings,
    generate_tsvectors,
    vector_search,
    query,
    archive,
    get_verified_in_database
)]
impl Collection {
    /// Creates a new collection
    ///
    /// This should not be called directly. Use [crate::Database::create_or_get_collection] instead.
    ///
    /// Note that a default text splitter and model are created automatically.
    pub fn new(name: &str, database_url: Option<String>) -> Self {
        let (
            documents_table_name,
            transforms_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
        ) = Self::generate_table_names(name);
        Self {
            name: name.to_string(),
            database_url,
            documents_table_name,
            transforms_table_name,
            chunks_table_name,
            documents_tsvectors_table_name,
            verified_in_database: false,
        }
    }

    // Unfortunately the async-recursion macro does not play nice with pyo3 so this function is a
    // bit more verbose than it otherwise could be
    async fn verify_in_database(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        if !self.verified_in_database {
            let result = sqlx::query(
                "INSERT INTO pgml.collections (name, active) VALUES ($1, TRUE) ON CONFLICT (name) DO NOTHING",
            )
            .bind(&self.name)
            .execute(pool)
            .await;

            match result {
                Ok(_r) => {
                    pool.execute(
                        query_builder!("CREATE SCHEMA IF NOT EXISTS %s", self.name).as_str(),
                    )
                    .await?;
                    self.verified_in_database = true;
                    Ok(())
                }
                Err(e) => {
                    match e.as_database_error() {
                        Some(db_e) => {
                            // Error 42P01 is "undefined_table"
                            if db_e.code() == Some(std::borrow::Cow::from("42P01")) {
                                sqlx::query(queries::CREATE_COLLECTIONS_TABLE)
                                    .execute(pool)
                                    .await?;
                                sqlx::query(
                                    "INSERT INTO pgml.collections (name, active) VALUES ($1, TRUE) ON CONFLICT (name) DO NOTHING",
                                )
                                .bind(&self.name)
                                .execute(pool)
                                .await?;
                                pool.execute(
                                    query_builder!("CREATE SCHEMA IF NOT EXISTS %s", self.name)
                                        .as_str(),
                                )
                                .await?;
                                self.verified_in_database = true;
                                Ok(())
                            } else {
                                Err(e.into())
                            }
                        }
                        None => Err(e.into())
                    }
                }
            }
        } else {
            Ok(())
        }
    }

    async fn create_documents_table(&mut self, pool: &PgPool) -> anyhow::Result<()> {
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

    async fn create_transforms_table(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        pool.execute(
            query_builder!(queries::CREATE_TRANSFORMS_TABLE, self.transforms_table_name).as_str(),
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

    async fn create_chunks_table(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        pool.execute(
            query_builder!(
                queries::CREATE_CHUNKS_TABLE,
                self.chunks_table_name,
                self.documents_table_name
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

    async fn create_documents_tsvectors_table(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        pool.execute(
            query_builder!(
                queries::CREATE_DOCUMENTS_TSVECTORS_TABLE,
                self.documents_tsvectors_table_name,
                self.documents_table_name
            )
            .as_str(),
        )
        .await?;
        pool.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "configuration_index",
                self.documents_tsvectors_table_name,
                "configuration"
            )
            .as_str(),
        )
        .await?;
        pool.execute(
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
        &mut self,
        documents: Vec<Json>,
        text_key: Option<String>,
        id_key: Option<String>,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(&pool).await?;
        self.create_documents_table(&pool).await?;

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
                .execute(&pool).await?;
        }
        Ok(())
    }

    pub async fn generate_tsvectors(
        &mut self,
        configuration: Option<String>,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(&pool).await?;
        self.create_documents_tsvectors_table(&pool).await?;
        let (count,): (i64,) = sqlx::query_as(&query_builder!(
            "SELECT count(*) FROM (SELECT 1 FROM %s LIMIT 1) AS t",
            self.documents_table_name
        ))
        .fetch_one(&pool)
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
        .execute(&pool)
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
    pub async fn generate_chunks(&mut self, splitter: &mut Splitter) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(&pool).await?;
        self.create_chunks_table(&pool).await?;
        let (count,): (i64,) = sqlx::query_as(&query_builder!(
            "SELECT count(*) FROM (SELECT 1 FROM %s LIMIT 1) AS t",
            self.documents_table_name
        ))
        .fetch_one(&pool)
        .await?;

        if count == 0 {
            anyhow::bail!("No documents in the documents table. Make sure to upsert documents before generating chunks")
        }

        sqlx::query(&query_builder!(
            queries::GENERATE_CHUNKS,
            self.chunks_table_name,
            self.documents_table_name,
            self.chunks_table_name
        ))
        .bind(splitter.get_id().await?)
        .bind(&splitter.name)
        .bind(&splitter.parameters)
        .execute(&pool)
        .await?;
        Ok(())
    }

    async fn create_or_get_embeddings_table(
        &mut self,
        model: &mut Model,
        splitter: &mut Splitter,
    ) -> anyhow::Result<String> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.create_transforms_table(&pool).await?;
        let table_name = self.get_embeddings_table_name(model, splitter)?;
        let exists: Option<(String,)> = sqlx::query_as(&query_builder!(
            "SELECT table_name from %s WHERE table_name = $1",
            self.transforms_table_name
        ))
        .bind(&table_name)
        .fetch_optional(&pool)
        .await?;

        match exists {
            Some(_e) => Ok(table_name),
            None => {
                let embedding_length = match model.source.as_str() {
                    "pgml" => {
                        let embedding: (Vec<f32>,) = sqlx::query_as(
                        "SELECT embedding from pgml.embed(transformer => $1, text => 'Hello, World!', kwargs => $2) as embedding") 
                        .bind(&model.name)
                        .bind(&model.parameters)
                        .fetch_one(&pool).await?;
                        embedding.0.len() as i64
                    }
                    t => {
                        let remote_embeddings =
                            build_remote_embeddings(t, &model.name, &model.parameters)?;
                        remote_embeddings.get_embedding_size().await?
                    }
                };
                pool.execute(
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
                    .bind(model.get_id().await?)
                    .bind(splitter.get_id().await?)
                    .execute(&pool).await?;
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
        &mut self,
        model: &mut Model,
        splitter: &mut Splitter,
    ) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.verify_in_database(&pool).await?;
        let (count,): (i64,) = sqlx::query_as(&query_builder!(
            "SELECT count(*) FROM (SELECT 1 FROM %s WHERE splitter_id = $1 LIMIT 1) AS t",
            self.chunks_table_name
        ))
        .bind(splitter.id)
        .fetch_one(&pool)
        .await?;

        if count == 0 {
            anyhow::bail!("No chunks in the chunks table with the associated splitter_id. Make sure to generate chunks with the correct splitter_id before generating embeddings")
        }

        let embeddings_table_name = self.create_or_get_embeddings_table(model, splitter).await?;

        match model.source.as_str() {
            "pgml" => {
                sqlx::query(&query_builder!(
                    queries::GENERATE_EMBEDDINGS,
                    embeddings_table_name,
                    self.chunks_table_name,
                    embeddings_table_name
                ))
                .bind(&model.name)
                .bind(&model.parameters)
                .bind(splitter.id)
                .execute(&pool)
                .await?;
            }
            t => {
                let remote_embeddings = build_remote_embeddings(t, &model.name, &model.parameters)?;
                remote_embeddings
                    .generate_embeddings(
                        &embeddings_table_name,
                        &self.chunks_table_name,
                        splitter.get_id().await?,
                        &pool,
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
        model: &Model,
        splitter: &Splitter,
        query_parameters: Option<Json>,
        top_k: Option<i64>,
    ) -> anyhow::Result<Vec<(f64, String, Json)>> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let query_parameters = match query_parameters {
            Some(params) => params,
            None => Json(serde_json::json!({})),
        };
        let top_k = top_k.unwrap_or(5);
        let embeddings_table_name = self.get_embeddings_table_name(model, splitter)?;

        Ok(match model.source.as_str() {
            "pgml" => {
                sqlx::query_as(&query_builder!(
                    queries::EMBED_AND_VECTOR_SEARCH,
                    embeddings_table_name,
                    embeddings_table_name,
                    self.chunks_table_name,
                    self.documents_table_name
                ))
                .bind(&model.name)
                .bind(query)
                .bind(query_parameters)
                .bind(top_k)
                .fetch_all(&pool)
                .await?
            }
            t => {
                let remote_embeddings = build_remote_embeddings(t, &model.name, &query_parameters)?;
                let mut embeddings = remote_embeddings.embed(vec![query.to_string()]).await?;
                let embedding = std::mem::take(&mut embeddings[0]);
                sqlx::query_as(&query_builder!(
                    queries::VECTOR_SEARCH,
                    embeddings_table_name,
                    embeddings_table_name,
                    self.chunks_table_name,
                    self.documents_table_name
                ))
                .bind(embedding)
                .bind(top_k)
                .fetch_all(&pool)
                .await?
            }
        })
    }

    pub async fn archive(&mut self) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Error getting system time")
            .as_secs();
        let archive_table_name = format!("{}_archive_{}", &self.name, timestamp);
        sqlx::query("UPDATE pgml.collections SET name = $1, active = FALSE where name = $2")
            .bind(&archive_table_name)
            .bind(&self.name)
            .execute(&pool)
            .await?;
        sqlx::query(&query_builder!(
            "ALTER SCHEMA %s RENAME TO %s",
            &self.name,
            archive_table_name
        ))
        .execute(&pool)
        .await?;
        Ok(())
    }

    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(self.clone())
    }

    pub fn get_verified_in_database(&self) -> bool {
        self.verified_in_database
    }

    // We will probably want to add a task parameter to this function
    pub fn get_embeddings_table_name(
        &self,
        model: &Model,
        splitter: &Splitter,
    ) -> anyhow::Result<String> {
        // There are other ways to acomplish the same thing, but I like hashing it and moving it
        // into a UUID
        let model_splitter_hash = md5::compute(
            format!(
                "{}_{}_{}_{}_{}_{}",
                model.name,
                model.task,
                *model.parameters,
                model.source,
                splitter.name,
                *splitter.parameters
            )
            .as_bytes(),
        );
        Ok(format!(
            "{}.embeddings_{}",
            self.name,
            &uuid::Uuid::from_slice(&model_splitter_hash.0)?
        ))
    }

    fn generate_table_names(name: &str) -> (String, String, String, String) {
        [
            ".documents",
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
