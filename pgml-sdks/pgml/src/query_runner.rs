use sqlx::postgres::PgArguments;
use sqlx::query::Query;
use sqlx::{Postgres, Row};

use crate::{get_or_initialize_pool, types::Json};

#[cfg(feature = "python")]
use crate::types::JsonPython;

#[cfg(feature = "c")]
use crate::languages::c::JsonC;

#[cfg(feature = "rust_bridge")]
use rust_bridge::{alias, alias_methods};

#[derive(Clone, Debug)]
enum BindValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Json(Json),
}

#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[derive(Clone, Debug)]
pub struct QueryRunner {
    query: String,
    bind_values: Vec<BindValue>,
    database_url: Option<String>,
}

#[cfg_attr(
    feature = "rust_bridge",
    alias_methods(
        fetch_all,
        execute,
        bind_string,
        bind_int,
        bind_float,
        bind_bool,
        bind_json
    )
)]
impl QueryRunner {
    pub fn new(query: &str, database_url: Option<String>) -> Self {
        Self {
            query: query.to_string(),
            bind_values: Vec::new(),
            database_url,
        }
    }

    pub async fn fetch_all(mut self) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        self.query = format!("SELECT json_agg(j) FROM ({}) j", self.query);
        let query = self.build_query();
        let results = query.fetch_one(&pool).await?;
        let results = results.try_get::<serde_json::Value, _>(0);
        match results {
            Ok(r) => Ok(Json(r)),
            _ => Ok(Json(serde_json::json!([]))),
        }
    }

    pub async fn execute(self) -> anyhow::Result<()> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let query = self.build_query();
        query.execute(&pool).await?;
        Ok(())
    }

    pub fn bind_string(mut self, bind_value: String) -> Self {
        self.bind_values.push(BindValue::String(bind_value));
        self
    }

    pub fn bind_int(mut self, bind_value: i64) -> Self {
        self.bind_values.push(BindValue::Int(bind_value));
        self
    }

    pub fn bind_float(mut self, bind_value: f64) -> Self {
        self.bind_values.push(BindValue::Float(bind_value));
        self
    }

    pub fn bind_bool(mut self, bind_value: bool) -> Self {
        self.bind_values.push(BindValue::Bool(bind_value));
        self
    }

    pub fn bind_json(mut self, bind_value: Json) -> Self {
        self.bind_values.push(BindValue::Json(bind_value));
        self
    }

    fn build_query(&self) -> Query<Postgres, PgArguments> {
        let mut query = sqlx::query(&self.query);
        for bind_value in self.bind_values.iter() {
            query = match bind_value {
                BindValue::String(v) => query.bind(v),
                BindValue::Int(v) => query.bind(v),
                BindValue::Float(v) => query.bind(v),
                BindValue::Bool(v) => query.bind(v),
                BindValue::Json(v) => query.bind(&v.0),
            };
        }
        query
    }
}
