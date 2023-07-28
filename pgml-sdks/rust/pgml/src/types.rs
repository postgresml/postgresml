use std::ops::{Deref, DerefMut};

/// A wrapper around serde_json::Value
// #[derive(sqlx::Type, sqlx::FromRow, Debug)]
#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(transparent)]
pub struct Json(pub serde_json::Value);

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

/// A wrapper around sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>
#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(transparent)]
// pub struct DateTime(pub sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>);
pub struct DateTime(pub sqlx::types::chrono::NaiveDateTime);
