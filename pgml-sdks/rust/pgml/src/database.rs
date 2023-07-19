use log::warn;
use pgml_macros::{custom_derive, custom_methods};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::Row;
use std::borrow::Borrow;
use std::str::FromStr;
use std::time::SystemTime;

use crate::collection::*;
use crate::model::Model;
use crate::models;
use crate::queries;
use crate::query_builder;
use crate::query_runner::QueryRunner;
use crate::splitter::Splitter;
use crate::types::Json;

#[cfg(feature = "javascript")]
use crate::{languages::javascript::*, model::ModelJavascript, splitter::SplitterJavascript};

#[cfg(feature = "python")]
use crate::{model::ModelPython, query_runner::QueryRunnerPython, splitter::SplitterPython};

/// A connection to a postgres database
#[derive(custom_derive, Clone, Debug)]
pub struct Database {
    pub pool: PgPool,
}

#[custom_methods(
    new,
    create_or_get_collection,
    does_collection_exist,
    archive_collection,
    query,
    transform,
    register_model,
    get_models,
    get_model,
    register_text_splitter,
    get_text_splitters,
    get_text_splitter
)]
impl Database {
    /// Create a new [Database]
    ///
    ///  # Arguments
    ///
    ///  * `connection_string` - A postgres connection string, e.g. `postgres://user:pass@localhost:5432/db`
    ///
    ///  # Example
    ///  ```
    ///  use pgml::Database;
    ///
    ///  const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    ///  async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    // Do stuff with the database
    ///    Ok(())
    ///  }
    ///  ```
    pub async fn new(connection_string: &str) -> anyhow::Result<Self> {
        println!("Connecting to: {:?}", connection_string);
        let connection_options = PgConnectOptions::from_str(connection_string)?;
        let connection_options = connection_options.statement_cache_capacity(0);
        let pool = PgPoolOptions::new()
            .min_connections(1)
            .max_connections(5)
            .connect_with(connection_options)
            .await?;
        // let _test = sqlx::query("SELECT * FROM pgml.collections")
        //     .fetch_all(&pool)
        //     .await?;
        Ok(Self { pool })
    }

    /// Create a new [Collection] or get an existing one
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the [Collection]
    ///
    /// # Example
    ///```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    let collection = db.create_or_get_collection("collection number 1").await?;
    ///    // Do stuff with the collection
    ///    Ok(())
    /// }
    /// ```
    pub async fn create_or_get_collection(&self, name: &str) -> anyhow::Result<Collection> {
        println!("Yo 1");
        let collection: Result<Option<models::Collection>, _> =
            sqlx::query_as::<_, models::Collection>(
                "SELECT * FROM pgml.collections WHERE name = $1 AND active = TRUE;",
            )
            .bind(name)
            .fetch_optional(&self.pool)
            .await;

        println!("Yo 2");
        match collection {
            Ok(result) => match result {
                Some(c) => Ok(Collection::from_model_and_pool(c, self.pool.clone())),
                None => Ok(Collection::new(name.to_string(), self.pool.clone()).await?),
            },
            Err(e) => {
                println!("Yo 4");
                match e.as_database_error() {
                    Some(db_e) => {
                        // Error 42P01 is "undefined_table"
                        if db_e.code() == Some(std::borrow::Cow::from("42P01")) {
                            sqlx::query(queries::CREATE_COLLECTIONS_TABLE)
                                .execute(&self.pool)
                                .await?;
                            Ok(Collection::new(name.to_string(), self.pool.clone()).await?)
                        } else {
                            return Err(e.into());
                        }
                    }
                    None => return Err(e.into()),
                }
            }
        }
    }

    /// Check if a [Collection] exists
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the [Collection]
    ///
    /// # Example
    /// ```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@localhost:5432/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///   let db = Database::new(CONNECTION_STRING).await?;
    ///   let collection_exists = db.does_collection_exist("collection number 1").await?;
    ///   // Do stuff with your new found information
    ///   Ok(())
    /// }
    /// ```
    pub async fn does_collection_exist(&self, name: &str) -> anyhow::Result<bool> {
        let collection: Option<models::Collection> = sqlx::query_as::<_, models::Collection>(
            "SELECT * FROM pgml.collections WHERE name = $1 AND active = TRUE;",
        )
        .bind(name)
        .fetch_optional(self.pool.borrow())
        .await?;
        Ok(collection.is_some())
    }

    /// Archive a [Collection]
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the [Collection]
    ///
    /// # Example
    ///```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    db.archive_collection("collection number 1").await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn archive_collection(&self, name: &str) -> anyhow::Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Error getting system time")
            .as_secs();
        let archive_table_name = format!("{}_archive_{}", name, timestamp);
        sqlx::query("UPDATE pgml.collections SET name = $1, active = FALSE where name = $2")
            .bind(&archive_table_name)
            .bind(name)
            .execute(self.pool.borrow())
            .await?;
        sqlx::query(&query_builder!(
            "ALTER SCHEMA %s RENAME TO %s",
            name,
            archive_table_name
        ))
        .execute(self.pool.borrow())
        .await?;
        Ok(())
    }

    /// Run an arbitrary query
    ///
    /// # Arguments
    ///
    /// * `query` - The query to run
    ///
    /// # Example
    ///```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@localhost:5432/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///   let db = Database::new(CONNECTION_STRING).await?;
    ///   let query = "SELECT * FROM pgml.collections";
    ///   let results = db.query(query).fetch_all().await?;
    ///   Ok(())
    /// }
    ///```
    pub fn query(&self, query: &str) -> QueryRunner {
        QueryRunner::new(query, self.pool.clone())
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
    // use pgml::Database;
    //
    // const CONNECTION_STRING: &str = "postgres://postgres@localhost:5432/pgml_development";
    //
    // async fn example() -> anyhow::Result<()> {
    //  let db = Database::new(CONNECTION_STRING).await?;
    //  let task = Json::from(serde_json::json!("translation_en_to_fr"));
    //  let inputs = vec![
    //    "test1".to_string(),
    //    "test2".to_string(),
    //  ];
    //  let results = db.transform(task, inputs, None).await?;
    //  Ok(())
    //}
    // ```
    pub async fn transform(
        &self,
        task: Json,
        inputs: Vec<String>,
        args: Option<Json>,
    ) -> anyhow::Result<Json> {
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
        let results = query
            .bind(inputs)
            .bind(args)
            .fetch_all(self.pool.borrow())
            .await?;
        let results = results.get(0).unwrap().get::<serde_json::Value, _>(0);
        Ok(Json(results))
    }

    /// Registers new models for specific tasks
    ///
    /// # Arguments
    ///
    /// * `task` - The name of the task.
    /// * `model_name` - The name of the [models::Model].
    /// * `model_params` - A [std::collections::HashMap] of parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    db.register_model(None, None, None).await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn register_model(
        &self,
        model_name: Option<String>,
        model_task: Option<String>,
        model_params: Option<Json>,
        model_source: Option<String>,
    ) -> anyhow::Result<Model> {
        let model_name = model_name.unwrap_or("intfloat/e5-small".to_string());
        let model_task = model_task.unwrap_or("embedding".to_string());
        let model_params = match model_params {
            Some(params) => params.0,
            None => serde_json::json!({}),
        };
        let model_source = model_source.unwrap_or("pgml".to_string());

        let current_model: Option<models::Model> = sqlx::query_as(
            "SELECT * FROM pgml.sdk_models WHERE task = $1 AND name = $2 AND parameters = $3 AND source = $4;",
        )
        .bind(&model_task)
        .bind(&model_name)
        .bind(&model_params)
        .bind(&model_source)
        .fetch_optional(&self.pool)
        .await?;

        match current_model {
            Some(model) => {
                warn!(
                    "Model with task: {} - name: {} - parameters: {:?} already exists",
                    model_task, model_name, model_params
                );
                Ok(model.into())
            }
            None => {
                let model: models::Model = sqlx::query_as(
                    "INSERT INTO pgml.sdk_models (task, name, parameters, source) VALUES ($1, $2, $3, $4) RETURNING *",
                )
                .bind(model_task)
                .bind(model_name)
                .bind(model_params)
                .bind(model_source)
                .fetch_one(&self.pool)
                .await?;
                Ok(model.into())
            }
        }
    }

    /// Gets all registered [models::Model]s
    pub async fn get_models(&self) -> anyhow::Result<Vec<Model>> {
        Ok(sqlx::query_as("SELECT * from pgml.sdk_models")
            .fetch_all(self.pool.borrow())
            .await?
            .into_iter()
            .map(|m: models::Model| m.into())
            .collect())
    }

    /// Gets a specific [Model] by id
    pub async fn get_model(&self, id: i64) -> anyhow::Result<Model> {
        Ok(
            sqlx::query_as::<_, models::Model>("SELECT * from pgml.sdk_models WHERE id = $1")
                .bind(id)
                .fetch_one(self.pool.borrow())
                .await?
                .into(),
        )
    }

    /// Registers new text splitters
    ///
    /// # Arguments
    ///
    /// * `splitter_name` - The name of the text splitter.
    /// * `splitter_params` - A [std::collections::HashMap] of parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::Database;
    ///
    /// const CONNECTION_STRING: &str = "postgres://postgres@127.0.0.1:5433/pgml_development";
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///    let db = Database::new(CONNECTION_STRING).await?;
    ///    db.register_text_splitter(None, None).await?;
    ///    Ok(())
    /// }
    /// ```
    pub async fn register_text_splitter(
        &self,
        splitter_name: Option<String>,
        splitter_params: Option<Json>,
    ) -> anyhow::Result<Splitter> {
        let splitter_name = splitter_name.unwrap_or("recursive_character".to_string());
        let splitter_params = match splitter_params {
            Some(params) => params.0,
            None => serde_json::json!({}),
        };

        let current_splitter: Option<models::Splitter> =
            sqlx::query_as("SELECT * from pgml.sdk_splitters where name = $1 and parameters = $2;")
                .bind(&splitter_name)
                .bind(&splitter_params)
                .fetch_optional(self.pool.borrow())
                .await?;

        match current_splitter {
            Some(splitter) => {
                warn!(
                    "Text splitter with name: {} and parameters: {:?} already exists",
                    splitter_name, splitter_params
                );
                Ok(splitter.into())
            }
            None => {
                let splitter: models::Splitter = sqlx::query_as(
                    "INSERT INTO pgml.sdk_splitters (name, parameters) VALUES ($1, $2) RETURNING *",
                )
                .bind(splitter_name)
                .bind(splitter_params)
                .fetch_one(self.pool.borrow())
                .await?;
                Ok(splitter.into())
            }
        }
    }

    /// Gets all registered text [models::Splitter]s
    pub async fn get_text_splitters(&self) -> anyhow::Result<Vec<Splitter>> {
        Ok(sqlx::query_as("SELECT * from pgml.sdk_splitters")
            .fetch_all(self.pool.borrow())
            .await?
            .into_iter()
            .map(|s: models::Splitter| s.into())
            .collect())
    }

    /// Gets a specific [Splitter] by id
    pub async fn get_text_splitter(&self, id: i64) -> anyhow::Result<Splitter> {
        Ok(
            sqlx::query_as::<_, models::Splitter>("SELECT * from pgml.sdk_splitters WHERE id = $1")
                .bind(id)
                .fetch_one(self.pool.borrow())
                .await?
                .into(),
        )
    }
}
