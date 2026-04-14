# RustAuth API - Authentication & Blog Service

A production-ready **Rust authentication service** with user management, blog functionality, and comprehensive API documentation. Built with modern async web frameworks and industry best practices.

## 📋 Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Technology Stack](#technology-stack)
- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Core Modules](#core-modules)
- [API Endpoints](#api-endpoints)
- [Database Models](#database-models)
- [Configuration](#configuration)
- [CLI Commands](#cli-commands)
- [Development](#development)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

**RustAuth** is a fully-featured authentication and content management API demonstrating modern backend development practices in Rust. It provides enterprise-grade security, scalability, and developer experience.

### Core Capabilities

- **JWT-based authentication** with secure password hashing (Argon2)
- **User management** with CRUD operations and profiles
- **Blog platform** with posts and comments
- **Email support** for notifications and verification
- **Redis integration** for caching and sessions
- **Comprehensive API documentation** via Swagger UI
- **CLI tooling** for database management and scaffolding
- **Database migrations** with automatic change detection
- **RBAC** (Role-Based Access Control) ready
- **Admin panel** with AdminX framework

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

### Complete Setup Guide for New Users

If you're new to Rust or this project, follow this detailed step-by-step guide. Each step explains what you're doing and why.

#### Step 1: Check Your System

Before starting, verify your operating system and determine which installation method to use:

**Windows Users:**
- Use Windows Subsystem for Linux 2 (WSL2) or install native tools
- We recommend WSL2 for better compatibility

**macOS Users:**
- Make sure you have Xcode Command Line Tools installed
  ```bash
  xcode-select --install
  ```

**Linux Users (Ubuntu/Debian):**
- Ensure you have build essentials installed
  ```bash
  sudo apt-get update
  sudo apt-get install build-essential
  ```

#### Step 2: Install Rust (The Programming Language)

Rust is the core language needed to compile and run this project.

**All Operating Systems:**
```bash
# Download and run the Rust installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts (usually just press Enter to accept defaults)

# Load Rust environment variables
source $HOME/.cargo/env
```

**Verify Installation:**
```bash
rustc --version    # Should print: rustc X.Y.Z (...)
cargo --version    # Should print: cargo X.Y.Z (...)
```

If you see version numbers, you're good to go! If not, restart your terminal and try again.

#### Step 3: Install PostgreSQL (The Database)

PostgreSQL is where we store user accounts, blog posts, and other data.

**macOS (using Homebrew):**
```bash
# Install Homebrew if you don't have it
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install PostgreSQL
brew install postgresql@15

# Start PostgreSQL service
brew services start postgresql@15

# Verify installation
psql --version    # Should print: psql (PostgreSQL) X.Y.Z
```

**Linux (Ubuntu/Debian):**
```bash
# Update package list
sudo apt-get update

# Install PostgreSQL
sudo apt-get install postgresql postgresql-contrib

# Start PostgreSQL
sudo systemctl start postgresql

# Verify installation
psql --version
```

**Windows (WSL2):**
```bash
# Inside your WSL2 terminal
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib

# Start PostgreSQL
sudo systemctl start postgresql

# Verify installation
psql --version
```

**Docker Alternative (All Platforms):**
If you have Docker installed, you can run PostgreSQL in a container:
```bash
docker run -d \
  --name postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgres:15-alpine

# Verify it's running
docker ps
```

#### Step 4: Install Diesel CLI (Database Migration Tool)

Diesel CLI helps manage database schema changes safely.

```bash
# Install Diesel CLI for PostgreSQL
cargo install diesel_cli --no-default-features --features postgres

# This takes a few minutes - it's compiling from source

# Verify installation
diesel --version    # Should print: diesel X.Y.Z
```

#### Step 5: Install Redis (Optional but Recommended)

Redis is used for caching and session management. It's optional but recommended for production-like testing.

**macOS:**
```bash
brew install redis
brew services start redis

# Verify installation
redis-cli ping    # Should print: PONG
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install redis-server

sudo systemctl start redis-server

# Verify installation
redis-cli ping
```

**Windows (WSL2):**
```bash
sudo apt-get install redis-server
sudo systemctl start redis-server
redis-cli ping
```

**Docker Alternative:**
```bash
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:7-alpine

# Verify it's running
docker exec redis redis-cli ping    # Should print: PONG
```

#### Step 6: Clone the Repository

Get the project code from version control:

```bash
# Clone the repository
git clone <repository-url>
cd authentication

# Verify you're in the right directory
pwd    # Should print path ending with 'authentication'
ls     # Should show: Cargo.toml, README.md, migrations/, src/
```

#### Step 7: Create Environment Configuration

The `.env` file stores sensitive configuration that varies between environments:

```bash
# Create the environment file
cat > .env << 'EOF'
# Database Configuration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/authentication_dev
DATABASE_POOL_SIZE=5

# Server Configuration
SERVER_ADDR=127.0.0.1
SERVER_PORT=8000

# JWT Configuration (use a strong 32+ character random string)
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
EOF
```

**Important:** The `.env` file contains sensitive information and should never be committed to version control. Check your `.gitignore` file to ensure it contains `.env`.

#### Step 8: Create the Database

Create a fresh PostgreSQL database for development:

```bash
# Simple method using createdb
createdb authentication_dev

# Verify it was created
psql -l | grep authentication_dev
```

If `createdb` is not found, use this alternative:
```bash
psql -U postgres -c "CREATE DATABASE authentication_dev;"
```

#### Step 9: Run Database Migrations

Migrations set up the database schema (tables, columns, relationships):

```bash
# Apply all pending migrations
cargo run --bin migrate

# Verify migrations ran successfully
# You should see messages like: "Running migration 20260407101804"

# Check migration status
cargo run --bin showmigrations
```

#### Step 10: Download Dependencies and Build

Compile the project and download all required libraries:

```bash
# Build the project (this takes several minutes on first run)
cargo build

# If build succeeds, you'll see: "Finished debug [unoptimized + debuginfo]"

# If there are errors, see the Troubleshooting section below
```

#### Step 11: Run the Application

Start the web server:

```bash
# Start the application
cargo run

# Watch for output like:
# INFO authentication: Server listening on 0.0.0.0:8000
# INFO authentication: Connected to PostgreSQL
```

The server is now running! Keep this terminal open.

#### Step 12: Verify Everything Works

In a new terminal window, test the application:

```bash
# Check if the server is responding
curl http://127.0.0.1:8000/health

# Expected output: JSON response with status

# Open API documentation in your browser
open http://127.0.0.1:8000/swagger-ui/
# or on Linux/WSL:
# xdg-open http://127.0.0.1:8000/swagger-ui/
```

🎉 **Congratulations!** Your project is now set up and running!

---

### TL;DR (Quick Setup)

For experienced developers who just want to get started quickly:

```bash
# 1. Ensure Rust, PostgreSQL, Diesel CLI are installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. For macOS
brew install postgresql@15 redis
brew services start postgresql@15 redis
cargo install diesel_cli --no-default-features --features postgres

# 3. For Linux (Ubuntu/Debian)
sudo apt-get install postgresql postgresql-contrib redis-server
sudo systemctl start postgresql redis-server
cargo install diesel_cli --no-default-features --features postgres

# 4. Clone and setup
git clone <repository-url>
cd authentication

# 5. Create .env
cp .env.example .env    # OR create manually with your DATABASE_URL and JWT_SECRET

# 6. Setup database
createdb authentication_dev
cargo run --bin migrate

# 7. Build and run
cargo build
cargo run

# 8. Open your browser
open http://127.0.0.1:8000/swagger-ui/
```

---

## Prerequisites

### System Requirements
- **OS**: Linux (Ubuntu 20.04+), macOS (10.15+), or Windows (WSL2)
- **RAM**: 2GB minimum for compilation (4GB+ recommended)
- **Disk**: 5GB available space for dependencies and build artifacts
- **Internet**: Required for downloading dependencies

### Required Software

| Software | Version | Purpose | Status |
|----------|---------|---------|--------|
| **Rust** | 1.70.0+ | Language & compiler | Required |
| **PostgreSQL** | 14+ | Primary database | Required |
| **Diesel CLI** | 2.0.0+ | Database migrations | Required |
| **Git** | 2.25+ | Version control | Required |
| **Redis** | 6.0+ | Cache & sessions | Optional* |

*Optional but recommended for production-like testing

### Optional Tools
- **Docker**: Run PostgreSQL and Redis in containers
- **cargo-watch**: Auto-rebuild on file changes (`cargo install cargo-watch`)
- **sqlx-cli**: Advanced database debugging
- **HTTPie** or **Insomnia**: API testing (alternative to curl)

---

## Setup & Configuration

This section covers post-installation configuration for the application.

### 1. Create Environment File

The `.env` file contains configuration specific to your development environment. Create it in the project root:

```bash
cp .env.example .env    # If an example file exists
```

Or manually create `.env` with:

```env
# Database Configuration
# Modify the password if you set a different PostgreSQL password during installation
DATABASE_URL=postgres://postgres:postgres@localhost:5432/authentication_dev
DATABASE_POOL_SIZE=5

# Server Configuration
# 127.0.0.1 = localhost (only accessible from your machine)
# 0.0.0.0 = all network interfaces (accessible from other machines)
SERVER_ADDR=127.0.0.1
SERVER_PORT=8000

# JWT Configuration
# Generate a strong random secret: `openssl rand -base64 32`
# Must be at least 32 characters for production
JWT_SECRET=your_super_secret_key_here_change_in_production_min_32_chars
JWT_EXPIRATION_HOURS=24

# Redis Configuration (optional, leave as-is if using default Redis setup)
REDIS_URL=redis://127.0.0.1:6379/0

# Email Configuration (optional, for sending verification emails)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-specific-password
SMTP_FROM_EMAIL=noreply@example.com

# Logging Level
# Options: trace, debug, info, warn, error
# Use 'debug' for development, 'info' for production
RUST_LOG=info,authentication=debug

# Environment Type
# Options: development, staging, production
APP_ENV=development
```

**Security Best Practices:**
- ⚠️ Never commit `.env` to git (check `.gitignore` includes it)
- Use strong, random `JWT_SECRET` (minimum 32 characters)
- Generate secure secret: `openssl rand -base64 32`
- Rotate credentials regularly in production
- Use environment-specific files: `.env.production`, `.env.staging`
- Update SMTP credentials for email functionality

### 2. Create Database

Create a PostgreSQL database for development:

**Method 1 (Recommended):**
```bash
createdb authentication_dev
```

**Method 2 (Using psql):**
```bash
psql -U postgres -c "CREATE DATABASE authentication_dev;"
```

**Method 3 (Interactive):**
```bash
psql -U postgres
# At the psql prompt type:
# postgres=# CREATE DATABASE authentication_dev;
# postgres=# \q
```

Verify the database was created:
```bash
psql -l | grep authentication_dev
```

### 3. Run Database Migrations

Apply the database schema using migrations:

```bash
# Run all pending migrations
cargo run --bin migrate

# Expected output shows each migration being applied
```

Check migration status:
```bash
cargo run --bin showmigrations
```

### 4. Verify Setup

Test that everything is configured correctly:

```bash
# Validate Rust code compiles
cargo check

# Expected: "Finished `dev` profile" with no errors
```

### 5. Generate and Review Documentation

Generate Rust code documentation:

```bash
# Generate and open HTML documentation
cargo doc --open

# Creates documentation for all crates and dependencies
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

# Generate migrations from app model changes
cargo makemigrations
```

### Scaffolding & Development

```bash
# Generate new app module (scaffolding)
cargo run --bin startapp -- my_app

# This creates: src/apps/my_app/
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

## Troubleshooting

This section covers common issues and their solutions.

### Build Issues

#### Error: "Could not compile `authentication`"

**Problem:** The project fails to compile.

**Solutions:**
1. Ensure Rust is up to date:
   ```bash
   rustup update
   cargo clean
   cargo build
   ```

2. Check for conflicting dependencies:
   ```bash
   cargo tree
   cargo update
   ```

3. For Windows users, ensure you have Visual Studio Build Tools installed

---

#### Error: "error: linker `cc` not found"

**Problem:** Missing C compiler for linking native code.

**Solutions:**
- **macOS:** Install Xcode Command Line Tools
  ```bash
  xcode-select --install
  ```

- **Linux (Ubuntu):** Install build essentials
  ```bash
  sudo apt-get install build-essential
  ```

- **Windows:** Download Visual Studio Build Tools from official website

---

### Database Issues

#### Error: "database does not exist"

**Problem:** PostgreSQL database hasn't been created yet.

**Solutions:**
```bash
# Create the database
createdb authentication_dev

# Or explicitly
psql -U postgres -c "CREATE DATABASE authentication_dev;"

# Verify creation
psql -l | grep authentication_dev
```

---

#### Error: "password authentication failed for user"

**Problem:** PostgreSQL credentials in `.env` are incorrect.

**Solutions:**
1. Verify PostgreSQL is running:
   ```bash
   # macOS
   brew services list | grep postgres
   
   # Linux
   sudo systemctl status postgresql
   ```

2. Check your `.env` DATABASE_URL:
   ```bash
   # Format: postgres://username:password@host:port/database
   # Common defaults: postgres://postgres:postgres@localhost:5432/authentication_dev
   ```

3. Reset PostgreSQL password:
   ```bash
   psql -U postgres
   postgres=# ALTER USER postgres WITH PASSWORD 'newpassword';
   postgres=# \q
   ```

---

#### Error: "Migration failed" or "could not find migration"

**Problem:** Database migrations didn't apply correctly.

**Solutions:**
```bash
# Check migration status
cargo run --bin showmigrations

# Reset migrations (WARNING: deletes all data)
psql -U postgres -d authentication_dev -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

# Re-run migrations
cargo run --bin migrate
```

---

### Runtime Issues

#### Error: "could not connect to database" or "connection refused"

**Problem:** PostgreSQL is not running or network connection failed.

**Solutions:**
```bash
# Start PostgreSQL service
# macOS
brew services start postgresql@15

# Linux
sudo systemctl start postgresql

# Verify PostgreSQL is listening
psql -U postgres -c "SELECT 1"
```

---

#### Error: Server won't start / "Port 8000 already in use"

**Problem:** Another process is using the server port.

**Solutions:**
```bash
# Find process using port 8000
lsof -i :8000          # macOS/Linux
netstat -ano | grep 8000  # Windows

# Kill the process (replace PID with actual process ID)
kill -9 <PID>     # macOS/Linux
taskkill /PID <PID> /F  # Windows

# Or change SERVER_PORT in .env to a different port (e.g., 8001)
```

---

#### Error: "Redis connection refused"

**Problem:** Redis is not running (it's optional but recommended).

**Solutions:**
```bash
# Start Redis
# macOS
brew services start redis

# Linux
sudo systemctl start redis-server

# Verify Redis is running
redis-cli ping    # Should print: PONG

# To disable Redis (optional):
# Comment out or remove REDIS_URL from .env
```

---

### Environment Issues

#### Error: "Variable not found" or configuration errors

**Problem:** `.env` file isn't being loaded.

**Solutions:**
1. Verify `.env` exists in project root:
   ```bash
   ls -la .env
   ```

2. Check `.env` file format (no spaces around `=`):
   ```bash
   # Correct
   DATABASE_URL=postgres://localhost/db
   
   # Incorrect
   DATABASE_URL = postgres://localhost/db
   ```

3. Reload environment variables:
   ```bash
   # Stop cargo run (Ctrl+C)
   source .env
   cargo run
   ```

---

### Performance Issues

#### Slow startup time or build taking too long

**Problem:** First build or compilation is very slow.

**Solutions:**
```bash
# This is normal for Rust! First build can take 5-10 minutes
# Subsequent builds are faster

# For development, use debug builds (faster):
cargo build         # This is already the default

# Only use release if you need optimization:
cargo build --release  # Much slower to build, but runs faster
```

---

#### Slow API response times

**Problem:** Requests are taking longer than expected.

**Solutions:**
1. Enable query logging to see slow queries:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. Check PostgreSQL is running efficiently:
   ```bash
   psql -U postgres -d authentication_dev -c "SELECT count(*) FROM users;"
   ```

3. Ensure Redis is running (improves caching):
   ```bash
   redis-cli ping
   ```

---

### Testing Issues

#### Tests fail or don't run

**Problem:** Can't run tests successfully.

**Solutions:**
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in single-threaded mode (if tests interfere)
cargo test -- --test-threads=1
```

---

### API Issues

#### Error: "Invalid JWT token" or "Unauthorized"

**Problem:** Authentication token is invalid or expired.

**Solutions:**
1. Get a new token by logging in:
   ```bash
   curl -X POST http://127.0.0.1:8000/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{
       "email": "user@example.com",
       "password": "SecurePass123!"
     }'
   ```

2. Include the token in subsequent requests:
   ```bash
   curl -H "Authorization: Bearer YOUR_TOKEN_HERE" \
     http://127.0.0.1:8000/api/users
   ```

---

#### Error: "CORS policy: No 'Access-Control-Allow-Origin' header"

**Problem:** Frontend cannot make requests due to CORS restrictions.

**Solutions:**
1. Ensure CORS middleware is enabled in `src/main.rs`

2. Configure allowed origins in `.env` or code:
   ```bash
   # Add your frontend URL to allowed CORS origins
   ```

3. For development, temporarily allow all origins (not for production):
   ```rust
   // In src/main.rs
   .layer(cors_layer.allow_origin("*".parse().unwrap()))
   ```

---

### Getting Help

If you encounter an issue not listed here:

1. **Check logs**: Run with debug logging
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Search issues**: Check GitHub issues for similar problems

3. **Documentation**: Review [Axum](https://docs.rs/axum/) or [SQLx](https://docs.rs/sqlx/) docs

4. **Community**: Ask the Rust community at [r/rust](https://reddit.com/r/rust) or [Rust Discord](https://discord.gg/rust-lang)

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
│
├── .env                                # Local environment variables (gitignored)
├── .gitignore                          # Git ignore rules
│
├── migrations/                         # Database schema migrations
│   ├── 00000000000000_diesel_initial_setup/
│   ├── 20260407101804_initial/
│   ├── 20260407111624_auto/
│   └── 20260408074647_auto/
│
├── src/                                # Application source code
│   ├── main.rs                         # Startup, middleware, and router mounting
│   ├── config.rs                       # Configuration management
│   ├── db.rs                           # Database setup
│   ├── error.rs                        # Error types
│   ├── response.rs                     # Response handlers
│   ├── state.rs                        # Application state
│   ├── apps/                           # Self-contained feature apps
│   │   ├── mod.rs                      # Central app registry and OpenAPI wiring
│   │   ├── user/
│   │   │   ├── handlers.rs
│   │   │   ├── models.rs
│   │   │   ├── schemas.rs
│   │   │   └── mod.rs
│   │   └── blogs/
│   │       ├── handlers.rs
│   │       ├── models.rs
│   │       ├── schemas.rs
│   │       └── mod.rs
│   └── bin/                            # CLI binaries
│       ├── dbshell.rs                  # PostgreSQL shell wrapper
│       ├── migrate.rs                  # Run database migrations
│       ├── makemigrations.rs           # Create new migrations
│       ├── showmigrations.rs           # Show migration status
│       ├── shell.rs                    # Interactive SQL shell
│       └── startapp.rs                 # App scaffolding generator
│
├── setup.md                            # Setup instructions
├── installation.md                     # Installation guide
├── development.md                      # Development workflow
│
└── target/                             # Build artifacts (auto-generated)
    ├── debug/                          # Debug builds
    └── release/                        # Release builds
```

### Key Files Explained

| File | Purpose |
|------|---------|
| `Cargo.toml` | Project metadata, dependencies, build configuration |
| `src/main.rs` | Application startup, middleware, and top-level router mounting |
| `src/apps/mod.rs` | Central app registry, route aggregation, and OpenAPI wiring |
| `src/config.rs` | Environment variable loading, AppConfig struct |
| `src/db.rs` | SQLx connection pool initialization |
| `src/error.rs` | Unified error types and HTTP conversion |
| `src/state.rs` | Shared application state (db pool, cache, config) |
| `src/apps/user/handlers.rs` | Register, login, user management endpoints |
| `src/apps/blogs/handlers.rs` | Blog CRUD endpoints |
| `migrations/*.sql` | Database schema and structure |
| `.env` | Local environment configuration |

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

- [setup.md](./setup.md) — Detailed environment setup and configuration
- [installation.md](./installation.md) — Installation prerequisites and requirements
- [development.md](./development.md) — Development workflow and best practices

---

## Support & Community

### Getting Help

We're here to help! When you encounter an issue:

1. **Check this README** - Most common questions are answered here
2. **Review the Troubleshooting section** - See [Troubleshooting](#troubleshooting) above
3. **Search GitHub Issues** - Your question may already be answered
4. **Ask the Community** - Post in forums or Discord servers

### Resources

**Official Documentation:**
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust fundamentals
- [Rust Standard Library](https://doc.rust-lang.org/std/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

**Framework Documentation:**
- [Axum Web Framework](https://github.com/tokio-rs/axum) - HTTP request handling
- [Tokio Async Runtime](https://tokio.rs) - Async/await and multitasking
- [SQLx Database Toolkit](https://sqlx.rs) - Type-safe SQL queries
- [PostgreSQL Documentation](https://www.postgresql.org/docs/) - Database reference
- [Redis Documentation](https://redis.io/documentation) - Caching and sessions

**Community:**
- [r/rust subreddit](https://reddit.com/r/rust) - Rust programming community
- [Rust Discord Server](https://discord.gg/rust-lang) - Real-time chat and support
- [Rust Users Forum](https://users.rust-lang.org/) - Structured discussions
- [Stack Overflow - rust tag](https://stackoverflow.com/questions/tagged/rust) - Q&A platform

### Reporting Issues

Found a bug? Have a feature request?

1. **Check existing issues** - Avoid duplicates
2. **Provide details**:
   - Error message and stack trace
   - Steps to reproduce
   - Your environment (OS, Rust version)
   - `.env` configuration (without secrets)

3. **Create an issue** on GitHub with the information above

---

## Frequently Asked Questions (FAQ)

### General Questions

**Q: Can I use this in production?**

A: Yes! This project is designed as a production-ready scaffold with security best practices. Before going live:
- [ ] Change `JWT_SECRET` to a strong random value
- [ ] Update SMTP credentials for emails
- [ ] Configure CORS origins properly
- [ ] Enable HTTPS/TLS
- [ ] Set up monitoring and logging
- [ ] Run `cargo audit` to check for vulnerabilities
- [ ] Review security policies and access controls

---

**Q: How long does the first build take?**

A: The first build typically takes **5-15 minutes** depending on your machine. This is normal for Rust projects! Subsequent builds are much faster (usually under 1 minute).

To speed it up:
- Use a machine with multiple CPU cores
- Increase RAM available to your system
- Ensure you're using SSD storage (not spinning disk)

---

**Q: Is Redis required?**

A: No, Redis is **optional** for development. 

- **For development**: Set `REDIS_URL` or leave it blank to disable caching
- **For production**: Recommended for caching and session management

To disable Redis:
1. Comment out Redis imports in `src/main.rs`
2. Remove or comment out `REDIS_URL` from `.env`
3. Remove redis-related dependencies from `Cargo.toml`

---

### Development Questions

**Q: How do I add a new module/app?**

A: Use the scaffolding tool:
```bash
cargo run --bin startapp -- my_new_app

# This creates:
# src/apps/my_new_app/
# ├── mod.rs
# ├── models.rs
# ├── schemas.rs
# └── handlers.rs
```

Then register it in `src/apps/mod.rs`.

---

**Q: How do I modify the database schema?**

A: Use Diesel migrations:
```bash
# 1. Create a new migration
cargo run --bin makemigrations -- migration_name

# 2. Edit the generated migration files
nano migrations/TIMESTAMP_migration_name/up.sql
nano migrations/TIMESTAMP_migration_name/down.sql

# 3. Apply the migration
cargo run --bin migrate

# 4. If needed, rollback
diesel migration redo --database-url "$DATABASE_URL"
```

---

**Q: How do I implement role-based access control (RBAC)?**

A: Here's a basic implementation:

1. **Add role column to users table**:
   ```sql
   ALTER TABLE users ADD COLUMN role VARCHAR(50) DEFAULT 'user';
   ```

2. **Create a role enum in models**:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq)]
   pub enum UserRole {
       Admin,
       Moderator,
       User,
   }
   ```

3. **Create authorization middleware**:
   ```rust
   pub fn require_admin(user_role: UserRole) -> Result<(), AppError> {
       if user_role != UserRole::Admin {
           return Err(AppError::Forbidden("Admin access required".to_string()));
       }
       Ok(())
   }
   ```

4. **Use in handlers**:
   ```rust
   pub async fn admin_endpoint(Extension(user): Extension<User>) -> Result<()> {
       require_admin(user.role)?;
       // ... handler logic
       Ok(())
   }
   ```

---

**Q: How do I test my API endpoints?**

A: Several options:

**Using curl** (command line):
```bash
curl -X POST http://127.0.0.1:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"pass123"}'
```

**Using Swagger UI**:
- Open `http://127.0.0.1:8000/swagger-ui/` in your browser
- Interactive testing with built-in documentation

**Using Insomnia** (GUI tool):
- Download from [insomnia.rest](https://insomnia.rest)
- Import OpenAPI spec from `http://127.0.0.1:8000/openapi.json`

**Using automated tests**:
```bash
cargo test
cargo test -- --nocapture  # See output
RUST_LOG=debug cargo test  # With logging
```

---

### Deployment Questions

**Q: How do I deploy this to production?**

A: Basic deployment steps:

1. **Build for release**:
   ```bash
   cargo build --release
   # Binary at: ./target/release/authentication
   ```

2. **Set up environment**:
   - Install dependencies on server
   - Create production `.env` with real credentials
   - Set up PostgreSQL database

3. **Run migrations**:
   ```bash
   cargo run --bin migrate
   ```

4. **Run the application**:
   ```bash
   ./target/release/authentication
   ```

5. **Set up reverse proxy** (Nginx/Apache):
   - Forward requests to `127.0.0.1:8000`
   - Enable TLS/HTTPS
   - Set up rate limiting

---

**Q: How do I handle secrets in production?**

A: Best practices:

- **Never commit `.env` to git**
- Use environment variables on your server
- Use secrets management tools:
  - AWS Secrets Manager
  - HashiCorp Vault
  - DigitalOcean App Platform
  - Heroku Config Vars

Example with environment variables only:
```bash
export DATABASE_URL="postgres://..."
export JWT_SECRET="very_secure_key"
./target/release/authentication
```

---

**Q: Can I dockerize this application?**

A: Yes! Create a `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/authentication /usr/local/bin/
EXPOSE 8000
CMD ["authentication"]
```

Build and run:
```bash
docker build -t rustauth .
docker run -p 8000:8000 -e DATABASE_URL="..." rustauth
```

---

### Performance & Optimization

**Q: How do I improve API response times?**

A: Optimization strategies:

1. **Enable logging to find bottlenecks**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Add database indexes**:
   ```sql
   CREATE INDEX idx_user_email ON users(email);
   CREATE INDEX idx_posts_user_id ON blog_posts(user_id);
   ```

3. **Use Redis caching**:
   - Cache frequently accessed data
   - Reduce database queries

4. **Connection pooling**:
   - Adjust `DATABASE_POOL_SIZE` in `.env`

5. **Use release builds**:
   ```bash
   cargo build --release
   ./target/release/authentication
   ```

---

**Q: How do I profile and optimize performance?**

A: Tools and techniques:

```bash
# Build with profiling support
cargo flamegraph

# Or use perf on Linux
sudo perf record ./target/release/authentication
sudo perf report

# Check for slow SQL queries
RUST_LOG=sqlx=debug cargo run
```

---

### Troubleshooting Questions

**Q: I get "error: could not compile authentication"**

A: Try:
1. Update Rust: `rustup update`
2. Clean build: `cargo clean && cargo build`
3. Check Rust version: `rustc --version` (should be 1.70.0+)

---

**Q: My changes aren't showing up when I run the app**

A: Solution:
1. Stop the running server (Ctrl+C)
2. Rebuild: `cargo build`
3. Run again: `cargo run`

For automatic rebuilds during development:
```bash
cargo install cargo-watch
cargo watch -x run
```

---

**Q: Permission denied when accessing PostgreSQL**

A: PostgreSQL authentication issue. Try:
```bash
# Reset to default credentials
sudo -u postgres psql

# In psql prompt:
postgres=# ALTER USER postgres WITH PASSWORD 'postgres';
postgres=# \q
```

---

### Contributing Questions

**Q: How do I contribute to this project?**

A: Steps for contributors:

1. **Fork the repository** on GitHub
2. **Create a feature branch**: `git checkout -b feature/your-feature`
3. **Make changes** and test thoroughly
4. **Run quality checks**:
   ```bash
   cargo fmt      # Format code
   cargo clippy   # Lint checks
   cargo test     # Run tests
   cargo audit    # Security audit
   ```
5. **Commit with descriptive message**: `git commit -m 'feat: add new feature'`
6. **Push to your fork**: `git push origin feature/your-feature`
7. **Create a Pull Request** with description and screenshots

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](./LICENSE) file for complete details.

### MIT License Summary
- ✅ **Allowed**: Commercial use, modification, distribution, private use
- ❌ **Prohibited**: Trademark use, liability
- ⚠️ **Required**: License and copyright notice

---

**Project Status**: Active Development  
**Last Updated**: April 13, 2026  
**Maintained By**: Development Team  
**Rust Version**: 1.70.0+  
**Current Build**: stable

---

## What's Next?

- Explore the [Swagger UI](http://127.0.0.1:8000/swagger-ui/) for interactive API testing
- Read [development.md](./development.md) for advanced development practices
- Check [setup.md](./setup.md) for detailed configuration options
- Review code examples in `src/apps/` for reference implementations

Happy coding! 🚀
