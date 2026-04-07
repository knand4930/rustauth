# Development

Guidance for developing and extending this Rust authentication project.

## Development workflow

1. Start by ensuring your local environment is configured and migrations are applied.
2. Modify or add application code under `src/`.
3. If you add or change database models, update migrations and schema files.
4. Run the app locally and verify behavior.

## Common commands

Build the project:

```bash
cargo build
```

Run the main application:

```bash
cargo run
```

Run the project in release mode:

```bash
cargo run --release
```

Run tests:

```bash
cargo test
```

Run linting or formatting:

```bash
cargo fmt
cargo clippy
```

## Database development commands

Apply migrations:

```bash
cargo run --bin migrate
```

Create a new migration manually:

```bash
cargo run --bin makemigrations <migration_name>
```

List migration status:

```bash
cargo run --bin showmigrations
```

Open the interactive SQL shell:

```bash
cargo run --bin shell
```

Open a native Postgres shell:

```bash
cargo run --bin dbshell
```

## Add a new app module

This repository includes a helper to scaffold app modules.

```bash
cargo run --bin startapp myapp
```

Then update the generated module with your models, handlers, and routes.

## Notes

- `src/bin/startapp.rs` automatically patches `src/main.rs`, `src/models/mod.rs`, `src/schema.rs`, `diesel.toml`, and `src/bin/makemigrations.rs`.
- Use `scripts/diesel-schema.sh sync` after schema changes to refresh `src/*/schemas.rs` files.
