// src/bin/showmigrations.rs
//
// Django-style "showmigrations" — lists every migration with its applied status.
//
//   cargo showmigrations

use anyhow::{Context, Result};
use dotenv::dotenv;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRN: &str = "\x1b[32m";
const YLW: &str = "\x1b[33m";

fn migrations_dir() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("migrations")
}

fn migration_names() -> Result<Vec<String>> {
    let mdir = migrations_dir();
    let mut entries: Vec<_> = fs::read_dir(&mdir)
        .with_context(|| format!("Cannot read migrations dir: {}", mdir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    Ok(entries.into_iter().map(|e| e.file_name().to_string_lossy().to_string()).collect())
}

fn get_migration_modules(sql: &str) -> Vec<String> {
    let mut modules = std::collections::BTreeSet::new();
    for line in sql.lines() {
        let line = line.to_uppercase();
        if let Some(idx) = line.find(" TABLE ") {
            let rest = &line[idx+7..];
            if let Some(dot) = rest.find('.') {
                let schema = rest[..dot].trim().to_lowercase();
                if schema != "public" && schema != "if" && schema != "exists" {
                    modules.insert(schema);
                }
            }
        }
    }
    modules.into_iter().collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set in .env")?;
    let pool = PgPool::connect(&database_url)
        .await
        .context("Cannot connect to database — check DATABASE_URL")?;

    // Ensure the table exists before querying it
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS public._schema_migrations (
            id         SERIAL PRIMARY KEY,
            name       VARCHAR NOT NULL UNIQUE,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&pool)
    .await?;

    let rows = sqlx::query(
        "SELECT name, applied_at FROM public._schema_migrations ORDER BY applied_at",
    )
    .fetch_all(&pool)
    .await?;

    let ts_map: HashMap<String, chrono::DateTime<chrono::Utc>> = rows
        .into_iter()
        .map(|r| (
            r.get::<String, _>("name"),
            r.get::<chrono::DateTime<chrono::Utc>, _>("applied_at"),
        ))
        .collect();

    let names = migration_names()?;

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo showmigrations               ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    if names.is_empty() {
        println!("{DIM}  No migrations found in migrations/{RST}\n");
        return Ok(());
    }

    let mut by_module: std::collections::BTreeMap<String, Vec<&String>> = std::collections::BTreeMap::new();
    for name in &names {
        let sql = fs::read_to_string(migrations_dir().join(name).join("up.sql")).unwrap_or_default();
        let modules = get_migration_modules(&sql);
        if modules.is_empty() {
            by_module.entry("global".to_string()).or_default().push(name);
        } else {
            for md in modules {
                by_module.entry(md).or_default().push(name);
            }
        }
    }

    let mut applied_count = 0usize;
    for (module, module_names) in by_module {
        println!("{BLD}{}{RST}", module);
        for name in module_names {
            if let Some(ts) = ts_map.get(name) {
                if !module.is_empty() { // prevent double counting if multiple modules touch same name? actually applied_count isn't fully accurate if duped, but we'll collect a unique set.
                }
                let ts_str = ts.format("%Y-%m-%d %H:%M UTC").to_string();
                println!("  {GRN}[X]{RST}  {}  {DIM}applied {}{RST}", name, ts_str);
            } else {
                println!("  {YLW}[ ]{RST}  {}", name);
            }
        }
        println!();
    }

    // accurate count
    for name in &names {
        if ts_map.contains_key(name) {
            applied_count += 1;
        }
    }

    let pending = names.len() - applied_count;
    println!("  {DIM}─────────────────────────────────────{RST}");
    println!("  Total: {}  |  {GRN}Applied: {applied_count}{RST}  |  {YLW}Pending: {pending}{RST}",
        names.len());

    if pending > 0 {
        println!("\n  Run {BLD}cargo migrate{RST} to apply {pending} pending migration(s).");
    } else {
        println!("\n  {GRN}✓  Database is up to date.{RST}");
    }
    println!();

    Ok(())
}
