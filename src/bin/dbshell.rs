// src/bin/dbshell.rs
//
// Opens a native psql session against DATABASE_URL from .env
//
//   cargo dbshell
//
// Useful psql shortcuts once inside:
//   \dt auth.*        — list tables in auth schema
//   \dt blog.*        — list tables in blog schema
//   \d auth.users     — describe a table
//   \dn               — list schemas
//   \q                — quit

use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;
use std::os::unix::process::CommandExt; // exec() replaces this process

fn main() -> Result<()> {
    dotenv().ok();

    let url = env::var("DATABASE_URL").context("DATABASE_URL not set in .env")?;

    // Parse out a friendly db name for the banner
    let db_name = url.split('/').last().unwrap_or("rustauth");

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  cargo dbshell  →  psql @ {:<31}║", db_name);
    println!("║  \\dt auth.*  \\dt blog.*  \\d <table>  \\q to quit         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Replace this process with psql — ctrl-c, tab completion, history all work
    let err = std::process::Command::new("psql").arg(&url).exec(); // never returns on success

    Err(err).context("Failed to launch psql — is it installed? (apt install postgresql-client)")
}
