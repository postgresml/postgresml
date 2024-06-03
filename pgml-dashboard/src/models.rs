use std::collections::HashMap;

use comrak::{markdown_to_html, ComrakExtensionOptions, ComrakOptions};
use csv_async::AsyncReaderBuilder;
use pgml_components::Component;
use sailfish::TemplateOnce;
use sqlx::postgres::types::PgInterval;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{FromRow, PgPool, Row};
use tokio::io::{AsyncBufReadExt, AsyncSeekExt};

use crate::templates;

#[derive(FromRow, Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub task: Option<String>,
    pub created_at: PrimitiveDateTime,
}

impl Project {
    pub async fn get_by_id(pool: impl sqlx::PgExecutor<'_>, id: i64) -> anyhow::Result<Project> {
        Ok(sqlx::query_as!(
            Project,
            "SELECT
                    id,
                    name,
                    task::text,
                    created_at
                FROM pgml.projects
                WHERE id = $1",
            id,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn all(pool: &PgPool) -> anyhow::Result<Vec<Project>> {
        Ok(sqlx::query_as!(
            Project,
            "SELECT
                    id,
                    name,
                    task::TEXT,
                    created_at
                FROM pgml.projects
                WHERE task::text != 'embedding'
                ORDER BY id DESC"
        )
        .fetch_all(pool)
        .await?)
    }

    pub fn key_metric_name(&self) -> anyhow::Result<&'static str> {
        match self.task.as_ref().unwrap().as_str() {
            "classification" | "text_classification" | "question_answering" => Ok("f1"),
            "regression" => Ok("r2"),
            "clustering" => Ok("silhouette"),
            "decomposition" => Ok("cumulative_explained_variance"),
            "summarization" => Ok("rouge_ngram_f1"),
            "translation" => Ok("bleu"),
            "text_generation" | "text2text" => Ok("perplexity"),
            task => Err(anyhow::anyhow!("Unhandled task: {}", task)),
        }
    }

    pub fn key_metric_display_name(&self) -> anyhow::Result<&'static str> {
        match self.task.as_ref().unwrap().as_str() {
            "classification" | "text_classification" | "question_answering" => Ok("F<sup>1</sup>"),
            "regression" => Ok("R<sup>2</sup>"),
            "clustering" => Ok("silhouette"),
            "decomposition" => Ok("Cumulative Explained Variance"),
            "summarization" => Ok("Rouge Ngram F<sup>1</sup>"),
            "translation" => Ok("Bleu"),
            "text_generation" | "text2text" => Ok("Perplexity"),
            task => Err(anyhow::anyhow!("Unhandled task: {}", task)),
        }
    }
}

#[derive(FromRow, Debug, Clone)]
pub struct Notebook {
    pub id: i64,
    pub name: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

impl Notebook {
    pub async fn get_by_id(pool: &PgPool, id: i64) -> anyhow::Result<Notebook> {
        Ok(
            sqlx::query_as!(Notebook, "SELECT * FROM pgml.notebooks WHERE id = $1", id,)
                .fetch_one(pool)
                .await?,
        )
    }

    pub async fn create(pool: &PgPool, name: &str) -> anyhow::Result<Notebook> {
        Ok(sqlx::query_as!(
            Notebook,
            "INSERT INTO pgml.notebooks (name) VALUES ($1) RETURNING *",
            name,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn all(pool: &PgPool) -> anyhow::Result<Vec<Notebook>> {
        Ok(sqlx::query_as!(Notebook, "SELECT * FROM pgml.notebooks")
            .fetch_all(pool)
            .await?)
    }

    pub async fn cells(&self, pool: &PgPool) -> anyhow::Result<Vec<Cell>> {
        Ok(sqlx::query_as!(
            Cell,
            "SELECT * FROM pgml.notebook_cells
                WHERE notebook_id = $1
                AND deleted_at IS NULL
            ORDER BY cell_number",
            self.id,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn reset(&self, pool: &PgPool) -> anyhow::Result<()> {
        let _ = sqlx::query!(
            "UPDATE pgml.notebook_cells
                SET
                execution_time = NULL,
                rendering = NULL
            WHERE notebook_id = $1
            AND cell_type = $2",
            self.id,
            CellType::Sql as i32,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn created_by(&self) -> &'static str {
        if self.id <= 9 {
            "PostgresML"
        } else {
            "User"
        }
    }
}

#[derive(PartialEq)]
pub enum CellType {
    Sql = 3,
    Markdown = 1,
}

impl std::convert::From<i32> for CellType {
    fn from(value: i32) -> CellType {
        match value {
            1 => CellType::Markdown,
            3 => CellType::Sql,
            _ => todo!(),
        }
    }
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CellType::Sql => write!(f, "sql"),
            CellType::Markdown => write!(f, "markdown"),
        }
    }
}

#[derive(FromRow, Debug, Clone, Default)]
pub struct Cell {
    pub id: i64,
    pub notebook_id: i64,
    pub cell_type: i32,
    pub contents: String,
    pub rendering: Option<String>,
    pub execution_time: Option<PgInterval>,
    pub cell_number: i32,
    pub version: i32,
    pub deleted_at: Option<PrimitiveDateTime>,
}

impl Cell {
    pub async fn create(pool: &PgPool, notebook: &Notebook, cell_type: i32, contents: &str) -> anyhow::Result<Cell> {
        Ok(sqlx::query_as!(
            Cell,
            "
            WITH
                lock AS (
                    SELECT * FROM pgml.notebooks WHERE id = $1 FOR UPDATE
                ),
                max_cell AS (
                    SELECT COALESCE(MAX(cell_number), 0) AS cell_number
                    FROM pgml.notebook_cells
                    WHERE notebook_id = $1
                    AND deleted_at IS NULL
                )
            INSERT INTO pgml.notebook_cells
                (notebook_id, cell_type, contents, cell_number, version)
            VALUES
                ($1, $2, $3, (SELECT cell_number + 1 FROM max_cell), 1)
            RETURNING id,
                    notebook_id,
                    cell_type,
                    contents,
                    rendering,
                    execution_time,
                    cell_number,
                    version,
                    deleted_at",
            notebook.id,
            cell_type,
            contents,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn get_by_id(pool: impl sqlx::PgExecutor<'_>, id: i64) -> anyhow::Result<Cell> {
        Ok(sqlx::query_as!(
            Cell,
            "SELECT
                    id,
                    notebook_id,
                    cell_type,
                    contents,
                    rendering,
                    execution_time,
                    cell_number,
                    version,
                    deleted_at
                FROM pgml.notebook_cells
                WHERE id = $1
                ",
            id,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn update(&mut self, pool: &PgPool, cell_type: i32, contents: &str) -> anyhow::Result<()> {
        self.cell_type = cell_type;
        self.contents = contents.to_string();

        let _ = sqlx::query!(
            "UPDATE pgml.notebook_cells
            SET
                cell_type = $1,
                contents = $2,
                version = version + 1
            WHERE id = $3",
            cell_type,
            contents,
            self.id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &PgPool) -> anyhow::Result<Cell> {
        Ok(sqlx::query_as!(
            Cell,
            "UPDATE pgml.notebook_cells
            SET deleted_at = NOW()
            WHERE id = $1
            RETURNING id,
                    notebook_id,
                    cell_type,
                    contents,
                    rendering,
                    execution_time,
                    cell_number,
                    version,
                    deleted_at",
            self.id
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn reorder(self, pool: impl sqlx::PgExecutor<'_>, cell_number: i32) -> anyhow::Result<Cell> {
        Ok(sqlx::query_as!(
            Cell,
            "
            UPDATE pgml.notebook_cells
            SET cell_number = $1
            WHERE id = $2
            RETURNING *
        ",
            cell_number,
            self.id
        )
        .fetch_one(pool)
        .await?)
    }

    pub fn tag(&self) -> String {
        format!("/* pgml_dashboard_cell_id: {} */", self.id)
    }

    pub async fn cancel(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query(&format!(
            "SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE query LIKE '{}%'",
            self.tag(),
        ))
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn state(&self) -> &'static str {
        if self.contents.is_empty() {
            "new"
        } else if self.rendering.is_some() {
            "rendered"
        } else {
            "rendering"
        }
    }

    pub async fn render(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        let cell_type: CellType = self.cell_type.into();

        let (rendering, execution_time) = match cell_type {
            CellType::Sql => {
                let queries: Vec<&str> = self.contents.split(';').filter(|q| !q.trim().is_empty()).collect();
                let mut rendering = String::new();
                let mut total_execution_duration = std::time::Duration::default();

                for query in queries {
                    let query = self.tag() + query;
                    let result = match templates::Sql::new(pool, &query).await {
                        Ok(sql) => {
                            total_execution_duration += sql.execution_duration;
                            sql.render_once()?
                        }
                        Err(err) => templates::SqlError {
                            error: format!("{:?}", err),
                        }
                        .render_once()?,
                    };

                    rendering.push_str(&result);
                }

                let execution_time = PgInterval {
                    months: 0,
                    days: 0,
                    microseconds: total_execution_duration.as_micros().try_into().unwrap_or(0),
                };
                (rendering, Some(execution_time))
            }

            CellType::Markdown => {
                let options = ComrakOptions {
                    extension: ComrakExtensionOptions {
                        strikethrough: true,
                        tagfilter: true,
                        table: true,
                        autolink: true,
                        tasklist: true,
                        superscript: true,
                        header_ids: None,
                        footnotes: true,
                        description_lists: true,
                        front_matter_delimiter: None,
                    },
                    ..Default::default()
                };

                (
                    format!(
                        "<div class=\"markdown-body\">{}</div>",
                        markdown_to_html(&self.contents, &options)
                    ),
                    None,
                )
            }
        };

        sqlx::query!(
            "UPDATE pgml.notebook_cells SET rendering = $1, execution_time = $2 WHERE id = $3",
            rendering,
            execution_time,
            self.id
        )
        .execute(pool)
        .await?;

        self.execution_time = execution_time;
        self.rendering = Some(rendering);

        Ok(())
    }

    pub fn code(&self) -> bool {
        CellType::Sql == self.cell_type.into()
    }

    pub fn html(&self) -> Option<String> {
        self.rendering.clone()
    }

    pub fn cell_type_display(&self) -> String {
        let cell_type: CellType = self.cell_type.into();
        cell_type.to_string()
    }
}

#[derive(sqlx::Type, PartialEq, Debug)]
pub enum Runtime {
    Python,
    Rust,
}

#[derive(FromRow, Debug)]
#[allow(dead_code)]
pub struct Model {
    pub id: i64,
    pub project_id: i64,
    pub snapshot_id: Option<i64>,
    pub num_features: i32,
    pub algorithm: String,
    pub runtime: Option<String>,
    pub hyperparams: serde_json::Value,
    pub status: String,
    pub metrics: Option<serde_json::Value>,
    pub search: Option<String>,
    pub search_params: serde_json::Value,
    pub search_args: serde_json::Value,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

impl Model {
    pub async fn get_by_id(pool: &PgPool, id: i64) -> anyhow::Result<Model> {
        Ok(sqlx::query_as!(
            Model,
            "SELECT
                    id,
                    project_id,
                    snapshot_id,
                    num_features,
                    algorithm,
                    runtime::TEXT,
                    hyperparams,
                    status,
                    metrics,
                    search,
                    search_params,
                    search_args,
                    created_at,
                    updated_at
                FROM pgml.models
                WHERE id = $1
                ",
            id,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn get_by_project_id(pool: &PgPool, project_id: i64) -> anyhow::Result<Vec<Model>> {
        Ok(sqlx::query_as!(
            Model,
            "SELECT
                    id,
                    project_id,
                    snapshot_id,
                    num_features,
                    algorithm,
                    runtime::TEXT,
                    hyperparams,
                    status,
                    metrics,
                    search,
                    search_params,
                    search_args,
                    created_at,
                    updated_at
                FROM pgml.models
                WHERE project_id = $1
                ",
            project_id,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn get_by_snapshot_id(pool: &PgPool, snapshot_id: i64) -> anyhow::Result<Vec<Model>> {
        Ok(sqlx::query_as!(
            Model,
            "SELECT
                    id,
                    project_id,
                    snapshot_id,
                    num_features,
                    algorithm,
                    runtime::TEXT,
                    hyperparams,
                    status,
                    metrics,
                    search,
                    search_params,
                    search_args,
                    created_at,
                    updated_at
                FROM pgml.models
                WHERE snapshot_id = $1
                ",
            snapshot_id,
        )
        .fetch_all(pool)
        .await?)
    }

    pub fn metrics(&self) -> &serde_json::Map<String, serde_json::Value> {
        self.metrics.as_ref().unwrap().as_object().unwrap()
    }

    pub fn hyperparams(&self) -> &serde_json::Map<String, serde_json::Value> {
        self.hyperparams.as_object().unwrap()
    }

    pub fn search_params(&self) -> &serde_json::Map<String, serde_json::Value> {
        self.search_params.as_object().unwrap()
    }

    pub fn search_results(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        match self.metrics().get("search_results") {
            Some(value) => Some(value.as_object().unwrap()),
            None => None,
        }
    }

    pub fn key_metric(&self, project: &Project) -> anyhow::Result<f64> {
        let key_metric_name = project.key_metric_name()?;

        match self.metrics()[key_metric_name].as_f64() {
            Some(metric) => Ok(metric),
            None => Ok(0.),
        }
    }

    pub async fn deployed(&self, pool: &PgPool) -> anyhow::Result<bool> {
        let row = sqlx::query!(
            "SELECT
                (model_id = $1) AS deployed
            FROM pgml.deployments
            WHERE project_id = $2
            ORDER BY created_at DESC
            LIMIT 1",
            self.id,
            self.project_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(row.deployed.unwrap())
    }

    pub async fn project(&self, pool: &PgPool) -> anyhow::Result<Project> {
        Project::get_by_id(pool, self.project_id).await
    }
}

#[derive(FromRow, Debug)]
#[allow(dead_code)]
pub struct Snapshot {
    pub id: i64,
    pub relation_name: String,
    pub y_column_name: Option<Vec<String>>,
    pub test_size: f32,
    pub test_sampling: Option<String>,
    pub status: String,
    pub columns: Option<serde_json::Value>,
    pub analysis: Option<serde_json::Value>,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
    pub exists: bool,
    pub table_size: String,
}

impl Snapshot {
    pub async fn all(pool: &PgPool) -> anyhow::Result<Vec<Snapshot>> {
        Ok(sqlx::query_as!(
            Snapshot,
            "SELECT id,
                    relation_name,
                    y_column_name,
                    test_size,
                    test_sampling::TEXT,
                    status,
                    columns,
                    analysis,
                    created_at,
                    updated_at,
                    CASE 
                        WHEN EXISTS (
                                SELECT 1
                                FROM pg_class c
                                WHERE c.oid::regclass::text = relation_name
                            ) THEN pg_size_pretty(pg_total_relation_size(relation_name::regclass))
                        ELSE '0 Bytes'
                    END AS \"table_size!\", 
                    EXISTS (
                        SELECT 1
                        FROM pg_class c
                        WHERE c.oid::regclass::text = relation_name
                    ) AS \"exists!\"
                    FROM pgml.snapshots
                    "
        )
        .fetch_all(pool)
        .await?)
    }
    pub async fn get_by_id(pool: &PgPool, id: i64) -> anyhow::Result<Snapshot> {
        Ok(sqlx::query_as!(
            Snapshot,
            "SELECT id,
                    relation_name,
                    y_column_name,
                    test_size,
                    test_sampling::TEXT,
                    status,
                    columns,
                    analysis,
                    created_at,
                    updated_at,
                    CASE 
                        WHEN EXISTS (
                                SELECT 1
                                FROM pg_class c
                                WHERE c.oid::regclass::text = relation_name
                            ) THEN pg_size_pretty(pg_total_relation_size(relation_name::regclass))
                        ELSE '0 Bytes'
                    END AS \"table_size!\", 
                    EXISTS (
                        SELECT 1
                        FROM pg_class c
                        WHERE c.oid::regclass::text = relation_name
                    ) AS \"exists!\"
                    FROM pgml.snapshots WHERE id = $1",
            id,
        )
        .fetch_one(pool)
        .await?)
    }

    pub fn rows(&self) -> Option<i64> {
        match self.analysis.as_ref() {
            Some(analysis) => analysis.get("samples").map(|samples| samples.as_f64().unwrap() as i64),
            None => None,
        }
    }

    pub async fn samples(&self, pool: &PgPool, rows: i64) -> anyhow::Result<HashMap<String, Vec<f32>>> {
        let mut samples = HashMap::new();

        if self.exists {
            let rows = sqlx::query(&format!(
                "SELECT row_to_json(row) r
                FROM (SELECT * FROM {} LIMIT $1) row",
                self.relation_name
            ))
            .bind(rows)
            .fetch_all(pool)
            .await?;

            rows.iter().for_each(|row| {
                let r: serde_json::Value = row.try_get("r").unwrap();
                let obj = r.as_object().unwrap();

                for (key, value) in obj.iter() {
                    let rf = samples.entry(key.clone()).or_insert(Vec::new());
                    rf.push(value.as_f64().unwrap_or(0.) as f32);
                }
            });
        }

        Ok(samples)
    }

    pub fn feature_size(&self) -> Option<usize> {
        self.features().map(|features| features.len())
    }

    pub fn columns(&self) -> Option<Vec<&serde_json::Map<String, serde_json::Value>>> {
        match self.columns.as_ref() {
            Some(columns) => columns
                .as_array()
                .map(|columns| columns.iter().map(|column| column.as_object().unwrap()).collect()),

            None => None,
        }
    }

    pub fn features(&self) -> Option<Vec<&serde_json::Map<String, serde_json::Value>>> {
        match self.columns() {
            Some(columns) => {
                if self.y_column_name.is_none() {
                    return Some(columns.into_iter().collect());
                }

                Some(
                    columns
                        .into_iter()
                        .filter(|column| {
                            !self
                                .y_column_name
                                .as_ref()
                                .unwrap()
                                .contains(&column["name"].as_str().unwrap().to_string())
                        })
                        .collect(),
                )
            }
            None => None,
        }
    }

    pub fn labels(&self) -> Option<Vec<&serde_json::Map<String, serde_json::Value>>> {
        if self.y_column_name.is_none() {
            return Some(Vec::new());
        }

        self.columns().map(|columns| {
            columns
                .into_iter()
                .filter(|column| {
                    self.y_column_name
                        .as_ref()
                        .unwrap()
                        .contains(&column["name"].as_str().unwrap().to_string())
                })
                .collect()
        })
    }

    pub async fn models(&self, pool: &PgPool) -> anyhow::Result<Vec<Model>> {
        Model::get_by_snapshot_id(pool, self.id).await
    }

    pub fn target_stddev(&self, name: &str) -> f32 {
        match self
            .analysis
            .as_ref()
            .unwrap()
            .as_object()
            .unwrap()
            .get(&format!("{}_stddev", name))
        {
            // 2.1
            Some(value) => value.as_f64().unwrap() as f32,
            // 2.2+
            None => {
                let columns = self.columns().unwrap();
                let column = columns.iter().find(|column| column["name"].as_str().unwrap() == name);
                match column {
                    Some(column) => column
                        .get("statistics")
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .get("std_dev")
                        .unwrap()
                        .as_f64()
                        .unwrap() as f32,
                    None => 0.,
                }
            }
        }
    }
}

#[derive(FromRow)]
#[allow(dead_code)]
pub struct Deployment {
    pub id: i64,
    pub project_id: i64,
    pub model_id: i64,
    pub strategy: Option<String>,
    pub created_at: PrimitiveDateTime,
    pub active: Option<bool>,
}

impl Deployment {
    pub async fn get_by_project_id(pool: &PgPool, project_id: i64) -> anyhow::Result<Vec<Deployment>> {
        Ok(sqlx::query_as!(
            Deployment,
            "SELECT
                    a.id,
                    project_id,
                    model_id,
                    strategy::TEXT,
                    created_at,
                    a.id = last_deployment.id AS active
                FROM pgml.deployments a
                CROSS JOIN LATERAL (
                    SELECT id FROM pgml.deployments b
                    WHERE b.project_id = a.project_id
                    ORDER BY b.id DESC
                    LIMIT 1
                ) last_deployment
                WHERE project_id = $1
                ORDER BY a.id DESC",
            project_id,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn get_by_id(pool: &PgPool, id: i64) -> anyhow::Result<Deployment> {
        Ok(sqlx::query_as!(
            Deployment,
            "SELECT
                    a.id,
                    project_id,
                    model_id,
                    strategy::TEXT,
                    created_at,
                    a.id = last_deployment.id AS active
                FROM pgml.deployments a
                CROSS JOIN LATERAL (
                    SELECT id FROM pgml.deployments b
                    WHERE b.project_id = a.project_id
                    ORDER BY b.id DESC
                    LIMIT 1
                ) last_deployment
                WHERE a.id = $1
                ORDER BY a.id DESC",
            id,
        )
        .fetch_one(pool)
        .await?)
    }

    pub fn human_readable_strategy(&self) -> String {
        self.strategy.as_ref().unwrap().replace('_', " ")
    }
}

#[derive(FromRow)]
pub struct UploadedFile {
    pub id: i64,
    pub created_at: PrimitiveDateTime,
}

impl UploadedFile {
    pub fn table_name(&self) -> String {
        format!("data_{}", self.id)
    }

    pub async fn create(pool: &PgPool) -> anyhow::Result<UploadedFile> {
        Ok(sqlx::query_as!(
            UploadedFile,
            "INSERT INTO pgml.uploaded_files (id, created_at) VALUES (DEFAULT, DEFAULT)
                RETURNING id, created_at"
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn upload(&mut self, pool: &PgPool, file: &std::path::Path, headers: bool) -> anyhow::Result<()> {
        // Open the temp file.
        let mut reader = tokio::io::BufReader::new(tokio::fs::File::open(file).await?);

        // Let's create the column names for the table.
        let mut maybe_headers = String::new();
        reader.read_line(&mut maybe_headers).await?;

        let mut csv = AsyncReaderBuilder::new().create_reader(maybe_headers.as_bytes());

        let maybe_headers = csv.headers().await?;

        let column_names = maybe_headers
            .iter()
            .enumerate()
            .map(|(i, c)| {
                // You said we have headers right?
                if headers {
                    c.to_string()
                } else {
                    // Generate column names instead.
                    format!("column_{}", i).to_string()
                }
            })
            .collect::<Vec<String>>();

        // Create table.
        let columns = column_names
            .iter()
            .map(|c| format!("{} TEXT", c))
            .collect::<Vec<String>>()
            .join(",");

        let stmt = format!(
            "
            CREATE TABLE data_{} (
                {}
            );
        ",
            self.id, columns
        );

        sqlx::query(&stmt).execute(pool).await?;

        // COPY FROM STDIN
        let mut connection = pool.acquire().await?;

        let mut copy = match connection
            .copy_in_raw(&format!(
                "COPY data_{} FROM STDIN CSV {}",
                self.id,
                if headers { "HEADER" } else { "" }
            ))
            .await
        {
            Ok(copy) => copy,
            Err(err) => return Err(err.into()),
        };

        // If we have no readers, don't skip rows.
        if !headers {
            match reader.rewind().await {
                Ok(_) => (),
                Err(err) => {
                    copy.finish().await?;
                    return Err(err.into());
                }
            };
        }

        match copy.read_from(reader).await {
            Ok(_) => (),
            Err(err) => {
                copy.finish().await?;
                return Err(err.into());
            }
        };

        copy.finish().await?;

        Ok(())
    }
}

// Shared context models.
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
}

impl Default for User {
    fn default() -> User {
        User {
            id: -1,
            email: "".to_string(),
        }
    }
}

impl User {
    pub fn is_anonymous(&self) -> bool {
        self.id == 0
    }
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: i64,
    pub name: String,
    pub tier: Option<Component>,
    pub status: Option<Component>,
}

impl Default for Cluster {
    fn default() -> Cluster {
        Cluster {
            id: -1,
            name: "Local".to_string(),
            tier: None,
            status: None,
        }
    }
}
