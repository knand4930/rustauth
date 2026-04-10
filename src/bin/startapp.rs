use anyhow::{Context, Result, bail};
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

fn apps_mod_path(root: &Path) -> PathBuf {
    apps_dir(root).join("mod.rs")
}

fn gen_mod_rs(app: &str) -> String {
    let pascal = snake_to_pascal(app);

    format!(
        r#"pub mod handlers;
pub mod models;
pub mod schemas;

pub use models::{pascal};
pub use schemas::{{Create{pascal}Request, Update{pascal}Request, {pascal}Response}};

use axum::Router;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {{
    handlers::routes()
}}
"#
    )
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
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct {pascal} {{
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}
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
    let table = pluralize(app);

    format!(
        r#"use axum::Router;

use crate::state::AppState;

/// Register `{table}` routes here as the app grows.
pub fn routes() -> Router<AppState> {{
    Router::new()
}}
"#
    )
}

fn insert_before(path: &Path, anchor: &str, new_line: &str) -> Result<bool> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Cannot read {}", path.display()))?;

    if content.contains(new_line.trim()) {
        return Ok(false);
    }

    let mut lines: Vec<&str> = content.lines().collect();
    let pos = lines
        .iter()
        .position(|line| line.contains(anchor))
        .with_context(|| format!("Anchor '{anchor}' not found in {}", path.display()))?;

    lines.insert(pos, new_line);
    fs::write(path, lines.join("\n") + "\n")?;
    Ok(true)
}

fn patch_apps_mod(app: &str, root: &Path) -> Result<()> {
    let path = apps_mod_path(root);

    let added_module = insert_before(&path, "// startapp:modules", &format!("pub mod {app};"))?;
    if added_module {
        println!("  {GRN}✓{RST}  {BLD}src/apps/mod.rs{RST}  {DIM}← pub mod {app};{RST}");
    } else {
        println!("  {DIM}≡  src/apps/mod.rs  (module already registered){RST}");
    }

    let added_route = insert_before(
        &path,
        "// startapp:routes",
        &format!("    let router = router.merge({app}::routes());"),
    )?;
    if added_route {
        println!(
            "  {GRN}✓{RST}  {BLD}src/apps/mod.rs{RST}  {DIM}← let router = router.merge({app}::routes());{RST}"
        );
    } else {
        println!("  {DIM}≡  src/apps/mod.rs  (route already registered){RST}");
    }

    Ok(())
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

    if app_dir.exists() {
        bail!("{YLW}Directory src/apps/{app}/ already exists.{RST}  Use a different name.");
    }

    if !apps_mod_path(&root).exists() {
        bail!(
            "{RED}src/apps/mod.rs is missing.{RST}  Restore the app registry before scaffolding."
        );
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

    println!("{CYN}Creating app files...{RST}");
    fs::create_dir_all(&app_dir)?;

    let files: &[(&str, fn(&str, &str) -> String)] = &[
        ("mod.rs", |app, _| gen_mod_rs(app)),
        ("models.rs", gen_models_rs),
        ("schemas.rs", |app, _| gen_schemas_rs(app)),
        ("handlers.rs", |app, _| gen_handlers_rs(app)),
    ];

    for (filename, generator) in files {
        let content = generator(&app, &schema);
        let file_path = app_dir.join(filename);
        fs::write(&file_path, &content)?;
        println!(
            "  {GRN}✓{RST}  {BLD}src/apps/{app}/{filename}{RST}  {DIM}({} lines){RST}",
            content.lines().count()
        );
    }

    println!();
    println!("{CYN}Registering app...{RST}");
    patch_apps_mod(&app, &root)?;

    println!();
    println!("{GRN}{BLD}✓  App '{app}' created successfully!{RST}");
    println!();
    println!("{BLD}Structure:{RST}");
    println!("  {DIM}src/apps/{app}/{RST}");
    println!("  {DIM}├── {RST}{BLD}mod.rs{RST}       {DIM}— app exports and route entrypoint{RST}");
    println!("  {DIM}├── {RST}{BLD}models.rs{RST}    {DIM}— database models{RST}");
    println!("  {DIM}├── {RST}{BLD}schemas.rs{RST}   {DIM}— request and response DTOs{RST}");
    println!("  {DIM}└── {RST}{BLD}handlers.rs{RST}  {DIM}— route handlers and app router{RST}");
    println!();
    println!("{BLD}Next steps:{RST}");
    println!(
        "  {DIM}1.{RST}  Edit  {BLD}src/apps/{app}/handlers.rs{RST}  — add routes and business logic"
    );
    println!(
        "  {DIM}2.{RST}  Edit  {BLD}src/apps/{app}/models.rs{RST}    — expand your data model"
    );
    println!(
        "  {DIM}3.{RST}  Edit  {BLD}src/apps/{app}/schemas.rs{RST}   — shape request and response types"
    );
    println!(
        "  {DIM}4.{RST}  Update {BLD}src/apps/mod.rs{RST}            — add OpenAPI paths when endpoints exist"
    );
    println!(
        "  {DIM}5.{RST}  Run   {BLD}cargo makemigrations{RST}        — generate SQL migration"
    );
    println!();

    Ok(())
}
