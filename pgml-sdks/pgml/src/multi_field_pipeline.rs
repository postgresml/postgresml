use anyhow::Context;
use indicatif::MultiProgress;
use rust_bridge::{alias, alias_manual, alias_methods};
use serde::Deserialize;
use sqlx::{Executor, PgConnection, PgPool, Postgres, Transaction};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::{collections::HashMap, sync::atomic::AtomicBool};
use tokio::join;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::remote_embeddings::PoolOrArcMutextTransaction;
use crate::{
    collection::ProjectInfo,
    get_or_initialize_pool,
    model::{Model, ModelRuntime},
    models, queries, query_builder,
    remote_embeddings::build_remote_embeddings,
    splitter::Splitter,
    types::{DateTime, Json, TryToNumeric},
    utils,
};

#[cfg(feature = "python")]
use crate::{model::ModelPython, splitter::SplitterPython, types::JsonPython};

type ParsedSchema = HashMap<String, FieldAction>;

#[derive(Deserialize)]
struct ValidSplitterAction {
    model: Option<String>,
    parameters: Option<Json>,
}

#[derive(Deserialize)]
struct ValidEmbedAction {
    model: String,
    source: Option<String>,
    parameters: Option<Json>,
    hnsw: Option<Json>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FullTextSearchAction {
    configuration: String,
}

#[derive(Deserialize)]
struct ValidFieldAction {
    splitter: Option<ValidSplitterAction>,
    embed: Option<ValidEmbedAction>,
    full_text_search: Option<FullTextSearchAction>,
}

#[derive(Debug, Clone)]
pub struct HNSW {
    m: u64,
    ef_construction: u64,
}

impl Default for HNSW {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 64,
        }
    }
}

impl TryFrom<Json> for HNSW {
    type Error = anyhow::Error;
    fn try_from(value: Json) -> anyhow::Result<Self> {
        let m = if !value["hnsw"]["m"].is_null() {
            value["hnsw"]["m"]
                .try_to_u64()
                .context("hnsw.m must be an integer")?
        } else {
            16
        };
        let ef_construction = if !value["hnsw"]["ef_construction"].is_null() {
            value["hnsw"]["ef_construction"]
                .try_to_u64()
                .context("hnsw.ef_construction must be an integer")?
        } else {
            64
        };
        Ok(Self { m, ef_construction })
    }
}

#[derive(Debug, Clone)]
pub struct SplitterAction {
    pub model: Splitter,
}

#[derive(Debug, Clone)]
pub struct EmbedAction {
    pub model: Model,
    pub hnsw: HNSW,
}

#[derive(Debug, Clone)]
pub struct FieldAction {
    pub splitter: Option<SplitterAction>,
    pub embed: Option<EmbedAction>,
    pub full_text_search: Option<FullTextSearchAction>,
}

impl TryFrom<ValidFieldAction> for FieldAction {
    type Error = anyhow::Error;
    fn try_from(value: ValidFieldAction) -> Result<Self, Self::Error> {
        let embed = value
            .embed
            .map(|v| {
                let model = Model::new(Some(v.model), v.source, v.parameters);
                let hnsw = v
                    .hnsw
                    .map(|v2| HNSW::try_from(v2))
                    .unwrap_or_else(|| Ok(HNSW::default()))?;
                anyhow::Ok(EmbedAction { model, hnsw })
            })
            .transpose()?;
        let splitter = value
            .splitter
            .map(|v| {
                let splitter = Splitter::new(v.model, v.parameters);
                anyhow::Ok(SplitterAction { model: splitter })
            })
            .transpose()?;
        Ok(Self {
            splitter,
            embed,
            full_text_search: value.full_text_search,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MultiFieldPipelineDatabaseData {
    pub id: i64,
    pub created_at: DateTime,
}

#[derive(Debug)]
pub struct MultiFieldPipeline {
    // TODO: Make the schema and parsed_schema optional fields only required if they try to save a new pipeline that does not exist
    pub name: String,
    pub schema: Option<Json>,
    pub parsed_schema: Option<ParsedSchema>,
    project_info: Option<ProjectInfo>,
    database_data: Option<MultiFieldPipelineDatabaseData>,
}

fn json_to_schema(schema: &Json) -> anyhow::Result<ParsedSchema> {
    schema
        .as_object()
        .context("Schema object must be a JSON object")?
        .iter()
        .try_fold(ParsedSchema::new(), |mut acc, (key, value)| {
            if acc.contains_key(key) {
                Err(anyhow::anyhow!("Schema contains duplicate keys"))
            } else {
                // First lets deserialize it normally
                let action: ValidFieldAction = serde_json::from_value(value.to_owned())?;
                // Now lets actually build the models and splitters
                acc.insert(key.to_owned(), action.try_into()?);
                Ok(acc)
            }
        })
}

impl MultiFieldPipeline {
    pub fn new(name: &str, schema: Option<Json>) -> anyhow::Result<Self> {
        let parsed_schema = schema.as_ref().map(|s| json_to_schema(s)).transpose()?;
        Ok(Self {
            name: name.to_string(),
            schema,
            parsed_schema,
            project_info: None,
            database_data: None,
        })
    }

    #[instrument(skip(self))]
    pub(crate) async fn verify_in_database(&mut self, throw_if_exists: bool) -> anyhow::Result<()> {
        if self.database_data.is_none() {
            let pool = self.get_pool().await?;

            let project_info = self
                .project_info
                .as_ref()
                .context("Cannot verify pipeline wihtout project info")?;

            let pipeline: Option<models::MultiFieldPipeline> = sqlx::query_as(&query_builder!(
                "SELECT * FROM %s WHERE name = $1",
                format!("{}.pipelines", project_info.name)
            ))
            .bind(&self.name)
            .fetch_optional(&pool)
            .await?;

            let pipeline = if let Some(pipeline) = pipeline {
                if throw_if_exists {
                    anyhow::bail!("Pipeline {} already exists. You do not need to add this pipeline to the collection as it has already been added.", pipeline.name);
                }

                let mut parsed_schema = json_to_schema(&pipeline.schema)?;

                for (_key, value) in parsed_schema.iter_mut() {
                    if let Some(splitter) = &mut value.splitter {
                        splitter.model.set_project_info(project_info.clone());
                        splitter.model.verify_in_database(false).await?;
                    }
                    if let Some(embed) = &mut value.embed {
                        embed.model.set_project_info(project_info.clone());
                        embed.model.verify_in_database(false).await?;
                    }
                }
                self.schema = Some(pipeline.schema.clone());
                self.parsed_schema = Some(parsed_schema.clone());

                pipeline
            } else {
                let schema = self
                    .schema
                    .as_ref()
                    .context("Pipeline must have schema to store in database")?;
                let mut parsed_schema = json_to_schema(schema)?;

                for (_key, value) in parsed_schema.iter_mut() {
                    if let Some(splitter) = &mut value.splitter {
                        splitter.model.set_project_info(project_info.clone());
                        splitter.model.verify_in_database(false).await?;
                    }
                    if let Some(embed) = &mut value.embed {
                        embed.model.set_project_info(project_info.clone());
                        embed.model.verify_in_database(false).await?;
                    }
                }
                self.parsed_schema = Some(parsed_schema);

                // Here we actually insert the pipeline into the collection.pipelines table
                // and create the collection_pipeline schema and required tables
                let mut transaction = pool.begin().await?;
                let pipeline = sqlx::query_as(&query_builder!(
                    "INSERT INTO %s (name, schema) VALUES ($1, $2) RETURNING *",
                    format!("{}.pipelines", project_info.name)
                ))
                .bind(&self.name)
                .bind(&self.schema)
                .fetch_one(&mut *transaction)
                .await?;
                self.create_tables(&mut transaction).await?;
                transaction.commit().await?;

                pipeline
            };
            self.database_data = Some(MultiFieldPipelineDatabaseData {
                id: pipeline.id,
                created_at: pipeline.created_at,
            })
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn create_tables(
        &mut self,
        transaction: &mut Transaction<'static, Postgres>,
    ) -> anyhow::Result<()> {
        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to create_or_get_tables")?;
        let collection_name = &project_info.name;
        let documents_table_name = format!("{}.documents", collection_name);

        let schema = format!("{}_{}", collection_name, self.name);

        transaction
            .execute(query_builder!("CREATE SCHEMA IF NOT EXISTS %s", schema).as_str())
            .await?;

        let parsed_schema = self
            .parsed_schema
            .as_ref()
            .context("Pipeline must have schema to create_tables")?;

        for (key, value) in parsed_schema.iter() {
            let chunks_table_name = format!("{}.{}_chunks", schema, key);
            transaction
                .execute(
                    query_builder!(
                        queries::CREATE_CHUNKS_TABLE,
                        chunks_table_name,
                        documents_table_name
                    )
                    .as_str(),
                )
                .await?;
            let index_name = format!("{}_pipeline_chunk_document_id_index", key);
            transaction
                .execute(
                    query_builder!(
                        queries::CREATE_INDEX,
                        "",
                        index_name,
                        chunks_table_name,
                        "document_id"
                    )
                    .as_str(),
                )
                .await?;

            if let Some(embed) = &value.embed {
                let embeddings_table_name = format!("{}.{}_embeddings", schema, key);
                let embedding_length = match &embed.model.runtime {
                    ModelRuntime::Python => {
                        let embedding: (Vec<f32>,) = sqlx::query_as(
                                    "SELECT embedding from pgml.embed(transformer => $1, text => 'Hello, World!', kwargs => $2) as embedding")
                                    .bind(&embed.model.name)
                                    .bind(&embed.model.parameters)
                                    .fetch_one(&mut *transaction).await?;
                        embedding.0.len() as i64
                    }
                    t => {
                        let remote_embeddings = build_remote_embeddings(
                            t.to_owned(),
                            &embed.model.name,
                            Some(&embed.model.parameters),
                        )?;
                        remote_embeddings.get_embedding_size().await?
                    }
                };

                // Create the embeddings table
                sqlx::query(&query_builder!(
                    queries::CREATE_EMBEDDINGS_TABLE,
                    &embeddings_table_name,
                    chunks_table_name,
                    documents_table_name,
                    embedding_length
                ))
                .execute(&mut *transaction)
                .await?;
                let index_name = format!("{}_pipeline_embedding_chunk_id_index", key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX,
                            "",
                            index_name,
                            &embeddings_table_name,
                            "chunk_id"
                        )
                        .as_str(),
                    )
                    .await?;
                let index_name = format!("{}_pipeline_embedding_document_id_index", key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX,
                            "",
                            index_name,
                            &embeddings_table_name,
                            "document_id"
                        )
                        .as_str(),
                    )
                    .await?;
                let index_with_parameters = format!(
                    "WITH (m = {}, ef_construction = {})",
                    embed.hnsw.m, embed.hnsw.ef_construction
                );
                let index_name = format!("{}_pipeline_embedding_hnsw_vector_index", key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX_USING_HNSW,
                            "",
                            index_name,
                            &embeddings_table_name,
                            "embedding vector_cosine_ops",
                            index_with_parameters
                        )
                        .as_str(),
                    )
                    .await?;
            }

            // Create the tsvectors table
            if value.full_text_search.is_some() {
                let tsvectors_table_name = format!("{}.{}_tsvectors", schema, key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_CHUNKS_TSVECTORS_TABLE,
                            tsvectors_table_name,
                            chunks_table_name,
                            documents_table_name
                        )
                        .as_str(),
                    )
                    .await?;
                let index_name = format!("{}_pipeline_tsvector_chunk_id_index", key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX,
                            "",
                            index_name,
                            tsvectors_table_name,
                            "chunk_id"
                        )
                        .as_str(),
                    )
                    .await?;
                let index_name = format!("{}_pipeline_tsvector_document_id_index", key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX,
                            "",
                            index_name,
                            tsvectors_table_name,
                            "document_id"
                        )
                        .as_str(),
                    )
                    .await?;
                let index_name = format!("{}_pipeline_tsvector_index", key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_INDEX_USING_GIN,
                            "",
                            index_name,
                            tsvectors_table_name,
                            "ts"
                        )
                        .as_str(),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) async fn sync_document(
        &mut self,
        document_id: i64,
        transaction: Arc<Mutex<Transaction<'static, Postgres>>>,
    ) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;

        // We are assuming we have manually verified the pipeline before doing this
        let parsed_schema = self
            .parsed_schema
            .as_ref()
            .context("Pipeline must have schema to execute")?;

        for (key, value) in parsed_schema.iter() {
            let chunk_ids = self
                .sync_chunks_for_document(
                    key,
                    value.splitter.as_ref().map(|v| &v.model),
                    document_id,
                    transaction.clone(),
                )
                .await?;
            if !chunk_ids.is_empty() {
                if let Some(embed) = &value.embed {
                    self.sync_embeddings_for_chunks(
                        key,
                        &embed.model,
                        &chunk_ids,
                        transaction.clone(),
                    )
                    .await?;
                }
                if let Some(full_text_search) = &value.full_text_search {
                    self.sync_tsvectors_for_chunks(
                        key,
                        &full_text_search.configuration,
                        &chunk_ids,
                        transaction.clone(),
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn sync_chunks_for_document(
        &self,
        key: &str,
        splitter: Option<&Splitter>,
        document_id: i64,
        transaction: Arc<Mutex<Transaction<'static, Postgres>>>,
    ) -> anyhow::Result<Vec<i64>> {
        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync chunks")?;

        let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
        let documents_table_name = format!("{}.documents", project_info.name);
        let json_key_query = format!("document->>'{}'", key);

        if let Some(splitter) = splitter {
            let splitter_database_data = splitter
                .database_data
                .as_ref()
                .context("Splitter must be verified to sync chunks")?;

            sqlx::query(&query_builder!(
                queries::GENERATE_CHUNKS_FOR_DOCUMENT_ID,
                &chunks_table_name,
                &json_key_query,
                documents_table_name
            ))
            .bind(splitter_database_data.id)
            .bind(document_id)
            .execute(&mut *transaction.lock().await)
            .await?;

            sqlx::query_scalar(&query_builder!(
                "SELECT id FROM %s WHERE document_id = $1",
                &chunks_table_name
            ))
            .bind(document_id)
            .fetch_all(&mut *transaction.lock().await)
            .await
            .map_err(anyhow::Error::msg)
        } else {
            sqlx::query_scalar(&query_builder!(
                r#"
                        INSERT INTO %s(
                            document_id, chunk_index, chunk
                        )
                        SELECT 
                            id,
                            1,
                            %d
                        FROM %s
                        WHERE id = $1
                        ON CONFLICT (document_id, chunk_index) DO UPDATE SET chunk = EXCLUDED.chunk 
                        RETURNING id
                    "#,
                &chunks_table_name,
                &json_key_query,
                &documents_table_name
            ))
            .bind(document_id)
            .fetch_all(&mut *transaction.lock().await)
            .await
            .map_err(anyhow::Error::msg)
        }
    }

    #[instrument(skip(self))]
    async fn sync_embeddings_for_chunks(
        &self,
        key: &str,
        model: &Model,
        chunk_ids: &Vec<i64>,
        transaction: Arc<Mutex<Transaction<'static, Postgres>>>,
    ) -> anyhow::Result<()> {
        // Remove the stored name from the parameters
        let mut parameters = model.parameters.clone();
        parameters
            .as_object_mut()
            .context("Model parameters must be an object")?
            .remove("name");

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync chunks")?;

        let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
        let embeddings_table_name =
            format!("{}_{}.{}_embeddings", project_info.name, self.name, key);

        match model.runtime {
            ModelRuntime::Python => {
                sqlx::query(&query_builder!(
                    queries::GENERATE_EMBEDDINGS_FOR_CHUNK_IDS,
                    embeddings_table_name,
                    chunks_table_name
                ))
                .bind(&model.name)
                .bind(&parameters)
                .bind(chunk_ids)
                .execute(&mut *transaction.lock().await)
                .await?;
            }
            r => {
                let remote_embeddings = build_remote_embeddings(r, &model.name, Some(&parameters))?;
                remote_embeddings
                    .generate_embeddings(
                        &embeddings_table_name,
                        &chunks_table_name,
                        Some(chunk_ids),
                        PoolOrArcMutextTransaction::ArcMutextTransaction(transaction),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn sync_tsvectors_for_chunks(
        &self,
        key: &str,
        configuration: &str,
        chunk_ids: &Vec<i64>,
        transaction: Arc<Mutex<Transaction<'static, Postgres>>>,
    ) -> anyhow::Result<()> {
        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync TSVectors")?;

        let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
        let tsvectors_table_name = format!("{}_{}.{}_tsvectors", project_info.name, self.name, key);

        sqlx::query(&query_builder!(
            queries::GENERATE_TSVECTORS_FOR_CHUNK_IDS,
            tsvectors_table_name,
            configuration,
            chunks_table_name
        ))
        .bind(chunk_ids)
        .execute(&mut *transaction.lock().await)
        .await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn resync(&mut self) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;

        // We are assuming we have manually verified the pipeline before doing this
        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync chunks")?;
        let parsed_schema = self
            .parsed_schema
            .as_ref()
            .context("Pipeline must have schema to execute")?;

        // Before doing any syncing, delete all old and potentially outdated documents
        let pool = self.get_pool().await?;
        for (key, _value) in parsed_schema.iter() {
            let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
            pool.execute(query_builder!("DELETE FROM %s CASCADE", chunks_table_name).as_str())
                .await?;
        }

        for (key, value) in parsed_schema.iter() {
            self.resync_chunks(key, value.splitter.as_ref().map(|v| &v.model))
                .await?;
            if let Some(embed) = &value.embed {
                self.resync_embeddings(key, &embed.model).await?;
            }
            if let Some(full_text_search) = &value.full_text_search {
                self.resync_tsvectors(key, &full_text_search.configuration)
                    .await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn resync_chunks(&self, key: &str, splitter: Option<&Splitter>) -> anyhow::Result<()> {
        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync chunks")?;

        let pool = self.get_pool().await?;

        let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
        let documents_table_name = format!("{}.documents", project_info.name);
        let json_key_query = format!("document->>'{}'", key);

        if let Some(splitter) = splitter {
            let splitter_database_data = splitter
                .database_data
                .as_ref()
                .context("Splitter must be verified to sync chunks")?;

            sqlx::query(&query_builder!(
                queries::GENERATE_CHUNKS,
                &chunks_table_name,
                &json_key_query,
                documents_table_name,
                &chunks_table_name
            ))
            .bind(splitter_database_data.id)
            .execute(&pool)
            .await?;
        } else {
            sqlx::query(&query_builder!(
                r#"
                    INSERT INTO %s(
                        document_id, chunk_index, chunk
                    )
                    SELECT
                        id,
                        1,
                        %d
                    FROM %s
                    WHERE id NOT IN (SELECT document_id FROM %s)
                    ON CONFLICT (document_id, chunk_index) DO UPDATE SET chunk = EXCLUDED.chunk
                    RETURNING id
                "#,
                &chunks_table_name,
                &json_key_query,
                &documents_table_name,
                &chunks_table_name
            ))
            .execute(&pool)
            .await?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn resync_embeddings(&self, key: &str, model: &Model) -> anyhow::Result<()> {
        let pool = self.get_pool().await?;

        // Remove the stored name from the parameters
        let mut parameters = model.parameters.clone();
        parameters
            .as_object_mut()
            .context("Model parameters must be an object")?
            .remove("name");

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync chunks")?;

        let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
        let embeddings_table_name =
            format!("{}_{}.{}_embeddings", project_info.name, self.name, key);

        match model.runtime {
            ModelRuntime::Python => {
                sqlx::query(&query_builder!(
                    queries::GENERATE_EMBEDDINGS,
                    embeddings_table_name,
                    chunks_table_name,
                    embeddings_table_name
                ))
                .bind(&model.name)
                .bind(&parameters)
                .execute(&pool)
                .await?;
            }
            r => {
                let remote_embeddings = build_remote_embeddings(r, &model.name, Some(&parameters))?;
                remote_embeddings
                    .generate_embeddings(
                        &embeddings_table_name,
                        &chunks_table_name,
                        None,
                        PoolOrArcMutextTransaction::Pool(pool),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn resync_tsvectors(&self, key: &str, configuration: &str) -> anyhow::Result<()> {
        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync TSVectors")?;

        let pool = self.get_pool().await?;

        let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
        let tsvectors_table_name = format!("{}_{}.{}_tsvectors", project_info.name, self.name, key);

        sqlx::query(&query_builder!(
            queries::GENERATE_TSVECTORS,
            tsvectors_table_name,
            configuration,
            chunks_table_name,
            tsvectors_table_name
        ))
        .execute(&pool)
        .await?;
        Ok(())
    }

    async fn get_pool(&self) -> anyhow::Result<PgPool> {
        let database_url = &self
            .project_info
            .as_ref()
            .context("Project info required to call method pipeline.get_pool()")?
            .database_url;
        get_or_initialize_pool(database_url).await
    }

    #[instrument(skip(self))]
    pub(crate) fn set_project_info(&mut self, project_info: ProjectInfo) {
        if let Some(parsed_schema) = &mut self.parsed_schema {
            for (_key, value) in parsed_schema.iter_mut() {
                if let Some(splitter) = &mut value.splitter {
                    splitter.model.set_project_info(project_info.clone());
                }
                if let Some(embed) = &mut value.embed {
                    embed.model.set_project_info(project_info.clone());
                }
            }
        }
        self.project_info = Some(project_info);
    }

    #[instrument]
    pub(crate) async fn create_multi_field_pipelines_table(
        project_info: &ProjectInfo,
        conn: &mut PgConnection,
    ) -> anyhow::Result<()> {
        let pipelines_table_name = format!("{}.pipelines", project_info.name);
        sqlx::query(&query_builder!(
            queries::CREATE_MULTI_FIELD_PIPELINES_TABLE,
            pipelines_table_name
        ))
        .execute(&mut *conn)
        .await?;
        conn.execute(
            query_builder!(
                queries::CREATE_INDEX,
                "",
                "pipeline_name_index",
                pipelines_table_name,
                "name"
            )
            .as_str(),
        )
        .await?;
        Ok(())
    }
}

impl TryFrom<models::Pipeline> for MultiFieldPipeline {
    type Error = anyhow::Error;
    fn try_from(value: models::Pipeline) -> anyhow::Result<Self> {
        let parsed_schema = json_to_schema(&value.schema).unwrap();
        // NOTE: We do not set the database data here even though we have it
        // self.verify_in_database() also verifies all models in the schema so we don't want to set it here
        Ok(Self {
            name: value.name,
            schema: Some(value.schema),
            parsed_schema: Some(parsed_schema),
            project_info: None,
            database_data: None,
        })
    }
}
