use crate::commands::common::{
    BLD, CYN, DIM, GRN, RED, RST, YLW, applied_migration_rows, applied_set, connect_pool,
    ensure_history_table, get_migration_modules, mark_applied, mark_unapplied, migration_names,
    migrations_dir,
};
use anyhow::{Context, Result};
use sqlx::PgPool;
use std::fs;

#[derive(Debug)]
struct RiskItem {
    label: &'static str,
    level: &'static str,
    message: &'static str,
}

fn print_migrate_usage() {
    eprintln!("Usage:");
    eprintln!("  {BLD}cargo migrate{RST}                    apply all pending migrations");
    eprintln!("  {BLD}cargo migrate status{RST}             show migration status");
    eprintln!("  {BLD}cargo migrate --check{RST}            exit 1 if there are unapplied migrations");
    eprintln!("  {BLD}cargo migrate --plan{RST}             show what would be applied without running");
    eprintln!(
        "  {BLD}cargo migrate --fake-initial{RST}     mark first migration as applied (no SQL)"
    );
    eprintln!("  {BLD}cargo migrate zero{RST}               roll back ALL applied migrations");
    eprintln!("  {BLD}cargo migrate <name>{RST}             move DB to that migration state");
    eprintln!("  {BLD}cargo migrate <name> --fake{RST}      mark migration as applied (no SQL)");
    eprintln!();
}

fn print_showmigrations_usage() {
    println!("Usage:");
    println!("  {BLD}cargo showmigrations{RST}             list migrations with applied status");
    println!();
}

async fn ensure_database_prereqs(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .execute(pool)
        .await
        .context("Cannot enable pgcrypto extension required for gen_random_uuid()")?;

    Ok(())
}

fn detect_risks(sql: &str) -> Vec<RiskItem> {
    let upper = sql.to_uppercase();
    let mut risks = vec![];

    if upper.contains("DROP TABLE") {
        risks.push(RiskItem {
            label: "DROP TABLE",
            level: "HIGH",
            message: "destroys table and all its data",
        });
    }
    if upper.contains("DROP COLUMN") {
        risks.push(RiskItem {
            label: "DROP COLUMN",
            level: "HIGH",
            message: "permanently removes column data",
        });
    }
    if upper.contains("ALTER COLUMN") && upper.contains("TYPE") {
        risks.push(RiskItem {
            label: "ALTER COLUMN TYPE",
            level: "MEDIUM",
            message: "may fail or lose data on incompatible casts",
        });
    }
    if upper.contains("TRUNCATE") {
        risks.push(RiskItem {
            label: "TRUNCATE",
            level: "HIGH",
            message: "deletes every row in the table",
        });
    }
    if upper.contains("DELETE FROM") {
        risks.push(RiskItem {
            label: "DELETE FROM",
            level: "MEDIUM",
            message: "removes rows permanently",
        });
    }

    risks
}

fn print_sql_preview(sql: &str) {
    let statements: Vec<&str> = sql
        .split("\n\n")
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .collect();
    let show_count = statements.len().min(4);

    for statement in &statements[..show_count] {
        let first_line = statement.lines().next().unwrap_or("").trim();
        let has_more = statement.lines().count() > 1;
        let color = {
            let upper = first_line.to_uppercase();
            if upper.starts_with("CREATE TABLE") {
                GRN
            } else if upper.starts_with("DROP") {
                RED
            } else if upper.starts_with("ALTER") {
                YLW
            } else {
                DIM
            }
        };

        print!("     {color}{first_line}{RST}");
        if has_more {
            println!("  {DIM}...{RST}");
        } else {
            println!();
        }
    }

    if statements.len() > show_count {
        println!(
            "     {DIM}  … and {} more statement(s){RST}",
            statements.len() - show_count
        );
    }
}

fn find_migration<'a>(names: &'a [String], target: &str) -> Option<(usize, &'a str)> {
    if let Some(index) = names.iter().position(|name| name == target) {
        return Some((index, names[index].as_str()));
    }

    let matches: Vec<(usize, &String)> = names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.ends_with(target) || name.contains(target))
        .collect();

    match matches.len() {
        0 => None,
        1 => Some((matches[0].0, matches[0].1.as_str())),
        _ => {
            eprintln!("{YLW}Ambiguous name '{target}' — matches:{RST}");
            for (_, name) in matches {
                eprintln!("  {name}");
            }
            None
        }
    }
}

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

    for risk in detect_risks(&sql) {
        let color = if risk.level == "HIGH" { RED } else { YLW };
        println!(
            "     {color}⚠ {}{RST}  {DIM}{}{RST}",
            risk.label, risk.message
        );
    }

    sqlx::raw_sql(&sql)
        .execute(pool)
        .await
        .with_context(|| format!("Migration failed: {name}"))?;

    mark_applied(pool, name).await?;
    println!("     {GRN}✓ Applied{RST}");
    Ok(())
}

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

fn render_showmigrations_table(
    names: &[String],
    ts_map: &std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>,
) {
    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo showmigrations               ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    if names.is_empty() {
        println!("{DIM}  No migrations found in migrations/{RST}\n");
        return;
    }

    println!(
        "  {BLD}{:<4} {:<48} {}{RST}",
        "Stat", "Migration", "Modules"
    );
    println!("  {DIM}{}{RST}", "─".repeat(90));

    let mut applied_count = 0usize;
    for name in names {
        let sql =
            fs::read_to_string(migrations_dir().join(name).join("up.sql")).unwrap_or_default();
        let modules = get_migration_modules(&sql);
        let module_label = if modules.is_empty() {
            "global".to_string()
        } else {
            modules.join(", ")
        };

        if let Some(timestamp) = ts_map.get(name) {
            applied_count += 1;
            let applied_at = timestamp.format("%Y-%m-%d %H:%M UTC").to_string();
            println!(
                "  {GRN}[X]{RST}  {:<48} {DIM}{}  (applied {}){RST}",
                name, module_label, applied_at
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
}

async fn run_showmigrations_inner(pool: &PgPool) -> Result<()> {
    ensure_history_table(pool).await?;
    let names = migration_names()?;
    let ts_map = applied_migration_rows(pool).await?;
    render_showmigrations_table(&names, &ts_map);
    Ok(())
}

async fn cmd_run(pool: &PgPool) -> Result<()> {
    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo migrate                      ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    ensure_history_table(pool).await?;
    let applied = applied_set(pool).await?;
    let names = migration_names()?;

    let pending: Vec<&String> = names
        .iter()
        .filter(|name| !applied.contains(*name))
        .collect();
    if pending.is_empty() {
        println!("{GRN}✓  No pending migrations — database is up to date.{RST}\n");
        return Ok(());
    }

    println!("Applying {} pending migration(s)...\n", pending.len());
    for name in pending {
        apply_one(pool, name, false).await?;
        println!();
    }

    println!("{GRN}{BLD}✓  All pending migrations applied successfully.{RST}\n");
    Ok(())
}

async fn cmd_fake_initial(pool: &PgPool) -> Result<()> {
    ensure_history_table(pool).await?;
    let names = migration_names()?;
    let first = names
        .first()
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

async fn cmd_target(pool: &PgPool, target: &str, fake: bool) -> Result<()> {
    ensure_history_table(pool).await?;
    let names = migration_names()?;
    let applied = applied_set(pool).await?;

    let (target_index, full_name) = find_migration(&names, target).with_context(|| {
        format!(
            "Migration '{target}' not found.\nRun 'cargo showmigrations' to list available migrations."
        )
    })?;

    println!("\n{BLD}Target migration:{RST} {full_name}\n");

    let to_rollback: Vec<&String> = names[target_index + 1..]
        .iter()
        .filter(|name| applied.contains(*name))
        .rev()
        .collect();
    let to_apply: Vec<&String> = names[..=target_index]
        .iter()
        .filter(|name| !applied.contains(*name))
        .collect();

    if to_apply.is_empty() && to_rollback.is_empty() {
        println!("{GRN}✓  Already at '{full_name}' — nothing to do.{RST}\n");
        return Ok(());
    }

    if !to_rollback.is_empty() {
        println!("Rolling back {} migration(s)...\n", to_rollback.len());
        for name in to_rollback {
            rollback_one(pool, name).await?;
            println!();
        }
    }

    if !to_apply.is_empty() {
        println!("Applying {} migration(s)...\n", to_apply.len());
        for name in to_apply {
            apply_one(pool, name, fake).await?;
            println!();
        }
    }

    println!("{GRN}{BLD}✓  Database is now at: {full_name}{RST}\n");
    Ok(())
}

async fn cmd_target_module(pool: &PgPool, module: &str) -> Result<()> {
    ensure_history_table(pool).await?;
    let names = migration_names()?;
    let applied = applied_set(pool).await?;
    let module_upper = module.to_uppercase();

    let mut matched = vec![];
    for name in names {
        if applied.contains(&name) {
            continue;
        }

        let up_path = migrations_dir().join(&name).join("up.sql");
        let Ok(sql) = fs::read_to_string(&up_path) else {
            continue;
        };
        let sql_upper = sql.to_uppercase();
        if sql_upper.contains(&format!("TABLE {module_upper}."))
            || sql_upper.contains(&format!("SCHEMA {module_upper}"))
            || sql_upper.contains(&format!("INDEX IDX_{module_upper}_"))
            || sql_upper.contains(&format!("TABLE IF EXISTS {module_upper}."))
        {
            matched.push(name);
        }
    }

    if matched.is_empty() {
        anyhow::bail!("Migration or module '{module}' not found or no pending migrations for it.");
    }

    println!("\n{BLD}Targeting Module:{RST} {module}");
    println!(
        "Faking {} pending migration(s) for module...\n",
        matched.len()
    );

    for name in matched {
        println!("  {YLW}⊘{RST}  {BLD}{name}{RST}  {DIM}(faked — SQL not executed){RST}");
        mark_applied(pool, &name).await?;
    }

    println!("\n{GRN}{BLD}✓  Module '{module}' migrations faked.{RST}\n");
    Ok(())
}

async fn cmd_check(pool: &PgPool) -> Result<()> {
    ensure_history_table(pool).await?;
    let applied = applied_set(pool).await?;
    let names = migration_names()?;
    let pending: Vec<&String> = names.iter().filter(|n| !applied.contains(*n)).collect();

    if pending.is_empty() {
        println!("{GRN}✓  No pending migrations — database is up to date.{RST}\n");
        return Ok(());
    }

    eprintln!("{RED}{BLD}{} unapplied migration(s):{RST}", pending.len());
    for name in &pending {
        eprintln!("  {YLW}[ ]{RST}  {name}");
    }
    eprintln!("\nRun {BLD}cargo migrate{RST} to apply.\n");
    std::process::exit(1);
}

async fn cmd_plan(pool: &PgPool) -> Result<()> {
    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo migrate --plan               ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    ensure_history_table(pool).await?;
    let applied = applied_set(pool).await?;
    let names = migration_names()?;

    let pending: Vec<&String> = names.iter().filter(|n| !applied.contains(*n)).collect();
    if pending.is_empty() {
        println!("{GRN}✓  No pending migrations — database is up to date.{RST}\n");
        return Ok(());
    }

    println!("Would apply {} migration(s):\n", pending.len());
    for name in &pending {
        let up_path = migrations_dir().join(name).join("up.sql");
        let sql = fs::read_to_string(&up_path).unwrap_or_default();
        println!("  {CYN}↑{RST}  {BLD}{name}{RST}");
        print_sql_preview(&sql);
        let risks = detect_risks(&sql);
        for risk in &risks {
            let color = if risk.level == "HIGH" { RED } else { YLW };
            println!("     {color}⚠ {}{RST}  {DIM}{}{RST}", risk.label, risk.message);
        }
        println!();
    }

    println!("{YLW}Plan only — nothing was applied. Run {BLD}cargo migrate{RST}{YLW} to execute.{RST}\n");
    Ok(())
}

async fn cmd_zero(pool: &PgPool) -> Result<()> {
    println!("\n{BLD}╔══════════════════════════════════════╗");
    println!("║  cargo migrate zero                 ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    ensure_history_table(pool).await?;
    let applied = applied_set(pool).await?;
    let names = migration_names()?;

    let to_rollback: Vec<&String> = names
        .iter()
        .filter(|n| applied.contains(*n))
        .rev()
        .collect();

    if to_rollback.is_empty() {
        println!("{GRN}✓  No applied migrations — database is already at zero.{RST}\n");
        return Ok(());
    }

    println!("{RED}{BLD}Rolling back ALL {} applied migration(s)...{RST}\n", to_rollback.len());
    for name in to_rollback {
        rollback_one(pool, name).await?;
        println!();
    }

    println!("{GRN}{BLD}✓  All migrations rolled back. Database is at zero.{RST}\n");
    Ok(())
}

pub async fn run_showmigrations(args: &[String]) -> Result<()> {
    match args {
        [] => {}
        [flag] if matches!(flag.as_str(), "-h" | "--help") => {
            print_showmigrations_usage();
            return Ok(());
        }
        _ => {
            eprintln!("{YLW}showmigrations does not accept extra arguments.{RST}\n");
            print_showmigrations_usage();
            std::process::exit(1);
        }
    }

    let pool = connect_pool().await?;
    run_showmigrations_inner(&pool).await
}

pub async fn run_migrate(args: &[String]) -> Result<()> {
    if matches!(args, [flag] if matches!(flag.as_str(), "-h" | "--help")) {
        print_migrate_usage();
        return Ok(());
    }

    let pool = connect_pool().await?;
    ensure_database_prereqs(&pool).await?;

    match args {
        [] => cmd_run(&pool).await?,
        [status] if status == "status" => run_showmigrations_inner(&pool).await?,
        [check] if check == "--check" => cmd_check(&pool).await?,
        [plan] if plan == "--plan" => cmd_plan(&pool).await?,
        [zero] if zero == "zero" => cmd_zero(&pool).await?,
        [fake_initial] if fake_initial == "--fake-initial" => cmd_fake_initial(&pool).await?,
        [name, fake] if fake == "--fake" && !name.starts_with("--") => {
            if let Err(error) = cmd_target(&pool, name, true).await {
                if cmd_target_module(&pool, name).await.is_err() {
                    eprintln!("{RED}Error:{RST} {}", error);
                    std::process::exit(1);
                }
            }
        }
        [name] if !name.starts_with("--") => cmd_target(&pool, name, false).await?,
        [flag] => {
            eprintln!("{RED}Unknown option: {flag}{RST}\n");
            print_migrate_usage();
            std::process::exit(1);
        }
        _ => {
            eprintln!("{RED}Invalid migrate arguments.{RST}\n");
            print_migrate_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}
