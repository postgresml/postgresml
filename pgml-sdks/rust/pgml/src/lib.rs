//! # pgml
//!
//! pgml is an open source alternative for building end-to-end vector search applications without OpenAI and Pinecone
//!
//! With this SDK, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using PgVector for fast and accurate queries.

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use tokio::runtime::{Builder, Runtime};

mod collection;
mod database;
mod languages;
pub mod models;
mod queries;
mod query_builder;
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

    #[tokio::test]
    async fn can_connect_to_database() {
        let connection_string = env::var("DATABASE_URL").unwrap();
        Database::new(&connection_string).await.unwrap();
    }

    #[tokio::test]
    async fn can_create_collection_and_vector_search() {
        let connection_string = env::var("DATABASE_URL").unwrap();

        init_logger(LevelFilter::Error).ok();

        let collection_name = "test34";

        let db = Database::new(&connection_string).await.unwrap();
        let collection = db.create_or_get_collection(collection_name).await.unwrap();

        let documents: Vec<Json> = vec![
            serde_json::json!( {
                "id": 1,
                "text": "This is a document"
            })
            .into(),
            serde_json::json!( {
                "id": 2,
                "text": "This is a second document"
            })
            .into(),
        ];

        collection
            .upsert_documents(documents, None, None)
            .await
            .unwrap();
        let parameters = Json::from(serde_json::json!({
            "chunk_size": 1500,
            "chunk_overlap": 40,
        }));
        collection
            .register_text_splitter(None, Some(parameters))
            .await
            .unwrap();
        collection.generate_chunks(None).await.unwrap();
        collection.register_model(None, None, None).await.unwrap();
        collection.generate_embeddings(None, None).await.unwrap();
        collection
            .vector_search("Here is a test", None, None, None, None)
            .await
            .unwrap();
        db.archive_collection(&collection_name).await.unwrap();
    }

    #[tokio::test]
    async fn query_builder() {
        let connection_string = env::var("DATABASE_URL").unwrap();

        init_logger(LevelFilter::Error).ok();

        let collection_name = "rqtest5";

        let db = Database::new(&connection_string).await.unwrap();
        let collection = db.create_or_get_collection(collection_name).await.unwrap();

        let mut documents: Vec<Json> = Vec::new();
        for i in 0..2 {
            documents.push(serde_json::json!({
                "id": i,
                "text": format!("{} This is some document with some filler text filler filler filler filler filler filler filler filler filler", i)
            }).into());
        }

        collection
            .upsert_documents(documents, None, None)
            .await
            .unwrap();
        let parameters = Json::from(serde_json::json!({
            "chunk_size": 1500,
            "chunk_overlap": 40,
        }));
        collection
            .register_text_splitter(None, Some(parameters))
            .await
            .unwrap();
        collection.generate_chunks(None).await.unwrap();
        collection.register_model(None, None, None).await.unwrap();
        collection.generate_embeddings(None, None).await.unwrap();

        let filter = serde_json::json! ({
            "id": 1
        });

        let query = collection
            .query()
            .vector_recall("test query".to_string(), None, None, None)
            .await
            .unwrap()
            .limit(10);
            // .filter(filter.into());
        query.debug();
        let results = query.run().await.unwrap();
        println!("{:?}", results);
    }
}
