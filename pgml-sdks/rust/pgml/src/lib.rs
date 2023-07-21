//! # pgml
//!
//! pgml is an open source alternative for building end-to-end vector search applications without OpenAI and Pinecone
//!
//! With this SDK, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using PgVector for fast and accurate queries.

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::runtime::{Builder, Runtime};

mod builtins;
mod collection;
mod filter_builder;
mod languages;
mod model;
pub mod models;
mod queries;
mod query_builder;
mod query_runner;
mod remote_embeddings;
mod splitter;
pub mod types;
mod utils;

// Pub re-export the Database and Collection structs for use in the rust library
pub use collection::Collection;

// Store the database(s) in a global variable so that we can access them from anywhere
// This is not necessarily idiomatic Rust, but it is a good way to acomplish what we need
static DATABASE_POOLS: RwLock<Option<HashMap<String, PgPool>>> = RwLock::new(None);

async fn get_or_initialize_pool(database_url: &Option<String>) -> anyhow::Result<PgPool> {
    let mut pools = DATABASE_POOLS
        .write()
        .expect("Error getting DATABASE_POOLS for writing");
    let pools = pools.get_or_insert_with(|| HashMap::new());
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

// Normally libraries leave it up to up to the rust executable using the library to init the
// logger, but because we are used by programs in Python and other languages that do
// not have the ability to do that, we init it for those languages, but leave it uninitialized when
// used natively with rust
struct SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn init_logger(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}

// Normally the global async runtime is handled by tokio but because we are a library being called
// by javascript and other langauges, we occasionally need to handle it ourselves
static mut RUNTIME: Option<Runtime> = None;

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
fn setup_logger(level: &str) -> pyo3::PyResult<()> {
    let level = match level {
        "DEBUG" => LevelFilter::Debug,
        "INFO" => LevelFilter::Info,
        "WARN" => LevelFilter::Warn,
        "ERROR" => LevelFilter::Error,
        _ => LevelFilter::Error,
    };
    init_logger(level).ok();
    Ok(())
}

#[cfg(feature = "python")]
#[pyo3::pymodule]
fn pgml(_py: pyo3::Python, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    // We may want to move this into the new function in the DatabasePython struct and give the
    // user the oppertunity to pass in the log level filter
    m.add_function(pyo3::wrap_pyfunction!(setup_logger, m)?)?;
    m.add_class::<collection::CollectionPython>()?;
    m.add_class::<model::ModelPython>()?;
    m.add_class::<splitter::SplitterPython>()?;
    m.add_class::<builtins::BuiltinsPython>()?;
    Ok(())
}

#[cfg(feature = "javascript")]
#[neon::main]
fn main(mut cx: neon::context::ModuleContext) -> neon::result::NeonResult<()> {
    // We may want to move this into the new function in the DatabaseJavascript struct and give the
    // user the oppertunity to pass in the log level filter
    init_logger(LevelFilter::Error).unwrap();
    cx.export_function("newCollection", database::CollectionJavascript::new)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use crate::{builtins::Builtins, model::Model, splitter::Splitter, types::Json};

    fn setup_logger() {
        let log_level = env::var("LOG_LEVEL").unwrap_or("".to_string());
        match log_level.as_str() {
            "DEBUG" => init_logger(LevelFilter::Debug),
            "INFO" => init_logger(LevelFilter::Info),
            "WARN" => init_logger(LevelFilter::Warn),
            "ERROR" => init_logger(LevelFilter::Error),
            _ => Ok(()),
        }
        .ok();
    }

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

    #[tokio::test]
    async fn can_lazily_create_collection() {
        setup_logger();
        let collection_name = "r_ccc_test_4";
        let mut collection = Collection::new(collection_name, None);
        let builtins = Builtins::new(None);
        // Collection will not exist in the database because it does not need to
        let does_collection_exist = builtins
            .does_collection_exist(collection_name)
            .await
            .unwrap();
        assert!(!does_collection_exist);
        // Do something that requires the collection to be created
        collection
            .upsert_documents(generate_dummy_documents(1), None, None)
            .await
            .unwrap();
        // Now the collection will exist because it had to be created to upsert documents
        let does_collection_exist = builtins
            .does_collection_exist(collection_name)
            .await
            .unwrap();
        collection.archive().await.unwrap();
        assert!(does_collection_exist);
    }

    #[tokio::test]
    async fn can_lazily_create_model() {
        setup_logger();
        let mut model = Model::new(None, None, None, None, None);
        assert!(model.id.is_none());
        let id = model.get_id().await.unwrap();
        assert_eq!(id, model.id.unwrap());
    }

    #[tokio::test]
    async fn can_lazily_create_splitter() {
        setup_logger();
        let mut splitter = Splitter::new(None, None, None);
        assert!(splitter.id.is_none());
        let id = splitter.get_id().await.unwrap();
        assert_eq!(id, splitter.id.unwrap());
    }

    #[tokio::test]
    async fn can_vector_search() {
        setup_logger();
        let collection_name = "r_cvs_test_5";
        let mut collection = Collection::new(collection_name, None);
        let mut model = Model::new(None, None, None, None, None);
        let mut splitter = Splitter::new(None, None, None);
        collection
            .upsert_documents(generate_dummy_documents(2), None, None)
            .await
            .unwrap();
        // Splitter should not be verified in the database yet
        assert!(!splitter.verified_in_database);
        collection.generate_chunks(&mut splitter).await.unwrap();
        // Now splitter should be verified in the database
        assert!(splitter.verified_in_database);
        // Model should not be verified in the database yet
        assert!(!model.verified_in_database);
        collection
            .generate_embeddings(&mut model, &mut splitter)
            .await
            .unwrap();
        // Now model should be verified in the database
        assert!(model.verified_in_database);
        let results = collection
            .vector_search("Here is some query", &model, &splitter, None, None)
            .await
            .unwrap();
        collection.archive().await.unwrap();
        assert!(results.len() > 0);
    }

    #[tokio::test]
    async fn can_vector_search_with_remote_embeddings() {
        setup_logger();
        let collection_name = "r_cvswre_test_1";
        let mut model = Model::new(
            Some("text-embedding-ada-002".to_string()),
            None,
            Some("openai".to_string()),
            None,
            None,
        );
        let mut splitter = Splitter::new(None, None, None);
        let mut collection = Collection::new(collection_name, None);
        collection
            .upsert_documents(generate_dummy_documents(2), None, None)
            .await
            .unwrap();
        collection.generate_chunks(&mut splitter).await.unwrap();
        collection
            .generate_embeddings(&mut model, &mut splitter)
            .await
            .unwrap();
        let results = collection
            .vector_search("Here is some query", &model, &splitter, None, None)
            .await
            .unwrap();
        collection.archive().await.unwrap();
        assert!(results.len() > 0);
    }

    #[tokio::test]
    async fn can_vector_search_with_query_builder() {
        setup_logger();
        let collection_name = "r_cvswqb_test_2";
        let mut model = Model::new(None, None, None, None, None);
        let mut splitter = Splitter::new(None, None, None);
        let mut collection = Collection::new(collection_name, None);
        collection
            .upsert_documents(generate_dummy_documents(2), None, None)
            .await
            .unwrap();
        collection.generate_chunks(&mut splitter).await.unwrap();
        collection
            .generate_embeddings(&mut model, &mut splitter)
            .await
            .unwrap();
        collection.generate_tsvectors(None).await.unwrap();
        let filter = serde_json::json! ({
            "metadata": {
                "metadata": {
                    "$or": [
                        {"uuid": {"$eq": 0 }},
                        {"uuid": {"$eq": 10 }},
                        {"category": {"$eq": [1, 2, 3]}}
                    ]

                }
            },
            "full_text": {
                "text": "Test document"
            }
        });
        let results = collection
            .query()
            .vector_recall("Here is some query".to_string(), &model, &splitter, None)
            .filter(filter.into())
            .limit(10)
            .run()
            .await
            .unwrap();
        collection.archive().await.unwrap();
        assert!(results.len() > 0);
    }

    // #[tokio::test]
    // async fn can_register_and_get_model() {
    //     let db = create_db_connection_from_env().await.unwrap();
    //     let model = db.register_model(None, None, None, None).await.unwrap();
    //     let model_from_database = db.get_model(model.id).await.unwrap();
    //     assert_eq!(model.id, model_from_database.id);
    // }
    //
    // #[tokio::test]
    // async fn can_register_and_get_text_splitter() {
    //     let db = create_db_connection_from_env().await.unwrap();
    //     let text_splitter = db.register_text_splitter(None, None).await.unwrap();
    //     let text_splitter_from_database = db.get_text_splitter(text_splitter.id).await.unwrap();
    //     assert_eq!(text_splitter.id, text_splitter_from_database.id);
    // }
    //
    // #[tokio::test]
    // async fn can_vector_search() {
    //     let collection_name = "r_cvs_test_1";
    //     let db = create_db_connection_from_env().await.unwrap();
    //     let collection = db.create_or_get_collection(collection_name).await.unwrap();
    //     let documents = generate_dummy_documents(2);
    //     collection
    //         .upsert_documents(documents, None, None)
    //         .await
    //         .unwrap();
    //     let text_splitter = db.register_text_splitter(None, None).await.unwrap();
    //     let model = db.register_model(None, None, None, None).await.unwrap();
    //     collection.generate_chunks(&text_splitter).await.unwrap();
    //     collection
    //         .generate_embeddings(&model, &text_splitter)
    //         .await
    //         .unwrap();
    //     let results = collection
    //         .vector_search("Here is some query", &model, &text_splitter, None, None)
    //         .await
    //         .unwrap();
    //     db.archive_collection(collection_name).await.unwrap();
    //     assert!(results.len() > 0);
    // }
    //
    // #[tokio::test]
    // async fn can_vector_search_with_remote_embeddings() {
    //     let collection_name = "r_cvswre_test_0";
    //     let db = create_db_connection_from_env().await.unwrap();
    //     let collection = db.create_or_get_collection(collection_name).await.unwrap();
    //     let documents = generate_dummy_documents(2);
    //     collection
    //         .upsert_documents(documents, None, None)
    //         .await
    //         .unwrap();
    //     let text_splitter = db.register_text_splitter(None, None).await.unwrap();
    //     let model = db
    //         .register_model(
    //             Some("text-embedding-ada-002".to_string()),
    //             None,
    //             None,
    //             Some("openai".to_string()),
    //         )
    //         .await
    //         .unwrap();
    //     collection.generate_chunks(&text_splitter).await.unwrap();
    //     collection
    //         .generate_embeddings(&model, &text_splitter)
    //         .await
    //         .unwrap();
    //     let results = collection
    //         .vector_search("Here is some query", &model, &text_splitter, None, None)
    //         .await
    //         .unwrap();
    //     db.archive_collection(collection_name).await.unwrap();
    //     assert!(results.len() > 0);
    // }

    // #[tokio::test]
    // async fn query_builder() {
    //     let connection_string = env::var("DATABASE_URL").unwrap();
    //     init_logger(LevelFilter::Error).ok();
    //
    //     let collection_name = "rqbmftest11";
    //
    //     let db = Database::new(&connection_string).await.unwrap();
    //     let collection = db.create_or_get_collection(collection_name).await.unwrap();
    //
    //     let mut documents: Vec<Json> = Vec::new();
    //     for i in 0..5 {
    //         documents.push(serde_json::json!({
    //             "id": i,
    //             "metadata": {
    //                 "uuid": i,
    //                 "category": [i, 2, 3]
    //             },
    //             "text": format!("{} This is some document with some filler text filler filler filler filler filler filler filler filler filler", i)
    //         }).into());
    //     }
    //
    //     collection
    //         .upsert_documents(documents, None, None)
    //         .await
    //         .unwrap();
    //     let parameters = Json::from(serde_json::json!({
    //         "chunk_size": 1500,
    //         "chunk_overlap": 40,
    //     }));
    //     collection
    //         .register_text_splitter(None, Some(parameters))
    //         .await
    //         .unwrap();
    //     collection.generate_chunks(None).await.unwrap();
    //     collection
    //         .register_model(None, None, None, None)
    //         .await
    //         .unwrap();
    //     collection.generate_embeddings(None, None).await.unwrap();
    //     collection.generate_tsvectors(None).await.unwrap();
    //
    //     let filter = serde_json::json! ({
    //         "metadata": {
    //             "metadata": {
    //                 "$or": [
    //                     {"uuid": {"$eq": 1 }},
    //                     {"uuid": {"$eq": 2 }},
    //                     {"category": {"$eq": [1, 2, 3]}}
    //                 ]
    //
    //             }
    //         },
    //         "full_text": {
    //             "text": "filler text"
    //         }
    //     });
    //
    //     let query = collection
    //         .query()
    //         .vector_recall("test query".to_string(), None, None, None)
    //         .filter(filter.into())
    //         .limit(10);
    //     println!("\n{}\n", query.to_string());
    //     let results = query.run().await.unwrap();
    //     println!("{:?}", results);
    // }
    //
    // #[tokio::test]
    // async fn query_runner() {
    //     let connection_string = env::var("DATABASE_URL").unwrap();
    //     init_logger(LevelFilter::Info).ok();
    //
    //     let db = Database::new(&connection_string).await.unwrap();
    //     let query = db.query("SELECT * from pgml.collections");
    //     let results = query.fetch_all().await.unwrap();
    //     println!("{:?}", results);
    // }
    //
    // #[tokio::test]
    // async fn transform() {
    //     let connection_string = env::var("DATABASE_URL").unwrap();
    //     init_logger(LevelFilter::Info).ok();
    //
    //     let db = Database::new(&connection_string).await.unwrap();
    //     // let task = Json::from(serde_json::json!("text-classification"));
    //     let task = Json::from(serde_json::json!("translation_en_to_fr"));
    //     let inputs = vec!["test1".to_string(), "test2".to_string()];
    //     let results = db.transform(task, inputs, None).await.unwrap();
    //     println!("{:?}", results);
    // }
    //
    // #[tokio::test]
    // async fn collection_errors() {
    //     let connection_string = env::var("DATABASE_URL").unwrap();
    //     init_logger(LevelFilter::Info).ok();
    //
    //     let db = Database::new(&connection_string).await.unwrap();
    //     let collection_name = "cetest0";
    //     let collection = db.create_or_get_collection(collection_name).await.unwrap();
    //
    //     // Test that we cannot generate tsvectors without upserting documents first
    //     assert!(collection.generate_tsvectors(None).await.is_err());
    //     // Test that we cannot generate chunks without upserting documents first
    //     assert!(collection.generate_chunks(None).await.is_err());
    //     // Test that we cannot generate embeddings without generating chunks first
    //     assert!(collection.generate_embeddings(None, None).await.is_err());
    // }
}
