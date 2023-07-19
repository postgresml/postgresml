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

/// A wrapper around sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>
#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(transparent)]
pub struct DateTime(pub sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>);
