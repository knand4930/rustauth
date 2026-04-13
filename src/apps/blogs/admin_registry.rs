use crate::admin::AdminPanelBuilder;

use super::admin_config;

pub fn register(builder: &mut AdminPanelBuilder) {
    builder.register_app(admin_config::admin_config());
}
