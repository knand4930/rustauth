// src/apps/user/models.rs
// @schema user
//
// Database models ONLY — these map 1:1 to PostgreSQL tables.
// Do NOT put request/response DTOs here; those belong in schema.rs.
//

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ─── users ───────────────────────────────────────────────────────────

// @table users
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    // @unique
    // @validate email
    pub email: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    #[serde(skip_serializing)]
    pub store_password: Option<String>,
    pub full_name: Option<String>,
    pub company: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    // @default 'UTC'
    pub timezone: String,
    // @default 'en'
    pub language: String,
    #[serde(skip_serializing)]
    pub salt: Option<String>,

    pub location: Option<String>,
    pub ipaddress: Option<String>,

    // @index
    // @default true
    pub is_active: bool,
    // @default false
    pub is_superuser: bool,
    // @default false
    pub is_staffuser: bool,
    // @default false
    pub is_guest: Option<bool>,
    // @default false
    pub email_verified: bool,
    // @default false
    pub phone_verified: bool,
    // @default false
    pub mfa_enabled: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,

    #[schema(value_type = Option<Vec<String>>)]
    pub backup_codes: Option<Vec<String>>,
    #[schema(value_type = Option<serde_json::Value>)]
    pub preferences: Option<serde_json::Value>,

    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    // @default 0
    pub login_count: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(User, "user", "users");

// ─── refresh_tokens ──────────────────────────────────────────────────

// @table refresh_tokens
// @index columns=user_id,is_active
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct RefreshToken {
    pub id: Uuid,
    // @unique
    pub refresh_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    // @default true
    pub is_active: bool,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub device_fingerprint: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    // @references self
    pub rotated_from_id: Option<Uuid>,
    // @references user.users
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(RefreshToken, "user", "refresh_tokens");

// ─── access_tokens ───────────────────────────────────────────────────

// @table access_tokens
// @index columns=user_id,is_active
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct AccessToken {
    pub id: Uuid,
    // @references user.users
    pub user_id: Uuid,
    // @references user.refresh_tokens
    pub refresh_token_id: Uuid,
    // @unique
    pub access_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    // @default true
    pub is_active: bool,
    pub last_used: Option<DateTime<Utc>>,
    // @default false
    pub is_single_use: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(AccessToken, "user", "access_tokens");

// ─── token_blacklist ─────────────────────────────────────────────────

// @table token_blacklists
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct TokenBlacklist {
    pub id: Uuid,
    // @unique
    pub token_jti: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(TokenBlacklist, "user", "token_blacklists");

// ─── password_reset_tokens ───────────────────────────────────────────

// @table password_reset_tokens
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct PasswordResetToken {
    pub id: Uuid,
    // @references user.users
    pub user_id: Option<Uuid>,
    // @unique
    pub token_hash: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    // @default false
    pub is_used: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(PasswordResetToken, "user", "password_reset_tokens");

// ─── user_sessions ───────────────────────────────────────────────────

// @table user_sessions
// @index columns=user_id,is_active
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserSession {
    pub id: Uuid,
    // @references user.users
    pub user_id: Option<Uuid>,
    // @unique
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    // @default true
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(UserSession, "user", "user_sessions");

// ─── permissions ─────────────────────────────────────────────────────

// @table permissions
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Permission {
    pub id: Uuid,
    // @unique
    pub name: Option<String>,
    // @default true
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(Permission, "user", "permissions");

// ─── user_roles ──────────────────────────────────────────────────────

// @table user_roles
// @unique columns=user_id,role_id
// @index columns=role_id,is_active
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserRole {
    pub id: Uuid,
    // @references user.users
    pub user_id: Uuid,
    // @references user.roles
    pub role_id: Uuid,
    // @references user.users
    pub assigned_by_id: Option<Uuid>,
    // @default true
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(UserRole, "user", "user_roles");

// ─── roles ───────────────────────────────────────────────────────────

// @table roles
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Role {
    pub id: Uuid,
    // @unique
    pub name: String,
    pub description: Option<String>,
    // @default true
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(Role, "user", "roles");

// ─── role_permissions ────────────────────────────────────────────────

// @table role_permissions
// @unique columns=role_id,permission_id
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct RolePermission {
    pub id: Uuid,
    // @references user.roles
    pub role_id: Uuid,
    // @references user.permissions
    pub permission_id: Uuid,
    // @default false
    pub can_read: bool,
    // @default false
    pub can_write: bool,
    // @default false
    pub can_delete: bool,
    pub created_at: DateTime<Utc>,
}

crate::declare_model_table!(RolePermission, "user", "role_permissions");
