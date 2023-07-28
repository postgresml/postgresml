use sea_query::enum_def;
use sqlx::types::Uuid;
use sqlx::FromRow;

use crate::types::{DateTime, Json};

#[cfg(feature = "javascript")]
use crate::languages::javascript::*;

/// A pipeline
#[derive(FromRow)]
pub struct Pipeline {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime,
    pub model_id: i64,
    pub splitter_id: i64,
    pub active: bool,
    pub chunks_status: String,
    pub embeddings_status: String,
    pub tsvectors_status: String,
}

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
    pub project_id: i64,
}

/// A text splitter
#[enum_def]
#[derive(FromRow)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub parameters: Json,
}

/// A model used to perform some task
#[enum_def]
#[derive(FromRow)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime,
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
    pub chunk: String,
}
