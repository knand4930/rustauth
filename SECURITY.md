# Security Best Practices

Comprehensive security guidelines and best practices for RustAuth.

## 📋 Table of Contents

- [Authentication Security](#authentication-security)
- [Password Security](#password-security)
- [JWT Token Security](#jwt-token-security)
- [API Security](#api-security)
- [Database Security](#database-security)
- [Deployment Security](#deployment-security)
- [Data Protection](#data-protection)
- [Security Checklist](#security-checklist)

---

## Authentication Security

### Password Requirements

**Minimum Standards:**
- Length: 12+ characters recommended
- Complexity: Uppercase, lowercase, numbers, special characters
- No common patterns or dictionary words
- No personal information (name, email, etc.)

**Example Validation:**
```rust
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 12 {
        return Err("Password must be at least 12 characters".to_string());
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain uppercase letter".to_string());
    }
    
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain lowercase letter".to_string());
    }
    
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain number".to_string());
    }
    
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Password must contain special character".to_string());
    }
    
    Ok(())
}
```

### Hashing Algorithm

**Algorithm: Argon2**
- Resistant to GPU/ASIC attacks
- Memory-hard function
- Configurable time/memory cost

```rust
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;

// Hash password
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

// Verify password
let parsed_hash = PasswordHash::new(&stored_hash)?;
argon2.verify_password(password.as_bytes(), &parsed_hash)?;
```

### Account Lockout

Implement account lockout after failed login attempts:

```rust
pub struct LoginAttempt {
    pub user_id: Uuid,
    pub failed_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

// Lock account after 5 failed attempts for 15 minutes
const MAX_LOGIN_ATTEMPTS: i32 = 5;
const LOCKOUT_DURATION_MINUTES: i64 = 15;

pub async fn check_login_attempts(
    pool: &PgPool,
    email: &str,
) -> Result<bool, Error> {
    let user = sqlx::query!("
        SELECT failed_attempts, locked_until FROM user.users
        WHERE email = $1
    ", email)
    .fetch_optional(pool)
    .await?;
    
    if let Some(user) = user {
        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                return Ok(false); // Account locked
            }
        }
    }
    
    Ok(true)
}
```

---

## Password Security

### Password Reset Flow

**Security Measures:**
1. One-time use tokens
2. 24-hour expiration
3. Secure token generation (96-bit entropy)
4. Invalid tokens don't reveal user existence

```rust
use rand::Rng;

pub fn generate_reset_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn reset_password(
    pool: &PgPool,
    token: &str,
    new_password: &str,
) -> Result<(), Error> {
    // Validate password
    validate_password(new_password)?;
    
    // Find unexpired, unused token
    let token_record = sqlx::query!(
        "SELECT user_id FROM user.password_reset_tokens
         WHERE token = $1 
         AND expires_at > NOW()
         AND used = false",
        token
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::InvalidToken)?;
    
    let password_hash = hash_password(new_password).await?;
    
    // Hash password
    sqlx::query!(
        "UPDATE user.users SET password = $1, updated_at = NOW()
         WHERE id = $2",
        password_hash,
        token_record.user_id
    )
    .execute(pool)
    .await?;
    
    // Mark token as used
    sqlx::query!(
        "UPDATE user.password_reset_tokens SET used = true
         WHERE token = $1",
        token
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
```

### Avoid Common Mistakes

❌ **Don't:**
- Store passwords in plain text
- Use simple hashing (MD5, SHA1, SHA256)
- Implement custom crypto
- Log passwords
- Send passwords via email
- Allow unlimited password reset attempts

✅ **Do:**
- Use Argon2 for hashing
- Use industry-standard libraries
- Rate limit password reset
- Implement account lockout
- Notify via email on resets
- Regenerate sessions after password change

---

## JWT Token Security

### Token Generation

**Token Components:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // User ID
    pub iat: i64,          // Issued at
    pub exp: i64,          // Expiration
    pub roles: Vec<String>, // User roles
    pub permissions: Vec<String>, // User permissions
}

pub fn generate_token(
    user_id: &Uuid,
    roles: Vec<String>,
    secret: &str,
    expires_in_hours: u64,
) -> Result<String, JwtError> {
    let now = Utc::now();
    let expiry = now + Duration::hours(expires_in_hours as i64);
    
    let claims = JwtClaims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: expiry.timestamp(),
        roles,
        permissions: vec![],
    };
    
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
    )?;
    
    Ok(token)
}
```

### Token Validation

```rust
pub fn validate_token(
    token: &str,
    secret: &str,
) -> Result<JwtClaims, JwtError> {
    let token_data = jsonwebtoken::decode::<JwtClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
        &jsonwebtoken::Validation::default(),
    )?;
    
    Ok(token_data.claims)
}
```

### Token Security Best Practices

**Security Measures:**
1. **Short expiration**: 24 hours for access tokens
2. **Refresh token rotation**: Issue new refresh token on each use
3. **Token blacklisting**: Invalidate tokens on logout
4. **Secure storage**: Use httpOnly, secure cookies
5. **HTTPS only**: Never transmit over HTTP
6. **No sensitive data**: Don't include passwords or PII in claims

**Token Storage:**
```rust
// Configuration for secure token storage
pub struct TokenConfig {
    // JWT Secret (min 64 chars)
    pub jwt_secret: String,
    
    // Token expiration
    pub access_token_expiry_hours: u64,
    pub refresh_token_expiry_days: u64,
    
    // Storage
    pub use_secure_cookies: bool,
    pub use_same_site: String,
}

// Recommended .env values
// JWT_SECRET=your-very-long-secure-random-key-min-64-chars
// JWT_EXPIRY_HOURS=24
// JWT_REFRESH_EXPIRY_DAYS=7
```

---

## API Security

### CORS Configuration

```rust
use tower_http::cors::CorsLayer;
use axum::http::{Method, header};

let cors = CorsLayer::new()
    // Specify allowed origins (not wildcard in production)
    .allow_origin("https://example.com".parse()?)
    .allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
    ])
    .allow_headers([
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
    ])
    .allow_credentials()
    .max_age(Duration::from_secs(3600));
```

### Rate Limiting

Implement rate limiting to prevent abuse:

```rust
use tower_governor::Governor;

let auth_limiter = Governor::builder()
    .per_second(10)
    .burst_size(20)
    .finish()
    .unwrap();

// Apply to sensitive endpoints
let auth_routes = Router::new()
    .route("/auth/login", post(login))
    .route("/auth/register", post(register))
    .layer(GovernorLayer {
        state: Box::new(auth_limiter),
        key_extractor: Box::new(|req| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("127.0.0.1")
                .to_string()
        }),
    });
```

### Input Validation

**Always validate user input:**

```rust
use validator::{Validate, ValidationError};

#[derive(Validate, Deserialize)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 255))]
    pub password: String,
}

pub async fn login(
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    req.validate()?; // Returns error if validation fails
    
    // ... rest of handler
}
```

### SQL Injection Prevention

Use parameterized queries with SQLx:

```rust
// ✅ GOOD - Parameterized query
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM user.users WHERE email = $1"
)
.bind(email)
.fetch_optional(&pool)
.await?;

// ❌ BAD - String interpolation (vulnerable)
let query = format!(
    "SELECT * FROM user.users WHERE email = '{}'",
    email  // SQL injection vulnerability!
);
```

---

## Database Security

### Connection Security

```rust
// Use SSL/TLS for database connections
pub async fn init_pool(url: &str) -> Result<PgPool> {
    let connect_options = PgConnectOptions::from_str(url)?
        .ssl_mode(sqlx::postgres::PgSslMode::Require);
    
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect_with(connect_options)
        .await?;
    
    Ok(pool)
}
```

### Database Permissions

Apply principle of least privilege:

```sql
-- Create role for application
CREATE ROLE rustauth WITH PASSWORD 'strong_password';

-- Grant schema permissions
GRANT USAGE ON SCHEMA user, blogs TO rustauth;
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA user TO rustauth;
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA blogs TO rustauth;

-- Deny sensitive operations
REVOKE DELETE ON user.users FROM rustauth;
REVOKE ALL ON postgres FROM rustauth;
```

### Data Encryption

**At Rest:**
```rust
// Use database encryption
// PostgreSQL: pgcrypto extension
CREATE EXTENSION IF NOT EXISTS pgcrypto;

// Store sensitive data encrypted
ALTER TABLE user.users 
ADD COLUMN ssn_encrypted bytea;

INSERT INTO user.users (ssn_encrypted) 
VALUES (pgp_sym_encrypt('123-45-6789', 'encryption_key'));

SELECT pgp_sym_decrypt(ssn_encrypted, 'encryption_key') FROM user.users;
```

**In Transit:**
- Always use HTTPS
- Use SSL/TLS for database connections
- Use VPN for administrative access

---

## Deployment Security

### Environment Variables

**Never commit secrets:**

```bash
# .env (local development - use .env.example for git)
DATABASE_URL=postgres://user:password@localhost:5432/auth_dev
JWT_SECRET=your-secret-key  # Min 64 characters
REDIS_URL=redis://127.0.0.1:6379

# Production (set via environment)
export DATABASE_URL=postgres://...
export JWT_SECRET=...
```

### Docker Security

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Production image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Don't run as root
RUN useradd -m -u 1001 appuser
USER appuser

COPY --from=builder /app/target/release/rustauth /usr/local/bin/
EXPOSE 8000
CMD ["rustauth"]
```

### Secrets Management

**Use secrets management tools:**

```bash
# Kubernetes Secrets
kubectl create secret generic rustauth \
  --from-literal=database-url="postgres://..." \
  --from-literal=jwt-secret="..."

# AWS Secrets Manager
aws secretsmanager create-secret \
  --name rustauth/prod \
  --secret-string file://secrets.json

# HashiCorp Vault
vault write secret/rustauth/prod \
  database_url="..." \
  jwt_secret="..."
```

---

## Data Protection

### PII (Personally Identifiable Information)

**Sensitive Fields:**
- Email addresses
- Phone numbers
- Passwords
- IP addresses
- Location data

**Protection Measures:**
1. Encrypt at rest
2. Hash where possible (emails)
3. Minimize collection
4. Secure deletion
5. Access logging

### GDPR Compliance

```rust
// Right to be forgotten
pub async fn delete_user_data(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<()> {
    // Delete user personal data
    sqlx::query!("
        UPDATE user.users 
        SET 
            email = 'deleted-' || gen_random_uuid(),
            password = '',
            full_name = NULL,
            phone_number = NULL,
            avatar_url = NULL,
            is_active = false
        WHERE id = $1
    ", user_id)
    .execute(pool)
    .await?;
    
    // Delete related data
    sqlx::query!("DELETE FROM user.refresh_tokens WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;
    
    Ok(())
}
```

---

## Security Checklist

### Pre-Deployment Checklist

- [ ] All secrets in environment variables (not code)
- [ ] JWT_SECRET is strong (min 64 chars)
- [ ] HTTPS/TLS enabled
- [ ] Database uses SSL/TLS
- [ ] Firewall configured
- [ ] Rate limiting enabled
- [ ] Input validation on all endpoints
- [ ] CORS properly configured (not wildcard)
- [ ] Logging doesn't contain passwords/tokens
- [ ] Secrets not in Git history
- [ ] Database backups encrypted
- [ ] Admin credentials changed
- [ ] Security headers configured
- [ ] Tests include security checks

### Runtime Security Monitoring

- [ ] Log all authentication attempts
- [ ] Monitor failed login attempts
- [ ] Alert on suspicious activity
- [ ] Track API endpoint usage
- [ ] Monitor database queries
- [ ] Regular security audits

---

For more information:
- [DEPLOYMENT.md](DEPLOYMENT.md) - Deployment guide
- [API.md](API.md) - API security
- [DATABASE.md](DATABASE.md) - Database security
