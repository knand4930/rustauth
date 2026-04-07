# Authentication Service

A Rust authentication and app scaffold project using Axum, Diesel, SQLx, and PostgreSQL.

## What this repository contains

- `src/` — application code, modules for `user`, `activitylog`, `blogs`, `products`, and middleware.
- `src/bin/` — helper binaries for database shell, migrations, scaffolding, and inspection.
- `migrations/` — Diesel migration scripts for the database schema.
- `diesel.toml` — Diesel CLI configuration.
- `.env` — local environment variables (not committed).

## Highlights

- Axum-based HTTP service scaffold
- Diesel + SQLx database integration with Postgres
- JWT auth, password hashing, and email/Redis-ready dependencies
- App generator support via `cargo run --bin startapp`
- CLI helpers: `cargo run --bin shell`, `cargo run --bin migrate`, `cargo run --bin showmigrations`, `cargo run --bin dbshell`

## Quick start

1. Install prerequisites from `installation.md`.
2. Create a `.env` file with your local settings.
3. Run database migrations:

```bash
cargo run --bin migrate
```

4. Start the app:

```bash
cargo run
```

## Project commands

- `cargo build` — compile the project.
- `cargo run` — run the main application.
- `cargo run --bin migrate` — apply Diesel database migrations.
- `cargo run --bin makemigrations` — create new Diesel migrations (requires manual source changes).
- `cargo run --bin shell` — interactive SQL shell.
- `cargo run --bin showmigrations` — list migration status.
- `cargo run --bin dbshell` — open a native `psql` session against `DATABASE_URL`.

## More documentation

- `setup.md` — environment setup and configuration.
- `installation.md` — installing Rust, PostgreSQL, and CLI tools.
- `development.md` — development workflow and useful commands.
