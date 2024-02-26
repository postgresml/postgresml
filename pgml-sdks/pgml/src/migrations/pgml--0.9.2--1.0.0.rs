use sqlx::PgPool;
use tracing::instrument;

#[instrument(skip(_pool))]
pub async fn migrate(_pool: PgPool, _: Vec<i64>) -> anyhow::Result<String> {
    anyhow::bail!(
        "There is no automatic migration to SDK version 1.0. Please upgrade the SDK and create a new collection, or contact your PostgresML support to create a migration plan.",
    )
}
