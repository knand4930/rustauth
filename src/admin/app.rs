use axum::{Router, routing::get};

use crate::state::AppState;

use super::resource;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/adminx/dashboard", get(resource::dashboard::dashboard))
        .route(
            "/api/adminx/resources",
            get(resource::registry::list_resources),
        )
        .route(
            "/api/adminx/resources/{app_key}",
            get(resource::registry::get_app_resources),
        )
        .route("/api/adminx/users", get(resource::users::list_users))
        .route(
            "/api/adminx/users/{id}",
            get(resource::users::get_user).patch(resource::users::update_user),
        )
}
