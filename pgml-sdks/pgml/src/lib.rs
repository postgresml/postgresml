//! # pgml
//!
//! pgml is an open source alternative for building end-to-end vector search applications without OpenAI and Pinecone
//!
//! With this SDK, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using PgVector for fast and accurate queries.

use sqlx::PgPool;
use std::collections::HashMap;
use std::env;
use std::sync::RwLock;
use tokio::runtime::{Builder, Runtime};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod builtins;
mod collection;
mod filter_builder;
mod languages;
mod model;
pub mod models;
mod pipeline;
mod queries;
mod query_builder;
mod query_runner;
mod remote_embeddings;
mod splitter;
pub mod types;
mod utils;

// Re-export
pub use builtins::Builtins;
pub use collection::Collection;
pub use model::Model;
pub use pipeline::Pipeline;
pub use splitter::Splitter;

// Store the database(s) in a global variable so that we can access them from anywhere
// This is not necessarily idiomatic Rust, but it is a good way to acomplish what we need
static DATABASE_POOLS: RwLock<Option<HashMap<String, PgPool>>> = RwLock::new(None);

// Even though this function does not use async anywhere, for whatever reason it must be async or
// sqlx's connect_lazy will throw an error
async fn get_or_initialize_pool(database_url: &Option<String>) -> anyhow::Result<PgPool> {
    let mut pools = DATABASE_POOLS
        .write()
        .expect("Error getting DATABASE_POOLS for writing");
    let pools = pools.get_or_insert_with(HashMap::new);
    let environment_url = std::env::var("DATABASE_URL");
    let environment_url = environment_url.as_deref();
    let url = database_url
        .as_deref()
        .unwrap_or(environment_url.expect("Please set DATABASE_URL environment variable"));
    if let Some(pool) = pools.get(url) {
        Ok(pool.clone())
    } else {
        let pool = PgPool::connect_lazy(url)?;
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
fn init_logger(level: Option<String>, format: Option<String>) -> anyhow::Result<()> {
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
#[allow(dead_code)]
static mut RUNTIME: Option<Runtime> = None;

#[allow(dead_code)]
fn get_or_set_runtime<'a>() -> &'a Runtime {
    unsafe {
        if let Some(r) = &RUNTIME {
            r
        } else {
            let runtime = Builder::new_current_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap();
            RUNTIME = Some(runtime);
            get_or_set_runtime()
        }
    }
}

#[cfg(feature = "python")]
#[pyo3::prelude::pyfunction]
fn py_init_logger(level: Option<String>, format: Option<String>) -> pyo3::PyResult<()> {
    init_logger(level, format).ok();
    Ok(())
}

#[cfg(feature = "python")]
#[pyo3::pymodule]
fn pgml(_py: pyo3::Python, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(py_init_logger, m)?)?;
    m.add_class::<pipeline::PipelinePython>()?;
    m.add_class::<collection::CollectionPython>()?;
    m.add_class::<model::ModelPython>()?;
    m.add_class::<splitter::SplitterPython>()?;
    m.add_class::<builtins::BuiltinsPython>()?;
    Ok(())
}

#[cfg(feature = "javascript")]
fn js_init_logger(
    mut cx: neon::context::FunctionContext,
) -> neon::result::JsResult<neon::types::JsUndefined> {
    use rust_bridge::javascript::{FromJsType, IntoJsResult};
    let level = cx.argument_opt(0);
    let level = <Option<String>>::from_option_js_type(&mut cx, level)?;
    let format = cx.argument_opt(1);
    let format = <Option<String>>::from_option_js_type(&mut cx, format)?;
    init_logger(level, format).ok();
    ().into_js_result(&mut cx)
}

#[cfg(feature = "javascript")]
#[neon::main]
fn main(mut cx: neon::context::ModuleContext) -> neon::result::NeonResult<()> {
    cx.export_function("js_init_logger", js_init_logger)?;
    cx.export_function("newCollection", collection::CollectionJavascript::new)?;
    cx.export_function("newModel", model::ModelJavascript::new)?;
    cx.export_function("newSplitter", splitter::SplitterJavascript::new)?;
    cx.export_function("newBuiltins", builtins::BuiltinsJavascript::new)?;
    cx.export_function("newPipeline", pipeline::PipelineJavascript::new)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::Model, pipeline::Pipeline, splitter::Splitter, types::Json};
    use serde_json::json;

    fn generate_dummy_documents(count: usize) -> Vec<Json> {
        let mut documents = Vec::new();
        for i in 0..count {
            let document = serde_json::json!(
            {
                "id": i,
                "text": format!("This is a test document: {}", i),
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

    #[sqlx::test]
    async fn can_create_collection() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_ccc_0", None);
        assert!(collection.database_data.is_none());
        collection.verify_in_database(false).await?;
        assert!(collection.database_data.is_some());
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_add_remove_pipeline() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_p_cap_57",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );
        let mut collection = Collection::new("test_r_c_carp_3", None);
        assert!(collection.database_data.is_none());
        collection.add_pipeline(&mut pipeline).await?;
        assert!(collection.database_data.is_some());
        collection.remove_pipeline(&mut pipeline).await?;
        let pipelines = collection.get_pipelines().await?;
        assert!(pipelines.is_empty());
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_add_remove_pipelines() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline1 = Pipeline::new(
            "test_r_p_carps_0",
            Some(model.clone()),
            Some(splitter.clone()),
            None,
        );
        let mut pipeline2 = Pipeline::new("test_r_p_carps_1", Some(model), Some(splitter), None);
        let mut collection = Collection::new("test_r_c_carps_1", None);
        collection.add_pipeline(&mut pipeline1).await?;
        collection.add_pipeline(&mut pipeline2).await?;
        let pipelines = collection.get_pipelines().await?;
        assert!(pipelines.len() == 2);
        collection.remove_pipeline(&mut pipeline1).await?;
        let pipelines = collection.get_pipelines().await?;
        assert!(pipelines.len() == 1);
        assert!(collection.get_pipeline("test_r_p_carps_0").await.is_err());
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn disable_enable_pipeline() -> anyhow::Result<()> {
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new("test_p_dep_0", Some(model), Some(splitter), None);
        let mut collection = Collection::new("test_r_c_dep_1", None);
        collection.add_pipeline(&mut pipeline).await?;
        let queried_pipeline = &collection.get_pipelines().await?[0];
        assert_eq!(pipeline.name, queried_pipeline.name);
        collection.disable_pipeline(&mut pipeline).await?;
        let queried_pipelines = &collection.get_pipelines().await?;
        assert!(queried_pipelines.is_empty());
        collection.enable_pipeline(&mut pipeline).await?;
        let queried_pipeline = &collection.get_pipelines().await?[0];
        assert_eq!(pipeline.name, queried_pipeline.name);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn sync_multiple_pipelines() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline1 = Pipeline::new(
            "test_r_p_smp_0",
            Some(model.clone()),
            Some(splitter.clone()),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );
        let mut pipeline2 = Pipeline::new(
            "test_r_p_smp_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );
        let mut collection = Collection::new("test_r_c_smp_3", None);
        collection.add_pipeline(&mut pipeline1).await?;
        collection.add_pipeline(&mut pipeline2).await?;
        collection
            .upsert_documents(generate_dummy_documents(3))
            .await?;
        let status_1 = pipeline1.get_status().await?;
        let status_2 = pipeline2.get_status().await?;
        assert!(
            status_1.chunks_status.synced == status_1.chunks_status.total
                && status_1.chunks_status.not_synced == 0
        );
        assert!(
            status_2.chunks_status.synced == status_2.chunks_status.total
                && status_2.chunks_status.not_synced == 0
        );
        collection.archive().await?;
        Ok(())
    }

    ///////////////////////////////
    // Various Searches ///////////
    ///////////////////////////////

    #[sqlx::test]
    async fn can_vector_search_with_local_embeddings() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cvswle_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );
        let mut collection = Collection::new("test_r_c_cvswle_28", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let mut pipeline = Pipeline::new("test_r_p_cvswle_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3))
            .await?;
        let results = collection
            .vector_search("Here is some query", &mut pipeline, None, None)
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_remote_embeddings() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::new(
            Some("text-embedding-ada-002".to_string()),
            Some("openai".to_string()),
            None,
        );
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cvswre_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );
        let mut collection = Collection::new("test_r_c_cvswre_20", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let mut pipeline = Pipeline::new("test_r_p_cvswre_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3))
            .await?;
        let results = collection
            .vector_search("Here is some query", &mut pipeline, None, None)
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_query_builder() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cvswqb_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );
        let mut collection = Collection::new("test_r_c_cvswqb_3", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let mut pipeline = Pipeline::new("test_r_p_cvswqb_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3))
            .await?;
        let results = collection
            .query()
            .vector_recall("Here is some query", &mut pipeline, None)
            .fetch_all()
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_query_builder_with_remote_embeddings() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::new(
            Some("text-embedding-ada-002".to_string()),
            Some("openai".to_string()),
            None,
        );
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cvswqbwre_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );
        let mut collection = Collection::new("test_r_c_cvswqbwre_3", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let mut pipeline = Pipeline::new("test_r_p_cvswqbwre_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3))
            .await?;
        let results = collection
            .query()
            .vector_recall("Here is some query", &mut pipeline, None)
            .fetch_all()
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_filter_documents() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::new(None, None, None);
        let splitter = Splitter::new(None, None);
        let mut pipeline = Pipeline::new(
            "test_r_p_cfd_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                "full_text_search": {
                    "active": true,
                    "configuration": "english"
                }
                })
                .into(),
            ),
        );
        let mut collection = Collection::new("test_r_c_cfd_2", None);
        collection.add_pipeline(&mut pipeline).await?;
        collection
            .upsert_documents(generate_dummy_documents(5))
            .await?;

        let filters = vec![
            (5, json!({}).into()),
            (
                3,
                json!({
                    "metadata": {
                        "id": {
                            "$lt": 3
                        }
                    }
                })
                .into(),
            ),
            (
                1,
                json!({
                    "full_text_search": {
                        "configuration": "english",
                        "text": "1",
                    }
                })
                .into(),
            ),
        ];

        for (expected_result_count, filter) in filters {
            let results = collection
                .query()
                .vector_recall("Here is some query", &mut pipeline, None)
                .filter(filter)
                .fetch_all()
                .await?;
            println!("{:?}", results);
            assert_eq!(results.len(), expected_result_count);
        }

        collection.archive().await?;
        Ok(())
    }

    ///////////////////////////////
    // Working With Documents /////
    ///////////////////////////////

    #[sqlx::test]
    async fn can_upsert_and_filter_get_documents() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cuafgd_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );

        let mut collection = Collection::new("test_r_c_cuagd_2", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Test basic upsert
        let documents = vec![
            serde_json::json!({"id": 1, "random_key": 10, "text": "hello world 1"}).into(),
            serde_json::json!({"id": 2, "random_key": 11, "text": "hello world 2"}).into(),
            serde_json::json!({"id": 3, "random_key": 12, "text": "hello world 3"}).into(),
        ];
        collection.upsert_documents(documents.clone()).await?;
        let document = &collection.get_documents(None).await?[0];
        assert_eq!(document["document"]["text"], "hello world 1");

        // Test upsert of text and metadata
        let documents = vec![
            serde_json::json!({"id": 1, "text": "hello world new"}).into(),
            serde_json::json!({"id": 2, "random_key": 12}).into(),
            serde_json::json!({"id": 3, "random_key": 13}).into(),
        ];
        collection.upsert_documents(documents.clone()).await?;

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "metadata": {
                            "random_key": {
                                "$eq": 12
                            }
                        }
                    }
                })
                .into(),
            ))
            .await?;
        assert_eq!(documents[0]["document"]["text"], "hello world 2");

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "metadata": {
                            "random_key": {
                                "$gte": 13
                            }
                        }
                    }
                })
                .into(),
            ))
            .await?;
        assert_eq!(documents[0]["document"]["text"], "hello world 3");

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "full_text_search": {
                            "configuration": "english",
                            "text": "new"
                        }
                    }
                })
                .into(),
            ))
            .await?;
        assert_eq!(documents[0]["document"]["text"], "hello world new");
        assert_eq!(documents[0]["document"]["id"].as_i64().unwrap(), 1);

        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_paginate_get_documents() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cpgd_2", None);
        collection
            .upsert_documents(generate_dummy_documents(10))
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

    #[sqlx::test]
    async fn can_filter_and_paginate_get_documents() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cfapgd_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );

        let mut collection = Collection::new("test_r_c_cfapgd_1", None);
        collection.add_pipeline(&mut pipeline).await?;

        collection
            .upsert_documents(generate_dummy_documents(10))
            .await?;

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "metadata": {
                            "id": {
                                "$gte": 2
                            }
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
                        "metadata": {
                            "id": {
                                "$lte": 5
                            }
                        }
                    },
                    "limit": 100,
                    "offset": 4
                })
                .into(),
            ))
            .await?;
        let last_row_id = documents.last().unwrap()["row_id"].as_i64().unwrap();
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["document"]["id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![4, 5]
        );

        let documents = collection
            .get_documents(Some(
                serde_json::json!({
                    "filter": {
                        "full_text_search": {
                            "configuration": "english",
                            "text": "document"
                        }
                    },
                    "limit": 100,
                    "last_row_id": last_row_id
                })
                .into(),
            ))
            .await?;
        assert_eq!(
            documents
                .into_iter()
                .map(|d| d["document"]["id"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9]
        );

        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_filter_and_delete_documents() -> anyhow::Result<()> {
        init_logger(None, None).ok();
        let model = Model::new(None, None, None);
        let splitter = Splitter::new(None, None);
        let mut pipeline = Pipeline::new(
            "test_r_p_cfadd_1",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "full_text_search": {
                        "active": true,
                        "configuration": "english"
                    }
                })
                .into(),
            ),
        );

        let mut collection = Collection::new("test_r_c_cfadd_1", None);
        collection.add_pipeline(&mut pipeline).await?;
        collection
            .upsert_documents(generate_dummy_documents(10))
            .await?;

        collection
            .delete_documents(
                serde_json::json!({
                    "metadata": {
                        "id": {
                            "$lt": 2
                        }
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
                    "full_text_search": {
                        "configuration": "english",
                        "text": "2"
                    }
                })
                .into(),
            )
            .await?;
        let documents = collection.get_documents(None).await?;
        assert_eq!(documents.len(), 7);
        assert!(documents
            .iter()
            .all(|d| d["document"]["id"].as_i64().unwrap() > 2));

        collection
            .delete_documents(
                serde_json::json!({
                    "metadata": {
                        "id": {
                            "$gte": 6
                        }
                    },
                    "full_text_search": {
                        "configuration": "english",
                        "text": "6"
                    }
                })
                .into(),
            )
            .await?;
        let documents = collection.get_documents(None).await?;
        assert_eq!(documents.len(), 6);
        assert!(documents
            .iter()
            .all(|d| d["document"]["id"].as_i64().unwrap() != 6));

        collection.archive().await?;
        Ok(())
    }
}
