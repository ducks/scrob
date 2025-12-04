pub mod models;

use sqlx::{sqlite::SqlitePool, migrate::MigrateDatabase, Sqlite};

pub type DbPool = SqlitePool;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
  if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
    tracing::info!("Creating database: {}", database_url);
    Sqlite::create_database(database_url).await?;
  }

  let pool = SqlitePool::connect(database_url).await?;

  tracing::info!("Running migrations...");
  sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;

  tracing::info!("Database ready");
  Ok(pool)
}
