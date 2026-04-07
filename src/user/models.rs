// src/user/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub password: Option<String>,
    pub store_password: Option<String>,
    pub full_name: Option<String>,
    pub company: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub timezone: String,
    pub language: String,
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
    pub mfa_secret: Option<String>,

    pub backup_codes: Option<Vec<String>>, // PostgreSQL ARRAY
    pub preferences: Option<serde_json::Value>, // JSON

    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub login_count: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenBlacklist {
    pub id: Uuid,
    pub token_jti: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub token_hash: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_used: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub name: Option<String>,
    pub is_active: Option<bool>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,

    pub assigned_by_id: Option<Uuid>,
    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,

    pub can_read: bool,
    pub can_write: bool,
    pub can_delete: bool,

    pub created_at: DateTime<Utc>,
}
