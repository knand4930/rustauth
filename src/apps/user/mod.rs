// src/apps/user/mod.rs

pub mod admin_config;
pub mod admin_registry;
pub mod handlers;
pub mod models;
pub mod schemas;

// Re-export key public types — callers can write `user::UserResponse` etc.
pub use models::User;
pub use schemas::{
    AuthTokenResponse, LoginRequest, RegisterRequest, UpdateUserRequest, UserResponse,
};

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{admin::AdminPanelBuilder, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Auth
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        // User CRUD
        .route("/api/users", get(handlers::list_users))
        .route("/api/users/{id}", get(handlers::get_user))
        .route("/api/users/{id}", put(handlers::update_user))
        .route("/api/users/{id}", delete(handlers::delete_user))
}

pub fn register_admin(builder: &mut AdminPanelBuilder) {
    admin_registry::register(builder);
}
