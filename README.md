# RustAuth API - Authentication & Blog Service

A production-ready **Rust authentication service** with user management, blog functionality, and comprehensive API documentation. Built with modern async web frameworks and industry best practices.

## 📋 Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Technology Stack](#technology-stack)
- [Project Architecture](#project-architecture)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Setup & Configuration](#setup--configuration)
- [Quick Start](#quick-start)
- [API Documentation](#api-documentation)
- [Project Commands](#project-commands)
- [Database Management](#database-management)
- [Development Workflow](#development-workflow)
- [Error Handling](#error-handling)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

**RustAuth** is a fully-featured authentication and content management API built with **Rust**, designed to demonstrate modern backend development practices. It provides:

- **JWT-based authentication** with secure password hashing (Argon2)
- **User management** with CRUD operations
- **Blog platform** with posts and comments system
- **Email support** for notifications and verification
- **Redis integration** for caching and sessions
- **Comprehensive API documentation** via Swagger UI
- **CLI tooling** for database management and scaffolding

This project serves as both a **production-ready service** and a **learning resource** for Rust backend developers.

---

## Key Features

### 🔐 Authentication & Security
- JWT token-based authentication with configurable expiration
- Argon2 password hashing (resistant to GPU attacks)
- Role-based access control (RBAC) ready
- Secure credential validation
- Email verification support

### 👥 User Management
- User registration with email validation
- User profile updates (email, password, profile info)
- User listing with pagination support
- Soft delete support for user data
- Account deactivation

### 📝 Blog Platform
- Create, read, update, delete blog posts
- Comments system for community engagement
- Post filtering and pagination
- Author attribution and timestamps
- Draft/publish status management

### 🔧 Infrastructure
- PostgreSQL for persistent data storage
- Redis for caching and session management
- Structured logging with tracing
- Environment-based configuration
- Database migrations via Diesel
- CORS support for cross-origin requests

### 📚 Developer Experience
- Interactive API documentation (Swagger UI)
- OpenAPI 3.0 specification auto-generation
- Custom error handling with semantic HTTP status codes
- Database shell (`psql` wrapper)
- Migration scaffolding tools
- Health check endpoints

---

## Technology Stack

### Core Framework
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Web Framework** | [Axum](https://github.com/tokio-rs/axum) | 0.8.8 | Modular & composable HTTP framework |
| **Async Runtime** | [Tokio](https://tokio.rs) | 1.51.0 | Async execution engine |
| **ORM/Query Builder** | [SQLx](https://sqlx.rs) | 0.8.6 | Compile-time checked SQL queries |
| **Migrations** | [Diesel CLI](https://diesel.rs) | - | Database schema management |

### Authentication & Security
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **JWT Tokens** | [jsonwebtoken](https://github.com/Keats/jsonwebtoken) | 10.3.0 | Token generation & validation |
| **Password Hashing** | [Argon2](https://github.com/RustCrypto/password-hashing) | 0.5.3 | GPU-resistant hashing |
| **Random Generation** | [Rand](https://crates.io/crates/rand) | 0.10.0 | Secure randomization |

### Data & Serialization
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **JSON** | [Serde](https://serde.rs) + [Serde JSON](https://github.com/serde-rs/json) | 1.0.228 + 1.0.149 | Type-safe serialization |
| **Configuration** | [Config](https://crates.io/crates/config) | 0.15.22 | Environment config parsing |
| **Environment** | [dotenv](https://crates.io/crates/dotenv) | 0.15.0 | `.env` file loading |
| **Date/Time** | [Chrono](https://github.com/chronotope/chrono) | 0.4.44 | Timezone-aware timestamps |
| **UUID** | [UUID](https://crates.io/crates/uuid) | 1.23.0 | Unique identifiers |

### Caching & Sessions
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Cache/Session** | [Redis](https://github.com/redis-rs/redis-rs) | 1.2.0 | In-memory data store |

### Email & Scheduling
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Email** | [Lettre](https://lettre.rs) | 0.11.21 | Email delivery |
| **Cron Jobs** | [Tokio Cron Scheduler](https://crates.io/crates/tokio-cron-scheduler) | 0.15.1 | Scheduled task execution |

### Logging & Monitoring
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Logging** | [Tracing](https://docs.rs/tracing/) + [Tracing Subscriber](https://docs.rs/tracing-subscriber/) | 0.1.44 + 0.3.23 | Structured logging |

### API Documentation
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **OpenAPI Spec** | [Utoipa](https://github.com/juhaku/utoipa) | 5.4.0 | Auto-generated API docs |
| **Swagger UI** | [Utoipa Swagger UI](https://github.com/juhaku/utoipa) | 9.0.2 | Interactive API explorer |

### HTTP Utilities
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **CORS** | [Tower HTTP](https://github.com/tokio-rs/tower-http) | 0.6.8 | Cross-origin resource sharing |
| **Tower** | [Tower](https://crates.io/crates/tower) | 0.5.3 | Middleware & service utilities |
| **Cookies** | [Axum Extra](https://docs.rs/axum-extra/) | 0.12.5 | Extended Axum utilities |
| **Cookie Handling** | [Cookie](https://crates.io/crates/cookie) | - | HTTP cookie operations |

### Error Handling & Validation
| Component | Technology | Version | Purpose |
|-----------|---------|---------|---------|
| **Error Types** | [Thiserror](https://crates.io/crates/thiserror) | 2.0.18 | Structured error definitions |
| **Validation** | [Validator](https://github.com/Keats/validator) | 0.20.0 | Field-level validation |
| **Error Context** | [Anyhow](https://crates.io/crates/anyhow) | 1.0.102 | Error context and chains |

### Build & Compilation
- **Rust Edition**: 2024
- **Default Binary**: `authentication`

---

## Project Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Client Applications                          │
│              (Web, Mobile, Desktop, CLI Tools)                   │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP/HTTPS
                         │
┌────────────────────────▼────────────────────────────────────────┐
│                    Axum Web Server                               │
│  ┌─────────────────────────────────────────────────────────────┤
│  │  Routing Layer (Handlers, Controllers)                      │
│  │  ├── Auth Handlers (register, login)                        │
│  │  ├── User Handlers (CRUD operations)                        │
│  │  ├── Blog Handlers (posts, comments)                        │
│  │  └── Health & Status Endpoints                              │
│  └─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┤
│  │  Middleware Stack                                            │
│  │  ├── Auth Middleware (JWT validation)                       │
│  │  ├── Logging Middleware (request/response tracking)         │
│  │  ├── CORS Middleware (cross-origin handling)                │
│  │  └── Error Handling Middleware                              │
│  └─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┤
│  │  Application Layer                                           │
│  │  ├── State Management (AppState)                            │
│  │  ├── Configuration (AppConfig)                              │
│  │  └── Error Handling (AppError enums)                        │
│  └─────────────────────────────────────────────────────────────┘
└────────────────┬──────────────┬───────────────┬─────────────────┘
                 │              │               │
                 │              │               │
        ┌────────▼──┐  ┌───────▼──┐  ┌────────▼──┐
        │ PostgreSQL│  │   Redis   │  │   Email   │
        │ (Primary  │  │ (Sessions,│  │  Service  │
        │  Datastore)  │  Cache)   │  │  (SMTP)   │
        └───────────┘  └───────────┘  └───────────┘
```

### Layered Architecture

```
┌────────────────────────────────────┐
│     REST API Endpoints              │
│  (OpenAPI Swagger UI Docs)          │
└────────────────────────────────────┘
                  │
┌────────────────▼────────────────────┐
│      Request Handlers Layer          │
│  (user, blogs, auth controllers)    │
└────────────────────────────────────┘
                  │
┌────────────────▼────────────────────┐
│     Business Logic Layer             │
│  (Validation, Authorization)        │
└────────────────────────────────────┘
                  │
┌────────────────▼────────────────────┐
│      Models & Schemas                │
│  (Data structures, validation)      │
└────────────────────────────────────┘
                  │
┌────────────────▼────────────────────┐
│     Data Access Layer (SQLx)         │
│  (Database queries, migrations)     │
└────────────────────────────────────┘
                  │
┌────────────────▼────────────────────┐
│     External Services                │
│  (PostgreSQL, Redis, Email)         │
└────────────────────────────────────┘
```

---

## Prerequisites

### System Requirements
- **OS**: Linux, macOS, or Windows (WSL2 recommended)
- **RAM**: 2GB minimum (4GB+ recommended)
- **Disk**: 2GB for dependencies and build artifacts

### Required Software

| Software | Version | Purpose | Installation |
|----------|---------|---------|--------------|
| **Rust** | 1.70.0+ | Language & toolchain | [rustup.rs](https://rustup.rs) |
| **PostgreSQL** | 14+ | Primary database | [postgresql.org](https://www.postgresql.org/download/) |
| **Diesel CLI** | 2.0.0+ | Migration management | `cargo install diesel_cli --no-default-features --features postgres` |
| **Redis** | 6.0+ | Cache & sessions | [redis.io](https://redis.io/download) (optional) |
| **psql** | 14+ | PostgreSQL client | Included with PostgreSQL |

### Optional Tools
- **Docker**: For containerized PostgreSQL/Redis
- **Git**: For version control
- **cargo-watch**: For auto-reload during development (`cargo install cargo-watch`)
- **sqlx-cli**: For advanced SQL debugging (`cargo install sqlx-cli`)

---

## Installation

### 1. Clone the Repository

```bash
git clone <repository-url>
cd authentication
```

### 2. Install Rust

Follow the official [Rust installation guide](https://www.rust-lang.org/tools/install):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

### 3. Install PostgreSQL

**macOS:**
```bash
brew install postgresql
brew services start postgresql
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:**
Download from [PostgreSQL Downloads](https://www.postgresql.org/download/windows/) and follow the installer.

### 4. Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres
```

Verify installation:
```bash
diesel --version
```

### 5. Install Redis (Optional but Recommended)

**macOS:**
```bash
brew install redis
brew services start redis
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install redis-server
sudo systemctl start redis-server
```

**Docker:**
```bash
docker run -d -p 6379:6379 redis:latest
```

---

## Setup & Configuration

### 1. Create Environment File

Copy the environment template and configure for your local setup:

```bash
cp .env.example .env  # If example exists, or create from scratch
```

Create `.env` in the project root:

```env
# Database Configuration
DATABASE_URL=postgres://postgres:password@localhost:5432/authentication_dev
DATABASE_POOL_SIZE=5

# Server Configuration
SERVER_ADDR=127.0.0.1
SERVER_PORT=8000

# JWT Configuration
JWT_SECRET=your_super_secret_key_here_change_in_production_min_32_chars
JWT_EXPIRATION_HOURS=24

# Redis Configuration (optional)
REDIS_URL=redis://127.0.0.1:6379/0

# Email Configuration (optional)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@example.com

# Logging
RUST_LOG=info,authentication=debug

# Environment
APP_ENV=development
```

**Important Security Notes:**
- Never commit `.env` to git
- Use strong, random `JWT_SECRET` (minimum 32 characters)
- Rotate credentials regularly in production
- Use environment-specific `.env.production`, `.env.staging`

### 2. Create Database

```bash
createdb authentication_dev
```

Or using `psql`:
```bash
psql -U postgres -c "CREATE DATABASE authentication_dev;"
```

### 3. Run Database Migrations

```bash
cargo run --bin migrate
```

This creates the database schema from migration files in `migrations/`.

### 4. Verify Setup

Run a quick build to ensure all dependencies resolve:

```bash
cargo build
```

### 5. (Optional) Sync Diesel Schema Cache

If you modify migrations, refresh the schema cache:

```bash
bash scripts/diesel-schema.sh sync
```

---

## Quick Start

### 1. Start the Server

```bash
cargo run
```

Server will start at `http://127.0.0.1:8000`

Output:
```
2026-04-10T10:30:45.123456Z INFO  authentication: Server listening on 0.0.0.0:8000
2026-04-10T10:30:45.234567Z INFO  authentication: Connected to PostgreSQL
2026-04-10T10:30:45.345678Z INFO  authentication: Redis cache enabled
```

### 2. Check API Documentation

Open in browser:
- **Swagger UI**: [http://127.0.0.1:8000/swagger-ui/](http://127.0.0.1:8000/swagger-ui/)
- **OpenAPI JSON**: [http://127.0.0.1:8000/openapi.json](http://127.0.0.1:8000/openapi.json)

### 3. Register a User

```bash
curl -X POST http://127.0.0.1:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!",
    "first_name": "John",
    "last_name": "Doe"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "created_at": "2026-04-10T10:30:45Z"
}
```

### 4. Login

```bash
curl -X POST http://127.0.0.1:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!"
  }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### 5. Create a Blog Post

```bash
curl -X POST http://127.0.0.1:8000/api/blogs/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -d '{
    "title": "My First Post",
    "content": "This is the content of my first blog post.",
    "tags": ["rust", "webdev"]
  }'
```

---

## API Documentation

### Authentication Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|----------------|
| `POST` | `/api/auth/register` | User registration | No |
| `POST` | `/api/auth/login` | User login (returns JWT token) | No |
| `POST` | `/api/auth/refresh` | Refresh JWT token | Yes |
| `POST` | `/api/auth/logout` | Invalidate token | Yes |

### User Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|----------------|
| `GET` | `/api/users` | List all users (paginated) | Yes |
| `GET` | `/api/users/:id` | Get user by ID | Yes |
| `PUT` | `/api/users/:id` | Update user profile | Yes (owner or admin) |
| `DELETE` | `/api/users/:id` | Delete user account | Yes (owner or admin) |

### Blog Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|----------------|
| `POST` | `/api/blogs/posts` | Create blog post | Yes |
| `GET` | `/api/blogs/posts` | List blog posts (paginated) | No |
| `GET` | `/api/blogs/posts/:id` | Get specific post | No |
| `PUT` | `/api/blogs/posts/:id` | Update post | Yes (author) |
| `DELETE` | `/api/blogs/posts/:id` | Delete post | Yes (author) |

### Comment Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|----------------|
| `POST` | `/api/blogs/posts/:postId/comments` | Create comment | Yes |
| `GET` | `/api/blogs/posts/:postId/comments` | List comments | No |
| `PUT` | `/api/blogs/comments/:id` | Update comment | Yes (author) |
| `DELETE` | `/api/blogs/comments/:id` | Delete comment | Yes (author) |

### Health Check

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|----------------|
| `GET` | `/health` | Service health status | No |
| `GET` | `/ready` | Readiness probe | No |

### Response Format

All API responses follow a consistent JSON structure:

**Success Response (2xx):**
```json
{
  "data": { /* actual response payload */ },
  "status": "success",
  "timestamp": "2026-04-10T10:30:45Z"
}
```

**Error Response (4xx/5xx):**
```json
{
  "error": "Error message here",
  "status": "error",
  "code": "ERROR_CODE",
  "timestamp": "2026-04-10T10:30:45Z"
}
```

### Authentication Header

Include JWT token in all protected requests:

```bash
Authorization: Bearer <your_jwt_token>
```

---

## Project Commands

### Build & Compilation

```bash
# Debug build (faster, includes debug symbols)
cargo build

# Release build (optimized for production)
cargo build --release

# Check compilation without building
cargo check

# Watch mode (auto-rebuild on file changes)
cargo watch -x build
```

### Running

```bash
# Run the main application
cargo run

# Run with release optimizations
cargo run --release

# Run with arguments
cargo run -- --port 9000

# Run specific binary
cargo run --bin migrate
cargo run --bin shell
cargo run --bin dbshell
```

### Testing & Quality

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_user_registration

# Check code formatting
cargo fmt --check

# Auto-format code
cargo fmt

# Lint with Clippy
cargo clippy

# Full lint report
cargo clippy -- -W clippy::all
```

### Database Management

```bash
# Apply pending migrations
cargo run --bin migrate

# Show migration status
cargo run --bin showmigrations

# Create new migration
cargo run --bin makemigrations -- create_table_users

# Get interactive SQL shell
cargo run --bin shell

# Open native PostgreSQL shell
cargo run --bin dbshell

# Refresh Diesel schema cache
bash scripts/diesel-schema.sh sync
```

### Scaffolding & Development

```bash
# Generate new app module (scaffolding)
cargo run --bin startapp -- my_app

# This creates: src/my_app/
#   ├── mod.rs
#   ├── models.rs
#   ├── handlers.rs
#   ├── schemas.rs
```

### Dependency Management

```bash
# Add a dependency
cargo add serde_yaml

# Update all dependencies
cargo update

# Outdated check
cargo outdated

# Generate dependency tree
cargo tree

# Security audit
cargo audit
```

### Documentation

```bash
# Generate and open API docs
cargo doc --open

# Generate without opening
cargo doc

# Document private items
cargo doc --document-private-items
```

---

## Database Management

### Migration Files

Migrations are located in `migrations/` directory:

```
migrations/
├── 00000000000000_diesel_initial_setup/
├── 2026-04-07-081504-0000_init/
├── 20260407101804_initial/
├── 20260407111624_auto/
├── 20260408074619_auto/
└── 20260408074647_auto/
```

Each migration contains:
- `up.sql` — Schema changes to apply
- `down.sql` — Rollback instructions

### Creating a New Migration

```bash
diesel migration generate add_user_roles

# Creates: migrations/TIMESTAMP_add_user_roles/{up.sql,down.sql}
# Edit the SQL files, then run:
cargo run --bin migrate
```

### Viewing Migration Status

```bash
cargo run --bin showmigrations
```

Output:
```
Migrations:
 [X] 00000000000000_diesel_initial_setup
 [X] 2026-04-07-081504-0000_init
 [X] 20260407101804_initial
 [X] 20260407111624_auto
 [X] 20260408074619_auto
 [X] 20260408074647_auto
```

### Rolling Back

To rollback the last migration:
```bash
diesel migration redo --database-url "$DATABASE_URL"
```

### Direct Database Access

Interactive SQL shell:
```bash
cargo run --bin shell
```

Or use native `psql`:
```bash
cargo run --bin dbshell
```

---

## Development Workflow

### Local Development Setup

1. **Start Development Environment:**
   ```bash
   # Terminal 1: Start PostgreSQL
   brew services start postgresql
   
   # Terminal 2: Start Redis
   redis-server
   
   # Terminal 3: Run application with auto-reload
   cargo watch -x "run"
   ```

2. **Edit Files & Test:**
   - Make code changes
   - Cargo will auto-rebuild
   - Test via Swagger UI or curl

3. **Database Changes:**
   ```bash
   # Create new migration
   diesel migration generate add_new_field
   
   # Edit migrations/TIMESTAMP_add_new_field/up.sql
   # Then apply
   cargo run --bin migrate
   ```

### Code Organization

```
src/
├── main.rs              # Application entry point, routing, OpenAPI setup
├── config.rs            # Configuration loading & validation
├── db.rs                # Database connection pool setup
├── error.rs             # Error types & conversion
├── response.rs          # Response structures & serialization
├── state.rs             # Application state management
│
├── models/              # Shared domain models
│   └── mod.rs
│
├── middleware/          # HTTP middleware
│   ├── auth.rs          # JWT token validation
│   ├── logging.rs       # Request/response logging
│   └── mod.rs
│
├── user/                # User module (auth & profile)
│   ├── mod.rs           # Module exports
│   ├── models.rs        # User data structures
│   ├── schemas.rs       # Request/response schemas
│   └── handlers.rs      # Endpoint handlers
│
└── blogs/               # Blog module (posts & comments)
    ├── mod.rs
    ├── models.rs        # BlogPost, Comment models
    ├── schemas.rs       # API request/response schemas
    └── handlers.rs      # Blog handlers
```

### Naming Conventions

- **Files**: `module_name.rs` (snake_case)
- **Modules**: `mod.rs` for public exports
- **Functions**: `handle_user_registration` (snake_case)
- **Types**: `UserRegistrationRequest` (PascalCase)
- **Constants**: `DATABASE_TIMEOUT` (UPPER_SNAKE_CASE)
- **Database tables**: `users`, `blog_posts`, `comments` (snake_case, plural)

### Testing Strategy

```bash
# Unit tests (same file)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        // ...
    }
}

# Integration tests (tests/ directory)
# tests/auth_integration.rs
# tests/blog_integration.rs

# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## Error Handling

The application uses a unified error handling system via `AppError`:

### Error Types

```rust
pub enum AppError {
    NotFound(String),       // 404
    BadRequest(String),     // 400
    Unauthorized(String),   // 401
    Forbidden(String),      // 403
    Conflict(String),       // 409
    Internal(String),       // 500
    Database(sqlx::Error),  // 500
}
```

### Error Responses

Errors are automatically converted to JSON responses:

```json
{
  "error": "User not found",
  "status": "error",
  "code": 404,
  "timestamp": "2026-04-10T10:30:45Z"
}
```

### Middleware Error Handling

- **400 Bad Request**: Invalid JSON, missing required fields, validation errors
- **401 Unauthorized**: Missing or invalid JWT token
- **403 Forbidden**: Insufficient permissions for the resource
- **404 Not Found**: Resource doesn't exist
- **409 Conflict**: Duplicate email, conflicting state
- **500 Internal Error**: Unexpected server errors, database failures

---

## Project Structure

### Directory Layout

```
authentication/
├── Cargo.toml                          # Project manifest, dependencies
├── Cargo.lock                          # Locked dependency versions
├── README.md                           # This file
├── readme.md                           # Legacy readme
│
├── .env                                # Local environment variables (gitignored)
├── .env.example                        # Environment template
├── .gitignore                          # Git ignore rules
├── diesel.toml                         # Diesel ORM configuration
│
├── migrations/                         # Database schema migrations
│   ├── 00000000000000_diesel_initial_setup/
│   ├── 20260407101804_initial/
│   ├── 20260407111624_auto/
│   └── 20260408074647_auto/
│
├── scripts/                            # Utility scripts
│   ├── diesel-schema.sh                # Diesel schema cache management
│   └── ...
│
├── src/                                # Application source code
│   ├── main.rs                         # Application entry point
│   ├── config.rs                       # Configuration management
│   ├── db.rs                           # Database setup
│   ├── error.rs                        # Error types
│   ├── response.rs                     # Response handlers
│   ├── state.rs                        # Application state
│   ├── models/                         # Shared models
│   ├── middleware/                     # HTTP middleware
│   │   ├── auth.rs
│   │   ├── logging.rs
│   │   └── mod.rs
│   ├── user/                           # User authentication module
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── schemas.rs
│   │   └── mod.rs
│   ├── blogs/                          # Blog content module
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── schemas.rs
│   │   └── mod.rs
│   └── bin/                            # CLI binaries
│       ├── dbshell.rs                  # PostgreSQL shell wrapper
│       ├── migrate.rs                  # Run database migrations
│       ├── makemigrations.rs           # Create new migrations
│       ├── showmigrations.rs           # Show migration status
│       ├── shell.rs                    # Interactive SQL shell
│       └── startapp.rs                 # App scaffolding generator
│
├── docs/                               # Additional documentation
│   ├── README.md                       # This file
│   ├── setup.md                        # Setup instructions
│   ├── installation.md                 # Installation guide
│   └── development.md                  # Development workflow
│
└── target/                             # Build artifacts (auto-generated)
    ├── debug/                          # Debug builds
    └── release/                        # Release builds
```

### Key Files Explained

| File | Purpose |
|------|---------|
| `Cargo.toml` | Project metadata, dependencies, build configuration |
| `src/main.rs` | Application entry point, Axum setup, route definitions |
| `src/config.rs` | Environment variable loading, AppConfig struct |
| `src/db.rs` | SQLx connection pool initialization |
| `src/error.rs` | Unified error types and HTTP conversion |
| `src/state.rs` | Shared application state (db pool, cache, config) |
| `src/middleware/auth.rs` | JWT token extraction and validation |
| `src/user/handlers.rs` | Register, login, user management endpoints |
| `src/blogs/handlers.rs` | Blog CRUD endpoints |
| `migrations/*.sql` | Database schema and structure |
| `.env` | Local environment configuration |
| `diesel.toml` | Diesel configuration for migrations & schema generation |

---

## Contributing

### Development Guidelines

1. **Code Style**
   - Follow Rust naming conventions (snake_case for variables/functions)
   - Maximum line length: 100 characters
   - Use `cargo fmt` before committing
   - Run `cargo clippy` and fix warnings

2. **Commits**
   - Use descriptive commit messages
   - Format: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`
   - Example: `feat: add user email verification endpoint`

3. **Testing**
   - Write tests for new functionality
   - Ensure all tests pass: `cargo test`
   - Test error cases and edge conditions

4. **Documentation**
   - Add doc comments to public functions
   - Update README for significant changes
   - Keep error messages descriptive

### Pull Request Process

1. Create feature branch: `git checkout -b feature/amazing-feature`
2. Make changes and add tests
3. Run: `cargo fmt`, `cargo clippy`, `cargo test`
4. Commit with descriptive messages
5. Push to repository
6. Create pull request with description

### Reporting Issues

Include:
- Rust version (`rustc --version`)
- PostgreSQL version
- Error message and backtrace
- Steps to reproduce
- Expected vs actual behavior

---

## Performance Considerations

### Optimization Tips

1. **Database**
   - Use connection pooling (configured in `db.rs`)
   - Enable query caching with Redis
   - Add database indexes on frequently queried columns

2. **API**
   - Implement pagination for list endpoints
   - Use gzip compression for responses
   - Cache static content

3. **Async Runtime**
   - Leverage Tokio's async capabilities
   - Avoid blocking operations in handlers
   - Use `tokio::spawn` for background tasks

4. **Build**
   - Use `cargo build --release` for production
   - Link-time optimization reduces binary size
   - Strip debug symbols for deployment

### Monitoring

Enable structured logging:
```bash
RUST_LOG=info,authentication=debug cargo run
```

Monitor with ELK stack or similar for production deployment.

---

## Deployment

### Production Checklist

- [ ] Set strong `JWT_SECRET` (min 32 characters)
- [ ] Use HTTPS/TLS certificates
- [ ] Enable CORS with specific origins
- [ ] Configure production database backups
- [ ] Set up monitoring and alerting
- [ ] Enable structured logging
- [ ] Review security headers
- [ ] Test database migrations
- [ ] Document deployment process
- [ ] Set up rollback procedures

### Docker Deployment

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/authentication /usr/local/bin/
EXPOSE 8000
CMD ["authentication"]
```

Build and run:
```bash
docker build -t rustauth:latest .
docker run -p 8000:8000 -e DATABASE_URL=... rustauth:latest
```

---

## Troubleshooting

### Common Issues

**Issue: `DATABASE_URL` not found**
```
Error: `DATABASE_URL` environment variable not set
```
**Solution:** Create `.env` file with `DATABASE_URL` set.

**Issue: Connection refused to PostgreSQL**
```
Error: failed to connect to postgres://localhost:5432
```
**Solution:** Ensure PostgreSQL is running:
```bash
brew services start postgresql  # macOS
sudo systemctl start postgresql  # Linux
```

**Issue: Migration fails**
```
Error: Migration failed: column "field" does not exist
```
**Solution:** Check migration order and ensure all `.up.sql` files are valid SQL.

**Issue: JWT token invalid**
```json
{"error": "Invalid token"}
```
**Solution:** Ensure `JWT_SECRET` in `.env` matches the secret used to create the token.

**Issue: Port already in use**
```
Error: Address already in use (os error 48)
```
**Solution:** Change `SERVER_PORT` in `.env` or kill process on port 8000:
```bash
lsof -ti:8000 | xargs kill -9
```

---

## Related Documentation

- [setup.md](./setup.md) — Detailed environment setup
- [installation.md](./installation.md) — Installation prerequisites
- [development.md](./development.md) — Development workflow

---

## License

This project is licensed under the MIT License. See LICENSE file for details.

---

## Support & Resources

- **Rust Documentation**: https://doc.rust-lang.org/
- **Axum Framework**: https://github.com/tokio-rs/axum
- **Tokio Runtime**: https://tokio.rs
- **SQLx Documentation**: https://sqlx.rs
- **PostgreSQL Docs**: https://www.postgresql.org/docs/

---

## FAQ

**Q: Can I use this in production?**  
A: Yes, it's designed as a production-ready scaffold. Ensure you've completed the security checklist.

**Q: How do I add a new module?**  
A: Use `cargo run --bin startapp -- module_name` or manually create the module structure.

**Q: Is Redis required?**  
A: No, it's optional. Remove Redis dependencies from `Cargo.toml` if not needed.

**Q: How do I implement role-based access control?**  
A: Add a `role` column to users table, check role in handlers, create permission middleware.

**Q: Can I modify the database schema?**  
A: Yes, create a new migration, edit `up.sql` and `down.sql`, then run `cargo run --bin migrate`.

---

**Last Updated**: April 10, 2026  
**Maintained By**: Development Team  
**Status**: Active Development
