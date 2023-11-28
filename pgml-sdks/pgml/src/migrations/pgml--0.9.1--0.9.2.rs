use crate::{queries, query_builder};
use sqlx::Executor;
use sqlx::PgPool;
use tracing::instrument;

#[instrument(skip(pool))]
pub async fn migrate(pool: PgPool, _: Vec<i64>) -> anyhow::Result<String> {
    pool.execute("ALTER EXTENSION vector UPDATE").await?;
    let version: String =
        sqlx::query_scalar("SELECT extversion FROM pg_extension WHERE extname = 'vector'")
            .fetch_one(&pool)
            .await?;
    let value = version.split('.').collect::<Vec<&str>>()[1].parse::<u64>()?;
    anyhow::ensure!(
        value >= 5,
        "Vector extension must be at least version 0.5.0"
    );

    let collection_names: Vec<String> = sqlx::query_scalar("SELECT name FROM pgml.collections")
        .fetch_all(&pool)
        .await?;
    for collection_name in collection_names {
        let table_name = format!("{}.pipelines", collection_name);
        let pipeline_names: Vec<String> =
            sqlx::query_scalar(&query_builder!("SELECT name FROM %s", table_name))
                .fetch_all(&pool)
                .await?;
        for pipeline_name in pipeline_names {
            let embeddings_table_name = format!("{}_embeddings", pipeline_name);
            let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT * FROM information_schema.tables WHERE table_name = $1 and table_schema = $2)")
                .bind(embeddings_table_name)
                .bind(&collection_name)
                .fetch_one(&pool)
                .await?;
            if exists {
                let table_name = format!("{}.{}_embeddings", collection_name, pipeline_name);
                let index_name = format!("{}_pipeline_hnsw_vector_index", pipeline_name);
                pool.execute(
                    query_builder!(
                        queries::CREATE_INDEX_USING_HNSW,
                        "",
                        index_name,
                        &table_name,
                        "embedding vector_cosine_ops",
                        ""
                    )
                    .as_str(),
                )
                .await?;
            }
        }
        // We can get rid of the old IVFFlat index now. There was a bug where we named it the same
        // thing no matter what, so we only need to remove one index.
        pool.execute(
            query_builder!(
                "DROP INDEX CONCURRENTLY IF EXISTS %s.vector_index",
                collection_name
            )
            .as_str(),
        )
        .await?;
    }

    // Required to set the default value for a not null column being added, but we want to remove
    // it right after
    let mut transaction = pool.begin().await?;
    transaction.execute("ALTER TABLE pgml.collections ADD COLUMN IF NOT EXISTS sdk_version text NOT NULL DEFAULT '0.9.2'").await?;
    transaction
        .execute("ALTER TABLE pgml.collections ALTER COLUMN sdk_version DROP DEFAULT")
        .await?;
    transaction.commit().await?;
    Ok("0.9.2".to_string())
}
