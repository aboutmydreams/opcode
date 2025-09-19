use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod error;
mod handlers;
mod models;
mod services;
mod websocket;

use config::AppConfig;
use error::Result;
use handlers::{agents_router, claude_router, mcp_router, storage_router};
use services::{ClaudeService, DatabaseService, MCPService};
use websocket::{WebSocketManager, websocket_router};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::agents::list_agents,
        handlers::agents::get_agent,
        handlers::agents::create_agent,
        handlers::agents::delete_agent,
        handlers::claude::list_projects,
        handlers::claude::list_sessions,
        handlers::claude::get_session,
        handlers::claude::start_session,
        handlers::storage::get_storage_usage,
    ),
    components(
        schemas(
            models::agent::Agent,
            models::agent::CreateAgentRequest,
            models::claude::Project,
            models::claude::StartSessionRequest,
            models::claude::SessionRecord,
        )
    ),
    tags(
        (name = "agents", description = "Agent management API"),
        (name = "claude", description = "Claude Code session management API"),
        (name = "storage", description = "Storage management API"),
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::load()?;

    // Initialize tracing
    init_tracing(&config)?;

    tracing::info!("Starting OpCode API Server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Configuration loaded: {:#?}", config);

    // Initialize services
    let db_service = Arc::new(DatabaseService::new()?);
    let _claude_service = Arc::new(ClaudeService::new(db_service.clone())?);
    let mcp_service = Arc::new(MCPService::new()?);
    let ws_manager = Arc::new(WebSocketManager::new());

    // Create API documentation
    let api_doc = ApiDoc::openapi();

    // Build the application router
    let app = Router::new()
        // API routes
        .nest("/api", 
            Router::new()
                .merge(agents_router().with_state(db_service.clone()))
                .merge(claude_router().with_state(db_service.clone()))
                .merge(mcp_router().with_state(mcp_service.clone()))
                .merge(storage_router())
        )
        // WebSocket routes
        .merge(websocket_router().with_state(ws_manager.clone()))
        // Health check
        .route("/health", get(health_check))
        // Swagger UI
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api_doc))
        // Middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::very_permissive())
        );

    // Start the server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("🚀 OpCode API Server started on http://{}", addr);
    tracing::info!("📚 API Documentation available at http://{}/docs", addr);
    tracing::info!("🏥 Health check at http://{}/health", addr);
    tracing::info!("🔌 WebSocket endpoints at ws://{}/ws/...", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

fn init_tracing(config: &AppConfig) -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.logging.level));

    let subscriber = tracing_subscriber::registry().with(filter);

    if config.logging.json_format {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        subscriber
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received");
}

async fn health_check() -> &'static str {
    "OK"
}