use crate::apps;

use super::AdminPanelBuilder;

pub fn register_app_registries(builder: &mut AdminPanelBuilder) {
    // startapp:apps:start
    apps::blogs::admin_registry::register(builder);
    apps::user::admin_registry::register(builder);
    // startapp:apps:end
}
