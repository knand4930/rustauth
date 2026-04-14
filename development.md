# Development Guide

Complete development workflow and guidelines for working on RustAuth.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Common Development Tasks](#common-development-tasks)
- [Database Development](#database-development)
- [Creating New Features](#creating-new-features)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Debugging](#debugging)
- [Performance](#performance)
- [Best Practices](#best-practices)

---

## Quick Start

### Start Development Server

```bash
# Terminal 1: Start the application
cargo run

# You should see:
# Starting server on 127.0.0.1:8000
# Database connected
# AdminX initialized...

# Application is now running at http://localhost:8000
```

### View API Documentation

Open in browser:
```
http://localhost:8000/docs
```

### Make Changes & Reload

```bash
# Terminal 2: Make code changes
# Application will auto-rebuild (see terminal 1)

# Or use cargo-watch for auto-reload
cargo watch -x run
```

---

## Project Structure

### Main Files

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library exports
├── config.rs            # Configuration management
├── db.rs                # Database initialization
├── error.rs             # Error types
├── response.rs          # Response types
└── state.rs             # Application state
```

### Application Modules (src/apps/)

Each app is self-contained with its own models, handlers, schemas:

```
apps/
├── user/                # User management
│   ├── mod.rs          # App exports & routes
│   ├── models.rs       # Database models
│   ├── schemas.rs      # Request/response DTOs
│   ├── handlers.rs     # HTTP handlers
│   └── admin_*.rs      # Admin panel integration
│
└── blogs/               # Blog functionality
    ├── mod.rs
    ├── models.rs       # BlogPost, Comment
    ├── schemas.rs
    └── handlers.rs
```

### CLI Commands (src/bin/)

Standalone executables for database management:

```
bin/
├── migrate.rs           # Apply migrations
├── makemigrations.rs    # Generate migrations
├── startapp.rs          # Scaffold new app
├── createsuperuser.rs   # Create admin user
└── shell.rs             # Interactive shell
```

---

## Common Development Tasks

### Build Project

```bash
# Development build (unoptimized, faster compilation)
cargo build

# Release build (optimized, slower compilation, runs faster)
cargo build --release

# Check for errors without producing binary
cargo check
```

### Format Code

```bash
# Auto-format all code
cargo fmt

# Check if code needs formatting
cargo fmt --check

# Format specific file
cargo fmt -- src/main.rs
```

### Lint Code

```bash
# Run Clippy linter
cargo clippy

# Strict mode (all warnings)
cargo clippy -- -D warnings

# Specific lint check
cargo clippy --fix
```

### Run Application

```bash
# Standard run
cargo run

# With logging
RUST_LOG=debug cargo run

# With specific log level for modules
RUST_LOG=sqlx=debug,tower_http=trace cargo run

# In release mode
cargo run --release
```

### View Docs

```bash
# Generate and open documentation
cargo doc --open

# For specific crate
cargo doc --package sqlx --open
```

---

## Database Development

### Apply Migrations

```bash
# Apply all pending migrations
cargo run --bin migrate

# Check status
cargo run --bin showmigrations

# Revert last migration
cargo run --bin migrate down
```

### Generate Migrations

```bash
# Auto-generate migration from model changes
cargo makemigrations

# With custom label
cargo makemigrations add_user_preferences

# SQL migration files appear in migrations/
ls migrations/*/
```

### Database Shell

```bash
# Interactive SQL shell with psql
cargo run --bin dbshell

# Or use interactive shell helper
cargo run --bin shell

# Direct SQL query
psql -U rustauth -d auth_dev -c "SELECT * FROM user.users;"
```

### Database Inspection

```bash
# List all tables
\dt user.*
\dt blogs.*

# Show table structure
\d user.users

# Show indexes
\di

# Show table size
SELECT pg_size_pretty(pg_total_relation_size('user.users'));
```

---

## Creating New Features

### 1. Create New App

```bash
# Scaffold new app structure
cargo startapp products

# Generated files:
# src/apps/products/
#   ├── mod.rs
#   ├── models.rs
#   ├── schemas.rs
#   ├── handlers.rs
#   └── admin_*.rs
```

### 2. Define Models

Edit `src/apps/products/models.rs`:

```rust
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// @schema products
// @table products
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    
    // @unique
    pub name: String,
    
    pub description: String,
    
    // @index
    pub category: String,
    
    // @default 0
    pub price: f64,
    
    // @references user.users
    pub owner_id: Uuid,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 3. Define Schemas (DTOs)

Edit `src/apps/products/schemas.rs`:

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: String,
    pub category: String,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProductResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub price: f64,
    pub owner_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub price: Option<f64>,
}
```

### 4. Create Handlers

Edit `src/apps/products/handlers.rs`:

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use crate::state::AppState;
use super::{models::Product, schemas::*};

/// Create new product
pub async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), StatusCode> {
    // 1. Validate input
    // 2. Create in database
    // 3. Return response
    Ok((StatusCode::CREATED, Json(/* ... */)))
}

/// Get product by ID
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>, StatusCode> {
    // Query product
    // Return response
    Ok(Json(/* ... */))
}

/// List products
pub async fn list_products(
    State(state): State<AppState>,
) -> Json<Vec<ProductResponse>> {
    // Query products
    Json(vec![])
}

/// Update product
pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, StatusCode> {
    // Update product
    Ok(Json(/* ... */))
}

/// Delete product
pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Delete product
    Ok(StatusCode::NO_CONTENT)
}
```

### 5. Register Routes

Edit `src/apps/products/mod.rs`:

```rust
pub mod models;
pub mod schemas;
pub mod handlers;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create_product))
        .route("/", get(handlers::list_products))
        .route("/:id", get(handlers::get_product))
        .route("/:id", put(handlers::update_product))
        .route("/:id", delete(handlers::delete_product))
}
```

### 6. Update Main Router

Edit `src/apps/mod.rs`:

```rust
// Add module
pub mod products;

// In router_builder function
.nest("/api/products", products::router())
```

### 7. Generate Migration

```bash
# Auto-detect model changes
cargo makemigrations

# Apply migration
cargo run --bin migrate

# Verify
cargo run --bin showmigrations
```

---

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run in specific module
cargo test --lib user::
```

### Integration Tests

Create `tests/integration_test.rs`:

```rust
use rustauth::*;

#[tokio::test]
async fn test_user_login() {
    // Setup
    let pool = create_test_pool().await;
    
    // Test
    let result = login_user(&pool, "user@test.com", "password").await;
    
    // Assert
    assert!(result.is_ok());
}
```

### Run Tests

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test file
cargo test --test integration_test
```

---

## Code Quality

### Pre-Commit Checks

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Tests
cargo test

# Documentation
cargo doc --no-deps
```

### Commit Hooks (Optional)

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "🔍 Running pre-commit checks..."

echo "  📝 Formatting..."
cargo fmt --check

echo "  🚨 Linting..."
cargo clippy -- -D warnings

echo "  ✅ Testing..."
cargo test --quiet

echo "✅ All checks passed!"
```

Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

---

## Debugging

### Enable All Logging

```bash
# Maximum verbosity
RUST_LOG=trace cargo run

# Specific module
RUST_LOG=rustauth=debug,sqlx=debug cargo run

# With timestamp
RUST_LOG=debug,debug RUST_LOG_FORMAT=timestamp cargo run
```

### Print Debugging

```rust
// Print variables
println!("Value: {:?}", value);

// Or use dbg! macro
let x = dbg!(some_calculation());

// Logging is better for production
tracing::debug!("Debug info");
tracing::error!("Error: {}", error);
```

### Debug with Breakpoints

```bash
# Install debugging tools
brew install lldb  # macOS
sudo apt install lldb  # Linux

# Run with debugger
rust-lldb ./target/debug/rustauth
```

---

## Performance

### Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flame graph
cargo flamegraph

# Open result
# Generated in flamegraph.svg
```

### Benchmarking

```bash
# Install criterion
cargo install cargo-criterion

# Create benchmark
# benches/benchmarks.rs

# Run benchmarks
cargo bench
```

### Query Analysis

```sql
-- Show query plan
EXPLAIN ANALYZE SELECT * FROM user.users WHERE email = ?;

-- Check for slow queries
SELECT query, calls, total_time, mean_time 
FROM pg_stat_statements 
ORDER BY mean_time DESC LIMIT 10;

-- Check table/index sizes
SELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) 
FROM pg_tables 
WHERE schemaname = 'user'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

---

## Best Practices

### Error Handling

```rust
// ✅ Use Result types
pub async fn get_user(id: Uuid) -> Result<User> {
    // ...
}

// ❌ Don't use unwrap
let user = get_user(id).await.unwrap();  // Panic if error!

// ✅ Use ? operator
let user = get_user(id).await?;  // Propagate error
```

### Async/Await

```rust
// ✅ Use async/await
pub async fn fetch_data() -> Result<Data> {
    // Async operations
    let data = db.query().await?;
    Ok(data)
}

// ❌ Don't block
std::thread::sleep(Duration::from_secs(1));  // Blocks event loop!
```

### Database Queries

```rust
// ✅ Use parameterized queries
sqlx::query("SELECT * FROM user.users WHERE email = $1")
    .bind(email)
    .fetch_optional(&pool)
    .await?

// ❌ Don't concatenate
format!("SELECT * FROM users WHERE email = '{}'", email)  // SQL injection!
```

### Resource Management

```rust
// ✅ Connections are automatically closed
{
    let user = sqlx::query_as::<_, User>(/* ... */)
        .fetch_one(&pool)
        .await?;
    // Automatically returned to pool
}

// ✅ Transactions are automatically rolled back on error
let mut tx = pool.begin().await?;
// ... operations ...
tx.commit().await?;  // Only commits if explicit
```

---

## Useful Commands Reference

```bash
# Setup
cargo build
cargo run

# Quality
cargo fmt
cargo clippy
cargo test

# Database
cargo run --bin migrate
cargo makemigrations
cargo run --bin startapp myapp

# Inspection
cargo tree           # Dependencies
cargo outdated       # Outdated packages
cargo audit          # Security vulnerabilities

# Performance
cargo build --release  # Optimized build
cargo flamegraph       # Flame graph profiling
```

---

## Need Help?

- 📖 [README.md](README.md) - Project overview
- 🏗️ [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- 📚 [API.md](API.md) - API reference
- 🛠️ [commands.md](commands.md) - CLI commands
- 🆘 [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues

Happy coding! 🚀
