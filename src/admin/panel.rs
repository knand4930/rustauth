use serde::Serialize;
use utoipa::ToSchema;

use super::config::AdminAppConfig;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminExtension {
    pub key: String,
    pub label: String,
    pub description: String,
    pub route: String,
}

impl AdminExtension {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
        route: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            description: description.into(),
            route: route.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminPanel {
    pub site_title: String,
    pub site_header: String,
    pub app_count: usize,
    pub resource_count: usize,
    pub apps: Vec<AdminAppConfig>,
    pub extensions: Vec<AdminExtension>,
}

impl AdminPanel {
    pub fn find_app(&self, key: &str) -> Option<&AdminAppConfig> {
        self.apps.iter().find(|app| app.key == key)
    }
}

pub struct AdminPanelBuilder {
    site_title: String,
    site_header: String,
    apps: Vec<AdminAppConfig>,
    extensions: Vec<AdminExtension>,
}

impl AdminPanelBuilder {
    pub fn new(site_title: impl Into<String>, site_header: impl Into<String>) -> Self {
        Self {
            site_title: site_title.into(),
            site_header: site_header.into(),
            apps: Vec::new(),
            extensions: Vec::new(),
        }
    }

    pub fn register_app(&mut self, app: AdminAppConfig) {
        self.apps.push(app);
    }

    pub fn register_extension(&mut self, extension: AdminExtension) {
        self.extensions.push(extension);
    }

    pub fn build(mut self) -> AdminPanel {
        self.apps
            .sort_by(|left, right| left.label.cmp(&right.label));
        for app in &mut self.apps {
            app.resources
                .sort_by(|left, right| left.label.cmp(&right.label));
        }
        self.extensions
            .sort_by(|left, right| left.label.cmp(&right.label));

        let app_count = self.apps.len();
        let resource_count = self.apps.iter().map(|app| app.resources.len()).sum();

        AdminPanel {
            site_title: self.site_title,
            site_header: self.site_header,
            app_count,
            resource_count,
            apps: self.apps,
            extensions: self.extensions,
        }
    }
}
