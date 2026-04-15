// src/apps/user/mod.rs

pub mod admin_config;
pub mod admin_registry;
pub mod handlers;
pub mod models;
pub mod schemas;

// Re-export key public types
pub use models::User;
pub use schemas::{
    LoginRefreshResponse, LoginRequest, RegisterRequest, TokenPairResponse, UpdateUserRequest,
    UserResponse, VerifyTokenRequest, VerifyTokenResponse,
};

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Auth
        .route("/api/v1/auth/register", post(handlers::register))
        .route("/api/v1/auth/login", post(handlers::login))
        .route("/api/v1/auth/token/{refresh_token}", post(handlers::token_exchange))
        .route("/api/v1/auth/verify", post(handlers::verify_token))
        .route("/api/v1/auth/me", get(handlers::me))
        // User CRUD
        .route("/api/v1/users", get(handlers::list_users))
        .route("/api/v1/users/{id}", get(handlers::get_user))
        .route("/api/v1/users/{id}", put(handlers::update_user))
        .route("/api/v1/users/{id}", delete(handlers::delete_user))
}
