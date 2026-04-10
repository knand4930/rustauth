# Setup

This document explains how to prepare the repository for development and local execution.

## 1. Clone the repository

```bash
git clone <repository-url>
cd authentication
```

## 2. Install dependencies

Follow the instructions in `installation.md` to install:

- Rust toolchain
- PostgreSQL
- Diesel CLI
- `psql` command line client

## 3. Create a `.env` file

Create a `.env` file in the repository root and add the required environment variables.

Example `.env`:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/authentication_dev
REDIS_URL=redis://127.0.0.1/
JWT_SECRET=replace_with_a_secure_random_secret
SERVER_ADDR=127.0.0.1:8000
```

Adjust values for your environment.

## 4. Initialize the database

Create the database in PostgreSQL:

```bash
createdb authentication_dev
```

Then run migrations:

```bash
cargo run --bin migrate
```

## 5. Generate migrations after model changes

If you change app models or schema definitions, generate a migration from the current `src/apps/*/models.rs` files:

```bash
cargo makemigrations
```

## 6. Optional Redis setup

If the project uses Redis for session or cache support, install and start Redis:

```bash
redis-server
```

Then point `REDIS_URL` to the running Redis instance.

## 7. Verify the environment

Run a quick build:

```bash
cargo build
```

If build succeeds, the repository is ready for development.
