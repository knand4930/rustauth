// src/blogs/mod.rs

pub mod handlers;
pub mod models;
pub mod schemas;

// Re-export key public types
pub use models::{BlogPost, Comment};
pub use schemas::{CreateBlogPostRequest, CreateCommentRequest, UpdateBlogPostRequest};

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/blogs", post(handlers::create_blog_post))
        .route("/api/v1/blogs", get(handlers::list_blog_posts))
        .route("/api/v1/blogs/{id}", get(handlers::get_blog_post))
        .route("/api/v1/blogs/{id}", put(handlers::update_blog_post))
        .route("/api/v1/blogs/{id}", delete(handlers::delete_blog_post))
        // Comments
        .route("/api/v1/blogs/{blog_id}/comments", post(handlers::create_comment))
        .route("/api/v1/blogs/{blog_id}/comments", get(handlers::list_comments))
}
