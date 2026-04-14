pub mod api;
pub mod app;
pub mod config;
pub mod initializer;
pub mod panel;
pub mod registry;
pub mod resource;
pub mod web;

pub use app::routes;
pub use config::{AdminAppConfig, AdminCrudConfig, AdminEndpointConfig, AdminResourceConfig};
pub use initializer::initialize_adminx;
pub use panel::{AdminExtension, AdminPanel, AdminPanelBuilder};
pub use web::web_routes;
