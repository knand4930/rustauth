use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::models::{AdminDashboardMetrics, AdminManagedUser};

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
    pub fn is_empty(&self) -> bool {
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

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminDashboardResponse {
    pub total_users: i64,
    pub active_users: i64,
    pub admin_users: i64,
    pub verified_users: i64,
    pub total_blog_posts: i64,
    pub published_blog_posts: i64,
    pub pending_comments: i64,
}

impl From<AdminDashboardMetrics> for AdminDashboardResponse {
    fn from(metrics: AdminDashboardMetrics) -> Self {
        Self {
            total_users: metrics.total_users,
            active_users: metrics.active_users,
            admin_users: metrics.admin_users,
            verified_users: metrics.verified_users,
            total_blog_posts: metrics.total_blog_posts,
            published_blog_posts: metrics.published_blog_posts,
            pending_comments: metrics.pending_comments,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
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
