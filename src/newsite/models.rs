// src/newsite/models.rs
//
// Add your structs here. Each public struct becomes a DB table.
// Run `cargo makemigrations` to generate the migration, then `cargo migrate`.
//
// Field → SQL type mapping:
//   Uuid              → UUID
//   String            → VARCHAR
//   bool              → BOOLEAN
//   i32               → INTEGER
//   i64               → BIGINT
//   f64               → DOUBLE PRECISION
//   DateTime<Utc>     → TIMESTAMPTZ
//   serde_json::Value → JSONB
//   Vec<String>       → TEXT[]
//   Option<T>         → nullable column

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Main model for the `newsite.newsites` table.
/// Rename or add fields — then run `cargo makemigrations`.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Newsite {
    pub id:         Uuid,
    pub name:       String,
    pub is_active:  bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
