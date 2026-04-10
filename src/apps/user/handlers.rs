// src/apps/user/handlers.rs
//
// Business logic & API route handlers for the User app.
// Uses: models.rs (DB) + schemas.rs (I/O) + crate::response (envelopes)
//

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::response::{ApiMessage, ApiPaginated, ApiSuccess};
use crate::state::AppState;

use super::models::User;
use super::schemas::{
    AuthTokenResponse, ListUsersQuery, LoginRequest, RegisterRequest, UpdateUserRequest,
    UserResponse,
};

// ─── Auth handlers ───────────────────────────────────────────────────

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already exists"),
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let existing =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(&body.email)
            .fetch_one(&state.db)
            .await?;

    if existing {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    use argon2::PasswordHasher;
    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let password_hash = argon2::Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (
            id, email, password, salt, full_name, company, phone_number,
            timezone, language, is_active, is_superuser, is_staffuser,
            email_verified, phone_verified, mfa_enabled, login_count,
            created_at, updated_at
        )
        VALUES (
            gen_random_uuid(), $1, $2, $3, $4, $5, $6,
            'UTC', 'en', true, false, false,
            false, false, false, 0,
            NOW(), NOW()
        )
        RETURNING *
        "#,
    )
    .bind(&body.email)
    .bind(&password_hash)
    .bind(salt.as_str())
    .bind(&body.full_name)
    .bind(&body.company)
    .bind(&body.phone_number)
    .fetch_one(&state.db)
    .await?;

    let response: UserResponse = user.into();
    Ok(ApiSuccess::created(response))
}

/// Login with email and password
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND is_active = true")
            .bind(&body.email)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let parsed_hash = argon2::PasswordHash::new(user.password.as_deref().unwrap_or(""))
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    argon2::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        body.password.as_bytes(),
        &parsed_hash,
    )
    .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    sqlx::query(
        "UPDATE users SET login_count = login_count + 1, last_login_at = NOW() WHERE id = $1",
    )
    .bind(user.id)
    .execute(&state.db)
    .await?;

    // Use shared config from state — no env re-read on each request
    let config = &state.config;
    let now = chrono::Utc::now();
    let access_exp = now + chrono::Duration::minutes(config.jwt_maxage);
    let refresh_exp = now + chrono::Duration::days(7);

    let access_claims = serde_json::json!({
        "sub": user.id.to_string(),
        "email": user.email,
        "exp": access_exp.timestamp(),
        "iat": now.timestamp(),
        "type": "access",
    });

    let refresh_claims = serde_json::json!({
        "sub": user.id.to_string(),
        "exp": refresh_exp.timestamp(),
        "iat": now.timestamp(),
        "type": "refresh",
    });

    let encoding_key = jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let header = jsonwebtoken::Header::default();

    let access_token = jsonwebtoken::encode(&header, &access_claims, &encoding_key)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {e}")))?;

    let refresh_token = jsonwebtoken::encode(&header, &refresh_claims, &encoding_key)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {e}")))?;

    let response = AuthTokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt_maxage * 60,
        user: user.into(),
    };

    Ok(ApiSuccess::ok(response))
}

// ─── User CRUD handlers ─────────────────────────────────────────────

/// List all users (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default 20)"),
        ("search" = Option<String>, Query, description = "Search by email or name"),
    ),
    responses(
        (status = 200, description = "Paginated list of users"),
    ),
    tag = "Users"
)]
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let (users, total) = if let Some(ref search) = params.search {
        let pattern = format!("%{search}%");

        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email ILIKE $1 OR full_name ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE email ILIKE $1 OR full_name ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(&state.db)
        .await?;

        (users, total)
    } else {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&state.db)
            .await?;

        (users, total)
    };

    let user_responses: Vec<UserResponse> = users.into_iter().map(Into::into).collect();
    Ok(ApiPaginated::new(user_responses, total, page, per_page))
}

/// Get a single user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(("id" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 200, description = "User details"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {id} not found")))?;

    let response: UserResponse = user.into();
    Ok(ApiSuccess::ok(response))
}

/// Update a user
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    params(("id" = Uuid, Path, description = "User UUID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET
            full_name    = COALESCE($2, full_name),
            company      = COALESCE($3, company),
            phone_number = COALESCE($4, phone_number),
            timezone     = COALESCE($5, timezone),
            language     = COALESCE($6, language),
            avatar_url   = COALESCE($7, avatar_url),
            location     = COALESCE($8, location),
            updated_at   = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&body.full_name)
    .bind(&body.company)
    .bind(&body.phone_number)
    .bind(&body.timezone)
    .bind(&body.language)
    .bind(&body.avatar_url)
    .bind(&body.location)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {id} not found")))?;

    let response: UserResponse = user.into();
    Ok(ApiSuccess::ok(response))
}

/// Delete a user (soft-delete by deactivating)
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(("id" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result =
        sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&state.db)
            .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("User {id} not found")));
    }

    Ok(ApiMessage::deleted("User"))
}
