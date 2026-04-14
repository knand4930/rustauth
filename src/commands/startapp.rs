use crate::commands::common::{BLD, CYN, DIM, GRN, RED, RST, YLW};
use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

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

fn validate_name(kind: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{RED}Invalid {kind}: value cannot be empty.{RST}");
    }

    if !value.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    }) || value.starts_with(|character: char| character.is_ascii_digit())
    {
        bail!(
            "{RED}Invalid {kind} '{value}' — use lowercase letters, digits, and underscores only.{RST}"
        );
    }

    Ok(())
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

fn gen_admin_registry_rs() -> String {
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

fn render_with_trailing_newline(lines: &[String], had_trailing_newline: bool) -> String {
    let mut output = lines.join("\n");
    if had_trailing_newline || output.is_empty() {
        output.push('\n');
    }
    output
}

fn ensure_pub_mod(source: &str, module_name: &str) -> Result<(String, bool)> {
    let target = format!("pub mod {module_name};");
    let trimmed_target = target.trim();
    if source.lines().any(|line| line.trim() == trimmed_target) {
        return Ok((source.to_string(), false));
    }

    let lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let module_indexes = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            line.trim_start().starts_with("pub mod ") && line.trim_end().ends_with(';')
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    if module_indexes.is_empty() {
        bail!("Could not find module declaration block to register '{module_name}'.");
    }

    let start = *module_indexes.first().unwrap();
    let end = *module_indexes.last().unwrap();
    let mut modules = module_indexes
        .iter()
        .map(|&index| lines[index].trim().to_string())
        .collect::<Vec<_>>();
    modules.push(target);
    modules.sort();
    modules.dedup();

    let mut output = Vec::new();
    output.extend_from_slice(&lines[..start]);
    output.extend(modules);
    output.extend_from_slice(&lines[end + 1..]);

    Ok((
        render_with_trailing_newline(&output, source.ends_with('\n')),
        true,
    ))
}

fn ensure_route_merge(source: &str, module_name: &str) -> Result<(String, bool)> {
    let target = format!(".merge({module_name}::routes())");
    if source.contains(&target) {
        return Ok((source.to_string(), false));
    }

    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    for line in &mut lines {
        if line.contains("let router = Router::new()") && line.trim_end().ends_with(';') {
            if let Some(position) = line.rfind(';') {
                line.insert_str(position, &target);
                return Ok((
                    render_with_trailing_newline(&lines, source.ends_with('\n')),
                    true,
                ));
            }
        }
    }

    bail!("Could not find the route builder in src/apps/mod.rs to register '{module_name}'.");
}

fn ensure_admin_registry_call(source: &str, module_name: &str) -> Result<(String, bool)> {
    let target = format!("apps::{module_name}::admin_registry::register(builder);");
    if source.lines().any(|line| line.trim() == target) {
        return Ok((source.to_string(), false));
    }

    let lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let call_indexes = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            line.trim_start().starts_with("apps::")
                && line
                    .trim_end()
                    .ends_with("::admin_registry::register(builder);")
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    if call_indexes.is_empty() {
        bail!("Could not find the admin registry block to register '{module_name}'.");
    }

    let start = *call_indexes.first().unwrap();
    let end = *call_indexes.last().unwrap();
    let mut calls = call_indexes
        .iter()
        .map(|&index| lines[index].trim().to_string())
        .collect::<Vec<_>>();
    calls.push(target);
    calls.sort();
    calls.dedup();

    let indentation = lines[start]
        .chars()
        .take_while(|character| character.is_whitespace())
        .collect::<String>();
    let calls = calls
        .into_iter()
        .map(|call| format!("{indentation}{call}"))
        .collect::<Vec<_>>();

    let mut output = Vec::new();
    output.extend_from_slice(&lines[..start]);
    output.extend(calls);
    output.extend_from_slice(&lines[end + 1..]);

    Ok((
        render_with_trailing_newline(&output, source.ends_with('\n')),
        true,
    ))
}

fn update_file_if_changed(
    file_path: &Path,
    updater: impl FnOnce(&str) -> Result<(String, bool)>,
) -> Result<bool> {
    let existing = fs::read_to_string(file_path)?;
    let (updated, changed) = updater(&existing)?;
    if changed {
        fs::write(file_path, updated)?;
    }
    Ok(changed)
}

pub fn run(args: &[String]) -> Result<()> {
    let mut app_name: Option<String> = None;
    let mut pg_schema: Option<String> = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "-h" | "--help" => {
                println!("Usage:");
                println!("  {BLD}cargo startapp <appname> [--schema <pgschema>]{RST}");
                println!("  {BLD}cargo startapp --list{RST}                         list existing apps");
                println!("  Example: cargo startapp products");
                println!("  Example: cargo startapp products --schema catalog");
                println!();
                return Ok(());
            }
            "--list" => {
                let root = PathBuf::from(MANIFEST_DIR);
                let apps_root = apps_dir(&root);
                println!("\n{BLD}Apps in src/apps/{RST}");
                println!("{DIM}─────────────────────────────────{RST}");
                if let Ok(entries) = std::fs::read_dir(&apps_root) {
                    let mut apps: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    apps.sort();
                    if apps.is_empty() {
                        println!("{DIM}  (no apps yet){RST}");
                    } else {
                        for app in &apps {
                            let has_models = apps_root.join(app).join("models.rs").exists();
                            let model_mark = if has_models { format!("{GRN}models{RST}") } else { format!("{DIM}no models{RST}") };
                            println!("  {BLD}{:<20}{RST}  {model_mark}", app);
                        }
                        println!("\n{DIM}Total: {} app(s){RST}", apps.len());
                    }
                } else {
                    println!("{DIM}  src/apps/ directory not found{RST}");
                }
                println!();
                return Ok(());
            }
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

    validate_name("app name", &app)?;
    let schema = pg_schema.unwrap_or_else(|| app.clone());
    validate_name("schema", &schema)?;

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
        ("admin_registry.rs", |_, _| gen_admin_registry_rs()),
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
    println!("{CYN}Registering the app project-wide...{RST}");

    let apps_mod_path = root.join("src").join("apps").join("mod.rs");
    let mut global_updates = 0usize;
    if update_file_if_changed(&apps_mod_path, |source| ensure_pub_mod(source, &app))? {
        global_updates += 1;
        println!("  {GRN}✓{RST}  {BLD}src/apps/mod.rs{RST}  {DIM}(module declaration added){RST}");
    } else {
        println!("  {YLW}•{RST}  {BLD}src/apps/mod.rs{RST}  {DIM}(module already registered){RST}");
    }

    if update_file_if_changed(&apps_mod_path, |source| ensure_route_merge(source, &app))? {
        global_updates += 1;
        println!("  {GRN}✓{RST}  {BLD}src/apps/mod.rs{RST}  {DIM}(route merge added){RST}");
    } else {
        println!("  {YLW}•{RST}  {BLD}src/apps/mod.rs{RST}  {DIM}(route already wired){RST}");
    }

    let admin_registry_path = root.join("src").join("admin").join("registry.rs");
    if update_file_if_changed(&admin_registry_path, |source| {
        ensure_admin_registry_call(source, &app)
    })? {
        global_updates += 1;
        println!(
            "  {GRN}✓{RST}  {BLD}src/admin/registry.rs{RST}  {DIM}(admin registry added){RST}"
        );
    } else {
        println!(
            "  {YLW}•{RST}  {BLD}src/admin/registry.rs{RST}  {DIM}(admin registry already wired){RST}"
        );
    }

    println!();
    if app_exists {
        if generated_file_count == 0 && global_updates == 0 {
            println!("{GRN}{BLD}✓  App '{app}' was already fully scaffolded and wired.{RST}");
        } else {
            println!(
                "{GRN}{BLD}✓  App '{app}' repaired successfully with {generated_file_count} generated file(s) and {global_updates} project registration update(s).{RST}"
            );
        }
    } else {
        println!(
            "{GRN}{BLD}✓  App '{app}' created successfully with {global_updates} project registration update(s).{RST}"
        );
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
        "  {DIM}1.{RST}  Edit  {BLD}src/apps/{app}/models.rs{RST}          — define your database models with @schema and @table directives"
    );
    println!(
        "  {DIM}2.{RST}  Edit  {BLD}src/apps/{app}/schemas.rs{RST}         — shape request and response DTOs"
    );
    println!(
        "  {DIM}3.{RST}  Edit  {BLD}src/apps/{app}/handlers.rs{RST}        — add request handlers and implement routes()"
    );
    println!(
        "  {DIM}4.{RST}  Edit  {BLD}src/apps/{app}/mod.rs{RST}             — wire app routes locally in routes() function"
    );
    println!(
        "  {DIM}5.{RST}  Edit  {BLD}src/apps/{app}/admin_config.rs{RST}    — tune app admin metadata for AdminX"
    );
    println!(
        "  {DIM}6.{RST}  Run   {BLD}cargo tests -v{RST}                    — validate your app structure and imports"
    );
    println!(
        "  {DIM}7.{RST}  Run   {BLD}cargo makemigrations{RST}              — generate SQL migration from models"
    );
    println!(
        "  {DIM}8.{RST}  Run   {BLD}cargo migrate{RST}                     — apply migration to database"
    );
    println!();
    println!("{BLD}Automatic Integration:{RST}");
    println!("  {DIM}✓ Module declared in src/apps/mod.rs{RST}");
    println!("  {DIM}✓ Routes merged in src/apps/mod.rs{RST}");
    println!("  {DIM}✓ Admin registry wired in src/admin/registry.rs{RST}");
    println!("  {DIM}✓ Models will be auto-detected by cargo makemigrations{RST}");
    println!();
    println!("{BLD}Tips:{RST}");
    println!("  {DIM}• Use // @schema and // @table directives in models.rs{RST}");
    println!("  {DIM}• Add // @index, // @unique, // @default to fields{RST}");
    println!("  {DIM}• Use // @references schema.table for foreign keys{RST}");
    println!("  {DIM}• Run cargo tests -v to validate your changes{RST}");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_admin_registry_call, ensure_pub_mod, ensure_route_merge};

    #[test]
    fn inserts_new_app_module_sorted() {
        let source = "pub mod blogs;\npub mod user;\n\nuse axum::Router;\n";
        let (updated, changed) = ensure_pub_mod(source, "catalog").expect("updated module block");
        assert!(changed);
        assert!(updated.contains("pub mod blogs;\npub mod catalog;\npub mod user;"));
    }

    #[test]
    fn appends_route_merge_once() {
        let source = r#"pub fn routes() -> Router<AppState> {
    let router = Router::new().merge(blogs::routes()).merge(user::routes());

    router
}
"#;
        let (updated, changed) = ensure_route_merge(source, "catalog").expect("updated routes");
        assert!(changed);
        assert!(updated.contains(".merge(user::routes()).merge(catalog::routes());"));

        let (_, changed_again) = ensure_route_merge(&updated, "catalog").expect("second pass");
        assert!(!changed_again);
    }

    #[test]
    fn inserts_admin_registry_call_sorted() {
        let source = r#"pub fn register_app_registries(builder: &mut AdminPanelBuilder) {
    apps::blogs::admin_registry::register(builder);
    apps::user::admin_registry::register(builder);
}
"#;
        let (updated, changed) =
            ensure_admin_registry_call(source, "catalog").expect("updated admin registry");
        assert!(changed);
        assert!(updated.contains(
            "    apps::blogs::admin_registry::register(builder);\n    apps::catalog::admin_registry::register(builder);\n    apps::user::admin_registry::register(builder);"
        ));
    }
}
