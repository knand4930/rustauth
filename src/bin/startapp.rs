use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRN: &str = "\x1b[32m";
const YLW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYN: &str = "\x1b[36m";

fn snake_to_pascal(value: &str) -> String {
    value
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn pluralize(value: &str) -> String {
    if value.ends_with("ss")
        || value.ends_with('x')
        || value.ends_with('z')
        || value.ends_with("ch")
        || value.ends_with("sh")
    {
        format!("{value}es")
    } else if value.ends_with('s') {
        value.to_string()
    } else {
        format!("{value}s")
    }
}

fn apps_dir(root: &Path) -> PathBuf {
    root.join("src").join("apps")
}

fn gen_mod_rs() -> String {
    r#"pub mod admin_config;
pub mod admin_registry;
pub mod handlers;
pub mod models;
pub mod schemas;

use axum::Router;

use crate::state::AppState;

/// App-local route entrypoint. Add routes here as handlers are introduced.
pub fn routes() -> Router<AppState> {
    Router::new()
}
"#
    .to_string()
}

fn gen_admin_config_rs(app: &str) -> String {
    let pascal = snake_to_pascal(app);
    let label = pascal.clone();
    let plural_path = pluralize(app);

    format!(
        r#"use crate::admin::{{
    AdminAppConfig, AdminCrudConfig, AdminEndpointConfig, AdminResourceConfig,
}};

pub fn admin_config() -> AdminAppConfig {{
    AdminAppConfig::new(
        "{app}",
        "{label}",
        vec![AdminResourceConfig::new(
            "{plural_path}",
            "{label}",
            "{pascal}",
            vec!["id", "name", "is_active", "created_at"],
            vec!["is_active"],
            AdminCrudConfig::new(
                AdminEndpointConfig::new("POST", "/api/v1/{plural_path}"),
                AdminEndpointConfig::new("GET", "/api/v1/{plural_path}/{{id}}"),
                AdminEndpointConfig::new("PUT", "/api/v1/{plural_path}/{{id}}"),
                AdminEndpointConfig::new("DELETE", "/api/v1/{plural_path}/{{id}}"),
            ),
        )],
    )
}}
"#
    )
}

fn gen_admin_registry_rs(_: &str) -> String {
    r#"use crate::admin::AdminPanelBuilder;

use super::admin_config;

pub fn register(builder: &mut AdminPanelBuilder) {
    builder.register_app(admin_config::admin_config());
}
"#
    .to_string()
}

fn gen_models_rs(app: &str, pg_schema: &str) -> String {
    let pascal = snake_to_pascal(app);
    let table = pluralize(app);

    format!(
        r#"// @schema {pg_schema}

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use utoipa::ToSchema;
use uuid::Uuid;

/// Base model for the `{pg_schema}.{table}` table.
// @table {table}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct {pascal} {{
    pub id: Uuid,
    // @unique
    pub name: String,
    // @index
    // @default true
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}

crate::declare_model_table!({pascal}, "{pg_schema}", "{table}");
"#
    )
}

fn gen_schemas_rs(app: &str) -> String {
    let pascal = snake_to_pascal(app);

    format!(
        r#"use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::models::{pascal};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Create{pascal}Request {{
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Update{pascal}Request {{
    pub name: Option<String>,
    pub is_active: Option<bool>,
}}

#[derive(Debug, Serialize, ToSchema)]
pub struct {pascal}Response {{
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}

impl From<{pascal}> for {pascal}Response {{
    fn from(model: {pascal}) -> Self {{
        Self {{
            id: model.id,
            name: model.name,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }}
    }}
}}
"#
    )
}

fn gen_handlers_rs(app: &str) -> String {
    format!(
        r#"// Request handlers for the `{app}` app live here.
// Keep transport logic here and wire routes from mod.rs.
"#
    )
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut app_name: Option<String> = None;
    let mut pg_schema: Option<String> = None;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--schema" => {
                index += 1;
                pg_schema = args.get(index).cloned();
            }
            value if !value.starts_with("--") => app_name = Some(value.to_string()),
            flag => {
                eprintln!("{RED}Unknown flag: {flag}{RST}");
                std::process::exit(1);
            }
        }
        index += 1;
    }

    let app = match app_name {
        Some(value) => value,
        None => {
            eprintln!("{RED}Usage: cargo startapp <appname> [--schema <pgschema>]{RST}");
            eprintln!("  Example: cargo startapp products");
            eprintln!("  Example: cargo startapp products --schema catalog");
            std::process::exit(1);
        }
    };

    if !app
        .chars()
        .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '_')
        || app.starts_with(|char: char| char.is_ascii_digit())
    {
        bail!(
            "{RED}Invalid app name '{app}' — use lowercase letters, digits, and underscores only.{RST}"
        );
    }

    let schema = pg_schema.unwrap_or_else(|| app.clone());
    let pascal = snake_to_pascal(&app);
    let root = PathBuf::from(MANIFEST_DIR);
    let apps_root = apps_dir(&root);
    let app_dir = apps_root.join(&app);
    let app_exists = app_dir.exists();

    if app_exists && !app_dir.is_dir() {
        bail!("{RED}src/apps/{app} exists but is not a directory.{RST}");
    }

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo startapp                     ║");
    println!("╚══════════════════════════════════════╝{RST}");
    println!();
    println!("  {CYN}App name  :{RST}  {BLD}{app}{RST}");
    println!("  {CYN}PG schema :{RST}  {BLD}{schema}{RST}");
    println!("  {CYN}Struct    :{RST}  {BLD}{pascal}{RST}");
    println!("  {CYN}Location  :{RST}  {BLD}src/apps/{app}{RST}");
    println!();

    println!("{CYN}Ensuring app files...{RST}");
    fs::create_dir_all(&app_dir)?;

    let files: &[(&str, fn(&str, &str) -> String)] = &[
        ("mod.rs", |_, _| gen_mod_rs()),
        ("admin_config.rs", |app, _| gen_admin_config_rs(app)),
        ("admin_registry.rs", |app, _| gen_admin_registry_rs(app)),
        ("models.rs", gen_models_rs),
        ("schemas.rs", |app, _| gen_schemas_rs(app)),
        ("handlers.rs", |app, _| gen_handlers_rs(app)),
    ];

    let mut generated_file_count = 0usize;

    for (filename, generator) in files {
        let file_path = app_dir.join(filename);
        if file_path.exists() {
            println!(
                "  {YLW}•{RST}  {BLD}src/apps/{app}/{filename}{RST}  {DIM}(exists, kept as-is){RST}"
            );
            continue;
        }

        let content = generator(&app, &schema);
        fs::write(&file_path, &content)?;
        generated_file_count += 1;
        println!(
            "  {GRN}✓{RST}  {BLD}src/apps/{app}/{filename}{RST}  {DIM}({} lines){RST}",
            content.lines().count()
        );
    }

    println!();
    if app_exists {
        if generated_file_count == 0 {
            println!(
                "{GRN}{BLD}✓  App '{app}' already had all scaffold files. No global modules were changed.{RST}"
            );
        } else {
            println!(
                "{GRN}{BLD}✓  App '{app}' repaired successfully with {generated_file_count} generated file(s).{RST}"
            );
        }
    } else {
        println!("{GRN}{BLD}✓  App '{app}' created successfully!{RST}");
    }
    println!();
    println!("{BLD}Structure:{RST}");
    println!("  {DIM}src/apps/{app}/{RST}");
    println!(
        "  {DIM}├── {RST}{BLD}mod.rs{RST}             {DIM}— app exports and route entrypoint{RST}"
    );
    println!(
        "  {DIM}├── {RST}{BLD}admin_config.rs{RST}    {DIM}— app-level admin list/filter/CRUD metadata{RST}"
    );
    println!(
        "  {DIM}├── {RST}{BLD}admin_registry.rs{RST}  {DIM}— central admin registration hook{RST}"
    );
    println!("  {DIM}├── {RST}{BLD}models.rs{RST}          {DIM}— database models{RST}");
    println!("  {DIM}├── {RST}{BLD}schemas.rs{RST}         {DIM}— request and response DTOs{RST}");
    println!(
        "  {DIM}└── {RST}{BLD}handlers.rs{RST}        {DIM}— route handlers and app router{RST}"
    );
    println!();
    println!("{BLD}Next steps:{RST}");
    println!(
        "  {DIM}1.{RST}  Edit  {BLD}src/apps/{app}/models.rs{RST}          — define your database models"
    );
    println!(
        "  {DIM}2.{RST}  Edit  {BLD}src/apps/{app}/schemas.rs{RST}         — shape request and response DTOs"
    );
    println!(
        "  {DIM}3.{RST}  Edit  {BLD}src/apps/{app}/handlers.rs{RST}        — add request handlers"
    );
    println!(
        "  {DIM}4.{RST}  Edit  {BLD}src/apps/{app}/mod.rs{RST}             — wire app routes locally"
    );
    println!(
        "  {DIM}5.{RST}  Edit  {BLD}src/apps/{app}/admin_config.rs{RST}    — tune app admin metadata"
    );
    println!(
        "  {DIM}6.{RST}  Update {BLD}src/apps/mod.rs{RST}                  — register the app module, routes, and OpenAPI entries"
    );
    println!(
        "  {DIM}7.{RST}  Update {BLD}src/admin/registry.rs{RST}            — register the app admin registry manually"
    );
    println!(
        "  {DIM}8.{RST}  Run   {BLD}cargo makemigrations{RST}              — generate SQL migration"
    );
    println!();

    Ok(())
}
