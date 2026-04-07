// src/blogs/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String, // SEO
    pub author_id: Uuid,
    pub content: String,
    pub short_description: String,

    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub user_id: Option<Uuid>, // registered user
    pub guest_name: Option<String>,

    pub blog_post_id: Uuid,
    pub parent_id: Option<Uuid>, // nested comments

    pub content: String,
    pub is_approved: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
