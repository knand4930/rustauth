use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, header},
    response::IntoResponse,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    response::{ApiPaginated, ApiSuccess},
    state::AppState,
};

use super::{
    models::{AdminActor, AdminDashboardMetrics, AdminManagedUser},
    schemas::{
        AdminDashboardResponse, AdminUserResponse, ListAdminUsersQuery, UpdateAdminUserRequest,
    },
};

const MANAGED_USER_COLUMNS: &str = r#"
    id,
    email,
    full_name,
    company,
    avatar_url,
    phone_number,
    timezone,
    language,
    location,
    is_active,
    is_superuser,
    is_staffuser,
    email_verified,
    phone_verified,
    mfa_enabled,
    last_login_at,
    login_count,
    created_at,
    updated_at
"#;

#[derive(Debug, Deserialize)]
struct AccessClaims {
    sub: String,
    #[serde(rename = "exp")]
    _exp: usize,
    #[serde(rename = "type")]
    token_type: String,
}

#[utoipa::path(
    get,
    path = "/api/adminx/dashboard",
    params(
        ("Authorization" = String, Header, description = "Bearer admin access token"),
    ),
    responses(
        (status = 200, description = "Admin dashboard summary"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Admin privileges required"),
    ),
    tag = "AdminX"
)]
pub async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let actor = require_admin(&state, &headers).await?;
    tracing::debug!(admin_id = %actor.id, "Loading admin dashboard");

    let metrics = sqlx::query_as::<_, AdminDashboardMetrics>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM users) AS total_users,
            (SELECT COUNT(*) FROM users WHERE is_active = true) AS active_users,
            (
                SELECT COUNT(*)
                FROM users
                WHERE is_superuser = true OR is_staffuser = true
            ) AS admin_users,
            (
                SELECT COUNT(*)
                FROM users
                WHERE email_verified = true
            ) AS verified_users,
            (SELECT COUNT(*) FROM blog_posts) AS total_blog_posts,
            (
                SELECT COUNT(*)
                FROM blog_posts
                WHERE is_published = true
            ) AS published_blog_posts,
            (
                SELECT COUNT(*)
                FROM comments
                WHERE is_approved = false
            ) AS pending_comments
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(ApiSuccess::ok(AdminDashboardResponse::from(metrics)))
}

#[utoipa::path(
    get,
    path = "/api/adminx/users",
    params(
        ("Authorization" = String, Header, description = "Bearer admin access token"),
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default 20, max 100)"),
        ("search" = Option<String>, Query, description = "Search by email or full name"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("is_staffuser" = Option<bool>, Query, description = "Filter by staff role"),
        ("is_superuser" = Option<bool>, Query, description = "Filter by superuser role"),
    ),
    responses(
        (status = 200, description = "Paginated list of managed users"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Admin privileges required"),
    ),
    tag = "AdminX"
)]
pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListAdminUsersQuery>,
) -> Result<impl IntoResponse, AppError> {
    let actor = require_admin(&state, &headers).await?;
    tracing::debug!(admin_id = %actor.id, "Listing users in adminx");

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let search = normalize_search(params.search.as_deref());

    let list_sql = format!(
        r#"
        SELECT {MANAGED_USER_COLUMNS}
        FROM users
        WHERE ($1::text IS NULL OR email ILIKE $1 OR full_name ILIKE $1)
          AND ($2::bool IS NULL OR is_active = $2)
          AND ($3::bool IS NULL OR is_staffuser = $3)
          AND ($4::bool IS NULL OR is_superuser = $4)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#
    );

    let count_sql = r#"
        SELECT COUNT(*)
        FROM users
        WHERE ($1::text IS NULL OR email ILIKE $1 OR full_name ILIKE $1)
          AND ($2::bool IS NULL OR is_active = $2)
          AND ($3::bool IS NULL OR is_staffuser = $3)
          AND ($4::bool IS NULL OR is_superuser = $4)
    "#;

    let users = sqlx::query_as::<_, AdminManagedUser>(&list_sql)
        .bind(search.as_deref())
        .bind(params.is_active)
        .bind(params.is_staffuser)
        .bind(params.is_superuser)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(count_sql)
        .bind(search.as_deref())
        .bind(params.is_active)
        .bind(params.is_staffuser)
        .bind(params.is_superuser)
        .fetch_one(&state.db)
        .await?;

    let response: Vec<AdminUserResponse> = users.into_iter().map(Into::into).collect();
    Ok(ApiPaginated::new(response, total, page, per_page))
}

#[utoipa::path(
    get,
    path = "/api/adminx/users/{id}",
    params(
        ("Authorization" = String, Header, description = "Bearer admin access token"),
        ("id" = Uuid, Path, description = "Managed user UUID"),
    ),
    responses(
        (status = 200, description = "Admin view of a single user"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Admin privileges required"),
        (status = 404, description = "User not found"),
    ),
    tag = "AdminX"
)]
pub async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let actor = require_admin(&state, &headers).await?;
    tracing::debug!(admin_id = %actor.id, user_id = %id, "Loading managed user");

    let sql = format!("SELECT {MANAGED_USER_COLUMNS} FROM users WHERE id = $1");
    let user = sqlx::query_as::<_, AdminManagedUser>(&sql)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {id} not found")))?;

    Ok(ApiSuccess::ok(AdminUserResponse::from(user)))
}

#[utoipa::path(
    patch,
    path = "/api/adminx/users/{id}",
    params(
        ("Authorization" = String, Header, description = "Bearer admin access token"),
        ("id" = Uuid, Path, description = "Managed user UUID"),
    ),
    request_body = UpdateAdminUserRequest,
    responses(
        (status = 200, description = "Managed user updated"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Admin privileges required"),
        (status = 404, description = "User not found"),
    ),
    tag = "AdminX"
)]
pub async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateAdminUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let actor = require_admin(&state, &headers).await?;
    tracing::debug!(admin_id = %actor.id, user_id = %id, "Updating managed user");

    body.validate()
        .map_err(|error| AppError::BadRequest(error.to_string()))?;

    if body.is_empty() {
        return Err(AppError::BadRequest(
            "At least one field must be provided for update".to_string(),
        ));
    }

    let sql = format!(
        r#"
        UPDATE users
        SET
            full_name = COALESCE($2, full_name),
            company = COALESCE($3, company),
            phone_number = COALESCE($4, phone_number),
            timezone = COALESCE($5, timezone),
            language = COALESCE($6, language),
            avatar_url = COALESCE($7, avatar_url),
            location = COALESCE($8, location),
            is_active = COALESCE($9, is_active),
            is_staffuser = COALESCE($10, is_staffuser),
            is_superuser = COALESCE($11, is_superuser),
            email_verified = COALESCE($12, email_verified),
            phone_verified = COALESCE($13, phone_verified),
            mfa_enabled = COALESCE($14, mfa_enabled),
            updated_at = NOW()
        WHERE id = $1
        RETURNING {MANAGED_USER_COLUMNS}
        "#
    );

    let user = sqlx::query_as::<_, AdminManagedUser>(&sql)
        .bind(id)
        .bind(&body.full_name)
        .bind(&body.company)
        .bind(&body.phone_number)
        .bind(&body.timezone)
        .bind(&body.language)
        .bind(&body.avatar_url)
        .bind(&body.location)
        .bind(body.is_active)
        .bind(body.is_staffuser)
        .bind(body.is_superuser)
        .bind(body.email_verified)
        .bind(body.phone_verified)
        .bind(body.mfa_enabled)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {id} not found")))?;

    Ok(ApiSuccess::ok(AdminUserResponse::from(user)))
}

async fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<AdminActor, AppError> {
    let token = bearer_token_from_headers(headers)?;

    let validation = Validation::new(Algorithm::HS256);
    let token_data = jsonwebtoken::decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired access token".to_string()))?;

    if token_data.claims.token_type != "access" {
        return Err(AppError::Unauthorized(
            "Only access tokens can be used for admin endpoints".to_string(),
        ));
    }

    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;

    let actor = sqlx::query_as::<_, AdminActor>(
        r#"
        SELECT id, is_active, is_superuser, is_staffuser
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Admin account not found".to_string()))?;

    if !actor.is_active {
        return Err(AppError::Forbidden(
            "Inactive accounts cannot access admin endpoints".to_string(),
        ));
    }

    if !(actor.is_superuser || actor.is_staffuser) {
        return Err(AppError::Forbidden(
            "Admin privileges are required for this endpoint".to_string(),
        ));
    }

    Ok(actor)
}

fn bearer_token_from_headers(headers: &HeaderMap) -> Result<&str, AppError> {
    let raw_header = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Authorization header is required".to_string()))?;

    let value = raw_header.to_str().map_err(|_| {
        AppError::Unauthorized("Authorization header must be valid UTF-8".to_string())
    })?;

    value.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized("Authorization header must use Bearer token".to_string())
    })
}

fn normalize_search(search: Option<&str>) -> Option<String> {
    let trimmed = search?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("%{trimmed}%"))
    }
}
