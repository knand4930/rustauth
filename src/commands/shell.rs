use crate::commands::common::{BLD, RST, connect_pool, database_url, parse_db_name};
use anyhow::{Context, Result};
use sqlx::postgres::PgRow;
use sqlx::{Column, PgPool, Row, TypeInfo, ValueRef};
use std::io::{self, BufRead, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShellFlavor {
    Shell,
    DbShell,
}

impl ShellFlavor {
    fn command_name(self) -> &'static str {
        match self {
            Self::Shell => "shell",
            Self::DbShell => "dbshell",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Shell => "Interactive SQL Shell",
            Self::DbShell => "Database Shell",
        }
    }
}

fn print_usage(flavor: ShellFlavor) {
    let command = flavor.command_name();
    println!("Usage:");
    println!(
        "  {BLD}cargo {command}{RST}                          start the interactive SQL shell"
    );
    println!("  {BLD}cargo {command} --command \"SELECT 1\"{RST}   run one SQL statement and exit");
    println!(
        "  {BLD}cargo {command} --command \"\\\\tables\"{RST}   run one built-in shell command and exit"
    );
    println!();
    println!("Built-ins: \\tables  \\dt [schema.*]  \\dn  \\schema <name>  \\d <schema.table>");
    println!("           \\indexes <schema.table>  \\migrations  \\count <schema.table>  \\q");
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

fn cell_str(row: &PgRow, idx: usize) -> String {
    let column = &row.columns()[idx];
    let type_name = column.type_info().name();

    if row
        .try_get_raw(idx)
        .map(|value| value.is_null())
        .unwrap_or(false)
    {
        return "NULL".to_string();
    }

    match type_name {
        "BOOL" => row.try_get::<bool, _>(idx).map(|value| value.to_string()),
        "INT2" => row.try_get::<i16, _>(idx).map(|value| value.to_string()),
        "INT4" => row.try_get::<i32, _>(idx).map(|value| value.to_string()),
        "INT8" => row.try_get::<i64, _>(idx).map(|value| value.to_string()),
        "FLOAT4" => row.try_get::<f32, _>(idx).map(|value| value.to_string()),
        "FLOAT8" => row.try_get::<f64, _>(idx).map(|value| value.to_string()),
        "UUID" => row
            .try_get::<sqlx::types::Uuid, _>(idx)
            .map(|value| value.to_string()),
        "TIMESTAMPTZ" => row
            .try_get::<chrono::DateTime<chrono::Utc>, _>(idx)
            .map(|value| value.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        "TIMESTAMP" => row
            .try_get::<chrono::NaiveDateTime, _>(idx)
            .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string()),
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(idx)
            .map(|value| value.to_string()),
        "JSONB" | "JSON" => row
            .try_get::<serde_json::Value, _>(idx)
            .map(|value| value.to_string()),
        "VARCHAR" | "TEXT" | "BPCHAR" | "NAME" => row.try_get::<String, _>(idx),
        _ => row.try_get::<String, _>(idx),
    }
    .unwrap_or_else(|_| format!("<{}>", type_name))
}

fn print_table(headers: &[String], rows: &[Vec<String>]) {
    if headers.is_empty() {
        println!("(no columns)");
        return;
    }

    let mut widths: Vec<usize> = headers.iter().map(|header| header.len()).collect();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if index < widths.len() {
                widths[index] = widths[index].max(cell.len());
            }
        }
    }

    let separator: String = widths
        .iter()
        .map(|width| "─".repeat(width + 2))
        .collect::<Vec<_>>()
        .join("┼");
    let top: String = widths
        .iter()
        .map(|width| "─".repeat(width + 2))
        .collect::<Vec<_>>()
        .join("┬");
    let bottom: String = widths
        .iter()
        .map(|width| "─".repeat(width + 2))
        .collect::<Vec<_>>()
        .join("┴");

    println!("┌{}┐", top);

    let header_row: String = headers
        .iter()
        .enumerate()
        .map(|(index, header)| format!(" {:<width$} ", header, width = widths[index]))
        .collect::<Vec<_>>()
        .join("│");
    println!("│{}│", header_row);
    println!("├{}┤", separator);

    if rows.is_empty() {
        let empty = widths
            .iter()
            .map(|width| " ".repeat(width + 2))
            .collect::<Vec<_>>()
            .join("│");
        println!("│{}│", empty);
    } else {
        for row in rows {
            let line = widths
                .iter()
                .enumerate()
                .map(|(index, width)| {
                    let cell = row.get(index).map(|value| value.as_str()).unwrap_or("");
                    format!(" {:<width$} ", cell, width = width)
                })
                .collect::<Vec<_>>()
                .join("│");
            println!("│{}│", line);
        }
    }

    println!("└{}┘", bottom);
    println!(
        "({} row{})",
        rows.len(),
        if rows.len() == 1 { "" } else { "s" }
    );
}

async fn cmd_tables(pool: &PgPool, schema: Option<&str>) -> Result<()> {
    let rows = if let Some(schema_name) = schema {
        sqlx::query(
            "SELECT table_schema, table_name
             FROM information_schema.tables
             WHERE table_schema = $1
             ORDER BY table_schema, table_name",
        )
        .bind(schema_name)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT table_schema, table_name
             FROM information_schema.tables
             WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
             ORDER BY table_schema, table_name",
        )
        .fetch_all(pool)
        .await?
    };

    let headers = vec!["schema".to_string(), "table".to_string()];
    let data = rows
        .iter()
        .map(|row| {
            vec![
                row.get::<String, _>("table_schema"),
                row.get::<String, _>("table_name"),
            ]
        })
        .collect::<Vec<_>>();
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_schemas(pool: &PgPool) -> Result<()> {
    let rows = sqlx::query(
        "SELECT schema_name
         FROM information_schema.schemata
         WHERE schema_name NOT IN ('pg_catalog', 'information_schema')
         ORDER BY schema_name",
    )
    .fetch_all(pool)
    .await?;

    let headers = vec!["schema".to_string()];
    let data = rows
        .iter()
        .map(|row| vec![row.get::<String, _>("schema_name")])
        .collect::<Vec<_>>();
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_schema(pool: &PgPool, schema: &str) -> Result<()> {
    cmd_tables(pool, Some(schema)).await
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
    let data = rows
        .iter()
        .map(|row| {
            let data_type: String = row.get("data_type");
            let udt_name: String = row.get("udt_name");
            let display_type = if data_type == "ARRAY" {
                format!("{}[]", udt_name.trim_start_matches('_'))
            } else {
                data_type
            };
            vec![
                row.get::<String, _>("column_name"),
                display_type,
                row.get::<String, _>("is_nullable"),
                row.try_get::<String, _>("column_default")
                    .unwrap_or_else(|_| "-".to_string()),
            ]
        })
        .collect::<Vec<_>>();

    println!("Table: {full_table}");
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_indexes(pool: &PgPool, full_table: &str) -> Result<()> {
    let (schema, table) = parse_table_reference(full_table)?;

    let rows = sqlx::query(
        "SELECT indexname, indexdef
         FROM pg_indexes
         WHERE schemaname = $1 AND tablename = $2
         ORDER BY indexname",
    )
    .bind(&schema)
    .bind(&table)
    .fetch_all(pool)
    .await?;

    let headers = vec!["index_name".to_string(), "definition".to_string()];
    let data = rows
        .iter()
        .map(|row| {
            vec![
                row.get::<String, _>("indexname"),
                row.get::<String, _>("indexdef"),
            ]
        })
        .collect::<Vec<_>>();
    print_table(&headers, &data);
    Ok(())
}

async fn cmd_migrations(pool: &PgPool) -> Result<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = '_schema_migrations'
        )",
    )
    .fetch_one(pool)
    .await?;

    if !exists {
        println!("No migration history yet. Run `cargo migrate` first.");
        return Ok(());
    }

    let rows = sqlx::query(
        "SELECT name, applied_at
         FROM public._schema_migrations
         ORDER BY applied_at",
    )
    .fetch_all(pool)
    .await?;

    let headers = vec!["migration".to_string(), "applied_at".to_string()];
    let data = rows
        .iter()
        .map(|row| {
            vec![
                row.get::<String, _>("name"),
                row.get::<chrono::DateTime<chrono::Utc>, _>("applied_at")
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            ]
        })
        .collect::<Vec<_>>();
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

fn normalize_dt_target(raw: Option<&str>) -> Option<&str> {
    raw.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else if let Some(schema) = trimmed.strip_suffix(".*") {
            Some(schema)
        } else {
            Some(trimmed)
        }
    })
}

async fn run_shell_command(pool: &PgPool, command: &str) -> Result<()> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if trimmed.starts_with('\\') {
        let mut parts = trimmed.split_whitespace();
        let builtin = parts.next().unwrap_or(trimmed);
        let arg = parts.collect::<Vec<_>>().join(" ");
        let arg = if arg.trim().is_empty() {
            None
        } else {
            Some(arg.as_str())
        };

        match builtin {
            "\\tables" => cmd_tables(pool, None).await,
            "\\dt" => {
                let target = normalize_dt_target(arg);
                match target {
                    Some(schema) if validate_identifier(schema) => {
                        cmd_tables(pool, Some(schema)).await
                    }
                    Some(other) => anyhow::bail!(
                        "Invalid schema selector '{other}'. Use \\dt or \\dt <schema>.*."
                    ),
                    None => cmd_tables(pool, None).await,
                }
            }
            "\\dn" => cmd_schemas(pool).await,
            "\\schema" => cmd_schema(pool, arg.unwrap_or("public")).await,
            "\\d" => cmd_describe(pool, arg.unwrap_or("")).await,
            "\\indexes" => cmd_indexes(pool, arg.unwrap_or("")).await,
            "\\migrations" => cmd_migrations(pool).await,
            "\\count" => cmd_count(pool, arg.unwrap_or("")).await,
            "\\help" => {
                print_usage(ShellFlavor::Shell);
                Ok(())
            }
            other => anyhow::bail!("Unknown command: {other}"),
        }
    } else {
        exec_sql(pool, trimmed.trim_end_matches(';').trim()).await
    }
}

async fn exec_sql(pool: &PgPool, sql: &str) -> Result<()> {
    let sql = sql.trim();
    if sql.is_empty() {
        return Ok(());
    }

    let upper = sql.to_uppercase();
    if upper.starts_with("SELECT")
        || upper.starts_with("WITH")
        || upper.starts_with("SHOW")
        || upper.starts_with("EXPLAIN")
    {
        let rows = sqlx::query(sql)
            .fetch_all(pool)
            .await
            .context("SQL error")?;

        if rows.is_empty() {
            println!("(0 rows)");
            return Ok(());
        }

        let headers = rows[0]
            .columns()
            .iter()
            .map(|column| column.name().to_string())
            .collect::<Vec<_>>();
        let data = rows
            .iter()
            .map(|row| {
                (0..row.columns().len())
                    .map(|index| cell_str(row, index))
                    .collect()
            })
            .collect::<Vec<Vec<String>>>();
        print_table(&headers, &data);
    } else {
        let result = sqlx::raw_sql(sql)
            .execute(pool)
            .await
            .context("SQL error")?;
        println!("OK — {} row(s) affected", result.rows_affected());
    }

    Ok(())
}

async fn run(flavor: ShellFlavor, args: &[String]) -> Result<()> {
    let one_shot_command = match args {
        [] => None,
        [flag] if matches!(flag.as_str(), "-h" | "--help") => {
            print_usage(flavor);
            return Ok(());
        }
        [flag, command] if matches!(flag.as_str(), "-c" | "--command") => Some(command.clone()),
        _ => {
            eprintln!("Invalid {} arguments.\n", flavor.command_name());
            print_usage(flavor);
            std::process::exit(1);
        }
    };

    let database_url = database_url()?;
    let pool = connect_pool().await?;

    if let Some(command) = one_shot_command {
        run_shell_command(&pool, &command).await?;
        return Ok(());
    }

    let db_name = parse_db_name(&database_url);
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!(
        "║  cargo {:<8} — {:<41}║",
        flavor.command_name(),
        flavor.title()
    );
    println!("║  DB: {:<56}║", db_name);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  \\tables / \\dt       list tables                            ║");
    println!("║  \\dt schema.*        list tables in a schema                ║");
    println!("║  \\dn                 list schemas                           ║");
    println!("║  \\d <schema.table>   describe table                         ║");
    println!("║  \\indexes <table>    show indexes                           ║");
    println!("║  \\migrations         show migration history                 ║");
    println!("║  \\count <table>      row count                              ║");
    println!("║  \\q / exit / quit    exit                                   ║");
    println!("║  Any SQL statement   execute and display results            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let stdin = io::stdin();
    let mut buffer = String::new();

    loop {
        if buffer.trim().is_empty() {
            print!("sql> ");
        } else {
            print!("  -> ");
        }
        io::stdout().flush()?;

        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;
        if bytes_read == 0 {
            println!("\nBye.");
            break;
        }

        let trimmed = line.trim();
        if matches!(trimmed, "\\q" | "exit" | "quit") {
            println!("Bye.");
            break;
        }

        if trimmed.starts_with('\\') {
            if let Err(error) = run_shell_command(&pool, trimmed).await {
                eprintln!("Error: {error:#}");
            }
            buffer.clear();
            println!();
            continue;
        }

        buffer.push_str(&line);
        if trimmed.ends_with(';') || (trimmed.is_empty() && !buffer.trim().is_empty()) {
            let sql = std::mem::take(&mut buffer);
            if !sql.trim().is_empty() {
                if let Err(error) = exec_sql(&pool, sql.trim_end_matches(';').trim()).await {
                    eprintln!("Error: {error:#}");
                }
                println!();
            }
        }
    }

    Ok(())
}

pub async fn run_shell(args: &[String]) -> Result<()> {
    run(ShellFlavor::Shell, args).await
}

pub async fn run_dbshell(args: &[String]) -> Result<()> {
    run(ShellFlavor::DbShell, args).await
}

#[cfg(test)]
mod tests {
    use super::{normalize_dt_target, parse_table_reference};

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

    #[test]
    fn normalizes_dt_schema_targets() {
        assert_eq!(normalize_dt_target(Some("blogs.*")), Some("blogs"));
        assert_eq!(normalize_dt_target(Some("blogs")), Some("blogs"));
        assert_eq!(normalize_dt_target(Some("   ")), None);
    }
}
