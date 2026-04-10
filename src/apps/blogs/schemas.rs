// src/apps/blogs/schemas.rs
//
// Request & Response DTOs for the Blogs app.
//

use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Request schemas
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// POST /api/v1/blogs
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBlogPostRequest {
    #[validate(length(min = 1, message = "Title cannot be empty"))]
    pub title: String,
    pub content: String,
    pub short_description: Option<String>,
    pub author_id: Uuid,
    pub is_published: Option<bool>,
}

/// PUT /api/v1/blogs/{id}
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateBlogPostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub short_description: Option<String>,
    pub is_published: Option<bool>,
}

/// POST /api/v1/blogs/{blog_id}/comments
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCommentRequest {
    pub content: String,
    pub user_id: Option<Uuid>,
    pub guest_name: Option<String>,
    pub parent_id: Option<Uuid>,
}

/// GET /api/v1/blogs  (query params)
#[derive(Debug, Deserialize)]
pub struct ListBlogsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub published_only: Option<bool>,
}
