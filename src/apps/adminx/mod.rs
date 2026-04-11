pub mod handlers;
pub mod models;
pub mod schemas;

pub use schemas::{AdminDashboardResponse, AdminUserResponse, UpdateAdminUserRequest};

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/adminx/dashboard", get(handlers::dashboard))
        .route("/api/adminx/users", get(handlers::list_users))
        .route(
            "/api/adminx/users/{id}",
            get(handlers::get_user).patch(handlers::update_user),
        )
}
