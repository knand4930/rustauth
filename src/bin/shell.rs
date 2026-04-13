// src/bin/shell.rs
//
// Interactive SQL + inspection shell for development and testing.
//
//   cargo shell
//
// Built-in commands:
//   \tables               — list all tables across non-system schemas
//   \schema <name>        — list tables in a specific schema
//   \d <schema.table>     — describe a table (columns, types, nullable)
//   \indexes <schema.table> — list indexes on a table
//   \migrations           — show migration history
//   \count <schema.table> — row count for a table
//   \q  /  exit  /  quit  — exit the shell
//
// Anything else is executed as SQL and results are printed as an ASCII table.
// Multi-line SQL: end your statement with ; on its own line or inline.

use anyhow::{Context, Result};
use dotenv::dotenv;
use sqlx::postgres::PgRow;
use sqlx::{Column, PgPool, Row, TypeInfo, ValueRef};
use std::env;
use std::io::{self, BufRead, Write};

const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";

fn print_usage() {
    println!("Usage:");
    println!("  {BLD}cargo shell{RST}                          start the interactive SQL shell");
    println!("  {BLD}cargo shell --command \"SELECT 1\"{RST}   run one SQL statement and exit");
    println!(
        "  {BLD}cargo shell --command \"\\\\tables\"{RST}   run one built-in shell command and exit"
    );
    println!();
    println!("Built-ins: \\tables  \\schema <name>  \\d <schema.table>  \\indexes <schema.table>");
    println!("           \\migrations  \\count <schema.table>  \\q");
    println!();
}

fn validate_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn parse_table_reference(full_table: &str) -> Result<(String, String)> {
    let trimmed = full_table.trim();
    if trimmed.is_empty() {
        anyhow::bail!("A table name is required. Use schema.table or table.");
    }

    let parts: Vec<&str> = trimmed.split('.').collect();
    let (schema, table) = match parts.as_slice() {
        [table] => ("public", *table),
        [schema, table] => (*schema, *table),
        _ => anyhow::bail!("Invalid table reference '{trimmed}'. Use schema.table or table."),
    };

    if !validate_identifier(schema) || !validate_identifier(table) {
        anyhow::bail!(
            "Invalid table reference '{trimmed}'. Use only letters, digits, underscores, and one optional schema separator."
        );
    }

    Ok((schema.to_string(), table.to_string()))
}

// ── Cell extraction ────────────────────────────────────────────────────────────
// Try common postgres types in order; fall back to "<type>" label.

fn cell_str(row: &PgRow, idx: usize) -> String {
    let col = &row.columns()[idx];
    let type_name = col.type_info().name();

    // NULL check via raw value
    if row.try_get_raw(idx).map(|v| v.is_null()).unwrap_or(false) {
        return "NULL".to_string();
    }

    match type_name {
        "BOOL" => row.try_get::<bool, _>(idx).map(|v| v.to_string()),
        "INT2" => row.try_get::<i16, _>(idx).map(|v| v.to_string()),
        "INT4" => row.try_get::<i32, _>(idx).map(|v| v.to_string()),
        "INT8" => row.try_get::<i64, _>(idx).map(|v| v.to_string()),
        "FLOAT4" => row.try_get::<f32, _>(idx).map(|v| v.to_string()),
        "FLOAT8" => row.try_get::<f64, _>(idx).map(|v| v.to_string()),
        "UUID" => row
            .try_get::<sqlx::types::Uuid, _>(idx)
            .map(|v| v.to_string()),
        "TIMESTAMPTZ" | "TIMESTAMP" => row
            .try_get::<chrono::DateTime<chrono::Utc>, _>(idx)
            .map(|v| v.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(idx)
            .map(|v| v.to_string()),
        "JSONB" | "JSON" => row
            .try_get::<serde_json::Value, _>(idx)
            .map(|v| v.to_string()),
        _ => row.try_get::<String, _>(idx),
    }
    .unwrap_or_else(|_| format!("<{}>", type_name))
}

// ── ASCII table printer ────────────────────────────────────────────────────────

fn print_table(headers: &[String], rows: &[Vec<String>]) {
    if headers.is_empty() {
        println!("(no columns)");
        return;
    }

    // Compute column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let sep: String = widths
        .iter()
        .map(|&w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┼");
    let top: String = widths
        .iter()
        .map(|&w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┬");
    let bot: String = widths
        .iter()
        .map(|&w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┴");

    println!("┌{}┐", top);

    // Header
    let header_row: String = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!(" {:<w$} ", h, w = widths[i]))
        .collect::<Vec<_>>()
        .join("│");
    println!("│{}│", header_row);
    println!("├{}┤", sep);

    if rows.is_empty() {
        let empty = widths
            .iter()
            .map(|&w| " ".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("│");
        println!("│{}│", empty);
    } else {
        for row in rows {
            let row_str: String = widths
                .iter()
                .enumerate()
                .map(|(i, &w)| {
                    let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                    format!(" {:<w$} ", cell, w = w)
                })
                .collect::<Vec<_>>()
                .join("│");
            println!("│{}│", row_str);
        }
    }

    println!("└{}┘", bot);
    println!(
        "({} row{})",
        rows.len(),
        if rows.len() == 1 { "" } else { "s" }
    );
}

// ── Built-in commands ──────────────────────────────────────────────────────────

async fn cmd_tables(pool: &PgPool) -> Result<()> {
    let sql = "
        SELECT table_schema, table_name
        FROM information_schema.tables
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY table_schema, table_name";
    let rows = sqlx::query(sql).fetch_all(pool).await?;
    let headers = vec!["schema".to_string(), "table".to_string()];
    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            vec![
                r.get::<String, _>("table_schema"),
                r.get::<String, _>("table_name"),
            ]
        })
        .collect();
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_schema(pool: &PgPool, schema: &str) -> Result<()> {
    let rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 ORDER BY table_name"
    )
    .bind(schema)
    .fetch_all(pool)
    .await?;
    let headers = vec![format!("tables in {schema}")];
    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|r| vec![r.get::<String, _>("table_name")])
        .collect();
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_describe(pool: &PgPool, full_table: &str) -> Result<()> {
    let (schema, table) = parse_table_reference(full_table)?;

    let rows = sqlx::query(
        "SELECT column_name, data_type, udt_name, is_nullable, column_default
         FROM information_schema.columns
         WHERE table_schema = $1 AND table_name = $2
         ORDER BY ordinal_position",
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        println!("Table '{full_table}' not found.");
        return Ok(());
    }

    let headers = vec![
        "column".to_string(),
        "type".to_string(),
        "nullable".to_string(),
        "default".to_string(),
    ];
    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            let data_type: String = r.get("data_type");
            let udt: String = r.get("udt_name");
            let display_type = if data_type == "ARRAY" {
                format!("{}[]", udt.trim_start_matches('_'))
            } else {
                data_type
            };
            vec![
                r.get::<String, _>("column_name"),
                display_type,
                r.get::<String, _>("is_nullable"),
                r.try_get::<String, _>("column_default")
                    .unwrap_or_else(|_| "-".to_string()),
            ]
        })
        .collect();
    println!("Table: {full_table}");
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_indexes(pool: &PgPool, full_table: &str) -> Result<()> {
    let (schema, table) = parse_table_reference(full_table)?;

    let rows = sqlx::query(
        "SELECT indexname, indexdef FROM pg_indexes
         WHERE schemaname = $1 AND tablename = $2
         ORDER BY indexname",
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(pool)
    .await?;

    let headers = vec!["index_name".to_string(), "definition".to_string()];
    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            vec![
                r.get::<String, _>("indexname"),
                r.get::<String, _>("indexdef"),
            ]
        })
        .collect();
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_migrations(pool: &PgPool) -> Result<()> {
    // table may not exist yet
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables
         WHERE table_schema='public' AND table_name='_schema_migrations')",
    )
    .fetch_one(pool)
    .await?;

    if !exists {
        println!("No migration history yet. Run `cargo migrate` first.");
        return Ok(());
    }

    let rows =
        sqlx::query("SELECT name, applied_at FROM public._schema_migrations ORDER BY applied_at")
            .fetch_all(pool)
            .await?;

    let headers = vec!["migration".to_string(), "applied_at".to_string()];
    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            vec![
                r.get::<String, _>("name"),
                r.get::<chrono::DateTime<chrono::Utc>, _>("applied_at")
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            ]
        })
        .collect();
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_count(pool: &PgPool, full_table: &str) -> Result<()> {
    let (schema, table) = parse_table_reference(full_table)?;
    let qualified_table = format!("\"{schema}\".\"{table}\"");
    let sql = format!("SELECT COUNT(*) AS count FROM {qualified_table}");
    let row = sqlx::query(&sql)
        .fetch_one(pool)
        .await
        .with_context(|| format!("Cannot query {qualified_table}"))?;
    let count: i64 = row.get("count");
    println!("{qualified_table}: {count} rows");
    Ok(())
}

async fn run_shell_command(pool: &PgPool, command: &str) -> Result<()> {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return Ok(());
    }

    if trimmed.starts_with('\\') {
        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        match parts[0] {
            "\\tables" => cmd_tables(pool).await,
            "\\schema" => cmd_schema(pool, parts.get(1).copied().unwrap_or("public")).await,
            "\\d" => cmd_describe(pool, parts.get(1).copied().unwrap_or("")).await,
            "\\indexes" => cmd_indexes(pool, parts.get(1).copied().unwrap_or("")).await,
            "\\migrations" => cmd_migrations(pool).await,
            "\\count" => cmd_count(pool, parts.get(1).copied().unwrap_or("")).await,
            other => anyhow::bail!("Unknown command: {other}"),
        }
    } else {
        exec_sql(pool, trimmed.trim_end_matches(';').trim()).await
    }
}

// ── Generic SQL executor ───────────────────────────────────────────────────────

async fn exec_sql(pool: &PgPool, sql: &str) -> Result<()> {
    let sql = sql.trim();
    if sql.is_empty() {
        return Ok(());
    }

    let upper = sql.to_uppercase();

    // For SELECT / SHOW / EXPLAIN queries, fetch and display rows
    if upper.starts_with("SELECT")
        || upper.starts_with("WITH")
        || upper.starts_with("SHOW")
        || upper.starts_with("EXPLAIN")
    {
        let rows = sqlx::query(sql)
            .fetch_all(pool)
            .await
            .with_context(|| format!("SQL error"))?;

        if rows.is_empty() {
            println!("(0 rows)");
            return Ok(());
        }

        let headers: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        let data: Vec<Vec<String>> = rows
            .iter()
            .map(|r| (0..r.columns().len()).map(|i| cell_str(r, i)).collect())
            .collect();

        print_table(&headers, &data);
    } else {
        // INSERT / UPDATE / DELETE / CREATE / etc.
        let result = sqlx::raw_sql(sql)
            .execute(pool)
            .await
            .with_context(|| "SQL error")?;
        println!("OK — {} row(s) affected", result.rows_affected());
    }

    Ok(())
}

// ── Entry point ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let one_shot_command = match args.as_slice() {
        [] => None,
        [flag] if matches!(flag.as_str(), "-h" | "--help") => {
            print_usage();
            return Ok(());
        }
        [flag, command] if matches!(flag.as_str(), "-c" | "--command") => Some(command.clone()),
        _ => {
            eprintln!("Invalid shell arguments.\n");
            print_usage();
            std::process::exit(1);
        }
    };

    dotenv().ok();

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set in .env")?;
    let pool = PgPool::connect(&database_url)
        .await
        .context("Cannot connect to database")?;

    if let Some(command) = one_shot_command {
        run_shell_command(&pool, &command).await?;
        return Ok(());
    }

    let db_name = database_url.split('/').last().unwrap_or("db");

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  cargo shell  —  Interactive SQL Shell                      ║");
    println!("║  DB: {:<56}║", db_name);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  \\tables              list all tables                       ║");
    println!("║  \\schema <name>       list tables in schema                 ║");
    println!("║  \\d <schema.table>    describe table                        ║");
    println!("║  \\indexes <table>     show indexes                          ║");
    println!("║  \\migrations          show migration history                ║");
    println!("║  \\count <table>       row count                             ║");
    println!("║  \\q / exit / quit     exit                                  ║");
    println!("║  Any SQL statement    execute and display results            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let stdin = io::stdin();
    let mut buffer = String::new();

    loop {
        // Prompt
        if buffer.trim().is_empty() {
            print!("sql> ");
        } else {
            print!("  -> ");
        }
        io::stdout().flush()?;

        let mut line = String::new();
        let n = stdin.lock().read_line(&mut line)?;

        // EOF (ctrl-d)
        if n == 0 {
            println!("\nBye.");
            break;
        }

        let trimmed = line.trim();

        // Quit commands
        if matches!(trimmed, "\\q" | "exit" | "quit") {
            println!("Bye.");
            break;
        }

        // Built-in backslash commands (only on their own line)
        if trimmed.starts_with('\\') {
            if let Err(e) = run_shell_command(&pool, trimmed).await {
                eprintln!("Error: {e:#}");
            }
            buffer.clear();
            println!();
            continue;
        }

        // Accumulate multi-line SQL until we see a ";" at end of line
        buffer.push_str(&line);

        if trimmed.ends_with(';') || trimmed.is_empty() && !buffer.trim().is_empty() {
            let sql = std::mem::take(&mut buffer);
            if !sql.trim().is_empty() {
                if let Err(e) = exec_sql(&pool, sql.trim_end_matches(';').trim()).await {
                    eprintln!("Error: {e:#}");
                }
                println!();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_table_reference;

    #[test]
    fn parses_schema_qualified_table_names() {
        let parsed = parse_table_reference("user.users").expect("valid table ref");
        assert_eq!(parsed, ("user".to_string(), "users".to_string()));
    }

    #[test]
    fn rejects_invalid_table_references() {
        assert!(parse_table_reference("user.users.extra").is_err());
        assert!(parse_table_reference("user;drop").is_err());
    }
}
