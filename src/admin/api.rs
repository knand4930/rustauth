use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    apps::{
        blogs::models::{BlogPost, Comment},
        user::models::{
            AccessToken, PasswordResetToken, Permission, RefreshToken, Role, RolePermission,
            TokenBlacklist, User, UserRole, UserSession,
        },
    },
    error::AppError,
    state::AppState,
};

#[derive(serde::Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

pub async fn config_handler(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(Json(state.admin.clone()))
}

pub async fn dashboard_handler(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    // Attempt to get meaningful metrics from the registered resources
    // For now, we aggregate core counts. In a real system, we'd iterate over all resources.
    let total_users: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", User::QUALIFIED_TABLE)).fetch_one(&state.db).await.unwrap_or(0);
    let active_users: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {} WHERE is_active = true", User::QUALIFIED_TABLE)).fetch_one(&state.db).await.unwrap_or(0);
    let blog_posts: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", BlogPost::QUALIFIED_TABLE)).fetch_one(&state.db).await.unwrap_or(0);
    let comments: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", Comment::QUALIFIED_TABLE)).fetch_one(&state.db).await.unwrap_or(0);

    Ok(Json(json!({
        "total_users": total_users,
        "active_users": active_users,
        "blog_posts": blog_posts,
        "comments": comments,
        "app_count": state.admin.app_count,
        "resource_count": state.admin.resource_count,
    })))
}

fn get_table_name(app: &str, resource: &str) -> Result<&'static str, AppError> {
    match (app, resource) {
        ("user", "users") => Ok(User::QUALIFIED_TABLE),
        ("user", "refresh_tokens") => Ok(RefreshToken::QUALIFIED_TABLE),
        ("user", "access_tokens") => Ok(AccessToken::QUALIFIED_TABLE),
        ("user", "token_blacklists") => Ok(TokenBlacklist::QUALIFIED_TABLE),
        ("user", "password_reset_tokens") => Ok(PasswordResetToken::QUALIFIED_TABLE),
        ("user", "user_sessions") => Ok(UserSession::QUALIFIED_TABLE),
        ("user", "permissions") => Ok(Permission::QUALIFIED_TABLE),
        ("user", "user_roles") => Ok(UserRole::QUALIFIED_TABLE),
        ("user", "roles") => Ok(Role::QUALIFIED_TABLE),
        ("user", "role_permissions") => Ok(RolePermission::QUALIFIED_TABLE),
        ("blogs", "blog_posts") => Ok(BlogPost::QUALIFIED_TABLE),
        ("blogs", "comments") => Ok(Comment::QUALIFIED_TABLE),
        _ => Err(AppError::NotFound(format!("Resource {}/{} not found", app, resource))),
    }
}

pub async fn list_records_handler(
    State(state): State<AppState>,
    Path((app, resource)): Path<(String, String)>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let search = params.search.clone();

    let table = get_table_name(&app, &resource)?;

    match (app.as_str(), resource.as_str()) {
        ("user", "users") => handle_list::<User>(&state, table, per_page, offset, search).await,
        ("user", "refresh_tokens") => handle_list::<RefreshToken>(&state, table, per_page, offset, search).await,
        ("user", "access_tokens") => handle_list::<AccessToken>(&state, table, per_page, offset, search).await,
        ("user", "token_blacklists") => handle_list::<TokenBlacklist>(&state, table, per_page, offset, search).await,
        ("user", "password_reset_tokens") => handle_list::<PasswordResetToken>(&state, table, per_page, offset, search).await,
        ("user", "user_sessions") => handle_list::<UserSession>(&state, table, per_page, offset, search).await,
        ("user", "permissions") => handle_list::<Permission>(&state, table, per_page, offset, search).await,
        ("user", "user_roles") => handle_list::<UserRole>(&state, table, per_page, offset, search).await,
        ("user", "roles") => handle_list::<Role>(&state, table, per_page, offset, search).await,
        ("user", "role_permissions") => handle_list::<RolePermission>(&state, table, per_page, offset, search).await,
        ("blogs", "blog_posts") => handle_list::<BlogPost>(&state, table, per_page, offset, search).await,
        ("blogs", "comments") => handle_list::<Comment>(&state, table, per_page, offset, search).await,
        _ => unreachable!(),
    }
}

pub async fn get_record_handler(
    State(state): State<AppState>,
    Path((app, resource, id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let table = get_table_name(&app, &resource)?;

    match (app.as_str(), resource.as_str()) {
        ("user", "users") => handle_get::<User>(&state, table, id).await,
        ("user", "refresh_tokens") => handle_get::<RefreshToken>(&state, table, id).await,
        ("user", "access_tokens") => handle_get::<AccessToken>(&state, table, id).await,
        ("user", "token_blacklists") => handle_get::<TokenBlacklist>(&state, table, id).await,
        ("user", "password_reset_tokens") => handle_get::<PasswordResetToken>(&state, table, id).await,
        ("user", "user_sessions") => handle_get::<UserSession>(&state, table, id).await,
        ("user", "permissions") => handle_get::<Permission>(&state, table, id).await,
        ("user", "user_roles") => handle_get::<UserRole>(&state, table, id).await,
        ("user", "roles") => handle_get::<Role>(&state, table, id).await,
        ("user", "role_permissions") => handle_get::<RolePermission>(&state, table, id).await,
        ("blogs", "blog_posts") => handle_get::<BlogPost>(&state, table, id).await,
        ("blogs", "comments") => handle_get::<Comment>(&state, table, id).await,
        _ => unreachable!(),
    }
}

pub async fn delete_record_handler(
    State(state): State<AppState>,
    Path((app, resource, id)): Path<(String, String, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let table = get_table_name(&app, &resource)?;
    let sql = format!("DELETE FROM {} WHERE id = $1", table);
    let result = sqlx::query(&sql).bind(id).execute(&state.db).await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Record not found".into()));
    }
    
    Ok(Json(json!({"status": "success"})))
}

pub async fn create_record_handler(
    State(state): State<AppState>,
    Path((app, resource)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    let table = get_table_name(&app, &resource)?;
    let mut obj = payload.as_object().ok_or_else(|| AppError::BadRequest("Expected JSON object".into()))?.clone();
    
    // Auto-inject uuid if missing
    if !obj.contains_key("id") {
        obj.insert("id".to_string(), Value::String(Uuid::new_v4().to_string()));
    }
    
    if obj.is_empty() {
        return Err(AppError::BadRequest("Empty payload".into()));
    }

    let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));
    let mut keys = qb.separated(", ");
    for k in obj.keys() {
        keys.push(format!("\"{}\"", k));
    }
    qb.push(") VALUES (");
    
    let mut values = qb.separated(", ");
    for v in obj.values() {
        match v {
            Value::Null => { values.push_unseparated("NULL"); },
            Value::Bool(b) => { values.push_bind_unseparated(*b); },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() { values.push_bind_unseparated(i); }
                else if let Some(f) = n.as_f64() { values.push_bind_unseparated(f); }
                else { values.push_unseparated("NULL"); }
            },
            Value::String(s) => {
                if let Ok(u) = Uuid::parse_str(s) {
                    values.push_bind_unseparated(u);
                } else if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    values.push_bind_unseparated(dt.with_timezone(&Utc));
                } else {
                    values.push_bind_unseparated(s.clone());
                }
            },
            _ => { values.push_bind_unseparated(v.to_string()); },
        };
    }
    qb.push(")");
    
    qb.build().execute(&state.db).await?;
    
    Ok(Json(json!({"status": "success"})))
}

pub async fn update_record_handler(
    State(state): State<AppState>,
    Path((app, resource, id)): Path<(String, String, Uuid)>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    let table = get_table_name(&app, &resource)?;
    let obj = payload.as_object().ok_or_else(|| AppError::BadRequest("Expected JSON object".into()))?;
    
    if obj.is_empty() {
        return Err(AppError::BadRequest("Empty payload".into()));
    }

    let mut qb = sqlx::QueryBuilder::new(format!("UPDATE {} SET ", table));
    let mut separated = qb.separated(", ");
    
    for (k, v) in obj {
        // Skip updating 'id' if the client sent it
        if k == "id" { continue; }
        
        separated.push(format!("\"{}\" = ", k));
        match v {
            Value::Null => { separated.push_unseparated("NULL"); },
            Value::Bool(b) => { separated.push_bind_unseparated(*b); },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() { separated.push_bind_unseparated(i); }
                else if let Some(f) = n.as_f64() { separated.push_bind_unseparated(f); }
                else { separated.push_unseparated("NULL"); }
            },
            Value::String(s) => {
                // If the string starts with $2y$ (Argon2 hash), DO NOT parse as DateTime!
                // Though Argon2 hashes won't parse as RFC3339 anyway.
                if let Ok(u) = Uuid::parse_str(s) {
                    separated.push_bind_unseparated(u);
                } else if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    separated.push_bind_unseparated(dt.with_timezone(&Utc));
                } else {
                    separated.push_bind_unseparated(s.clone());
                }
            },
            _ => { separated.push_bind_unseparated(v.to_string()); },
        };
    }
    
    qb.push(" WHERE id = ");
    qb.push_bind(id);
    
    qb.build().execute(&state.db).await?;
    
    Ok(Json(json!({"status": "success"})))
}

async fn handle_get<T>(
    state: &AppState,
    table: &str,
    id: Uuid,
) -> Result<Json<Value>, AppError>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + serde::Serialize + Send + Unpin,
{
    let sql = format!("SELECT * FROM {} WHERE id = $1", table);
    let item = sqlx::query_as::<_, T>(&sql)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Record not found".into()))?;

    Ok(Json(json!(item)))
}

async fn handle_list<T>(
    state: &AppState,
    table: &str,
    limit: i64,
    offset: i64,
    search: Option<String>,
) -> Result<Json<Value>, AppError>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + serde::Serialize + Send + Unpin,
{
    let mut qb = sqlx::QueryBuilder::new(format!("SELECT * FROM {}", table));
    
    let search_term = search.filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim().to_lowercase()));
    
    if let Some(ref term) = search_term {
        qb.push(" WHERE (");
        // Generic search strategy: try common string fields
        // Since we don't have schema info here easily, we rely on common names or just email/name
        if table.contains("users") {
            qb.push("email ILIKE ").push_bind(term.clone()).push(" OR full_name ILIKE ").push_bind(term.clone());
        } else if table.contains("blog_posts") {
            qb.push("title ILIKE ").push_bind(term.clone());
        } else if table.contains("comments") {
            qb.push("content ILIKE ").push_bind(term.clone());
        } else {
            // Fallback: search by id if it matches uuid form, or just skip
            qb.push("1=1");
        }
        qb.push(")");
    }
    
    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let items = qb.build_query_as::<T>().fetch_all(&state.db).await?;

    // Parallel count query
    let mut cqb = sqlx::QueryBuilder::new(format!("SELECT COUNT(*) FROM {}", table));
    if let Some(ref term) = search_term {
        cqb.push(" WHERE (");
        if table.contains("users") {
            cqb.push("email ILIKE ").push_bind(term.clone()).push(" OR full_name ILIKE ").push_bind(term.clone());
        } else if table.contains("blog_posts") {
            cqb.push("title ILIKE ").push_bind(term.clone());
        } else if table.contains("comments") {
            cqb.push("content ILIKE ").push_bind(term.clone());
        } else {
            cqb.push("1=1");
        }
        cqb.push(")");
    }
    let total: i64 = cqb.build_query_scalar().fetch_one(&state.db).await?;

    Ok(Json(json!({
        "data": items,
        "total": total,
        "page": (offset / limit) + 1,
        "per_page": limit,
    })))
}
