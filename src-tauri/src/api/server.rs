use crate::api::{
    routes::create_api_routes, middleware::middleware_stack, AppState, ApiError
};
use crate::commands::agents::AgentDb;
use crate::checkpoint::state::CheckpointState;
use crate::process::ProcessRegistryState;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::Redoc;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::health_check_handler,
        crate::api::handlers::list_agents_handler,
        crate::api::handlers::get_agent_handler,
        crate::api::handlers::create_agent_handler,
        crate::api::handlers::execute_agent_handler,
        crate::api::handlers::list_agent_runs_handler,
        crate::api::handlers::list_all_agent_runs_handler,
        crate::api::handlers::list_projects_handler,
        crate::api::handlers::get_project_sessions_handler,
        crate::api::handlers::get_session_history_handler,
    ),
    components(
        schemas(
            crate::api::handlers::CreateAgentRequest,
            crate::api::handlers::ExecuteAgentRequest,
            crate::api::handlers::HealthResponse,
            crate::api::handlers::ServiceStatus,
            crate::commands::agents::Agent,
            crate::commands::agents::AgentRun,
            crate::commands::agents::AgentRunWithMetrics,
            crate::commands::agents::AgentRunMetrics,
            crate::commands::claude::Project,
            crate::commands::claude::Session,
            crate::api::response::ApiResponse<Vec<crate::commands::agents::Agent>>,
            crate::api::response::ApiResponse<crate::commands::agents::Agent>,
            crate::api::response::ApiResponse<crate::api::handlers::HealthResponse>,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "agents", description = "Agent management endpoints"),
        (name = "projects", description = "Project management endpoints"),
        (name = "sessions", description = "Session management endpoints"),
    ),
    info(
        title = "Opcode HTTP API",
        version = "1.0.0",
        description = "HTTP API for Opcode - A powerful GUI app and Toolkit for Claude Code",
        contact(
            name = "Opcode Team",
            url = "https://github.com/getAsterisk/opcode",
        ),
        license(
            name = "AGPL-3.0",
            url = "https://www.gnu.org/licenses/agpl-3.0.html",
        ),
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development server"),
    ),
)]
pub struct ApiDoc;

pub struct ApiServer {
    app_state: AppState,
    port: u16,
}

impl ApiServer {
    pub fn new(
        agent_db: AgentDb,
        checkpoint_state: CheckpointState,
        process_registry: ProcessRegistryState,
        port: Option<u16>,
    ) -> Self {
        let app_state = AppState::new(agent_db, checkpoint_state, process_registry);
        let port = port.unwrap_or(3001);
        
        Self { app_state, port }
    }
    
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = self.create_app();
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        
        log::info!("🚀 Starting HTTP API server on http://{}", addr);
        log::info!("📚 API Documentation available at:");
        log::info!("   - Swagger UI: http://{}/docs", addr);
        log::info!("   - RapiDoc: http://{}/rapidoc", addr);
        log::info!("   - ReDoc: http://{}/redoc", addr);
        log::info!("   - OpenAPI JSON: http://{}/api-docs/openapi.json", addr);
        
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    fn create_app(self) -> Router {
        let api_routes = create_api_routes();
        
        Router::new()
            // API routes
            .merge(api_routes)
            
            // Documentation routes
            .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            
            // Apply middleware
            .layer(middleware_stack())
            
            // Set application state
            .with_state(self.app_state)
    }
    
    pub fn get_app_for_testing(self) -> Router {
        self.create_app()
    }
}