// src/response.rs
//
// Generic, typed API response envelopes used across all app handlers.
//
//   use crate::response::{ApiSuccess, ApiPaginated, ApiList, ApiMessage};
//

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

// ─── Pagination metadata ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total:       i64,
    pub page:        i64,
    pub per_page:    i64,
    pub total_pages: i64,
}

impl PaginationMeta {
    pub fn new(total: i64, page: i64, per_page: i64) -> Self {
        Self {
            total,
            page,
            per_page,
            total_pages: (total as f64 / per_page as f64).ceil() as i64,
        }
    }
}

// ─── Single-object response ──────────────────────────────────────────

/// `{ "success": true, "data": T }`
#[derive(Debug, Serialize)]
pub struct ApiSuccess<T: Serialize> {
    pub success: bool,
    pub data:    T,
}

impl<T: Serialize> ApiSuccess<T> {
    pub fn ok(data: T) -> (StatusCode, Json<Self>) {
        (StatusCode::OK, Json(Self { success: true, data }))
    }

    pub fn created(data: T) -> (StatusCode, Json<Self>) {
        (StatusCode::CREATED, Json(Self { success: true, data }))
    }
}

impl<T: Serialize> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

// ─── Paginated list response ─────────────────────────────────────────

/// `{ "success": true, "data": [T], "pagination": { ... } }`
#[derive(Debug, Serialize)]
pub struct ApiPaginated<T: Serialize> {
    pub success:    bool,
    pub data:       Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T: Serialize> ApiPaginated<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success:    true,
                data,
                pagination: PaginationMeta::new(total, page, per_page),
            }),
        )
    }
}

// ─── Non-paginated list response ─────────────────────────────────────

/// `{ "success": true, "data": [T], "count": N }`
#[derive(Debug, Serialize)]
pub struct ApiList<T: Serialize> {
    pub success: bool,
    pub data:    Vec<T>,
    pub count:   usize,
}

impl<T: Serialize> ApiList<T> {
    pub fn new(data: Vec<T>) -> (StatusCode, Json<Self>) {
        let count = data.len();
        (StatusCode::OK, Json(Self { success: true, data, count }))
    }
}

// ─── Message-only response ───────────────────────────────────────────

/// `{ "success": true, "message": "..." }`
#[derive(Debug, Serialize)]
pub struct ApiMessage {
    pub success: bool,
    pub message: String,
}

impl ApiMessage {
    pub fn ok(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (StatusCode::OK, Json(Self { success: true, message: msg.into() }))
    }

    pub fn deleted(resource: &str) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                message: format!("{resource} deleted successfully"),
            }),
        )
    }
}
