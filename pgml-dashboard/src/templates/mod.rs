use std::collections::HashMap;

use sailfish::TemplateOnce;
use sqlx::postgres::types::PgMoney;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{Column, Executor, PgPool, Row, Statement, TypeInfo, ValueRef};

use crate::models;

pub mod docs;
pub mod head;
pub mod components;

pub use head::*;

#[derive(TemplateOnce, Default)]
#[template(path = "content/not_found.html")]
pub struct NotFound {}

#[derive(TemplateOnce, Default)]
#[template(path = "content/error.html")]
pub struct Error {
    pub error: String,
}

#[derive(TemplateOnce, Clone, Default)]
#[template(path = "layout/base.html")]
pub struct Layout {
    pub head: Head,
    pub content: Option<String>,
    pub user: Option<models::User>,
    pub nav_title: Option<String>,
    pub nav_links: Vec<docs::NavLink>,
    pub toc_links: Vec<docs::TocLink>,
}

impl Layout {
    pub fn new(title: &str) -> Self {
        Layout {
            head: Head::new().title(title),
            ..Default::default()
        }
    }

    pub fn description(&mut self, description: &str) -> &mut Self {
        self.head.description = Some(description.to_owned());
        self
    }

    pub fn image(&mut self, image: &str) -> &mut Self {
        self.head.image = Some(image.to_owned());
        self
    }

    pub fn content(&mut self, content: &str) -> &mut Self {
        self.content = Some(content.to_owned());
        self
    }

    pub fn user(&mut self, user: &models::User) -> &mut Self {
        self.user = Some(user.to_owned());
        self
    }

    pub fn nav_title(&mut self, nav_title: &str) -> &mut Self {
        self.nav_title = Some(nav_title.to_owned());
        self
    }

    pub fn nav_links(&mut self, nav_links: &[docs::NavLink]) -> &mut Self {
        self.nav_links = nav_links.to_vec();
        self
    }

    pub fn toc_links(&mut self, toc_links: &[docs::TocLink]) -> &mut Self {
        self.toc_links = toc_links.to_vec();
        self
    }

    pub fn render<T>(&mut self, template: T) -> String
    where T : sailfish::TemplateOnce {
        self.content = Some(template.render_once().unwrap());
        (*self).clone().into()
    }
}

impl From<Layout> for String
{
    fn from(layout: Layout) -> String {
        layout.render_once().unwrap()
    }
}

#[derive(TemplateOnce)]
#[template(path = "content/article.html")]
pub struct Article {
    pub content: String,
}

#[derive(TemplateOnce)]
#[template(path = "content/projects.html")]
pub struct Projects {
    pub projects: Vec<models::Project>,
}

#[derive(TemplateOnce)]
#[template(path = "content/notebooks.html")]
pub struct Notebooks {
    pub notebooks: Vec<models::Notebook>,
}

#[derive(TemplateOnce)]
#[template(path = "content/notebook.html")]
pub struct Notebook {
    pub notebook: models::Notebook,
    pub cells: Vec<models::Cell>,
}

#[derive(TemplateOnce)]
#[template(path = "content/cell.html")]
pub struct Cell {
    pub notebook: models::Notebook,
    pub cell: models::Cell,
    pub edit: bool,
    pub selected: bool,
    pub bust_cache: String,
}

#[derive(TemplateOnce)]
#[template(path = "content/undo.html")]
pub struct Undo {
    pub notebook: models::Notebook,
    pub cell: models::Cell,
    pub bust_cache: String,
}

#[derive(TemplateOnce, Default)]
#[template(path = "content/sql.html")]
pub struct Sql {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_duration: std::time::Duration,
    pub render_execution_duration: bool,
}

impl Sql {
    pub async fn new(pool: &PgPool, query: &str, render_execution_duration: bool) -> anyhow::Result<Sql> {
        let prepared_stmt = pool.prepare(query).await?;
        let cols = prepared_stmt.columns();

        let mut columns = Vec::new();
        let mut rows = Vec::new();

        cols.iter().for_each(|c| columns.push(c.name().to_string()));

        let now = std::time::Instant::now();
        let result = prepared_stmt.query().fetch_all(pool).await?;
        let execution_duration = now.elapsed();

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

                    "vector" => {
                        let value: pgvector::Vector = row.try_get(i)?;
                        format!("{:?}", value.to_vec())
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

        Ok(Sql { columns, rows, execution_duration, render_execution_duration })
    }
}

#[derive(TemplateOnce)]
#[template(path = "content/sql_error.html")]
pub struct SqlError {
    pub error: String,
}

#[derive(TemplateOnce)]
#[template(path = "content/models.html")]
pub struct Models {
    pub projects: Vec<models::Project>,
    pub models: HashMap<i64, Vec<models::Model>>,
    // pub min_scores: HashMap<i64, f64>,
    // pub max_scores: HashMap<i64, f64>,
}

#[derive(TemplateOnce)]
#[template(path = "content/model.html")]
pub struct Model {
    pub model: models::Model,
    pub project: models::Project,
    pub snapshot: models::Snapshot,
    pub deployed: bool,
}

#[derive(TemplateOnce)]
#[template(path = "content/snapshots.html")]
pub struct Snapshots {
    pub snapshots: Vec<models::Snapshot>,
}

#[derive(TemplateOnce)]
#[template(path = "content/snapshot.html")]
pub struct Snapshot {
    pub snapshot: models::Snapshot,
    pub models: Vec<models::Model>,
    pub projects: HashMap<i64, models::Project>,
    pub samples: HashMap<String, Vec<f32>>,
}

#[derive(TemplateOnce)]
#[template(path = "content/deployments.html")]
pub struct Deployments {
    pub projects: Vec<models::Project>,
    pub deployments: HashMap<i64, Vec<models::Deployment>>,
}

#[derive(TemplateOnce)]
#[template(path = "content/deployment.html")]
pub struct Deployment {
    pub project: models::Project,
    pub model: models::Model,
    pub deployment: models::Deployment,
}

#[derive(TemplateOnce)]
#[template(path = "content/project.html")]
pub struct Project {
    pub project: models::Project,
    pub models: Vec<models::Model>,
}

#[derive(TemplateOnce)]
#[template(path = "content/uploader.html")]
pub struct Uploader {
    pub error: Option<String>,
}

#[derive(TemplateOnce)]
#[template(path = "content/uploaded.html")]
pub struct Uploaded {
    pub sql: Sql,
    pub columns: Vec<String>,
    pub table_name: String,
}
