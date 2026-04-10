// db.rs

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Create a PostgreSQL connection pool.
///
/// Uses `DATABASE_URL` from the application config.
/// The pool is shared across the entire application via Axum's State extractor.
pub async fn init_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to create database pool — check DATABASE_URL")
}
