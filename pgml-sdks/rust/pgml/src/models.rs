use pgml_macros::custom_into_py;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::types::Uuid;
use sqlx::FromRow;
use std::collections::HashMap;

/// A document
#[derive(FromRow, Debug, Clone)]
pub struct Document {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub source_uuid: Uuid,
    pub metadata: Json<HashMap<String, String>>,
    pub text: String,
}

/// A collection of documents
#[derive(FromRow, Debug, Clone)]
pub struct Collection {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub active: bool,
}

/// A text splitter
#[derive(custom_into_py, FromRow, Debug, Clone)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub parameters: Json<HashMap<String, String>>,
}

/// A model used to perform some task
#[derive(custom_into_py, FromRow, Debug, Clone)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub task: String,
    pub name: String,
    pub parameters: Json<HashMap<String, String>>,
}
