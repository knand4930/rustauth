use crate::admin::{AdminAppConfig, AdminCrudConfig, AdminEndpointConfig, AdminResourceConfig};

pub fn admin_config() -> AdminAppConfig {
    AdminAppConfig::new(
        "blogs",
        "Blogs",
        vec![blog_posts_resource(), comments_resource()],
    )
}

fn blog_posts_resource() -> AdminResourceConfig {
    AdminResourceConfig::new(
        "blog_posts",
        "Blog Posts",
        "BlogPost",
        vec![
            "id",
            "title",
            "slug",
            "author_id",
            "is_published",
            "published_at",
            "created_at",
        ],
        vec!["author_id", "is_published", "published_at"],
        AdminCrudConfig::new(
            AdminEndpointConfig::new("POST", "/api/v1/blogs"),
            AdminEndpointConfig::new("GET", "/api/v1/blogs/{id}"),
            AdminEndpointConfig::new("PUT", "/api/v1/blogs/{id}"),
            AdminEndpointConfig::new("DELETE", "/api/v1/blogs/{id}"),
        ),
    )
}

fn comments_resource() -> AdminResourceConfig {
    AdminResourceConfig::new(
        "comments",
        "Comments",
        "Comment",
        vec![
            "id",
            "blog_post_id",
            "user_id",
            "guest_name",
            "is_approved",
            "created_at",
        ],
        vec!["blog_post_id", "user_id", "parent_id", "is_approved"],
        AdminCrudConfig::new(
            AdminEndpointConfig::new("POST", "/api/v1/blogs/{blog_id}/comments"),
            AdminEndpointConfig::new("GET", "/api/v1/blogs/{blog_id}/comments/{id}"),
            AdminEndpointConfig::new("PUT", "/api/v1/blogs/{blog_id}/comments/{id}"),
            AdminEndpointConfig::new("DELETE", "/api/v1/blogs/{blog_id}/comments/{id}"),
        ),
    )
}
