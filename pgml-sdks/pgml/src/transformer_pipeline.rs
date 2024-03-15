use anyhow::Context;
use futures::Stream;
use rust_bridge::{alias, alias_methods};
use sqlx::{postgres::PgRow, Row};
use sqlx::{Postgres, Transaction};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;
use tracing::instrument;

/// Provides access to builtin database methods
#[derive(alias, Debug, Clone)]
pub struct TransformerPipeline {
    task: Json,
    database_url: Option<String>,
}

use crate::types::GeneralJsonAsyncIterator;
use crate::{get_or_initialize_pool, types::Json};

#[cfg(feature = "python")]
use crate::types::{GeneralJsonAsyncIteratorPython, JsonPython};

#[allow(clippy::type_complexity)]
struct TransformerStream {
    transaction: Option<Transaction<'static, Postgres>>,
    future: Option<Pin<Box<dyn Future<Output = Result<Vec<PgRow>, sqlx::Error>> + Send + 'static>>>,
    commit: Option<Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'static>>>,
    done: bool,
    query: String,
    db_batch_size: i32,
    results: VecDeque<PgRow>,
}

impl std::fmt::Debug for TransformerStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransformerStream").finish()
    }
}

impl TransformerStream {
    fn new(transaction: Transaction<'static, Postgres>, db_batch_size: i32) -> Self {
        let query = format!("FETCH {} FROM c", db_batch_size);
        Self {
            transaction: Some(transaction),
            future: None,
            commit: None,
            done: false,
            query,
            db_batch_size,
            results: VecDeque::new(),
        }
    }
}

impl Stream for TransformerStream {
    type Item = anyhow::Result<Json>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.done {
            if let Some(c) = self.commit.as_mut() {
                if c.as_mut().poll(cx).is_ready() {
                    self.commit = None;
                }
            }
        } else {
            if self.future.is_none() {
                unsafe {
                    let s = self.as_mut().get_unchecked_mut();
                    let s: *mut Self = s;
                    let s = Box::leak(Box::from_raw(s));
                    s.future = Some(Box::pin(
                        sqlx::query(&s.query).fetch_all(&mut **s.transaction.as_mut().unwrap()),
                    ));
                }
            }

            if let Poll::Ready(o) = self.as_mut().future.as_mut().unwrap().as_mut().poll(cx) {
                let rows = o?;
                if rows.len() < self.db_batch_size as usize {
                    self.done = true;
                    unsafe {
                        let s = self.as_mut().get_unchecked_mut();
                        let transaction = std::mem::take(&mut s.transaction).unwrap();
                        s.commit = Some(Box::pin(transaction.commit()));
                    }
                } else {
                    unsafe {
                        let s = self.as_mut().get_unchecked_mut();
                        let s: *mut Self = s;
                        let s = Box::leak(Box::from_raw(s));
                        s.future = Some(Box::pin(
                            sqlx::query(&s.query).fetch_all(&mut **s.transaction.as_mut().unwrap()),
                        ));
                    }
                }
                for r in rows.into_iter() {
                    self.results.push_back(r)
                }
            }
        }

        if !self.results.is_empty() {
            let r = self.results.pop_front().unwrap();
            Poll::Ready(Some(Ok(r.get::<Json, _>(0))))
        } else if self.done {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

#[alias_methods(new, transform, transform_stream)]
impl TransformerPipeline {
    /// Creates a new [TransformerPipeline]
    ///
    /// # Arguments
    /// * `task` - The task to run
    /// * `model` - The model to use
    /// * `args` - The arguments to pass to the task
    /// * `database_url` - The database url to use. If None, the `PGML_DATABASE_URL` environment variable will be used
    pub fn new(
        task: &str,
        model: Option<String>,
        args: Option<Json>,
        database_url: Option<String>,
    ) -> Self {
        let mut args = args.unwrap_or_default();
        let a = args.as_object_mut().expect("args must be an object");
        a.insert("task".to_string(), task.to_string().into());
        if let Some(m) = model {
            a.insert("model".to_string(), m.into());
        }
        // We must convert any floating point values to integers or our extension will get angry
        if let Some(v) = a.remove("gpu_layers") {
            let int_v = v.as_f64().expect("gpu_layers must be an integer") as i64;
            a.insert("gpu_layers".to_string(), int_v.into());
        }

        Self {
            task: args,
            database_url,
        }
    }

    /// Calls transform
    ///
    /// # Arguments
    /// * `inputs` - The inputs to the task
    /// * `args` - The arguments to pass to the task
    #[instrument(skip(self))]
    pub async fn transform(&self, inputs: Vec<Json>, args: Option<Json>) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let args = args.unwrap_or_default();

        // We set the task in the new constructor so we can unwrap here
        let results = if self.task["task"].as_str().unwrap() == "conversational" {
            let inputs: Vec<serde_json::Value> = inputs.into_iter().map(|j| j.0).collect();
            sqlx::query("SELECT pgml.transform(task => $1, inputs => $2, args => $3)")
                .bind(&self.task)
                .bind(inputs)
                .bind(&args)
                .fetch_all(&pool)
                .await?
        } else {
            let inputs: anyhow::Result<Vec<String>> =
                inputs
                    .into_iter()
                    .map(|input| {
                        input.as_str().context(
                        "the inputs arg must be strings when not using the conversational task",
                    ).map(|s| s.to_string())
                    })
                    .collect();
            sqlx::query("SELECT pgml.transform(task => $1, inputs => $2, args => $3)")
                .bind(&self.task)
                .bind(inputs?)
                .bind(&args)
                .fetch_all(&pool)
                .await?
        };
        let results = results.get(0).unwrap().get::<serde_json::Value, _>(0);
        Ok(Json(results))
    }

    /// Calls transform
    /// The same as transformer but it returns an iterator
    /// The `batch_size` argument can be used to control the number of results returned in each batch
    #[instrument(skip(self))]
    pub async fn transform_stream(
        &self,
        input: Json,
        args: Option<Json>,
        batch_size: Option<i32>,
    ) -> anyhow::Result<GeneralJsonAsyncIterator> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let args = args.unwrap_or_default();
        let batch_size = batch_size.unwrap_or(10);

        let mut transaction = pool.begin().await?;
        // We set the task in the new constructor so we can unwrap here
        if self.task["task"].as_str().unwrap() == "conversational" {
            let inputs = input
                .as_array()
                .context("`input` to transformer_stream must be an array of objects")?
                .to_vec();
            sqlx::query(
                "DECLARE c CURSOR FOR SELECT pgml.transform_stream(task => $1, inputs => $2, args => $3)",
            )
            .bind(&self.task)
            .bind(inputs)
            .bind(&args)
            .execute(&mut *transaction)
            .await?;
        } else {
            let input = input
                .as_str()
                .context(
                    "`input` to transformer_stream must be a string if task is not conversational",
                )?
                .to_string();
            sqlx::query(
                "DECLARE c CURSOR FOR SELECT pgml.transform_stream(task => $1, input => $2, args => $3)",
            )
            .bind(&self.task)
            .bind(input)
            .bind(&args)
            .execute(&mut *transaction)
            .await?;
        }

        Ok(GeneralJsonAsyncIterator(Box::pin(TransformerStream::new(
            transaction,
            batch_size,
        ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_init_logger;
    use futures::StreamExt;

    #[sqlx::test]
    async fn transformer_pipeline_can_transform() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let t = TransformerPipeline::new(
            "translation_en_to_fr",
            Some("t5-base".to_string()),
            None,
            None,
        );
        let results = t
            .transform(
                vec![
                    serde_json::Value::String("How are you doing today?".to_string()).into(),
                    serde_json::Value::String("How are you doing today?".to_string()).into(),
                ],
                None,
            )
            .await?;
        assert!(results.as_array().is_some());
        Ok(())
    }

    #[sqlx::test]
    async fn transformer_pipeline_can_transform_with_default_model() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let t = TransformerPipeline::new("translation_en_to_fr", None, None, None);
        let results = t
            .transform(
                vec![
                    serde_json::Value::String("How are you doing today?".to_string()).into(),
                    serde_json::Value::String("How are you doing today?".to_string()).into(),
                ],
                None,
            )
            .await?;
        assert!(results.as_array().is_some());
        Ok(())
    }

    #[sqlx::test]
    async fn transformer_can_transform_stream() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let t = TransformerPipeline::new(
            "text-generation",
            Some("TheBloke/zephyr-7B-beta-GPTQ".to_string()),
            Some(
                serde_json::json!({
                  "model_type": "mistral", "revision": "main", "device_map": "auto"
                })
                .into(),
            ),
            None,
        );
        let mut stream = t
            .transform_stream(
                serde_json::json!("AI is going to").into(),
                Some(
                    serde_json::json!({
                        "max_new_tokens": 10
                    })
                    .into(),
                ),
                None,
            )
            .await?;
        while let Some(o) = stream.next().await {
            o?;
        }
        Ok(())
    }
}
