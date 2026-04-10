// src/bin/makemigrations.rs
//
// Django-style "makemigrations" — reads Rust model structs, validates them,
// diffs against last known state, and generates SQL migration files.
//
//   cargo makemigrations              → label defaults to "auto"
//   cargo makemigrations add_bio      → custom label

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

// We discover app models directly from src/apps/*/models.rs.

fn discover_models(src_dir: &Path) -> Result<Vec<(String, String)>> {
    let apps_dir = src_dir.join("apps");
    if !apps_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<_> = fs::read_dir(&apps_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());

    let mut files = vec![];
    for entry in entries {
        let module = entry.file_name().to_string_lossy().to_string();
        let model_path = entry.path().join("models.rs");
        if model_path.exists() {
            files.push((format!("apps/{module}/models.rs"), module));
        }
    }

    Ok(files)
}

// ── ANSI colours ──────────────────────────────────────────────────────────────
const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRN: &str = "\x1b[32m";
const YLW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYN: &str = "\x1b[36m";
const BLU: &str = "\x1b[34m";
const MAG: &str = "\x1b[35m";

// ── Parsed types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ParsedField {
    name: String,
    sql_type: String,
    nullable: bool,
    is_pk: bool,
    is_unique: bool,
    is_index: bool,
    line: usize, // 1-based line in source file
}

#[derive(Debug, Clone)]
struct ParsedTable {
    schema: String,
    table: String,
    struct_name: String,
    source_file: String,
    #[allow(dead_code)]
    source_line: usize,
    fields: Vec<ParsedField>,
}

impl ParsedTable {
    fn full_name(&self) -> String {
        format!("{}.{}", self.schema, self.table)
    }
}

#[derive(Debug)]
enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
struct Issue {
    severity: Severity,
    file: String,
    line: usize,
    message: String,
    hint: Option<String>,
}

// ── Persisted schema state ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ColState {
    name: String,
    sql_type: String,
    nullable: bool,
    is_pk: bool,
    is_unique: bool,
    is_index: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TableState {
    columns: Vec<ColState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SchemaState {
    tables: HashMap<String, TableState>,
}

// ── Rust type → SQL type ──────────────────────────────────────────────────────

fn rust_to_sql(ty: &str) -> (&'static str, bool) {
    let (inner, nullable) = if ty.starts_with("Option<") && ty.ends_with('>') {
        (&ty[7..ty.len() - 1], true)
    } else {
        (ty, false)
    };
    let sql = match inner.trim() {
        "Uuid" => "UUID",
        "String" => "VARCHAR",
        "bool" => "BOOLEAN",
        "i16" => "SMALLINT",
        "i32" => "INTEGER",
        "i64" | "i128" => "BIGINT",
        "f32" | "f64" => "DOUBLE PRECISION",
        "DateTime<Utc>" => "TIMESTAMPTZ",
        "serde_json::Value" => "JSONB",
        "Vec<String>" => "TEXT[]",
        _ => "TEXT",
    };
    (sql, nullable)
}

fn is_known_rust_type(ty: &str) -> bool {
    matches!(
        ty.trim(),
        "Uuid"
            | "String"
            | "bool"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "usize"
            | "isize"
            | "f32"
            | "f64"
            | "char"
            | "DateTime<Utc>"
            | "serde_json::Value"
            | "Vec<String>"
    )
}

// ── Name helpers ──────────────────────────────────────────────────────────────

fn camel_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

fn pluralize(s: &str) -> String {
    if s.ends_with("ss")
        || s.ends_with('x')
        || s.ends_with('z')
        || s.ends_with("ch")
        || s.ends_with("sh")
    {
        format!("{s}es")
    } else if s.ends_with('s') {
        s.to_string()
    } else {
        format!("{s}s")
    }
}

fn struct_to_table(name: &str) -> String {
    pluralize(&camel_to_snake(name))
}

// ── FK registry + resolution ──────────────────────────────────────────────────

fn build_fk_registry(tables: &[ParsedTable]) -> HashMap<String, String> {
    tables
        .iter()
        .map(|t| (t.table.clone(), t.full_name()))
        .collect()
}

fn resolve_fk(field: &str, sql_type: &str, reg: &HashMap<String, String>) -> Option<String> {
    if field == "id" || !field.ends_with("_id") || sql_type != "UUID" {
        return None;
    }
    let base = &field[..field.len() - 3];
    reg.get(&pluralize(base)).cloned()
}

// ── Parser ────────────────────────────────────────────────────────────────────

fn parse_model_file(path: &Path, default_schema: &str) -> Result<(Vec<ParsedTable>, Vec<Issue>)> {
    let src =
        fs::read_to_string(path).with_context(|| format!("Cannot read {}", path.display()))?;

    let mut actual_schema = default_schema.to_string();
    for line in src.lines() {
        if line.contains("@schema ") {
            if let Some(idx) = line.find("@schema ") {
                actual_schema = line[idx + 8..].trim().to_string();
                break;
            }
        }
    }

    Ok(parse_structs(&src, &actual_schema, &path.to_string_lossy()))
}

fn parse_structs(src: &str, schema: &str, file: &str) -> (Vec<ParsedTable>, Vec<Issue>) {
    let lines: Vec<&str> = src.lines().collect();
    let mut tables = vec![];
    let mut issues = vec![];
    let mut seen_structs: HashMap<String, usize> = HashMap::new();
    let mut i = 0usize;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if !trimmed.starts_with("pub struct ") {
            i += 1;
            continue;
        }

        let after = &trimmed["pub struct ".len()..];
        let name_end = after
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());
        let sname = &after[..name_end];

        // Skip: empty, lowercase start, or generic
        if sname.is_empty()
            || !sname.chars().next().is_some_and(|c| c.is_uppercase())
            || after.contains('<')
        {
            i += 1;
            continue;
        }

        let struct_line = i + 1;

        if let Some(&prev) = seen_structs.get(sname) {
            issues.push(Issue {
                severity: Severity::Error,
                file: file.to_string(),
                line: struct_line,
                message: format!(
                    "Duplicate struct '{}' (also declared at line {})",
                    sname, prev
                ),
                hint: None,
            });
        }
        seen_structs.insert(sname.to_string(), struct_line);

        // Find opening brace
        let mut brace_i = i;
        while brace_i < lines.len() && !lines[brace_i].contains('{') {
            brace_i += 1;
        }
        if brace_i >= lines.len() {
            i += 1;
            continue;
        }

        // Scan body until depth == 0
        let mut depth = 0i32;
        let mut body: Vec<(usize, &str)> = vec![];
        let mut j = brace_i;
        let mut started = false;

        while j < lines.len() {
            let l = lines[j];
            for ch in l.chars() {
                match ch {
                    '{' => {
                        depth += 1;
                        started = true;
                    }
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if started && j > brace_i {
                body.push((j + 1, l));
            } else if started {
                // content after `{` on the opening line
                if let Some(pos) = l.find('{') {
                    let rest = l[pos + 1..].trim();
                    if !rest.is_empty() && rest != "}" {
                        body.push((j + 1, &l[pos + 1..]));
                    }
                }
            }
            if started && depth == 0 {
                break;
            }
            j += 1;
        }

        // Parse fields
        let mut fields: Vec<ParsedField> = vec![];
        let mut seen_fields: HashMap<String, usize> = HashMap::new();
        let mut pending_unique = false;
        let mut pending_index = false;

        for &(line_num, raw) in &body {
            let t = raw.trim();
            if t.starts_with("//") {
                if t.contains("@unique") {
                    pending_unique = true;
                }
                if t.contains("@index") {
                    pending_index = true;
                }
                continue;
            }

            let l = { t.find("//").map(|idx| &t[..idx]).unwrap_or(t).trim() };
            if !l.starts_with("pub ") {
                // reset flags if it's not an attribute
                if !l.is_empty() && !l.starts_with("#[") {
                    pending_unique = false;
                    pending_index = false;
                }
                continue;
            }
            let rest = l["pub ".len()..].trim_start();
            let Some(colon) = rest.find(':') else {
                continue;
            };

            let fname = rest[..colon].trim().to_string();
            let ftype = rest[colon + 1..].trim().trim_end_matches(',').to_string();

            if fname.is_empty() || ftype.is_empty() || fname.contains(' ') {
                continue;
            }

            if let Some(&prev) = seen_fields.get(&fname) {
                issues.push(Issue {
                    severity: Severity::Error,
                    file: file.to_string(),
                    line: line_num,
                    message: format!(
                        "Duplicate field '{}' in struct '{}' (first at line {})",
                        fname, sname, prev
                    ),
                    hint: None,
                });
                continue;
            }
            seen_fields.insert(fname.clone(), line_num);

            let (sql_type, nullable) = rust_to_sql(&ftype);

            // Warn on unrecognised Rust types
            let inner: &str = if ftype.starts_with("Option<") && ftype.ends_with('>') {
                &ftype[7..ftype.len() - 1]
            } else {
                &ftype
            };
            if sql_type == "TEXT" && !is_known_rust_type(inner.trim()) {
                issues.push(Issue {
                    severity: Severity::Warning,
                    file: file.to_string(),
                    line: line_num,
                    message: format!(
                        "Unknown type '{}' for field '{}' — mapped to TEXT",
                        inner, fname
                    ),
                    hint: Some("Add a custom entry to rust_to_sql() if TEXT is wrong".to_string()),
                });
            }

            let is_pk = fname == "id";
            let is_unique = pending_unique;
            let is_index = pending_index;
            pending_unique = false;
            pending_index = false;

            fields.push(ParsedField {
                name: fname,
                sql_type: sql_type.to_string(),
                nullable,
                is_pk,
                is_unique,
                is_index,
                line: line_num,
            });
        }

        // Every mapped struct must have a primary key
        if !fields.is_empty() && !fields.iter().any(|f| f.is_pk) {
            issues.push(Issue {
                severity: Severity::Error,
                file: file.to_string(),
                line: struct_line,
                message: format!("'{}' has no primary key field", sname),
                hint: Some("Add:  pub id: Uuid,".to_string()),
            });
        }

        if !fields.is_empty() {
            tables.push(ParsedTable {
                schema: schema.to_string(),
                table: struct_to_table(sname),
                struct_name: sname.to_string(),
                source_file: file.to_string(),
                source_line: struct_line,
                fields,
            });
        }

        i = j + 1;
    }

    (tables, issues)
}

// ── FK validation (cross-table) ───────────────────────────────────────────────

fn validate_fks(tables: &[ParsedTable], reg: &HashMap<String, String>) -> Vec<Issue> {
    let mut issues = vec![];
    for t in tables {
        for f in &t.fields {
            if f.name == "id" || !f.name.ends_with("_id") || f.sql_type != "UUID" {
                continue;
            }
            let base = &f.name[..f.name.len() - 3];
            let target = pluralize(base);
            if !reg.contains_key(&target) {
                issues.push(Issue {
                    severity: Severity::Warning,
                    file: t.source_file.clone(), line: f.line,
                    message: format!(
                        "'{}' in '{}' looks like a FK but table '{}' not found in models",
                        f.name, t.struct_name, target
                    ),
                    hint: Some(format!(
                        "Define a struct that maps to table '{}', or it may live in another service",
                        target
                    )),
                });
            }
        }
    }
    issues
}

// ── Topological sort (FK-safe creation order) ─────────────────────────────────

fn topo_sort(tables: Vec<ParsedTable>, reg: &HashMap<String, String>) -> Vec<ParsedTable> {
    let mut placed: HashMap<String, bool> = HashMap::new();
    let mut result = Vec::with_capacity(tables.len());
    let mut progress = true;

    while progress {
        progress = false;
        for t in &tables {
            if placed.contains_key(&t.full_name()) {
                continue;
            }
            let all_deps_placed = t
                .fields
                .iter()
                .filter_map(|f| resolve_fk(&f.name, &f.sql_type, reg))
                .filter(|d| d != &t.full_name())
                .all(|d| placed.contains_key(&d));
            if all_deps_placed {
                placed.insert(t.full_name(), true);
                result.push(t.clone());
                progress = true;
            }
        }
    }
    // Append any unresolved (circular FK)
    for t in &tables {
        if !placed.contains_key(&t.full_name()) {
            result.push(t.clone());
        }
    }
    result
}

// ── SQL generation ────────────────────────────────────────────────────────────

fn col_sql(f: &ParsedField, reg: &HashMap<String, String>) -> String {
    if f.is_pk {
        return format!("    {} UUID PRIMARY KEY DEFAULT gen_random_uuid()", f.name);
    }
    let mut parts = vec![f.name.clone(), f.sql_type.clone()];
    if !f.nullable {
        parts.push("NOT NULL".to_string());
    }
    if f.is_unique {
        parts.push("UNIQUE".to_string());
    }
    if matches!(f.name.as_str(), "created_at" | "updated_at") {
        parts.push("DEFAULT NOW()".to_string());
    }
    if let Some(ref_tbl) = resolve_fk(&f.name, &f.sql_type, reg) {
        parts.push(format!("REFERENCES {ref_tbl}(id)"));
    }
    format!("    {}", parts.join(" "))
}

fn create_table_sql(t: &ParsedTable, reg: &HashMap<String, String>) -> String {
    let cols = t
        .fields
        .iter()
        .map(|f| col_sql(f, reg))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("CREATE TABLE {}.{} (\n{cols}\n);", t.schema, t.table)
}

// ── Diff engine ───────────────────────────────────────────────────────────────

struct Diff {
    up: Vec<String>,
    down: Vec<String>,
    summary: Vec<String>, // human-readable change list
}

fn compute_diff(tables: &[ParsedTable], prev: &SchemaState, reg: &HashMap<String, String>) -> Diff {
    let mut up = vec![];
    let mut down = vec![];
    let mut summary = vec![];

    // Schema CREATE (idempotent)
    let schemas: BTreeSet<&str> = tables.iter().map(|t| t.schema.as_str()).collect();
    for s in &schemas {
        up.push(format!("CREATE SCHEMA IF NOT EXISTS {s};"));
    }

    for t in tables {
        let key = t.full_name();

        match prev.tables.get(&key) {
            // ── New table ──────────────────────────────────────────────────
            None => {
                summary.push(format!("  {GRN}{BLD}+ {key}{RST}  {DIM}(new table){RST}"));
                for f in &t.fields {
                    let nullable_str = if f.nullable { "NULL" } else { "NOT NULL" };
                    summary.push(format!(
                        "    {GRN}+{RST} {BLD}{:<22}{RST} {}  {DIM}{}{RST}",
                        f.name, f.sql_type, nullable_str
                    ));
                }
                up.push(create_table_sql(t, reg));
                for f in &t.fields {
                    if f.is_index {
                        up.push(format!(
                            "CREATE INDEX IF NOT EXISTS idx_{tbl}_{col} ON {key} ({col});",
                            tbl = t.table,
                            col = f.name,
                            key = key
                        ));
                        down.push(format!(
                            "DROP INDEX IF EXISTS {schema}.idx_{tbl}_{col};",
                            schema = t.schema,
                            tbl = t.table,
                            col = f.name
                        ));
                    }
                }
                down.push(format!(
                    "DROP TABLE IF EXISTS {}.{} CASCADE;",
                    t.schema, t.table
                ));
            }

            // ── Existing table: check columns ──────────────────────────────
            Some(prev_t) => {
                let prev_map: HashMap<&str, &ColState> = prev_t
                    .columns
                    .iter()
                    .map(|c| (c.name.as_str(), c))
                    .collect();
                let mut header_printed = false;

                let mut print_header = |summary: &mut Vec<String>| {
                    if !header_printed {
                        summary.push(format!("  {BLD}{key}:{RST}"));
                        header_printed = true;
                    }
                };

                for f in &t.fields {
                    match prev_map.get(f.name.as_str()) {
                        // New column
                        None => {
                            print_header(&mut summary);
                            let clause = col_sql(f, reg).trim().to_string();
                            summary.push(format!(
                                "    {GRN}+{RST} {BLD}{:<22}{RST} {}  {DIM}(new column){RST}",
                                f.name, f.sql_type
                            ));
                            up.push(format!("ALTER TABLE {key} ADD COLUMN {clause};"));
                            down.push(format!(
                                "ALTER TABLE {key} DROP COLUMN IF EXISTS {};",
                                f.name
                            ));
                        }
                        Some(pc) => {
                            if pc.sql_type != f.sql_type {
                                print_header(&mut summary);
                                summary.push(format!(
                                    "    {YLW}~{RST} {BLD}{:<22}{RST} {} → {}  {YLW}⚠ type change{RST}",
                                    f.name, pc.sql_type, f.sql_type
                                ));
                                up.push(format!(
                                    "ALTER TABLE {key} ALTER COLUMN {col} TYPE {ty} USING {col}::{ty};",
                                    col = f.name, ty = f.sql_type,
                                ));
                            }
                            if !pc.is_unique && f.is_unique {
                                print_header(&mut summary);
                                summary.push(format!(
                                    "    {YLW}~{RST} {BLD}{:<22}{RST}  {DIM}+unique{RST}",
                                    f.name
                                ));
                                up.push(format!(
                                    "ALTER TABLE {key} ADD UNIQUE ({col});",
                                    col = f.name
                                ));
                            }
                            if !pc.is_index && f.is_index {
                                print_header(&mut summary);
                                summary.push(format!(
                                    "    {YLW}~{RST} {BLD}{:<22}{RST}  {DIM}+index{RST}",
                                    f.name
                                ));
                                up.push(format!(
                                    "CREATE INDEX IF NOT EXISTS idx_{tbl}_{col} ON {key} ({col});",
                                    tbl = t.table,
                                    col = f.name,
                                    key = key
                                ));
                                down.push(format!(
                                    "DROP INDEX IF EXISTS {schema}.idx_{tbl}_{col};",
                                    schema = t.schema,
                                    tbl = t.table,
                                    col = f.name
                                ));
                            }
                        }
                    }
                }

                // Removed fields — warn but never auto-drop
                let cur_names: BTreeSet<&str> = t.fields.iter().map(|f| f.name.as_str()).collect();
                for pc in &prev_t.columns {
                    if !cur_names.contains(pc.name.as_str()) {
                        print_header(&mut summary);
                        summary.push(format!(
                            "    {RED}-{RST} {BLD}{:<22}{RST} {}  {YLW}⚠ removed — add DROP COLUMN manually{RST}",
                            pc.name, pc.sql_type
                        ));
                    }
                }
            }
        }
    }

    // Removed tables — warn, never auto-drop
    for pk in prev.tables.keys() {
        if !tables.iter().any(|t| &t.full_name() == pk) {
            summary.push(format!(
                "  {YLW}⚠ Table removed: {pk} — add DROP TABLE manually if intended{RST}"
            ));
        }
    }

    Diff { up, down, summary }
}

// ── State I/O ─────────────────────────────────────────────────────────────────

fn state_path() -> PathBuf {
    PathBuf::from(MANIFEST_DIR)
        .join("migrations")
        .join(".schema_state.json")
}

fn load_state() -> SchemaState {
    let p = state_path();
    if !p.exists() {
        return SchemaState::default();
    }
    serde_json::from_str(&fs::read_to_string(p).unwrap_or_default()).unwrap_or_default()
}

fn save_state(tables: &[ParsedTable]) -> Result<()> {
    let mut state = SchemaState::default();
    for t in tables {
        state.tables.insert(
            t.full_name(),
            TableState {
                columns: t
                    .fields
                    .iter()
                    .map(|f| ColState {
                        name: f.name.clone(),
                        sql_type: f.sql_type.clone(),
                        nullable: f.nullable,
                        is_pk: f.is_pk,
                        is_unique: f.is_unique,
                        is_index: f.is_index,
                    })
                    .collect(),
            },
        );
    }
    fs::write(state_path(), serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

// ── Migration writer ──────────────────────────────────────────────────────────

fn migrations_dir() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("migrations")
}

fn write_migration(diff: &Diff, label: &str) -> Result<PathBuf> {
    let ts = Utc::now().format("%Y%m%d%H%M%S");
    let dir = migrations_dir().join(format!("{ts}_{label}"));
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("up.sql"), diff.up.join("\n\n"))?;
    fs::write(
        dir.join("down.sql"),
        if diff.down.is_empty() {
            "-- No automatic down migration.\n-- Add DROP statements manually if needed."
                .to_string()
        } else {
            diff.down.join("\n\n")
        },
    )?;
    Ok(dir)
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let label = args.get(1).map(|s| s.as_str()).unwrap_or("auto");

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo makemigrations               ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    // ── 1. Scan model files ───────────────────────────────────────────────────
    println!("{CYN}Scanning model files...{RST}");
    let src_dir = PathBuf::from(MANIFEST_DIR).join("src");
    let mut all_tables: Vec<ParsedTable> = vec![];
    let mut all_issues: Vec<Issue> = vec![];

    let model_files = match discover_models(&src_dir) {
        Ok(files) => files,
        Err(e) => {
            println!("{RED}Failed to discover models: {e}{RST}");
            std::process::exit(1);
        }
    };

    for (rel, schema) in model_files {
        let path = src_dir.join(&rel);
        if !path.exists() {
            println!("  {DIM}[skip]{RST} {rel}");
            continue;
        }
        let (tables, issues) = parse_model_file(&path, &schema)?;
        println!(
            "  {GRN}✓{RST}  {BLD}{rel:<32}{RST}  {} struct(s)   {DIM}schema: {MAG}{schema}{RST}",
            tables.len()
        );
        all_tables.extend(tables);
        all_issues.extend(issues);
    }

    // ── 2. Build registry & validate FKs ─────────────────────────────────────
    let fk_reg = build_fk_registry(&all_tables);
    all_issues.extend(validate_fks(&all_tables, &fk_reg));

    // ── 3. Show validation results ────────────────────────────────────────────
    println!("\n{CYN}Validating models...{RST}");

    let errors: Vec<&Issue> = all_issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .collect();
    let warnings: Vec<&Issue> = all_issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Warning))
        .collect();

    for w in &warnings {
        let short_file = w.file.rsplit("src/").next().unwrap_or(&w.file);
        println!("  {YLW}⚠{RST}  {BLD}src/{short_file}:{}{RST}", w.line);
        println!("     {}", w.message);
        if let Some(ref h) = w.hint {
            println!("     {DIM}hint: {h}{RST}");
        }
    }

    for e in &errors {
        let short_file = e.file.rsplit("src/").next().unwrap_or(&e.file);
        println!("  {RED}✗{RST}  {BLD}{RED}src/{short_file}:{}{RST}", e.line);
        println!("     {}", e.message);
        if let Some(ref h) = e.hint {
            println!("     {BLU}fix:{RST} {h}");
        }
    }

    if !errors.is_empty() {
        println!(
            "\n  {RED}{BLD}{} error(s) — fix the above before generating migrations.{RST}\n",
            errors.len()
        );
        std::process::exit(1);
    }

    if warnings.is_empty() {
        println!("  {GRN}✓  No issues found{RST}");
    } else {
        println!(
            "  {YLW}⚠  {} warning(s) — migration will still be generated{RST}",
            warnings.len()
        );
    }

    // ── 4. Diff ───────────────────────────────────────────────────────────────
    let sorted = topo_sort(all_tables, &fk_reg);
    let prev = load_state();
    let is_first = prev.tables.is_empty();

    println!("\n{CYN}Detecting changes...{RST}");
    let diff = compute_diff(&sorted, &prev, &fk_reg);

    let has_real = diff
        .up
        .iter()
        .any(|s| !s.trim().starts_with("CREATE SCHEMA"));
    if !has_real {
        println!("  {GRN}✓  No changes detected — database is already up to date.{RST}\n");
        return Ok(());
    }

    for line in &diff.summary {
        println!("{line}");
    }

    // ── 5. Write migration files ──────────────────────────────────────────────
    let mig_dir = write_migration(&diff, label)?;
    save_state(&sorted)?;

    let stmt_count = diff
        .up
        .iter()
        .filter(|s| !s.trim().starts_with("CREATE SCHEMA"))
        .count();

    let dir_name = mig_dir.file_name().unwrap().to_string_lossy();
    println!("\n{GRN}{BLD}Migration generated:{RST} {dir_name}");
    println!("  {DIM}up.sql    — {stmt_count} SQL statement(s){RST}");
    println!("  {DIM}down.sql  — rollback{RST}");
    println!("  {DIM}.schema_state.json — updated  (commit alongside migration files){RST}");

    if is_first {
        println!("\n{YLW}First migration!{RST}  Apply with:  {BLD}cargo migrate{RST}\n");
    } else {
        println!("\nApply with:  {BLD}cargo migrate{RST}\n");
    }

    Ok(())
}
