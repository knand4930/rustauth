// src/bin/dbshell.rs
//
// Opens a native psql session against DATABASE_URL from .env
//
//   cargo dbshell
//
// Useful psql shortcuts once inside:
//   \dt user.*        — list tables in the user schema
//   \dt blogs.*       — list tables in the blogs schema
//   \d user.users     — describe a table
//   \dn               — list schemas
//   \q                — quit

use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;
use std::os::unix::process::CommandExt; // exec() replaces this process

fn print_usage() {
    println!("Usage:");
    println!("  cargo dbshell");
    println!("  cargo dbshell -- -c \"SELECT 1\"");
    println!();
    println!("Anything after `--` is passed directly to `psql`.");
    println!();
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if matches!(args.as_slice(), [flag] if matches!(flag.as_str(), "-h" | "--help")) {
        print_usage();
        return Ok(());
    }

    dotenv().ok();

    let url = env::var("DATABASE_URL").context("DATABASE_URL not set in .env")?;

    // Parse out a friendly db name for the banner
    let db_name = url
        .rsplit('/')
        .next()
        .unwrap_or("rustauth")
        .split('?')
        .next()
        .unwrap_or("rustauth");

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  cargo dbshell  →  psql @ {:<31}║", db_name);
    println!("║  \\dt user.*  \\dt blogs.*  \\d <table>  \\q to quit        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Replace this process with psql — ctrl-c, tab completion, history all work
    let mut command = std::process::Command::new("psql");
    command.arg(&url);
    command.args(&args);
    let err = command.exec(); // never returns on success

    Err(err).context("Failed to launch psql — is it installed? (apt install postgresql-client)")
}
