use pgml_macros::{custom_into_js_result, custom_into_py};
use sea_query::enum_def;
use sqlx::types::Uuid;
use sqlx::FromRow;

use crate::types::{DateTime, Json};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

/// A document
#[enum_def]
#[derive(FromRow)]
pub struct Document {
    pub id: i64,
    pub created_at: DateTime,
    pub source_uuid: Uuid,
    pub metadata: Json,
    pub text: String,
}

/// A collection of documents
#[enum_def]
#[derive(FromRow)]
pub struct Collection {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub active: bool,
}

/// A text splitter
#[enum_def]
#[derive(custom_into_js_result, custom_into_py, FromRow)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub parameters: Json,
}

/// A model used to perform some task
#[enum_def]
#[derive(custom_into_js_result, custom_into_py, FromRow)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime,
    pub task: String,
    pub name: String,
    pub source: String,
    pub parameters: Json,
}

/// An embedding
#[enum_def]
#[derive(FromRow)]
pub struct Embedding {
    pub id: i64,
    pub created_at: DateTime,
    pub chunk_id: i64,
    pub embedding: Vec<f32>,
}

#[derive(FromRow)]
pub struct Chunk {
    pub id: i64,
    pub created_at: DateTime,
    pub document_id: i64,
    pub splitter_id: i64,
    pub chunk_index: i64,
    pub chunk: String
}
