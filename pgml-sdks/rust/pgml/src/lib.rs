//! # pgml
//!
//! pgml is an open source alternative for building end-to-end vector search applications without OpenAI and Pinecone
//!
//! With this SDK, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using PgVector for fast and accurate queries.

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use neon::prelude::*;
use pyo3::prelude::*;
use tokio::runtime::{Builder, Runtime};

mod collection;
mod database;
mod languages;
pub mod models;
mod queries;
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

#[pymodule]
fn pgml(_py: Python, m: &PyModule) -> PyResult<()> {
    // We may want to move this into the new function in the DatabasePython struct and give the
    // user the oppertunity to pass in the log level filter
    init_logger(LevelFilter::Error).unwrap();
    m.add_class::<database::DatabasePython>()?;
    Ok(())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // We may want to move this into the new function in the DatabaseJavascript struct and give the
    // user the oppertunity to pass in the log level filter
    init_logger(LevelFilter::Error).unwrap();
    cx.export_function("newDatabase", database::DatabaseJavascript::new)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env;

    #[tokio::test]
    async fn can_connect_to_database() {
        let connection_string = env::var("DATABASE_URL").unwrap();
        Database::new(&connection_string).await.unwrap();
    }

    #[tokio::test]
    async fn can_create_collection_and_vector_search() {
        let connection_string = env::var("DATABASE_URL").unwrap();

        init_logger(LevelFilter::Info).unwrap();
        let collection_name = "test27";

        let db = Database::new(&connection_string).await.unwrap();
        let collection = db.create_or_get_collection(collection_name).await.unwrap();
        let documents = vec![HashMap::from([
            ("id".to_string(), "1".to_string()),
            ("text".to_string(), "This is a document".to_string()),
        ])];
        collection
            .upsert_documents(documents, None, None)
            .await
            .unwrap();
        collection.register_text_splitter(None, None).await.unwrap();
        collection.generate_chunks(None).await.unwrap();
        collection.register_model(None, None, None).await.unwrap();
        collection.generate_embeddings(None, None).await.unwrap();
        collection
            .vector_search("Here is a test", None, None, None, None)
            .await
            .unwrap();
        db.archive_collection(&collection_name).await.unwrap();
    }
}
