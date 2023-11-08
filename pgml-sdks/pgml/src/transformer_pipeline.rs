use futures::Stream;
use rust_bridge::{alias, alias_manual, alias_methods};
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

use crate::{get_or_initialize_pool, types::Json};

#[cfg(feature = "python")]
use crate::types::JsonPython;

#[derive(alias_manual)]
pub struct TransformerStream {
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
    type Item = Result<String, sqlx::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.done {
            if let Some(c) = self.commit.as_mut() {
                if let Poll::Ready(_) = c.as_mut().poll(cx) {
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
                        sqlx::query(&s.query).fetch_all(s.transaction.as_mut().unwrap()),
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
                            sqlx::query(&s.query).fetch_all(s.transaction.as_mut().unwrap()),
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
            Poll::Ready(Some(Ok(r.get::<String, _>(0))))
        } else if self.done {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

#[alias_methods(new, transform, transform_stream)]
impl TransformerPipeline {
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

    #[instrument(skip(self))]
    pub async fn transform(&self, inputs: Vec<String>, args: Option<Json>) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let args = args.unwrap_or_default();

        let results = sqlx::query("SELECT pgml.transform(task => $1, inputs => $2, args => $3)")
            .bind(&self.task)
            .bind(inputs)
            .bind(&args)
            .fetch_all(&pool)
            .await?;
        let results = results.get(0).unwrap().get::<serde_json::Value, _>(0);
        Ok(Json(results))
    }

    #[instrument(skip(self))]
    pub async fn transform_stream(
        &self,
        input: &str,
        args: Option<Json>,
        batch_size: Option<i32>,
    ) -> anyhow::Result<TransformerStream> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let args = args.unwrap_or_default();
        let batch_size = batch_size.unwrap_or(10);

        let mut transaction = pool.begin().await?;
        sqlx::query(
            "DECLARE c CURSOR FOR SELECT pgml.transform_stream(task => $1, input => $2, args => $3)",
        )
        .bind(&self.task)
        .bind(input)
        .bind(&args)
        .execute(&mut *transaction)
        .await?;

        Ok(TransformerStream::new(transaction, batch_size))
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
                    "How are you doing today?".to_string(),
                    "What is a good song?".to_string(),
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
                    "How are you doing today?".to_string(),
                    "What is a good song?".to_string(),
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
            Some("TheBloke/zephyr-7B-beta-GGUF".to_string()),
            Some(
                serde_json::json!({
                  "model_file": "zephyr-7b-beta.Q5_K_M.gguf", "model_type": "mistral"
                })
                .into(),
            ),
            None,
        );
        let mut stream = t
            .transform_stream(
                "AI is going to",
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
