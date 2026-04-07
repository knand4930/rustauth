// src/bin/migrate.rs
//
// Django-style "migrate" — applies, rolls back, fakes, and targets migrations.
//
//   cargo migrate                       → apply all pending migrations
//   cargo migrate --fake-initial        → mark first migration applied (no SQL)
//   cargo migrate <name>                → move DB to exactly that migration state
//   cargo migrate <name> --fake         → mark a migration applied without SQL
//   cargo migrate status                → alias for `cargo showmigrations`

use anyhow::{Context, Result};
use dotenv::dotenv;
use sqlx::{PgPool, Row};
use std::collections::HashSet;
use std::env;
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

// ── Migration history table ───────────────────────────────────────────────────

async fn ensure_history_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS public._schema_migrations (
            id         SERIAL PRIMARY KEY,
            name       VARCHAR NOT NULL UNIQUE,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await
    .context("Cannot create public._schema_migrations tracking table")?;
    Ok(())
}

async fn applied_set(pool: &PgPool) -> Result<HashSet<String>> {
    let rows = sqlx::query("SELECT name FROM public._schema_migrations")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.get::<String, _>("name")).collect())
}

async fn mark_applied(pool: &PgPool, name: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO public._schema_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING",
    )
    .bind(name)
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_unapplied(pool: &PgPool, name: &str) -> Result<()> {
    sqlx::query("DELETE FROM public._schema_migrations WHERE name = $1")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Directory helpers ─────────────────────────────────────────────────────────

fn migrations_dir() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("migrations")
}

fn migration_dirs() -> Result<Vec<std::fs::DirEntry>> {
    let mdir = migrations_dir();
    let mut entries: Vec<_> = fs::read_dir(&mdir)
        .with_context(|| format!("Cannot read migrations dir: {}", mdir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    Ok(entries)
}

fn migration_names() -> Result<Vec<String>> {
    Ok(migration_dirs()?
        .into_iter()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect())
}

// ── Risk detection ────────────────────────────────────────────────────────────

struct RiskItem {
    label:   &'static str,
    level:   &'static str,  // "HIGH" | "MEDIUM"
    message: &'static str,
}

fn detect_risks(sql: &str) -> Vec<RiskItem> {
    let up = sql.to_uppercase();
    let mut risks = vec![];
    if up.contains("DROP TABLE") {
        risks.push(RiskItem { label: "DROP TABLE",       level: "HIGH",   message: "destroys table and all its data" });
    }
    if up.contains("DROP COLUMN") {
        risks.push(RiskItem { label: "DROP COLUMN",      level: "HIGH",   message: "permanently removes column data" });
    }
    if up.contains("ALTER COLUMN") && up.contains("TYPE") {
        risks.push(RiskItem { label: "ALTER COLUMN TYPE",level: "MEDIUM", message: "may fail or lose data on incompatible casts" });
    }
    if up.contains("TRUNCATE") {
        risks.push(RiskItem { label: "TRUNCATE",         level: "HIGH",   message: "deletes every row in the table" });
    }
    if up.contains("DELETE FROM") {
        risks.push(RiskItem { label: "DELETE FROM",      level: "MEDIUM", message: "removes rows permanently" });
    }
    risks
}

// ── SQL preview printer ───────────────────────────────────────────────────────

fn print_sql_preview(sql: &str) {
    // Split on blank lines between statements, show first line of each
    let stmts: Vec<&str> = sql.split("\n\n").map(str::trim).filter(|s| !s.is_empty()).collect();
    let show_count = stmts.len().min(4);

    for stmt in &stmts[..show_count] {
        let first = stmt.lines().next().unwrap_or("").trim();
        let more  = stmt.lines().count() > 1;
        let color = {
            let up = first.to_uppercase();
            if up.starts_with("CREATE TABLE")  { GRN }
            else if up.starts_with("DROP")     { RED }
            else if up.starts_with("ALTER")    { YLW }
            else                               { DIM }
        };
        print!("     {color}{first}{RST}");
        if more { println!("  {DIM}...{RST}"); } else { println!(); }
    }
    if stmts.len() > show_count {
        println!("     {DIM}  … and {} more statement(s){RST}", stmts.len() - show_count);
    }
}

// ── Migration finder (exact + partial name match) ─────────────────────────────

fn find_migration<'a>(names: &'a [String], target: &str) -> Option<(usize, &'a str)> {
    // Exact match
    if let Some(idx) = names.iter().position(|n| n == target) {
        return Some((idx, &names[idx]));
    }
    // Partial (suffix or contains)
    let matches: Vec<(usize, &String)> = names
        .iter()
        .enumerate()
        .filter(|(_, n)| n.ends_with(target) || n.contains(target))
        .collect();
    match matches.len() {
        0 => None,
        1 => Some((matches[0].0, matches[0].1.as_str())),
        _ => {
            eprintln!("{YLW}Ambiguous name '{target}' — matches:{RST}");
            for (_, n) in &matches { eprintln!("  {n}"); }
            None
        }
    }
}

// ── Apply a single migration ──────────────────────────────────────────────────

async fn apply_one(pool: &PgPool, name: &str, fake: bool) -> Result<()> {
    let up_path = migrations_dir().join(name).join("up.sql");
    if !up_path.exists() {
        anyhow::bail!("up.sql not found for migration: {name}");
    }

    let sql = fs::read_to_string(&up_path)?;

    if sql.trim().is_empty() {
        mark_applied(pool, name).await?;
        return Ok(());
    }

    if fake {
        println!("  {YLW}⊘{RST}  {BLD}{name}{RST}  {DIM}(faked — SQL not executed){RST}");
        mark_applied(pool, name).await?;
        return Ok(());
    }

    println!("  {CYN}↑{RST}  {BLD}{name}{RST}");
    print_sql_preview(&sql);

    // Risk warnings
    for r in detect_risks(&sql) {
        let color = if r.level == "HIGH" { RED } else { YLW };
        println!("     {color}⚠ {}{RST}  {DIM}{}{RST}", r.label, r.message);
    }

    sqlx::raw_sql(&sql)
        .execute(pool)
        .await
        .with_context(|| format!("Migration failed: {name}"))?;

    mark_applied(pool, name).await?;
    println!("     {GRN}✓ Applied{RST}");
    Ok(())
}

// ── Rollback a single migration ───────────────────────────────────────────────

async fn rollback_one(pool: &PgPool, name: &str) -> Result<()> {
    let down_path = migrations_dir().join(name).join("down.sql");
    if !down_path.exists() {
        anyhow::bail!("down.sql not found for migration: {name}");
    }

    let sql = fs::read_to_string(&down_path)?;
    if sql.trim().is_empty() || sql.trim().starts_with("--") {
        anyhow::bail!(
            "down.sql for '{name}' contains no executable SQL.\n\
             Add rollback statements manually then retry."
        );
    }

    println!("  {YLW}↓{RST}  {BLD}{name}{RST}  {DIM}(rolling back){RST}");

    sqlx::raw_sql(&sql)
        .execute(pool)
        .await
        .with_context(|| format!("Rollback failed: {name}"))?;

    mark_unapplied(pool, name).await?;
    println!("     {GRN}✓ Rolled back{RST}");
    Ok(())
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Apply all pending migrations in order.
async fn cmd_run(pool: &PgPool) -> Result<()> {
    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo migrate                      ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    ensure_history_table(pool).await?;
    let applied = applied_set(pool).await?;
    let names   = migration_names()?;

    let pending: Vec<&String> = names.iter().filter(|n| !applied.contains(*n)).collect();
    if pending.is_empty() {
        println!("{GRN}✓  No pending migrations — database is up to date.{RST}\n");
        return Ok(());
    }

    println!("Applying {} pending migration(s)...\n", pending.len());
    let mut count = 0u32;
    for name in pending {
        apply_one(pool, name, false).await?;
        count += 1;
        println!();
    }

    println!("{GRN}{BLD}✓  {count} migration(s) applied successfully.{RST}\n");
    Ok(())
}

/// Mark the very first migration as applied without running its SQL.
/// Useful when the database was created manually or by another tool.
async fn cmd_fake_initial(pool: &PgPool) -> Result<()> {
    ensure_history_table(pool).await?;
    let names = migration_names()?;

    let first = names.iter().next()
        .context("No migrations found in migrations/")?;

    let applied = applied_set(pool).await?;
    if applied.contains(first) {
        println!("{YLW}⊘  '{first}' is already marked as applied.{RST}");
        return Ok(());
    }

    mark_applied(pool, first).await?;
    println!("{GRN}✓  Faked initial migration: {first}{RST}");
    println!("{DIM}   SQL was not executed — assumed DB already matches this state.{RST}");
    Ok(())
}

/// Move the database to exactly the target migration state.
/// Applies forward or rolls back as needed.
async fn cmd_target(pool: &PgPool, target: &str, fake: bool) -> Result<()> {
    ensure_history_table(pool).await?;
    let names   = migration_names()?;
    let applied = applied_set(pool).await?;

    let (target_idx, full_name) = find_migration(&names, target)
        .with_context(|| {
            format!(
                "Migration '{target}' not found.\nRun 'cargo showmigrations' to list available migrations."
            )
        })?;

    println!("\n{BLD}Target migration:{RST} {full_name}\n");

    // Migrations that need to be rolled back (applied AND after target)
    let to_rollback: Vec<&String> = names[target_idx + 1..]
        .iter()
        .filter(|n| applied.contains(*n))
        .collect::<Vec<_>>()
        .into_iter()
        .rev() // reverse order
        .collect();

    // Migrations that need to be applied (not applied AND up to+including target)
    let to_apply: Vec<&String> = names[..=target_idx]
        .iter()
        .filter(|n| !applied.contains(*n))
        .collect();

    if to_apply.is_empty() && to_rollback.is_empty() {
        println!("{GRN}✓  Already at '{full_name}' — nothing to do.{RST}\n");
        return Ok(());
    }

    if !to_rollback.is_empty() {
        println!("Rolling back {} migration(s)...\n", to_rollback.len());
        for name in &to_rollback {
            rollback_one(pool, name).await?;
            println!();
        }
    }

    if !to_apply.is_empty() {
        println!("Applying {} migration(s)...\n", to_apply.len());
        for name in &to_apply {
            apply_one(pool, name, fake).await?;
            println!();
        }
    }

    println!("{GRN}{BLD}✓  Database is now at: {full_name}{RST}\n");
    Ok(())
}

/// Show applied/pending status inline (same output as showmigrations).
async fn cmd_status(pool: &PgPool) -> Result<()> {
    ensure_history_table(pool).await?;
    let names   = migration_names()?;
    let applied = applied_set(pool).await?;

    // Fetch applied_at timestamps
    let rows = sqlx::query(
        "SELECT name, applied_at FROM public._schema_migrations ORDER BY applied_at",
    )
    .fetch_all(pool)
    .await?;
    let ts_map: std::collections::HashMap<String, chrono::DateTime<chrono::Utc>> = rows
        .into_iter()
        .map(|r| (
            r.get::<String, _>("name"),
            r.get::<chrono::DateTime<chrono::Utc>, _>("applied_at"),
        ))
        .collect();

    println!("\n{BLD}Migration status:{RST}\n");
    println!("  {BLD}{:<55}  {:<10}  {}{RST}", "Migration", "Status", "Applied at");
    println!("  {DIM}{}{RST}", "─".repeat(90));

    let mut pending_count = 0usize;
    for name in &names {
        if applied.contains(name) {
            let ts = ts_map.get(name)
                .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
                .unwrap_or_default();
            println!("  {GRN}[✓]{RST} {BLD}{name:<52}{RST}  {DIM}{ts}{RST}");
        } else {
            pending_count += 1;
            println!("  {YLW}[ ]{RST} {name:<52}  {YLW}PENDING{RST}");
        }
    }

    println!("\n  Total: {}  |  {GRN}Applied: {}{RST}  |  {YLW}Pending: {}{RST}\n",
        names.len(), names.len() - pending_count, pending_count);
    Ok(())
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set in .env")?;
    let pool = PgPool::connect(&database_url)
        .await
        .context("Cannot connect to database — check DATABASE_URL")?;

    let args: Vec<String> = env::args().collect();
    let arg1 = args.get(1).map(|s| s.as_str());
    let arg2 = args.get(2).map(|s| s.as_str());

    match (arg1, arg2) {
        // cargo migrate
        (None, _)                         => cmd_run(&pool).await?,
        // cargo migrate status
        (Some("status"), _)               => cmd_status(&pool).await?,
        // cargo migrate --fake-initial
        (Some("--fake-initial"), _)        => cmd_fake_initial(&pool).await?,
        // cargo migrate <name> --fake
        (Some(name), Some("--fake"))
            if !name.starts_with("--")   => cmd_target(&pool, name, true).await?,
        // cargo migrate <name>
        (Some(name), _)
            if !name.starts_with("--")   => cmd_target(&pool, name, false).await?,
        // unknown flag
        (Some(flag), _) => {
            eprintln!("{RED}Unknown option: {flag}{RST}\n");
            eprintln!("Usage:");
            eprintln!("  {BLD}cargo migrate{RST}                    apply all pending migrations");
            eprintln!("  {BLD}cargo migrate status{RST}             show migration status");
            eprintln!("  {BLD}cargo migrate --fake-initial{RST}     mark first migration as applied (no SQL)");
            eprintln!("  {BLD}cargo migrate <name>{RST}             move DB to that migration state");
            eprintln!("  {BLD}cargo migrate <name> --fake{RST}      mark migration as applied (no SQL)");
            std::process::exit(1);
        }
    }

    Ok(())
}
