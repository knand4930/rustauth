// src/main.rs

mod activitylog;
mod blogs;
mod config;
mod db;
mod error;
mod models;
mod response;
mod user;

use axum::http::{header, Method};
use config::AppConfig;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// ─── OpenAPI spec ────────────────────────────────────────────────────

#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustAuth API",
        version = "1.0.0",
        description = "Authentication & Blog API built with Axum + SQLx",
    ),
    paths(
        // Auth
        user::handler::register,
        user::handler::login,
        // Users
        user::handler::list_users,
        user::handler::get_user,
        user::handler::update_user,
        user::handler::delete_user,
        // Blogs
        blogs::handler::create_blog_post,
        blogs::handler::list_blog_posts,
        blogs::handler::get_blog_post,
        blogs::handler::update_blog_post,
        blogs::handler::delete_blog_post,
        // Comments
        blogs::handler::create_comment,
        blogs::handler::list_comments,
    ),
    components(schemas(
        // User DB models
        user::models::User,
        // User schemas (request/response DTOs)
        user::schema::UserResponse,
        user::schema::RegisterRequest,
        user::schema::LoginRequest,
        user::schema::UpdateUserRequest,
        user::schema::AuthTokenResponse,
        // Blog DB models
        blogs::models::BlogPost,
        blogs::models::Comment,
        // Blog schemas (request DTOs)
        blogs::schema::CreateBlogPostRequest,
        blogs::schema::UpdateBlogPostRequest,
        blogs::schema::CreateCommentRequest,
        // Activity log
        activitylog::models::ActivityLog,
    )),
    tags(
        (name = "Authentication", description = "Register & Login endpoints"),
        (name = "Users", description = "User CRUD operations"),
        (name = "Blog Posts", description = "Blog post management"),
        (name = "Comments", description = "Blog post comments"),
    )
)]
struct ApiDoc;

// ─── Entry point ─────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Load .env
    dotenv::dotenv().ok();

    // Init structured logging (tracing)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = AppConfig::from_env();
    tracing::info!("Starting server on {}:{}", config.server_host, config.server_port);

    // Connect to database
    let pool = db::init_pool(&config.database_url).await;
    tracing::info!("✓ Database connected");

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Build application
    let app = axum::Router::new()
        // Health check
        .route("/api/v1/health", axum::routing::get(health_check))
        // Merge app routes
        .merge(user::routes())
        .merge(blogs::routes())
        // Swagger UI at /swagger-ui
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // Shared state
        .with_state(pool);

    // Bind & serve
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("╔══════════════════════════════════════════╗");
    tracing::info!("║  🚀 Server running on http://{}  ║", addr);
    tracing::info!("║  📖 Swagger UI:  http://{}/swagger-ui  ║", addr);
    tracing::info!("╚══════════════════════════════════════════╝");

    axum::serve(listener, app).await.unwrap();
}

// ─── Health check ────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses((status = 200, description = "Server is running")),
    tag = "System"
)]
async fn health_check() -> &'static str {
    "OK"
}
