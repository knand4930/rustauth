use crate::commands::common::{BLD, CYN, DIM, GRN, RED, RST, connect_pool};
use anyhow::{Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use std::io::{self, Write};
use uuid::Uuid;

fn prompt(label: &str, hidden: bool) -> Result<String> {
    print!("{CYN}{label}:{RST} ");
    io::stdout().flush()?;

    if hidden {
        // Read without echo using a simple approach
        let value = rpassword_read()?;
        println!();
        Ok(value)
    } else {
        let mut value = String::new();
        io::stdin().read_line(&mut value)?;
        Ok(value.trim().to_string())
    }
}

/// Minimal no-echo password read using termios (POSIX).
/// Falls back to normal stdin read if the terminal cannot be configured.
fn rpassword_read() -> Result<String> {
    // Attempt to disable echo via stty; fall back to visible input gracefully.
    let disable = std::process::Command::new("stty")
        .arg("-echo")
        .status();

    let mut value = String::new();
    io::stdin().read_line(&mut value)?;

    if disable.is_ok() {
        let _ = std::process::Command::new("stty").arg("echo").status();
    }

    Ok(value.trim().to_string())
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {e}"))
}

fn print_usage() {
    println!("Usage:");
    println!("  {BLD}cargo createsuperuser{RST}                         interactive prompts");
    println!("  {BLD}cargo createsuperuser --email <email>{RST}         set email non-interactively");
    println!("  {BLD}cargo createsuperuser --email <e> --password <p>{RST}  fully non-interactive");
    println!();
}

pub async fn run(args: &[String]) -> Result<()> {
    let mut email_arg: Option<String> = None;
    let mut password_arg: Option<String> = None;
    let mut full_name_arg: Option<String> = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            "--email" => {
                i += 1;
                email_arg = args.get(i).cloned();
            }
            "--password" => {
                i += 1;
                password_arg = args.get(i).cloned();
            }
            "--name" => {
                i += 1;
                full_name_arg = args.get(i).cloned();
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
    println!("║  cargo createsuperuser              ║");
    println!("╚══════════════════════════════════════╝{RST}\n");

    // ── Collect inputs ─────────────────────────────────────────────────────────
    let email = match email_arg {
        Some(e) => e,
        None => {
            let e = prompt("Email address", false)?;
            if e.is_empty() {
                eprintln!("{RED}Error: email cannot be empty.{RST}");
                std::process::exit(1);
            }
            e
        }
    };

    if !email.contains('@') || !email.contains('.') {
        eprintln!("{RED}Error: '{email}' does not look like a valid email address.{RST}");
        std::process::exit(1);
    }

    let password = match password_arg {
        Some(p) => p,
        None => {
            let p1 = prompt("Password", true)?;
            if p1.len() < 8 {
                eprintln!("{RED}Error: password must be at least 8 characters.{RST}");
                std::process::exit(1);
            }
            let p2 = prompt("Password (again)", true)?;
            if p1 != p2 {
                eprintln!("{RED}Error: passwords do not match.{RST}");
                std::process::exit(1);
            }
            p1
        }
    };

    let full_name = full_name_arg.unwrap_or_else(|| {
        prompt("Full name (optional — press Enter to skip)", false)
            .unwrap_or_default()
    });

    // ── Hash and insert ────────────────────────────────────────────────────────
    println!("\n{CYN}Connecting to database...{RST}");
    let pool = connect_pool().await?;

    // Check for duplicate email
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM \"user\".users WHERE email = $1)"
    )
    .bind(&email)
    .fetch_one(&pool)
    .await
    .context("Failed to check for existing email")?;

    if exists {
        eprintln!("{RED}Error: a user with email '{email}' already exists.{RST}");
        std::process::exit(1);
    }

    println!("{CYN}Hashing password...{RST}");
    let hashed = hash_password(&password)?;

    let id = Uuid::new_v4();
    let full_name_val: Option<String> = if full_name.is_empty() {
        None
    } else {
        Some(full_name.clone())
    };

    sqlx::query(
        r#"
        INSERT INTO "user".users
            (id, email, password, full_name, is_active, is_superuser, is_staffuser,
             is_guest, email_verified, phone_verified, mfa_enabled, timezone, language)
        VALUES
            ($1, $2, $3, $4, TRUE, TRUE, TRUE,
             FALSE, FALSE, FALSE, FALSE, 'UTC', 'en')
        "#,
    )
    .bind(id)
    .bind(&email)
    .bind(&hashed)
    .bind(full_name_val.as_deref())
    .execute(&pool)
    .await
    .context("Failed to insert superuser")?;

    println!("\n{GRN}{BLD}✓  Superuser created successfully!{RST}");
    println!("  {CYN}ID         :{RST}  {BLD}{id}{RST}");
    println!("  {CYN}Email      :{RST}  {BLD}{email}{RST}");
    if !full_name.is_empty() {
        println!("  {CYN}Full name  :{RST}  {BLD}{full_name}{RST}");
    }
    println!("  {CYN}Superuser  :{RST}  {BLD}true{RST}");
    println!("  {CYN}Staff      :{RST}  {BLD}true{RST}");
    println!();
    println!("{DIM}You can now log in with this account at your API/admin panel.{RST}");
    println!();

    Ok(())
}
