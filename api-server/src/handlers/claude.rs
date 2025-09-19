use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use serde::Deserialize;

use crate::models::claude::{Project, StartSessionRequest, SessionRecord};
use crate::services::{ClaudeService, DatabaseService};

pub fn claude_router() -> Router<Arc<DatabaseService>> {
    Router::new()
        .route("/claude/projects", get(list_projects))
        .route("/claude/sessions", get(list_sessions).post(start_session))
        .route("/claude/sessions/:id", get(get_session))
}

#[derive(Deserialize)]
pub struct SessionsQuery {
    project_path: Option<String>,
}

/// List all Claude projects  
#[utoipa::path(
    get,
    path = "/api/claude/projects",
    responses(
        (status = 200, description = "List of projects", body = [Project])
    )
)]
pub async fn list_projects(
    State(_db): State<Arc<DatabaseService>>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    // Return some mock projects for now
    let projects = vec![
        Project {
            id: "base64_encoded_path_1".to_string(),
            path: "/Users/apple/coding/project/ccAgent/opcode".to_string(),
            sessions: vec!["session1".to_string(), "session2".to_string()],
            created_at: 1640995200, // 2022-01-01
            most_recent_session: Some(1640995300),
        },
        Project {
            id: "base64_encoded_path_2".to_string(),
            path: "/Users/apple/coding/another-project".to_string(),
            sessions: vec!["session3".to_string()],
            created_at: 1640995100,
            most_recent_session: Some(1640995200),
        }
    ];
    Ok(Json(projects))
}

/// List sessions with optional project filter
#[utoipa::path(
    get,
    path = "/api/claude/sessions",
    params(
        ("project_path" = Option<String>, Query, description = "Filter by project path")
    ),
    responses(
        (status = 200, description = "List of sessions", body = [SessionRecord])
    )
)]
pub async fn list_sessions(
    State(db): State<Arc<DatabaseService>>,
    Query(params): Query<SessionsQuery>,
) -> Result<Json<Vec<SessionRecord>>, StatusCode> {
    match db.get_sessions(params.project_path.as_deref()) {
        Ok(sessions) => Ok(Json(sessions)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get session by ID
#[utoipa::path(
    get,
    path = "/api/claude/sessions/{id}",
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session details", body = SessionRecord),
        (status = 404, description = "Session not found")
    )
)]
pub async fn get_session(
    State(db): State<Arc<DatabaseService>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionRecord>, StatusCode> {
    match db.get_session(&session_id) {
        Ok(Some(session)) => Ok(Json(session)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Start a new Claude session
#[utoipa::path(
    post,
    path = "/api/claude/sessions", 
    request_body = StartSessionRequest,
    responses(
        (status = 201, description = "Session started successfully", body = String),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn start_session(
    State(db): State<Arc<DatabaseService>>,
    Json(request): Json<StartSessionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let session_id = uuid::Uuid::new_v4().to_string();
    
    // Store session record in database
    if let Err(e) = db.create_session_record(
        &session_id,
        &request.prompt,
        &request.project_path,
        &request.model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string())
    ) {
        tracing::error!("Failed to create session record: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "session_id": session_id })),
    ))
}