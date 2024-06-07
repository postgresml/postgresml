//! # pgml
//!
//! pgml is an open source alternative for building end-to-end vector search applications without OpenAI and Pinecone
//!
//! With this SDK, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using PgVector for fast and accurate queries.

use anyhow::Context;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::{collections::HashMap, time::Duration};
use tokio::runtime::{Builder, Runtime};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod builtins;
#[cfg(any(feature = "python", feature = "javascript"))]
mod cli;
mod collection;
mod filter_builder;
mod languages;
pub mod migrations;
mod model;
mod models;
mod open_source_ai;
mod order_by_builder;
mod pipeline;
mod queries;
mod query_builder;
mod query_runner;
mod rag_query_builder;
mod remote_embeddings;
mod search_query_builder;
mod single_field_pipeline;
mod splitter;
pub mod transformer_pipeline;
pub mod types;
mod utils;
mod vector_search_query_builder;

// Re-export
pub use builtins::Builtins;
pub use collection::Collection;
pub use model::Model;
pub use open_source_ai::OpenSourceAI;
pub use pipeline::Pipeline;
pub use splitter::Splitter;
pub use transformer_pipeline::TransformerPipeline;

// This is use when inserting collections to set the sdk_version used during creation
// This doesn't actually mean the verion of the SDK it was created on, it means the
// version it is compatible with
static SDK_VERSION: &str = "1.0.0";

// Store the database(s) in a global variable so that we can access them from anywhere
// This is not necessarily idiomatic Rust, but it is a good way to acomplish what we need
static DATABASE_POOLS: RwLock<Option<HashMap<String, PgPool>>> = RwLock::new(None);

// Even though this function does not use async anywhere, for whatever reason it must be async or
// sqlx's connect_lazy will throw an error
async fn get_or_initialize_pool(database_url: &Option<String>) -> anyhow::Result<PgPool> {
    let mut pools = DATABASE_POOLS.write();
    let pools = pools.get_or_insert_with(HashMap::new);
    let url = database_url.clone().unwrap_or_else(|| {
        std::env::var("PGML_DATABASE_URL").unwrap_or_else(|_|
            std::env::var("DATABASE_URL").expect("Please set PGML_DATABASE_URL environment variable or explicitly pass a database connection string to your collection"))
    });
    if let Some(pool) = pools.get(&url) {
        Ok(pool.clone())
    } else {
        let acquire_timeout = std::env::var("PGML_CHECKOUT_TIMEOUT")
            .ok()
            .map(|v| v.parse::<u64>())
            .transpose()
            .context("Error parsing PGML_CHECKOUT_TIMEOUT, expected an integer")?
            .map(anyhow::Ok)
            .unwrap_or_else(|| {
                Ok(std::env::var("PGML_POOL_ACQUIRE_TIMEOUT")
                    .ok()
                    .map(|v| v.parse::<u64>())
                    .transpose()
                    .context("Error parsing PGML_POOL_ACQUIRE_TIMEOUT, expected an integer")?
                    .unwrap_or(30000))
            })?;
        let acquire_timeout = Duration::from_millis(acquire_timeout);

        let max_lifetime = std::env::var("PGML_POOL_MAX_LIFETIME")
            .ok()
            .map(|v| {
                anyhow::Ok(Duration::from_millis(v.parse::<u64>().context(
                    "Error parsing PGML_POOL_MAX_LIFETIME, expected an integer",
                )?))
            })
            .transpose()?;

        let idle_timeout = std::env::var("PGML_POOL_IDLE_TIMEOUT")
            .ok()
            .map(|v| {
                anyhow::Ok(Duration::from_millis(v.parse::<u64>().context(
                    "Error parsing PGML_POOL_IDLE_TIMEOUT, expected an integer",
                )?))
            })
            .transpose()?;

        let max_connections = std::env::var("PGML_POOL_MAX_CONNECTIONS")
            .ok()
            .map(|v| v.parse::<u32>())
            .transpose()
            .context("Error parsing PGML_POOL_MAX_CONNECTIONS, expected an integer")?
            .unwrap_or(10);

        let min_connections = std::env::var("PGML_POOL_MIN_CONNECTIONS")
            .ok()
            .map(|v| v.parse::<u32>())
            .transpose()
            .context("Error parsing PGML_POOL_MIN_CONNECTIONS, expected an integer")?
            .unwrap_or(0);

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(acquire_timeout)
            .max_lifetime(max_lifetime)
            .idle_timeout(idle_timeout)
            .connect_lazy(&url)?;

        pools.insert(url.to_string(), pool.clone());
        Ok(pool)
    }
}

enum LogFormat {
    Json,
    Pretty,
}

impl From<&str> for LogFormat {
    fn from(s: &str) -> Self {
        match s {
            "Json" => LogFormat::Json,
            _ => LogFormat::Pretty,
        }
    }
}

#[allow(dead_code)]
fn internal_init_logger(level: Option<String>, format: Option<String>) -> anyhow::Result<()> {
    let level = level.unwrap_or_else(|| env::var("LOG_LEVEL").unwrap_or("".to_string()));
    let level = match level.as_str() {
        "TRACE" => Level::TRACE,
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        _ => Level::ERROR,
    };

    let format = format.unwrap_or_else(|| env::var("LOG_FORMAT").unwrap_or("".to_string()));

    match format.as_str().into() {
        LogFormat::Json => FmtSubscriber::builder()
            .json()
            .with_max_level(level)
            .try_init()
            .map_err(|e| anyhow::anyhow!(e)),
        _ => FmtSubscriber::builder()
            .pretty()
            .with_max_level(level)
            .try_init()
            .map_err(|e| anyhow::anyhow!(e)),
    }
}

// Normally the global async runtime is handled by tokio but because we are a library being called
// by javascript and other langauges, we occasionally need to handle it ourselves
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Error creating tokio runtime")
});

fn get_or_set_runtime<'a>() -> &'a Runtime {
    &RUNTIME
}

#[cfg(feature = "python")]
#[pyo3::prelude::pyfunction]
fn init_logger(level: Option<String>, format: Option<String>) -> pyo3::PyResult<()> {
    internal_init_logger(level, format).ok();
    Ok(())
}

#[cfg(feature = "python")]
#[pyo3::prelude::pyfunction]
fn migrate(py: pyo3::Python) -> pyo3::PyResult<&pyo3::PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        migrations::migrate().await?;
        Ok(())
    })
}

#[cfg(feature = "python")]
#[pyo3::pymodule]
fn pgml(_py: pyo3::Python, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(init_logger, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(migrate, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(cli::cli, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(
        single_field_pipeline::SingleFieldPipeline,
        m
    )?)?;
    m.add_class::<pipeline::PipelinePython>()?;
    m.add_class::<collection::CollectionPython>()?;
    m.add_class::<model::ModelPython>()?;
    m.add_class::<splitter::SplitterPython>()?;
    m.add_class::<builtins::BuiltinsPython>()?;
    m.add_class::<transformer_pipeline::TransformerPipelinePython>()?;
    m.add_class::<open_source_ai::OpenSourceAIPython>()?;
    Ok(())
}

#[cfg(feature = "javascript")]
fn init_logger(
    mut cx: neon::context::FunctionContext,
) -> neon::result::JsResult<neon::types::JsUndefined> {
    use rust_bridge::javascript::{FromJsType, IntoJsResult};
    let level = cx.argument_opt(0);
    let level = <Option<String>>::from_option_js_type(&mut cx, level)?;
    let format = cx.argument_opt(1);
    let format = <Option<String>>::from_option_js_type(&mut cx, format)?;
    internal_init_logger(level, format).ok();
    ().into_js_result(&mut cx)
}

#[cfg(feature = "javascript")]
fn migrate(
    mut cx: neon::context::FunctionContext,
) -> neon::result::JsResult<neon::types::JsPromise> {
    use neon::prelude::*;
    use rust_bridge::javascript::IntoJsResult;
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    deferred
        .try_settle_with(&channel, move |mut cx| {
            let runtime = crate::get_or_set_runtime();
            let x = runtime.block_on(migrations::migrate());
            let x = x.expect("Error running migration");
            x.into_js_result(&mut cx)
        })
        .expect("Error sending js");
    Ok(promise)
}

#[cfg(feature = "javascript")]
#[neon::main]
fn main(mut cx: neon::context::ModuleContext) -> neon::result::NeonResult<()> {
    cx.export_function("init_logger", init_logger)?;
    cx.export_function("migrate", migrate)?;
    cx.export_function(
        "newSingleFieldPipeline",
        single_field_pipeline::SingleFieldPipeline,
    )?;
    cx.export_function("cli", cli::cli)?;
    cx.export_function("newCollection", collection::CollectionJavascript::new)?;
    cx.export_function("newModel", model::ModelJavascript::new)?;
    cx.export_function("newSplitter", splitter::SplitterJavascript::new)?;
    cx.export_function("newBuiltins", builtins::BuiltinsJavascript::new)?;
    cx.export_function(
        "newTransformerPipeline",
        transformer_pipeline::TransformerPipelineJavascript::new,
    )?;
    cx.export_function("newPipeline", pipeline::PipelineJavascript::new)?;
    cx.export_function(
        "newOpenSourceAI",
        open_source_ai::OpenSourceAIJavascript::new,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Json;
    use futures::StreamExt;
    use serde_json::json;

    fn generate_dummy_documents(count: usize) -> Vec<Json> {
        let mut documents = Vec::new();
        for i in 0..count {
            let body_text = vec![format!(
                "Here is some text that we will end up splitting on! {i}"
            )]
            .into_iter()
            .cycle()
            .take(100)
            .collect::<Vec<String>>()
            .join("\n");
            let document = serde_json::json!(
            {
                "id": i,
                "title": format!("Test document: {}", i),
                "body": body_text,
                "text": "here is some test text",
                "notes": format!("Here are some notes or something for test document {}", i),
                "metadata": {
                    "uuid": i * 10,
                    "name": format!("Test Document {}", i)
                }
            });
            documents.push(document.into());
        }
        documents
    }

    ///////////////////////////////
    // Collection & Pipelines /////
    ///////////////////////////////

    #[tokio::test]
    async fn can_create_collection() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_ccc_0", None)?;
        assert!(collection.database_data.is_none());
        collection.verify_in_database(false).await?;
        assert!(collection.database_data.is_some());
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_add_remove_pipeline() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut pipeline = Pipeline::new("0", Some(json!({}).into()))?;
        let mut collection = Collection::new("test_r_c_carp_1", None)?;
        assert!(collection.database_data.is_none());
        collection.add_pipeline(&mut pipeline).await?;
        assert!(collection.database_data.is_some());
        collection.remove_pipeline(&pipeline).await?;
        let pipelines = collection.get_pipelines().await?;
        assert!(pipelines.is_empty());
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_add_remove_pipelines() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut pipeline1 = Pipeline::new("0", Some(json!({}).into()))?;
        let mut pipeline2 = Pipeline::new("1", Some(json!({}).into()))?;
        let mut collection = Collection::new("test_r_c_carps_11", None)?;
        collection.add_pipeline(&mut pipeline1).await?;
        collection.add_pipeline(&mut pipeline2).await?;
        let pipelines = collection.get_pipelines().await?;
        assert!(pipelines.len() == 2);
        collection.remove_pipeline(&pipeline1).await?;
        let pipelines = collection.get_pipelines().await?;
        assert!(pipelines.len() == 1);
        assert!(collection.get_pipeline("0").await.is_err());
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_add_pipeline_and_upsert_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_capaud_107";
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character",
                            "parameters": {
                                "chunk_size": 1000,
                                "chunk_overlap": 40
                            }
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    }
                })
                .into(),
            ),
        )?;
        let mut collection = Collection::new(collection_name, None)?;
        collection.add_pipeline(&mut pipeline).await?;
        let documents = generate_dummy_documents(2);
        collection.upsert_documents(documents.clone(), None).await?;
        let pool = get_or_initialize_pool(&None).await?;
        let documents_table = format!("{}.documents", collection_name);
        let queried_documents: Vec<models::Document> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", documents_table))
                .fetch_all(&pool)
                .await?;
        assert!(queried_documents.len() == 2);
        for (d, qd) in std::iter::zip(documents, queried_documents) {
            assert_eq!(d, qd.document);
        }
        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 2);
        let chunks_table = format!("{}_{}.body_chunks", collection_name, pipeline_name);
        let body_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(body_chunks.len() == 12);
        let tsvectors_table = format!("{}_{}.body_tsvectors", collection_name, pipeline_name);
        let tsvectors: Vec<models::TSVector> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", tsvectors_table))
                .fetch_all(&pool)
                .await?;
        assert!(tsvectors.len() == 12);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_add_pipeline_and_upsert_documents_with_parallel_batches() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_capaud_107";
        let pipeline_name = "test_r_p_capaud_6";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character",
                            "parameters": {
                                "chunk_size": 1000,
                                "chunk_overlap": 40
                            }
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    }
                })
                .into(),
            ),
        )?;
        let mut collection = Collection::new(collection_name, None)?;
        collection.add_pipeline(&mut pipeline).await?;
        let documents = generate_dummy_documents(20);
        collection
            .upsert_documents(
                documents.clone(),
                Some(
                    json!({
                        "batch_size": 2,
                        "parallel_batches": 5
                    })
                    .into(),
                ),
            )
            .await?;
        let pool = get_or_initialize_pool(&None).await?;
        let documents_table = format!("{}.documents", collection_name);
        let queried_documents: Vec<models::Document> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", documents_table))
                .fetch_all(&pool)
                .await?;
        assert!(queried_documents.len() == 20);
        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 20);
        let chunks_table = format!("{}_{}.body_chunks", collection_name, pipeline_name);
        let body_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(body_chunks.len() == 120);
        let tsvectors_table = format!("{}_{}.body_tsvectors", collection_name, pipeline_name);
        let tsvectors: Vec<models::TSVector> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", tsvectors_table))
                .fetch_all(&pool)
                .await?;
        assert!(tsvectors.len() == 120);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_upsert_documents_and_add_pipeline() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_cudaap_51";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(2);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                             }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    }
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let pool = get_or_initialize_pool(&None).await?;
        let documents_table = format!("{}.documents", collection_name);
        let queried_documents: Vec<models::Document> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", documents_table))
                .fetch_all(&pool)
                .await?;
        assert!(queried_documents.len() == 2);
        for (d, qd) in std::iter::zip(documents, queried_documents) {
            assert_eq!(d, qd.document);
        }
        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 2);
        let chunks_table = format!("{}_{}.body_chunks", collection_name, pipeline_name);
        let body_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(body_chunks.len() == 4);
        let tsvectors_table = format!("{}_{}.body_tsvectors", collection_name, pipeline_name);
        let tsvectors: Vec<models::TSVector> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", tsvectors_table))
                .fetch_all(&pool)
                .await?;
        assert!(tsvectors.len() == 4);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn disable_enable_pipeline() -> anyhow::Result<()> {
        let mut pipeline = Pipeline::new("0", Some(json!({}).into()))?;
        let mut collection = Collection::new("test_r_c_dep_1", None)?;
        collection.add_pipeline(&mut pipeline).await?;
        let queried_pipeline = &collection.get_pipelines().await?[0];
        assert_eq!(pipeline.name, queried_pipeline.name);
        collection.disable_pipeline(&pipeline).await?;
        let queried_pipelines = &collection.get_pipelines().await?;
        assert!(queried_pipelines.is_empty());
        collection.enable_pipeline(&mut pipeline).await?;
        let queried_pipeline = &collection.get_pipelines().await?[0];
        assert_eq!(pipeline.name, queried_pipeline.name);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_upsert_documents_and_enable_pipeline() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_cudaep_43";
        let mut collection = Collection::new(collection_name, None)?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    }
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        collection.disable_pipeline(&pipeline).await?;
        let documents = generate_dummy_documents(2);
        collection.upsert_documents(documents, None).await?;
        let pool = get_or_initialize_pool(&None).await?;
        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.is_empty());
        collection.enable_pipeline(&mut pipeline).await?;
        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 2);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn random_pipelines_documents_test() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_rpdt_3";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(6);
        collection
            .upsert_documents(documents[..2].to_owned(), None)
            .await?;
        let pipeline_name1 = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name1,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    }
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;

        collection
            .upsert_documents(documents[2..4].to_owned(), None)
            .await?;

        let pool = get_or_initialize_pool(&None).await?;
        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name1);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 4);
        let chunks_table = format!("{}_{}.body_chunks", collection_name, pipeline_name1);
        let body_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(body_chunks.len() == 8);
        let tsvectors_table = format!("{}_{}.body_tsvectors", collection_name, pipeline_name1);
        let tsvectors: Vec<models::TSVector> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", tsvectors_table))
                .fetch_all(&pool)
                .await?;
        assert!(tsvectors.len() == 8);

        let pipeline_name2 = "1";
        let mut pipeline = Pipeline::new(
            pipeline_name2,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    }
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;

        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name2);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 4);
        let chunks_table = format!("{}_{}.body_chunks", collection_name, pipeline_name2);
        let body_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(body_chunks.len() == 8);
        let tsvectors_table = format!("{}_{}.body_tsvectors", collection_name, pipeline_name2);
        let tsvectors: Vec<models::TSVector> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", tsvectors_table))
                .fetch_all(&pool)
                .await?;
        assert!(tsvectors.len() == 8);

        collection
            .upsert_documents(documents[4..6].to_owned(), None)
            .await?;

        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name2);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 6);
        let chunks_table = format!("{}_{}.body_chunks", collection_name, pipeline_name2);
        let body_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(body_chunks.len() == 12);
        let tsvectors_table = format!("{}_{}.body_tsvectors", collection_name, pipeline_name2);
        let tsvectors: Vec<models::TSVector> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", tsvectors_table))
                .fetch_all(&pool)
                .await?;
        assert!(tsvectors.len() == 12);

        let chunks_table = format!("{}_{}.title_chunks", collection_name, pipeline_name1);
        let title_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(title_chunks.len() == 6);
        let chunks_table = format!("{}_{}.body_chunks", collection_name, pipeline_name1);
        let body_chunks: Vec<models::Chunk> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", chunks_table))
                .fetch_all(&pool)
                .await?;
        assert!(body_chunks.len() == 12);
        let tsvectors_table = format!("{}_{}.body_tsvectors", collection_name, pipeline_name1);
        let tsvectors: Vec<models::TSVector> =
            sqlx::query_as(&query_builder!("SELECT * FROM %s", tsvectors_table))
                .fetch_all(&pool)
                .await?;
        assert!(tsvectors.len() == 12);

        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn pipeline_sync_status() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_pss_6";
        let mut collection = Collection::new(collection_name, None)?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        },
                        "splitter": {
                            "model": "recursive_character"
                        }
                    }
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let documents = generate_dummy_documents(4);
        collection
            .upsert_documents(documents[..2].to_owned(), None)
            .await?;
        let status = collection.get_pipeline_status(&mut pipeline).await?;
        assert_eq!(
            status.0,
            json!({
                "title": {
                    "chunks": {
                        "not_synced": 0,
                        "synced": 2,
                        "total": 2
                    },
                    "embeddings": {
                        "not_synced": 0,
                        "synced": 2,
                        "total": 2
                    },
                    "tsvectors": {
                        "not_synced": 0,
                        "synced": 2,
                        "total": 2
                    },
                }
            })
        );
        collection.disable_pipeline(&pipeline).await?;
        collection
            .upsert_documents(documents[2..4].to_owned(), None)
            .await?;
        let status = collection.get_pipeline_status(&mut pipeline).await?;
        assert_eq!(
            status.0,
            json!({
                "title": {
                    "chunks": {
                        "not_synced": 2,
                        "synced": 2,
                        "total": 4
                    },
                    "embeddings": {
                        "not_synced": 0,
                        "synced": 2,
                        "total": 2
                    },
                    "tsvectors": {
                        "not_synced": 0,
                        "synced": 2,
                        "total": 2
                    },
                }
            })
        );
        collection.enable_pipeline(&mut pipeline).await?;
        let status = collection.get_pipeline_status(&mut pipeline).await?;
        assert_eq!(
            status.0,
            json!({
                "title": {
                    "chunks": {
                        "not_synced": 0,
                        "synced": 4,
                        "total": 4
                    },
                    "embeddings": {
                        "not_synced": 0,
                        "synced": 4,
                        "total": 4
                    },
                    "tsvectors": {
                        "not_synced": 0,
                        "synced": 4,
                        "total": 4
                    },
                }
            })
        );
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_specify_custom_hnsw_parameters_for_pipelines() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_cschpfp_4";
        let mut collection = Collection::new(collection_name, None)?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            },
                            "hnsw": {
                                "m": 100,
                                "ef_construction": 200
                            }
                        }
                    }
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let schema = format!("{collection_name}_{pipeline_name}");
        let full_embeddings_table_name = format!("{schema}.title_embeddings");
        let embeddings_table_name = full_embeddings_table_name.split('.').collect::<Vec<_>>()[1];
        let pool = get_or_initialize_pool(&None).await?;
        let results: Vec<(String, String)> = sqlx::query_as(&query_builder!(
            "select indexname, indexdef from pg_indexes where tablename = '%d' and schemaname = '%d'",
            embeddings_table_name,
            schema
        )).fetch_all(&pool).await?;
        let names = results.iter().map(|(name, _)| name).collect::<Vec<_>>();
        let definitions = results
            .iter()
            .map(|(_, definition)| definition)
            .collect::<Vec<_>>();
        assert!(names.contains(&&"title_pipeline_embedding_hnsw_vector_index".to_string()));
        assert!(definitions.contains(&&format!("CREATE INDEX title_pipeline_embedding_hnsw_vector_index ON {full_embeddings_table_name} USING hnsw (embedding vector_cosine_ops) WITH (m='100', ef_construction='200')")));
        collection.archive().await?;
        Ok(())
    }

    ///////////////////////////////
    // Searches ///////////////////
    ///////////////////////////////

    #[tokio::test]
    async fn can_search_with_local_embeddings() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test_r_c_cswle_123";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(10);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    },
                    "notes": {
                       "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    }
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let query = json!({
            "query": {
                "full_text_search": {
                    "title": {
                        "query": "test 9",
                        "boost": 4.0
                    },
                    "body": {
                        "query": "Test",
                        "boost": 1.2
                    }
                },
                "semantic_search": {
                    "title": {
                        "query": "This is a test",
                        "parameters": {
                            "prompt": "query: ",
                        },
                        "boost": 2.0
                    },
                    "body": {
                        "query": "This is the body test",
                        "parameters": {
                            "prompt": "query: ",
                        },
                        "boost": 1.01
                    },
                    "notes": {
                        "query": "This is the notes test",
                        "parameters": {
                            "prompt": "query: ",
                        },
                        "boost": 1.01
                    }
                },
                "filter": {
                   "id": {
                        "$gt": 1
                    }
                }

            },
            "limit": 5
        });
        let results = collection
            .search(query.clone().into(), &mut pipeline)
            .await?;
        let ids: Vec<u64> = results["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["document"]["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, vec![9, 3, 4, 7, 5]);

        let pool = get_or_initialize_pool(&None).await?;

        let searches_table = format!("{}_{}.searches", collection_name, pipeline_name);
        let searches: Vec<(i64, serde_json::Value)> =
            sqlx::query_as(&query_builder!("SELECT id, query FROM %s", searches_table))
                .fetch_all(&pool)
                .await?;
        assert!(searches.len() == 1);
        assert!(searches[0].0 == results["search_id"].as_i64().unwrap());
        assert!(searches[0].1 == query);

        let search_results_table = format!("{}_{}.search_results", collection_name, pipeline_name);
        let search_results: Vec<(i64, i64, i64, serde_json::Value, i32)> =
            sqlx::query_as(&query_builder!(
                "SELECT id, search_id, document_id, scores, rank FROM %s ORDER BY rank ASC",
                search_results_table
            ))
            .fetch_all(&pool)
            .await?;
        assert!(search_results.len() == 5);
        // Document ids are 1 based in the db not 0 based like they are here
        assert_eq!(
            search_results.iter().map(|sr| sr.2).collect::<Vec<i64>>(),
            vec![10, 4, 5, 8, 6]
        );

        let event = json!({"clicked": true});
        collection
            .add_search_event(
                results["search_id"].as_i64().unwrap(),
                2,
                event.clone().into(),
                &pipeline,
            )
            .await?;
        let search_events_table = format!("{}_{}.search_events", collection_name, pipeline_name);
        let (search_result, retrieved_event): (i64, Json) = sqlx::query_as(&query_builder!(
            "SELECT search_result, event FROM %s LIMIT 1",
            search_events_table
        ))
        .fetch_one(&pool)
        .await?;
        assert_eq!(search_result, 2);
        assert_eq!(event, retrieved_event.0);

        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_search_with_remote_embeddings() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test r_c_cswre_66";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(10);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "text-embedding-ada-002",
                            "source": "openai",
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    },
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let mut pipeline = Pipeline::new(pipeline_name, None)?;
        let results = collection
            .search(
                json!({
                    "query": {
                        "full_text_search": {
                            "body": {
                                "query": "Test",
                                "boost": 1.2
                            }
                        },
                        "semantic_search": {
                            "title": {
                                "query": "This is a test",
                                "parameters": {
                                    "prompt": "query: ",
                                },
                                "boost": 2.0
                            },
                            "body": {
                                "query": "This is the body test",
                                "boost": 1.01
                            },
                        },
                        "filter": {
                           "id": {
                                "$gt": 1
                            }
                        }
                    },
                    "limit": 5
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let ids: Vec<u64> = results["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["document"]["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, vec![3, 9, 4, 7, 5]);
        collection.archive().await?;
        Ok(())
    }

    ///////////////////////////////
    // Vector Searches ////////////
    ///////////////////////////////

    #[tokio::test]
    async fn can_vector_search_with_local_embeddings() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test r_c_cvswle_13";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(10);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                    },
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let results = collection
            .vector_search(
                json!({
                    "query": {
                        "fields": {
                            "title": {
                                "query": "Test document: 2",
                                "parameters": {
                                    "prompt": "passage: "
                                },
                                "full_text_filter": "test",
                                "boost": 1.2
                            },
                            "body": {
                                "query": "Test document: 2",
                                "parameters": {
                                    "prompt": "passage: "
                                },
                                "boost": 1.0
                            },
                        },
                        "filter": {
                            "id": {
                                "$gt": 3
                            }
                        }
                    },
                    "document": {
                        "keys": [
                            "id"
                        ]
                    },
                    "limit": 5
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let ids: Vec<u64> = results
            .into_iter()
            .map(|r| r["document"]["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, vec![4, 8, 5, 6, 9]);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_vector_search_with_remote_embeddings() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test r_c_cvswre_7";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(10);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "source": "openai",
                            "model": "text-embedding-ada-002"
                        },
                    },
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let mut pipeline = Pipeline::new(pipeline_name, None)?;
        let results = collection
            .vector_search(
                json!({
                    "query": {
                        "fields": {
                            "title": {
                                "full_text_filter": "test",
                                "query": "Test document: 2",
                                "parameters": {
                                    "prompt": "passage: "
                                },
                            },
                            "body": {
                                "query": "Test document: 2"
                            },
                        },
                        "filter": {
                            "id": {
                                "$gt": 3
                            }
                        }
                    },
                    "limit": 5
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let ids: Vec<u64> = results
            .into_iter()
            .map(|r| r["document"]["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, vec![4, 8, 5, 6, 9]);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_vector_search_with_query_builder() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test r_c_cvswqb_7", None)?;
        let mut pipeline = Pipeline::new(
            "0",
            Some(
                json!({
                    "text": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    },
                })
                .into(),
            ),
        )?;
        collection
            .upsert_documents(generate_dummy_documents(10), None)
            .await?;
        collection.add_pipeline(&mut pipeline).await?;
        let results = collection
            .query()
            .vector_recall(
                "test query",
                &pipeline,
                Some(
                    json!({
                        "prompt": "query: "
                    })
                    .into(),
                ),
            )
            .limit(3)
            .filter(
                json!({
                    "metadata": {
                        "id": {
                            "$gt": 3
                        }
                    },
                    "full_text": {
                        "configuration": "english",
                        "text": "test"
                    }
                })
                .into(),
            )
            .fetch_all()
            .await?;
        let ids: Vec<u64> = results
            .into_iter()
            .map(|r| r.2["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, vec![4, 5, 6]);
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_vector_search_with_local_embeddings_and_specify_document_keys(
    ) -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test r_c_cvswleasdk_0";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(2);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                    },
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let results = collection
            .vector_search(
                json!({
                    "query": {
                        "fields": {
                            "body": {
                                "query": "Test document: 2",
                                "parameters": {
                                    "prompt": "query: "
                                },
                            },
                        },
                    },
                    "document": {
                        "keys": [
                            "id",
                            "title"
                        ]
                    },
                    "limit": 5
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(results[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("id"));
        assert!(results[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("title"));
        assert!(!results[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("body"));

        let results = collection
            .vector_search(
                json!({
                    "query": {
                        "fields": {
                            "body": {
                                "query": "Test document: 2",
                                "parameters": {
                                    "prompt": "query: "
                                },
                            },
                        },
                    },
                    "limit": 5
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(results[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("id"));
        assert!(results[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("title"));
        assert!(results[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("body"));
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_vector_search_with_local_embeddings_and_rerank() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test r_c_cvswlear_1";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(10);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "title": {
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                        "full_text_search": {
                            "configuration": "english"
                        }
                    },
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                    },
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;
        let results = collection
            .vector_search(
                json!({
                    "query": {
                        "fields": {
                            "title": {
                                "query": "Test document: 2",
                                "parameters": {
                                    "prompt": "passage: "
                                },
                                "full_text_filter": "test",
                                "boost": 1.2
                            },
                            "body": {
                                "query": "Test document: 2",
                                "parameters": {
                                    "prompt": "passage: "
                                },
                                "boost": 1.0
                            },
                        }
                    },
                    "rerank": {
                        "query": "Test document 2",
                        "model": "mixedbread-ai/mxbai-rerank-base-v1",
                        "num_documents_to_rerank": 100
                    },
                    "limit": 5
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(results[0]["rerank_score"].as_f64().is_some());
        let ids: Vec<u64> = results
            .into_iter()
            .map(|r| r["document"]["id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, vec![2, 1, 3, 8, 6]);
        collection.archive().await?;
        Ok(())
    }

    ///////////////////////////////
    // Working With Documents /////
    ///////////////////////////////

    #[tokio::test]
    async fn can_upsert_and_filter_get_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test r_c_cuafgd_1", None)?;

        let documents = vec![
            serde_json::json!({"id": 1, "random_key": 10, "text": "hello world 1"}).into(),
            serde_json::json!({"id": 2, "random_key": 11, "text": "hello world 2"}).into(),
            serde_json::json!({"id": 3, "random_key": 12, "text": "hello world 3"}).into(),
        ];
        collection.upsert_documents(documents.clone(), None).await?;
        let document = &collection.get_documents(None).await?[0];
        assert_eq!(document["document"]["text"], "hello world 1");

        let documents = vec![
            serde_json::json!({"id": 1, "text": "hello world new"}).into(),
            serde_json::json!({"id": 2, "random_key": 12}).into(),
            serde_json::json!({"id": 3, "random_key": 13}).into(),
        ];
        collection.upsert_documents(documents.clone(), None).await?;

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "random_key": {
                            "$eq": 12
                        }
                    }
                })
                .into(),
            ))
            .await?;
        assert_eq!(documents[0]["document"]["random_key"], 12);

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "random_key": {
                            "$gte": 13
                        }
                    }
                })
                .into(),
            ))
            .await?;
        assert_eq!(documents[0]["document"]["random_key"], 13);

        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_get_document_keys_get_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test r_c_cuafgd_1", None)?;

        let documents = vec![
            serde_json::json!({"id": 1, "random_key": 10, "nested": {"nested2": "test" } , "text": "hello world 1"}).into(),
            serde_json::json!({"id": 2, "random_key": 11, "text": "hello world 2"}).into(),
            serde_json::json!({"id": 3, "random_key": 12, "text": "hello world 3"}).into(),
        ];
        collection.upsert_documents(documents.clone(), None).await?;

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "keys": [
                        "id",
                        "random_key",
                        "nested,nested2"
                    ]
                })
                .into(),
            ))
            .await?;
        assert!(!documents[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("text"));
        assert!(documents[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("id"));
        assert!(documents[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("random_key"));
        assert!(documents[0]["document"]
            .as_object()
            .unwrap()
            .contains_key("nested,nested2"));
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_paginate_get_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cpgd_2", None)?;
        collection
            .upsert_documents(generate_dummy_documents(10), None)
            .await?;

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "limit": 5,
                    "offset": 0
                })
                .into(),
            ))
            .await?;
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["row_id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "limit": 2,
                    "offset": 5
                })
                .into(),
            ))
            .await?;
        let last_row_id = documents.last().unwrap()["row_id"].as_i64().unwrap();
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["row_id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![6, 7]
        );

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "limit": 2,
                    "last_row_id": last_row_id
                })
                .into(),
            ))
            .await?;
        let last_row_id = documents.last().unwrap()["row_id"].as_i64().unwrap();
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["row_id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![8, 9]
        );

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "limit": 1,
                    "last_row_id": last_row_id
                })
                .into(),
            ))
            .await?;
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["row_id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![10]
        );

        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_filter_and_paginate_get_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cfapgd_1", None)?;

        collection
            .upsert_documents(generate_dummy_documents(10), None)
            .await?;

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "id": {
                            "$gte": 2
                        }
                    },
                    "limit": 2,
                    "offset": 0
                })
                .into(),
            ))
            .await?;
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["document"]["id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![2, 3]
        );

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "id": {
                            "$lte": 5
                        }
                    },
                    "limit": 100,
                    "offset": 4
                })
                .into(),
            ))
            .await?;
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["document"]["id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![4, 5]
        );

        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_filter_and_delete_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cfadd_1", None)?;
        collection
            .upsert_documents(generate_dummy_documents(10), None)
            .await?;

        collection
            .delete_documents(
                serde_json::json!({
                    "id": {
                        "$lt": 2
                    }
                })
                .into(),
            )
            .await?;
        let documents = collection.get_documents(None).await?;
        assert_eq!(documents.len(), 8);
        assert!(documents
            .iter()
            .all(|d| d["document"]["id"].as_i64().unwrap() >= 2));

        collection
            .delete_documents(
                serde_json::json!({
                    "id": {
                        "$gte": 6
                    }
                })
                .into(),
            )
            .await?;
        let documents = collection.get_documents(None).await?;
        assert_eq!(documents.len(), 4);
        assert!(documents
            .iter()
            .all(|d| d["document"]["id"].as_i64().unwrap() < 6));

        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_order_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cod_1", None)?;
        collection
            .upsert_documents(
                vec![
                    json!({
                        "id": 1,
                        "text": "Test Document 1",
                        "number": 99,
                        "nested_number": {
                            "number": 3
                        },
                        "tie": 2,
                    })
                    .into(),
                    json!({
                        "id": 2,
                        "text": "Test Document 1",
                        "number": 98,
                        "nested_number": {
                            "number": 2
                        },
                        "tie": 2,
                    })
                    .into(),
                    json!({
                        "id": 3,
                        "text": "Test Document 1",
                        "number": 97,
                        "nested_number": {
                            "number": 1
                        },
                        "tie": 2
                    })
                    .into(),
                ],
                None,
            )
            .await?;
        let documents = collection
            .get_documents(Some(json!({"order_by": {"number": "asc"}}).into()))
            .await?;
        assert_eq!(
            documents
                .iter()
                .map(|d| d["document"]["number"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![97, 98, 99]
        );
        let documents = collection
            .get_documents(Some(
                json!({"order_by": {"nested_number": {"number": "asc"}}}).into(),
            ))
            .await?;
        assert_eq!(
            documents
                .iter()
                .map(|d| d["document"]["nested_number"]["number"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        let documents = collection
            .get_documents(Some(
                json!({"order_by": {"nested_number": {"number": "asc"}, "tie": "desc"}}).into(),
            ))
            .await?;
        assert_eq!(
            documents
                .iter()
                .map(|d| d["document"]["nested_number"]["number"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        let documents = collection
            .get_documents(Some(json!({"order_by": { "COLUMN_id": "desc"}}).into()))
            .await?;
        assert_eq!(
            documents
                .iter()
                .map(|d| d["row_id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        let documents = collection
            .get_documents(Some(json!({"order_by": { "COLUMN_id": "asc"}}).into()))
            .await?;
        assert_eq!(
            documents
                .iter()
                .map(|d| d["row_id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_update_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cud_5", None)?;
        collection
            .upsert_documents(
                vec![
                    json!({
                        "id": 1,
                        "text": "Test Document 1"
                    })
                    .into(),
                    json!({
                        "id": 2,
                        "text": "Test Document 1"
                    })
                    .into(),
                    json!({
                        "id": 3,
                        "text": "Test Document 1"
                    })
                    .into(),
                ],
                None,
            )
            .await?;
        collection
            .upsert_documents(
                vec![
                    json!({
                        "id": 1,
                        "number": 0,
                    })
                    .into(),
                    json!({
                        "id": 2,
                        "number": 1,
                    })
                    .into(),
                    json!({
                        "id": 3,
                        "number": 2,
                    })
                    .into(),
                ],
                None,
            )
            .await?;
        let documents = collection
            .get_documents(Some(json!({"order_by": {"number": "asc"}}).into()))
            .await?;
        assert_eq!(
            documents
                .iter()
                .map(|d| d["document"]["number"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        for document in documents {
            assert!(document["document"]["text"].as_str().is_none());
        }
        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_merge_metadata() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cmm_5", None)?;
        collection
            .upsert_documents(
                vec![
                    json!({
                        "id": 1,
                        "text": "Test Document 1",
                        "number": 99,
                        "second_number": 10,
                    })
                    .into(),
                    json!({
                        "id": 2,
                        "text": "Test Document 1",
                        "number": 98,
                        "second_number": 11,
                    })
                    .into(),
                    json!({
                        "id": 3,
                        "text": "Test Document 1",
                        "number": 97,
                        "second_number": 12,
                    })
                    .into(),
                ],
                None,
            )
            .await?;
        let documents = collection
            .get_documents(Some(json!({"order_by": {"number": "asc"}}).into()))
            .await?;
        assert_eq!(
            documents
                .iter()
                .map(|d| (
                    d["document"]["number"].as_i64().unwrap(),
                    d["document"]["second_number"].as_i64().unwrap()
                ))
                .collect::<Vec<_>>(),
            vec![(97, 12), (98, 11), (99, 10)]
        );

        collection
            .upsert_documents(
                vec![
                    json!({
                        "id": 1,
                        "number": 0,
                        "another_number": 1
                    })
                    .into(),
                    json!({
                        "id": 2,
                        "number": 1,
                        "another_number": 2
                    })
                    .into(),
                    json!({
                        "id": 3,
                        "number": 2,
                        "another_number": 3
                    })
                    .into(),
                ],
                Some(
                    json!({
                        "merge": true
                    })
                    .into(),
                ),
            )
            .await?;
        let documents = collection
            .get_documents(Some(json!({"order_by": {"number": "asc"}}).into()))
            .await?;

        assert_eq!(
            documents
                .iter()
                .map(|d| (
                    d["document"]["number"].as_i64().unwrap(),
                    d["document"]["another_number"].as_i64().unwrap(),
                    d["document"]["second_number"].as_i64().unwrap()
                ))
                .collect::<Vec<_>>(),
            vec![(0, 1, 10), (1, 2, 11), (2, 3, 12)]
        );
        collection.archive().await?;
        Ok(())
    }

    ///////////////////////////////
    // ER Diagram /////////////////
    ///////////////////////////////

    #[tokio::test]
    async fn generate_er_diagram() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut pipeline = Pipeline::new(
            "test_p_ged_57",
            Some(
                json!({
                        "title": {
                            "semantic_search": {
                                "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                            },
                            "full_text_search": {
                                "configuration": "english"
                            }
                        },
                        "body": {
                            "splitter": {
                                "model": "recursive_character"
                            },
                            "semantic_search": {
                                "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                            },
                            "full_text_search": {
                                "configuration": "english"
                            }
                        },
                        "notes": {
                           "semantic_search": {
                                "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                            }
                        }
                })
                .into(),
            ),
        )?;
        let mut collection = Collection::new("test_r_c_ged_2", None)?;
        collection.add_pipeline(&mut pipeline).await?;
        let diagram = collection.generate_er_diagram(&mut pipeline).await?;
        assert!(!diagram.is_empty());
        println!("{diagram}");
        collection.archive().await?;
        Ok(())
    }

    ///////////////////////////////
    // RAG ////////////////////////
    ///////////////////////////////

    #[tokio::test]
    async fn can_rag_with_local_embeddings() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test r_c_crwle_1";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(10);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                    },
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;

        // Single variable test
        let results = collection
            .rag(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "rerank": {
                                "query": "Test document 2",
                                "model": "mixedbread-ai/mxbai-rerank-base-v1",
                                "num_documents_to_rerank": 100
                            },
                            "limit": 5
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "completion": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "prompt": "Some text with {CONTEXT}",
                        "max_tokens": 10,
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(!results["rag"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .is_empty());

        // Multi-variable test
        let results = collection
            .rag(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "boost": 1.0,
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "CONTEXT2": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 3",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                }
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "completion": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "prompt": "Some text with {CONTEXT} AND {CONTEXT2}",
                        "max_tokens": 10
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(!results["rag"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .is_empty());

        // Chat test
        let results = collection
            .rag(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "chat": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a friendly and helpful chatbot"
                            },
                            {
                                "role": "user",
                                "content": "Some text with {CONTEXT}",
                            }
                        ],
                        "max_tokens": 10
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(!results["rag"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .is_empty());

        // Multi-variable chat test
        let results = collection
            .rag(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "boost": 1.0,
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "CONTEXT2": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 3",
                                        "boost": 1.0,
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                }
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "chat": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a friendly and helpful chatbot"
                            },
                            {
                                "role": "user",
                                "content": "Some text with {CONTEXT} AND {CONTEXT2}",
                            }
                        ],
                        "max_tokens": 10
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(!results["rag"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .is_empty());

        // Chat test with custom SQL query
        let results = collection
            .rag(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "boost": 1.0,
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "CUSTOM": {
                        "sql": "SELECT 'test'"
                    },
                    "chat": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a friendly and helpful chatbot"
                            },
                            {
                                "role": "user",
                                "content": "Some text with {CONTEXT} - {CUSTOM}",
                            }
                        ],
                        "max_tokens": 10
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        assert!(!results["rag"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .is_empty());

        collection.archive().await?;
        Ok(())
    }

    #[tokio::test]
    async fn can_rag_stream_with_local_embeddings() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let collection_name = "test r_c_crswle_1";
        let mut collection = Collection::new(collection_name, None)?;
        let documents = generate_dummy_documents(10);
        collection.upsert_documents(documents.clone(), None).await?;
        let pipeline_name = "0";
        let mut pipeline = Pipeline::new(
            pipeline_name,
            Some(
                json!({
                    "body": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "semantic_search": {
                            "model": "intfloat/e5-small-v2",
                            "parameters": {
                                "prompt": "passage: "
                            }
                        },
                    },
                })
                .into(),
            ),
        )?;
        collection.add_pipeline(&mut pipeline).await?;

        // Single variable test
        let mut results = collection
            .rag_stream(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 5
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "completion": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "prompt": "Some text with {CONTEXT}",
                        "max_tokens": 10,
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let mut stream = results.stream()?;
        while let Some(o) = stream.next().await {
            o?;
        }

        // Multi-variable test
        let mut results = collection
            .rag_stream(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "CONTEXT2": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "completion": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "prompt": "Some text with {CONTEXT} - {CONTEXT2}",
                        "max_tokens": 10,
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let mut stream = results.stream()?;
        while let Some(o) = stream.next().await {
            o?;
        }

        // Single variable chat test
        let mut results = collection
            .rag_stream(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 5
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "chat": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a friendly and helpful chatbot"
                            },
                            {
                                "role": "user",
                                "content": "Some text with {CONTEXT}",
                            }
                        ],
                        "max_tokens": 10
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let mut stream = results.stream()?;
        while let Some(o) = stream.next().await {
            o?;
        }

        // Multi-variable chat test
        let mut results = collection
            .rag_stream(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "CONTEXT2": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "chat": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a friendly and helpful chatbot"
                            },
                            {
                                "role": "user",
                                "content": "Some text with {CONTEXT} - {CONTEXT2}",
                            }
                        ],
                        "max_tokens": 10
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let mut stream = results.stream()?;
        while let Some(o) = stream.next().await {
            o?;
        }

        // Raw SQL test
        let mut results = collection
            .rag_stream(
                json!({
                    "CONTEXT": {
                        "vector_search": {
                            "query": {
                                "fields": {
                                    "body": {
                                        "query": "Test document: 2",
                                        "parameters": {
                                            "prompt": "query: "
                                        }
                                    },
                                },
                            },
                            "document": {
                                "keys": [
                                    "id"
                                ]
                            },
                            "limit": 2
                        },
                        "aggregate": {
                          "join": "\n"
                        }
                    },
                    "CUSTOM": {
                        "sql": "SELECT 'test'"
                    },
                    "chat": {
                        "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a friendly and helpful chatbot"
                            },
                            {
                                "role": "user",
                                "content": "Some text with {CONTEXT} - {CUSTOM}",
                            }
                        ],
                        "max_tokens": 10
                    }
                })
                .into(),
                &mut pipeline,
            )
            .await?;
        let mut stream = results.stream()?;
        while let Some(o) = stream.next().await {
            o?;
        }

        collection.archive().await?;
        Ok(())
    }
}
