# Setup Guide

Complete setup instructions for RustAuth development environment.

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Repository Setup](#repository-setup)
- [Environment Configuration](#environment-configuration)
- [Database Setup](#database-setup)
- [Verification](#verification)
- [Next Steps](#next-steps)

---

## Prerequisites

Before starting, ensure you have installed:

- ✅ Rust 1.70+ ([installation.md](installation.md))
- ✅ PostgreSQL 12+ ([installation.md](installation.md))
- ✅ Redis 6.0+ ([installation.md](installation.md))
- ✅ Git
- ✅ Text editor (VS Code, Vim, Nano, etc.)

### Verify Prerequisites

```bash
# Check all required tools are installed
rustc --version   # Should be 1.70+
cargo --version
psql --version    # Should be 12+
redis-cli ping    # Should return PONG
git --version
```

---

## Repository Setup

### 1. Clone Repository

```bash
# Clone from GitHub
git clone https://github.com/yourname/rustauth.git
cd rustauth

# Or if you forked the repository
git clone https://github.com/YOUR_USERNAME/rustauth.git
cd rustauth

# Add upstream remote (optional, for staying updated)
git remote add upstream https://github.com/original/rustauth.git
```

### 2. Project Structure

```
rustauth/
├── src/                # Source code
│   ├── main.rs        # Application entry point
│   ├── lib.rs         # Library exports
│   ├── admin/         # Admin panel
│   ├── apps/          # Application modules (user, blogs)
│   ├── bin/           # CLI commands
│   └── commands/      # Command logic
├── migrations/        # Database migrations
├── Cargo.toml        # Project manifest
├── .env.example      # Environment template
└── README.md         # Project readme
```

### 3. Install Dependencies

```bash
# Update dependency index
cargo update

# Download dependencies (this may take a few minutes)
cargo fetch
```

---

## Environment Configuration

### 1. Create .env File

```bash
# Copy example to .env
cp .env.example .env

# Edit with your values
nano .env
# or
code .env
# or
vim .env
```

### 2. Configure .env

Update the `.env` file with your local values:

```env
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8000
RUST_LOG=info,tower_http=warn

# Database Connection
DATABASE_URL=postgres://rustauth:secure_password@localhost:5432/auth_dev

# Redis
REDIS_URL=redis://127.0.0.1:6379

# JWT Security
JWT_SECRET=your-very-long-secure-random-key-minimum-64-characters-long
JWT_EXPIRY_HOURS=24
JWT_REFRESH_EXPIRY_DAYS=7

# Email (optional - for development)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587

# Development settings
APP_ENV=development
RUST_LOG=debug
```

### 3. Create .env.local (Optional)

For local overrides without affecting .env:

```bash
# Create local overrides
echo "RUST_LOG=debug" > .env.local
echo "SERVER_PORT=8001" >> .env.local

# .env.local is git-ignored and only used locally
```

---

## Database Setup

### 1. Verify PostgreSQL Services

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# If not running, start it
sudo systemctl start postgresql

# Enable auto-start on boot
sudo systemctl enable postgresql
```

### 2. Verify Database & User Exist

```bash
# List all databases
psql -U postgres -l

# You should see: auth_dev

# List users
psql -U postgres -c "SELECT usename FROM pg_user;"

# You should see: rustauth
```

### 3. Create Database (If Not Exists)

```bash
# Connect as postgres user
sudo -u postgres psql

# Create user (if needed)
CREATE USER rustauth WITH PASSWORD 'secure_password';
ALTER USER rustauth CREATEDB;

# Create database
CREATE DATABASE auth_dev OWNER rustauth;

# Verify creation
\c auth_dev    # Connect to database
\dt            # List tables (should be empty)
\q             # Quit psql
```

### 4. Apply Migrations

```bash
# Run all pending migrations
cargo run --bin migrate

# Check migration status
cargo run --bin showmigrations

# Output should show:
# ✅ 20260414110602_auto        [applied]
# ✅ 20260414115951_auto        [applied]
```

### 5. Create Superuser (Optional)

```bash
# Create admin account
cargo run --bin createsuperuser

# Follow the prompts:
# Email: admin@example.com
# Password: (enter secure password)
# Full Name: Admin User

# Verify user was created
psql -U rustauth -d auth_dev -c "SELECT id, email, is_superuser FROM user.users;"
```

---

## Build & Verify

### 1. Initial Build

```bash
# Build the project
cargo build

# This may take 2-5 minutes on first build

# You should see:
# Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

### 2. Verify Database Connection

```bash
# Check migrations are applied
cargo run --bin showmigrations

# Should show all migrations as [applied]
```

### 3. Test Basic Functionality

```bash
# Run tests (optional)
cargo test

# Format check
cargo fmt --check

# Lint check  
cargo clippy

# All should pass
```

---

## Verification

### 1. Start Application

```bash
# Terminal 1: Start the server
cargo run

# You should see:
#   Starting server on 127.0.0.1:8000
#   Database connected
#   AdminX initialized with N apps and M resources
```

### 2. Test API

**In another terminal:**

```bash
# Health check
curl http://localhost:8000/health

# Response:
# {"status":"ok","database":"connected","redis":"connected"}
```

### 3. View API Documentation

Open browser and navigate to:

```
http://localhost:8000/docs
```

You should see interactive Swagger API documentation.

### 4. Register a User

```bash
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!",
    "full_name": "Test User"
  }'

# Response should include user ID and email
```

### 5. Login

```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!"
  }'

# Response should include access_token and refresh_token
```

---

## Troubleshooting Setup

### Database Connection Failed

```bash
# Verify DATABASE_URL in .env
echo $DATABASE_URL

# Test direct connection
psql postgres://rustauth:password@localhost:5432/auth_dev

# If still failing, check PostgreSQL is running
sudo systemctl status postgresql
sudo systemctl start postgresql
```

### Port Already in Use

```bash
# Find process using port 8000
lsof -i :8000

# Kill the process (replace PID)
kill -9 <PID>

# Or use different port
SERVER_PORT=8001 cargo run
```

### Cargo Build Fails

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Build again
cargo build

# With verbose output if still failing
cargo build --verbose
```

### Migrations Won't Apply

```bash
# Check migration status
cargo run --bin showmigrations

# Check database state
psql -U rustauth -d auth_dev -c "SELECT * FROM _sqlx_migrations;"

# Reset database (development only!)
psql -U rustauth -d auth_dev -c "DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;"

# Then reapply migrations
cargo run --bin migrate
```

---

## Next Steps

### Development Workflow

1. Read [development.md](development.md) for daily development workflow
2. Check [commands.md](commands.md) for available CLI commands
3. Review [API.md](API.md) for API reference

### IDE Setup (Optional)

**VS Code:**
```bash
# Install extensions:
# - rust-analyzer
# - CodeLLDB
# - Even Better TOML
# - Better Comments
```

**Create .vscode/launch.json:**
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug",
      "cargo": {
        "args": [
          "build",
          "--bin=rustauth"
        ]
      }
    }
  ]
}
```

### Git Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: description of feature"

# Push to your fork
git push origin feature/my-feature

# Create Pull Request on GitHub
```

### Database GUI (Optional)

View/manage database with GUI tools:

- **pgAdmin** - Web-based PostgreSQL manager
- **DBeaver** - Full-featured database IDE
- **DataGrip** - JetBrains database IDE

```bash
# Install pgAdmin (Docker)
docker run -p 5050:80 -e PGADMIN_DEFAULT_EMAIL=admin@example.com \
  -e PGADMIN_DEFAULT_PASSWORD=password \
  dpage/pgadmin4
```

---

## Monitoring Development

### Check Server Status

```bash
# Get server PID
ps aux | grep rustauth

# Monitor resource usage
top

# Or use htop (better interface)
htop
```

### View Logs

```bash
# Tail logs (if redirected to file)
tail -f app.log

# Or directly in console with:
RUST_LOG=debug cargo run
```

### Database Queries

```bash
# Connect to database to inspect data
psql -U rustauth -d auth_dev

# Common queries:
SELECT * FROM user.users;
SELECT * FROM blogs.blog_posts;
SELECT * FROM user.user_roles;
```

---

## Production Preparation

For deploying to production, see:
- [DEPLOYMENT.md](DEPLOYMENT.md) - Deployment instructions
- [SECURITY.md](SECURITY.md) - Security best practices
- [DATABASE.md](DATABASE.md) - Database setup for production

---

**Setup complete!** 🚀 Start developing with `cargo run` and happy coding!

