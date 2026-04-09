// src/main.rs

mod blogs;
mod config;
mod db;
mod error;
mod models;
mod response;
mod state;
mod user;

use axum::http::{header, Method};
use config::AppConfig;
use state::AppState;
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
        user::handlers::register,
        user::handlers::login,
        // Users
        user::handlers::list_users,
        user::handlers::get_user,
        user::handlers::update_user,
        user::handlers::delete_user,
        // Blogs
        blogs::handlers::create_blog_post,
        blogs::handlers::list_blog_posts,
        blogs::handlers::get_blog_post,
        blogs::handlers::update_blog_post,
        blogs::handlers::delete_blog_post,
        // Comments
        blogs::handlers::create_comment,
        blogs::handlers::list_comments,
       
    ),
    components(schemas(
        // User
        user::User,
        user::UserResponse,
        user::RegisterRequest,
        user::LoginRequest,
        user::UpdateUserRequest,
        user::AuthTokenResponse,
        // Blogs
        blogs::BlogPost,
        blogs::Comment,
        blogs::CreateBlogPostRequest,
        blogs::UpdateBlogPostRequest,
        blogs::CreateCommentRequest,
    )),
    tags(
        (name = "Authentication", description = "Register & Login endpoints"),
        (name = "Users",          description = "User CRUD operations"),
        (name = "Blog Posts",     description = "Blog post management"),
        (name = "Comments",       description = "Blog post comments"),
        (name = "News",           description = "News management"),
    )
)]
struct ApiDoc;

// ─── Entry point ─────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    tracing::info!("Starting server on {}:{}", config.server_host, config.server_port);

    let pool = db::init_pool(&config.database_url).await;
    tracing::info!("Database connected");

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let state = AppState::new(pool, config);

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = axum::Router::new()
        .route("/api/v1/health", axum::routing::get(health_check))
        .merge(user::routes())
        .merge(blogs::routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server running on http://{addr}");
    tracing::info!("Swagger UI at http://{addr}/swagger-ui");

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
