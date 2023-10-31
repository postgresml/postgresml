use rust_bridge::{alias, alias_methods};
use sqlx::Row;
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

#[alias_methods(new, transform)]
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_init_logger;

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
}
