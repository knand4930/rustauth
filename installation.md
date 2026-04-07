# Installation

Install the required tools to run this Rust authentication project.

## Rust toolchain

Install Rust using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
rustup update
```

Verify the toolchain:

```bash
rustc --version
cargo --version
```

## PostgreSQL

Install PostgreSQL for your platform.

Ubuntu example:

```bash
sudo apt update
sudo apt install -y postgresql postgresql-contrib libpq-dev
```

Start PostgreSQL:

```bash
sudo service postgresql start
```

Create a local database user and database:

```bash
sudo -u postgres createuser --superuser $USER
createdb authentication_dev
```

## Redis (optional)

If the application uses Redis, install it:

```bash
sudo apt install -y redis-server
sudo service redis-server start
```

## Diesel CLI

Install Diesel CLI with Postgres support:

```bash
cargo install diesel_cli --no-default-features --features postgres
```

Confirm installation:

```bash
dice --version
```

## Other useful tools

- `psql` — Postgres client
- `sqlx-cli` — optional SQLx tooling
- `git` — version control

## Optional

Install `sqlx-cli` if you want SQLx helpers:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## Environment variables

Create a `.env` file in the repository root with the values required by the app.

Example:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/authentication_dev
REDIS_URL=redis://127.0.0.1/
JWT_SECRET=replace_with_a_secure_random_secret
SERVER_ADDR=127.0.0.1:8000
```

Then load the environment when running commands:

```bash
dotenv cargo run
```
