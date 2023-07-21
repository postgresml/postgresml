use pgml_macros::{custom_derive, custom_methods};

#[derive(custom_derive, Debug, Clone)]
pub struct Builtins {
    pub database_url: Option<String>,
}

use crate::{get_or_initialize_pool, models};

#[custom_methods(
    new,
    does_collection_exist
)]
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
    ///   let b = Builtins::new(None).await?;
    ///   let collection_exists = b.does_collection_exist("collection number 1").await?;
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
}
