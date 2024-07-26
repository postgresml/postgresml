use sqlx::PgPool;

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
