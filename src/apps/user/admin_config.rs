use crate::admin::{AdminAppConfig, AdminCrudConfig, AdminEndpointConfig, AdminResourceConfig};

pub fn admin_config() -> AdminAppConfig {
    AdminAppConfig::new(
        "user",
        "Users",
        vec![
            users_resource(),
            refresh_tokens_resource(),
            access_tokens_resource(),
            token_blacklists_resource(),
            password_reset_tokens_resource(),
            user_sessions_resource(),
            permissions_resource(),
            user_roles_resource(),
            roles_resource(),
            role_permissions_resource(),
        ],
    )
}

fn users_resource() -> AdminResourceConfig {
    AdminResourceConfig::new(
        "users",
        "Users",
        "User",
        vec![
            "id",
            "email",
            "full_name",
            "is_active",
            "is_staffuser",
            "is_superuser",
            "created_at",
        ],
        vec![
            "is_active",
            "is_staffuser",
            "is_superuser",
            "email_verified",
            "phone_verified",
            "mfa_enabled",
        ],
        user_crud(),
    )
}

fn refresh_tokens_resource() -> AdminResourceConfig {
    resource(
        "refresh_tokens",
        "Refresh Tokens",
        "RefreshToken",
        vec![
            "id",
            "user_id",
            "is_active",
            "expires_at",
            "last_used_at",
            "created_at",
        ],
        vec!["user_id", "is_active", "expires_at", "rotated_from_id"],
    )
}

fn access_tokens_resource() -> AdminResourceConfig {
    resource(
        "access_tokens",
        "Access Tokens",
        "AccessToken",
        vec![
            "id",
            "user_id",
            "refresh_token_id",
            "is_active",
            "is_single_use",
            "expires_at",
            "created_at",
        ],
        vec!["user_id", "refresh_token_id", "is_active", "is_single_use"],
    )
}

fn token_blacklists_resource() -> AdminResourceConfig {
    resource(
        "token_blacklists",
        "Token Blacklists",
        "TokenBlacklist",
        vec!["id", "token_jti", "expires_at", "created_at", "updated_at"],
        vec!["expires_at", "created_at"],
    )
}

fn password_reset_tokens_resource() -> AdminResourceConfig {
    resource(
        "password_reset_tokens",
        "Password Reset Tokens",
        "PasswordResetToken",
        vec![
            "id",
            "user_id",
            "token_hash",
            "is_used",
            "expires_at",
            "created_at",
        ],
        vec!["user_id", "is_used", "expires_at"],
    )
}

fn user_sessions_resource() -> AdminResourceConfig {
    resource(
        "user_sessions",
        "User Sessions",
        "UserSession",
        vec![
            "id",
            "user_id",
            "session_token",
            "ip_address",
            "is_active",
            "expires_at",
            "created_at",
        ],
        vec!["user_id", "is_active", "expires_at"],
    )
}

fn permissions_resource() -> AdminResourceConfig {
    resource(
        "permissions",
        "Permissions",
        "Permission",
        vec!["id", "name", "is_active", "created_at", "updated_at"],
        vec!["is_active", "created_at"],
    )
}

fn user_roles_resource() -> AdminResourceConfig {
    resource(
        "user_roles",
        "User Roles",
        "UserRole",
        vec![
            "id",
            "user_id",
            "role_id",
            "assigned_by_id",
            "is_active",
            "created_at",
        ],
        vec!["user_id", "role_id", "assigned_by_id", "is_active"],
    )
}

fn roles_resource() -> AdminResourceConfig {
    resource(
        "roles",
        "Roles",
        "Role",
        vec!["id", "name", "description", "is_active", "created_at"],
        vec!["is_active", "created_at"],
    )
}

fn role_permissions_resource() -> AdminResourceConfig {
    resource(
        "role_permissions",
        "Role Permissions",
        "RolePermission",
        vec![
            "id",
            "role_id",
            "permission_id",
            "can_read",
            "can_write",
            "can_delete",
            "created_at",
        ],
        vec![
            "role_id",
            "permission_id",
            "can_read",
            "can_write",
            "can_delete",
        ],
    )
}

fn resource(
    key: &'static str,
    label: &'static str,
    model: &'static str,
    list_display: Vec<&'static str>,
    list_filter: Vec<&'static str>,
) -> AdminResourceConfig {
    AdminResourceConfig::new(key, label, model, list_display, list_filter, crud_for(key))
}

fn user_crud() -> AdminCrudConfig {
    AdminCrudConfig::new(
        AdminEndpointConfig::new("POST", "/api/auth/register"),
        AdminEndpointConfig::new("GET", "/api/users/{id}"),
        AdminEndpointConfig::new("PUT", "/api/users/{id}"),
        AdminEndpointConfig::new("DELETE", "/api/users/{id}"),
    )
}

fn crud_for(resource_key: &str) -> AdminCrudConfig {
    let collection_path = format!("/api/{resource_key}");
    let detail_path = format!("{collection_path}/{{id}}");

    AdminCrudConfig::new(
        AdminEndpointConfig::new("POST", collection_path.clone()),
        AdminEndpointConfig::new("GET", detail_path.clone()),
        AdminEndpointConfig::new("PUT", detail_path.clone()),
        AdminEndpointConfig::new("DELETE", detail_path),
    )
}
