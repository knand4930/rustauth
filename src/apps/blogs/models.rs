// src/apps/blogs/models.rs
// @schema blogs
//
// Database models ONLY — map 1:1 to PostgreSQL tables.
//

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ─── blog_posts ──────────────────────────────────────────────────────

// @table blog_posts
// @index columns=author_id,is_published
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    // @unique
    pub slug: String,
    // @references user.users
    pub author_id: Uuid,
    pub content: String,
    pub short_description: String,

    // @default false
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(BlogPost, "blogs", "blog_posts");

// ─── comments ────────────────────────────────────────────────────────

// @table comments
// @index columns=blog_post_id,is_approved
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Comment {
    pub id: Uuid,
    // @references user.users
    pub user_id: Option<Uuid>,
    pub guest_name: Option<String>,

    // @references blogs.blog_posts
    pub blog_post_id: Uuid,
    // @references self
    pub parent_id: Option<Uuid>,

    pub content: String,
    // @default false
    pub is_approved: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

crate::declare_model_table!(Comment, "blogs", "comments");
