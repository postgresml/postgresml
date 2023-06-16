use pgml_macros::{custom_into_js_result, custom_into_py};
use sqlx::types::Uuid;
use sqlx::FromRow;
use std::collections::HashMap;

use crate::languages::javascript::*;

/// A wrapper around sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>
#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct DateTime(pub sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>);

/// A wrapper around sqlx::types::Json<HashMap<String, String>>
#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct JsonHashMap(pub sqlx::types::Json<HashMap<String, String>>);

/// A document
#[derive(FromRow)]
pub struct Document {
    pub id: i64,
    pub created_at: DateTime,
    pub source_uuid: Uuid,
    pub metadata: JsonHashMap,
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
    pub parameters: JsonHashMap,
}

/// A model used to perform some task
#[derive(custom_into_js_result, custom_into_py, FromRow)]
pub struct Model {
    pub id: i64,
    pub created_at: DateTime,
    pub task: String,
    pub name: String,
    pub parameters: JsonHashMap,
}
