// src/products/handler.rs
//
// Axum route handlers for the `products.products` resource.
// Wire these into your router in main.rs:
//
//   use crate::products::handler as products_handler;
//
//   Router::new()
//       .route("/products",     get(products_handler::list).post(products_handler::create))
//       .route("/products/:id", get(products_handler::get).put(products_handler::update)
//                                                     .delete(products_handler::delete))

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::Products;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateProductsRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductsRequest {
    pub name:      Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProductsResponse {
    pub id:         Uuid,
    pub name:       String,
    pub is_active:  bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Products> for ProductsResponse {
    fn from(m: Products) -> Self {
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

/// GET /products  — list all
pub async fn list(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ProductsResponse>>, (StatusCode, String)> {
    let rows = sqlx::query_as!(
        Products,
        "SELECT * FROM products.products ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows.into_iter().map(ProductsResponse::from).collect()))
}

/// GET /products/:id  — fetch one
pub async fn get(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductsResponse>, (StatusCode, String)> {
    let row = sqlx::query_as!(
        Products,
        "SELECT * FROM products.products WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, format!("Products not found")))?;

    Ok(Json(ProductsResponse::from(row)))
}

/// POST /products  — create
pub async fn create(
    State(pool): State<PgPool>,
    Json(body): Json<CreateProductsRequest>,
) -> Result<(StatusCode, Json<ProductsResponse>), (StatusCode, String)> {
    let row = sqlx::query_as!(
        Products,
        "INSERT INTO products.products (id, name, is_active, created_at, updated_at)
         VALUES (gen_random_uuid(), $1, true, NOW(), NOW())
         RETURNING *",
        body.name
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ProductsResponse::from(row))))
}

/// PUT /products/:id  — update
pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProductsRequest>,
) -> Result<Json<ProductsResponse>, (StatusCode, String)> {
    let row = sqlx::query_as!(
        Products,
        "UPDATE products.products
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
    .ok_or((StatusCode::NOT_FOUND, format!("Products not found")))?;

    Ok(Json(ProductsResponse::from(row)))
}

/// DELETE /products/:id  — delete
pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query!(
        "DELETE FROM products.products WHERE id = $1",
        id
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, format!("Products not found")))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
