// src/activitylog/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ActivityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity: Option<String>,
    pub entity_id: Option<Uuid>,
    pub status: String,
    pub message: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
