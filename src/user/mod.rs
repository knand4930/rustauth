// src/user/mod.rs

pub mod handlers;
pub mod models;
pub mod schemas;

// Re-export key public types — callers can write `user::UserResponse` etc.
pub use models::User;
pub use schemas::{
    AuthTokenResponse, LoginRequest, RegisterRequest, UpdateUserRequest, UserResponse,
};

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Auth
        .route("/api/v1/auth/register", post(handlers::register))
        .route("/api/v1/auth/login", post(handlers::login))
        // User CRUD
        .route("/api/v1/users", get(handlers::list_users))
        .route("/api/v1/users/{id}", get(handlers::get_user))
        .route("/api/v1/users/{id}", put(handlers::update_user))
        .route("/api/v1/users/{id}", delete(handlers::delete_user))
}
