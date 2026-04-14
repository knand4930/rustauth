use axum::http::{Method, header};
use rustauth::{admin, apps, config::AppConfig, db, health_check, state::AppState};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    tracing::info!(
        "Starting server on {}:{}",
        config.server_host,
        config.server_port
    );

    let pool = db::init_pool(&config.database_url).await;
    tracing::info!("Database connected");

    let address = format!("{}:{}", config.server_host, config.server_port);
    let admin_panel = admin::initialize_adminx();
    tracing::info!(
        "AdminX initialized with {} apps and {} resources",
        admin_panel.app_count,
        admin_panel.resource_count
    );
    let state = AppState::new(pool, config, admin_panel);

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = axum::Router::new()
        .route("/api/v1/health", axum::routing::get(health_check))
        .merge(admin::routes())
        .merge(admin::web_routes(state.clone()))
        .merge(apps::routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", apps::ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    tracing::info!("Server running on http://{address}");
    tracing::info!("Swagger UI at http://{address}/swagger-ui");
    tracing::info!("Admin panel at http://{address}/adminx/login/");

    axum::serve(listener, app).await.unwrap();
}
