use anyhow::Context;
use indicatif::MultiProgress;
use rust_bridge::{alias, alias_manual, alias_methods};
use serde::Deserialize;
use sqlx::{Executor, PgConnection, PgPool};
use std::sync::atomic::Ordering::Relaxed;
use std::{collections::HashMap, sync::atomic::AtomicBool};
use tokio::join;
use tracing::instrument;

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
struct ValidEmbedAction {
    model: String,
    source: Option<String>,
    model_parameters: Option<Json>,
    splitter: Option<String>,
    splitter_parameters: Option<Json>,
    hnsw: Option<Json>,
}

#[derive(Deserialize, Debug)]
struct FullTextSearchAction {
    configuration: String,
}

#[derive(Deserialize)]
struct ValidFieldAction {
    embed: Option<ValidEmbedAction>,
    full_text_search: Option<FullTextSearchAction>,
}

#[derive(Debug)]
struct HNSW {
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

#[derive(Debug)]
struct EmbedAction {
    splitter: Option<Splitter>,
    model: Model,
    hnsw: HNSW,
}

#[derive(Debug)]
struct FieldAction {
    embed: Option<EmbedAction>,
    full_text_search: Option<FullTextSearchAction>,
}

impl TryFrom<ValidFieldAction> for FieldAction {
    type Error = anyhow::Error;
    fn try_from(value: ValidFieldAction) -> Result<Self, Self::Error> {
        let embed = value
            .embed
            .map(|v| {
                let model = Model::new(Some(v.model), v.source, v.model_parameters);
                let splitter = v
                    .splitter
                    .map(|v2| Splitter::new(Some(v2), v.splitter_parameters));
                let hnsw = v
                    .hnsw
                    .map(|v2| HNSW::try_from(v2))
                    .unwrap_or_else(|| Ok(HNSW::default()))?;
                anyhow::Ok(EmbedAction {
                    model,
                    splitter,
                    hnsw,
                })
            })
            .transpose()?;
        Ok(Self {
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
    schema: Json,
    parsed_schema: ParsedSchema,
    project_info: Option<ProjectInfo>,
    database_data: Option<MultiFieldPipelineDatabaseData>,
}

pub enum PipelineTableTypes {
    Embedding,
    TSVector,
}

fn validate_schema(schema: &Json) -> anyhow::Result<()> {
    Ok(())
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
    pub fn new(name: &str, schema: Json) -> Self {
        let parsed_schema = json_to_schema(&schema).unwrap();
        Self {
            name: name.to_string(),
            schema,
            parsed_schema,
            project_info: None,
            database_data: None,
        }
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

            for (_key, value) in self.parsed_schema.iter_mut() {
                if let Some(embed) = &mut value.embed {
                    embed.model.set_project_info(project_info.clone());
                    embed.model.verify_in_database(false).await?;
                    if let Some(splitter) = &mut embed.splitter {
                        splitter.set_project_info(project_info.clone());
                        splitter.verify_in_database(false).await?;
                    }
                }
            }

            let pipeline = if let Some(pipeline) = pipeline {
                if throw_if_exists {
                    anyhow::bail!("Pipeline {} already exists", pipeline.name);
                }
                pipeline
            } else {
                sqlx::query_as(&query_builder!(
                    "INSERT INTO %s (name, schema) VALUES ($1, $2) RETURNING *",
                    format!("{}.pipelines", project_info.name)
                ))
                .bind(&self.name)
                .bind(&self.schema)
                .fetch_one(&pool)
                .await?
            };
            self.database_data = Some(MultiFieldPipelineDatabaseData {
                id: pipeline.id,
                created_at: pipeline.created_at,
            })
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) async fn create_tables(&mut self) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        let pool = self.get_pool().await?;

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to create_or_get_tables")?;
        let collection_name = &project_info.name;
        let documents_table_name = format!("{}.documents", collection_name);

        let schema = format!("{}_{}", collection_name, self.name);

        let mut transaction = pool.begin().await?;
        transaction
            .execute(query_builder!("CREATE SCHEMA IF NOT EXISTS %s", schema).as_str())
            .await?;

        for (key, value) in self.parsed_schema.iter() {
            if let Some(embed) = &value.embed {
                let embeddings_table_name = format!("{}.{}_embeddings", schema, key);
                let exists: bool = sqlx::query_scalar(
                        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = $1 AND table_name = $2)"
                    )
                    .bind(&schema)
                    .bind(&embeddings_table_name).fetch_one(&pool).await?;

                if !exists {
                    let embedding_length = match &embed.model.runtime {
                        ModelRuntime::Python => {
                            let embedding: (Vec<f32>,) = sqlx::query_as(
                                "SELECT embedding from pgml.embed(transformer => $1, text => 'Hello, World!', kwargs => $2) as embedding")
                                .bind(&embed.model.name)
                                .bind(&embed.model.parameters)
                                .fetch_one(&pool).await?;
                            embedding.0.len() as i64
                        }
                        t => {
                            let remote_embeddings = build_remote_embeddings(
                                t.to_owned(),
                                &embed.model.name,
                                &embed.model.parameters,
                            )?;
                            remote_embeddings.get_embedding_size().await?
                        }
                    };

                    let chunks_table_name = format!("{}.{}_chunks", schema, key);

                    // Create the chunks table
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

                    // Create the embeddings table
                    sqlx::query(&query_builder!(
                        queries::CREATE_EMBEDDINGS_TABLE,
                        &embeddings_table_name,
                        chunks_table_name,
                        embedding_length
                    ))
                    .execute(&mut *transaction)
                    .await?;
                    let index_name = format!("{}_pipeline_chunk_id_index", key);
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
                    let index_with_parameters = format!(
                        "WITH (m = {}, ef_construction = {})",
                        embed.hnsw.m, embed.hnsw.ef_construction
                    );
                    let index_name = format!("{}_pipeline_hnsw_vector_index", key);
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
            }

            // Create the tsvectors table
            if value.full_text_search.is_some() {
                let tsvectors_table_name = format!("{}.{}_tsvectors", schema, key);
                transaction
                    .execute(
                        query_builder!(
                            queries::CREATE_DOCUMENTS_TSVECTORS_TABLE,
                            tsvectors_table_name,
                            documents_table_name
                        )
                        .as_str(),
                    )
                    .await?;
                let index_name = format!("{}_tsvector_index", key);
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
        transaction.commit().await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) async fn execute(
        &mut self,
        document_ids: &Option<Vec<i64>>,
        mp: MultiProgress,
    ) -> anyhow::Result<()> {
        self.verify_in_database(false).await?;
        self.create_tables().await?;
        for (key, value) in self.parsed_schema.iter() {
            if let Some(embed) = &value.embed {
                let chunk_ids = self
                    .sync_chunks(key, &embed.splitter, document_ids, &mp)
                    .await?;
                self.sync_embeddings(key, &embed.model, &chunk_ids, &mp)
                    .await?;
            }
            if let Some(full_text_search) = &value.full_text_search {
                self.sync_tsvectors(key, &full_text_search.configuration, document_ids, &mp)
                    .await?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn sync_chunks(
        &self,
        key: &str,
        splitter: &Option<Splitter>,
        document_ids: &Option<Vec<i64>>,
        mp: &MultiProgress,
    ) -> anyhow::Result<Vec<i64>> {
        let pool = self.get_pool().await?;

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

            let progress_bar = mp
                .add(utils::default_progress_spinner(1))
                .with_prefix(format!("{} - {}", self.name.clone(), key))
                .with_message("Generating chunks");

            let is_done = AtomicBool::new(false);
            let work = async {
                let chunk_ids: Result<Vec<i64>, _> = if document_ids.is_some() {
                    sqlx::query(&query_builder!(
                        queries::GENERATE_CHUNKS_FOR_DOCUMENT_IDS,
                        &chunks_table_name,
                        &json_key_query,
                        documents_table_name,
                        &chunks_table_name
                    ))
                    .bind(splitter_database_data.id)
                    .bind(document_ids)
                    .execute(&pool)
                    .await
                    .map_err(|e| {
                        is_done.store(true, Relaxed);
                        e
                    })?;
                    sqlx::query_scalar(&query_builder!(
                        "SELECT id FROM %s WHERE document_id = ANY($1)",
                        &chunks_table_name
                    ))
                    .bind(document_ids)
                    .fetch_all(&pool)
                    .await
                } else {
                    sqlx::query_scalar(&query_builder!(
                        queries::GENERATE_CHUNKS,
                        &chunks_table_name,
                        &json_key_query,
                        documents_table_name,
                        &chunks_table_name
                    ))
                    .bind(splitter_database_data.id)
                    .fetch_all(&pool)
                    .await
                };
                is_done.store(true, Relaxed);
                chunk_ids
            };
            let progress_work = async {
                while !is_done.load(Relaxed) {
                    progress_bar.inc(1);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            };
            let (chunk_ids, _) = join!(work, progress_work);
            progress_bar.set_message("Done generating chunks");
            progress_bar.finish();
            chunk_ids.map_err(anyhow::Error::msg)
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
                    ON CONFLICT (document_id, chunk_index) DO NOTHING 
                    RETURNING id
                "#,
                &chunks_table_name,
                &json_key_query,
                &documents_table_name
            ))
            .fetch_all(&pool)
            .await
            .map_err(anyhow::Error::msg)
        }
    }

    #[instrument(skip(self))]
    async fn sync_embeddings(
        &self,
        key: &str,
        model: &Model,
        chunk_ids: &Vec<i64>,
        mp: &MultiProgress,
    ) -> anyhow::Result<()> {
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

        let progress_bar = mp
            .add(utils::default_progress_spinner(1))
            .with_prefix(self.name.clone())
            .with_message("Generating emmbeddings");

        let chunks_table_name = format!("{}_{}.{}_chunks", project_info.name, self.name, key);
        let embeddings_table_name =
            format!("{}_{}.{}_embeddings", project_info.name, self.name, key);

        let is_done = AtomicBool::new(false);
        // We need to be careful about how we handle errors here. We do not want to return an error
        // from the async block before setting is_done to true. If we do, the progress bar will
        // will load forever. We also want to make sure to propogate any errors we have
        let work = async {
            let res = match model.runtime {
                ModelRuntime::Python => sqlx::query(&query_builder!(
                    queries::GENERATE_EMBEDDINGS_FOR_CHUNK_IDS,
                    embeddings_table_name,
                    chunks_table_name,
                    embeddings_table_name
                ))
                .bind(&model.name)
                .bind(&parameters)
                .bind(chunk_ids)
                .execute(&pool)
                .await
                .map_err(|e| anyhow::anyhow!(e))
                .map(|_t| ()),
                r => {
                    let remote_embeddings = build_remote_embeddings(r, &model.name, &parameters)?;
                    remote_embeddings
                        .generate_embeddings(
                            &embeddings_table_name,
                            &chunks_table_name,
                            chunk_ids,
                            &pool,
                        )
                        .await
                        .map(|_t| ())
                }
            };
            is_done.store(true, Relaxed);
            res
        };
        let progress_work = async {
            while !is_done.load(Relaxed) {
                progress_bar.inc(1);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        };
        let (res, _) = join!(work, progress_work);
        res?;
        progress_bar.set_message("done generating embeddings");
        progress_bar.finish();
        Ok(())
    }

    #[instrument(skip(self))]
    async fn sync_tsvectors(
        &self,
        key: &str,
        configuration: &str,
        document_ids: &Option<Vec<i64>>,
        mp: &MultiProgress,
    ) -> anyhow::Result<()> {
        let pool = self.get_pool().await?;

        let project_info = self
            .project_info
            .as_ref()
            .context("Pipeline must have project info to sync TSVectors")?;

        let progress_bar = mp
            .add(utils::default_progress_spinner(1))
            .with_prefix(self.name.clone())
            .with_message("Syncing TSVectors for full text search");

        let documents_table_name = format!("{}.documents", project_info.name);
        let tsvectors_table_name = format!("{}_{}.{}_tsvectors", project_info.name, self.name, key);
        let json_key_query = format!("document->>'{}'", key);

        let is_done = AtomicBool::new(false);
        let work = async {
            let res = if document_ids.is_some() {
                sqlx::query(&query_builder!(
                    queries::GENERATE_TSVECTORS_FOR_DOCUMENT_IDS,
                    tsvectors_table_name,
                    configuration,
                    json_key_query,
                    documents_table_name
                ))
                .bind(document_ids)
                .execute(&pool)
                .await
            } else {
                sqlx::query(&query_builder!(
                    queries::GENERATE_TSVECTORS,
                    tsvectors_table_name,
                    configuration,
                    json_key_query,
                    documents_table_name
                ))
                .execute(&pool)
                .await
            };
            is_done.store(true, Relaxed);
            res.map(|_t| ()).map_err(|e| anyhow::anyhow!(e))
        };
        let progress_work = async {
            while !is_done.load(Relaxed) {
                progress_bar.inc(1);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        };
        let (res, _) = join!(work, progress_work);
        res?;
        progress_bar.set_message("Done syncing TSVectors for full text search");
        progress_bar.finish();

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
        for (_key, value) in self.parsed_schema.iter_mut() {
            if let Some(embed) = &mut value.embed {
                embed.model.set_project_info(project_info.clone());
                if let Some(splitter) = &mut embed.splitter {
                    splitter.set_project_info(project_info.clone());
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
            schema: value.schema,
            parsed_schema,
            project_info: None,
            database_data: None,
        })
    }
}
