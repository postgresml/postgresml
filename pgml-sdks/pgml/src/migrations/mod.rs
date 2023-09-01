use futures::FutureExt;
use itertools::Itertools;
use sqlx::PgPool;
use tracing::instrument;

use crate::get_or_initialize_pool;

#[path = "pgml--0.9.1--0.9.2.rs"]
mod pgml091_092;

// There is probably a better way to write these types and bypass the need for the closure pass
// through, but it is proving to be difficult
// We could also probably remove some unnecessary clones in the call_migrate function if I was savy
// enough to reconcile the lifetimes
type MigrateFn =
    &'static dyn Fn(PgPool, Vec<i64>) -> futures::future::BoxFuture<'static, anyhow::Result<()>>;
const VERSION_MIGRATIONS: &'static [(&'static str, MigrateFn)] =
    &[("0.9.2", &|p, c| pgml091_092::migrate(p, c).boxed())];

#[instrument]
pub async fn migrate() -> anyhow::Result<()> {
    let pool = get_or_initialize_pool(&None).await?;
    let results: Result<Vec<(String, i64)>, _> =
        sqlx::query_as("SELECT version, id FROM pgml.collections")
            .fetch_all(&pool)
            .await;
    match results {
        Ok(collections) => {
            let collections = collections.into_iter().into_group_map();
            for (version, collection_ids) in collections.into_iter() {
                call_migrate(pool.clone(), version, collection_ids).await?
            }
            Ok(())
        }
        Err(error) => {
            let morphed_error = error
                .as_database_error()
                .map(|e| e.code().map(|c| c.to_string()));
            if let Some(Some(db_error_code)) = morphed_error {
                if db_error_code == "42703" {
                    pgml091_092::migrate(pool, vec![]).await
                } else {
                    anyhow::bail!(error)
                }
            } else {
                anyhow::bail!(error)
            }
        }
    }
}

async fn call_migrate(
    pool: PgPool,
    version: String,
    collection_ids: Vec<i64>,
) -> anyhow::Result<()> {
    let position = VERSION_MIGRATIONS.iter().position(|(v, _)| v == &version);
    if let Some(p) = position {
        // We run each migration in order that needs to be ran for the collections
        for (_, callback) in VERSION_MIGRATIONS.iter().skip(p + 1) {
            callback(pool.clone(), collection_ids.clone()).await?
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_init_logger;

    #[tokio::test]
    async fn test_migrate() -> anyhow::Result<()> {
        internal_init_logger(None, None).ok();
        migrate().await?;
        Ok(())
    }
}
