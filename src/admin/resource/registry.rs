use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
};

use crate::{error::AppError, response::ApiSuccess, state::AppState};

use super::auth::require_admin;

#[utoipa::path(
    get,
    path = "/api/adminx/resources",
    params(
        ("Authorization" = String, Header, description = "Bearer admin access token"),
    ),
    responses(
        (status = 200, description = "Registered admin apps and resources", body = crate::admin::AdminPanel),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Superuser privileges required"),
    ),
    tag = "AdminX"
)]
pub async fn list_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let actor = require_admin(&state, &headers).await?;
    tracing::debug!(admin_id = %actor.id, "Listing admin resources");

    Ok(ApiSuccess::ok(state.admin.as_ref().clone()))
}

#[utoipa::path(
    get,
    path = "/api/adminx/resources/{app_key}",
    params(
        ("Authorization" = String, Header, description = "Bearer admin access token"),
        ("app_key" = String, Path, description = "Registered app key"),
    ),
    responses(
        (status = 200, description = "Registered resources for one app", body = crate::admin::AdminAppConfig),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Superuser privileges required"),
        (status = 404, description = "Admin app not found"),
    ),
    tag = "AdminX"
)]
pub async fn get_app_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(app_key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let actor = require_admin(&state, &headers).await?;
    tracing::debug!(admin_id = %actor.id, app = %app_key, "Loading admin app resources");

    let app = state
        .admin
        .find_app(&app_key)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("Admin app '{app_key}' not found")))?;

    Ok(ApiSuccess::ok(app))
}
