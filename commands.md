# CLI Commands Reference

Complete reference for all RustAuth CLI commands.

## 📋 Table of Contents

- [Overview](#overview)
- [Cargo Commands](#cargo-commands)
- [Migration Commands](#migration-commands)
- [Scaffolding Commands](#scaffolding-commands)
- [User Management](#user-management)
- [Development Commands](#development-commands)
- [Database Commands](#database-commands)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## Overview

RustAuth provides CLI commands through both standard Cargo tooling and custom binaries. Commands are available via:

```bash
# Cargo shorthand (via alias in Cargo.toml)
cargo <command>

# Or explicit binary
cargo run --bin <command> -- <args>
```

---

## Cargo Commands

### Standard Cargo Commands

**Build:**
```bash
# Development build (unoptimized, fast compilation)
cargo build

# Release build (optimized, slow compilation, fast execution)
cargo build --release

# Check for errors (no artifact generation)
cargo check
```

**Code Quality:**
```bash
# Format code
cargo fmt

# Format check
cargo fmt --check

# Lint with Clippy
cargo clippy

# Lint strict (all warnings)
cargo clippy -- -D warnings

# Auto-fix issues
cargo clippy --fix
```

**Testing:**
```bash
# Run all tests
cargo test

# Run test in release mode
cargo test --release

# Run specific test
cargo test test_login

# Run with output
cargo test -- --nocapture

# Run in parallel
cargo test -- --test-threads=4
```

**Documentation:**
```bash
# Generate and open docs
cargo doc --open

# Generate without opening
cargo doc

# For specific crate
cargo doc --package sqlx --open
```

---

## Migration Commands

### migrate - Apply Migrations

Apply all pending database migrations.

```bash
# Apply all pending migrations
cargo run --bin migrate

# Apply with verbose output
RUST_LOG=debug cargo run --bin migrate
```

**What it does:**
- Reads migration files from `migrations/` directory
- Applies pending migrations in order
- Updates `_sqlx_migrations` table
- Returns error if migration fails

**Output:**
```
✓ Running migration 20260414110602_auto
✓ Running migration 20260414115951_auto
All migrations completed successfully
```

---

### makemigrations - Generate Migrations

Auto-detect model changes and generate SQL migrations.

```bash
# Generate migration with default label "auto"
cargo run --bin makemigrations

# Generate with custom label
cargo makemigrations add_user_preferences

# Force regeneration
cargo makemigrations --force
```

**Features:**
- ✅ Auto-detects models in `src/apps/*/models.rs`
- ✅ Analyzes model directives (@schema, @table, @unique, @index)
- ✅ Generates both `up.sql` and `down.sql`
- ✅ Creates rollback script
- ✅ Validates relationships

**Generated Files:**
```
migrations/
├── 20260414110602_auto/
│   ├── up.sql        # Forward migration
│   └── down.sql      # Rollback script
```

**Model Directives:**
```rust
// @schema <name>           - PostgreSQL schema
// @table <name>            - Table name
// @unique                  - Unique constraint on field
// @unique columns=c1,c2    - Multi-column unique
// @index                   - Index on field
// @default <value>         - Default value
// @references schema.table - Foreign key reference
```

---

### showmigrations - Show Migration Status

Display migration status and history.

```bash
# Show all migrations
cargo run --bin showmigrations

# With verbose output
RUST_LOG=debug cargo run --bin showmigrations
```

**Output Format:**
```
[APPLIED] 20260414110602_auto    (2024-04-14 11:06:02)
[APPLIED] 20260414115951_auto    (2024-04-14 11:59:51)
[PENDING] 20260414125000_example
```

---

## Scaffolding Commands

### startapp - Create New App

Generate new app with all required files and configuration.

```bash
# Create basic app
cargo startapp products

# Create with custom schema
cargo startapp products --schema catalog

# Create with multiple schemas
cargo startapp products --schema catalog --schema inventory
```

**Generated Structure:**
```
src/apps/products/
├── mod.rs                  # Module exports & routes
├── models.rs              # Database models
├── schemas.rs             # Request/response DTOs
├── handlers.rs            # HTTP endpoint handlers
├── admin_config.rs        # Admin panel metadata
└── admin_registry.rs      # Admin registration
```

**Generated mod.rs:**
```rust
pub mod models;
pub mod schemas;
pub mod handlers;
pub mod admin_config;
pub mod admin_registry;

use axum::Router;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create))
        .route("/", get(handlers::list))
        .route("/:id", get(handlers::get))
        .route("/:id", put(handlers::update))
        .route("/:id", delete(handlers::delete))
}
```

**What You Need To Do:**
1. Define models in `models.rs`
2. Define DTOs in `schemas.rs`
3. Implement handlers in `handlers.rs`
4. Run `cargo makemigrations` to generate migration
5. Add routes to parent `mod.rs` if needed

---

## User Management

### createsuperuser - Create Admin User

Create a superuser account interactively.

```bash
# Create superuser (interactive)
cargo run --bin createsuperuser

# With predefined values (non-interactive)
cargo run --bin createsuperuser -- \
  --email admin@example.com \
  --password "SecurePass123!" \
  --name "Admin User"
```

**Prompts:**
```
Email: admin@example.com
Password: ••••••••••••
Full Name: Admin User
```

**Creates User With:**
- is_superuser = true
- is_active = true
- email_verified = true

---

## Development Commands

### shell - Interactive Shell

Open interactive shell for database queries.

```bash
# Open shell
cargo run --bin shell

# Example interactive session:
# rustauth> SELECT * FROM user.users LIMIT 1;
# (returns SQL results)
# rustauth> CREATE TABLE ...;
```

---

### dbshell - Postgres Shell

Open native `psql` shell with configured database connection.

```bash
# Open psql
cargo run --bin dbshell

# Equivalent to:
# psql postgres://rustauth:password@localhost:5432/auth_dev
```

---

### dbinspect - Database Inspection

Inspect database schema and statistics.

```bash
# Inspect database
cargo run --bin inspectdb

# Output includes:
# - All tables and columns
# - Data types
# - Constraints
# - Indexes
# - Table sizes
```

---

## Database Commands

### Full Database Operations

**Initialize Fresh Database:**
```bash
# 1. Drop old database (development only)
dropdb auth_dev

# 2. Create new database
createdb auth_dev

# 3. Apply all migrations
cargo run --bin migrate

# 4. Create superuser (optional)
cargo run --bin createsuperuser
```

**Backup Database:**
```bash
# SQL dump
pg_dump auth_dev > backup.sql

# Compressed dump
pg_dump -Fc auth_dev > backup.dump
```

**Restore Database:**
```bash
# From SQL file
psql auth_dev < backup.sql

# From compressed dump
pg_restore -d auth_dev backup.dump
```

---

## Other Commands

### tests - Run Tests

Run project tests with database setup.

```bash
# Run all tests
cargo run --bin tests

# Run specific test
cargo run --bin tests -- test_name

# With logging
RUST_LOG=debug cargo run --bin tests
```

---

## Examples

### Complete Setup Flow

```bash
# 1. Clone repository
git clone https://github.com/yourname/rustauth.git
cd rustauth

# 2. Install dependencies
cargo build

# 3. Initialize database
cargo run --bin migrate

# 4. Create superuser
cargo run --bin createsuperuser

# 5. Start development server
cargo run

# 6. View API docs
# Open http://localhost:8000/docs
```

### Adding New Feature

```bash
# 1. Scaffold new app
cargo startapp products

# 2. Edit generated files
# - src/apps/products/models.rs
# - src/apps/products/handlers.rs
# - etc.

# 3. Generate migration from models
cargo makemigrations add_products_table

# 4. Review generated migration
cat migrations/*_add_products_table/up.sql

# 5. Apply migration
cargo run --bin migrate

# 6. Test the app
cargo test

# 7. Start server
cargo run
```

### Database Troubleshooting

```bash
# Check migration status
cargo run --bin showmigrations

# Inspect database
cargo run --bin inspectdb

# Open shell for manual SQL
cargo run --bin shell

# View raw SQL dump
pg_dump auth_dev --schema-only | less
```

---

## Command Aliases

Add to `.bashrc` or `.zshrc` for convenience:

```bash
# Quick commands
alias rustauth-build='cargo build'
alias rustauth-run='cargo run'
alias rustauth-test='cargo test'
alias rustauth-fmt='cargo fmt && cargo clippy'
alias rustauth-migrate='cargo run --bin migrate'
alias rustauth-migrations='cargo run --bin showmigrations'
alias rustauth-startapp='cargo run --bin startapp'
alias rustauth-shell='cargo run --bin shell'
alias rustauth-db='cargo run --bin dbshell'

# Load in shell
source ~/.bashrc
```

Then use:
```bash
rustauth-run
rustauth-migrate
rustauth-startapp myapp
```

---

## Troubleshooting

### Command Not Found

```bash
# Make sure you're in project root
pwd  # Should be .../rustauth

# Check Cargo.toml exists
ls Cargo.toml

# Build may not have compiled the binary
cargo build

# Try with explicit path
cargo run --bin migrate
```

### Migrations Failed

```bash
# Check migration status
cargo run --bin showmigrations

# Check specific failure
RUST_LOG=debug cargo run --bin migrate

# Inspect database state
cargo run --bin shell
# SELECT * FROM _sqlx_migrations;
```

### Database Connection Issues

```bash
# Verify connection string
echo $DATABASE_URL

# Test direct connection
psql $DATABASE_URL -c "SELECT 1"

# Check if PostgreSQL is running
sudo systemctl status postgresql

# Restart PostgreSQL
sudo systemctl restart postgresql
```

---

## Related Documentation

- [setup.md](setup.md) - Initial setup
- [development.md](development.md) - Development workflow
- [DATABASE.md](DATABASE.md) - Database schema
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues

---

**Last Updated:** April 2024
- `// @index columns=col1,col2` - Multi-column index
- `// @default <value>` - Default value
- `// @nullable` - Force nullable
- `// @required` - Force NOT NULL
- `// @sql_type <type>` - Override SQL type
- `// @validate <validator>` - Field validator
- `// @references schema.table` - Foreign key reference

**Reserved Keyword Handling:**
The command automatically quotes PostgreSQL reserved keywords:
- `user` → `"user"`
- `order` → `"order"`
- `group` → `"group"`
- And 150+ more keywords

---

### 3. `cargo migrate` - Apply Migrations

Applies, rolls back, or targets specific migration states.

```bash
cargo migrate                    # Apply all pending migrations
cargo migrate status             # Show migration status
cargo migrate --fake-initial     # Mark first migration applied (no SQL)
cargo migrate <name>             # Move to specific migration
cargo migrate <name> --fake      # Mark migration applied (no SQL)
```

**Features:**
- ✅ Automatic migration ordering by timestamp
- ✅ Risk detection (DROP TABLE, DROP COLUMN, etc.)
- ✅ SQL preview before execution
- ✅ Forward and backward migration support
- ✅ Module-specific migration targeting
- ✅ Transaction-safe execution

**Example:**
```bash
# Apply all pending migrations
cargo migrate

# Show status
cargo migrate status

# Target specific migration
cargo migrate add_users_table

# Fake apply for testing
cargo migrate add_users_table --fake
```

---

### 4. `cargo tests` - Comprehensive Validation

Multi-phase validation system that checks database, code structure, imports/exports, and compilation.

```bash
cargo tests                      # Run all validation phases
cargo tests --no-db              # Skip database checks
cargo tests --no-compile         # Skip compilation check
cargo tests -v                   # Verbose output
```

**Validation Phases:**

#### Phase 1: Database & Schema Validation
- ✅ Checks migration status
- ✅ Validates database schema exists
- ✅ Verifies model-schema consistency
- ✅ Checks @schema and @table directives
- ✅ Lists all schemas and tables

#### Phase 2: Code Structure Validation
- ✅ Verifies required files exist (mod.rs, models.rs, schemas.rs, handlers.rs, etc.)
- ✅ Checks handler functions are present
- ✅ Validates request/response types in schemas
- ✅ Verifies model derive macros (sqlx::FromRow, Serialize)
- ✅ Checks declare_model_table! macro usage

#### Phase 3: Import/Export Validation
- ✅ Validates module declarations in mod.rs
- ✅ Checks routes() function exists
- ✅ Verifies admin registry wiring
- ✅ Confirms app is registered in apps/mod.rs
- ✅ Validates route merging
- ✅ Checks admin registry calls
- ✅ Verifies OpenAPI documentation setup

#### Phase 4: Compilation Check
- ✅ Runs `cargo check`
- ✅ Reports compilation errors
- ✅ Ensures project builds successfully

**Example Output:**
```
Phase 1: Database & Schema Validation
─────────────────────────────────────────────

  1.1 Checking migration status...
    ✓ All migrations applied (1 total)

  1.2 Validating database schema...
    ✓ Found 2 schema(s): blogs, user
      • Schema 'blogs': 2 table(s)
          - blog_posts
          - comments
      • Schema 'user': 10 table(s)
          - users
          - refresh_tokens
          ...

  1.3 Checking model-schema consistency...
    ✓ All models have proper schema directives

✓ Phase 1 passed: Database & Schema

Phase 2: Code Structure Validation
...

Phase 3: Import/Export Validation
...

Phase 4: Compilation Check
...

═══════════════════════════════════════
✓ All test phases passed successfully!
═══════════════════════════════════════
```

---

### 5. `cargo showmigrations` - Migration Status

Lists all migrations with their applied status.

```bash
cargo showmigrations
```

**Features:**
- ✅ Shows all migrations in order
- ✅ Displays applied/pending status
- ✅ Shows which modules each migration affects
- ✅ Shows application timestamps

---

### 6. `cargo dbshell` - Database Shell

Opens a PostgreSQL shell connected to your database.

```bash
cargo dbshell
```

---

## Workflow Example

### Creating a New Feature

1. **Create a new app:**
   ```bash
   cargo startapp products
   ```

2. **Define models in `src/apps/products/models.rs`:**
   ```rust
   // @schema products
   // @table products
   #[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
   pub struct Product {
       pub id: Uuid,
       pub name: String,
       pub price: f64,
       pub created_at: DateTime<Utc>,
   }
   crate::declare_model_table!(Product, "products", "products");
   ```

3. **Implement handlers in `src/apps/products/handlers.rs`:**
   ```rust
   pub async fn list_products(...) -> Result<impl IntoResponse, AppError> {
       // Implementation
   }
   ```

4. **Wire routes in `src/apps/products/mod.rs`:**
   ```rust
   pub fn routes() -> Router<AppState> {
       Router::new()
           .route("/api/v1/products", get(handlers::list_products))
   }
   ```

5. **Validate your work:**
   ```bash
   cargo tests -v
   ```

6. **Generate migration:**
   ```bash
   cargo makemigrations add_products
   ```

7. **Apply migration:**
   ```bash
   cargo migrate
   ```

8. **Run server:**
   ```bash
   cargo run
   ```

---

## Automatic Integration

All commands work together seamlessly:

- **`startapp`** → Auto-registers modules, routes, and admin
- **`makemigrations`** → Auto-discovers models from all apps
- **`migrate`** → Auto-applies migrations in order
- **`tests`** → Auto-validates everything end-to-end

**No manual wiring required!**

---

## Reserved Keywords

The system automatically quotes PostgreSQL reserved keywords in:
- Schema names
- Table names
- Column names (when used in ALTER statements)
- Foreign key references

This prevents syntax errors when using common names like `user`, `order`, `group`, etc.

**150+ keywords handled**, including:
- `user`, `order`, `group`, `select`, `from`, `where`
- `table`, `index`, `key`, `primary`, `foreign`
- `schema`, `database`, `role`, `grant`, `revoke`
- And many more...

---

## Troubleshooting

### Migration fails with "syntax error at or near 'user'"
**Solution:** This is now automatically fixed. The system quotes reserved keywords.

### Models not detected by makemigrations
**Solution:** Ensure you have:
1. `// @schema <name>` directive at the top of models.rs
2. `// @table <name>` directive before each struct
3. `crate::declare_model_table!()` macro call for each model

### App not showing in routes
**Solution:** Run `cargo startapp <appname>` again to repair the integration, or manually check:
- `src/apps/mod.rs` has `pub mod <appname>;`
- `src/apps/mod.rs` has `.merge(<appname>::routes())`
- `src/admin/registry.rs` has `<appname>::admin_registry::register(builder);`

### Tests fail on Phase 3
**Solution:** Check that:
- All required files exist in your app directory
- Module declarations are present in mod.rs
- Routes are properly merged
- Admin registry is called

---

## Best Practices

1. **Always run `cargo tests -v`** after making changes
2. **Use meaningful migration labels** (e.g., `add_user_profiles`)
3. **Review generated SQL** before applying migrations
4. **Commit migrations alongside `.schema_state.json`**
5. **Use directives liberally** (@index, @unique, @references, etc.)
6. **Keep handlers separate** from models and schemas
7. **Test with `--no-db` flag** when database is unavailable

---

## Advanced Usage

### Module-Specific Migrations
```bash
# Only apply migrations for a specific module
cargo migrate user --fake
```

### Target Migration State
```bash
# Rollback to a specific migration
cargo migrate initial_schema
```

### Verbose Testing
```bash
# See detailed validation output
cargo tests -v

# Skip database checks (faster)
cargo tests --no-db -v
```

### Custom Schema Names
```bash
# Create app with different PostgreSQL schema
cargo startapp orders --schema ecommerce
```

---

## Summary

RustAuth provides a complete, integrated development workflow:

- **No third-party tools** - Everything is built-in
- **Automatic detection** - Models, routes, and admin are auto-wired
- **Comprehensive validation** - 4-phase testing ensures correctness
- **Reserved keyword safety** - Automatic quoting prevents SQL errors
- **Clear documentation** - Step-by-step guidance at every stage

Start building with:
```bash
cargo startapp myapp
cargo tests -v
cargo makemigrations
cargo migrate
cargo run
```
