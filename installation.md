# Installation Guide

Complete installation instructions for RustAuth development environment.

## 📋 Table of Contents

- [System Requirements](#system-requirements)
- [Rust Toolchain](#rust-toolchain)
- [PostgreSQL](#postgresql)
- [Redis](#redis)
- [Development Tools](#development-tools)
- [Environment Configuration](#environment-configuration)
- [Verification](#verification)

---

## System Requirements

### Operating System

- **Linux**: Ubuntu 20.04+, Debian 11+, Fedora 35+
- **macOS**: 10.15+ (Intel or Apple Silicon)
- **Windows**: Windows 10+ (WSL2 recommended for development)

### Hardware Minimum

- **CPU**: 2+ cores
- **RAM**: 4+ GB
- **Disk**: 20 GB available

### Network

- Internet connection for downloading dependencies
- Can operate behind corporate firewall/proxy

---

## Rust Toolchain

### Installation

**Linux & macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Windows (WSL2):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Configure Toolchain

```bash
# Set default toolchain
rustup default stable

# Or use nightly (optional)
rustup default nightly

# Update to latest
rustup update
```

### Verify Installation

```bash
# Check Rust version
rustc --version   # Should be 1.70+

# Check Cargo version
cargo --version   # Should be 1.70+

# Check Rust location
which rustc
which cargo
```

### IDE Support (Optional)

```bash
# Install rust-analyzer for IDE support
rustup component add rust-analyzer

# Install clippy for linting
rustup component add clippy

# Install rustfmt for formatting
rustup component add rustfmt
```

---

## PostgreSQL

### macOS

```bash
# Using Homebrew
brew install postgresql

# Start service
brew services start postgresql

# Verify installation
psql --version  # Should be 12+
```

### Ubuntu/Debian

```bash
# Update package list
sudo apt update

# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib postgresql-client

# Install development libraries
sudo apt install -y libpq-dev

# Verify installation
psql --version

# Start service
sudo systemctl start postgresql
sudo systemctl enable postgresql  # Auto-start on boot
```

### Fedora/RHEL

```bash
# Install PostgreSQL
sudo dnf install -y postgresql postgresql-contrib postgresql-devel

# Initialize database
sudo /usr/bin/postgresql-setup initdb

# Start service
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### Windows

1. Download from [postgresql.org](https://www.postgresql.org/download/windows/)
2. Run installer
3. Note the password for `postgres` user
4. Add to PATH: `C:\Program Files\PostgreSQL\15\bin`

### Create Database & User

**Linux/macOS:**
```bash
# Switch to postgres user
sudo -u postgres psql

# Create user (in psql)
CREATE USER rustauth WITH PASSWORD 'secure_password';
ALTER USER rustauth CREATEDB;

# Create database
CREATE DATABASE auth_dev OWNER rustauth;

# Exit
\q
```

**Windows (Command Prompt):**
```cmd
psql -U postgres

# Enter password when prompted, then in psql:
CREATE USER rustauth WITH PASSWORD 'secure_password';
ALTER USER rustauth CREATEDB;
CREATE DATABASE auth_dev OWNER rustauth;
\q
```

### Verify Database Connection

```bash
# Test connection
psql -U rustauth -d auth_dev -h localhost -c "SELECT 1"

# Or using connection string
psql postgres://rustauth:secure_password@localhost:5432/auth_dev
```

---

## Redis

### macOS

```bash
# Using Homebrew
brew install redis

# Start service
brew services start redis

# Verify
redis-cli ping   # Should respond with PONG
```

### Ubuntu/Debian

```bash
# Install Redis
sudo apt install -y redis-server

# Start service
sudo systemctl start redis-server
sudo systemctl enable redis-server  # Auto-start

# Verify
redis-cli ping   # Should respond with PONG
```

### Fedora/RHEL

```bash
# Install Redis
sudo dnf install -y redis

# Start service
sudo systemctl start redis
sudo systemctl enable redis

# Verify
redis-cli ping
```

### Windows

1. Install via WSL2: `sudo apt install redis-server`
2. Or use Windows binaries from [github.com/microsoftarchive/redis](https://github.com/microsoftarchive/redis/releases)
3. Start service

### Verify Installation

```bash
# Connect to Redis
redis-cli

# In redis-cli prompt:
ping           # Response: PONG
SET test 123   # SET key value
GET test       # GET test -> 123
DEL test       # Clean up
exit           # Quit
```

---

## Development Tools

### Cargo Watch (Auto-reload)

```bash
# Install
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run

# Or with tests
cargo watch -x test
```

### Clippy (Linting)

```bash
# Already installed with rustup, just run
cargo clippy

# Or strict mode
cargo clippy -- -D warnings
```

### SQLx CLI (Optional)

```bash
# Install SQLx CLI for database tools
cargo install sqlx-cli --no-default-features --features postgres

# Verify migration schemas
sqlx database prepare
```

### Other Useful Tools

```bash
# Rustfmt for code formatting
cargo install rustfmt-nightly

# Flamegraph for profiling
cargo install flamegraph

# Cargo-expand to expand macros
cargo install cargo-expand

# Cargo-tree to visualize dependencies
cargo tree
```

---

## Environment Configuration

### Create .env File

```bash
# In project root directory
cp .env.example .env

# Edit with your values
nano .env
# or
code .env
```

### .env Template

```env
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8000
RUST_LOG=info,tower_http=warn

# Database
DATABASE_URL=postgres://rustauth:secure_password@localhost:5432/auth_dev
DATABASE_MAX_CONNECTIONS=10

# Redis
REDIS_URL=redis://127.0.0.1:6379

# JWT Configuration
JWT_SECRET=your-very-long-secure-random-key-minimum-64-characters-long
JWT_EXPIRY_HOURS=24
JWT_REFRESH_EXPIRY_DAYS=7

# Email Configuration (optional)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@example.com

# Application
APP_NAME=RustAuth
APP_ENV=development
APP_DOMAIN=http://localhost:8000
```

### Security Notes

- **Never commit .env to Git** - Use .env.example instead
- **Keep JWT_SECRET secure** - Use strong random string (min 64 chars)
- **Use different values for production**
- **Store sensitive data in secrets manager** for deployment

---

## Verification

### Quick Verification Script

```bash
#!/bin/bash

echo "=== RustAuth Installation Verification ==="

# Rust
echo "Rust version:"
rustc --version

# Cargo
echo "Cargo version:"
cargo --version

# PostgreSQL
echo "PostgreSQL version:"
psql --version

# Redis
echo "Redis version:"
redis-server --version

# Check database connection
echo "Testing database connection..."
psql postgres://rustauth:secure_password@localhost:5432/auth_dev -c "SELECT 1"

# Check Redis connection
echo "Testing Redis connection..."
redis-cli ping

echo "=== All systems ready! ==="
```

### Step-by-Step Verification

```bash
# 1. Verify Rust
rustc --version  # Should be 1.70+

# 2. Verify PostgreSQL
psql --version   # Should be 12+
psql -U rustauth -d auth_dev -c "SELECT 1"  # Should return 1

# 3. Verify Redis
redis-cli ping   # Should return PONG

# 4. Clone repository (if not done)
git clone <repository-url>
cd authentication

# 5. Create .env
cp .env.example .env

# 6. Build project
cargo build

# 7. Test database migrations
cargo run --bin migrate

# 8. Run application
cargo run

# Should see:
# - "Starting server on 127.0.0.1:8000"
# - "Database connected"
# - "AdminX initialized"

echo "✓ Installation complete!"
```

---

## Troubleshooting

### PostgreSQL Connection Refused

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Start if not running
sudo systemctl start postgresql

# Verify user and database exist
sudo -u postgres psql -l  # List databases
sudo -u postgres psql -c "SELECT usename FROM pg_user;"  # List users
```

### Redis Connection Refused

```bash
# Check if Redis is running
sudo systemctl status redis-server

# Start if not running
sudo systemctl start redis-server

# Check listening port
sudo netstat -tlnp | grep redis
```

### Permission Denied on macOS

```bash
# If getting permission errors
# Reinstall PostgreSQL with proper permissions
brew uninstall postgresql
brew install postgresql
brew services start postgresql
```

### Windows WSL2 Issues

```bash
# Update WSL2
wsl --update

# Install in WSL2
wsl apt update
wsl apt install -y postgresql postgresql-client redis-server

# Start services
sudo service postgresql start
sudo service redis-server start
```

---

## Next Steps

Once installation is complete:

1. Read [setup.md](setup.md) for initial project setup
2. Check [development.md](development.md) for development workflow
3. Review [README.md](README.md) for project overview
4. See [API.md](API.md) for API documentation

---

## System Compatibility

| Component | Linux | macOS | Windows (WSL2) |
|-----------|-------|-------|----------|
| Rust | ✅ | ✅ | ✅ |
| PostgreSQL | ✅ | ✅ | ✅ |
| Redis | ✅ | ✅ | ✅ |
| Cargo tools | ✅ | ✅ | ✅ |

---

For more help, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
```
