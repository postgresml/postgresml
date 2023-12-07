//! # pgml
//!
//! pgml is an open source alternative for building end-to-end vector search applications without OpenAI and Pinecone
//!
//! With this SDK, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using PgVector for fast and accurate queries.

use parking_lot::RwLock;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use std::env;
use tokio::runtime::Runtime;
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
pub mod models;
mod open_source_ai;
mod order_by_builder;
mod pipeline;
mod queries;
mod query_builder;
mod query_runner;
mod remote_embeddings;
mod splitter;
pub mod transformer_pipeline;
pub mod types;
mod utils;

// Re-export
pub use builtins::Builtins;
pub use collection::Collection;
pub use model::Model;
pub use open_source_ai::OpenSourceAI;
pub use pipeline::Pipeline;
pub use splitter::Splitter;
pub use transformer_pipeline::TransformerPipeline;

// This is use when inserting collections to set the sdk_version used during creation
static SDK_VERSION: &str = "0.9.2";

// Store the database(s) in a global variable so that we can access them from anywhere
// This is not necessarily idiomatic Rust, but it is a good way to acomplish what we need
static DATABASE_POOLS: RwLock<Option<HashMap<String, PgPool>>> = RwLock::new(None);

// Even though this function does not use async anywhere, for whatever reason it must be async or
// sqlx's connect_lazy will throw an error
async fn get_or_initialize_pool(database_url: &Option<String>) -> anyhow::Result<PgPool> {
    let mut pools = DATABASE_POOLS.write();
    let pools = pools.get_or_insert_with(HashMap::new);
    let environment_url = std::env::var("DATABASE_URL");
    let environment_url = environment_url.as_deref();
    let url = database_url
        .as_deref()
        .unwrap_or_else(|| environment_url.expect("Please set DATABASE_URL environment variable"));
    if let Some(pool) = pools.get(url) {
        Ok(pool.clone())
    } else {
        let timeout = std::env::var("PGML_CHECKOUT_TIMEOUT")
            .unwrap_or_else(|_| "5000".to_string())
            .parse::<u64>()
            .expect("Error parsing PGML_CHECKOUT_TIMEOUT, expected an integer");

        let pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(timeout))
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
#[allow(dead_code)]
static mut RUNTIME: Option<Runtime> = None;

#[allow(dead_code)]
fn get_or_set_runtime<'a>() -> &'a Runtime {
    unsafe {
        if let Some(r) = &RUNTIME {
            r
        } else {
            let runtime = Runtime::new().unwrap();
            RUNTIME = Some(runtime);
            get_or_set_runtime()
        }
    }
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
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_ccc_0", None);
        assert!(collection.database_data.is_none());
        collection.verify_in_database(false).await?;
        assert!(collection.database_data.is_some());
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_add_remove_pipeline() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
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

    // #[sqlx::test]
    // async fn can_add_remove_pipelines() -> anyhow::Result<()> {
    //     internal_init_logger(None, None).ok();
    //     let model = Model::default();
    //     let splitter = Splitter::default();
    //     let mut pipeline1 = Pipeline::new(
    //         "test_r_p_carps_0",
    //         Some(model.clone()),
    //         Some(splitter.clone()),
    //         None,
    //     );
    //     let mut pipeline2 = Pipeline::new("test_r_p_carps_1", Some(model), Some(splitter), None);
    //     let mut collection = Collection::new("test_r_c_carps_1", None);
    //     collection.add_pipeline(&mut pipeline1).await?;
    //     collection.add_pipeline(&mut pipeline2).await?;
    //     let pipelines = collection.get_pipelines().await?;
    //     assert!(pipelines.len() == 2);
    //     collection.remove_pipeline(&mut pipeline1).await?;
    //     let pipelines = collection.get_pipelines().await?;
    //     assert!(pipelines.len() == 1);
    //     assert!(collection.get_pipeline("test_r_p_carps_0").await.is_err());
    //     collection.archive().await?;
    //     Ok(())
    // }

    #[sqlx::test]
    async fn can_specify_custom_hnsw_parameters_for_pipelines() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cschpfp_0",
            Some(model),
            Some(splitter),
            Some(
                serde_json::json!({
                    "hnsw": {
                        "m": 100,
                        "ef_construction": 200
                    }
                })
                .into(),
            ),
        );
        let collection_name = "test_r_c_cschpfp_1";
        let mut collection = Collection::new(collection_name, None);
        collection.add_pipeline(&mut pipeline).await?;
        let full_embeddings_table_name = pipeline.create_or_get_embeddings_table().await?;
        let embeddings_table_name = full_embeddings_table_name.split('.').collect::<Vec<_>>()[1];
        let pool = get_or_initialize_pool(&None).await?;
        let results: Vec<(String, String)> = sqlx::query_as(&query_builder!(
            "select indexname, indexdef from pg_indexes where tablename = '%d' and schemaname = '%d'",
            embeddings_table_name,
            collection_name
        )).fetch_all(&pool).await?;
        let names = results.iter().map(|(name, _)| name).collect::<Vec<_>>();
        let definitions = results
            .iter()
            .map(|(_, definition)| definition)
            .collect::<Vec<_>>();
        assert!(names.contains(&&format!("{}_pipeline_hnsw_vector_index", pipeline.name)));
        assert!(definitions.contains(&&format!("CREATE INDEX {}_pipeline_hnsw_vector_index ON {} USING hnsw (embedding vector_cosine_ops) WITH (m='100', ef_construction='200')", pipeline.name, full_embeddings_table_name)));
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
        collection.disable_pipeline(&pipeline).await?;
        let queried_pipelines = &collection.get_pipelines().await?;
        assert!(queried_pipelines.is_empty());
        collection.enable_pipeline(&pipeline).await?;
        let queried_pipeline = &collection.get_pipelines().await?[0];
        assert_eq!(pipeline.name, queried_pipeline.name);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn sync_multiple_pipelines() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
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
            .upsert_documents(generate_dummy_documents(3), None)
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
        internal_init_logger(None, None).ok();
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
            .upsert_documents(generate_dummy_documents(3), None)
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
        internal_init_logger(None, None).ok();
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
        let mut collection = Collection::new("test_r_c_cvswre_21", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let mut pipeline = Pipeline::new("test_r_p_cvswre_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3), None)
            .await?;
        let results = collection
            .vector_search("Here is some query", &mut pipeline, None, Some(10))
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_query_builder() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
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
        let mut collection = Collection::new("test_r_c_cvswqb_4", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let pipeline = Pipeline::new("test_r_p_cvswqb_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(4), None)
            .await?;
        let results = collection
            .query()
            .vector_recall("Here is some query", &pipeline, None)
            .limit(3)
            .fetch_all()
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_query_builder_and_pass_model_parameters_in_search(
    ) -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let model = Model::new(
            Some("hkunlp/instructor-base".to_string()),
            Some("python".to_string()),
            Some(json!({"instruction": "Represent the Wikipedia document for retrieval: "}).into()),
        );
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cvswqbapmpis_1",
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
        let mut collection = Collection::new("test_r_c_cvswqbapmpis_4", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let pipeline = Pipeline::new("test_r_p_cvswqbapmpis_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3), None)
            .await?;
        let results = collection
            .query()
            .vector_recall(
                "Here is some query",
                &pipeline,
                Some(
                    json!({
                        "instruction": "Represent the Wikipedia document for retrieval: "
                    })
                    .into(),
                ),
            )
            .limit(10)
            .fetch_all()
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_query_builder_with_remote_embeddings() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
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
        let mut collection = Collection::new("test_r_c_cvswqbwre_5", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let pipeline = Pipeline::new("test_r_p_cvswqbwre_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(4), None)
            .await?;
        let results = collection
            .query()
            .vector_recall("Here is some query", &pipeline, None)
            .limit(3)
            .fetch_all()
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_query_builder_and_custom_hnsw_ef_search_value(
    ) -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
        let mut pipeline =
            Pipeline::new("test_r_p_cvswqbachesv_1", Some(model), Some(splitter), None);
        let mut collection = Collection::new("test_r_c_cvswqbachesv_3", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let pipeline = Pipeline::new("test_r_p_cvswqbachesv_1", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3), None)
            .await?;
        let results = collection
            .query()
            .vector_recall(
                "Here is some query",
                &pipeline,
                Some(
                    json!({
                        "hnsw": {
                            "ef_search": 2
                        }
                    })
                    .into(),
                ),
            )
            .fetch_all()
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_vector_search_with_query_builder_and_custom_hnsw_ef_search_value_and_remote_embeddings(
    ) -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let model = Model::new(
            Some("text-embedding-ada-002".to_string()),
            Some("openai".to_string()),
            None,
        );
        let splitter = Splitter::default();
        let mut pipeline = Pipeline::new(
            "test_r_p_cvswqbachesvare_2",
            Some(model),
            Some(splitter),
            None,
        );
        let mut collection = Collection::new("test_r_c_cvswqbachesvare_7", None);
        collection.add_pipeline(&mut pipeline).await?;

        // Recreate the pipeline to replicate a more accurate example
        let pipeline = Pipeline::new("test_r_p_cvswqbachesvare_2", None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(3), None)
            .await?;
        let results = collection
            .query()
            .vector_recall(
                "Here is some query",
                &pipeline,
                Some(
                    json!({
                        "hnsw": {
                            "ef_search": 2
                        }
                    })
                    .into(),
                ),
            )
            .fetch_all()
            .await?;
        assert!(results.len() == 3);
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    async fn can_filter_vector_search() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
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
            .upsert_documents(generate_dummy_documents(5), None)
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
                .vector_recall("Here is some query", &pipeline, None)
                .filter(filter)
                .fetch_all()
                .await?;
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
        internal_init_logger(None, None).ok();
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
        collection.upsert_documents(documents.clone(), None).await?;
        let document = &collection.get_documents(None).await?[0];
        assert_eq!(document["document"]["text"], "hello world 1");

        // Test upsert of text and metadata
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
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cpgd_2", None);
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

    #[sqlx::test]
    async fn can_filter_and_paginate_get_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
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
            .upsert_documents(generate_dummy_documents(10), None)
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
        internal_init_logger(None, None).ok();
        let model = Model::default();
        let splitter = Splitter::default();
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
            .upsert_documents(generate_dummy_documents(10), None)
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

    #[sqlx::test]
    fn can_order_documents() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cod_1", None);
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
        collection.archive().await?;
        Ok(())
    }

    #[sqlx::test]
    fn can_merge_metadata() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let mut collection = Collection::new("test_r_c_cmm_4", None);
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
                        "metadata": {
                            "merge": true
                        }
                    })
                    .into(),
                ),
            )
            .await?;
        let documents = collection
            .get_documents(Some(
                json!({"order_by": {"number": {"number": "asc"}}}).into(),
            ))
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
}
