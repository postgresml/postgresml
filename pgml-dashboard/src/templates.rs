use sailfish::TemplateOnce;
use sqlx::postgres::types::PgMoney;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{Column, Executor, PgPool, Row, Statement, TypeInfo, ValueRef};

use std::collections::HashMap;

use crate::{models, Context};

#[derive(TemplateOnce)]
#[template(path = "projects.html")]
pub struct Projects {
    pub topic: String,
    pub projects: Vec<models::Project>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "notebooks.html")]
pub struct Notebooks {
    pub topic: String,
    pub notebooks: Vec<models::Notebook>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "notebook.html")]
pub struct Notebook {
    pub topic: String,
    pub notebook: models::Notebook,
    pub cells: Vec<models::Cell>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "cell.html")]
pub struct Cell {
    pub notebook: models::Notebook,
    pub cell: models::Cell,
    pub edit: bool,
    pub selected: bool,
    pub bust_cache: String,
}

#[derive(TemplateOnce)]
#[template(path = "undo.html")]
pub struct Undo {
    pub notebook: models::Notebook,
    pub cell: models::Cell,
    pub bust_cache: String,
}

#[derive(TemplateOnce, Default)]
#[template(path = "sql.html")]
pub struct Sql {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Sql {
    pub async fn new(pool: &PgPool, query: &str) -> anyhow::Result<Sql> {
        let prepared_stmt = pool.prepare(query).await?;
        let cols = prepared_stmt.columns();

        let mut columns = Vec::new();
        let mut rows = Vec::new();

        cols.iter().for_each(|c| columns.push(c.name().to_string()));

        let result = prepared_stmt.query().fetch_all(pool).await?;

        for row in result.iter() {
            let mut values = Vec::new();

            for (i, _) in cols.iter().enumerate() {
                let type_ = cols[i].type_info().name();

                let null_check = row.try_get_raw(i)?;

                if null_check.is_null() {
                    values.push("".to_string());
                    continue;
                }

                let value = match type_ {
                    "TEXT" | "VARCHAR" | "CHAR(N)" | "NAME" => {
                        let value: String = row.try_get(i)?;
                        value
                    }

                    "TEXT[]" | "VARCHAR[]" => {
                        let value: Vec<String> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "INT8" | "BIGINT" | "BIGSERIAL" => {
                        let value: i64 = row.try_get(i)?;
                        value.to_string()
                    }

                    "INT8[]" | "BIGINT[]" => {
                        let value: Vec<i64> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "INT" | "SERIAL" | "INT4" => {
                        let value: i32 = row.try_get(i)?;
                        value.to_string()
                    }

                    "INT[]" | "INT4[]" => {
                        let value: Vec<i32> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "INT2" | "SMALLINT" | "SMALLSERIAL" => {
                        let value: i16 = row.try_get(i)?;
                        value.to_string()
                    }

                    "INT2[]" | "SMALLINT[]" => {
                        let value: Vec<i16> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "DOUBLE PRECISION" | "FLOAT8" => {
                        let value: f64 = row.try_get(i)?;
                        value.to_string()
                    }

                    "DOUBLE PRECISION[]" | "FLOAT8[]" => {
                        let value: Vec<f64> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "FLOAT4" | "REAL" => {
                        let value: f32 = row.try_get(i)?;
                        value.to_string()
                    }

                    "FLOAT4[]" | "REAL[]" => {
                        let value: Vec<f32> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "BYTEA" => {
                        // let value: Vec<u8> = row.try_get(i)?;
                        "<binary>".to_string()
                    }

                    "BOOL" => {
                        let value: bool = row.try_get(i)?;
                        value.to_string()
                    }

                    "NUMERIC" => {
                        let value: bigdecimal::BigDecimal = row.try_get(i)?;
                        value.to_string()
                    }

                    "TIMESTAMP" => {
                        let value: PrimitiveDateTime = row.try_get(i)?;
                        let (hour, minute, second, milli) = value.as_hms_milli();
                        let (year, month, day) = value.to_calendar_date();

                        format!(
                            "{}-{}-{} {}:{}:{}.{}",
                            year, month, day, hour, minute, second, milli
                        )
                    }

                    "MONEY" => {
                        let value: PgMoney = row.try_get(i)?;
                        value.to_bigdecimal(2).to_string()
                    }

                    "RECORD" => "OK".to_string(),

                    "JSON" | "JSONB" => {
                        let value: serde_json::Value = row.try_get(i)?;
                        serde_json::to_string(&value)?
                    }

                    unknown => {
                        // TODO
                        // Implement everything here: https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html
                        return Err(anyhow::anyhow!("Unsupported type: {}", unknown));
                    }
                };

                values.push(value);
            }

            rows.push(values);
        }

        Ok(Sql { columns, rows })
    }
}

#[derive(TemplateOnce)]
#[template(path = "sql_error.html")]
pub struct SqlError {
    pub error: String,
}

#[derive(TemplateOnce)]
#[template(path = "models.html")]
pub struct Models {
    pub topic: String,
    pub projects: Vec<models::Project>,
    pub models: HashMap<i64, Vec<models::Model>>,
    pub context: Context,
    // pub min_scores: HashMap<i64, f64>,
    // pub max_scores: HashMap<i64, f64>,
}

#[derive(TemplateOnce)]
#[template(path = "model.html")]
pub struct Model {
    pub topic: String,
    pub model: models::Model,
    pub project: models::Project,
    pub snapshot: models::Snapshot,
    pub deployed: bool,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "snapshots.html")]
pub struct Snapshots {
    pub topic: String,
    pub snapshots: Vec<models::Snapshot>,
    pub table_sizes: HashMap<i64, String>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "snapshot.html")]
pub struct Snapshot {
    pub topic: String,
    pub snapshot: models::Snapshot,
    pub models: Vec<models::Model>,
    pub projects: HashMap<i64, models::Project>,
    pub table_size: String,
    pub samples: HashMap<String, Vec<f32>>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "deployments.html")]
pub struct Deployments {
    pub topic: String,
    pub projects: Vec<models::Project>,
    pub deployments: HashMap<i64, Vec<models::Deployment>>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "deployment.html")]
pub struct Deployment {
    pub topic: String,
    pub project: models::Project,
    pub model: models::Model,
    pub deployment: models::Deployment,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "project.html")]
pub struct Project {
    pub topic: String,
    pub project: models::Project,
    pub models: Vec<models::Model>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "uploader.html")]
pub struct Uploader {
    pub topic: String,
    pub error: Option<String>,
    pub context: Context,
}

#[derive(TemplateOnce)]
#[template(path = "uploaded.html")]
pub struct Uploaded {
    pub topic: String,
    pub sql: Sql,
    pub columns: Vec<String>,
    pub table_name: String,
    pub context: Context,
}
