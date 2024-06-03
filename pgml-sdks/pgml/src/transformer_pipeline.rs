use anyhow::Context;
use sqlx::Row;
use tracing::instrument;

#[cfg(feature = "rust_bridge")]
use rust_bridge::{alias, alias_methods};

/// Provides access to builtin database methods
#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[derive(Debug, Clone)]
pub struct TransformerPipeline {
    task: Json,
    database_url: Option<String>,
}

use crate::types::{CustomU64Convertor, GeneralJsonAsyncIterator};
use crate::{get_or_initialize_pool, types::Json};

#[cfg(feature = "python")]
use crate::types::{GeneralJsonAsyncIteratorPython, JsonPython};

#[cfg(feature = "c")]
use crate::{languages::c::GeneralJsonAsyncIteratorC, languages::c::JsonC};

#[cfg_attr(
    feature = "rust_bridge",
    alias_methods(new, transform, transform_stream)
)]
impl TransformerPipeline {
    /// Creates a new [TransformerPipeline]
    ///
    /// # Arguments
    /// * `task` - The task to run
    /// * `model` - The model to use
    /// * `args` - The arguments to pass to the task
    /// * `database_url` - The database url to use. If None, the `PGML_DATABASE_URL` environment variable will be used
    pub fn new(task: &str, model: &str, args: Option<Json>, database_url: Option<String>) -> Self {
        let mut args = args.unwrap_or_default();
        let a = args.as_object_mut().expect("args must be an object");
        a.insert("task".to_string(), task.to_string().into());
        a.insert("model".to_string(), model.into());

        // We must convert any floating point values to integers or our extension will get angry
        for field in ["gpu_layers"] {
            if let Some(v) = a.remove(field) {
                let x: u64 = CustomU64Convertor(v).into();
                a.insert(field.to_string(), x.into());
            }
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
        let mut args = args.unwrap_or_default();
        let a = args.as_object_mut().context("args must be an object")?;

        // Backwards compatible
        if let Some(x) = a.remove("max_new_tokens") {
            a.insert("max_tokens".to_string(), x);
        }

        // We must convert any floating point values to integers or our extension will get angry
        for field in ["max_tokens", "n"] {
            if let Some(v) = a.remove(field) {
                let x: u64 = CustomU64Convertor(v).into();
                a.insert(field.to_string(), x.into());
            }
        }

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
        let results = results.first().unwrap().get::<serde_json::Value, _>(0);
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
        let batch_size = batch_size.unwrap_or(1);

        let mut args = args.unwrap_or_default();
        let a = args.as_object_mut().context("args must be an object")?;

        // Backwards compatible
        if let Some(x) = a.remove("max_new_tokens") {
            a.insert("max_tokens".to_string(), x);
        }

        // We must convert any floating point values to integers or our extension will get angry
        for field in ["max_tokens", "n"] {
            if let Some(v) = a.remove(field) {
                let x: u64 = CustomU64Convertor(v).into();
                a.insert(field.to_string(), x.into());
            }
        }

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

        let s = futures::stream::try_unfold(transaction, move |mut transaction| async move {
            let query = format!("FETCH {} FROM c", batch_size);
            let mut res: Vec<Json> = sqlx::query_scalar(&query)
                .fetch_all(&mut *transaction)
                .await?;
            if !res.is_empty() {
                if batch_size > 1 {
                    let res: Vec<String> = res
                        .into_iter()
                        .map(|v| {
                            v.0.as_array()
                                .context("internal SDK error - cannot parse db value as array. Please post a new github issue")
                                .map(|v| {
                                    v[0].as_str()
                                        .context(
                                            "internal SDK error - cannot parse db value as string. Please post a new github issue",
                                        )
                                        .map(|v| v.to_owned())
                                })
                        })
                        .collect::<anyhow::Result<anyhow::Result<Vec<String>>>>()??;
                    Ok(Some((serde_json::json!(res).into(), transaction)))
                } else {
                    Ok(Some((std::mem::take(&mut res[0]), transaction)))
                }
            } else {
                transaction.commit().await?;
                Ok(None)
            }
        });
        Ok(GeneralJsonAsyncIterator(Box::pin(s)))
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
        let t = TransformerPipeline::new("translation_en_to_fr", "t5-base", None, None);
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
            "meta-llama/Meta-Llama-3-8B-Instruct",
            None,
            None,
        );
        let mut stream = t
            .transform_stream(
                serde_json::json!("AI is going to").into(),
                Some(
                    serde_json::json!({
                        "max_new_tokens": 30
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
