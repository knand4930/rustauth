// src/bin/makemigrations.rs
//
// Django-style "makemigrations" — reads Rust model structs, validates them,
// diffs against last known state, and generates SQL migration files.
//
// Supported model directives:
//
//   // @schema blogs
//   // @table blog_posts
//   // @index columns=author_id,is_published
//   // @unique columns=user_id,role_id
//
//   // @index
//   // @unique
//   // @default NOW()
//   // @nullable
//   // @required
//   // @sql_type CITEXT
//   // @validate email
//   // @references user.users
//
//   cargo makemigrations              → label defaults to "auto"
//   cargo makemigrations add_bio      → custom label

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

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

// ── Discovery helpers ────────────────────────────────────────────────────────

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

fn migrations_dir() -> PathBuf {
    PathBuf::from(MANIFEST_DIR).join("migrations")
}

fn is_valid_migration_dir(path: &Path) -> bool {
    path.is_dir()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.chars().next())
            .is_some_and(|first| first.is_ascii_digit())
        && path.join("up.sql").exists()
}

fn has_existing_migrations() -> bool {
    fs::read_dir(migrations_dir())
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .any(|entry| is_valid_migration_dir(&entry.path()))
}

// ── Parsed types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct NamedColumns {
    #[serde(default)]
    name: String,
    #[serde(default)]
    columns: Vec<String>,
}

#[derive(Debug, Clone)]
struct ParsedField {
    name: String,
    sql_type: String,
    nullable: bool,
    is_pk: bool,
    default: Option<String>,
    validators: Vec<String>,
    reference: Option<String>,
    line: usize,
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
    indexes: Vec<NamedColumns>,
    unique_constraints: Vec<NamedColumns>,
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
    #[serde(default)]
    name: String,
    #[serde(default)]
    sql_type: String,
    #[serde(default)]
    nullable: bool,
    #[serde(default)]
    is_pk: bool,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    validators: Vec<String>,
    #[serde(default)]
    reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TableState {
    #[serde(default)]
    columns: Vec<ColState>,
    #[serde(default)]
    indexes: Vec<NamedColumns>,
    #[serde(default)]
    unique_constraints: Vec<NamedColumns>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SchemaState {
    #[serde(default)]
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
        "i8" | "u8" | "u16" => "SMALLINT",
        "i16" => "SMALLINT",
        "i32" | "u32" => "INTEGER",
        "i64" | "i128" | "u64" | "usize" | "isize" => "BIGINT",
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

fn columns_signature(columns: &[String]) -> String {
    columns.join(",")
}

fn default_index_name(table: &str, columns: &[String]) -> String {
    format!("idx_{}_{}", table, columns.join("_"))
}

fn default_unique_name(table: &str, columns: &[String]) -> String {
    format!("uq_{}_{}", table, columns.join("_"))
}

// ── Directive parsing ─────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct FieldDirectives {
    unique: bool,
    index: bool,
    default: Option<String>,
    nullable: Option<bool>,
    sql_type: Option<String>,
    validators: Vec<String>,
    reference: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingNamedColumns {
    name: Option<String>,
    columns: Vec<String>,
    line: usize,
}

#[derive(Debug, Default)]
struct StructDirectives {
    schema: Option<String>,
    table: Option<String>,
    indexes: Vec<PendingNamedColumns>,
    unique_constraints: Vec<PendingNamedColumns>,
}

fn comment_body(line: &str) -> Option<&str> {
    line.trim().strip_prefix("//").map(str::trim)
}

fn parse_directive_args(input: &str) -> (HashSet<String>, HashMap<String, String>) {
    let mut flags = HashSet::new();
    let mut values = HashMap::new();

    for token in input.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            values.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            flags.insert(token.trim().to_string());
        }
    }

    (flags, values)
}

fn parse_columns(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn push_named_columns_directive(
    kind: &str,
    raw: &str,
    file: &str,
    line: usize,
    items: &mut Vec<PendingNamedColumns>,
    issues: &mut Vec<Issue>,
) {
    let (flags, mut values) = parse_directive_args(raw.trim());
    let columns = if let Some(value) = values.remove("columns") {
        parse_columns(&value)
    } else if !raw.trim().is_empty() && !raw.contains('=') {
        parse_columns(raw)
    } else {
        vec![]
    };

    if columns.is_empty() {
        issues.push(Issue {
            severity: Severity::Error,
            file: file.to_string(),
            line,
            message: format!(
                "Invalid @{kind} directive — specify at least one column using columns=col_a,col_b"
            ),
            hint: Some(format!("Example: // @{kind} columns=user_id,role_id")),
        });
        return;
    }

    let mut name = values.remove("name");
    if name.is_none() && flags.contains("named") {
        issues.push(Issue {
            severity: Severity::Warning,
            file: file.to_string(),
            line,
            message: format!("@{kind} uses the 'named' flag without a name=... value"),
            hint: None,
        });
    }

    items.push(PendingNamedColumns {
        name: name.take(),
        columns,
        line,
    });
}

fn parse_struct_directives(
    lines: &[&str],
    struct_idx: usize,
    file: &str,
) -> (StructDirectives, Vec<Issue>) {
    let mut directives = StructDirectives::default();
    let mut issues = vec![];
    let mut idx = struct_idx;
    let mut comments = vec![];

    while idx > 0 {
        idx -= 1;
        let trimmed = lines[idx].trim();
        if trimmed.is_empty() {
            break;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("///") {
            continue;
        }
        if trimmed.starts_with("//") {
            comments.push((idx + 1, trimmed));
            continue;
        }
        break;
    }

    comments.reverse();
    for (line, raw) in comments {
        let Some(body) = comment_body(raw) else {
            continue;
        };
        if let Some(value) = body.strip_prefix("@schema ") {
            directives.schema = Some(value.trim().to_string());
        } else if let Some(value) = body.strip_prefix("@table ") {
            directives.table = Some(value.trim().to_string());
        } else if let Some(value) = body.strip_prefix("@index") {
            push_named_columns_directive(
                "index",
                value.trim(),
                file,
                line,
                &mut directives.indexes,
                &mut issues,
            );
        } else if let Some(value) = body.strip_prefix("@unique") {
            push_named_columns_directive(
                "unique",
                value.trim(),
                file,
                line,
                &mut directives.unique_constraints,
                &mut issues,
            );
        }
    }

    (directives, issues)
}

fn apply_field_directive(
    directives: &mut FieldDirectives,
    raw: &str,
    file: &str,
    line: usize,
    issues: &mut Vec<Issue>,
) {
    let Some(body) = comment_body(raw) else {
        return;
    };

    if body == "@unique" {
        directives.unique = true;
    } else if body == "@index" {
        directives.index = true;
    } else if body == "@nullable" {
        directives.nullable = Some(true);
    } else if body == "@required" {
        directives.nullable = Some(false);
    } else if let Some(value) = body.strip_prefix("@default ") {
        directives.default = Some(value.trim().to_string());
    } else if body == "@default" {
        issues.push(Issue {
            severity: Severity::Warning,
            file: file.to_string(),
            line,
            message: "@default directive is missing a value".to_string(),
            hint: Some("Example: // @default NOW()".to_string()),
        });
    } else if let Some(value) = body.strip_prefix("@sql_type ") {
        directives.sql_type = Some(value.trim().to_string());
    } else if let Some(value) = body.strip_prefix("@validate ") {
        directives.validators.push(value.trim().to_string());
    } else if let Some(value) = body.strip_prefix("@references ") {
        directives.reference = Some(value.trim().to_string());
    }
}

fn finalize_named_columns(
    table: &str,
    file: &str,
    field_names: &HashSet<String>,
    pending: Vec<PendingNamedColumns>,
    auto_name: fn(&str, &[String]) -> String,
    issues: &mut Vec<Issue>,
) -> Vec<NamedColumns> {
    let mut out = vec![];
    let mut seen = HashSet::new();

    for item in pending {
        let mut missing = vec![];
        for column in &item.columns {
            if !field_names.contains(column) {
                missing.push(column.clone());
            }
        }

        if !missing.is_empty() {
            issues.push(Issue {
                severity: Severity::Error,
                file: file.to_string(),
                line: item.line,
                message: format!(
                    "Directive references unknown column(s): {}",
                    missing.join(", ")
                ),
                hint: None,
            });
            continue;
        }

        let columns = item.columns;
        let key = columns_signature(&columns);
        if !seen.insert(key) {
            continue;
        }

        out.push(NamedColumns {
            name: item.name.unwrap_or_else(|| auto_name(table, &columns)),
            columns,
        });
    }

    out
}

fn remove_redundant_indexes(
    indexes: Vec<NamedColumns>,
    unique_constraints: &[NamedColumns],
) -> Vec<NamedColumns> {
    indexes
        .into_iter()
        .filter(|index| {
            !unique_constraints
                .iter()
                .any(|unique| unique.columns == index.columns)
        })
        .collect()
}

// ── Parser ────────────────────────────────────────────────────────────────────

fn parse_model_file(path: &Path, default_schema: &str) -> Result<(Vec<ParsedTable>, Vec<Issue>)> {
    let src =
        fs::read_to_string(path).with_context(|| format!("Cannot read {}", path.display()))?;

    let mut actual_schema = default_schema.to_string();
    for line in src.lines() {
        if let Some(body) = comment_body(line) {
            if let Some(value) = body.strip_prefix("@schema ") {
                actual_schema = value.trim().to_string();
                break;
            }
        }
    }

    Ok(parse_structs(&src, &actual_schema, &path.to_string_lossy()))
}

fn parse_structs(src: &str, default_schema: &str, file: &str) -> (Vec<ParsedTable>, Vec<Issue>) {
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
            .find(|ch: char| !ch.is_alphanumeric() && ch != '_')
            .unwrap_or(after.len());
        let sname = &after[..name_end];

        if sname.is_empty()
            || !sname.chars().next().is_some_and(|ch| ch.is_uppercase())
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

        let (struct_directives, struct_issues) = parse_struct_directives(&lines, i, file);
        issues.extend(struct_issues);

        let schema = struct_directives
            .schema
            .clone()
            .unwrap_or_else(|| default_schema.to_string());
        let table = struct_directives
            .table
            .clone()
            .unwrap_or_else(|| struct_to_table(sname));

        let mut brace_i = i;
        while brace_i < lines.len() && !lines[brace_i].contains('{') {
            brace_i += 1;
        }
        if brace_i >= lines.len() {
            i += 1;
            continue;
        }

        let mut depth = 0i32;
        let mut body: Vec<(usize, &str)> = vec![];
        let mut j = brace_i;
        let mut started = false;

        while j < lines.len() {
            let line = lines[j];
            for ch in line.chars() {
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
                body.push((j + 1, line));
            } else if started {
                if let Some(pos) = line.find('{') {
                    let rest = line[pos + 1..].trim();
                    if !rest.is_empty() && rest != "}" {
                        body.push((j + 1, &line[pos + 1..]));
                    }
                }
            }

            if started && depth == 0 {
                break;
            }

            j += 1;
        }

        let mut fields = vec![];
        let mut seen_fields: HashMap<String, usize> = HashMap::new();
        let mut field_names = HashSet::new();
        let mut pending_directives = FieldDirectives::default();
        let mut pending_indexes = vec![];
        let mut pending_unique_constraints = vec![];

        for &(line_num, raw) in &body {
            let trimmed = raw.trim();

            if trimmed.starts_with("//") {
                apply_field_directive(
                    &mut pending_directives,
                    trimmed,
                    file,
                    line_num,
                    &mut issues,
                );
                continue;
            }

            let line = trimmed
                .find("//")
                .map(|idx| &trimmed[..idx])
                .unwrap_or(trimmed)
                .trim();

            if !line.starts_with("pub ") {
                if !line.is_empty() && !line.starts_with("#[") {
                    pending_directives = FieldDirectives::default();
                }
                continue;
            }

            let rest = line["pub ".len()..].trim_start();
            let Some(colon) = rest.find(':') else {
                pending_directives = FieldDirectives::default();
                continue;
            };

            let name = rest[..colon].trim().to_string();
            let rust_type = rest[colon + 1..].trim().trim_end_matches(',').to_string();
            if name.is_empty() || rust_type.is_empty() || name.contains(' ') {
                pending_directives = FieldDirectives::default();
                continue;
            }

            if let Some(&prev) = seen_fields.get(&name) {
                issues.push(Issue {
                    severity: Severity::Error,
                    file: file.to_string(),
                    line: line_num,
                    message: format!(
                        "Duplicate field '{}' in struct '{}' (first at line {})",
                        name, sname, prev
                    ),
                    hint: None,
                });
                pending_directives = FieldDirectives::default();
                continue;
            }
            seen_fields.insert(name.clone(), line_num);
            field_names.insert(name.clone());

            let (inferred_sql_type, inferred_nullable) = rust_to_sql(&rust_type);
            let inner_type: &str = if rust_type.starts_with("Option<") && rust_type.ends_with('>') {
                &rust_type[7..rust_type.len() - 1]
            } else {
                &rust_type
            };

            if inferred_sql_type == "TEXT"
                && pending_directives.sql_type.is_none()
                && !is_known_rust_type(inner_type.trim())
            {
                issues.push(Issue {
                    severity: Severity::Warning,
                    file: file.to_string(),
                    line: line_num,
                    message: format!(
                        "Unknown type '{}' for field '{}' — mapped to TEXT",
                        inner_type, name
                    ),
                    hint: Some("Add // @sql_type ... if TEXT is wrong".to_string()),
                });
            }

            let is_pk = name == "id";
            let sql_type = pending_directives
                .sql_type
                .clone()
                .unwrap_or_else(|| inferred_sql_type.to_string());
            let nullable = pending_directives.nullable.unwrap_or(inferred_nullable);

            if pending_directives.unique {
                pending_unique_constraints.push(PendingNamedColumns {
                    name: None,
                    columns: vec![name.clone()],
                    line: line_num,
                });
            }
            if pending_directives.index {
                pending_indexes.push(PendingNamedColumns {
                    name: None,
                    columns: vec![name.clone()],
                    line: line_num,
                });
            }

            fields.push(ParsedField {
                name,
                sql_type,
                nullable,
                is_pk,
                default: pending_directives.default.clone(),
                validators: pending_directives.validators.clone(),
                reference: pending_directives.reference.clone(),
                line: line_num,
            });

            pending_directives = FieldDirectives::default();
        }

        if !fields.is_empty() && !fields.iter().any(|field| field.is_pk) {
            issues.push(Issue {
                severity: Severity::Error,
                file: file.to_string(),
                line: struct_line,
                message: format!("'{}' has no primary key field", sname),
                hint: Some("Add:  pub id: Uuid,".to_string()),
            });
        }

        if !fields.is_empty() {
            let unique_constraints = finalize_named_columns(
                &table,
                file,
                &field_names,
                pending_unique_constraints
                    .into_iter()
                    .chain(struct_directives.unique_constraints.into_iter())
                    .collect(),
                default_unique_name,
                &mut issues,
            );

            let indexes = remove_redundant_indexes(
                finalize_named_columns(
                    &table,
                    file,
                    &field_names,
                    pending_indexes
                        .into_iter()
                        .chain(struct_directives.indexes.into_iter())
                        .collect(),
                    default_index_name,
                    &mut issues,
                ),
                &unique_constraints,
            );

            tables.push(ParsedTable {
                schema,
                table,
                struct_name: sname.to_string(),
                source_file: file.to_string(),
                source_line: struct_line,
                fields,
                indexes,
                unique_constraints,
            });
        }

        i = j + 1;
    }

    (tables, issues)
}

// ── FK registry + resolution ──────────────────────────────────────────────────

fn build_fk_registry(tables: &[ParsedTable]) -> HashMap<String, String> {
    let mut registry = HashMap::new();

    for table in tables {
        let full_name = table.full_name();
        registry.insert(table.table.clone(), full_name.clone());
        registry.insert(camel_to_snake(&table.struct_name), full_name.clone());
        registry.insert(pluralize(&camel_to_snake(&table.struct_name)), full_name);
    }

    registry
}

fn resolve_reference(
    reference: &str,
    current_table: &ParsedTable,
    reg: &HashMap<String, String>,
) -> Option<String> {
    let normalized = reference.trim().trim_end_matches("(id)").trim();
    if normalized.eq_ignore_ascii_case("self") {
        return Some(current_table.full_name());
    }
    if normalized.contains('.') {
        return Some(normalized.to_string());
    }
    reg.get(normalized)
        .cloned()
        .or_else(|| reg.get(&pluralize(normalized)).cloned())
}

fn resolve_fk(
    table: &ParsedTable,
    field: &ParsedField,
    reg: &HashMap<String, String>,
) -> Option<String> {
    if field.is_pk || field.sql_type != "UUID" {
        return None;
    }

    if let Some(reference) = &field.reference {
        return resolve_reference(reference, table, reg);
    }

    if !field.name.ends_with("_id") {
        return None;
    }

    let base = &field.name[..field.name.len() - 3];
    reg.get(&pluralize(base)).cloned()
}

// ── FK validation (cross-table) ───────────────────────────────────────────────

fn validate_fks(tables: &[ParsedTable], reg: &HashMap<String, String>) -> Vec<Issue> {
    let mut issues = vec![];

    for table in tables {
        for field in &table.fields {
            if field.is_pk || field.sql_type != "UUID" {
                continue;
            }

            if let Some(reference) = &field.reference {
                if resolve_reference(reference, table, reg).is_none() {
                    issues.push(Issue {
                        severity: Severity::Warning,
                        file: table.source_file.clone(),
                        line: field.line,
                        message: format!(
                            "Explicit reference '{}' for '{}' could not be resolved locally",
                            reference, field.name
                        ),
                        hint: Some(
                            "Use a fully-qualified schema.table name for external services"
                                .to_string(),
                        ),
                    });
                }
                continue;
            }

            if !field.name.ends_with("_id") {
                continue;
            }

            let base = &field.name[..field.name.len() - 3];
            let target = pluralize(base);
            if !reg.contains_key(&target) {
                issues.push(Issue {
                    severity: Severity::Warning,
                    file: table.source_file.clone(),
                    line: field.line,
                    message: format!(
                        "'{}' in '{}' looks like a FK but table '{}' not found in models",
                        field.name, table.struct_name, target
                    ),
                    hint: Some(format!(
                        "Add // @references schema.table if this points outside the default naming pattern"
                    )),
                });
            }
        }
    }

    issues
}

// ── Topological sort (FK-safe creation order) ─────────────────────────────────

fn topo_sort(tables: Vec<ParsedTable>, reg: &HashMap<String, String>) -> Vec<ParsedTable> {
    let mut placed = HashSet::new();
    let mut result = Vec::with_capacity(tables.len());
    let mut progress = true;

    while progress {
        progress = false;
        for table in &tables {
            if placed.contains(&table.full_name()) {
                continue;
            }

            let all_deps_placed = table
                .fields
                .iter()
                .filter_map(|field| resolve_fk(table, field, reg))
                .filter(|dependency| dependency != &table.full_name())
                .all(|dependency| placed.contains(&dependency));

            if all_deps_placed {
                placed.insert(table.full_name());
                result.push(table.clone());
                progress = true;
            }
        }
    }

    for table in &tables {
        if !placed.contains(&table.full_name()) {
            result.push(table.clone());
        }
    }

    result
}

// ── SQL generation ────────────────────────────────────────────────────────────

fn effective_default(field: &ParsedField) -> Option<String> {
    if let Some(default) = &field.default {
        Some(default.clone())
    } else if matches!(field.name.as_str(), "created_at" | "updated_at") {
        Some("NOW()".to_string())
    } else if field.is_pk {
        Some("gen_random_uuid()".to_string())
    } else {
        None
    }
}

fn col_sql(table: &ParsedTable, field: &ParsedField, reg: &HashMap<String, String>) -> String {
    let mut parts = vec![field.name.clone(), field.sql_type.clone()];

    if field.is_pk {
        parts.push("PRIMARY KEY".to_string());
    }
    if !field.nullable {
        parts.push("NOT NULL".to_string());
    }
    if let Some(default) = effective_default(field) {
        parts.push(format!("DEFAULT {default}"));
    }
    if let Some(reference_table) = resolve_fk(table, field, reg) {
        parts.push(format!("REFERENCES {reference_table}(id)"));
    }

    format!("    {}", parts.join(" "))
}

fn create_table_sql(
    table: &ParsedTable,
    reg: &HashMap<String, String>,
    if_not_exists: bool,
) -> String {
    let columns = table
        .fields
        .iter()
        .map(|field| col_sql(table, field, reg))
        .collect::<Vec<_>>()
        .join(",\n");

    let if_not_exists = if if_not_exists { " IF NOT EXISTS" } else { "" };
    format!(
        "CREATE TABLE{if_not_exists} {}.{} (\n{columns}\n);",
        table.schema, table.table
    )
}

fn add_unique_constraint_sql(table_key: &str, constraint: &NamedColumns) -> String {
    format!(
        "DO $$ BEGIN ALTER TABLE {table_key} ADD CONSTRAINT {} UNIQUE ({}); EXCEPTION WHEN duplicate_object THEN NULL; END $$;",
        constraint.name,
        constraint.columns.join(", ")
    )
}

fn drop_unique_constraint_sql(table_key: &str, constraint: &NamedColumns) -> String {
    format!(
        "ALTER TABLE {table_key} DROP CONSTRAINT IF EXISTS {};",
        constraint.name
    )
}

fn add_index_sql(table_key: &str, index: &NamedColumns) -> String {
    format!(
        "CREATE INDEX IF NOT EXISTS {} ON {table_key} ({});",
        index.name,
        index.columns.join(", ")
    )
}

fn drop_index_sql(schema: &str, index: &NamedColumns) -> String {
    format!("DROP INDEX IF EXISTS {schema}.{};", index.name)
}

// ── Diff engine ───────────────────────────────────────────────────────────────

struct Diff {
    up: Vec<String>,
    down: Vec<String>,
    summary: Vec<String>,
}

fn has_named_columns(items: &[NamedColumns], columns: &[String]) -> bool {
    items.iter().any(|item| item.columns == columns)
}

fn compute_diff(tables: &[ParsedTable], prev: &SchemaState, reg: &HashMap<String, String>) -> Diff {
    let mut up = vec!["CREATE EXTENSION IF NOT EXISTS pgcrypto;".to_string()];
    let mut down = vec![];
    let mut summary = vec![];
    let is_first = prev.tables.is_empty();

    let schemas: BTreeSet<&str> = tables.iter().map(|table| table.schema.as_str()).collect();
    for schema in &schemas {
        up.push(format!("CREATE SCHEMA IF NOT EXISTS {schema};"));
    }

    for table in tables {
        let key = table.full_name();

        match prev.tables.get(&key) {
            None => {
                summary.push(format!("  {GRN}{BLD}+ {key}{RST}  {DIM}(new table){RST}"));
                up.push(create_table_sql(table, reg, is_first));

                for field in &table.fields {
                    let nullable = if field.nullable { "NULL" } else { "NOT NULL" };
                    let mut extras = vec![];
                    if let Some(default) = effective_default(field) {
                        extras.push(format!("default {default}"));
                    }
                    if !field.validators.is_empty() {
                        extras.push(format!("validate {}", field.validators.join(", ")));
                    }
                    let extras = if extras.is_empty() {
                        String::new()
                    } else {
                        format!("  {DIM}[{}]{RST}", extras.join(" | "))
                    };
                    summary.push(format!(
                        "    {GRN}+{RST} {BLD}{:<22}{RST} {}  {DIM}{}{RST}{}",
                        field.name, field.sql_type, nullable, extras
                    ));
                }

                for constraint in &table.unique_constraints {
                    summary.push(format!(
                        "    {GRN}+{RST} {BLD}{:<22}{RST} UNIQUE ({})",
                        constraint.name,
                        constraint.columns.join(", ")
                    ));
                    up.push(add_unique_constraint_sql(&key, constraint));
                    down.push(drop_unique_constraint_sql(&key, constraint));
                }

                for index in &table.indexes {
                    summary.push(format!(
                        "    {GRN}+{RST} {BLD}{:<22}{RST} INDEX ({})",
                        index.name,
                        index.columns.join(", ")
                    ));
                    up.push(add_index_sql(&key, index));
                    down.push(drop_index_sql(&table.schema, index));
                }

                down.push(format!(
                    "DROP TABLE IF EXISTS {}.{} CASCADE;",
                    table.schema, table.table
                ));
            }
            Some(prev_table) => {
                let prev_columns: HashMap<&str, &ColState> = prev_table
                    .columns
                    .iter()
                    .map(|column| (column.name.as_str(), column))
                    .collect();
                let mut header_printed = false;
                let mut print_header = |summary: &mut Vec<String>| {
                    if !header_printed {
                        summary.push(format!("  {BLD}{key}:{RST}"));
                        header_printed = true;
                    }
                };

                for field in &table.fields {
                    match prev_columns.get(field.name.as_str()) {
                        None => {
                            print_header(&mut summary);
                            summary.push(format!(
                                "    {GRN}+{RST} {BLD}{:<22}{RST} {}  {DIM}(new column){RST}",
                                field.name, field.sql_type
                            ));
                            up.push(format!(
                                "ALTER TABLE {key} ADD COLUMN {};",
                                col_sql(table, field, reg).trim()
                            ));
                            down.push(format!(
                                "ALTER TABLE {key} DROP COLUMN IF EXISTS {};",
                                field.name
                            ));
                        }
                        Some(previous) => {
                            if previous.sql_type != field.sql_type {
                                print_header(&mut summary);
                                summary.push(format!(
                                    "    {YLW}~{RST} {BLD}{:<22}{RST} {} → {}  {YLW}type change{RST}",
                                    field.name, previous.sql_type, field.sql_type
                                ));
                                up.push(format!(
                                    "ALTER TABLE {key} ALTER COLUMN {column} TYPE {sql_type} USING {column}::{sql_type};",
                                    column = field.name,
                                    sql_type = field.sql_type
                                ));
                            }

                            if previous.nullable != field.nullable {
                                print_header(&mut summary);
                                summary.push(format!(
                                    "    {YLW}~{RST} {BLD}{:<22}{RST} {} → {}",
                                    field.name,
                                    if previous.nullable {
                                        "NULL"
                                    } else {
                                        "NOT NULL"
                                    },
                                    if field.nullable { "NULL" } else { "NOT NULL" }
                                ));
                                up.push(format!(
                                    "ALTER TABLE {key} ALTER COLUMN {column} {};",
                                    if field.nullable {
                                        format!("DROP NOT NULL",)
                                    } else {
                                        format!("SET NOT NULL",)
                                    },
                                    column = field.name
                                ));
                            }

                            let previous_default = previous.default.clone();
                            let current_default = effective_default(field);
                            if previous_default != current_default {
                                print_header(&mut summary);
                                summary.push(format!(
                                    "    {YLW}~{RST} {BLD}{:<22}{RST} default {:?} → {:?}",
                                    field.name, previous_default, current_default
                                ));
                                if let Some(default) = current_default {
                                    up.push(format!(
                                        "ALTER TABLE {key} ALTER COLUMN {column} SET DEFAULT {default};",
                                        column = field.name
                                    ));
                                } else {
                                    up.push(format!(
                                        "ALTER TABLE {key} ALTER COLUMN {column} DROP DEFAULT;",
                                        column = field.name
                                    ));
                                }
                            }

                            if previous.reference != field.reference {
                                print_header(&mut summary);
                                summary.push(format!(
                                    "    {YLW}~{RST} {BLD}{:<22}{RST}  {DIM}reference metadata changed — review FK manually if needed{RST}",
                                    field.name
                                ));
                            }
                        }
                    }
                }

                let current_column_names: BTreeSet<&str> = table
                    .fields
                    .iter()
                    .map(|field| field.name.as_str())
                    .collect();
                for previous in &prev_table.columns {
                    if !current_column_names.contains(previous.name.as_str()) {
                        print_header(&mut summary);
                        summary.push(format!(
                            "    {RED}-{RST} {BLD}{:<22}{RST} {}  {YLW}removed — add DROP COLUMN manually if intended{RST}",
                            previous.name, previous.sql_type
                        ));
                    }
                }

                for constraint in &table.unique_constraints {
                    if !has_named_columns(&prev_table.unique_constraints, &constraint.columns) {
                        print_header(&mut summary);
                        summary.push(format!(
                            "    {YLW}~{RST} {BLD}{:<22}{RST}  {DIM}+unique ({}){RST}",
                            constraint.name,
                            constraint.columns.join(", ")
                        ));
                        up.push(add_unique_constraint_sql(&key, constraint));
                        down.push(drop_unique_constraint_sql(&key, constraint));
                    }
                }

                for previous in &prev_table.unique_constraints {
                    if !has_named_columns(&table.unique_constraints, &previous.columns) {
                        print_header(&mut summary);
                        summary.push(format!(
                            "    {RED}-{RST} {BLD}{:<22}{RST}  {YLW}removed unique ({}) — drop manually if intended{RST}",
                            previous.name,
                            previous.columns.join(", ")
                        ));
                    }
                }

                for index in &table.indexes {
                    if !has_named_columns(&prev_table.indexes, &index.columns) {
                        print_header(&mut summary);
                        summary.push(format!(
                            "    {YLW}~{RST} {BLD}{:<22}{RST}  {DIM}+index ({}){RST}",
                            index.name,
                            index.columns.join(", ")
                        ));
                        up.push(add_index_sql(&key, index));
                        down.push(drop_index_sql(&table.schema, index));
                    }
                }

                for previous in &prev_table.indexes {
                    if !has_named_columns(&table.indexes, &previous.columns) {
                        print_header(&mut summary);
                        summary.push(format!(
                            "    {RED}-{RST} {BLD}{:<22}{RST}  {YLW}removed index ({}) — drop manually if intended{RST}",
                            previous.name,
                            previous.columns.join(", ")
                        ));
                    }
                }
            }
        }
    }

    for previous_key in prev.tables.keys() {
        if !tables
            .iter()
            .any(|table| &table.full_name() == previous_key)
        {
            summary.push(format!(
                "  {YLW}⚠ Table removed: {previous_key} — add DROP TABLE manually if intended{RST}"
            ));
        }
    }

    Diff { up, down, summary }
}

// ── State I/O ─────────────────────────────────────────────────────────────────

fn state_path() -> PathBuf {
    migrations_dir().join(".schema_state.json")
}

fn state_from_tables(tables: &[ParsedTable]) -> SchemaState {
    let mut state = SchemaState::default();

    for table in tables {
        state.tables.insert(
            table.full_name(),
            TableState {
                columns: table
                    .fields
                    .iter()
                    .map(|field| ColState {
                        name: field.name.clone(),
                        sql_type: field.sql_type.clone(),
                        nullable: field.nullable,
                        is_pk: field.is_pk,
                        default: effective_default(field),
                        validators: field.validators.clone(),
                        reference: field.reference.clone(),
                    })
                    .collect(),
                indexes: table.indexes.clone(),
                unique_constraints: table.unique_constraints.clone(),
            },
        );
    }

    state
}

fn save_state(tables: &[ParsedTable]) -> Result<()> {
    fs::write(
        state_path(),
        serde_json::to_string_pretty(&state_from_tables(tables))?,
    )?;
    Ok(())
}

fn load_state(current_tables: &[ParsedTable]) -> Result<(SchemaState, bool)> {
    let path = state_path();
    if path.exists() {
        let raw = fs::read_to_string(path).unwrap_or_default();
        let state = serde_json::from_str(&raw).unwrap_or_default();
        return Ok((state, false));
    }

    if has_existing_migrations() {
        let state = state_from_tables(current_tables);
        fs::write(&path, serde_json::to_string_pretty(&state)?)?;
        return Ok((state, true));
    }

    Ok((SchemaState::default(), false))
}

// ── Migration writer ──────────────────────────────────────────────────────────

fn write_migration(diff: &Diff, label: &str) -> Result<PathBuf> {
    let ts = Utc::now().format("%Y%m%d%H%M%S");
    let dir = migrations_dir().join(format!("{ts}_{label}"));
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("up.sql"), diff.up.join("\n\n"))?;
    fs::write(
        dir.join("down.sql"),
        if diff.down.is_empty() {
            "-- No automatic down migration.\n-- Add rollback statements manually if needed."
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
    let label = args.get(1).map(|value| value.as_str()).unwrap_or("auto");

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo makemigrations               ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    println!("{CYN}Scanning model files...{RST}");
    let src_dir = PathBuf::from(MANIFEST_DIR).join("src");
    let mut all_tables = vec![];
    let mut all_issues = vec![];

    let model_files = match discover_models(&src_dir) {
        Ok(files) => files,
        Err(error) => {
            println!("{RED}Failed to discover models: {error}{RST}");
            std::process::exit(1);
        }
    };

    for (relative_path, schema) in model_files {
        let path = src_dir.join(&relative_path);
        if !path.exists() {
            println!("  {DIM}[skip]{RST} {relative_path}");
            continue;
        }

        let (tables, issues) = parse_model_file(&path, &schema)?;
        println!(
            "  {GRN}✓{RST}  {BLD}{relative_path:<32}{RST}  {} struct(s)   {DIM}schema: {MAG}{schema}{RST}",
            tables.len()
        );
        all_tables.extend(tables);
        all_issues.extend(issues);
    }

    let fk_registry = build_fk_registry(&all_tables);
    all_issues.extend(validate_fks(&all_tables, &fk_registry));

    println!("\n{CYN}Validating models...{RST}");
    let errors: Vec<&Issue> = all_issues
        .iter()
        .filter(|issue| matches!(issue.severity, Severity::Error))
        .collect();
    let warnings: Vec<&Issue> = all_issues
        .iter()
        .filter(|issue| matches!(issue.severity, Severity::Warning))
        .collect();

    for warning in &warnings {
        let short_file = warning.file.rsplit("src/").next().unwrap_or(&warning.file);
        println!("  {YLW}⚠{RST}  {BLD}src/{short_file}:{}{RST}", warning.line);
        println!("     {}", warning.message);
        if let Some(hint) = &warning.hint {
            println!("     {DIM}hint: {hint}{RST}");
        }
    }

    for error in &errors {
        let short_file = error.file.rsplit("src/").next().unwrap_or(&error.file);
        println!(
            "  {RED}✗{RST}  {BLD}{RED}src/{short_file}:{}{RST}",
            error.line
        );
        println!("     {}", error.message);
        if let Some(hint) = &error.hint {
            println!("     {BLU}fix:{RST} {hint}");
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

    let sorted_tables = topo_sort(all_tables, &fk_registry);
    let (previous_state, bootstrapped_state) = load_state(&sorted_tables)?;
    if bootstrapped_state {
        println!("\n{CYN}Bootstrapping schema state...{RST}");
        println!(
            "  {GRN}✓{RST}  Created {BLD}migrations/.schema_state.json{RST} from the current model definitions."
        );
        println!(
            "  {DIM}Existing migration folders were detected, so no new migration was generated on this first run.{RST}"
        );
        println!("\nRe-run {BLD}cargo makemigrations{RST} after your next model change.\n");
        return Ok(());
    }

    println!("\n{CYN}Detecting changes...{RST}");
    let diff = compute_diff(&sorted_tables, &previous_state, &fk_registry);

    let has_real_changes = diff.up.iter().any(|statement| {
        let trimmed = statement.trim();
        !trimmed.starts_with("CREATE SCHEMA")
            && !trimmed.starts_with("CREATE EXTENSION IF NOT EXISTS pgcrypto")
    });

    if !has_real_changes {
        println!("  {GRN}✓  No changes detected — database is already up to date.{RST}\n");
        return Ok(());
    }

    for line in &diff.summary {
        println!("{line}");
    }

    let is_first = previous_state.tables.is_empty();
    let migration_dir = write_migration(&diff, label)?;
    save_state(&sorted_tables)?;

    let statement_count = diff
        .up
        .iter()
        .filter(|statement| {
            let trimmed = statement.trim();
            !trimmed.starts_with("CREATE SCHEMA")
                && !trimmed.starts_with("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        })
        .count();

    let dir_name = migration_dir.file_name().unwrap().to_string_lossy();
    println!("\n{GRN}{BLD}Migration generated:{RST} {dir_name}");
    println!("  {DIM}up.sql    — {statement_count} SQL statement(s){RST}");
    println!("  {DIM}down.sql  — rollback{RST}");
    println!("  {DIM}.schema_state.json — updated (commit alongside migration files){RST}");

    if is_first {
        println!("\n{YLW}First migration!{RST}  Apply with:  {BLD}cargo migrate{RST}\n");
    } else {
        println!("\nApply with:  {BLD}cargo migrate{RST}\n");
    }

    Ok(())
}
