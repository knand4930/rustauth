use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    apps::user::User,
    error::AppError,
    response::{ApiPaginated, ApiSuccess},
    state::AppState,
};

use super::auth::require_admin;

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
pub struct ListAdminUsersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub is_staffuser: Option<bool>,
    pub is_superuser: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateAdminUserRequest {
    #[validate(length(min = 1, message = "Full name cannot be empty"))]
    pub full_name: Option<String>,
    #[validate(length(min = 1, message = "Company cannot be empty"))]
    pub company: Option<String>,
    #[validate(length(min = 1, message = "Phone number cannot be empty"))]
    pub phone_number: Option<String>,
    #[validate(length(min = 1, message = "Timezone cannot be empty"))]
    pub timezone: Option<String>,
    #[validate(length(min = 1, message = "Language cannot be empty"))]
    pub language: Option<String>,
    #[validate(url(message = "Avatar URL must be valid"))]
    pub avatar_url: Option<String>,
    #[validate(length(min = 1, message = "Location cannot be empty"))]
    pub location: Option<String>,
    pub is_active: Option<bool>,
    pub is_staffuser: Option<bool>,
    pub is_superuser: Option<bool>,
    pub email_verified: Option<bool>,
    pub phone_verified: Option<bool>,
    pub mfa_enabled: Option<bool>,
}

impl UpdateAdminUserRequest {
    fn is_empty(&self) -> bool {
        self.full_name.is_none()
            && self.company.is_none()
            && self.phone_number.is_none()
            && self.timezone.is_none()
            && self.language.is_none()
            && self.avatar_url.is_none()
            && self.location.is_none()
            && self.is_active.is_none()
            && self.is_staffuser.is_none()
            && self.is_superuser.is_none()
            && self.email_verified.is_none()
            && self.phone_verified.is_none()
            && self.mfa_enabled.is_none()
    }
}

#[derive(Debug, FromRow)]
struct AdminManagedUser {
    id: Uuid,
    email: Option<String>,
    full_name: Option<String>,
    company: Option<String>,
    avatar_url: Option<String>,
    phone_number: Option<String>,
    timezone: String,
    language: String,
    location: Option<String>,
    is_active: bool,
    is_superuser: bool,
    is_staffuser: bool,
    email_verified: bool,
    phone_verified: bool,
    mfa_enabled: bool,
    last_login_at: Option<DateTime<Utc>>,
    login_count: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub company: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub timezone: String,
    pub language: String,
    pub location: Option<String>,
    pub is_active: bool,
    pub is_superuser: bool,
    pub is_staffuser: bool,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub mfa_enabled: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub login_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AdminManagedUser> for AdminUserResponse {
    fn from(user: AdminManagedUser) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            company: user.company,
            avatar_url: user.avatar_url,
            phone_number: user.phone_number,
            timezone: user.timezone,
            language: user.language,
            location: user.location,
            is_active: user.is_active,
            is_superuser: user.is_superuser,
            is_staffuser: user.is_staffuser,
            email_verified: user.email_verified,
            phone_verified: user.phone_verified,
            mfa_enabled: user.mfa_enabled,
            last_login_at: user.last_login_at,
            login_count: user.login_count,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
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
        (status = 403, description = "Superuser privileges required"),
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
        FROM {}
        WHERE ($1::text IS NULL OR email ILIKE $1 OR full_name ILIKE $1)
          AND ($2::bool IS NULL OR is_active = $2)
          AND ($3::bool IS NULL OR is_staffuser = $3)
          AND ($4::bool IS NULL OR is_superuser = $4)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
        User::QUALIFIED_TABLE
    );

    let count_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {}
        WHERE ($1::text IS NULL OR email ILIKE $1 OR full_name ILIKE $1)
          AND ($2::bool IS NULL OR is_active = $2)
          AND ($3::bool IS NULL OR is_staffuser = $3)
          AND ($4::bool IS NULL OR is_superuser = $4)
    "#,
        User::QUALIFIED_TABLE
    );

    let users = sqlx::query_as::<_, AdminManagedUser>(&list_sql)
        .bind(search.as_deref())
        .bind(params.is_active)
        .bind(params.is_staffuser)
        .bind(params.is_superuser)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total = sqlx::query_scalar::<_, i64>(&count_sql)
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
        (status = 200, description = "Admin view of a single user", body = AdminUserResponse),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Superuser privileges required"),
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

    let sql = format!(
        "SELECT {MANAGED_USER_COLUMNS} FROM {} WHERE id = $1",
        User::QUALIFIED_TABLE
    );
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
        (status = 200, description = "Managed user updated", body = AdminUserResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Superuser privileges required"),
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
        UPDATE {}
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
        "#,
        User::QUALIFIED_TABLE
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

fn normalize_search(search: Option<&str>) -> Option<String> {
    let trimmed = search?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("%{trimmed}%"))
    }
}
