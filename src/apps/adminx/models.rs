use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct AdminActor {
    pub id: Uuid,
    pub is_active: bool,
    pub is_superuser: bool,
    pub is_staffuser: bool,
}

#[derive(Debug, FromRow)]
pub struct AdminDashboardMetrics {
    pub total_users: i64,
    pub active_users: i64,
    pub admin_users: i64,
    pub verified_users: i64,
    pub total_blog_posts: i64,
    pub published_blog_posts: i64,
    pub pending_comments: i64,
}

#[derive(Debug, FromRow)]
pub struct AdminManagedUser {
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
