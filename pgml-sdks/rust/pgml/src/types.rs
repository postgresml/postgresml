use pgml_macros::pgml_alias;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "python")]
use crate::languages::python::*;

/// A wrapper around serde_json::Value
// #[derive(sqlx::Type, sqlx::FromRow, Debug)]
#[derive(pgml_alias, sqlx::Type, Debug, Clone)]
#[sqlx(transparent)]
pub struct Json(pub serde_json::Value);

impl Default for Json {
    fn default() -> Self {
        Self(serde_json::json!({}))
    }
}

impl From<serde_json::Value> for Json {
    fn from(v: serde_json::Value) -> Self {
        Self(v)
    }
}

impl Deref for Json {
    type Target = serde_json::Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Json {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for Json {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_json::Value::serialize(&self.0, serializer)
    }
}

/// A wrapper around sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>
#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(transparent)]
// pub struct DateTime(pub sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>);
pub struct DateTime(pub sqlx::types::chrono::NaiveDateTime);

impl Serialize for DateTime {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.timestamp().serialize(serializer)
    }
}
