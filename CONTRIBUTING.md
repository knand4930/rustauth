# Contributing Guidelines

Thank you for your interest in contributing to RustAuth! This document provides guidelines and instructions for contributing.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contribution Process](#contribution-process)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Commit Messages](#commit-messages)
- [Pull Requests](#pull-requests)
- [Reporting Issues](#reporting-issues)
- [Documentation](#documentation)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inspiring community for all. Please read and adhere to our Code of Conduct:

- **Be Respectful**: Treat everyone with respect and courtesy
- **Be Inclusive**: Welcome diverse perspectives and experiences
- **Be Constructive**: Provide helpful and constructive feedback
- **Be Professional**: Maintain professional communication

### Unacceptable Behavior

The following behaviors are unacceptable:
- Harassment, discrimination, or hateful language
- Personal attacks or insults
- Unwanted sexual advances
- Sharing others' private information
- Spam or advertising

**Report violations to:** maintainers@example.com

---

## Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- Redis 6.0+
- Git
- Basic understanding of Rust and async programming

### Fork & Clone

```bash
# Fork on GitHub
# Click "Fork" button

# Clone your fork
git clone https://github.com/YOUR_USERNAME/rustauth.git
cd rustauth

# Add upstream remote
git remote add upstream https://github.com/original/rustauth.git
```

---

## Development Setup

### 1. Install Dependencies

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# PostgreSQL
sudo apt install postgresql postgresql-contrib

# Redis
sudo apt install redis-server

# Diesel CLI
cargo install diesel_cli --no-default-features --features postgres
```

### 2. Setup Local Database

```bash
# Create database and user
sudo -u postgres createuser rustauth
sudo -u postgres createdb -O rustauth auth_dev

# Create .env file
cp .env.example .env

# Update .env
DATABASE_URL=postgres://rustauth:password@localhost:5432/auth_dev

# Run migrations
cargo run --bin migrate
```

### 3. Start Development Server

```bash
# Terminal 1: Start Redis
redis-server

# Terminal 2: Start Rust server
cargo run

# Terminal 3: (Optional) Watch for changes
cargo watch -x run
```

---

## Contribution Process

### 1. Create Feature Branch

```bash
# Update local main
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name

# Example branch names:
# feature/add-mfa
# fix/password-reset-bug
# docs/update-readme
# refactor/database-queries
```

### 2. Make Changes

```bash
# Edit files
# Follow code standards (see below)

# Build
cargo build

# Format code
cargo fmt

# Lint
cargo clippy

# Run tests
cargo test
```

### 3. Commit Changes

```bash
# Stage changes
git add .

# Commit with message
git commit -m "Description of changes"

# Push to your fork
git push origin feature/your-feature-name
```

### 4. Create Pull Request

1. Go to GitHub repository
2. Click "Compare & pull request"
3. Fill in PR description
4. Link any related issues
5. Submit PR

---

## Code Standards

### Rust Style Guide

Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

**Example:**
```rust
// ✅ GOOD - Clear, descriptive names
pub async fn authenticate_user(
    email: &str,
    password: &str,
    pool: &PgPool,
) -> Result<User, AuthError> {
    // Implementation
}

// ❌ BAD - Unclear abbreviations
pub async fn auth_usr(em: &str, pwd: &str, p: &PgPool) -> Result<User, AuthError> {
    // Implementation
}
```

### Formatting

```bash
# Auto-format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Address all warnings
```

### Error Handling

**Use Result types:**
```rust
// ✅ GOOD
pub async fn get_user(id: Uuid, pool: &PgPool) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM user.users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::UserNotFound)?;
    
    Ok(user)
}

// ❌ BAD - Unwrap can panic
pub async fn get_user(id: Uuid, pool: &PgPool) -> User {
    let user = sqlx::query_as::<_, User>(/* ... */)
        .fetch_one(pool)
        .await
        .unwrap();  // Will panic if not found!
    
    user
}
```

### Comments & Documentation

```rust
/// Get user by ID from database
///
/// # Arguments
/// * `id` - The UUID of the user
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok(User)` - User found
/// * `Err(ApiError::UserNotFound)` - User not found
///
/// # Example
/// ```
/// let user = get_user(uuid, &pool).await?;
/// ```
pub async fn get_user(id: Uuid, pool: &PgPool) -> Result<User> {
    // Implementation
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password() {
        // Valid password
        assert!(validate_password("SecurePass123!").is_ok());
        
        // Invalid password (too short)
        assert!(validate_password("Pass1!").is_err());
        
        // Invalid password (no uppercase)
        assert!(validate_password("securepass123!").is_err());
    }
}
```

### Integration Tests

```bash
# Create test file
# tests/integration_test.rs

#[tokio::test]
async fn test_user_login() {
    let pool = create_test_pool().await;
    
    // Test login flow
    let result = login(&pool, "user@test.com", "password").await;
    
    assert!(result.is_ok());
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_validate_password

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

---

## Commit Messages

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation change
- `style:` - Code style change (formatting, missing semicolons, etc.)
- `refactor:` - Code refactor (no feature change or bug fix)
- `perf:` - Performance improvement
- `test:` - Add/update tests
- `chore:` - Build process or dependency update

### Examples

**Good commits:**
```
feat(auth): add passwordless authentication

- Implement email-based passwordless auth
- Add token generation and validation
- Support 24-hour token expiration

Closes #123
```

```
fix(user): correct password reset validation

User could reset password with invalid token.
Added proper token expiration check.

Fixes #456
```

```
docs(readme): update installation instructions

- Clarify PostgreSQL version requirement
- Add Redis installation steps
- Fix typo in environment variables section
```

---

## Pull Requests

### PR Title

Clear, descriptive title following commit message format:

```
feat(auth): add two-factor authentication (2FA)
fix(user): resolve profile update race condition
docs(api): add endpoint rate limiting documentation
```

### PR Description

```markdown
## Description
Brief description of what this PR does.

## Motivation and Context
Why is this change needed? Link to related issues.

Fixes #123
Related to #456

## Type of Change
- [ ] New feature
- [ ] Bug fix
- [ ] Breaking change
- [ ] Documentation

## How Has This Been Tested?
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Tested locally
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings with `cargo clippy`
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### PR Review Process

1. Automated checks must pass
2. Minimum 2 maintainer reviews
3. All conversations resolved
4. Tests passing
5. Coverage maintained

---

## Reporting Issues

### Bug Report Template

```markdown
## Describe the Bug
Clear description of the bug.

## Steps to Reproduce
1. First step
2. Second step
3. Third step

## Expected Behavior
What should happen

## Actual Behavior
What actually happened

## Environment
- OS: [e.g., Ubuntu 22.04]
- Rust version: [output of `rustc --version`]
- RustAuth version: [commit hash or tag]

## Logs/Error Messages
```
Include full error/stack trace
```

## Additional Context
Any other relevant information
```

### Feature Request Template

```markdown
## Description
Clear description of the desired feature.

## Motivation
Why is this feature needed? What problem does it solve?

## Suggested Implementation
How should this feature work?

## Alternatives
Have you considered any alternatives?

## Additional Context
Any other relevant information
```

---

## Documentation

### API Documentation

Update `API.md` when adding/changing endpoints:

```markdown
### New Endpoint

Brief description.

\`\`\`http
POST /api/endpoint
\`\`\`

**Request:**
\`\`\`json
{ "field": "value" }
\`\`\`

**Response:** \`200 OK\`
\`\`\`json
{ "id": "...", "field": "value" }
\`\`\`
```

### README Updates

Update README.md for major changes:

```markdown
- Add new section to table of contents
- Update feature list if applicable
- Add examples if applicable
```

### CHANGELOG

Keep CHANGELOG.md updated:

```markdown
## [1.1.0] - 2024-04-20

### Added
- New feature description

### Changed
- Change description

### Fixed
- Bug fix description

### Deprecated
- Deprecated feature description

### Removed
- Removed feature description

### Security
- Security fix description
```

---

## Development Workflow

### Watch Mode

```bash
# Install cargo-watch
cargo install cargo-watch

# Run app in watch mode
cargo watch -x run

# Run tests in watch mode
cargo watch -x test
```

### Database Migrations

```bash
# Generate migration from model changes
cargo makemigrations

# Apply migrations
cargo run --bin migrate

# Check migration status
cargo run --bin showmigrations
```

### Create New App

```bash
# Scaffold new app
cargo startapp my_feature

# Edit generated files
# Add models, handlers, routes
# Register in admin panel

# Generate migration
cargo makemigrations

# Apply migration
cargo run --bin migrate
```

---

## Questions & Support

- **Documentation:** See [README.md](README.md) and other docs/
- **Issues:** [GitHub Issues](https://github.com/yourrepo/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourrepo/discussions)
- **Email:** maintainers@example.com

---

## Recognition

Contributors will be recognized in:
- GitHub contributors page
- CONTRIBUTORS.md file
- Project announcements

Thank you for contributing to RustAuth!
