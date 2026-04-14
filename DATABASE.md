# Database Schema & Models

Complete reference for the RustAuth database schema, models, relationships, and migration system.

## 📋 Table of Contents

- [Database Overview](#database-overview)
- [Schemas](#schemas)
- [User Models](#user-models)
- [Blog Models](#blog-models)
- [Relationships](#relationships)
- [Indexes](#indexes)
- [Migrations](#migrations)
- [Database Operations](#database-operations)
- [Data Types](#data-types)

---

## Database Overview

### Database Statistics

| Metric | Value |
|--------|-------|
| **Database Type** | PostgreSQL 12+ |
| **Total Schemas** | 2 (user, blogs) |
| **Total Tables** | 12 |
| **Total Models** | 12 |
| **Relationships** | 15+ foreign keys |

### Connection Details

```env
DATABASE_URL=postgres://user:password@localhost:5432/auth_dev
```

### Query Statistics

```rust
// Connection pool configuration
sqlx::postgres::PgPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&url)
    .await?
```

---

## Schemas

### `public` Schema

Default PostgreSQL schema.

**Tables:**
- `schema_version` - Migration tracking

### `user` Schema

User authentication and access control.

**Tables:**
- `users` - User accounts
- `roles` - User roles
- `permissions` - System permissions
- `user_roles` - User-role assignments
- `role_permissions` - Role-permission mappings
- `refresh_tokens` - OAuth refresh tokens
- `access_tokens` - OAuth access tokens
- `token_blacklist` - Revoked tokens
- `password_reset_tokens` - Password reset tokens
- `user_sessions` - Active user sessions

### `blogs` Schema

Blog content management.

**Tables:**
- `blog_posts` - Blog articles
- `comments` - Post comments

---

## User Models

### User (user.users)

Core user account information.

```sql
CREATE TABLE user.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    store_password VARCHAR(255),
    full_name VARCHAR(255),
    details TEXT,
    company VARCHAR(255),
    avatar_url VARCHAR(255),
    phone_number VARCHAR(20),
    timezone VARCHAR(50) DEFAULT 'UTC',
    language VARCHAR(10) DEFAULT 'en',
    salt VARCHAR(255),
    location VARCHAR(255),
    ipaddress INET,
    is_active BOOLEAN DEFAULT true,
    is_superuser BOOLEAN DEFAULT false,
    is_staffuser BOOLEAN DEFAULT false,
    is_guest BOOLEAN DEFAULT false,
    email_verified BOOLEAN DEFAULT false,
    phone_verified BOOLEAN DEFAULT false,
    mfa_enabled BOOLEAN DEFAULT false,
    mfa_secret VARCHAR(255),
    backup_codes TEXT[],
    preferences JSONB DEFAULT '{}',
    last_login_at TIMESTAMPTZ,
    last_login_ip INET,
    login_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_user_email ON user.users(email);
CREATE INDEX idx_user_active ON user.users(is_active);
CREATE INDEX idx_user_created_at ON user.users(created_at DESC);
```

**Fields:**
| Field | Type | Constraints | Purpose |
|-------|------|-----------|---------|
| id | UUID | PK | Unique user identifier |
| email | VARCHAR(255) | UNIQUE, NOT NULL | User email address |
| password | VARCHAR(255) | NOT NULL | Hashed password (Argon2) |
| store_password | VARCHAR(255) | | Alternative password storage |
| full_name | VARCHAR(255) | | User's full name |
| details | TEXT | | User bio/details |
| company | VARCHAR(255) | | Company name |
| avatar_url | VARCHAR(255) | | Profile picture URL |
| phone_number | VARCHAR(20) | | Phone number |
| timezone | VARCHAR(50) | DEFAULT 'UTC' | User's timezone |
| language | VARCHAR(10) | DEFAULT 'en' | Preferred language |
| is_active | BOOLEAN | DEFAULT true | Account status |
| is_superuser | BOOLEAN | DEFAULT false | Admin flag |
| email_verified | BOOLEAN | DEFAULT false | Email verification status |
| mfa_enabled | BOOLEAN | DEFAULT false | Multi-factor auth enabled |
| last_login_at | TIMESTAMPTZ | | Last login timestamp |
| login_count | INTEGER | DEFAULT 0 | Total login count |
| created_at | TIMESTAMPTZ | DEFAULT NOW() | Account creation time |
| updated_at | TIMESTAMPTZ | DEFAULT NOW() | Last update time |

**Rust Model:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub password: Option<String>,
    pub store_password: Option<String>,
    pub full_name: Option<String>,
    pub details: Option<String>,
    pub company: Option<String>,
    pub avatar_url: Option<String>,
    pub phone_number: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub salt: Option<String>,
    pub location: Option<String>,
    pub ipaddress: Option<String>,
    pub is_active: Option<bool>,
    pub is_superuser: Option<bool>,
    pub is_staffuser: Option<bool>,
    pub is_guest: Option<bool>,
    pub email_verified: Option<bool>,
    pub phone_verified: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub mfa_secret: Option<String>,
    pub backup_codes: Option<Vec<String>>,
    pub preferences: Option<serde_json::Value>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub login_count: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

---

### Role (user.roles)

User roles for access control.

```sql
CREATE TABLE user.roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

**Default Roles:**
- `admin` - Full system access
- `moderator` - Content moderation
- `user` - Standard user access
- `guest` - Limited read-only access

---

### Permission (user.permissions)

System permissions.

```sql
CREATE TABLE user.permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

**Default Permissions:**
- `users.read` - View users
- `users.write` - Create/edit users
- `users.delete` - Delete users
- `blogs.read` - Read blog posts
- `blogs.write` - Create/edit posts
- `blogs.delete` - Delete posts
- `admin.access` - Access admin panel

---

### RefreshToken (user.refresh_tokens)

OAuth refresh tokens for long-lived sessions.

```sql
CREATE TABLE user.refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    token VARCHAR(500) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_refresh_token_user_id ON user.refresh_tokens(user_id);
CREATE INDEX idx_refresh_token_expires_at ON user.refresh_tokens(expires_at);
```

**Retention:** 7 days (rotated on each refresh)

---

### AccessToken (user.access_tokens)

OAuth access tokens.

```sql
CREATE TABLE user.access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    refresh_token_id UUID REFERENCES user.refresh_tokens(id),
    token VARCHAR(500) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_access_token_user_id ON user.access_tokens(user_id);
```

**Retention:** 24 hours

---

### TokenBlacklist (user.token_blacklist)

Revoked tokens (logout/invalidation).

```sql
CREATE TABLE user.token_blacklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token VARCHAR(500) NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES user.users(id),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_blacklist_expires_at ON user.token_blacklist(expires_at);
```

---

### PasswordResetToken (user.password_reset_tokens)

Password reset token storage.

```sql
CREATE TABLE user.password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    token VARCHAR(500) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_password_reset_user_id ON user.password_reset_tokens(user_id);
```

**Retention:** 24 hours (1-use only)

---

### UserSession (user.user_sessions)

Active user sessions with device tracking.

```sql
CREATE TABLE user.user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    device_name VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    last_activity TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_session_user_id ON user.user_sessions(user_id);
```

---

### UserRole (user.user_roles)

User-role assignment (N:N relationship).

```sql
CREATE TABLE user.user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user.users(id),
    role_id UUID NOT NULL REFERENCES user.roles(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, role_id)
);

CREATE INDEX idx_user_roles_user_id ON user.user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user.user_roles(role_id);
```

---

### RolePermission (user.role_permissions)

Role-permission assignment (N:N relationship).

```sql
CREATE TABLE user.role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES user.roles(id),
    permission_id UUID NOT NULL REFERENCES user.permissions(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role_id ON user.role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission_id ON user.role_permissions(permission_id);
```

---

## Blog Models

### BlogPost (blogs.blog_posts)

Blog article content.

```sql
CREATE TABLE blogs.blog_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    excerpt TEXT,
    content TEXT NOT NULL,
    author_id UUID NOT NULL REFERENCES user.users(id),
    status VARCHAR(20) DEFAULT 'draft', -- 'draft' or 'published'
    views INTEGER DEFAULT 0,
    featured BOOLEAN DEFAULT false,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_blog_post_slug ON blogs.blog_posts(slug);
CREATE INDEX idx_blog_post_author_id ON blogs.blog_posts(author_id);
CREATE INDEX idx_blog_post_status ON blogs.blog_posts(status);
CREATE INDEX idx_blog_post_published_at ON blogs.blog_posts(published_at DESC);
```

**Fields:**
| Field | Type | Purpose |
|-------|------|---------|
| id | UUID | Unique post ID |
| title | VARCHAR(255) | Post title |
| slug | VARCHAR(255) | URL-friendly slug |
| excerpt | TEXT | Short summary |
| content | TEXT | Full content (Markdown) |
| author_id | UUID | Author reference |
| status | VARCHAR(20) | draft/published status |
| views | INTEGER | View count |
| featured | BOOLEAN | Featured post flag |
| published_at | TIMESTAMPTZ | Publication timestamp |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update time |

---

### Comment (blogs.comments)

Blog post comments.

```sql
CREATE TABLE blogs.comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES user.users(id),
    blog_post_id UUID NOT NULL REFERENCES blogs.blog_posts(id),
    likes INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_comment_blog_post_id ON blogs.comments(blog_post_id);
CREATE INDEX idx_comment_user_id ON blogs.comments(user_id);
CREATE INDEX idx_comment_created_at ON blogs.comments(created_at DESC);
```

**Fields:**
| Field | Type | Purpose |
|-------|------|---------|
| id | UUID | Unique comment ID |
| content | TEXT | Comment text |
| user_id | UUID | Commenter reference |
| blog_post_id | UUID | Blog post reference |
| likes | INTEGER | Like count |
| created_at | TIMESTAMPTZ | Creation time |
| updated_at | TIMESTAMPTZ | Update time |

---

## Relationships

### Entity Relationship Diagram

```
┌─────────────────────┐
│  user.users         │
│  ───────────────────│
│  id (PK)            │
│  email (UNIQUE)     │
│  password           │
│  full_name          │
│  is_active          │
│  created_at         │
└────────┬────────────┘
         │
    ┌────┴──────────────────────┬──────────────┬─────────────────┐
    │                           │              │                 │
    │ 1:N                   1:N │          1:N │             1:N │
    │                           │              │                 │
    ▼                           ▼              ▼                 ▼
┌──────────────────┐ ┌──────────────────┐ ┌─────────┐ ┌──────────────────┐
│ user_roles       │ │ refresh_tokens   │ │ comments│ │ blog_posts       │
│ ──────────────   │ │ ────────────────│ │ ────────│ │ ──────────────   │
│ user_id (FK)───┐ │ │ user_id (FK) ──┐│ │user_id ││ │ author_id (FK)─┐ │
│ role_id (FK)  │ │ │ token           ││ │(FK)    ││ │ title          │ │
│ created_at    │ │ │ expires_at      ││ │content ││ │ content        │ │
│ ──────────────│ │ │ created_at      ││ │created ││ │ status         │ │
└──────┬────────┘ │ └──────────────┬─┘│ │_at     ││ │ published_at   │ │
       │          │                │  └─────────┘│ └────────────┬───┘ │
       │ N:1      │ 1:N            │                            │      │
       ▼          │                │                    1:N      │      │
    ┌──────────────────┐           │               ┌────────────▼─────┐
    │ user.roles       │           │               │ blogs.comments   │
    │ ────────────────│           │               │ ─────────────────│
    │ id (PK)         │           │               │ id (PK)          │
    │ name (UNIQUE)   │           ▼               │ blog_post_id(FK)─┘
    │ description     │       ┌──────────────┐    │ user_id (FK)
    └────────┬────────┘       │ permissions  │    │ content
             │                │ ────────────│    │ created_at
    N:N      │                │ id (PK)     │    └──────────────────┘
             ▼                │ name        │
    ┌──────────────────┐       └─────┬──────┘
    │role_permissions  │             │
    │ ──────────────   │             │ N:1
    │ role_id (FK) ────┤             │
    │ permission_id(FK)┤─────────────┘
    │ created_at       │
    └──────────────────┘
```

### Foreign Key Constraints

```sql
-- User relationships
ALTER TABLE user.refresh_tokens 
ADD CONSTRAINT fk_ref_token_user 
FOREIGN KEY (user_id) REFERENCES user.users(id) ON DELETE CASCADE;

ALTER TABLE user.access_tokens 
ADD CONSTRAINT fk_access_token_user 
FOREIGN KEY (user_id) REFERENCES user.users(id) ON DELETE CASCADE;

ALTER TABLE user.user_roles 
ADD CONSTRAINT fk_user_role_user 
FOREIGN KEY (user_id) REFERENCES user.users(id) ON DELETE CASCADE;

ALTER TABLE user.user_roles 
ADD CONSTRAINT fk_user_role_role 
FOREIGN KEY (role_id) REFERENCES user.roles(id) ON DELETE CASCADE;

ALTER TABLE user.role_permissions 
ADD CONSTRAINT fk_role_perm_role 
FOREIGN KEY (role_id) REFERENCES user.roles(id) ON DELETE CASCADE;

ALTER TABLE user.role_permissions 
ADD CONSTRAINT fk_role_perm_permission 
FOREIGN KEY (permission_id) REFERENCES user.permissions(id) ON DELETE CASCADE;

-- Blog relationships
ALTER TABLE blogs.blog_posts 
ADD CONSTRAINT fk_blog_post_author 
FOREIGN KEY (author_id) REFERENCES user.users(id) ON DELETE CASCADE;

ALTER TABLE blogs.comments 
ADD CONSTRAINT fk_comment_user 
FOREIGN KEY (user_id) REFERENCES user.users(id) ON DELETE CASCADE;

ALTER TABLE blogs.comments 
ADD CONSTRAINT fk_comment_post 
FOREIGN KEY (blog_post_id) REFERENCES blogs.blog_posts(id) ON DELETE CASCADE;
```

---

## Indexes

### Performance Indexes

**User Table:**
```sql
CREATE UNIQUE INDEX idx_user_email ON user.users(email);
CREATE INDEX idx_user_active ON user.users(is_active);
CREATE INDEX idx_user_created_at ON user.users(created_at DESC);
CREATE INDEX idx_user_superuser ON user.users(is_superuser) WHERE is_superuser = true;
```

**Blog Table:**
```sql
CREATE UNIQUE INDEX idx_blog_slug ON blogs.blog_posts(slug);
CREATE INDEX idx_blog_author ON blogs.blog_posts(author_id);
CREATE INDEX idx_blog_status ON blogs.blog_posts(status);
CREATE INDEX idx_blog_published ON blogs.blog_posts(published_at DESC) 
WHERE status = 'published';
```

**Token Tables:**
```sql
CREATE INDEX idx_refresh_token_user ON user.refresh_tokens(user_id);
CREATE INDEX idx_access_token_expires ON user.access_tokens(expires_at);
CREATE INDEX idx_blacklist_expires ON user.token_blacklist(expires_at);
```

---

## Migrations

### Migration System

Migrations are stored in the `migrations/` directory:

```
migrations/
├── 20260414110602_auto/
│   ├── up.sql      (Create/modify schema)
│   └── down.sql    (Rollback changes)
└── 20260414115951_auto/
    ├── up.sql
    └── down.sql
```

### Running Migrations

```bash
# Apply all pending migrations
cargo run --bin migrate

# Check migration status
cargo run --bin showmigrations

# Generate new migration from models
cargo makemigrations

# Revert last migration
cargo run --bin migrate down
```

---

## Database Operations

### Common Queries

**Get user by email:**
```rust
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM user.users WHERE email = $1"
)
.bind(email)
.fetch_optional(&pool)
.await?;
```

**Get user with roles:**
```rust
let user_roles = sqlx::query!(
    "SELECT ur.*, r.name FROM user.user_roles ur
     JOIN user.roles r ON ur.role_id = r.id
     WHERE ur.user_id = $1",
    user_id
)
.fetch_all(&pool)
.await?;
```

**Get published blog posts with pagination:**
```rust
let posts = sqlx::query_as::<_, BlogPost>(
    "SELECT * FROM blogs.blog_posts
     WHERE status = 'published'
     ORDER BY published_at DESC
     LIMIT $1 OFFSET $2"
)
.bind(limit)
.bind((page - 1) * limit)
.fetch_all(&pool)
.await?;
```

---

## Data Types

### PostgreSQL to Rust Type Mapping

| PostgreSQL | Rust | SQLx Feature |
|-----------|------|------------|
| UUID | `uuid::Uuid` | uuid |
| VARCHAR | `String` | - |
| TEXT | `String` | - |
| BOOLEAN | `bool` | - |
| INTEGER | `i32` | - |
| BIGINT | `i64` | - |
| TIMESTAMPTZ | `chrono::DateTime<Utc>` | chrono |
| JSONB | `serde_json::Value` | - |
| TEXT[] | `Vec<String>` | - |
| INET | `String` | - |

---

## Backup & Recovery

### Backup Database

```bash
# Full backup
pg_dump auth_dev > backup.sql

# Compressed backup
pg_dump -Fc auth_dev > backup.dump

# Backup specific table
pg_dump -t user.users auth_dev > users_backup.sql
```

### Restore Database

```bash
# Restore from SQL
psql auth_dev < backup.sql

# Restore from compressed
pg_restore -d auth_dev backup.dump

# Restore specific table
psql auth_dev < users_backup.sql
```

---

For more information:
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [API.md](API.md) - API endpoints
- [commands.md](commands.md) - CLI commands
