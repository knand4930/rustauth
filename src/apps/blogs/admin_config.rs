use crate::admin::{AdminAppConfig, AdminCrudConfig, AdminEndpointConfig, AdminResourceConfig};

pub fn admin_config() -> AdminAppConfig {
    AdminAppConfig::new(
        "blogs",
        "Blogs",
        vec![AdminResourceConfig::new(
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
                AdminEndpointConfig::new("POST", "/api/blogs"),
                AdminEndpointConfig::new("GET", "/api/blogs/{id}"),
                AdminEndpointConfig::new("PUT", "/api/blogs/{id}"),
                AdminEndpointConfig::new("DELETE", "/api/blogs/{id}"),
            ),
        )],
    )
}
