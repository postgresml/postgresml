use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::{PgArguments, PgPool};
use sqlx::query::Query;
use sqlx::{FromRow, Postgres};
use std::borrow::Borrow;

use crate::types::Json;

#[derive(Clone, Debug)]
enum BindValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Json(Json),
}

#[derive(custom_derive, Clone, Debug)]
pub struct QueryRunner {
    pool: PgPool,
    query: String,
    bind_values: Vec<BindValue>,
}

#[custom_methods(execute, bind_string)]
impl QueryRunner {
    pub fn new(pool: PgPool, query: &str) -> Self {
        Self {
            pool,
            query: query.to_string(),
            bind_values: Vec::new(),
        }
    }

    pub async fn execute(self) -> anyhow::Result<Json> {
        let mut query = sqlx::query(&self.query);
        for bind_value in self.bind_values.into_iter() {
            query = match bind_value {
                BindValue::String(v) => query.bind(v),
                BindValue::Int(v) => query.bind(v),
                BindValue::Float(v) => query.bind(v),
                BindValue::Bool(v) => query.bind(v),
                BindValue::Json(v) => query.bind(v.0),
            };
        }
        let values = query.fetch_all(&self.pool).await?;
        let values = values
            .into_iter()
            .map(|v| Json::from_row(&v).expect("Error parsing row to Json").0)
            .collect::<Vec<_>>();
        let values = serde_json::Value::Array(values);
        Ok(Json(values))
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
}
