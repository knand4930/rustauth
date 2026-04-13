use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::{
    apps::{
        blogs::{BlogPost, Comment},
        user::User,
    },
    error::AppError,
    response::ApiSuccess,
    state::AppState,
};

use super::auth::require_admin;

#[derive(Debug, FromRow)]
struct DashboardMetricsRow {
    total_users: i64,
    active_users: i64,
    admin_users: i64,
    verified_users: i64,
    total_blog_posts: i64,
    published_blog_posts: i64,
    pending_comments: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminDashboardResponse {
    pub site_title: String,
    pub site_header: String,
    pub registered_apps: usize,
    pub registered_resources: usize,
    pub total_users: i64,
    pub active_users: i64,
    pub admin_users: i64,
    pub verified_users: i64,
    pub total_blog_posts: i64,
    pub published_blog_posts: i64,
    pub pending_comments: i64,
}

#[utoipa::path(
    get,
    path = "/api/adminx/dashboard",
    params(
        ("Authorization" = String, Header, description = "Bearer admin access token"),
    ),
    responses(
        (status = 200, description = "Admin dashboard summary", body = AdminDashboardResponse),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Superuser privileges required"),
    ),
    tag = "AdminX"
)]
pub async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let actor = require_admin(&state, &headers).await?;
    tracing::debug!(admin_id = %actor.id, "Loading admin dashboard");

    let metrics_sql = format!(
        r#"
        SELECT
            (SELECT COUNT(*) FROM {users}) AS total_users,
            (SELECT COUNT(*) FROM {users} WHERE is_active = true) AS active_users,
            (
                SELECT COUNT(*)
                FROM {users}
                WHERE is_superuser = true OR is_staffuser = true
            ) AS admin_users,
            (
                SELECT COUNT(*)
                FROM {users}
                WHERE email_verified = true
            ) AS verified_users,
            (SELECT COUNT(*) FROM {blog_posts}) AS total_blog_posts,
            (
                SELECT COUNT(*)
                FROM {blog_posts}
                WHERE is_published = true
            ) AS published_blog_posts,
            (
                SELECT COUNT(*)
                FROM {comments}
                WHERE is_approved = false
            ) AS pending_comments
        "#,
        users = User::QUALIFIED_TABLE,
        blog_posts = BlogPost::QUALIFIED_TABLE,
        comments = Comment::QUALIFIED_TABLE,
    );
    let metrics = sqlx::query_as::<_, DashboardMetricsRow>(&metrics_sql)
        .fetch_one(&state.db)
        .await?;

    let response = AdminDashboardResponse {
        site_title: state.admin.site_title.clone(),
        site_header: state.admin.site_header.clone(),
        registered_apps: state.admin.app_count,
        registered_resources: state.admin.resource_count,
        total_users: metrics.total_users,
        active_users: metrics.active_users,
        admin_users: metrics.admin_users,
        verified_users: metrics.verified_users,
        total_blog_posts: metrics.total_blog_posts,
        published_blog_posts: metrics.published_blog_posts,
        pending_comments: metrics.pending_comments,
    };

    Ok(ApiSuccess::ok(response))
}
