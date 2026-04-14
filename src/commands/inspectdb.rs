use crate::commands::common::{BLD, CYN, DIM, GRN, RED, RST, YLW, connect_pool};
use anyhow::Result;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

fn print_usage() {
    println!("Usage:");
    println!("  {BLD}cargo inspectdb{RST}                      introspect all schemas → print Rust model stubs");
    println!("  {BLD}cargo inspectdb --schema <name>{RST}      introspect a single schema only");
    println!("  {BLD}cargo inspectdb --table <schema.table>{RST}  introspect one table only");
    println!("  {BLD}cargo inspectdb --output src/apps/foo/models.rs{RST}  write output to file");
    println!();
}

/// Map a PostgreSQL data type to the appropriate Rust type string.
fn pg_to_rust(pg_type: &str, is_nullable: bool) -> String {
    let inner = match pg_type.to_uppercase().trim_start_matches("_") {
        "UUID" => "Uuid",
        "VARCHAR" | "TEXT" | "BPCHAR" | "CHAR" | "NAME" | "CITEXT" => "String",
        "BOOL" | "BOOLEAN" => "bool",
        "INT2" | "SMALLINT" | "SMALLSERIAL" => "i16",
        "INT4" | "INTEGER" | "SERIAL" => "i32",
        "INT8" | "BIGINT" | "BIGSERIAL" => "i64",
        "FLOAT4" | "REAL" => "f32",
        "FLOAT8" | "DOUBLE PRECISION" | "NUMERIC" | "DECIMAL" => "f64",
        "TIMESTAMPTZ" | "TIMESTAMP WITH TIME ZONE" => "DateTime<Utc>",
        "TIMESTAMP" | "TIMESTAMP WITHOUT TIME ZONE" => "DateTime<Utc>",
        "DATE" => "chrono::NaiveDate",
        "TIME" | "TIME WITHOUT TIME ZONE" => "chrono::NaiveTime",
        "JSONB" | "JSON" => "serde_json::Value",
        "BYTEA" => "Vec<u8>",
        _ => "serde_json::Value", // fallback for arrays, custom types, etc.
    };
    // Handle array types (pg type starts with "_")
    let inner = if pg_type.starts_with('_') {
        &format!("Vec<{inner}>")
    } else {
        inner
    };
    if is_nullable {
        format!("Option<{inner}>")
    } else {
        inner.to_string()
    }
}

fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

struct ColumnInfo {
    name: String,
    pg_type: String,
    udt_name: String,
    is_nullable: bool,
    column_default: Option<String>,
    is_pk: bool,
    fk_target: Option<String>,
    has_unique: bool,
    has_index: bool,
}

async fn fetch_tables(pool: &PgPool, schema_filter: Option<&str>) -> Result<Vec<(String, String)>> {
    let excluded = vec![
        "pg_catalog",
        "information_schema",
        "pg_toast",
        "public",
    ];

    let rows = if let Some(schema) = schema_filter {
        sqlx::query(
            "SELECT table_schema, table_name
             FROM information_schema.tables
             WHERE table_type = 'BASE TABLE'
               AND table_schema = $1
             ORDER BY table_schema, table_name",
        )
        .bind(schema)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT table_schema, table_name
             FROM information_schema.tables
             WHERE table_type = 'BASE TABLE'
               AND table_schema != ALL($1)
             ORDER BY table_schema, table_name",
        )
        .bind(&excluded)
        .fetch_all(pool)
        .await?
    };

    Ok(rows
        .iter()
        .map(|row| {
            (
                row.get::<String, _>("table_schema"),
                row.get::<String, _>("table_name"),
            )
        })
        .collect())
}

async fn fetch_pk_columns(
    pool: &PgPool,
    schema: &str,
    table: &str,
) -> Result<Vec<String>> {
    let rows = sqlx::query(
        "SELECT kcu.column_name
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
           ON tc.constraint_name = kcu.constraint_name
          AND tc.table_schema = kcu.table_schema
         WHERE tc.constraint_type = 'PRIMARY KEY'
           AND tc.table_schema = $1
           AND tc.table_name = $2
         ORDER BY kcu.ordinal_position",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| row.get::<String, _>("column_name"))
        .collect())
}

async fn fetch_unique_columns(pool: &PgPool, schema: &str, table: &str) -> Result<Vec<String>> {
    let rows = sqlx::query(
        "SELECT DISTINCT kcu.column_name
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
           ON tc.constraint_name = kcu.constraint_name
          AND tc.table_schema = kcu.table_schema
         WHERE tc.constraint_type = 'UNIQUE'
           AND tc.table_schema = $1
           AND tc.table_name = $2",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| row.get::<String, _>("column_name"))
        .collect())
}

async fn fetch_indexed_columns(pool: &PgPool, schema: &str, table: &str) -> Result<Vec<String>> {
    // Return columns that appear in a non-unique, non-pk index
    let rows = sqlx::query(
        "SELECT a.attname AS column_name
         FROM pg_index ix
         JOIN pg_class t ON t.oid = ix.indrelid
         JOIN pg_class i ON i.oid = ix.indexrelid
         JOIN pg_namespace n ON n.oid = t.relnamespace
         JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
         WHERE n.nspname = $1
           AND t.relname = $2
           AND NOT ix.indisprimary
           AND NOT ix.indisunique",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| row.get::<String, _>("column_name"))
        .collect())
}

async fn fetch_fk_map(pool: &PgPool, schema: &str, table: &str) -> Result<HashMap<String, String>> {
    let rows = sqlx::query(
        "SELECT kcu.column_name, ccu.table_schema AS fk_schema, ccu.table_name AS fk_table
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
           ON tc.constraint_name = kcu.constraint_name
          AND tc.table_schema = kcu.table_schema
         JOIN information_schema.constraint_column_usage ccu
           ON ccu.constraint_name = tc.constraint_name
         WHERE tc.constraint_type = 'FOREIGN KEY'
           AND tc.table_schema = $1
           AND tc.table_name = $2",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| {
            let col: String = row.get("column_name");
            let fk_schema: String = row.get("fk_schema");
            let fk_table: String = row.get("fk_table");
            (col, format!("{fk_schema}.{fk_table}"))
        })
        .collect())
}

async fn fetch_columns(
    pool: &PgPool,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>> {
    let rows = sqlx::query(
        "SELECT column_name, data_type, udt_name, is_nullable, column_default
         FROM information_schema.columns
         WHERE table_schema = $1 AND table_name = $2
         ORDER BY ordinal_position",
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;

    let pk_cols = fetch_pk_columns(pool, schema, table).await?;
    let unique_cols = fetch_unique_columns(pool, schema, table).await?;
    let indexed_cols = fetch_indexed_columns(pool, schema, table).await?;
    let fk_map = fetch_fk_map(pool, schema, table).await?;

    Ok(rows
        .iter()
        .map(|row| {
            let name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let udt_name: String = row.get("udt_name");
            let is_nullable: bool = row.get::<String, _>("is_nullable") == "YES";
            let column_default: Option<String> = row.try_get("column_default").ok().flatten();
            let is_pk = pk_cols.contains(&name);
            let has_unique = unique_cols.contains(&name);
            let has_index = indexed_cols.contains(&name);
            let fk_target = fk_map.get(&name).cloned();

            ColumnInfo {
                name,
                pg_type: data_type,
                udt_name,
                is_nullable,
                column_default,
                is_pk,
                fk_target,
                has_unique,
                has_index,
            }
        })
        .collect())
}

fn effective_pg_type(col: &ColumnInfo) -> &str {
    // For user-defined types, prefer udt_name
    match col.pg_type.as_str() {
        "USER-DEFINED" | "ARRAY" => &col.udt_name,
        other => other,
    }
}

fn generate_model_block(schema: &str, table: &str, columns: &[ColumnInfo]) -> String {
    let struct_name = snake_to_pascal(table);
    let mut lines: Vec<String> = vec![];

    // Per-struct directives
    lines.push(format!("// @table {table}"));
    lines.push(format!(
        "#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]"
    ));
    lines.push(format!("pub struct {struct_name} {{"));

    for col in columns {
        let pg_type = effective_pg_type(col);
        let rust_type = pg_to_rust(pg_type, col.is_nullable && !col.is_pk);

        // Field-level directives
        if col.is_pk {
            // PK has no extra directive — it's detected by field name "id"
        } else if let Some(target) = &col.fk_target {
            lines.push(format!("    // @references {target}"));
        }

        if col.has_unique && !col.is_pk {
            lines.push("    // @unique".to_string());
        }
        if col.has_index && !col.is_pk {
            lines.push("    // @index".to_string());
        }
        if let Some(default) = &col.column_default {
            // Skip serial/sequence defaults (auto-handled)
            if !default.contains("nextval(") && !default.contains("gen_random_uuid()") {
                lines.push(format!("    // @default {default}"));
            }
        }

        lines.push(format!("    pub {}: {rust_type},", col.name));
    }

    lines.push("}".to_string());
    lines.push(String::new());
    lines.push(format!(
        "crate::declare_model_table!({struct_name}, \"{schema}\", \"{table}\");"
    ));

    lines.join("\n")
}

pub async fn run(args: &[String]) -> Result<()> {
    let mut schema_filter: Option<String> = None;
    let mut table_filter: Option<(String, String)> = None;
    let mut output_file: Option<String> = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            "--schema" => {
                i += 1;
                schema_filter = args.get(i).cloned();
            }
            "--table" => {
                i += 1;
                if let Some(val) = args.get(i) {
                    if let Some((s, t)) = val.split_once('.') {
                        table_filter = Some((s.to_string(), t.to_string()));
                    } else {
                        eprintln!("{RED}--table requires schema.table format{RST}");
                        std::process::exit(1);
                    }
                }
            }
            "--output" => {
                i += 1;
                output_file = args.get(i).cloned();
            }
            flag => {
                eprintln!("{RED}Unknown flag: {flag}{RST}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo inspectdb                    ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    let pool = connect_pool().await?;

    let tables = if let Some((s, t)) = &table_filter {
        vec![(s.clone(), t.clone())]
    } else {
        fetch_tables(&pool, schema_filter.as_deref()).await?
    };

    if tables.is_empty() {
        println!("{YLW}No tables found matching the filter.{RST}");
        return Ok(());
    }

    println!("{CYN}Found {} table(s). Generating model stubs...{RST}\n", tables.len());

    let mut output_blocks: Vec<String> = vec![];

    // Group tables by schema
    let mut current_schema = String::new();
    for (schema, table) in &tables {
        if *schema != current_schema {
            if !output_blocks.is_empty() {
                output_blocks.push(String::new());
            }
            output_blocks.push(format!("// ─── {} ─────────────────────────────────────────────────────────────────────", schema.to_uppercase()));
            output_blocks.push(format!("// @schema {schema}"));
            output_blocks.push(String::new());
            current_schema = schema.clone();
        }

        let columns = fetch_columns(&pool, schema, table).await?;
        let block = generate_model_block(schema, table, &columns);
        output_blocks.push(block);
        output_blocks.push(String::new());

        println!("  {GRN}✓{RST}  {BLD}{schema}.{table}{RST}  {DIM}({} columns){RST}", columns.len());
    }

    // Build file header
    let header = r#"// This file was generated by `cargo inspectdb`.
// Review and adjust types, directives, and derives before committing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

"#;

    let full_output = format!("{}{}", header, output_blocks.join("\n"));

    if let Some(path) = &output_file {
        std::fs::write(path, &full_output)?;
        println!("\n{GRN}{BLD}✓  Written to {path}{RST}");
        println!("{DIM}Review the file and adjust types/directives before running cargo makemigrations.{RST}\n");
    } else {
        println!("\n{CYN}{BLD}─── Generated models (stdout) ──────────────────────────────────{RST}");
        println!("{full_output}");
        println!("{DIM}Tip: use --output src/apps/<app>/models.rs to write directly to a file.{RST}\n");
    }

    Ok(())
}
