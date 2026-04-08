// src/user/models.rs
//
// Database models ONLY — these map 1:1 to PostgreSQL tables.
// Do NOT put request/response DTOs here; those belong in schema.rs.
//

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ─── users ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    #[serde(skip_serializing)]
    pub store_password: Option<String>,
    pub full_name: Option<String>,
    pub company: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub timezone: String,
    pub language: String,
    #[serde(skip_serializing)]
    pub salt: Option<String>,

    pub location: Option<String>,
    pub ipaddress: Option<String>,

    pub is_active: bool,
    pub is_superuser: bool,
    pub is_staffuser: bool,
    pub is_guest: Option<bool>,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub mfa_enabled: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,

    #[schema(value_type = Option<Vec<String>>)]
    pub backup_codes: Option<Vec<String>>,
    #[schema(value_type = Option<serde_json::Value>)]
    pub preferences: Option<serde_json::Value>,

    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub login_count: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── refresh_tokens ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct RefreshToken {
    pub id: Uuid,
    pub refresh_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_fingerprint: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub rotated_from_id: Option<Uuid>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── access_tokens ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct AccessToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_id: Uuid,
    pub access_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub last_used: Option<DateTime<Utc>>,
    pub is_single_use: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── token_blacklist ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct TokenBlacklist {
    pub id: Uuid,
    pub token_jti: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── password_reset_tokens ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub token_hash: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── user_sessions ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── permissions ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Permission {
    pub id: Uuid,
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── user_roles ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_by_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── roles ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── role_permissions ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct RolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub can_read: bool,
    pub can_write: bool,
    pub can_delete: bool,
    pub created_at: DateTime<Utc>,
}
