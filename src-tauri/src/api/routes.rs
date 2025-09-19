use crate::api::handlers::{
    // Agents
    list_agents_handler, get_agent_handler, create_agent_handler,
    execute_agent_handler, list_agent_runs_handler, list_all_agent_runs_handler,
    // Projects
    list_projects_handler, get_project_sessions_handler,
    // Sessions
    get_session_history_handler,
    // Health
    health_check_handler,
};
use crate::api::AppState;
use axum::routing::{get, post};
use axum::Router;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health_check_handler))
        
        // Agent routes
        .route("/agents", get(list_agents_handler).post(create_agent_handler))
        .route("/agents/:id", get(get_agent_handler))
        .route("/agents/:id/execute", post(execute_agent_handler))
        .route("/agents/:id/runs", get(list_agent_runs_handler))
        .route("/agents/runs", get(list_all_agent_runs_handler))
        
        // Project routes
        .route("/projects", get(list_projects_handler))
        .route("/projects/:project_id/sessions", get(get_project_sessions_handler))
        
        // Session routes
        .route("/sessions/:session_id/history/:project_id", get(get_session_history_handler))
}

pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .nest("/api", create_routes())
}