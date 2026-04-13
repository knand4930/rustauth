pub mod app;
pub mod config;
pub mod initializer;
pub mod panel;
pub mod registry;
pub mod resource;

pub use app::routes;
pub use config::{AdminAppConfig, AdminCrudConfig, AdminEndpointConfig, AdminResourceConfig};
pub use initializer::initialize_adminx;
pub use panel::{AdminExtension, AdminPanel, AdminPanelBuilder};
