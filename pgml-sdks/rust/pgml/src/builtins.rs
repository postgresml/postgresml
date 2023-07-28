use pgml_macros::{custom_derive, custom_methods};
use sqlx::Row;

#[derive(custom_derive, Debug, Clone)]
pub struct Builtins {
    pub database_url: Option<String>,
}

use crate::{get_or_initialize_pool, models, query_runner::QueryRunner, types::Json};

#[cfg(feature = "javascript")]
use crate::{languages::javascript::*, query_runner::QueryRunnerJavascript};

#[cfg(feature = "python")]
use crate::{query_runner::QueryRunnerPython, languages::CustomInto};

#[custom_methods(new, query, transform, does_collection_exist)]
impl Builtins {
    pub fn new(database_url: Option<String>) -> Self {
        Self { database_url }
    }

    /// Check if a [Collection] exists
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the [Collection]
    ///
    /// # Example
    /// ```
    /// async fn example() -> anyhow::Result<()> {
    ///   let builtins = Builtins::new(None);
    ///   let collection_exists = builtins.does_collection_exist("collection number 1").await?;
    ///   // Do stuff with your new found information
    ///   Ok(())
    /// }
    /// ```
    pub async fn does_collection_exist(&self, name: &str) -> anyhow::Result<bool> {
        let pool = get_or_initialize_pool(&self.database_url).await?;
        let collection: Option<models::Collection> = sqlx::query_as::<_, models::Collection>(
            "SELECT * FROM pgml.collections WHERE name = $1 AND active = TRUE;",
        )
        .bind(name)
        .fetch_optional(&pool)
        .await?;
        Ok(collection.is_some())
    }

    /// Run an arbitrary query
    ///
    /// # Arguments
    ///
    /// * `query` - The query to run
    ///
    /// # Example
    ///```
    /// async fn example() -> anyhow::Result<()> {
    ///   let builtins = Builtins::new();
    ///   let query = "SELECT * FROM pgml.collections";
    ///   let results = builtins.query(query).fetch_all().await?;
    ///   Ok(())
    /// }
    ///```
    pub fn query(&self, query: &str) -> QueryRunner {
        QueryRunner::new(query, self.database_url.clone())
    }

    // Run the builtin transform function
    //
    // # Arguments
    //
    // * `task` - The task to run
    // * `inputs` - The inputs to the model
    // * `args` - The arguments to the model
    //
    // # Example
    // ```
    // async fn example() -> anyhow::Result<()> {
    //  let builtins = Builtins::new(None);
    //  let task = Json::from(serde_json::json!("translation_en_to_fr"));
    //  let inputs = vec![
    //    "test1".to_string(),
    //    "test2".to_string(),
    //  ];
    //  let results = builtins.transform(task, inputs, None).await?;
    //  Ok(())
    //}
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
