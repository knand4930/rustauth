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

fn print_usage() {
    println!("Usage:");
    println!("  {BLD}cargo showmigrations{RST}             list migrations with applied status");
    println!();
}

fn migrations_dir() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("migrations")
}

fn migration_sort_key(name: &str) -> (String, String) {
    let prefix = name.split('_').next().unwrap_or(name);
    let digits: String = prefix.chars().filter(|ch| ch.is_ascii_digit()).collect();
    (digits, name.to_string())
}

fn migration_names() -> Result<Vec<String>> {
    let mdir = migrations_dir();
    let mut entries: Vec<_> = fs::read_dir(&mdir)
        .with_context(|| format!("Cannot read migrations dir: {}", mdir.display()))?
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_dir()
                && path.join("up.sql").exists()
                && entry
                    .file_name()
                    .to_str()
                    .and_then(|name| name.chars().next())
                    .is_some_and(|first| first.is_ascii_digit())
        })
        .collect();
    entries.sort_by_key(|entry| migration_sort_key(&entry.file_name().to_string_lossy()));
    Ok(entries
        .into_iter()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect())
}

fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn get_migration_modules(sql: &str) -> Vec<String> {
    let mut modules = std::collections::BTreeSet::new();

    for line in sql.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        let lower = trimmed.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();
        if tokens.len() >= 3 && tokens[0] == "create" && tokens[1] == "schema" {
            let schema = match tokens.as_slice() {
                ["create", "schema", "if", "not", "exists", schema, ..] => *schema,
                ["create", "schema", schema, ..] => *schema,
                _ => "",
            }
            .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_');

            if is_identifier(schema) && schema != "public" {
                modules.insert(schema.to_string());
            }
        }

        for token in trimmed.split_whitespace() {
            let cleaned = token
                .trim_matches(|ch: char| {
                    !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.' && ch != '"'
                })
                .trim_matches('"');

            if let Some((schema, _)) = cleaned.split_once('.') {
                let schema = schema.trim_matches('"').to_lowercase();
                if is_identifier(&schema) && schema != "public" {
                    modules.insert(schema);
                }
            }
        }
    }

    modules.into_iter().collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => {}
        [flag] if matches!(flag.as_str(), "-h" | "--help") => {
            print_usage();
            return Ok(());
        }
        _ => {
            eprintln!("{YLW}showmigrations does not accept extra arguments.{RST}\n");
            print_usage();
            std::process::exit(1);
        }
    }

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

    let rows =
        sqlx::query("SELECT name, applied_at FROM public._schema_migrations ORDER BY applied_at")
            .fetch_all(&pool)
            .await?;

    let ts_map: HashMap<String, chrono::DateTime<chrono::Utc>> = rows
        .into_iter()
        .map(|r| {
            (
                r.get::<String, _>("name"),
                r.get::<chrono::DateTime<chrono::Utc>, _>("applied_at"),
            )
        })
        .collect();

    let names = migration_names()?;

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo showmigrations               ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    if names.is_empty() {
        println!("{DIM}  No migrations found in migrations/{RST}\n");
        return Ok(());
    }

    println!(
        "  {BLD}{:<4} {:<48} {}{RST}",
        "Stat", "Migration", "Modules"
    );
    println!("  {DIM}{}{RST}", "─".repeat(90));

    let mut applied_count = 0usize;
    for name in &names {
        let sql =
            fs::read_to_string(migrations_dir().join(name).join("up.sql")).unwrap_or_default();
        let modules = get_migration_modules(&sql);
        let module_label = if modules.is_empty() {
            "global".to_string()
        } else {
            modules.join(", ")
        };

        if let Some(ts) = ts_map.get(name) {
            applied_count += 1;
            let ts_str = ts.format("%Y-%m-%d %H:%M UTC").to_string();
            println!(
                "  {GRN}[X]{RST}  {:<48} {DIM}{}  (applied {}){RST}",
                name, module_label, ts_str
            );
        } else {
            println!("  {YLW}[ ]{RST}  {:<48} {}", name, module_label);
        }
    }

    let pending = names.len() - applied_count;
    println!("  {DIM}─────────────────────────────────────{RST}");
    println!(
        "  Total: {}  |  {GRN}Applied: {applied_count}{RST}  |  {YLW}Pending: {pending}{RST}",
        names.len()
    );

    if pending > 0 {
        println!("\n  Run {BLD}cargo migrate{RST} to apply {pending} pending migration(s).");
    } else {
        println!("\n  {GRN}✓  Database is up to date.{RST}");
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::get_migration_modules;

    #[test]
    fn parses_schema_and_table_modules_without_if_not_exists_noise() {
        let sql = r#"
            CREATE SCHEMA IF NOT EXISTS blogs;
            CREATE TABLE IF NOT EXISTS blogs.blog_posts (
                id UUID PRIMARY KEY
            );
        "#;

        assert_eq!(get_migration_modules(sql), vec!["blogs".to_string()]);
    }

    #[test]
    fn collects_multiple_distinct_modules_once_each() {
        let sql = r#"
            CREATE TABLE user.users (id UUID PRIMARY KEY);
            ALTER TABLE blogs.blog_posts ADD COLUMN author_id UUID REFERENCES user.users(id);
        "#;

        assert_eq!(
            get_migration_modules(sql),
            vec!["blogs".to_string(), "user".to_string()]
        );
    }
}
