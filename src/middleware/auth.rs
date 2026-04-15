// src/middleware/auth.rs
//
// Global auth middleware.
//
// Flow:
//   public path  → pass through, no auth check
//   private path → validate Bearer access token (JWT + DB)
//                  ✓ valid   → inject AuthUser into extensions, continue
//                  ✗ invalid → 401 immediately
//
// Handlers on protected routes extract the user via:
//   Extension(AuthUser(user)): Extension<AuthUser>

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde_json::json;

use crate::{
    apps::user::models::{AccessToken, User},
    state::AppState,
};

/// Injected into request extensions by this middleware.
/// Protected handlers extract it with `Extension<AuthUser>`.
#[derive(Clone, Debug)]
pub struct AuthUser(pub User);

/// Paths that bypass authentication.
/// Everything else requires a valid Bearer access token.
fn is_public(path: &str) -> bool {
    let p = path.trim_end_matches('/');
    matches!(
        p,
        "" | "/api/v1/health"
            | "/api/v1/auth/login"
            | "/api/v1/auth/register"
            | "/api/v1/auth/verify"
    ) || p.starts_with("/api/v1/auth/token/")
        || p.starts_with("/api/v1/users")
        || p.starts_with("/api/v1/blogs")
        || p.starts_with("/swagger-ui")
        || p.starts_with("/api-docs")
        || p.starts_with("/adminx")
}

/// The middleware function — registered via `axum::middleware::from_fn_with_state`.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    if is_public(req.uri().path()) {
        return next.run(req).await;
    }

    // Extract Bearer token
    let token = match extract_bearer(&req) {
        Some(t) => t,
        None => return err_401("Missing Authorization header"),
    };

    // Fast path: validate JWT signature + expiry (no DB)
    let decoding_key =
        jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes());
    let validation = jsonwebtoken::Validation::default(); // validates `exp`

    let claims: serde_json::Value =
        match jsonwebtoken::decode(&token, &decoding_key, &validation) {
            Ok(td) => td.claims,
            Err(_) => return err_401("Invalid or expired access token"),
        };

    if claims.get("type").and_then(|t| t.as_str()) != Some("access") {
        return err_401("Expected an access token, got a different token type");
    }

    // DB validation: token must exist, be active, and belong to an active user
    let now = Utc::now();
    let sql = format!(
        r#"SELECT u.* FROM {at} at
           JOIN {ut} u ON u.id = at.user_id
           WHERE at.access_token = $1
             AND at.is_active    = true
             AND at.expires_at   > $2
             AND u.is_active     = true"#,
        at = AccessToken::QUALIFIED_TABLE,
        ut = User::QUALIFIED_TABLE,
    );

    match sqlx::query_as::<_, User>(&sql)
        .bind(&token)
        .bind(now)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(user)) => {
            // Fire-and-forget: update last_used without blocking the response
            {
                let db = state.db.clone();
                let tok = token.clone();
                let at_table = AccessToken::QUALIFIED_TABLE;
                tokio::spawn(async move {
                    let _ = sqlx::query(&format!(
                        "UPDATE {at_table} SET last_used = NOW() WHERE access_token = $1"
                    ))
                    .bind(&tok)
                    .execute(&db)
                    .await;
                });
            }

            req.extensions_mut().insert(AuthUser(user));
            next.run(req).await
        }
        Ok(None) => err_401("Access token not found or revoked"),
        Err(e) => {
            tracing::error!("Auth middleware DB error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

fn extract_bearer(req: &Request) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn err_401(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"success": false, "error": msg})),
    )
        .into_response()
}
