// src/blogs/mod.rs

pub mod handler;
pub mod models;
pub mod schema;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

/// Build the blog-related routes.
pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/api/v1/blogs", post(handler::create_blog_post))
        .route("/api/v1/blogs", get(handler::list_blog_posts))
        .route("/api/v1/blogs/{id}", get(handler::get_blog_post))
        .route("/api/v1/blogs/{id}", put(handler::update_blog_post))
        .route("/api/v1/blogs/{id}", delete(handler::delete_blog_post))
        // Comments
        .route("/api/v1/blogs/{blog_id}/comments", post(handler::create_comment))
        .route("/api/v1/blogs/{blog_id}/comments", get(handler::list_comments))
}
