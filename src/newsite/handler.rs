// src/newsite/handler.rs
//
// Axum route handlers for the `newsite.newsites` resource.
// Wire these into your router in main.rs:
//
//   use crate::newsite::handler as newsite_handler;
//
//   Router::new()
//       .route("/newsites",     get(newsite_handler::list).post(newsite_handler::create))
//       .route("/newsites/:id", get(newsite_handler::get).put(newsite_handler::update)
//                                                     .delete(newsite_handler::delete))

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::Newsite;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateNewsiteRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewsiteRequest {
    pub name:      Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct NewsiteResponse {
    pub id:         Uuid,
    pub name:       String,
    pub is_active:  bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Newsite> for NewsiteResponse {
    fn from(m: Newsite) -> Self {
        Self {
            id:         m.id,
            name:       m.name,
            is_active:  m.is_active,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /newsites  — list all
pub async fn list(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<NewsiteResponse>>, (StatusCode, String)> {
    let rows = sqlx::query_as!(
        Newsite,
        "SELECT * FROM newsite.newsites ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows.into_iter().map(NewsiteResponse::from).collect()))
}

/// GET /newsites/:id  — fetch one
pub async fn get(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<NewsiteResponse>, (StatusCode, String)> {
    let row = sqlx::query_as!(
        Newsite,
        "SELECT * FROM newsite.newsites WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, format!("Newsite not found")))?;

    Ok(Json(NewsiteResponse::from(row)))
}

/// POST /newsites  — create
pub async fn create(
    State(pool): State<PgPool>,
    Json(body): Json<CreateNewsiteRequest>,
) -> Result<(StatusCode, Json<NewsiteResponse>), (StatusCode, String)> {
    let row = sqlx::query_as!(
        Newsite,
        "INSERT INTO newsite.newsites (id, name, is_active, created_at, updated_at)
         VALUES (gen_random_uuid(), $1, true, NOW(), NOW())
         RETURNING *",
        body.name
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(NewsiteResponse::from(row))))
}

/// PUT /newsites/:id  — update
pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateNewsiteRequest>,
) -> Result<Json<NewsiteResponse>, (StatusCode, String)> {
    let row = sqlx::query_as!(
        Newsite,
        "UPDATE newsite.newsites
         SET    name      = COALESCE($2, name),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
         WHERE  id = $1
         RETURNING *",
        id, body.name, body.is_active
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, format!("Newsite not found")))?;

    Ok(Json(NewsiteResponse::from(row)))
}

/// DELETE /newsites/:id  — delete
pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query!(
        "DELETE FROM newsite.newsites WHERE id = $1",
        id
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, format!("Newsite not found")))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
