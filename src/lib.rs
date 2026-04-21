pub mod admin;
pub mod apps;
pub mod proto;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod model;
pub mod response;
pub mod state;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses((status = 200, description = "Server is running")),
    tag = "System"
)]
pub async fn health_check() -> &'static str {
    "OK"
}
