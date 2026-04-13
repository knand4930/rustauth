use axum::http::{HeaderMap, header};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{apps::user::User, error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
struct AccessClaims {
    sub: String,
    #[serde(rename = "exp")]
    _exp: usize,
    #[serde(rename = "type")]
    token_type: String,
}

#[derive(Debug, FromRow)]
pub struct AdminActor {
    pub id: Uuid,
    pub is_active: bool,
    pub is_superuser: bool,
}

pub async fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<AdminActor, AppError> {
    let token = bearer_token_from_headers(headers)?;

    let validation = Validation::new(Algorithm::HS256);
    let token_data = jsonwebtoken::decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired access token".to_string()))?;

    if token_data.claims.token_type != "access" {
        return Err(AppError::Unauthorized(
            "Only access tokens can be used for admin endpoints".to_string(),
        ));
    }

    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;

    let actor_sql = format!(
        r#"
        SELECT id, is_active, is_superuser
        FROM {}
        WHERE id = $1
        "#,
        User::QUALIFIED_TABLE
    );
    let actor = sqlx::query_as::<_, AdminActor>(&actor_sql)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Admin account not found".to_string()))?;

    if !actor.is_active {
        return Err(AppError::Forbidden(
            "Inactive accounts cannot access admin endpoints".to_string(),
        ));
    }

    if !actor.is_superuser {
        return Err(AppError::Forbidden(
            "Superuser privileges are required for this endpoint".to_string(),
        ));
    }

    Ok(actor)
}

fn bearer_token_from_headers(headers: &HeaderMap) -> Result<&str, AppError> {
    let raw_header = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Authorization header is required".to_string()))?;

    let value = raw_header.to_str().map_err(|_| {
        AppError::Unauthorized("Authorization header must be valid UTF-8".to_string())
    })?;

    value.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized("Authorization header must use Bearer token".to_string())
    })
}
