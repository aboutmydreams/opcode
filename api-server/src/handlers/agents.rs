use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::models::agent::{Agent, CreateAgentRequest};
use crate::services::DatabaseService;

pub fn agents_router() -> Router<Arc<DatabaseService>> {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).delete(delete_agent))
}

/// List all agents
#[utoipa::path(
    get,
    path = "/api/agents",
    responses(
        (status = 200, description = "List of agents", body = [Agent])
    )
)]
async fn list_agents(
    State(db): State<Arc<DatabaseService>>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    match db.get_agents() {
        Ok(agents) => Ok(Json(agents)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get agent by ID
#[utoipa::path(
    get,
    path = "/api/agents/{id}",
    params(
        ("id" = i64, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Agent details", body = Agent),
        (status = 404, description = "Agent not found")
    )
)]
async fn get_agent(
    Path(id): Path<i64>,
    State(db): State<Arc<DatabaseService>>,
) -> Result<Json<Agent>, StatusCode> {
    match db.get_agent(id) {
        Ok(Some(agent)) => Ok(Json(agent)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create a new agent
#[utoipa::path(
    post,
    path = "/api/agents",
    request_body = CreateAgentRequest,
    responses(
        (status = 201, description = "Agent created successfully", body = Agent),
        (status = 400, description = "Invalid request")
    )
)]
async fn create_agent(
    State(db): State<Arc<DatabaseService>>,
    Json(request): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<Agent>), StatusCode> {
    match db.create_agent(request) {
        Ok(agent) => Ok((StatusCode::CREATED, Json(agent))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// Delete an agent
#[utoipa::path(
    delete,
    path = "/api/agents/{id}",
    params(
        ("id" = i64, Path, description = "Agent ID")
    ),
    responses(
        (status = 204, description = "Agent deleted successfully"),
        (status = 404, description = "Agent not found")
    )
)]
async fn delete_agent(
    Path(id): Path<i64>,
    State(db): State<Arc<DatabaseService>>,
) -> StatusCode {
    match db.delete_agent(id) {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}