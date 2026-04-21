// src/apps/user/handlers.rs
//
// Business logic & API route handlers for the User app.
// Uses: models.rs (DB) + schemas.rs (I/O) + crate::response (envelopes)
//

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiMessage, ApiPaginated, ApiSuccess};
use crate::state::AppState;

use super::models::{AccessToken, RefreshToken, User};
use super::schemas::{
    ListUsersQuery, LoginRequest, LoginRefreshResponse, RegisterRequest, TokenPairResponse,
    UpdateUserRequest, UserResponse, VerifyTokenRequest, VerifyTokenResponse,
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

    let existing_sql = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE email = $1)",
        User::QUALIFIED_TABLE
    );
    let existing = sqlx::query_scalar::<_, bool>(&existing_sql)
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

    let insert_sql = format!(
        r#"
        INSERT INTO {} (
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
        User::QUALIFIED_TABLE
    );
    let user = sqlx::query_as::<_, User>(&insert_sql)
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

/// Login — returns only a refresh token.
/// Exchange it at POST /api/v1/auth/token/{refresh_token} to get an access token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful — returns refresh token"),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    axum::extract::ConnectInfo(client_addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(format!("Invalid login request: {}", e)))?;

    if body.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }
    if body.password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    let login_sql = format!(
        "SELECT * FROM {} WHERE email = $1 AND is_active = true",
        User::QUALIFIED_TABLE
    );
    let user = sqlx::query_as::<_, User>(&login_sql)
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

    let client_ip = client_addr.ip().to_string();

    let touch_login_sql = format!(
        "UPDATE {} SET login_count = login_count + 1, last_login_at = NOW(), last_login_ip = $2 WHERE id = $1",
        User::QUALIFIED_TABLE
    );
    sqlx::query(&touch_login_sql)
        .bind(user.id)
        .bind(&client_ip)
        .execute(&state.db)
        .await?;

    tracing::info!(
        user_id = %user.id,
        email   = %user.email.as_ref().unwrap_or(&String::new()),
        ip      = %client_ip,
        "User logged in — refresh token issued"
    );

    // Issue a refresh token only (access token is issued on /auth/token exchange)
    let config = &state.config;
    let now = chrono::Utc::now();
    let refresh_exp = now + chrono::Duration::days(7);

    let refresh_claims = serde_json::json!({
        "sub":  user.id.to_string(),
        "exp":  refresh_exp.timestamp(),
        "iat":  now.timestamp(),
        "jti":  Uuid::new_v4().to_string(),
        "type": "refresh",
    });

    let encoding_key = jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let refresh_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &refresh_claims,
        &encoding_key,
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {e}")))?;

    let insert_refresh_sql = format!(
        r#"INSERT INTO {} (id, refresh_token, expires_at, is_active, user_id, ip_address, created_at, updated_at)
           VALUES (gen_random_uuid(), $1, $2, true, $3, $4, NOW(), NOW())"#,
        RefreshToken::QUALIFIED_TABLE
    );
    sqlx::query(&insert_refresh_sql)
        .bind(&refresh_token)
        .bind(refresh_exp)
        .bind(user.id)
        .bind(&client_ip)
        .execute(&state.db)
        .await?;

    Ok(ApiSuccess::ok(LoginRefreshResponse {
        refresh_token,
        token_type: "Bearer".to_string(),
    }))
}

/// Exchange a refresh token for a new access + refresh token pair.
/// The old refresh token is rotated (invalidated) on every call.
#[utoipa::path(
    post,
    path = "/api/v1/auth/token/{refresh_token}",
    params(("refresh_token" = String, Path, description = "Refresh token from /auth/login")),
    responses(
        (status = 200, description = "New access + refresh token pair"),
        (status = 401, description = "Invalid or expired refresh token"),
    ),
    tag = "Authentication"
)]
pub async fn token_exchange(
    State(state): State<AppState>,
    axum::extract::ConnectInfo(client_addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(refresh_token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let client_ip = client_addr.ip().to_string();
    let config = &state.config;
    let now = chrono::Utc::now();

    // 1. Validate JWT signature + expiry
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let claims: serde_json::Value = jsonwebtoken::decode(
        &refresh_token,
        &decoding_key,
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?
    .claims;

    if claims.get("type").and_then(|t| t.as_str()) != Some("refresh") {
        return Err(AppError::Unauthorized("Expected a refresh token".to_string()));
    }

    // 2. Look up the refresh token record in DB
    let lookup_sql = format!(
        "SELECT * FROM {} WHERE refresh_token = $1 AND is_active = true AND expires_at > $2",
        RefreshToken::QUALIFIED_TABLE
    );
    let rt_record = sqlx::query_as::<_, RefreshToken>(&lookup_sql)
        .bind(&refresh_token)
        .bind(now)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Refresh token not found or expired".to_string()))?;

    // 3. Load the associated user
    let user_sql = format!(
        "SELECT * FROM {} WHERE id = $1 AND is_active = true",
        User::QUALIFIED_TABLE
    );
    let user = sqlx::query_as::<_, User>(&user_sql)
        .bind(rt_record.user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found or inactive".to_string()))?;

    let encoding_key = jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let header = jsonwebtoken::Header::default();

    // 4. Issue a new access token (short-lived)
    let access_exp = now + chrono::Duration::minutes(config.jwt_maxage);
    let access_claims = serde_json::json!({
        "sub":   user.id.to_string(),
        "email": user.email,
        "exp":   access_exp.timestamp(),
        "iat":   now.timestamp(),
        "type":  "access",
    });
    let access_token = jsonwebtoken::encode(&header, &access_claims, &encoding_key)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {e}")))?;

    // 5. Rotate refresh token — issue a new one, deactivate the old one
    let new_refresh_exp = now + chrono::Duration::days(7);
    let new_refresh_claims = serde_json::json!({
        "sub":  user.id.to_string(),
        "exp":  new_refresh_exp.timestamp(),
        "iat":  now.timestamp(),
        "jti":  Uuid::new_v4().to_string(),
        "type": "refresh",
    });
    let new_refresh_token = jsonwebtoken::encode(&header, &new_refresh_claims, &encoding_key)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {e}")))?;

    // Deactivate old refresh token
    sqlx::query(&format!(
        "UPDATE {} SET is_active = false, updated_at = NOW() WHERE id = $1",
        RefreshToken::QUALIFIED_TABLE
    ))
    .bind(rt_record.id)
    .execute(&state.db)
    .await?;

    // Insert new refresh token and get its ID (needed to link the access token)
    let new_rt_id: Uuid = sqlx::query_scalar(&format!(
        r#"INSERT INTO {} (id, refresh_token, expires_at, is_active, user_id, ip_address, rotated_from_id, created_at, updated_at)
           VALUES (gen_random_uuid(), $1, $2, true, $3, $4, $5, NOW(), NOW())
           RETURNING id"#,
        RefreshToken::QUALIFIED_TABLE
    ))
    .bind(&new_refresh_token)
    .bind(new_refresh_exp)
    .bind(user.id)
    .bind(&client_ip)
    .bind(rt_record.id)
    .fetch_one(&state.db)
    .await?;

    // 6. Persist access token to access_tokens table
    sqlx::query(&format!(
        r#"INSERT INTO {} (id, user_id, refresh_token_id, access_token, expires_at, is_active, is_single_use, created_at, updated_at)
           VALUES (gen_random_uuid(), $1, $2, $3, $4, true, false, NOW(), NOW())"#,
        AccessToken::QUALIFIED_TABLE
    ))
    .bind(user.id)
    .bind(new_rt_id)
    .bind(&access_token)
    .bind(access_exp)
    .execute(&state.db)
    .await?;

    tracing::info!(
        user_id = %user.id,
        ip      = %client_ip,
        "Token exchange — access + refresh tokens issued"
    );

    Ok(ApiSuccess::ok(TokenPairResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt_maxage * 60,
        user: user.into(),
    }))
}

/// Verify a token (access or refresh).
/// Checks JWT signature and expiry. Does NOT check DB revocation status.
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify",
    request_body = VerifyTokenRequest,
    responses(
        (status = 200, description = "Token is valid"),
        (status = 401, description = "Token is invalid or expired"),
    ),
    tag = "Authentication"
)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(body): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    let decoding_key =
        jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes());

    let token_data = jsonwebtoken::decode::<serde_json::Value>(
        &body.token,
        &decoding_key,
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    let claims = token_data.claims;
    let token_type = claims
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown")
        .to_string();

    // If caller specified an expected type, enforce it
    if let Some(ref expected) = body.token_type {
        if &token_type != expected {
            return Err(AppError::Unauthorized(format!(
                "Expected '{expected}' token but got '{token_type}'"
            )));
        }
    }

    Ok(ApiSuccess::ok(VerifyTokenResponse {
        valid: true,
        token_type,
        user_id: claims
            .get("sub")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        email: claims
            .get("email")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string()),
        expires_at: claims.get("exp").and_then(|e| e.as_i64()),
    }))
}

/// Return the currently authenticated user's profile.
/// Requires a valid Bearer access token — validated by the auth middleware.
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current user profile"),
        (status = 401, description = "Not authenticated"),
    ),
    security(("Bearer" = [])),
    tag = "Authentication"
)]
pub async fn me(
    Extension(AuthUser(user)): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    Ok(ApiSuccess::ok(UserResponse::from(user)))
}

// ─── User CRUD handlers ─────────────────────────────────────────────

/// List all users (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(
        ("page"     = Option<i64>, Query, description = "Page number (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default 20)"),
        ("search"   = Option<String>, Query, description = "Search by email or name"),
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

        let list_sql = format!(
            "SELECT * FROM {} WHERE email ILIKE $1 OR full_name ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            User::QUALIFIED_TABLE
        );
        let users = sqlx::query_as::<_, User>(&list_sql)
            .bind(&pattern)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await?;

        let count_sql = format!(
            "SELECT COUNT(*) FROM {} WHERE email ILIKE $1 OR full_name ILIKE $1",
            User::QUALIFIED_TABLE
        );
        let total = sqlx::query_scalar::<_, i64>(&count_sql)
            .bind(&pattern)
            .fetch_one(&state.db)
            .await?;

        (users, total)
    } else {
        let list_sql = format!(
            "SELECT * FROM {} ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            User::QUALIFIED_TABLE
        );
        let users = sqlx::query_as::<_, User>(&list_sql)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await?;

        let count_sql = format!("SELECT COUNT(*) FROM {}", User::QUALIFIED_TABLE);
        let total = sqlx::query_scalar::<_, i64>(&count_sql)
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
    let get_sql = format!("SELECT * FROM {} WHERE id = $1", User::QUALIFIED_TABLE);
    let user = sqlx::query_as::<_, User>(&get_sql)
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
    let update_sql = format!(
        r#"
        UPDATE {} SET
            full_name    = COALESCE($2, full_name),
            details      = COALESCE($3, details),
            company      = COALESCE($4, company),
            phone_number = COALESCE($5, phone_number),
            timezone     = COALESCE($6, timezone),
            language     = COALESCE($7, language),
            avatar_url   = COALESCE($8, avatar_url),
            location     = COALESCE($9, location),
            updated_at   = NOW()
        WHERE id = $1
        RETURNING *
        "#,
        User::QUALIFIED_TABLE
    );
    let user = sqlx::query_as::<_, User>(&update_sql)
        .bind(id)
        .bind(&body.full_name)
        .bind(&body.details)
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
    let delete_sql = format!(
        "UPDATE {} SET is_active = false, updated_at = NOW() WHERE id = $1",
        User::QUALIFIED_TABLE
    );
    let result = sqlx::query(&delete_sql).bind(id).execute(&state.db).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("User {id} not found")));
    }

    Ok(ApiMessage::deleted("User"))
}
