use sea_query::enum_def;
use serde::Serialize;
use sqlx::types::Uuid;
use sqlx::FromRow;

use crate::types::{DateTime, Json};

// A multi field pipeline
#[enum_def]
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Pipeline {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime,
    pub active: bool,
    pub schema: Json,
}

// A model used to perform some task
#[enum_def]
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime,
    pub runtime: String,
    pub hyperparams: Json,
}

// A text splitter
#[enum_def]
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub parameters: Json,
}

// A document
#[enum_def]
#[derive(FromRow, Serialize)]
pub struct Document {
    pub id: i64,
    pub created_at: DateTime,
    #[serde(with = "uuid::serde::compact")]
    // See: https://docs.rs/uuid/latest/uuid/serde/index.html
    pub source_uuid: Uuid,
    pub document: Json,
}

impl Document {
    pub fn into_user_friendly_json(self) -> Json {
        serde_json::json!({
            "row_id": self.id,
            "created_at": self.created_at,
            "source_uuid": self.source_uuid,
            "document": self.document,
        })
        .into()
    }
}

// A collection of documents
#[enum_def]
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Collection {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub active: bool,
    pub project_id: i64,
}

// An embedding
#[enum_def]
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Embedding {
    pub id: i64,
    pub created_at: DateTime,
    pub chunk_id: i64,
    pub embedding: Vec<f32>,
}

// A chunk of split text
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Chunk {
    pub id: i64,
    pub created_at: DateTime,
    pub document_id: i64,
    pub chunk_index: i64,
    pub chunk: String,
}

// A tsvector of a document
#[derive(FromRow)]
#[allow(dead_code)]
pub struct TSVector {
    pub id: i64,
    pub created_at: DateTime,
}
