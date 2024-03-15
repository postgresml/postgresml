use rust_bridge::{alias, alias_methods};
use sqlx::Row;
use tracing::instrument;

/// Provides access to builtin database methods
#[derive(alias, Debug, Clone)]
pub struct Builtins {
    database_url: Option<String>,
}

use crate::{get_or_initialize_pool, query_runner::QueryRunner, types::Json};

#[cfg(feature = "python")]
use crate::{query_runner::QueryRunnerPython, types::JsonPython};

#[alias_methods(new, query, transform)]
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
        let results = results.get(0).unwrap().get::<serde_json::Value, _>(0);
        Ok(Json(results))
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
        let task = Json::from(serde_json::json!("translation_en_to_fr"));
        let inputs = vec!["test1".to_string(), "test2".to_string()];
        let results = builtins.transform(task, inputs, None).await?;
        assert!(results.as_array().is_some());
        Ok(())
    }
}
