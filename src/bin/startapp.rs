// src/bin/startapp.rs
//
// Django-style "startapp" — scaffolds a new app module with all boilerplate
// and wires it into every system file automatically.
//
//   cargo startapp <appname>
//   cargo startapp <appname> --schema <pgschema>
//
// What it creates:
//   src/<appname>/
//       mod.rs       — module declarations
//       models.rs    — starter Rust structs (sqlx::FromRow)
//       handler.rs   — axum route handlers (CRUD stubs)
//       schemas.rs   — Diesel/DB schema (empty, filled by makemigrations)
//
// What it patches automatically:
//   src/main.rs              — add `mod <appname>;`
//   src/models/mod.rs        — re-export structs
//   src/schema.rs            — add schema include block
//   diesel.toml              — add [print_schema.<pgschema>] section
//   src/bin/makemigrations.rs — add to MODEL_FILES registry
//   .apps.json               — internal app registry (created if missing)

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

// ── ANSI colours ──────────────────────────────────────────────────────────────
const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRN: &str = "\x1b[32m";
const YLW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYN: &str = "\x1b[36m";

// ── App registry (.apps.json) ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppRegistry {
    apps: Vec<AppEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppEntry {
    name:       String,  // e.g. "products"
    pg_schema:  String,  // e.g. "products" or "catalog"
    models_rel: String,  // e.g. "products/models.rs"
}

fn registry_path() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join(".apps.json")
}

fn load_registry() -> AppRegistry {
    let p = registry_path();
    if !p.exists() { return AppRegistry::default(); }
    serde_json::from_str(&fs::read_to_string(p).unwrap_or_default()).unwrap_or_default()
}

fn save_registry(reg: &AppRegistry) -> Result<()> {
    fs::write(registry_path(), serde_json::to_string_pretty(reg)?)?;
    Ok(())
}

// ── Name helpers ──────────────────────────────────────────────────────────────

fn snake_to_pascal(s: &str) -> String {
    s.split('_').map(|w| {
        let mut c = w.chars();
        match c.next() {
            None    => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }).collect()
}

fn pluralize(s: &str) -> String {
    if s.ends_with("ss") || s.ends_with('x') || s.ends_with('z')
        || s.ends_with("ch") || s.ends_with("sh") { format!("{s}es") }
    else if s.ends_with('s') { s.to_string() }
    else { format!("{s}s") }
}

// ── Template generators ───────────────────────────────────────────────────────

fn gen_mod_rs(app: &str) -> String {
    format!(
"// src/{app}/mod.rs

pub mod handler;
pub mod models;
pub mod schemas;
"
    )
}

fn gen_models_rs(app: &str, pg_schema: &str) -> String {
    let pascal    = snake_to_pascal(app);
    let table     = pluralize(app);

    format!(
r#"// src/{app}/models.rs
//
// Add your structs here. Each public struct becomes a DB table.
// Run `cargo makemigrations` to generate the migration, then `cargo migrate`.
//
// Field → SQL type mapping:
//   Uuid              → UUID
//   String            → VARCHAR
//   bool              → BOOLEAN
//   i32               → INTEGER
//   i64               → BIGINT
//   f64               → DOUBLE PRECISION
//   DateTime<Utc>     → TIMESTAMPTZ
//   serde_json::Value → JSONB
//   Vec<String>       → TEXT[]
//   Option<T>         → nullable column

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

/// Main model for the `{pg_schema}.{table}` table.
/// Rename or add fields — then run `cargo makemigrations`.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct {pascal} {{
    pub id:         Uuid,
    pub name:       String,
    pub is_active:  bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}
"#
    )
}

fn gen_handler_rs(app: &str, pg_schema: &str) -> String {
    let pascal = snake_to_pascal(app);
    let table  = pluralize(app);

    format!(
r#"// src/{app}/handler.rs
//
// Axum route handlers for the `{pg_schema}.{table}` resource.
// Wire these into your router in main.rs:
//
//   use crate::{app}::handler as {app}_handler;
//
//   Router::new()
//       .route("/{table}",     get({app}_handler::list).post({app}_handler::create))
//       .route("/{table}/:id", get({app}_handler::get).put({app}_handler::update)
//                                                     .delete({app}_handler::delete))

use axum::{{
    extract::{{Path, State}},
    http::StatusCode,
    Json,
}};
use serde::{{Deserialize, Serialize}};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{pascal};

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Create{pascal}Request {{
    pub name: String,
}}

#[derive(Debug, Deserialize)]
pub struct Update{pascal}Request {{
    pub name:      Option<String>,
    pub is_active: Option<bool>,
}}

#[derive(Debug, Serialize)]
pub struct {pascal}Response {{
    pub id:         Uuid,
    pub name:       String,
    pub is_active:  bool,
    pub created_at: String,
    pub updated_at: String,
}}

impl From<{pascal}> for {pascal}Response {{
    fn from(m: {pascal}) -> Self {{
        Self {{
            id:         m.id,
            name:       m.name,
            is_active:  m.is_active,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }}
    }}
}}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /{table}  — list all
pub async fn list(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<{pascal}Response>>, (StatusCode, String)> {{
    let rows = sqlx::query_as!(
        {pascal},
        "SELECT * FROM {pg_schema}.{table} ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows.into_iter().map({pascal}Response::from).collect()))
}}

/// GET /{table}/:id  — fetch one
pub async fn get(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<{pascal}Response>, (StatusCode, String)> {{
    let row = sqlx::query_as!(
        {pascal},
        "SELECT * FROM {pg_schema}.{table} WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, format!("{pascal} not found")))?;

    Ok(Json({pascal}Response::from(row)))
}}

/// POST /{table}  — create
pub async fn create(
    State(pool): State<PgPool>,
    Json(body): Json<Create{pascal}Request>,
) -> Result<(StatusCode, Json<{pascal}Response>), (StatusCode, String)> {{
    let row = sqlx::query_as!(
        {pascal},
        "INSERT INTO {pg_schema}.{table} (id, name, is_active, created_at, updated_at)
         VALUES (gen_random_uuid(), $1, true, NOW(), NOW())
         RETURNING *",
        body.name
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json({pascal}Response::from(row))))
}}

/// PUT /{table}/:id  — update
pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Update{pascal}Request>,
) -> Result<Json<{pascal}Response>, (StatusCode, String)> {{
    let row = sqlx::query_as!(
        {pascal},
        "UPDATE {pg_schema}.{table}
         SET    name      = COALESCE($2, name),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
         WHERE  id = $1
         RETURNING *",
        id, body.name, body.is_active
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, format!("{pascal} not found")))?;

    Ok(Json({pascal}Response::from(row)))
}}

/// DELETE /{table}/:id  — delete
pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {{
    let result = sqlx::query!(
        "DELETE FROM {pg_schema}.{table} WHERE id = $1",
        id
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {{
        Err((StatusCode::NOT_FOUND, format!("{pascal} not found")))
    }} else {{
        Ok(StatusCode::NO_CONTENT)
    }}
}}
"#
    )
}

fn gen_schemas_rs(app: &str) -> String {
    format!(
"// src/{app}/schemas.rs
// @generated — updated automatically by `diesel print-schema` via diesel.toml.
// Do not edit by hand.
"
    )
}

// ── File patching helpers ─────────────────────────────────────────────────────

/// Read file, insert `new_line` right before the first line matching `anchor`.
/// If `anchor` is not found, append to end.
fn insert_before(path: &PathBuf, anchor: &str, new_line: &str) -> Result<bool> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;

    // Idempotency — skip if already present
    if content.contains(new_line.trim()) { return Ok(false); }

    let mut lines: Vec<&str> = content.lines().collect();
    let pos = lines.iter().position(|l| l.contains(anchor));
    match pos {
        Some(i) => lines.insert(i, new_line),
        None    => { lines.push(""); lines.push(new_line); }
    }

    fs::write(path, lines.join("\n") + "\n")?;
    Ok(true)
}

/// Append a line to a file if not already present (idempotent).
fn append_if_missing(path: &PathBuf, new_line: &str) -> Result<bool> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    if content.contains(new_line.trim()) { return Ok(false); }
    let sep = if content.ends_with('\n') { "" } else { "\n" };
    fs::write(path, format!("{content}{sep}{new_line}\n"))?;
    Ok(true)
}

/// Replace a specific marker line with two lines (the marker + new content).
#[allow(dead_code)]
fn insert_after_marker(path: &PathBuf, marker: &str, new_line: &str) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    if content.contains(new_line.trim()) { return Ok(false); }

    let mut out = String::with_capacity(content.len() + new_line.len() + 2);
    let mut inserted = false;
    for line in content.lines() {
        out.push_str(line);
        out.push('\n');
        if !inserted && line.contains(marker) {
            out.push_str(new_line);
            out.push('\n');
            inserted = true;
        }
    }
    if !inserted {
        out.push_str(new_line);
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(true)
}

// ── Patcher: src/main.rs ──────────────────────────────────────────────────────

fn patch_main_rs(app: &str, src: &PathBuf) -> Result<()> {
    let path = src.join("main.rs");
    // Insert `mod <app>;` right before `fn main()`
    let added = insert_before(&path, "fn main()", &format!("mod {app};"))?;
    if added {
        println!("  {GRN}✓{RST}  {BLD}src/main.rs{RST}  {DIM}← mod {app};{RST}");
    } else {
        println!("  {DIM}≡  src/main.rs  (already registered){RST}");
    }
    Ok(())
}

// ── Patcher: src/models/mod.rs ────────────────────────────────────────────────

fn patch_models_mod(app: &str, pascal: &str, src: &PathBuf) -> Result<()> {
    let path = src.join("models").join("mod.rs");
    let line = format!("pub use crate::{app}::models::{pascal};");
    let added = append_if_missing(&path, &line)?;
    if added {
        println!("  {GRN}✓{RST}  {BLD}src/models/mod.rs{RST}  {DIM}← pub use crate::{app}::models::{pascal};{RST}");
    } else {
        println!("  {DIM}≡  src/models/mod.rs  (already registered){RST}");
    }
    Ok(())
}

// ── Patcher: src/schema.rs ────────────────────────────────────────────────────

fn patch_schema_rs(app: &str, pg_schema: &str, src: &PathBuf) -> Result<()> {
    let path = src.join("schema.rs");
    let block = format!(
        "pub mod {pg_schema} {{\n    include!(\"{app}/schemas.rs\");\n}}"
    );
    // Check if already present (check just the mod line)
    let content = fs::read_to_string(&path)?;
    if content.contains(&format!("pub mod {pg_schema}")) {
        println!("  {DIM}≡  src/schema.rs  (already registered){RST}");
        return Ok(());
    }
    let sep = if content.ends_with('\n') { "" } else { "\n" };
    fs::write(&path, format!("{content}{sep}\n{block}\n"))?;
    println!("  {GRN}✓{RST}  {BLD}src/schema.rs{RST}  {DIM}← pub mod {pg_schema} {{ ... }}{RST}");
    Ok(())
}

// ── Patcher: diesel.toml ──────────────────────────────────────────────────────

fn patch_diesel_toml(app: &str, pg_schema: &str, root: &PathBuf) -> Result<()> {
    let path = root.join("diesel.toml");
    let section = format!(
        "[print_schema.{pg_schema}]\nfile = \"src/{app}/schemas.rs\"\nschema = \"{pg_schema}\"\nwith_docs = false"
    );
    // Check if section already exists
    let content = fs::read_to_string(&path)?;
    if content.contains(&format!("[print_schema.{pg_schema}]")) {
        println!("  {DIM}≡  diesel.toml  (already registered){RST}");
        return Ok(());
    }
    let sep = if content.ends_with('\n') { "" } else { "\n" };
    fs::write(&path, format!("{content}{sep}\n{section}\n"))?;
    println!("  {GRN}✓{RST}  {BLD}diesel.toml{RST}  {DIM}← [print_schema.{pg_schema}]{RST}");
    Ok(())
}

// ── Patcher: src/bin/makemigrations.rs ───────────────────────────────────────

fn patch_makemigrations(app: &str, pg_schema: &str, src: &PathBuf) -> Result<()> {
    let path = src.join("bin").join("makemigrations.rs");
    let new_entry = format!("    (\"{app}/models.rs\",  \"{pg_schema}\"),");

    // Check idempotency
    let content = fs::read_to_string(&path)?;
    if content.contains(&format!("\"{app}/models.rs\"")) {
        println!("  {DIM}≡  src/bin/makemigrations.rs  (already registered){RST}");
        return Ok(());
    }

    // Insert before the closing `];` of MODEL_FILES
    let added = insert_before(&path, "];", &new_entry)?;
    if added {
        println!(
            "  {GRN}✓{RST}  {BLD}src/bin/makemigrations.rs{RST}  {DIM}← MODEL_FILES += {app}{RST}"
        );
    }
    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse args
    let mut app_name:  Option<String> = None;
    let mut pg_schema: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--schema" => {
                i += 1;
                pg_schema = args.get(i).cloned();
            }
            name if !name.starts_with("--") => {
                app_name = Some(name.to_string());
            }
            flag => {
                eprintln!("{RED}Unknown flag: {flag}{RST}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let app = match app_name {
        Some(a) => a,
        None => {
            eprintln!("{RED}Usage: cargo startapp <appname> [--schema <pgschema>]{RST}");
            eprintln!("  Example: cargo startapp products");
            eprintln!("  Example: cargo startapp products --schema catalog");
            std::process::exit(1);
        }
    };

    // Validate name: lowercase letters, numbers, underscores only
    if !app.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        || app.starts_with(|c: char| c.is_ascii_digit())
    {
        bail!(
            "{RED}Invalid app name '{app}' — use lowercase letters, digits, and underscores only.{RST}"
        );
    }

    let schema = pg_schema.unwrap_or_else(|| app.clone());
    let pascal = snake_to_pascal(&app);

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo startapp                     ║");
    println!("╚══════════════════════════════════════╝{RST}");
    println!();
    println!("  {CYN}App name  :{RST}  {BLD}{app}{RST}");
    println!("  {CYN}PG schema :{RST}  {BLD}{schema}{RST}");
    println!("  {CYN}Struct    :{RST}  {BLD}{pascal}{RST}");
    println!();

    let root = PathBuf::from(MANIFEST_DIR);
    let src  = root.join("src");
    let app_dir = src.join(&app);

    // ── Check for duplicates ─────────────────────────────────────────────────
    let registry = load_registry();
    if registry.apps.iter().any(|a| a.name == app) {
        bail!("{YLW}App '{app}' already exists.{RST}  Use a different name.");
    }
    if app_dir.exists() {
        bail!("{YLW}Directory src/{app}/ already exists.{RST}  Remove it or choose a different name.");
    }

    // ── 1. Create app directory + files ──────────────────────────────────────
    println!("{CYN}Creating app files...{RST}");
    fs::create_dir_all(&app_dir)?;

    let files: &[(&str, fn(&str, &str) -> String, bool)] = &[
        ("mod.rs",     |a, _| gen_mod_rs(a),          false),
        ("models.rs",  gen_models_rs,                   true),
        ("handler.rs", gen_handler_rs,                  true),
        ("schemas.rs", |a, _| gen_schemas_rs(a),       false),
    ];

    for (filename, gen_fn, uses_schema) in files {
        let content = if *uses_schema {
            gen_fn(&app, &schema)
        } else {
            gen_fn(&app, "")
        };
        let file_path = app_dir.join(filename);
        fs::write(&file_path, &content)?;
        println!(
            "  {GRN}✓{RST}  {BLD}src/{app}/{filename}{RST}  {DIM}({} lines){RST}",
            content.lines().count()
        );
    }

    // ── 2. Patch project files ────────────────────────────────────────────────
    println!();
    println!("{CYN}Wiring into project...{RST}");

    patch_main_rs(&app, &src)?;
    patch_models_mod(&app, &pascal, &src)?;
    patch_schema_rs(&app, &schema, &src)?;
    patch_diesel_toml(&app, &schema, &root)?;
    patch_makemigrations(&app, &schema, &src)?;

    // ── 3. Update app registry ────────────────────────────────────────────────
    let mut registry = load_registry();
    registry.apps.push(AppEntry {
        name:       app.clone(),
        pg_schema:  schema.clone(),
        models_rel: format!("{app}/models.rs"),
    });
    save_registry(&registry)?;
    println!("  {GRN}✓{RST}  {BLD}.apps.json{RST}  {DIM}← app registry updated{RST}");

    // ── 4. Next steps ─────────────────────────────────────────────────────────
    let table = pluralize(&app);

    println!();
    println!("{GRN}{BLD}✓  App '{app}' created successfully!{RST}");
    println!();
    println!("{BLD}Next steps:{RST}");
    println!("  {DIM}1.{RST}  Edit    {BLD}src/{app}/models.rs{RST}  — add your fields");
    println!("  {DIM}2.{RST}  Run     {BLD}cargo makemigrations{RST}  — generate SQL");
    println!("  {DIM}3.{RST}  Run     {BLD}cargo migrate{RST}  — apply to database");
    println!("  {DIM}4.{RST}  Wire    {BLD}src/{app}/handler.rs{RST}  into your router:");
    println!();
    println!("       {DIM}// in main.rs{RST}");
    println!("       use crate::{app}::handler as {app}_handler;");
    println!();
    println!("       Router::new()");
    println!("           .route(\"/{table}\",     get({app}_handler::list).post({app}_handler::create))");
    println!("           .route(\"/{table}/:id\", get({app}_handler::get)");
    println!("                                  .put({app}_handler::update)");
    println!("                                  .delete({app}_handler::delete))");
    println!();

    Ok(())
}
