// src/bin/squashmigrations.rs
//
// Squash a contiguous range of migrations into a single new migration.
//
// Usage:
//   cargo squashmigrations <start> <end>       squash from <start> to <end> (inclusive)
//   cargo squashmigrations <start> <end> --delete-old   also delete the original files
//
// The squashed migration is placed in migrations/<timestamp>_squashed_<start_label>_to_<end_label>/
// The original migration dirs are kept (renamed with .squashed suffix) unless --delete-old is passed.
//
// After squashing:
//   1. Mark the original migrations as applied in the DB (they remain in history).
//   2. Apply the squashed migration via `cargo migrate`.

use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GRN: &str = "\x1b[32m";
const YLW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYN: &str = "\x1b[36m";

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
        .filter(|e| {
            let path = e.path();
            path.is_dir()
                && path.join("up.sql").exists()
                && e.file_name()
                    .to_str()
                    .and_then(|n| n.chars().next())
                    .is_some_and(|c| c.is_ascii_digit())
        })
        .collect();

    entries.sort_by_key(|e| migration_sort_key(&e.file_name().to_string_lossy()));
    Ok(entries
        .into_iter()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect())
}

fn find_migration<'a>(names: &'a [String], target: &str) -> Option<(usize, &'a str)> {
    if let Some(index) = names.iter().position(|n| n == target) {
        return Some((index, names[index].as_str()));
    }
    let matches: Vec<_> = names
        .iter()
        .enumerate()
        .filter(|(_, n)| n.ends_with(target) || n.contains(target))
        .collect();
    match matches.len() {
        1 => Some((matches[0].0, matches[0].1.as_str())),
        0 => None,
        _ => {
            eprintln!("{YLW}Ambiguous name '{target}' matches:{RST}");
            for (_, n) in &matches {
                eprintln!("  {n}");
            }
            None
        }
    }
}

fn short_label(name: &str) -> String {
    // Take the label part (after the timestamp prefix)
    let parts: Vec<&str> = name.splitn(2, '_').collect();
    if parts.len() == 2 {
        parts[1].to_string()
    } else {
        name.to_string()
    }
}

fn read_sql(dir: &Path, filename: &str) -> String {
    let path = dir.join(filename);
    if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    }
}

fn is_noop_sql(sql: &str) -> bool {
    sql.lines()
        .all(|line| line.trim().is_empty() || line.trim().starts_with("--"))
}

fn print_usage() {
    println!("Usage:");
    println!("  {BLD}cargo squashmigrations <start> <end>{RST}");
    println!("      Squash migrations from <start> to <end> (inclusive) into one file.");
    println!("      Partial names are matched (e.g., 'add_users' or the timestamp prefix).");
    println!();
    println!("  {BLD}cargo squashmigrations <start> <end> --delete-old{RST}");
    println!("      Same as above, but also delete the original migration directories.");
    println!();
    println!("After squashing:");
    println!("  1. The squashed file is placed at migrations/<ts>_squashed_<label>/");
    println!("  2. Original directories are kept unless --delete-old is used.");
    println!("  3. Update the DB history as needed (original migrations remain applied).");
    println!();
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty()
        || args.iter().any(|a| matches!(a.as_str(), "-h" | "--help"))
    {
        print_usage();
        if args.is_empty() {
            std::process::exit(1);
        }
        return Ok(());
    }

    let mut positional: Vec<String> = vec![];
    let mut delete_old = false;

    for arg in &args {
        match arg.as_str() {
            "--delete-old" => delete_old = true,
            flag if flag.starts_with("--") => {
                eprintln!("{RED}Unknown flag: {flag}{RST}");
                print_usage();
                std::process::exit(1);
            }
            val => positional.push(val.to_string()),
        }
    }

    if positional.len() != 2 {
        eprintln!("{RED}Expected exactly two positional arguments: <start> <end>{RST}\n");
        print_usage();
        std::process::exit(1);
    }

    let start_arg = &positional[0];
    let end_arg = &positional[1];

    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo squashmigrations             ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    let names = migration_names()?;

    let (start_idx, start_name) = find_migration(&names, start_arg).with_context(|| {
        format!("Start migration '{start_arg}' not found. Run 'cargo showmigrations' to list available migrations.")
    })?;

    let (end_idx, end_name) = find_migration(&names, end_arg).with_context(|| {
        format!("End migration '{end_arg}' not found. Run 'cargo showmigrations' to list available migrations.")
    })?;

    if start_idx > end_idx {
        anyhow::bail!(
            "Start migration '{start_name}' comes after end migration '{end_name}'. Reverse the order."
        );
    }

    let range = &names[start_idx..=end_idx];
    println!("{CYN}Squashing {} migration(s):{RST}\n", range.len());
    for name in range {
        println!("  {DIM}→{RST}  {name}");
    }
    println!();

    // Concatenate SQL
    let mdir = migrations_dir();
    let mut up_parts: Vec<String> = vec![];
    let mut down_parts: Vec<String> = vec![];
    let mut skipped_count = 0usize;

    for name in range {
        let dir = mdir.join(name);
        let up = read_sql(&dir, "up.sql");
        let down = read_sql(&dir, "down.sql");

        if !is_noop_sql(&up) {
            up_parts.push(format!("-- squashed from: {name}"));
            up_parts.push(up.trim().to_string());
        } else {
            skipped_count += 1;
        }

        if !is_noop_sql(&down) {
            down_parts.push(format!("-- squashed from: {name}"));
            down_parts.push(down.trim().to_string());
        }
    }

    if up_parts.is_empty() {
        println!("{YLW}All migrations in range have empty SQL. Nothing to squash.{RST}\n");
        return Ok(());
    }

    // Write squashed migration
    let ts = Utc::now().format("%Y%m%d%H%M%S");
    let start_label = short_label(start_name);
    let end_label = short_label(end_name);
    let dir_name = format!("{ts}_squashed_{start_label}_to_{end_label}");
    let squash_dir = mdir.join(&dir_name);
    fs::create_dir_all(&squash_dir)?;

    let up_sql = up_parts.join("\n\n");
    let down_sql = if down_parts.is_empty() {
        "-- No automatic down migration.\n-- Add rollback statements manually if needed.".to_string()
    } else {
        down_parts.join("\n\n")
    };

    fs::write(squash_dir.join("up.sql"), &up_sql)?;
    fs::write(squash_dir.join("down.sql"), &down_sql)?;

    println!("{GRN}{BLD}✓  Squashed migration created:{RST}  {dir_name}");
    println!(
        "  {DIM}up.sql    — {} original migration(s), {} skipped (empty){RST}",
        range.len() - skipped_count,
        skipped_count
    );
    println!("  {DIM}down.sql  — rollback SQL{RST}");

    // Optionally delete originals
    if delete_old {
        println!("\n{YLW}Deleting original migration directories...{RST}");
        for name in range {
            let dir = mdir.join(name);
            fs::remove_dir_all(&dir)?;
            println!("  {RED}✗{RST}  {DIM}Removed {name}{RST}");
        }
    } else {
        println!("\n{DIM}Original migration directories kept.");
        println!("Use --delete-old to remove them.{RST}");
    }

    println!("\n{BLD}Next steps:{RST}");
    println!("  {DIM}1. Review the squashed migration files.{RST}");
    println!("  {DIM}2. If the original migrations are already applied in the DB,");
    println!("     mark the squashed migration as applied:{RST}");
    println!("       {BLD}cargo migrate {dir_name} --fake{RST}");
    println!("  {DIM}3. If starting fresh, run:{RST}");
    println!("       {BLD}cargo migrate{RST}");
    println!();

    Ok(())
}
