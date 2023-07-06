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

impl<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> for Json {
    fn from_row(row: &'a sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        // let v: serde_json::Value = row.try_get(0)?;
        let v: serde_json::Value = serde_json::Value::Bool(true);
        Ok(Self(v))
    }
}

/// A wrapper around sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>
#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct DateTime(pub sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>);
