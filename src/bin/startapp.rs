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
//       models.rs    — database model structs (sqlx::FromRow)
//       schema.rs    — request/response DTOs (data contracts)
//       handler.rs   — axum route handlers (CRUD stubs)
//
// What it patches automatically:
//   src/main.rs              — add `mod <appname>;`
//   src/models/mod.rs        — re-export structs
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
pub mod schema;

use axum::{{routing::{{delete, get, post, put}}, Router}};
use sqlx::PgPool;

/// Build the {app} routes.
pub fn routes() -> Router<PgPool> {{
    let table = \"{table}\";
    Router::new()
        .route(&format!(\"/{{}}\", table), post(handler::create))
        .route(&format!(\"/{{}}\", table), get(handler::list))
        .route(&format!(\"/{{}}/:id\", table), get(handler::get))
        .route(&format!(\"/{{}}/:id\", table), put(handler::update))
        .route(&format!(\"/{{}}/:id\", table), delete(handler::delete))
}}
",
    table = pluralize(app)
    )
}

fn gen_models_rs(app: &str, pg_schema: &str) -> String {
    let pascal = snake_to_pascal(app);
    let table  = pluralize(app);

    format!(
r#"// src/{app}/models.rs
//
// Database models — map 1:1 to PostgreSQL tables.
// Do NOT put request/response DTOs here; those belong in schema.rs.
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
use utoipa::ToSchema;
use uuid::Uuid;

/// Main model for the `{pg_schema}.{table}` table.
/// Rename or add fields — then run `cargo makemigrations`.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
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

fn gen_schema_rs(app: &str, _pg_schema: &str) -> String {
    let pascal = snake_to_pascal(app);

    format!(
r#"// src/{app}/schema.rs
//
// Request & Response DTOs (data contracts / I/O layer).
// Separated from database models to keep concerns clean.
//

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::models::{pascal};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Request schemas
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// POST — create a new {pascal}
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Create{pascal}Request {{
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}}

/// PUT — update an existing {pascal}
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Update{pascal}Request {{
    pub name:      Option<String>,
    pub is_active: Option<bool>,
}}

/// GET — query params for listing
#[derive(Debug, Deserialize)]
pub struct List{pascal}Query {{
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
    pub search:   Option<String>,
}}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Response schemas
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Public-facing response for a single {pascal}.
#[derive(Debug, Serialize, ToSchema)]
pub struct {pascal}Response {{
    pub id:         Uuid,
    pub name:       String,
    pub is_active:  bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}

impl From<{pascal}> for {pascal}Response {{
    fn from(m: {pascal}) -> Self {{
        Self {{
            id:         m.id,
            name:       m.name,
            is_active:  m.is_active,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }}
    }}
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
// Business logic & API route handlers for the `{app}` app.
// Uses: models.rs (DB) + schema.rs (I/O) + crate::response (envelopes)
//

use axum::{{
    extract::{{Path, Query, State}},
    response::IntoResponse,
    Json,
}};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::response::{{ApiMessage, ApiPaginated, ApiSuccess}};

use super::models::{pascal};
use super::schema::{{
    Create{pascal}Request, Update{pascal}Request, List{pascal}Query, {pascal}Response,
}};

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST — create
#[utoipa::path(
    post,
    path = "/api/v1/{table}",
    request_body = Create{pascal}Request,
    responses(
        (status = 201, description = "{pascal} created"),
        (status = 400, description = "Validation error"),
    ),
    tag = "{pascal}"
)]
pub async fn create(
    State(pool): State<PgPool>,
    Json(body): Json<Create{pascal}Request>,
) -> Result<impl IntoResponse, AppError> {{
    body.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let row = sqlx::query_as::<_, {pascal}>(
        "INSERT INTO {pg_schema}.{table} (id, name, is_active, created_at, updated_at)
         VALUES (gen_random_uuid(), $1, true, NOW(), NOW())
         RETURNING *"
    )
    .bind(&body.name)
    .fetch_one(&pool)
    .await?;

    let response: {pascal}Response = row.into();
    Ok(ApiSuccess::created(response))
}}

/// GET — list (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/{table}",
    params(
        ("page"     = Option<i64>, Query, description = "Page number (default 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default 20)"),
        ("search"   = Option<String>, Query, description = "Search by name"),
    ),
    responses(
        (status = 200, description = "Paginated list of {table}"),
    ),
    tag = "{pascal}"
)]
pub async fn list(
    State(pool): State<PgPool>,
    Query(params): Query<List{pascal}Query>,
) -> Result<impl IntoResponse, AppError> {{
    let page     = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset   = (page - 1) * per_page;

    let (rows, total) = if let Some(ref search) = params.search {{
        let pattern = format!("%{{search}}%");

        let rows = sqlx::query_as::<_, {pascal}>(
            "SELECT * FROM {pg_schema}.{table} WHERE name ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(&pattern).bind(per_page).bind(offset)
        .fetch_all(&pool).await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM {pg_schema}.{table} WHERE name ILIKE $1"
        )
        .bind(&pattern).fetch_one(&pool).await?;

        (rows, total)
    }} else {{
        let rows = sqlx::query_as::<_, {pascal}>(
            "SELECT * FROM {pg_schema}.{table} ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(per_page).bind(offset)
        .fetch_all(&pool).await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM {pg_schema}.{table}"
        )
        .fetch_one(&pool).await?;

        (rows, total)
    }};

    let responses: Vec<{pascal}Response> = rows.into_iter().map(Into::into).collect();
    Ok(ApiPaginated::new(responses, total, page, per_page))
}}

/// GET — fetch one
#[utoipa::path(
    get,
    path = "/api/v1/{table}/{{id}}",
    params(("id" = Uuid, Path, description = "{pascal} UUID")),
    responses(
        (status = 200, description = "{pascal} details"),
        (status = 404, description = "Not found"),
    ),
    tag = "{pascal}"
)]
pub async fn get(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {{
    let row = sqlx::query_as::<_, {pascal}>(
        "SELECT * FROM {pg_schema}.{table} WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("{pascal} {{id}} not found")))?;

    let response: {pascal}Response = row.into();
    Ok(ApiSuccess::ok(response))
}}

/// PUT — update
#[utoipa::path(
    put,
    path = "/api/v1/{table}/{{id}}",
    params(("id" = Uuid, Path, description = "{pascal} UUID")),
    request_body = Update{pascal}Request,
    responses(
        (status = 200, description = "{pascal} updated"),
        (status = 404, description = "Not found"),
    ),
    tag = "{pascal}"
)]
pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Update{pascal}Request>,
) -> Result<impl IntoResponse, AppError> {{
    let row = sqlx::query_as::<_, {pascal}>(
        "UPDATE {pg_schema}.{table}
         SET    name       = COALESCE($2, name),
                is_active  = COALESCE($3, is_active),
                updated_at = NOW()
         WHERE  id = $1
         RETURNING *"
    )
    .bind(id)
    .bind(&body.name)
    .bind(body.is_active)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("{pascal} {{id}} not found")))?;

    let response: {pascal}Response = row.into();
    Ok(ApiSuccess::ok(response))
}}

/// DELETE — delete
#[utoipa::path(
    delete,
    path = "/api/v1/{table}/{{id}}",
    params(("id" = Uuid, Path, description = "{pascal} UUID")),
    responses(
        (status = 200, description = "{pascal} deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "{pascal}"
)]
pub async fn delete(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {{
    let result = sqlx::query(
        "DELETE FROM {pg_schema}.{table} WHERE id = $1"
    )
    .bind(id)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {{
        return Err(AppError::NotFound(format!("{pascal} {{id}} not found")));
    }}

    Ok(ApiMessage::deleted("{pascal}"))
}}
"#
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
    let _table = pluralize(&app);

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

    // (filename, generator_fn, uses_schema_param)
    let files: &[(&str, fn(&str, &str) -> String, bool)] = &[
        ("mod.rs",     |a, _| gen_mod_rs(a),          false),
        ("models.rs",  gen_models_rs,                   true),
        ("schema.rs",  gen_schema_rs,                   true),
        ("handler.rs", gen_handler_rs,                  true),
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
    println!();
    println!("{GRN}{BLD}✓  App '{app}' created successfully!{RST}");
    println!();
    println!("{BLD}App structure:{RST}");
    println!("  {DIM}src/{app}/{RST}");
    println!("  {DIM}├── {RST}{BLD}mod.rs{RST}       {DIM}— module exports & route wiring{RST}");
    println!("  {DIM}├── {RST}{BLD}models.rs{RST}    {DIM}— database models (sqlx::FromRow){RST}");
    println!("  {DIM}├── {RST}{BLD}schema.rs{RST}    {DIM}— request/response DTOs (data contracts){RST}");
    println!("  {DIM}└── {RST}{BLD}handler.rs{RST}   {DIM}— API handlers & business logic{RST}");
    println!();
    println!("{BLD}Next steps:{RST}");
    println!("  {DIM}1.{RST}  Edit    {BLD}src/{app}/models.rs{RST}    — add your DB fields");
    println!("  {DIM}2.{RST}  Edit    {BLD}src/{app}/schema.rs{RST}    — define request/response types");
    println!("  {DIM}3.{RST}  Run     {BLD}cargo makemigrations{RST}    — generate SQL");
    println!("  {DIM}4.{RST}  Run     {BLD}cargo migrate{RST}            — apply to database");
    println!("  {DIM}5.{RST}  Wire    {BLD}src/{app}/handler.rs{RST}   into your router:");
    println!();
    println!("       {DIM}// in main.rs — add to the app builder:{RST}");
    println!("       .merge({app}::routes())");
    println!();
    println!("       {DIM}// in main.rs — add to OpenAPI schema:{RST}");
    println!("       {app}::handler::create,");
    println!("       {app}::handler::list,");
    println!("       {app}::handler::get,");
    println!("       {app}::handler::update,");
    println!("       {app}::handler::delete,");
    println!();

    Ok(())
}
