use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminEndpointConfig {
    pub method: String,
    pub path: String,
}

impl AdminEndpointConfig {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminCrudConfig {
    pub create: AdminEndpointConfig,
    pub retrieve: AdminEndpointConfig,
    pub update: AdminEndpointConfig,
    pub delete: AdminEndpointConfig,
}

impl AdminCrudConfig {
    pub fn new(
        create: AdminEndpointConfig,
        retrieve: AdminEndpointConfig,
        update: AdminEndpointConfig,
        delete: AdminEndpointConfig,
    ) -> Self {
        Self {
            create,
            retrieve,
            update,
            delete,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminResourceConfig {
    pub key: String,
    pub label: String,
    pub model: String,
    pub list_display: Vec<String>,
    pub list_filter: Vec<String>,
    pub crud: AdminCrudConfig,
}

impl AdminResourceConfig {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        model: impl Into<String>,
        list_display: Vec<&str>,
        list_filter: Vec<&str>,
        crud: AdminCrudConfig,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            model: model.into(),
            list_display: list_display.into_iter().map(str::to_string).collect(),
            list_filter: list_filter.into_iter().map(str::to_string).collect(),
            crud,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminAppConfig {
    pub key: String,
    pub label: String,
    pub resources: Vec<AdminResourceConfig>,
}

impl AdminAppConfig {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        resources: Vec<AdminResourceConfig>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            resources,
        }
    }
}
