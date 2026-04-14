use anyhow::{Context, Result};
use dotenv::dotenv;
use sqlx::{PgPool, Row};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

pub const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub const RST: &str = "\x1b[0m";
pub const BLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const GRN: &str = "\x1b[32m";
pub const YLW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const CYN: &str = "\x1b[36m";
pub const BLU: &str = "\x1b[34m";
pub const MAG: &str = "\x1b[35m";

pub fn load_env() {
    dotenv().ok();
}

pub fn database_url() -> Result<String> {
    load_env();
    env::var("DATABASE_URL").context("DATABASE_URL not set in .env")
}

pub async fn connect_pool() -> Result<PgPool> {
    let database_url = database_url()?;
    PgPool::connect(&database_url)
        .await
        .context("Cannot connect to database — check DATABASE_URL")
}

pub fn parse_db_name(database_url: &str) -> String {
    database_url
        .rsplit('/')
        .next()
        .unwrap_or("rustauth")
        .split('?')
        .next()
        .unwrap_or("rustauth")
        .to_string()
}

pub fn migrations_dir() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("migrations")
}

pub fn migration_sort_key(name: &str) -> (String, String) {
    let prefix = name.split('_').next().unwrap_or(name);
    let digits: String = prefix.chars().filter(|ch| ch.is_ascii_digit()).collect();
    (digits, name.to_string())
}

pub fn migration_names() -> Result<Vec<String>> {
    let mdir = migrations_dir();
    let mut entries: Vec<_> = fs::read_dir(&mdir)
        .with_context(|| format!("Cannot read migrations dir: {}", mdir.display()))?
        .filter_map(|entry| entry.ok())
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
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect())
}

pub async fn ensure_history_table(pool: &PgPool) -> Result<()> {
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

pub async fn applied_set(pool: &PgPool) -> Result<HashSet<String>> {
    let rows = sqlx::query("SELECT name FROM public._schema_migrations")
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect())
}

pub async fn applied_migration_rows(
    pool: &PgPool,
) -> Result<HashMap<String, chrono::DateTime<chrono::Utc>>> {
    let rows =
        sqlx::query("SELECT name, applied_at FROM public._schema_migrations ORDER BY applied_at")
            .fetch_all(pool)
            .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("name"),
                row.get::<chrono::DateTime<chrono::Utc>, _>("applied_at"),
            )
        })
        .collect())
}

pub async fn mark_applied(pool: &PgPool, name: &str) -> Result<()> {
    sqlx::query("INSERT INTO public._schema_migrations (name) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(name)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn mark_unapplied(pool: &PgPool, name: &str) -> Result<()> {
    sqlx::query("DELETE FROM public._schema_migrations WHERE name = $1")
        .bind(name)
        .execute(pool)
        .await?;

    Ok(())
}

pub fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub fn get_migration_modules(sql: &str) -> Vec<String> {
    let mut modules = BTreeSet::new();

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

#[cfg(test)]
mod tests {
    use super::{get_migration_modules, parse_db_name};

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

    #[test]
    fn extracts_database_name_without_query_string() {
        assert_eq!(
            parse_db_name("postgresql://postgres:secret@localhost:5432/rustauth?sslmode=disable"),
            "rustauth"
        );
    }
}
