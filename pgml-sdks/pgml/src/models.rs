use sea_query::enum_def;
use serde::Serialize;
use sqlx::types::Uuid;
use sqlx::FromRow;

use crate::types::{DateTime, Json};

// A pipeline
#[enum_def]
#[derive(FromRow)]
pub struct Pipeline {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime,
    pub model_id: i64,
    pub splitter_id: i64,
    pub active: bool,
    pub parameters: Json,
}

// A model used to perform some task
#[enum_def]
#[derive(FromRow)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime,
    pub runtime: String,
    pub hyperparams: Json,
}

// A text splitter
#[enum_def]
#[derive(FromRow)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub parameters: Json,
}

// A pipeline with its model and splitter
#[derive(FromRow, Clone)]
pub struct PipelineWithModelAndSplitter {
    pub pipeline_id: i64,
    pub pipeline_name: String,
    pub pipeline_created_at: DateTime,
    pub pipeline_active: bool,
    pub pipeline_parameters: Json,
    pub model_id: i64,
    pub model_created_at: DateTime,
    pub model_runtime: String,
    pub model_hyperparams: Json,
    pub splitter_id: i64,
    pub splitter_created_at: DateTime,
    pub splitter_name: String,
    pub splitter_parameters: Json,
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
    pub metadata: Json,
    pub text: String,
}

impl Document {
    pub fn into_user_friendly_json(mut self) -> Json {
        self.metadata["text"] = self.text.into();
        serde_json::json!({
            "row_id": self.id,
            "created_at": self.created_at,
            "source_uuid": self.source_uuid,
            "document": self.metadata,
        })
        .into()
    }
}

// A collection of documents
#[enum_def]
#[derive(FromRow)]
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
pub struct Embedding {
    pub id: i64,
    pub created_at: DateTime,
    pub chunk_id: i64,
    pub embedding: Vec<f32>,
}

// A chunk of split text
#[derive(FromRow)]
pub struct Chunk {
    pub id: i64,
    pub created_at: DateTime,
    pub document_id: i64,
    pub splitter_id: i64,
    pub chunk_index: i64,
    pub chunk: String,
}
