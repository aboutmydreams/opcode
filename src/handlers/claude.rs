use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use serde::Deserialize;

use crate::models::claude::{Project, StartSessionRequest, SessionRecord, CreateProjectRequest, UpdateProjectRequest};
use crate::services::{ClaudeService, DatabaseService};

pub fn claude_router() -> Router<Arc<DatabaseService>> {
    Router::new()
        .route("/claude/projects", get(list_projects).post(create_project))
        .route("/claude/projects/:id", put(update_project).delete(delete_project))
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
    State(db): State<Arc<DatabaseService>>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    match db.get_projects() {
        Ok(projects) => Ok(Json(projects)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create a new Claude project
#[utoipa::path(
    post,
    path = "/api/claude/projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created successfully", body = Project),
        (status = 400, description = "Invalid request or path validation failed"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_project(
    State(db): State<Arc<DatabaseService>>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    match db.create_project(request) {
        Ok(project) => Ok((StatusCode::CREATED, Json(project))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Parent directory does not exist") || 
               error_msg.contains("Invalid path") ||
               error_msg.contains("UNIQUE constraint failed") {
                Err(StatusCode::BAD_REQUEST)
            } else {
                tracing::error!("Failed to create project: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
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

/// Update an existing project
#[utoipa::path(
    put,
    path = "/api/claude/projects/{id}",
    request_body = UpdateProjectRequest,
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project updated successfully", body = Project),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_project(
    State(db): State<Arc<DatabaseService>>,
    Path(project_id): Path<String>,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    match db.update_project(&project_id, request) {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a project and its directory
#[utoipa::path(
    delete,
    path = "/api/claude/projects/{id}",
    params(
        ("id" = String, Path, description = "Project ID")
    ),
    responses(
        (status = 204, description = "Project deleted successfully"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_project(
    State(db): State<Arc<DatabaseService>>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match db.delete_project(&project_id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}