// src/state.rs
//
// Shared application state injected into every handler via Axum's State extractor.
// Keeps the database pool and config in one place so handlers never need to
// re-read environment variables at runtime.

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(db: PgPool, config: AppConfig) -> Self {
        Self { db, config: Arc::new(config) }
    }
}
