use anyhow::Context;
use sqlx::Row;
use tracing::instrument;

use crate::{get_or_initialize_pool, query_runner::QueryRunner, types::Json};

#[cfg(feature = "rust_bridge")]
use rust_bridge::{alias, alias_methods};

#[cfg(feature = "python")]
use crate::{query_runner::QueryRunnerPython, types::JsonPython};

#[cfg(feature = "c")]
use crate::{languages::c::JsonC, query_runner::QueryRunnerC};

/// Provides access to builtin database methods
#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[derive(Debug, Clone)]
pub struct Builtins {
    database_url: Option<String>,
}

#[cfg_attr(
    feature = "rust_bridge",
    alias_methods(new, query, transform, embed, embed_batch)
)]
impl Builtins {
    pub fn new(database_url: Option<String>) -> Self {
        Self { database_url }
    }

    /// Run an arbitrary query
    ///
    /// # Arguments
    ///
    /// * `query` - The query to run
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Builtins;
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let builtins = Builtins::new(None);
    ///     let query = "SELECT * FROM pgml.collections";
    ///     let results = builtins.query(query).fetch_all().await?;
    ///     Ok(())
    /// }
    ///```
    #[instrument(skip(self))]
    pub fn query(&self, query: &str) -> QueryRunner {
        QueryRunner::new(query, self.database_url.clone())
    }

    // Run the builtin pgml.transform function
    //
    // # Arguments
    //
    // * `task` - The task to run
    // * `inputs` - The inputs to the model
    // * `args` - The arguments to the model
    //
    // # Example
    //
    // ```
    // use pgml::Builtins;
    //
    // async fn example() -> anyhow::Result<()> {
    //    let builtins = Builtins::new(None);
    //    let task = Json::from(serde_json::json!("translation_en_to_fr"));
    //    let inputs = vec![
    //       "test1".to_string(),
    //       "test2".to_string(),
    //    ];
    //    let results = builtins.transform(task, inputs, None).await?;
    //    Ok(())
    // }
    // ```
    pub async fn transform(
        &self,
        task: Json,
        inputs: Vec<String>,
        args: Option<Json>,
    ) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let args = match args {
            Some(a) => a.0,
            None => serde_json::json!({}),
        };
        let query = sqlx::query("SELECT pgml.transform(task => $1, inputs => $2, args => $3)");
        let query = if task.0.is_string() {
            query.bind(task.0.as_str())
        } else {
            query.bind(task.0)
        };
        let results = query.bind(inputs).bind(args).fetch_all(&pool).await?;
        let results = results.first().unwrap().get::<serde_json::Value, _>(0);
        Ok(Json(results))
    }

    /// Run the built-in `pgml.embed()` function.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to use.
    /// * `text` - The text to embed.
    ///
    pub async fn embed(&self, model: &str, text: &str) -> anyhow::Result<Json> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let query = sqlx::query("SELECT embed FROM pgml.embed($1, $2)");
        let result = query.bind(model).bind(text).fetch_one(&pool).await?;
        let result = result.get::<Vec<f32>, _>(0);
        let result = serde_json::to_value(result)?;
        Ok(Json(result))
    }

    /// Run the built-in `pgml.embed()` function, but with handling for batch inputs and outputs.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to use.
    /// * `texts` - The texts to embed.
    ///
    pub async fn embed_batch(&self, model: &str, texts: Json) -> anyhow::Result<Json> {
        let texts = texts
            .0
            .as_array()
            .with_context(|| "embed_batch takes an array of strings")?
            .iter()
            .map(|v| {
                v.as_str()
                    .with_context(|| "only text embeddings are supported")
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<String>>();
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let query = sqlx::query("SELECT embed AS embed_batch FROM pgml.embed($1, $2)");
        let results = query
            .bind(model)
            .bind(texts)
            .fetch_all(&pool)
            .await?
            .into_iter()
            .map(|embeddings| embeddings.get::<Vec<f32>, _>(0))
            .collect::<Vec<Vec<f32>>>();
        Ok(Json(serde_json::to_value(results)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_init_logger;

    #[sqlx::test]
    async fn can_query() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let builtins = Builtins::new(None);
        let query = "SELECT * from pgml.collections";
        let results = builtins.query(query).fetch_all().await?;
        assert!(results.as_array().is_some());
        Ok(())
    }

    #[sqlx::test]
    async fn can_transform() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let builtins = Builtins::new(None);
        let task = Json::from(serde_json::json!({
            "task": "text-generation",
            "model": "meta-llama/Meta-Llama-3-8B-Instruct"
        }));
        let inputs = vec!["test1".to_string(), "test2".to_string()];
        let results = builtins.transform(task, inputs, None).await?;
        assert!(results.as_array().is_some());
        Ok(())
    }

    #[tokio::test]
    async fn can_embed() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let builtins = Builtins::new(None);
        let results = builtins.embed("intfloat/e5-small-v2", "test").await?;
        assert!(results.as_array().is_some());
        Ok(())
    }

    #[tokio::test]
    async fn can_embed_batch() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        let builtins = Builtins::new(None);
        let results = builtins
            .embed_batch(
                "intfloat/e5-small-v2",
                Json(serde_json::json!(["test", "test2",])),
            )
            .await?;
        assert!(results.as_array().is_some());
        assert_eq!(results.as_array().unwrap().len(), 2);
        Ok(())
    }
}
