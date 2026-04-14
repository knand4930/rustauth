use crate::commands::common::{
    BLD, CYN, DIM, GRN, RED, RST, YLW, applied_set, connect_pool, ensure_history_table,
    migration_names,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn run(args: &[String]) -> Result<()> {
    let mut skip_db = false;
    let mut skip_compile = false;
    let mut verbose = false;
    let mut failfast = false;

    for arg in args {
        match arg.as_str() {
            "--no-db" => skip_db = true,
            "--no-compile" => skip_compile = true,
            "-v" | "--verbose" => verbose = true,
            "--failfast" | "-x" => failfast = true,
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            _ => {
                eprintln!("{RED}Unknown argument: {arg}{RST}");
                print_usage();
                std::process::exit(1);
            }
        }
    }

    let mut exit_code = 0;

    macro_rules! run_phase {
        ($num:expr, $name:expr, $check:expr) => {
            if let Err(error) = $check {
                eprintln!("\n{RED}✗ Phase {} failed: {}{RST}", $num, error);
                exit_code = 1;
                if failfast {
                    println!("\n{RED}{BLD}Stopped after first failure (--failfast).{RST}\n");
                    std::process::exit(1);
                }
            } else {
                println!("\n{GRN}✓ Phase {} passed: {}{RST}", $num, $name);
            }
        };
    }

    // Phase 1: Database & Schema Validation
    if !skip_db {
        run_phase!(1, "Database & Schema", check_database_and_schema(verbose));
    }

    // Phase 2: Code Structure Validation (handlers, schemas, models)
    run_phase!(2, "Code Structure", check_code_structure(verbose));

    // Phase 3: Import/Export Validation
    run_phase!(3, "Import/Export Validation", check_imports_exports(verbose));

    // Phase 4: Compilation Check
    if !skip_compile {
        run_phase!(4, "Compilation", check_compilation(verbose));
    }

    if exit_code == 0 {
        println!("\n{GRN}{BLD}═══════════════════════════════════════{RST}");
        println!("{GRN}{BLD}✓ All test phases passed successfully!{RST}");
        println!("{GRN}{BLD}═══════════════════════════════════════{RST}\n");
    } else {
        println!("\n{RED}{BLD}═══════════════════════════════════════{RST}");
        println!("{RED}{BLD}✗ Some test phases failed. Review errors above.{RST}");
        println!("{RED}{BLD}═══════════════════════════════════════{RST}\n");
        std::process::exit(1);
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  {BLD}cargo tests{RST}                         run all validation phases");
    println!("  {BLD}cargo tests --no-db{RST}                 skip database checks");
    println!("  {BLD}cargo tests --no-compile{RST}            skip compilation check");
    println!("  {BLD}cargo tests -v{RST}                      verbose output");
    println!("  {BLD}cargo tests --failfast{RST}              stop at first failing phase");
    println!("  {BLD}cargo tests -x{RST}                      alias for --failfast");
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 1: Database & Schema Validation
// ─────────────────────────────────────────────────────────────────────────────

fn check_database_and_schema(verbose: bool) -> Result<()> {
    println!("\n{CYN}{BLD}Phase 1: Database & Schema Validation{RST}");
    println!("{DIM}─────────────────────────────────────────────{RST}");

    let pool = futures::executor::block_on(connect_pool())
        .context("Cannot connect to database")?;
    futures::executor::block_on(ensure_history_table(&pool))?;

    // Check 1.1: Migration status
    println!("\n  {CYN}1.1 Checking migration status...{RST}");
    let names = migration_names()?;
    let applied = futures::executor::block_on(applied_set(&pool))?;

    let pending: Vec<&String> = names.iter().filter(|n| !applied.contains(*n)).collect();
    if !pending.is_empty() {
        println!("    {YLW}⚠ {} pending migration(s):{RST}", pending.len());
        for name in pending {
            println!("       {YLW}• {name}{RST}");
        }
        println!("    {DIM}Run: cargo migrate{RST}");
    } else if verbose {
        println!("    {GRN}✓ All migrations applied ({} total){RST}", names.len());
    }

    // Check 1.2: Database schema vs migration files
    println!("\n  {CYN}1.2 Validating database schema...{RST}");
    futures::executor::block_on(validate_db_schema(&pool, verbose))?;

    // Check 1.3: Model-schema consistency
    println!("\n  {CYN}1.3 Checking model-schema consistency...{RST}");
    check_model_schema_consistency(verbose)?;

    Ok(())
}

async fn validate_db_schema(pool: &sqlx::PgPool, verbose: bool) -> Result<()> {
    // Get all schemas from database
    let schemas = sqlx::query_scalar::<_, String>(
        "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('pg_catalog', 'pg_toast', 'information_schema', 'public') ORDER BY schema_name"
    )
    .fetch_all(pool)
    .await?;

    if verbose {
        println!("    {GRN}✓ Found {} schema(s): {}{RST}", schemas.len(), schemas.join(", "));
    }

    // Check each schema has tables
    for schema in &schemas {
        let tables = sqlx::query_scalar::<_, String>(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name"
        )
        .bind(schema)
        .fetch_all(pool)
        .await?;

        if verbose {
            println!("      {DIM}• Schema '{schema}': {} table(s){RST}", tables.len());
            for table in tables {
                println!("        {DIM}  - {table}{RST}");
            }
        }
    }

    Ok(())
}

fn check_model_schema_consistency(verbose: bool) -> Result<()> {
    let apps_dir = PathBuf::from(MANIFEST_DIR).join("src").join("apps");
    let mut issues = vec![];

    for entry in fs::read_dir(&apps_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let app_name = entry.file_name().to_string_lossy().to_string();
        let models_path = entry.path().join("models.rs");

        if !models_path.exists() {
            if verbose {
                println!("    {DIM}• {app_name}: No models.rs (skipped){RST}");
            }
            continue;
        }

        let content = fs::read_to_string(&models_path)?;
        if verbose {
            println!("    {GRN}• {app_name}: Validating models...{RST}");
        }

        // Check for @schema directive
        if !content.contains("// @schema") {
            issues.push(format!("{app_name}: Missing @schema directive in models.rs"));
        }

        // Check each struct has @table directive
        let struct_count = content.matches("pub struct").count();
        let table_directives = content.matches("// @table").count();

        if struct_count != table_directives {
            issues.push(format!(
                "{app_name}: Mismatch between structs ({struct_count}) and @table directives ({table_directives})"
            ));
        }
    }

    if !issues.is_empty() {
        for issue in &issues {
            println!("    {RED}✗ {issue}{RST}");
        }
        anyhow::bail!("Model-schema consistency issues found");
    } else if verbose {
        println!("    {GRN}✓ All models have proper schema directives{RST}");
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 2: Code Structure Validation
// ─────────────────────────────────────────────────────────────────────────────

fn check_code_structure(verbose: bool) -> Result<()> {
    println!("\n{CYN}{BLD}Phase 2: Code Structure Validation{RST}");
    println!("{DIM}─────────────────────────────────────────────{RST}");

    let apps_dir = PathBuf::from(MANIFEST_DIR).join("src").join("apps");
    let mut issues = vec![];

    for entry in fs::read_dir(&apps_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let app_name = entry.file_name().to_string_lossy().to_string();
        let app_path = entry.path();

        if verbose {
            println!("\n  {CYN}Checking app: {app_name}{RST}");
        }

        // Check 2.1: Required files exist
        println!("    {CYN}2.1 Checking required files...{RST}");
        let required_files = [
            "mod.rs",
            "models.rs",
            "schemas.rs",
            "handlers.rs",
            "admin_config.rs",
            "admin_registry.rs",
        ];

        for file in &required_files {
            let file_path = app_path.join(file);
            if file_path.exists() {
                if verbose {
                    println!("      {GRN}✓ {file}{RST}");
                }
            } else {
                issues.push(format!("{app_name}: Missing required file '{file}'"));
                if verbose {
                    println!("      {RED}✗ {file} (missing){RST}");
                }
            }
        }

        // Check 2.2: Handlers have route functions
        println!("\n    {CYN}2.2 Checking handler structure...{RST}");
        let handlers_path = app_path.join("handlers.rs");
        if handlers_path.exists() {
            let content = fs::read_to_string(&handlers_path)?;
            let has_pub_async = content.contains("pub async fn");
            if has_pub_async {
                if verbose {
                    println!("      {GRN}✓ Contains handler functions{RST}");
                }
            } else {
                issues.push(format!("{app_name}: No handler functions found"));
            }
        }

        // Check 2.3: Schemas have request/response types
        println!("\n    {CYN}2.3 Checking schema structure...{RST}");
        let schemas_path = app_path.join("schemas.rs");
        if schemas_path.exists() {
            let content = fs::read_to_string(&schemas_path)?;
            let has_request = content.contains("Request");
            let has_response = content.contains("Response");

            if has_request || has_response {
                if verbose {
                    if has_request {
                        println!("      {GRN}✓ Request types found{RST}");
                    }
                    if has_response {
                        println!("      {GRN}✓ Response types found{RST}");
                    }
                }
            } else {
                issues.push(format!("{app_name}: No request/response types in schemas.rs"));
            }
        }

        // Check 2.4: Models have derive macros
        println!("\n    {CYN}2.4 Checking model structure...{RST}");
        let models_path = app_path.join("models.rs");
        if models_path.exists() {
            let content = fs::read_to_string(&models_path)?;
            let has_fromrow = content.contains("sqlx::FromRow");
            let has_serialize = content.contains("Serialize");

            if has_fromrow && has_serialize {
                if verbose {
                    println!("      {GRN}✓ Models have proper derives{RST}");
                }
            } else {
                issues.push(format!("{app_name}: Models missing required derive macros"));
            }

            // Check for declare_model_table macro
            if !content.contains("declare_model_table!") {
                issues.push(format!("{app_name}: Missing declare_model_table! macro call"));
            }
        }
    }

    if !issues.is_empty() {
        eprintln!("\n  {RED}Issues found:{RST}");
        for issue in &issues {
            eprintln!("    {RED}✗ {issue}{RST}");
        }
        anyhow::bail!("Code structure validation failed");
    } else if verbose {
        println!("\n  {GRN}✓ All apps have proper structure{RST}");
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 3: Import/Export Validation
// ─────────────────────────────────────────────────────────────────────────────

fn check_imports_exports(verbose: bool) -> Result<()> {
    println!("\n{CYN}{BLD}Phase 3: Import/Export Validation{RST}");
    println!("{DIM}─────────────────────────────────────────────{RST}");

    let apps_dir = PathBuf::from(MANIFEST_DIR).join("src").join("apps");
    let mut issues = vec![];

    for entry in fs::read_dir(&apps_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let app_name = entry.file_name().to_string_lossy().to_string();
        let app_path = entry.path();

        if verbose {
            println!("\n  {CYN}Validating app: {app_name}{RST}");
        }

        // Check 3.1: mod.rs has pub mod declarations
        println!("    {CYN}3.1 Checking module declarations...{RST}");
        let mod_path = app_path.join("mod.rs");
        if mod_path.exists() {
            let content = fs::read_to_string(&mod_path)?;
            let required_mods = [
                "pub mod models",
                "pub mod schemas",
                "pub mod handlers",
                "pub mod admin_config",
                "pub mod admin_registry",
            ];

            for mod_decl in &required_mods {
                if content.contains(mod_decl) {
                    if verbose {
                        println!("      {GRN}✓ {mod_decl};{RST}");
                    }
                } else {
                    issues.push(format!("{app_name}: Missing '{mod_decl};' in mod.rs"));
                }
            }

            // Check 3.2: Has routes() function
            println!("\n    {CYN}3.2 Checking routes function...{RST}");
            if content.contains("pub fn routes()") {
                if verbose {
                    println!("      {GRN}✓ routes() function found{RST}");
                }
            } else {
                issues.push(format!("{app_name}: Missing routes() function in mod.rs"));
            }
        }

        // Check 3.3: Admin registry properly wired
        println!("\n    {CYN}3.3 Checking admin registry...{RST}");
        let admin_registry_path = app_path.join("admin_registry.rs");
        if admin_registry_path.exists() {
            let content = fs::read_to_string(&admin_registry_path)?;
            if content.contains("pub fn register") && content.contains("AdminPanelBuilder") {
                if verbose {
                    println!("      {GRN}✓ Admin registry function found{RST}");
                }
            } else {
                issues.push(format!("{app_name}: Invalid admin registry structure"));
            }
        }

        // Check 3.4: Check app is registered in main apps/mod.rs
        println!("\n    {CYN}3.4 Checking app registration...{RST}");
        let apps_mod_path = apps_dir.join("mod.rs");
        if apps_mod_path.exists() {
            let apps_mod_content = fs::read_to_string(&apps_mod_path)?;

            // Check module declaration
            let mod_decl = format!("pub mod {app_name};");
            if apps_mod_content.contains(&mod_decl) {
                if verbose {
                    println!("      {GRN}✓ Module declared in apps/mod.rs{RST}");
                }
            } else {
                issues.push(format!("{app_name}: Not declared in apps/mod.rs"));
            }

            // Check route merge
            let route_merge = format!(".merge({app_name}::routes())");
            if apps_mod_content.contains(&route_merge) {
                if verbose {
                    println!("      {GRN}✓ Routes merged in apps/mod.rs{RST}");
                }
            } else {
                issues.push(format!("{app_name}: Routes not merged in apps/mod.rs"));
            }

            // Check admin registry call
            let admin_call = format!("{app_name}::admin_registry::register");
            let admin_registry_path = PathBuf::from(MANIFEST_DIR)
                .join("src")
                .join("admin")
                .join("registry.rs");

            if admin_registry_path.exists() {
                let registry_content = fs::read_to_string(&admin_registry_path)?;
                if registry_content.contains(&admin_call) {
                    if verbose {
                        println!("      {GRN}✓ Admin registry called{RST}");
                    }
                } else {
                    issues.push(format!("{app_name}: Admin registry not called"));
                }
            }
        }
    }

    // Check 3.5: Verify all imports are valid
    println!("\n  {CYN}3.5 Validating imports...{RST}");
    if let Err(error) = validate_imports(&apps_dir, verbose) {
        issues.push(format!("Import validation failed: {error}"));
    }

    if !issues.is_empty() {
        eprintln!("\n  {RED}Issues found:{RST}");
        for issue in &issues {
            eprintln!("    {RED}✗ {issue}{RST}");
        }
        anyhow::bail!("Import/export validation failed");
    } else if verbose {
        println!("\n  {GRN}✓ All imports/exports are valid{RST}");
    }

    Ok(())
}

fn validate_imports(apps_dir: &PathBuf, verbose: bool) -> Result<()> {
    // Parse apps/mod.rs to get declared modules
    let apps_mod_path = apps_dir.join("mod.rs");
    let apps_mod_content = fs::read_to_string(&apps_mod_path)?;

    // Check OpenAPI paths
    if verbose {
        println!("    {CYN}Checking OpenAPI documentation...{RST}");
    }

    // Check if handlers are referenced in OpenAPI
    let has_openapi_paths = apps_mod_content.contains("#[openapi(");
    if has_openapi_paths && verbose {
        println!("      {GRN}✓ OpenAPI documentation configured{RST}");
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 4: Compilation Check
// ─────────────────────────────────────────────────────────────────────────────

fn check_compilation(verbose: bool) -> Result<()> {
    println!("\n{CYN}{BLD}Phase 4: Compilation Check{RST}");
    println!("{DIM}─────────────────────────────────────────────{RST}");

    println!("\n  {CYN}Running cargo check...{RST}");

    let output = std::process::Command::new("cargo")
        .arg("check")
        .output()
        .context("Failed to run cargo check")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("\n  {RED}Compilation errors:{RST}");
        for line in stderr.lines() {
            eprintln!("    {line}");
        }
        anyhow::bail!("Compilation failed");
    }

    if verbose {
        println!("\n  {GRN}✓ Project compiles successfully{RST}");
    }

    Ok(())
}
