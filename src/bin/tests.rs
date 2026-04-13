use anyhow::{Context, Result};
use std::env;
use std::process::Command;

const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";

fn print_usage() {
    println!("Usage:");
    println!("  {BLD}cargo tests{RST}                         run the project test suite");
    println!("  {BLD}cargo tests <filter>{RST}                run tests matching a filter");
    println!("  {BLD}cargo tests -- --nocapture{RST}         forward extra rust test flags");
    println!();
    println!("This command is a lightweight project wrapper around {BLD}cargo test{RST}.");
    println!();
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if matches!(args.as_slice(), [flag] if matches!(flag.as_str(), "-h" | "--help")) {
        print_usage();
        return Ok(());
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .arg("test")
        .args(&args)
        .status()
        .context("Failed to launch cargo test")?;

    if status.success() {
        return Ok(());
    }

    std::process::exit(status.code().unwrap_or(1));
}
