# Troubleshooting Guide

Solutions to common issues and problems when working with RustAuth.

## 📋 Table of Contents

- [Build Issues](#build-issues)
- [Runtime Issues](#runtime-issues)
- [Database Issues](#database-issues)
- [Authentication Issues](#authentication-issues)
- [Performance Issues](#performance-issues)
- [Deployment Issues](#deployment-issues)
- [Debugging](#debugging)
- [Getting Help](#getting-help)

---

## Build Issues

### Compilation Error: "could not find 'postgres' in `deps`"

**Problem:** Missing PostgreSQL development libraries

**Solution:**
```bash
# Ubuntu/Debian
sudo apt install libpq-dev

# macOS
brew install postgresql

# Then rebuild
cargo build
```

### Error: "failed to verify the checksum of 'sqlx'"

**Problem:** Corrupted dependency cache

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build

# Or update dependencies
cargo update
cargo build
```

### Clippy Error: "warning: this expression will panic at runtime"

**Problem:** Code uses unsafe patterns

**Solution:**
```rust
// ❌ BAD - Can panic
let value = vec![1, 2, 3][10];

// ✅ GOOD - Safe access
let value = vec![1, 2, 3].get(10);
```

### Error: "no default `database_url` environment variable set"

**Problem:** DATABASE_URL not configured

**Solution:**
```bash
# Create .env file
echo "DATABASE_URL=postgres://user:password@localhost:5432/auth_dev" > .env

# Or set environment variable
export DATABASE_URL=postgres://user:password@localhost:5432/auth_dev

# Verify
echo $DATABASE_URL
```

---

## Runtime Issues

### Application Won't Start: "Database connection failed"

**Problem:** Cannot connect to PostgreSQL

**Diagnostic:**
```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Try to connect directly
psql -h localhost -U postgres -d auth_dev

# Check connection string
echo $DATABASE_URL
```

**Solution:**
```bash
# Start PostgreSQL
sudo systemctl start postgresql

# Verify connection string
# Format: postgres://[user[:password]@][host[:port]][/database]
# Example: postgres://rustauth:password@localhost:5432/auth_dev

# Update .env
DATABASE_URL=postgres://rustauth:password@localhost:5432/auth_dev

# Restart application
cargo run
```

### Error: "connection pool size exceeded"

**Problem:** Too many connections or connection leak

**Diagnostic:**
```sql
-- Check active connections
SELECT datname, count(*) FROM pg_stat_activity GROUP BY datname;

-- List connections
SELECT * FROM pg_stat_activity;
```

**Solution:**
```rust
// Adjust pool size in config
let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(5)  // Reduce from 20
    .connect(&url)
    .await?;
```

```env
# Or in .env
DATABASE_MAX_CONNECTIONS=5
```

### Port Already in Use: "Address already in use"

**Problem:** Port 8000 is already in use

**Diagnostic:**
```bash
# Find process using port 8000
lsof -i :8000
netstat -tlnp | grep 8000
```

**Solution:**
```bash
# Kill the process
kill -9 <PID>

# Or use different port
SERVER_PORT=8001 cargo run
```

### Redis Connection Refused

**Problem:** Cannot connect to Redis

**Diagnostic:**
```bash
# Check if Redis is running
redis-cli ping

# Try to connect
telnet localhost 6379
```

**Solution:**
```bash
# Start Redis
redis-server

# Check Redis configuration
redis-cli INFO

# Verify connection in .env
REDIS_URL=redis://127.0.0.1:6379
```

### Out of Memory (OOM) Error

**Problem:** Application using too much memory

**Diagnostic:**
```bash
# Check memory usage
free -h
ps aux | grep rustauth

# Monitor with htop
htop
```

**Solution:**
```rust
// Reduce connection pool
DATABASE_MAX_CONNECTIONS=5  // was 20

// Limit query results
.limit(100)  // Add pagination
.offset((page - 1) * 100)

// Use streaming for large results
sqlx::query_as::<_, User>(/* ... */)
    .fetch_all(&pool)  // ❌ Loads all
    
sqlx::query_as::<_, User>(/* ... */)
    .fetch_optional(&pool)  // ✅ One at a time
```

---

## Database Issues

### Migration Failed: "relation already exists"

**Problem:** Migration script trying to create existing table

**Diagnostic:**
```bash
# Check existing tables
\dt user.*
\dt blogs.*
```

**Solution:**
```bash
# Check migration status
cargo run --bin showmigrations

# Rollback last migration
cargo run --bin migrate down

# Fix migration and retry
cargo run --bin migrate
```

### Database Locked

**Problem:** Database transaction taking too long

**Diagnostic:**
```sql
-- Find long-running queries
SELECT * FROM pg_stat_activity 
WHERE state = 'active'
AND query_start < now() - interval '5 minutes';
```

**Solution:**
```sql
-- Kill long-running query
SELECT pg_terminate_backend(pid) 
FROM pg_stat_activity 
WHERE query_start < now() - interval '5 minutes';
```

### Foreign Key Constraint Violation

**Problem:** Deleting record with child records

**Diagnostic:**
```bash
# Check referential integrity
-- Attempt delete
DELETE FROM user.users WHERE id = ? RETURNING *;
-- Check for foreign key errors
```

**Solution:**
```sql
-- Delete in correct order (children first)
DELETE FROM blogs.comments WHERE blog_post_id = ?;
DELETE FROM blogs.blog_posts WHERE author_id = ?;
DELETE FROM user.users WHERE id = ?;

-- Or enable cascade delete
ALTER TABLE comments 
DROP CONSTRAINT fk_comment_user;

ALTER TABLE comments 
ADD CONSTRAINT fk_comment_user 
FOREIGN KEY (user_id) 
REFERENCES users(id) ON DELETE CASCADE;
```

---

## Authentication Issues

### Login Fails: "Invalid credentials"

**Problem:** User cannot login with correct password

**Diagnostic:**
```rust
// Enable debug logging
RUST_LOG=debug cargo run

// Check user exists
SELECT * FROM user.users WHERE email = 'user@example.com';

// Verify password hash
SELECT password FROM user.users WHERE email = 'user@example.com';
```

**Solution:**
```bash
# Reset password
cargo run --bin createsuperuser

# Or directly in database
UPDATE user.users 
SET password = '$2b$12$...'  -- Argon2 hash
WHERE email = 'user@example.com';
```

### JWT Token Expired

**Problem:** "Token expired" error on valid token

**Diagnostic:**
```rust
// Decode token to check expiry
// Use jwt.io online tool or decode_token function
```

**Solution:**
```bash
# Refresh token
curl -X POST http://localhost:8000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "your-refresh-token"}'

# Or login again
```

### CORS Error: "No 'Access-Control-Allow-Origin' header"

**Problem:** Browser blocks cross-origin request

**Diagnostic:**
```javascript
// Check browser console for CORS errors
// Check request headers
```

**Solution:**
```env
# Allow specific origins in .env
CORS_ALLOWED_ORIGINS=https://example.com,https://app.example.com

# Or in code
let cors = CorsLayer::new()
    .allow_origin("https://example.com".parse()?)
    .allow_credentials()
    .allow_methods(/* ... */);
```

### Unauthorized Error Even With Token

**Problem:** Valid token still gets 401

**Diagnostic:**
```rust
// Enable trace logging
RUST_LOG=debug,tower_http=trace cargo run

// Check token in request
// Authorization: Bearer <token>
```

**Solution:**
```bash
# Verify token format (Bearer + space + token)
curl -H "Authorization: Bearer eyJhbGc..." http://localhost:8000/api/users/me

# Check JWT_SECRET matches
echo $JWT_SECRET
```

---

## Performance Issues

### Slow Query Performance

**Problem:** API requests taking >1 second

**Diagnostic:**
```bash
# Enable query logging
RUST_LOG=sqlx=debug cargo run

# Check query plans
EXPLAIN ANALYZE SELECT ...;
```

**Solution:**
```sql
-- Add indexes
CREATE INDEX idx_user_email ON user.users(email);
CREATE INDEX idx_user_created_at ON user.users(created_at DESC);
CREATE INDEX idx_blog_author ON blogs.blog_posts(author_id);
CREATE INDEX idx_blog_published ON blogs.blog_posts(published_at DESC);

-- Optimize queries
-- Use LIMIT for pagination
-- Select only needed columns
-- Avoid N+1 queries
```

### High Memory Usage

**Problem:** RAM usage keeps growing

**Diagnostic:**
```bash
# Monitor memory
top -b -n 1 | grep -E "PID|rustauth"

# Check which queries consume memory
EXPLAIN ANALYZE SELECT * FROM large_table;
```

**Solution:**
```rust
// Use pagination
let limit = 100;
let offset = (page - 1) * limit;

// Stream large results
sqlx::query_as::<_, User>(/* ... */)
    .fetch_all(&pool)  // ❌ Wrong
    
sqlx::query_as::<_, User>(/* ... */)
    .fetch_optional(&pool)  // ✅ Correct in loop

// Reduce connection pool
DATABASE_MAX_CONNECTIONS=5
```

### High CPU Usage

**Problem:** CPU consistently >80%

**Diagnostic:**
```bash
# Check CPU usage
top

# Find hot spots with profiler
cargo install flamegraph
cargo flamegraph
```

**Solution:**
```rust
// Cache frequently accessed data
// Use Redis for session data
// Optimize algorithms (O(n²) to O(n))
// Add indexes to slow queries
```

---

## Deployment Issues

### Container Won't Start: "error: couldn't compile"

**Problem:** Docker build fails

**Diagnostic:**
```bash
# Check build logs
docker build -t rustauth . --progress=plain

# Test locally first
cargo build --release
```

**Solution:**
```dockerfile
# Add error details to Dockerfile
FROM rust:1.70

WORKDIR /app
COPY . .

# Show compile errors
RUN cargo build --release 2>&1 | tail -50

# Continue build
```

### PostgreSQL Connection Refused in Container

**Problem:** Container can't reach database

**Diagnostic:**
```bash
# Check network
docker network ls

# Test connectivity
docker exec rustauth-app \
  psql -h postgres-host -U user -d auth_dev -c "SELECT 1"

# Check DNS resolution
docker exec rustauth-app nslookup postgres-host
```

**Solution:**
```yaml
# Use docker-compose
services:
  app:
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://user:pass@postgres:5432/auth_dev
  
  postgres:
    environment:
      POSTGRES_DB: auth_dev
```

---

## Debugging

### Enable Debug Logging

```bash
# Set trace level
RUST_LOG=trace cargo run

# Database queries
RUST_LOG=sqlx=debug cargo run

# HTTP requests
RUST_LOG=tower_http=debug cargo run

# Specific module
RUST_LOG=rustauth::apps::user=debug cargo run
```

### Using Debugger

```bash
# Install debugger
cargo install cargo-expand

# Expand macros
cargo expand

# Or use breakpoints
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    stderr.read_line(&mut String::new()).ok();  // Pause for debugger
}
```

### Common Debug Commands

```bash
# Check what's being compiled
cargo build -v

# Expand proc macros
cargo expand src/main.rs

# Generate docs
cargo doc --open

# Check dependencies
cargo tree

# Outdated dependencies
cargo outdated
```

---

## Getting Help

### Resources

- **Documentation:** [README.md](README.md)
- **API Reference:** [API.md](API.md)
- **Architecture:** [ARCHITECTURE.md](ARCHITECTURE.md)
- **Database:** [DATABASE.md](DATABASE.md)
- **Security:** [SECURITY.md](SECURITY.md)

### Community

- **GitHub Issues:** https://github.com/yourrepo/issues
- **Discussions:** https://github.com/yourrepo/discussions
- **Email:** support@example.com
- **Slack:** [Community Slack](#)

### Reporting Bugs

```markdown
## Describe the Bug
Clear description

## Steps to Reproduce
1. ...
2. ...

## Environment
- OS: Ubuntu 22.04
- Rust: 1.70
- RUST_LOG: debug

## Error Message
\`\`\`
Include full error/stack trace
\`\`\`

## Additional Context
Any relevant information
```

---

**Last Updated:** April 2024
