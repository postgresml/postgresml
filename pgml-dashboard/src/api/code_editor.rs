use crate::components::code_editor::Editor;
use crate::components::turbo::TurboFrame;
use anyhow::Context;
use once_cell::sync::OnceCell;
use sailfish::TemplateOnce;
use serde::Serialize;
use sqlparser::dialect::PostgreSqlDialect;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool, Row};

use crate::responses::ResponseOk;

static READONLY_POOL: OnceCell<PgPool> = OnceCell::new();
static ERROR: &str =
    "Thanks for trying PostgresML! If you would like to run more queries, sign up for an account and create a database.";

fn get_readonly_pool() -> PgPool {
    READONLY_POOL
        .get_or_init(|| {
            PgPoolOptions::new()
                .max_connections(1)
                .idle_timeout(std::time::Duration::from_millis(60_000))
                .max_lifetime(std::time::Duration::from_millis(60_000))
                .connect_lazy(&std::env::var("EDITOR_DATABASE_URL").expect("EDITOR_DATABASE_URL not set"))
                .expect("could not build lazy database connection")
        })
        .clone()
}

fn check_query(query: &str) -> anyhow::Result<()> {
    let ast = sqlparser::parser::Parser::parse_sql(&PostgreSqlDialect {}, query)?;

    if ast.len() != 1 {
        anyhow::bail!(ERROR);
    }

    let query = ast
        .into_iter()
        .next()
        .with_context(|| "impossible, ast is empty, even though we checked")?;

    match query {
        sqlparser::ast::Statement::Query(query) => match *query.body {
            sqlparser::ast::SetExpr::Select(_) => (),
            _ => anyhow::bail!(ERROR),
        },
        _ => anyhow::bail!(ERROR),
    };

    Ok(())
}

#[derive(FromForm, Debug)]
pub struct PlayForm {
    pub query: String,
}

pub async fn play(sql: &str) -> anyhow::Result<String> {
    check_query(sql)?;
    let pool = get_readonly_pool();
    let row = sqlx::query(sql).fetch_one(&pool).await?;
    let transform: serde_json::Value = row.try_get(0)?;
    Ok(serde_json::to_string_pretty(&transform)?)
}

/// Response expected by the frontend.
#[derive(Serialize)]
struct StreamResponse {
    error: Option<String>,
    result: Option<String>,
}

impl StreamResponse {
    fn from_error(error: &str) -> Self {
        StreamResponse {
            error: Some(error.to_string()),
            result: None,
        }
    }

    fn from_result(result: &str) -> Self {
        StreamResponse {
            error: None,
            result: Some(result.to_string()),
        }
    }
}

impl ToString for StreamResponse {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

/// An async iterator over a PostgreSQL cursor.
#[derive(Debug)]
struct AsyncResult<'a> {
    /// Open transaction.
    transaction: sqlx::Transaction<'a, sqlx::Postgres>,
    cursor_name: String,
}

impl<'a> AsyncResult<'a> {
    async fn from_message(message: ws::Message) -> anyhow::Result<Self> {
        if let ws::Message::Text(query) = message {
            let request = serde_json::from_str::<serde_json::Value>(&query)?;
            let query = request["sql"]
                .as_str()
                .context("Error sql key is required in websocket")?;
            Self::new(&query).await
        } else {
            anyhow::bail!(ERROR)
        }
    }

    /// Create new AsyncResult given a query.
    async fn new(query: &str) -> anyhow::Result<Self> {
        let cursor_name = format!(r#""{}""#, crate::utils::random_string(12));

        // Make sure it's a SELECT. Can't do too much damage there.
        check_query(query)?;

        let pool = get_readonly_pool();
        let mut transaction = pool.begin().await?;

        let query = format!("DECLARE {} CURSOR FOR {}", cursor_name, query);

        info!(
            "[stream] query: {}",
            query.trim().split("\n").collect::<Vec<&str>>().join(" ")
        );

        match transaction.execute(query.as_str()).await {
            Ok(_) => (),
            Err(err) => {
                info!("[stream] query error: {:?}", err);
                anyhow::bail!(err);
            }
        }

        Ok(AsyncResult {
            transaction,
            cursor_name,
        })
    }

    /// Fetch a row from the cursor, get the first column,
    /// decode the value and return it as a String.
    async fn next(&mut self) -> anyhow::Result<Option<String>> {
        use serde_json::Value;

        let result = sqlx::query(format!("FETCH 1 FROM {}", self.cursor_name).as_str())
            .fetch_optional(&mut *self.transaction)
            .await?;

        if let Some(row) = result {
            let _column = row.columns().get(0).with_context(|| "no columns")?;

            // Handle pgml.embed() which returns an array of floating points.
            if let Ok(value) = row.try_get::<Vec<f32>, _>(0) {
                return Ok(Some(serde_json::to_string(&value)?));
            }

            // Anything that just returns a String, e.g. pgml.version().
            if let Ok(value) = row.try_get::<String, _>(0) {
                return Ok(Some(value));
            }

            // Array of strings.
            if let Ok(value) = row.try_get::<Vec<String>, _>(0) {
                return Ok(Some(value.join("")));
            }

            // Integers.
            if let Ok(value) = row.try_get::<i64, _>(0) {
                return Ok(Some(value.to_string()));
            }

            if let Ok(value) = row.try_get::<i32, _>(0) {
                return Ok(Some(value.to_string()));
            }

            if let Ok(value) = row.try_get::<f64, _>(0) {
                return Ok(Some(value.to_string()));
            }

            if let Ok(value) = row.try_get::<f32, _>(0) {
                return Ok(Some(value.to_string()));
            }

            // Handle functions that return JSONB,
            // e.g. pgml.transform()
            if let Ok(value) = row.try_get::<Value, _>(0) {
                return Ok(Some(match value {
                    Value::Array(ref values) => {
                        let first_value = values.first();
                        match first_value {
                            Some(Value::Object(_)) => serde_json::to_string(&value)?,
                            _ => values
                                .into_iter()
                                .map(|v| v.as_str().unwrap_or("").to_string())
                                .collect::<Vec<String>>()
                                .join(""),
                        }
                    }

                    value => serde_json::to_string(&value)?,
                }));
            }
        }

        Ok(None)
    }

    async fn close(mut self) -> anyhow::Result<()> {
        self.transaction
            .execute(format!("CLOSE {}", self.cursor_name).as_str())
            .await?;
        self.transaction.rollback().await?;
        Ok(())
    }
}

#[get("/code_editor/play/stream")]
pub async fn play_stream(ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws =>
        for await message in ws {
            let message = match message {
                Ok(message) => message,
                Err(_err) => continue,
            };

            let mut got_something = false;
            match AsyncResult::from_message(message).await {
                Ok(mut result) => {
                    loop {
                        match result.next().await {
                            Ok(Some(result)) => {
                                got_something = true;
                                yield ws::Message::from(StreamResponse::from_result(&result).to_string());
                            }

                            Err(err) => {
                                yield ws::Message::from(StreamResponse::from_error(&err.to_string()).to_string());
                                break;
                            }

                            Ok(None) => {
                                if !got_something {
                                    yield ws::Message::from(StreamResponse::from_error(ERROR).to_string());
                                }
                                break;
                            }
                        }
                    };

                    match result.close().await {
                        Ok(_) => (),
                        Err(err) => {
                            info!("[stream] error closing: {:?}", err);
                        }
                    };
                }

                Err(err) => {
                    yield ws::Message::from(StreamResponse::from_error(&err.to_string()).to_string());
                }
            }
        };
    }
}

#[get("/code_editor/embed?<id>")]
pub fn embed_editor(id: String) -> ResponseOk {
    let comp = Editor::new();

    let rsp = TurboFrame::new().set_target_id(&id).set_content(comp.into());

    return ResponseOk(rsp.render_once().unwrap());
}

pub fn routes() -> Vec<Route> {
    routes![play_stream, embed_editor,]
}
