// src/apps/blogs/handlers.rs
//
// Business logic & API route handlers for the Blogs app.
//

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::AppError;
use crate::response::{ApiList, ApiMessage, ApiPaginated, ApiSuccess};
use crate::state::AppState;

use super::models::{BlogPost, Comment};
use super::schemas::{
    CreateBlogPostRequest, CreateCommentRequest, ListBlogsQuery, UpdateBlogPostRequest,
};

// ─── Blog Post handlers ─────────────────────────────────────────────

/// Create a new blog post
#[utoipa::path(
    post,
    path = "/api/v1/blogs",
    request_body = CreateBlogPostRequest,
    responses(
        (status = 201, description = "Blog post created"),
        (status = 400, description = "Validation error"),
    ),
    tag = "Blog Posts"
)]
pub async fn create_blog_post(
    State(state): State<AppState>,
    Json(body): Json<CreateBlogPostRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let slug = body
        .title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-");

    let is_published = body.is_published.unwrap_or(false);
    let short_desc = body.short_description.clone().unwrap_or_default();

    let insert_sql = format!(
        r#"
        INSERT INTO {} (
            id, title, slug, author_id, content, short_description,
            is_published, published_at, created_at, updated_at
        )
        VALUES (
            gen_random_uuid(), $1, $2, $3, $4, $5,
            $6, CASE WHEN $6 THEN NOW() ELSE NULL END, NOW(), NOW()
        )
        RETURNING *
        "#,
        BlogPost::QUALIFIED_TABLE
    );
    let post = sqlx::query_as::<_, BlogPost>(&insert_sql)
        .bind(&body.title)
        .bind(&slug)
        .bind(body.author_id)
        .bind(&body.content)
        .bind(&short_desc)
        .bind(is_published)
        .fetch_one(&state.db)
        .await?;

    Ok(ApiSuccess::created(post))
}

/// List blog posts (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/blogs",
    params(
        ("page" = Option<i64>, Query, description = "Page number"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("published_only" = Option<bool>, Query, description = "Only published posts"),
    ),
    responses(
        (status = 200, description = "Paginated list of blog posts"),
    ),
    tag = "Blog Posts"
)]
pub async fn list_blog_posts(
    State(state): State<AppState>,
    Query(params): Query<ListBlogsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let published_only = params.published_only.unwrap_or(false);

    let (posts, total) = if published_only {
        let list_sql = format!(
            "SELECT * FROM {} WHERE is_published = true ORDER BY published_at DESC LIMIT $1 OFFSET $2",
            BlogPost::QUALIFIED_TABLE
        );
        let posts = sqlx::query_as::<_, BlogPost>(&list_sql)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await?;

        let count_sql = format!(
            "SELECT COUNT(*) FROM {} WHERE is_published = true",
            BlogPost::QUALIFIED_TABLE
        );
        let total = sqlx::query_scalar::<_, i64>(&count_sql)
            .fetch_one(&state.db)
            .await?;

        (posts, total)
    } else {
        let list_sql = format!(
            "SELECT * FROM {} ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            BlogPost::QUALIFIED_TABLE
        );
        let posts = sqlx::query_as::<_, BlogPost>(&list_sql)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await?;

        let count_sql = format!("SELECT COUNT(*) FROM {}", BlogPost::QUALIFIED_TABLE);
        let total = sqlx::query_scalar::<_, i64>(&count_sql)
            .fetch_one(&state.db)
            .await?;

        (posts, total)
    };

    Ok(ApiPaginated::new(posts, total, page, per_page))
}

/// Get a single blog post
#[utoipa::path(
    get,
    path = "/api/v1/blogs/{id}",
    params(("id" = Uuid, Path, description = "Blog post UUID")),
    responses(
        (status = 200, description = "Blog post details"),
        (status = 404, description = "Not found"),
    ),
    tag = "Blog Posts"
)]
pub async fn get_blog_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let get_sql = format!("SELECT * FROM {} WHERE id = $1", BlogPost::QUALIFIED_TABLE);
    let post = sqlx::query_as::<_, BlogPost>(&get_sql)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Blog post {id} not found")))?;

    Ok(ApiSuccess::ok(post))
}

/// Update a blog post
#[utoipa::path(
    put,
    path = "/api/v1/blogs/{id}",
    params(("id" = Uuid, Path, description = "Blog post UUID")),
    request_body = UpdateBlogPostRequest,
    responses(
        (status = 200, description = "Blog post updated"),
        (status = 404, description = "Not found"),
    ),
    tag = "Blog Posts"
)]
pub async fn update_blog_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBlogPostRequest>,
) -> Result<impl IntoResponse, AppError> {
    let update_sql = format!(
        r#"
        UPDATE {} SET
            title             = COALESCE($2, title),
            content           = COALESCE($3, content),
            short_description = COALESCE($4, short_description),
            is_published      = COALESCE($5, is_published),
            published_at      = CASE
                                    WHEN $5 = true AND published_at IS NULL THEN NOW()
                                    ELSE published_at
                                END,
            updated_at        = NOW()
        WHERE id = $1
        RETURNING *
        "#,
        BlogPost::QUALIFIED_TABLE
    );
    let post = sqlx::query_as::<_, BlogPost>(&update_sql)
        .bind(id)
        .bind(&body.title)
        .bind(&body.content)
        .bind(&body.short_description)
        .bind(body.is_published)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Blog post {id} not found")))?;

    Ok(ApiSuccess::ok(post))
}

/// Delete a blog post
#[utoipa::path(
    delete,
    path = "/api/v1/blogs/{id}",
    params(("id" = Uuid, Path, description = "Blog post UUID")),
    responses(
        (status = 200, description = "Blog post deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "Blog Posts"
)]
pub async fn delete_blog_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let delete_sql = format!("DELETE FROM {} WHERE id = $1", BlogPost::QUALIFIED_TABLE);
    let result = sqlx::query(&delete_sql).bind(id).execute(&state.db).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Blog post {id} not found")));
    }

    Ok(ApiMessage::deleted("Blog post"))
}

// ─── Comment handlers ────────────────────────────────────────────────

/// Add a comment to a blog post
#[utoipa::path(
    post,
    path = "/api/v1/blogs/{blog_id}/comments",
    params(("blog_id" = Uuid, Path, description = "Blog post UUID")),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created"),
        (status = 404, description = "Blog post not found"),
    ),
    tag = "Comments"
)]
pub async fn create_comment(
    State(state): State<AppState>,
    Path(blog_id): Path<Uuid>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let exists_sql = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE id = $1)",
        BlogPost::QUALIFIED_TABLE
    );
    let exists = sqlx::query_scalar::<_, bool>(&exists_sql)
        .bind(blog_id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Err(AppError::NotFound(format!("Blog post {blog_id} not found")));
    }

    let insert_sql = format!(
        r#"
        INSERT INTO {} (
            id, blog_post_id, user_id, guest_name, parent_id,
            content, is_approved, created_at, updated_at
        )
        VALUES (
            gen_random_uuid(), $1, $2, $3, $4,
            $5, false, NOW(), NOW()
        )
        RETURNING *
        "#,
        Comment::QUALIFIED_TABLE
    );
    let comment = sqlx::query_as::<_, Comment>(&insert_sql)
        .bind(blog_id)
        .bind(body.user_id)
        .bind(&body.guest_name)
        .bind(body.parent_id)
        .bind(&body.content)
        .fetch_one(&state.db)
        .await?;

    Ok(ApiSuccess::created(comment))
}

/// List comments for a blog post
#[utoipa::path(
    get,
    path = "/api/v1/blogs/{blog_id}/comments",
    params(("blog_id" = Uuid, Path, description = "Blog post UUID")),
    responses(
        (status = 200, description = "List of comments"),
    ),
    tag = "Comments"
)]
pub async fn list_comments(
    State(state): State<AppState>,
    Path(blog_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let list_sql = format!(
        "SELECT * FROM {} WHERE blog_post_id = $1 ORDER BY created_at ASC",
        Comment::QUALIFIED_TABLE
    );
    let comments = sqlx::query_as::<_, Comment>(&list_sql)
        .bind(blog_id)
        .fetch_all(&state.db)
        .await?;

    Ok(ApiList::new(comments))
}
