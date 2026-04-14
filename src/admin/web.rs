use axum::{
    extract::{Json, State},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{apps::user::User, error::AppError, state::AppState};
use super::api;

/// Admin session stored in cookie
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminSession {
    pub user_id: Uuid,
    pub username: String,
    pub is_superuser: bool,
    pub expires: i64,
}

const SESSION_COOKIE_NAME: &str = "adminx_session";
const SESSION_DURATION_SECS: i64 = 3600 * 8; // 8 hours

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// Serve the Single Page Application
pub async fn spa_handler(_jar: CookieJar, State(_state): State<AppState>) -> impl IntoResponse {
    let index_html = include_str!("templates/index.html");
    Html(index_html.to_string()).into_response()
}

/// Check current session status
pub async fn auth_me(jar: CookieJar, State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    if let Some(session) = get_session(&jar, &state) {
        Ok(Json(json!({"authenticated": true, "user": session})))
    } else {
        Err(AppError::Unauthorized("Not authenticated".into()))
    }
}

/// Login API endpoint
pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(form): Json<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let user = authenticate_admin(&state, &form.username, &form.password).await?;
    
    let session = AdminSession {
        user_id: user.id,
        username: user.username,
        is_superuser: user.is_superuser,
        expires: Utc::now().timestamp() + SESSION_DURATION_SECS,
    };

    let session_json = serde_json::to_string(&session).unwrap_or_default();
    let encoded = base64_encode(&session_json);

    let cookie = Cookie::build((SESSION_COOKIE_NAME, encoded))
        .path("/adminx")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(SESSION_DURATION_SECS))
        .build();

    let jar = jar.add(cookie);

    Ok((jar, Json(json!({"status": "success", "user": session}))))
}

/// Logout API endpoint
pub async fn logout_handler(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/adminx")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(-1))
        .build();
    let jar = jar.add(cookie);
    (jar, Json(json!({"status": "success"}))).into_response()
}

/// Web and API routes
pub fn web_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Authenticated API group
        .route("/adminx/api/config", get(api::config_handler))
        .route("/adminx/api/dashboard", get(api::dashboard_handler))
        .route("/adminx/api/records/{app}/{resource}", get(api::list_records_handler).post(api::create_record_handler))
        .route("/adminx/api/records/{app}/{resource}/{id}", get(api::get_record_handler).put(api::update_record_handler).delete(api::delete_record_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            require_admin_auth_middleware
        ))
        // Public Auth API
        .route("/adminx/api/auth/me", get(auth_me))
        .route("/adminx/api/auth/login", post(login_handler))
        .route("/adminx/api/auth/logout", post(logout_handler))
        // SPA Fallback routes - match anything starting with /adminx that isn't API
        .route("/adminx", get(spa_handler))
        .route("/adminx/", get(spa_handler))
        .route("/adminx/{*rest}", get(spa_handler))
}

pub async fn require_admin_auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, AppError> {
    let session = require_admin_session(&state, &jar)?;
    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}

// Helper functions

pub fn get_session(jar: &CookieJar, _state: &AppState) -> Option<AdminSession> {
    let cookie = jar.get(SESSION_COOKIE_NAME)?;
    let decoded = base64_decode(cookie.value()).ok()?;
    let session: AdminSession = serde_json::from_str(&decoded).ok()?;

    if session.expires < Utc::now().timestamp() {
        return None;
    }

    Some(session)
}

pub fn require_admin_session(state: &AppState, jar: &CookieJar) -> Result<AdminSession, AppError> {
    get_session(jar, state)
        .ok_or_else(|| AppError::Unauthorized("Please log in to access the admin panel".to_string()))
}

async fn authenticate_admin(state: &AppState, username: &str, password: &str) -> Result<AuthenticatedUser, AppError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use sqlx::FromRow;

    #[derive(FromRow)]
    struct UserCredentials {
        id: Uuid,
        username: Option<String>,
        #[allow(dead_code)]
        email: String,
        password: Option<String>,
        is_active: bool,
        is_superuser: bool,
    }

    let user = sqlx::query_as::<_, UserCredentials>(
        &format!(
            "SELECT id, full_name as username, email, password, is_active, is_superuser FROM {} WHERE (email = $1)",
            User::QUALIFIED_TABLE
        )
    )
    .bind(username) // Admin login uses email
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !user.is_active {
        return Err(AppError::Forbidden("Account is inactive".to_string()));
    }

    if !user.is_superuser {
        return Err(AppError::Forbidden("Superuser privileges required".to_string()));
    }

    // Verify password
    let hash_str = user.password.unwrap_or_default();
    let parsed_hash = PasswordHash::new(&hash_str)
        .map_err(|_| AppError::Internal("Invalid password hash format".to_string()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

    Ok(AuthenticatedUser {
        id: user.id,
        username: user.username.unwrap_or_else(|| "Admin".to_string()),
        is_superuser: user.is_superuser,
    })
}

struct AuthenticatedUser {
    id: Uuid,
    username: String,
    is_superuser: bool,
}

fn base64_encode(input: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(input)
}

fn base64_decode(input: &str) -> Result<String, base64::DecodeError> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let decoded = STANDARD.decode(input)?;
    String::from_utf8(decoded).map_err(|_| base64::DecodeError::InvalidByte(0, b'\0'))
}
