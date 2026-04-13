use crate::apps;

use super::{AdminExtension, AdminPanel, AdminPanelBuilder};

pub fn initialize_adminx() -> AdminPanel {
    let mut builder = AdminPanelBuilder::new("RustAuth AdminX", "RustAuth Administration");

    apps::register_admin_apps(&mut builder);
    register_builtin_extensions(&mut builder);

    builder.build()
}

fn register_builtin_extensions(builder: &mut AdminPanelBuilder) {
    builder.register_extension(AdminExtension::new(
        "dashboard",
        "Dashboard",
        "Global metrics and panel overview for the registered admin apps.",
        "/api/adminx/dashboard",
    ));
    builder.register_extension(AdminExtension::new(
        "resource_registry",
        "Resource Registry",
        "Inspect registered app resources, list displays, filters, and CRUD endpoints.",
        "/api/adminx/resources",
    ));
    builder.register_extension(AdminExtension::new(
        "managed_users",
        "Managed Users",
        "Extended user management with admin-only filters and inline updates.",
        "/api/adminx/users",
    ));
}
