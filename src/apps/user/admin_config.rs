use crate::admin::{AdminAppConfig, AdminCrudConfig, AdminEndpointConfig, AdminResourceConfig};

pub fn admin_config() -> AdminAppConfig {
    AdminAppConfig::new(
        "user",
        "Users",
        vec![AdminResourceConfig::new(
            "users",
            "Users",
            "User",
            vec![
                "id",
                "email",
                "full_name",
                "is_active",
                "is_staffuser",
                "created_at",
            ],
            vec![
                "is_active",
                "is_staffuser",
                "is_superuser",
                "email_verified",
            ],
            AdminCrudConfig::new(
                AdminEndpointConfig::new("POST", "/api/auth/register"),
                AdminEndpointConfig::new("GET", "/api/users/{id}"),
                AdminEndpointConfig::new("PUT", "/api/users/{id}"),
                AdminEndpointConfig::new("DELETE", "/api/users/{id}"),
            ),
        )],
    )
}
