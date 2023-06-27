use pgml_macros::{custom_into_js_result, custom_into_py};
use sqlx::types::Uuid;
use sqlx::FromRow;

use crate::languages::javascript::*;
use crate::types::{Json, DateTime};

/// A document
#[derive(FromRow)]
pub struct Document {
    pub id: i64,
    pub created_at: DateTime,
    pub source_uuid: Uuid,
    pub metadata: Json,
    pub text: String,
}

/// A collection of documents
#[derive(FromRow)]
pub struct Collection {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub active: bool,
}

/// A text splitter
#[derive(custom_into_js_result, custom_into_py, FromRow)]
pub struct Splitter {
    pub id: i64,
    pub created_at: DateTime,
    pub name: String,
    pub parameters: Json,
}

/// A model used to perform some task
#[derive(custom_into_js_result, custom_into_py, FromRow)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime,
    pub task: String,
    pub name: String,
    pub parameters: Json,
}
