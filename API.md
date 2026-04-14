# API Reference

Complete API documentation for RustAuth endpoints, request/response formats, and error codes.

## 📋 Table of Contents

- [Base URL & Authentication](#base-url--authentication)
- [Authentication Endpoints](#authentication-endpoints)
- [User Endpoints](#user-endpoints)
- [Blog Endpoints](#blog-endpoints)
- [Error Codes](#error-codes)
- [Rate Limiting](#rate-limiting)
- [Examples](#examples)

---

## Base URL & Authentication

### Base URL

```
http://localhost:8000/api
```

### Authentication

All protected endpoints require JWT token in Authorization header:

```
Authorization: Bearer <access_token>
```

### Headers

**Required:**
```
Content-Type: application/json
```

**Optional:**
```
X-Request-ID: unique-request-identifier  (for tracing)
```

---

## Authentication Endpoints

### Register User

Create a new user account.

```http
POST /auth/register
```

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "full_name": "John Doe"
}
```

**Response:** `201 Created`
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "full_name": "John Doe",
    "created_at": "2026-04-14T10:30:00Z"
  },
  "message": "User registered successfully"
}
```

**Validation Rules:**
- Email: Valid email format, unique
- Password: Min 8 chars, uppercase, lowercase, number, special char
- Full name: 2-100 characters

**Error Responses:**
- `400 Bad Request` - Validation failed
- `409 Conflict` - Email already exists

---

### Login

Authenticate user and get tokens.

```http
POST /auth/login
```

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Response:** `200 OK`
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "full_name": "John Doe",
    "roles": ["user"]
  }
}
```

**Token Details:**
- Access token: Valid for 24 hours
- Refresh token: Valid for 7 days
- Use refresh token to get new access token before expiry

**Error Responses:**
- `401 Unauthorized` - Invalid credentials
- `403 Forbidden` - Account inactive

---

### Refresh Token

Get new access token using refresh token.

```http
POST /auth/refresh
```

**Request:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response:** `200 OK`
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

**Error Responses:**
- `401 Unauthorized` - Invalid or expired refresh token
- `400 Bad Request` - Malformed token

---

### Logout

Invalidate current token and logout user.

```http
POST /auth/logout
```

**Request:**
```
Authorization: Bearer <access_token>
```

**Response:** `200 OK`
```json
{
  "message": "Logged out successfully"
}
```

**Error Responses:**
- `401 Unauthorized` - Invalid or expired token

---

### Forgot Password

Request password reset token via email.

```http
POST /auth/forgot-password
```

**Request:**
```json
{
  "email": "user@example.com"
}
```

**Response:** `200 OK`
```json
{
  "message": "Password reset email sent",
  "email": "user@example.com"
}
```

**Note:** Returns success even if email doesn't exist (security measure)

---

### Reset Password

Reset password using reset token.

```http
POST /auth/reset-password
```

**Request:**
```json
{
  "token": "reset-token-from-email",
  "password": "NewPassword123!",
  "confirm_password": "NewPassword123!"
}
```

**Response:** `200 OK`
```json
{
  "message": "Password reset successfully",
  "redirect": "https://example.com/login"
}
```

**Error Responses:**
- `400 Bad Request` - Invalid or expired token
- `422 Unprocessable Entity` - Passwords don't match

---

## User Endpoints

### Get Current User

Get authenticated user's profile.

```http
GET /users/me
```

**Request:**
```
Authorization: Bearer <access_token>
```

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "full_name": "John Doe",
  "avatar_url": "https://cdn.example.com/avatar.jpg",
  "location": "San Francisco, USA",
  "phone_number": "+1-555-0123",
  "timezone": "America/Los_Angeles",
  "language": "en",
  "company": "Tech Corp",
  "is_active": true,
  "email_verified": true,
  "created_at": "2026-04-14T10:30:00Z",
  "updated_at": "2026-04-14T10:30:00Z"
}
```

**Error Responses:**
- `401 Unauthorized` - Missing or invalid token

---

### List Users

Get paginated list of users. **Admin only**

```http
GET /users/?page=1&limit=20&search=john
```

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| page | integer | 1 | Page number (1-based) |
| limit | integer | 20 | Results per page (max 100) |
| search | string | - | Search by email or name |
| is_active | boolean | - | Filter by active status |
| sort | string | -created_at | Sort field (prefix - for desc) |

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "john@example.com",
      "full_name": "John Doe",
      "is_active": true,
      "created_at": "2026-04-14T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 45,
    "pages": 3
  }
}
```

**Error Responses:**
- `401 Unauthorized` - Missing token
- `403 Forbidden` - Insufficient permissions

---

### Get User

Get specific user by ID. **Admin or own profile**

```http
GET /users/{id}
```

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| id | UUID | User ID |

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "full_name": "John Doe",
  "avatar_url": "https://cdn.example.com/avatar.jpg",
  "location": "San Francisco, USA",
  "phone_number": "+1-555-0123",
  "timezone": "America/Los_Angeles",
  "language": "en",
  "company": "Tech Corp",
  "is_active": true,
  "email_verified": true,
  "last_login_at": "2026-04-14T10:30:00Z",
  "login_count": 42,
  "created_at": "2026-04-14T10:30:00Z",
  "updated_at": "2026-04-14T10:30:00Z"
}
```

**Error Responses:**
- `401 Unauthorized` - Missing token
- `403 Forbidden` - Cannot access other users
- `404 Not Found` - User doesn't exist

---

### Update User

Update user profile. **Own profile or admin**

```http
PUT /users/{id}
```

**Request:**
```json
{
  "full_name": "Jane Doe",
  "avatar_url": "https://cdn.example.com/new-avatar.jpg",
  "location": "New York, USA",
  "phone_number": "+1-555-0456",
  "timezone": "America/New_York",
  "language": "en",
  "company": "New Corp"
}
```

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "full_name": "Jane Doe",
  "avatar_url": "https://cdn.example.com/new-avatar.jpg",
  "location": "New York, USA",
  "phone_number": "+1-555-0456",
  "timezone": "America/New_York",
  "language": "en",
  "company": "New Corp",
  "updated_at": "2026-04-14T11:45:00Z"
}
```

**Error Responses:**
- `400 Bad Request` - Validation failed
- `401 Unauthorized` - Missing token
- `403 Forbidden` - Cannot update other users
- `404 Not Found` - User doesn't exist

---

### Delete User

Delete/deactivate user account. **Own profile or admin**

```http
DELETE /users/{id}
```

**Response:** `204 No Content`

**Note:** Performs soft delete (marks as inactive)

**Error Responses:**
- `401 Unauthorized` - Missing token
- `403 Forbidden` - Cannot delete other users
- `404 Not Found` - User doesn't exist

---

## Blog Endpoints

### List Blog Posts

Get paginated list of published blog posts.

```http
GET /blogs/posts/?page=1&limit=10&author_id=uuid
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| page | integer | Page number (1-based) |
| limit | integer | Results per page (max 100) |
| author_id | UUID | Filter by author |
| search | string | Search in title/content |
| status | string | "draft" or "published" |
| sort | string | Sort field (default: -created_at) |

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440000",
      "title": "Getting Started with Rust",
      "excerpt": "Learn Rust basics...",
      "author": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "full_name": "John Doe"
      },
      "status": "published",
      "created_at": "2026-04-10T09:00:00Z",
      "views": 1250
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 45,
    "pages": 5
  }
}
```

---

### Get Blog Post

Get detailed blog post.

```http
GET /blogs/posts/{id}
```

**Response:** `200 OK`
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "title": "Getting Started with Rust",
  "content": "# Introduction to Rust\n\nRust is...",
  "excerpt": "Learn Rust basics...",
  "author": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "full_name": "John Doe",
    "avatar_url": "https://cdn.example.com/avatar.jpg"
  },
  "status": "published",
  "created_at": "2026-04-10T09:00:00Z",
  "updated_at": "2026-04-10T09:00:00Z",
  "views": 1250,
  "comments_count": 5
}
```

**Error Responses:**
- `404 Not Found` - Post doesn't exist

---

### Create Blog Post

Create new blog post. **Authenticated users**

```http
POST /blogs/posts/
```

**Request:**
```json
{
  "title": "Getting Started with Rust",
  "content": "# Introduction to Rust\n\nRust is...",
  "excerpt": "Learn Rust basics...",
  "status": "draft"
}
```

**Response:** `201 Created`
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "title": "Getting Started with Rust",
  "content": "# Introduction to Rust\n\nRust is...",
  "excerpt": "Learn Rust basics...",
  "author": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "full_name": "John Doe"
  },
  "status": "draft",
  "created_at": "2026-04-14T10:30:00Z"
}
```

**Error Responses:**
- `400 Bad Request` - Validation failed
- `401 Unauthorized` - Missing token

---

### Update Blog Post

Update blog post. **Author or admin**

```http
PUT /blogs/posts/{id}
```

**Request:**
```json
{
  "title": "Updated Title",
  "content": "Updated content...",
  "status": "published"
}
```

**Response:** `200 OK`
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "title": "Updated Title",
  "content": "Updated content...",
  "status": "published",
  "updated_at": "2026-04-14T11:45:00Z"
}
```

---

### Delete Blog Post

Delete blog post. **Author or admin**

```http
DELETE /blogs/posts/{id}
```

**Response:** `204 No Content`

---

### List Post Comments

Get comments for a blog post.

```http
GET /blogs/posts/{id}/comments?page=1&limit=20
```

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "content": "Great article! Really helpful.",
      "author": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "full_name": "Jane Smith"
      },
      "created_at": "2026-04-11T14:20:00Z",
      "likes": 5
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 12,
    "pages": 1
  }
}
```

---

### Create Comment

Add comment to blog post. **Authenticated users**

```http
POST /blogs/posts/{id}/comments
```

**Request:**
```json
{
  "content": "Great article! Really helpful."
}
```

**Response:** `201 Created`
```json
{
  "id": "770e8400-e29b-41d4-a716-446655440000",
  "content": "Great article! Really helpful.",
  "author": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "full_name": "Jane Smith"
  },
  "created_at": "2026-04-14T10:30:00Z"
}
```

---

### Update Comment

Update comment. **Author or admin**

```http
PUT /blogs/comments/{id}
```

**Request:**
```json
{
  "content": "Updated comment content"
}
```

**Response:** `200 OK`

---

### Delete Comment

Delete comment. **Author or admin**

```http
DELETE /blogs/comments/{id}
```

**Response:** `204 No Content`

---

## Error Codes

### HTTP Status Codes

| Code | Name | Description |
|------|------|-------------|
| 200 | OK | Successful request |
| 201 | Created | Resource created successfully |
| 204 | No Content | Successful but no content to return |
| 400 | Bad Request | Invalid request format or validation failed |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Authenticated but lacks permission |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource already exists |
| 422 | Unprocessable Entity | Request validation failed |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 503 | Service Unavailable | Server temporarily unavailable |

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_EMAIL_FORMAT",
    "message": "Email format is invalid",
    "details": {
      "field": "email",
      "value": "invalid-email"
    }
  }
}
```

### Common Error Codes

| Code | HTTP | Description |
|------|------|-------------|
| INVALID_CREDENTIALS | 401 | Email or password incorrect |
| UNAUTHORIZED | 401 | Missing or invalid token |
| FORBIDDEN | 403 | Insufficient permissions |
| USER_NOT_FOUND | 404 | User doesn't exist |
| EMAIL_ALREADY_EXISTS | 409 | Email already registered |
| VALIDATION_ERROR | 422 | Request validation failed |
| INVALID_TOKEN | 401 | Malformed or tampered token |
| TOKEN_EXPIRED | 401 | Token has expired |
| DATABASE_ERROR | 500 | Database operation failed |
| INTERNAL_ERROR | 500 | Unexpected server error |

---

## Rate Limiting

### Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/auth/login` | 10 requests | 15 minutes |
| `/auth/register` | 3 requests | 1 hour |
| `/auth/forgot-password` | 3 requests | 1 hour |
| All other endpoints | 100 requests | 1 minute |

### Rate Limit Headers

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1618413600
```

### 429 Response

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Try again in 60 seconds.",
    "retry_after": 60
  }
}
```

---

## Examples

### Complete Login Flow

```bash
# 1. Register
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePass123!",
    "full_name": "John Doe"
  }'

# 2. Login
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePass123!"
  }'

# Response includes access_token and refresh_token

# 3. Use access token to get profile
curl -X GET http://localhost:8000/api/users/me \
  -H "Authorization: Bearer <access_token>"

# 4. Refresh token when expiring
curl -X POST http://localhost:8000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<refresh_token>"
  }'

# 5. Logout
curl -X POST http://localhost:8000/api/auth/logout \
  -H "Authorization: Bearer <access_token>"
```

### Create & Publish Blog Post

```bash
# 1. Create draft post
curl -X POST http://localhost:8000/api/blogs/posts/ \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Rust Tips",
    "content": "# Useful Rust Tips...",
    "excerpt": "Learn advanced Rust techniques",
    "status": "draft"
  }'

# Response: { "id": "post-uuid", ... }

# 2. Publish the post
curl -X PUT http://localhost:8000/api/blogs/posts/post-uuid \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "published"
  }'

# 3. Get published post
curl -X GET http://localhost:8000/api/blogs/posts/post-uuid

# 4. Add comment
curl -X POST http://localhost:8000/api/blogs/posts/post-uuid/comments \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Great tips!"
  }'
```

---

For more information, see:
- [README.md](README.md) - Project overview
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [DATABASE.md](DATABASE.md) - Database schema

Interactive documentation available at: `http://localhost:8000/docs` (Swagger UI)
