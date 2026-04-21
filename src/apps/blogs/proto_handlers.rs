// src/apps/blogs/proto_handlers.rs
//
// Protobuf HTTP handlers for the Blogs app.
// Routes: POST/GET/PUT/DELETE /api/v1/proto/blogs
//         POST/GET /api/v1/proto/blogs/{blog_id}/comments
//
// Request bodies are decoded from application/x-protobuf.
// Responses are encoded to application/x-protobuf.
//

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::error::AppError;
use crate::proto::{
    Protobuf,
    blogs::{
        BlogPost as ProtoBlogPost, BlogPostList, Comment as ProtoComment, CommentList,
        CreateBlogPostRequest, CreateCommentRequest, DeleteResponse, UpdateBlogPostRequest,
    },
};
use crate::state::AppState;

use super::models::{BlogPost, Comment};
use super::schemas::ListBlogsQuery;

// ─── Conversion helpers ───────────────────────────────────────────────

fn blog_post_to_proto(p: BlogPost) -> ProtoBlogPost {
    ProtoBlogPost {
        id: p.id.to_string(),
        title: p.title,
        slug: p.slug,
        author_id: p.author_id.to_string(),
        content: p.content,
        short_description: p.short_description,
        is_published: p.is_published,
        published_at: p.published_at.map(|t| t.timestamp()),
        created_at: p.created_at.timestamp(),
        updated_at: p.updated_at.timestamp(),
    }
}

fn comment_to_proto(c: Comment) -> ProtoComment {
    ProtoComment {
        id: c.id.to_string(),
        user_id: c.user_id.map(|u| u.to_string()),
        guest_name: c.guest_name,
        blog_post_id: c.blog_post_id.to_string(),
        parent_id: c.parent_id.map(|u| u.to_string()),
        content: c.content,
        is_approved: c.is_approved,
        created_at: c.created_at.timestamp(),
        updated_at: c.updated_at.timestamp(),
    }
}

// ─── Blog Post handlers ───────────────────────────────────────────────

pub async fn create_blog_post(
    State(state): State<AppState>,
    Protobuf(body): Protobuf<CreateBlogPostRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("title cannot be empty".into()));
    }

    let slug = body
        .title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-");

    let author_id = Uuid::parse_str(&body.author_id)
        .map_err(|_| AppError::BadRequest("invalid author_id UUID".into()))?;

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
        .bind(author_id)
        .bind(&body.content)
        .bind(&body.short_description)
        .bind(body.is_published)
        .fetch_one(&state.db)
        .await?;

    Ok((StatusCode::CREATED, Protobuf(blog_post_to_proto(post))))
}

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

    Ok(Protobuf(BlogPostList {
        posts: posts.into_iter().map(blog_post_to_proto).collect(),
        total,
        page,
        per_page,
    }))
}

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

    Ok(Protobuf(blog_post_to_proto(post)))
}

pub async fn update_blog_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Protobuf(body): Protobuf<UpdateBlogPostRequest>,
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

    Ok(Protobuf(blog_post_to_proto(post)))
}

pub async fn delete_blog_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let delete_sql = format!("DELETE FROM {} WHERE id = $1", BlogPost::QUALIFIED_TABLE);
    let result = sqlx::query(&delete_sql).bind(id).execute(&state.db).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Blog post {id} not found")));
    }

    Ok(Protobuf(DeleteResponse {
        message: format!("Blog post {id} deleted"),
    }))
}

// ─── Comment handlers ─────────────────────────────────────────────────

pub async fn create_comment(
    State(state): State<AppState>,
    Path(blog_id): Path<Uuid>,
    Protobuf(body): Protobuf<CreateCommentRequest>,
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

    let user_id = body
        .user_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("invalid user_id UUID".into()))?;

    let parent_id = body
        .parent_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::BadRequest("invalid parent_id UUID".into()))?;

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
        .bind(user_id)
        .bind(&body.guest_name)
        .bind(parent_id)
        .bind(&body.content)
        .fetch_one(&state.db)
        .await?;

    Ok((StatusCode::CREATED, Protobuf(comment_to_proto(comment))))
}

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

    let count = comments.len() as i32;
    Ok(Protobuf(CommentList {
        comments: comments.into_iter().map(comment_to_proto).collect(),
        count,
    }))
}
