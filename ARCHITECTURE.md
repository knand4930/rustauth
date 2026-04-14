# System Architecture

A comprehensive guide to the RustAuth system architecture, design patterns, and component interactions.

## 📋 Table of Contents

- [High-Level Overview](#high-level-overview)
- [Architecture Layers](#architecture-layers)
- [Component Interaction](#component-interaction)
- [Request Flow](#request-flow)
- [Database Architecture](#database-architecture)
- [Authentication Flow](#authentication-flow)
- [Error Handling](#error-handling)
- [Scalability Considerations](#scalability-considerations)

---

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Applications                      │
│              (Web, Mobile, Desktop Clients)                  │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTP/HTTPS
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum Web Server                           │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Router & Middleware Layer                   │ │
│  │  ┌──────────────────────────────────────────────────┐   │ │
│  │  │ CORS │ Auth │ Logging │ Error Handling │ Tracing │   │ │
│  │  └──────────────────────────────────────────────────┘   │ │
│  └─────────────────────────────────────────────────────────┘ │
│                       │                                       │
│  ┌────────────────────┼────────────────────┐                 │
│  │                    │                    │                 │
│  ▼                    ▼                    ▼                 │
│ ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│ │  Auth Handler  │  │  User Handler  │  │  Blog Handler  │ │
│ └────────────────┘  └────────────────┘  └────────────────┘ │
│         │                    │                    │          │
└─────────┼────────────────────┼────────────────────┼──────────┘
          │                    │                    │
          │    ┌───────────────┴────────────────┐   │
          │    │                                │   │
          ▼    ▼                                ▼   ▼
    ┌─────────────────────────────────────────────────────┐
    │        Application State (AppState)                 │
    │  ┌────────────────┐  ┌──────────────────────────┐  │
    │  │ PgPool (DB)    │  │ Config & Redis Client    │  │
    │  └────────────────┘  └──────────────────────────┘  │
    └─────────────────────────────────────────────────────┘
          │                              │
    ┌─────▼──────────┐            ┌─────▼──────────┐
    │  PostgreSQL    │            │  Redis Cache   │
    │  Database      │            │  & Sessions    │
    └────────────────┘            └────────────────┘
```

---

## Architecture Layers

### 1. **Presentation Layer** (HTTP/API)

Handles incoming HTTP requests and outgoing responses.

**Components:**
- **Axum Router** - Request routing and method dispatch
- **Middleware Stack** - CORS, authentication, logging
- **Error Handlers** - HTTP status code mapping
- **Response Serialization** - JSON/error responses

**Entry Point:**
```rust
// src/main.rs
#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/api/auth/login", post(handlers::login))
        .route("/api/users/:id", get(handlers::get_user))
        .nest("/api/blogs", blogs_routes)
        .fallback(handlers::not_found)
        .layer(middleware_layer)
        .with_state(state);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
}
```

### 2. **Business Logic Layer** (Handlers)

Implements business rules and orchestrates service operations.

**Handler Structure:**
```rust
// src/apps/user/handlers.rs
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    // 1. Validate input
    // 2. Query user from database
    // 3. Verify password
    // 4. Generate JWT tokens
    // 5. Store session
    // 6. Return response
}
```

**Key Responsibilities:**
- Request validation
- Authentication & authorization
- Business logic orchestration
- Service coordination
- Response construction

### 3. **Data Access Layer** (Models & Database)

Direct database interaction using SQLx.

**Database Functions:**
```rust
// Query user by email
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM user.users WHERE email = $1"
)
.bind(email)
.fetch_optional(&state.db)
.await?;

// Create new user
let user = sqlx::query_as::<_, User>(
    "INSERT INTO user.users (email, password) VALUES ($1, $2) 
     RETURNING *"
)
.bind(email)
.bind(hashed_password)
.fetch_one(&state.db)
.await?;
```

**Model Files:**
- `src/apps/user/models.rs` - User, Role, Permission models
- `src/apps/blogs/models.rs` - BlogPost, Comment models

### 4. **Infrastructure Layer** (Configuration, Logging, Caching)

Manages external services and system resources.

**Components:**
- **Database Pool** - PostgreSQL connection management
- **Redis Client** - Session & cache management
- **Logger** - Structured logging with tracing
- **Configuration** - Environment variable management

**Initialization:**
```rust
// src/main.rs
let pool = db::init_pool(&config.database_url).await;
let redis_client = redis::Client::open(config.redis_url)?;
tracing_subscriber::registry()
    .with(EnvFilter::new(&config.rust_log))
    .init();
```

---

## Component Interaction

### Module Organization

```
src/apps/
├── user/                    ← User Management Module
│   ├── mod.rs              (exports + routing)
│   ├── models.rs           (User, Role, Permission, etc.)
│   ├── schemas.rs          (DTO - request/response)
│   ├── handlers.rs         (HTTP handlers + business logic)
│   ├── admin_config.rs     (Admin dashboard metadata)
│   └── admin_registry.rs   (Admin panel registration)
│
└── blogs/                   ← Blog Management Module
    ├── mod.rs
    ├── models.rs           (BlogPost, Comment)
    ├── schemas.rs          (DTOs)
    ├── handlers.rs         (Handlers)
    ├── admin_config.rs
    └── admin_registry.rs
```

### Data Flow Example: User Login

```
1. HTTP Request
   POST /api/auth/login
   {
     "email": "user@example.com",
     "password": "password123"
   }
   │
   ▼
2. Axum Router
   Routes to: handlers::auth::login
   │
   ▼
3. Handler Layer
   - Parse JSON request → LoginRequest DTO
   - Validate input
   │
   ▼
4. Database Query
   SELECT * FROM user.users WHERE email = ?
   │
   ▼
5. Password Verification
   argon2::verify_password()
   │
   ▼
6. Token Generation
   JWT access token + refresh token
   │
   ▼
7. Session Storage
   Redis: SET user_id:session_token ...
   │
   ▼
8. Response
   JSON:
   {
     "access_token": "...",
     "refresh_token": "...",
     "user": { ... }
   }
```

---

## Request Flow

### Complete Request Processing

```
┌───────────────────────────────────────────────────────────┐
│ 1. HTTP Request Received                                  │
│    GET /api/users/123                                      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌───────────────────────────────────────────────────────────┐
│ 2. Middleware Stack (Top-Down)                            │
│    ├─ Logger: Log request                                 │
│    ├─ CORS: Validate origin                               │
│    ├─ Auth: Validate JWT token                            │
│    └─ Tracing: Start trace span                           │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌───────────────────────────────────────────────────────────┐
│ 3. Router Dispatch                                        │
│    Match: GET /api/users/{id}                             │
│    → handlers::get_user                                   │
│    Extract: Path(id: Uuid), State(state), Auth(user)      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌───────────────────────────────────────────────────────────┐
│ 4. Handler Execution                                      │
│    pub async fn get_user(                                 │
│        Path(id): Path<Uuid>,                              │
│        State(state): State<Arc<AppState>>,                │
│        auth: JwtAuth,                                     │
│    ) -> Result<Json<UserResponse>> {                      │
│        // Validate authorization                          │
│        // Query database                                  │
│        // Transform model to response                     │
│    }                                                      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌───────────────────────────────────────────────────────────┐
│ 5. Response Processing                                    │
│    ├─ Serialize to JSON                                   │
│    ├─ Add response headers                                │
│    └─ Apply middleware response handlers                  │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌───────────────────────────────────────────────────────────┐
│ 6. HTTP Response Sent                                     │
│    200 OK                                                 │
│    Content-Type: application/json                         │
│    { "id": "...", "email": "...", ... }                  │
└───────────────────────────────────────────────────────────┘
```

---

## Database Architecture

### Schema Design

```
┌─────────────────────────────────────────────────────────────┐
│           PostgreSQL Database: auth_dev                      │
├──────────────────┬──────────────────┬───────────────────────┤
│  user schema     │  blogs schema     │  public schema        │
├──────────────────┼──────────────────┼───────────────────────┤
│                  │                  │                       │
│ users            │ blog_posts       │ migrations (tracking) │
│ ├─ id (PK)       │ ├─ id (PK)       │                       │
│ ├─ email (UQ)    │ ├─ title         │ schema_version        │
│ ├─ password      │ ├─ content       │ └─ version            │
│ ├─ full_name     │ ├─ author_id(FK) │                       │
│ ├─ is_active     │ ├─ status        │                       │
│ ├─ created_at    │ └─ created_at    │                       │
│ └─ updated_at    │                  │                       │
│                  │ comments         │                       │
│ roles            │ ├─ id (PK)       │                       │
│ ├─ id (PK)       │ ├─ content       │                       │
│ ├─ name (UQ)     │ ├─ user_id (FK)  │                       │
│ └─ permissions   │ ├─ post_id (FK)  │                       │
│                  │ └─ created_at    │                       │
│ permissions      │                  │                       │
│ ├─ id (PK)       │                  │                       │
│ └─ name          │                  │                       │
│                  │                  │                       │
│ refresh_tokens   │                  │                       │
│ access_tokens    │                  │                       │
│ token_blacklist  │                  │                       │
│ password_reset   │                  │                       │
│ user_sessions    │                  │                       │
│ user_roles       │                  │                       │
│ role_permissions │                  │                       │
│                  │                  │                       │
└──────────────────┴──────────────────┴───────────────────────┘
```

### Relationships

**User ↔ RefreshToken** (1:N)
```sql
ALTER TABLE user.refresh_tokens
ADD CONSTRAINT fk_user_refresh_tokens
FOREIGN KEY (user_id) REFERENCES user.users(id);
```

**User ↔ UserRole** (1:N)
```sql
ALTER TABLE user.user_roles
ADD CONSTRAINT fk_user_roles
FOREIGN KEY (user_id) REFERENCES user.users(id);
```

**Role ↔ Permission** (N:N via RolePermission)
```sql
ALTER TABLE user.role_permissions
ADD CONSTRAINT fk_role_permissions
FOREIGN KEY (role_id) REFERENCES user.roles(id);

ALTER TABLE user.role_permissions
ADD CONSTRAINT fk_permission_roles
FOREIGN KEY (permission_id) REFERENCES user.permissions(id);
```

**User ↔ BlogPost** (1:N)
```sql
ALTER TABLE blogs.blog_posts
ADD CONSTRAINT fk_blogpost_user
FOREIGN KEY (author_id) REFERENCES user.users(id);
```

---

## Authentication Flow

### JWT Token Architecture

```
┌─────────────────────────────────────┐
│   JWT Access Token (24 hours)       │
├─────────────────────────────────────┤
│ Header:                             │
│ {                                   │
│   "alg": "HS256",                   │
│   "typ": "JWT"                      │
│ }                                   │
├─────────────────────────────────────┤
│ Payload:                            │
│ {                                   │
│   "sub": "user-uuid",               │
│   "exp": 1234567890,                │
│   "iat": 1234567890,                │
│   "roles": ["user", "admin"],       │
│   "permissions": [...]              │
│ }                                   │
├─────────────────────────────────────┤
│ Signature: HMAC-SHA256              │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ JWT Refresh Token (7 days)          │
├─────────────────────────────────────┤
│ Purpose: Obtain new access token    │
│ Stored: Database + Redis            │
│ Rotation: Every refresh             │
└─────────────────────────────────────┘
```

### Login Sequence

```
1. Client submits credentials
   POST /api/auth/login
   { "email": "...", "password": "..." }

2. Server validates input
   - Check email format
   - Check password length
   
3. Query user from database
   SELECT * FROM user.users WHERE email = ?
   
4. Verify password
   argon2::verify(stored_hash, provided_password)
   
5. Generate tokens
   - Access token (24h): includes roles, permissions
   - Refresh token (7d): used to get new access token
   
6. Store session
   Redis: SET session:{token_id} { user_id, expires_at }
   
7. Return response
   {
     "access_token": "...",
     "refresh_token": "...",
     "user": { id, email, roles }
   }
```

### Protected Request Flow

```
1. Client includes JWT in Authorization header
   GET /api/users/me
   Authorization: Bearer eyJhbGc...

2. Middleware extracts token
   Extract "Bearer <token>" from header

3. Verify token signature
   Validate HMAC with JWT_SECRET

4. Check token expiration
   Verify exp < current_time

5. Extract user information
   Parse sub (user_id), roles, permissions

6. Optional: Check token blacklist
   Query Redis for blacklisted tokens

7. Inject AuthContext into handler
   Let handler use extracted user info

8. Handler may check specific permissions
   - User can only access own data
   - Admin can access all data
```

---

## Error Handling

### Error Hierarchy

```
ApiError (Custom Enum)
├── Unauthorized
│   └── HTTP 401
├── Forbidden
│   └── HTTP 403
├── UserNotFound
│   └── HTTP 404
├── ValidationError(String)
│   └── HTTP 400
├── DatabaseError
│   └── HTTP 500
└── InternalError
    └── HTTP 500
```

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Email or password is incorrect",
    "details": null
  }
}
```

### Error Handling in Handlers

```rust
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Validation errors
    req.validate()?; // Returns BadRequest
    
    // Not found errors
    let user = User::find_by_email(&state.db, &req.email)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;
    
    // Authorization errors
    if !user.is_active {
        return Err(ApiError::Unauthorized);
    }
    
    // Business logic errors
    if !verify_password(&req.password, &user.password)? {
        return Err(ApiError::InvalidCredentials);
    }
    
    // Success
    Ok(Json(response))
}
```

---

## Scalability Considerations

### Database Optimization

1. **Connection Pooling**
   - SQLx: 10 connections default
   - Adjust based on load

2. **Query Optimization**
   - Index frequently queried columns (email, user_id)
   - Use pagination for large datasets
   - Query compilation with SQLx

3. **Caching Strategy**
   - Redis for user sessions
   - Redis for role/permission caching
   - Cache invalidation on updates

### Horizontal Scaling

```
┌─────────────────────────────────────────┐
│         Load Balancer (Nginx)           │
└──────────────┬──────┬──────┬───────────┘
               │      │      │
        ┌──────▼──┐ ┌──▼────┐ ┌───▼────┐
        │Instance │ │Instance│ │Instance│
        │    1    │ │   2    │ │   3    │
        └──────┬──┘ └──┬────┘ └───┬────┘
               │       │          │
               └───────┼──────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
    ┌───▼──┐      ┌───▼──┐      ┌───▼──┐
    │  DB  │      │Redis │      │  S3  │
    └──────┘      └──────┘      └──────┘
```

### Async Performance

- Tokio runtime: Handle thousands of concurrent connections
- Non-blocking database queries with SQLx
- Streaming responses for large datasets

### Monitoring & Observability

- **Tracing**: Distributed request tracing
- **Metrics**: Request count, latency, errors
- **Logging**: Structured logs for debugging

---

## Summary

RustAuth uses a **layered architecture** with clear separation of concerns:

1. **Presentation Layer**: HTTP handling with Axum
2. **Business Logic Layer**: Handlers with validation and orchestration
3. **Data Access Layer**: SQLx for type-safe database queries
4. **Infrastructure Layer**: Database pools, caching, logging

This design enables:
- ✅ Scalability through async/await and connection pooling
- ✅ Maintainability through modular organization
- ✅ Reliability through type safety and compile-time checks
- ✅ Performance through optimized queries and caching

For more details on specific components, see:
- [API.md](API.md) - API endpoints
- [DATABASE.md](DATABASE.md) - Database schema
- [development.md](development.md) - Development guide
