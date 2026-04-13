// src/apps/blogs/mod.rs

pub mod admin_config;
pub mod admin_registry;
pub mod handlers;
pub mod models;
pub mod schemas;

// Re-export key public types
pub use models::{BlogPost, Comment};
pub use schemas::{CreateBlogPostRequest, CreateCommentRequest, UpdateBlogPostRequest};

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{admin::AdminPanelBuilder, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/blogs", post(handlers::create_blog_post))
        .route("/api/blogs", get(handlers::list_blog_posts))
        .route("/api/blogs/{id}", get(handlers::get_blog_post))
        .route("/api/blogs/{id}", put(handlers::update_blog_post))
        .route("/api/blogs/{id}", delete(handlers::delete_blog_post))
        // Comments
        .route(
            "/api/v1/blogs/{blog_id}/comments",
            post(handlers::create_comment),
        )
        .route(
            "/api/v1/blogs/{blog_id}/comments",
            get(handlers::list_comments),
        )
}

pub fn register_admin(builder: &mut AdminPanelBuilder) {
    admin_registry::register(builder);
}
