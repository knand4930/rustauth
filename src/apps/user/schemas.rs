// src/apps/user/schemas.rs
//
// Request & Response DTOs for the User app.
// This is the data-contract layer (I/O) — separated from DB models and handlers.
//

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::models::User;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Request schemas
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// POST /api/v1/auth/register
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub full_name: Option<String>,
    pub company: Option<String>,
    pub phone_number: Option<String>,
}

/// POST /api/v1/auth/login
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    pub password: String,
}

/// PUT /api/v1/users/{id}
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub details: Option<String>,
    pub company: Option<String>,
    pub phone_number: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
}

/// GET /api/v1/users  (query params)
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

/// POST /api/v1/auth/verify
#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyTokenRequest {
    pub token: String,
    /// Optional: "access" or "refresh". If set, the endpoint enforces the type.
    pub token_type: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Response schemas
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Public-facing user (no sensitive fields).
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub company: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub timezone: String,
    pub language: String,
    pub is_active: bool,
    pub is_superuser: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            company: u.company,
            avatar_url: u.avatar_url,
            phone_number: u.phone_number,
            timezone: u.timezone,
            language: u.language,
            is_active: u.is_active,
            is_superuser: u.is_superuser,
            email_verified: u.email_verified,
            created_at: u.created_at,
        }
    }
}

/// POST /api/v1/auth/login
/// Login issues only a refresh token. Use /api/v1/auth/token/{refresh_token} to get an access token.
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginRefreshResponse {
    pub refresh_token: String,
    pub token_type: String,
}

/// POST /api/v1/auth/token/{refresh_token}
/// Validates the refresh token, rotates it, and issues a new access + refresh pair.
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    /// Access token lifetime in seconds
    pub expires_in: i64,
    pub user: UserResponse,
}

/// POST /api/v1/auth/verify
#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyTokenResponse {
    pub valid: bool,
    pub token_type: String,
    pub user_id: Option<String>,
    pub email: Option<String>,
    /// Unix timestamp of token expiry
    pub expires_at: Option<i64>,
}
