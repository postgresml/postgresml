//! # pgml
//!
//! pgml is an open source alternative for building end-to-end vector search applications without OpenAI and Pinecone
//!
//! With this SDK, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using PgVector for fast and accurate queries.

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use tokio::runtime::{Builder, Runtime};

mod collection;
mod database;
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
pub use database::Database;

// Normally libraries leave it up to up to the rust executable using the library to init the
// logger, but because we are used by programs in Python and other languages that do
// not have the ability to do that, we init it for those languages, but leave it uninitialized when
// used natively with rust
struct SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
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
#[pyo3::pymodule]
fn pgml(_py: pyo3::Python, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    // We may want to move this into the new function in the DatabasePython struct and give the
    // user the oppertunity to pass in the log level filter
    init_logger(LevelFilter::Error).unwrap();
    m.add_class::<database::DatabasePython>()?;
    Ok(())
}

#[cfg(feature = "javascript")]
#[neon::main]
fn main(mut cx: neon::context::ModuleContext) -> neon::result::NeonResult<()> {
    // We may want to move this into the new function in the DatabaseJavascript struct and give the
    // user the oppertunity to pass in the log level filter
    init_logger(LevelFilter::Error).unwrap();
    cx.export_function("newDatabase", database::DatabaseJavascript::new)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use crate::types::Json;

    async fn create_db_connection_from_env() -> anyhow::Result<Database> {
        let connection_string =
            env::var("DATABASE_URL").expect("Please set DATABASE_URL environment variable");
        Database::new(&connection_string).await
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
    async fn can_connect_to_database() {
        let _ = create_db_connection_from_env().await.unwrap();
    }

    #[tokio::test]
    async fn can_create_collection() {
        let collection_name = "r_ccc_test_0";
        let db = create_db_connection_from_env().await.unwrap();
        let _ = db.create_or_get_collection(collection_name).await.unwrap();
        let does_collection_exist = db.does_collection_exist(collection_name).await.unwrap();
        db.archive_collection(collection_name).await.unwrap();
        assert_eq!(does_collection_exist, true);
    }

    #[tokio::test]
    async fn can_register_and_get_model() {
        let db = create_db_connection_from_env().await.unwrap();
        let model = db.register_model(None, None, None, None).await.unwrap();
        let model_from_database = db.get_model(model.id).await.unwrap();
        assert_eq!(model.id, model_from_database.id);
    }

    #[tokio::test]
    async fn can_register_and_get_text_splitter() {
        let db = create_db_connection_from_env().await.unwrap();
        let text_splitter = db.register_text_splitter(None, None).await.unwrap();
        let text_splitter_from_database = db.get_text_splitter(text_splitter.id).await.unwrap();
        assert_eq!(text_splitter.id, text_splitter_from_database.id);
    }

    #[tokio::test]
    async fn can_vector_search() {
        let collection_name = "r_cvs_test_1";
        let db = create_db_connection_from_env().await.unwrap();
        let collection = db.create_or_get_collection(collection_name).await.unwrap();
        let documents = generate_dummy_documents(2);
        collection
            .upsert_documents(documents, None, None)
            .await
            .unwrap();
        let text_splitter = db.register_text_splitter(None, None).await.unwrap();
        let model = db.register_model(None, None, None, None).await.unwrap();
        collection.generate_chunks(&text_splitter).await.unwrap();
        collection
            .generate_embeddings(&model, &text_splitter)
            .await
            .unwrap();
        let results = collection
            .vector_search("Here is some query", &model, &text_splitter, None, None)
            .await
            .unwrap();
        db.archive_collection(collection_name).await.unwrap();
        assert!(results.len() > 0);
    }

    #[tokio::test]
    async fn can_vector_search_with_remote_embeddings() {
        let collection_name = "r_cvswre_test_0";
        let db = create_db_connection_from_env().await.unwrap();
        let collection = db.create_or_get_collection(collection_name).await.unwrap();
        let documents = generate_dummy_documents(2);
        collection
            .upsert_documents(documents, None, None)
            .await
            .unwrap();
        let text_splitter = db.register_text_splitter(None, None).await.unwrap();
        let model = db
            .register_model(
                Some("text-embedding-ada-002".to_string()),
                None,
                None,
                Some("openai".to_string()),
            )
            .await
            .unwrap();
        collection.generate_chunks(&text_splitter).await.unwrap();
        collection
            .generate_embeddings(&model, &text_splitter)
            .await
            .unwrap();
        let results = collection
            .vector_search("Here is some query", &model, &text_splitter, None, None)
            .await
            .unwrap();
        db.archive_collection(collection_name).await.unwrap();
        assert!(results.len() > 0);
    }

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
