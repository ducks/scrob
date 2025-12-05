pub mod models;

use sqlx::postgres::PgPool;

pub type DbPool = PgPool;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
  let pool = PgPool::connect(database_url).await?;

  tracing::info!("Running migrations...");
  sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;

  tracing::info!("Database ready");
  Ok(pool)
}
