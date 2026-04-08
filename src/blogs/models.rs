// src/blogs/models.rs
//
// Database models ONLY — map 1:1 to PostgreSQL tables.
//

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ─── blog_posts ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub author_id: Uuid,
    pub content: String,
    pub short_description: String,

    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── comments ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Comment {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub guest_name: Option<String>,

    pub blog_post_id: Uuid,
    pub parent_id: Option<Uuid>,

    pub content: String,
    pub is_approved: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
