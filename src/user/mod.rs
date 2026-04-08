// src/user/mod.rs

pub mod handler;
pub mod models;
pub mod schema;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

/// Build the user-related routes.
pub fn routes() -> Router<PgPool> {
    Router::new()
        // Auth endpoints
        .route("/api/v1/auth/register", post(handler::register))
        .route("/api/v1/auth/login", post(handler::login))
        // User CRUD
        .route("/api/v1/users", get(handler::list_users))
        .route("/api/v1/users/{id}", get(handler::get_user))
        .route("/api/v1/users/{id}", put(handler::update_user))
        .route("/api/v1/users/{id}", delete(handler::delete_user))
}
