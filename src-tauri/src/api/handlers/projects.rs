use crate::api::{ApiError, ApiResult, AppState};
use crate::api::response::success;
use crate::commands::claude::{Project, Session};
use axum::extract::{Path, State};
use axum::Json;
use utoipa::ToSchema;

/// List all projects
#[utoipa::path(
    get,
    path = "/api/projects",
    responses(
        (status = 200, description = "List of projects", body = [Project]),
        (status = 500, description = "Internal server error")
    ),
    tag = "projects"
)]
pub async fn list_projects_handler(
    State(_app_state): State<AppState>,
) -> ApiResult<Json<crate::api::ApiResponse<Vec<Project>>>> {
    // Direct call to list_projects function (no Tauri dependency)
    let projects = crate::commands::claude::list_projects()
        .await
        .map_err(|e| ApiError::Internal(e))?;
    
    Ok(Json(success(projects)))
}

/// Get sessions for a project
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/sessions",
    responses(
        (status = 200, description = "List of sessions for the project", body = [Session]),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("project_id" = String, Path, description = "Project ID")
    ),
    tag = "projects"
)]
pub async fn get_project_sessions_handler(
    State(_app_state): State<AppState>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<crate::api::ApiResponse<Vec<Session>>>> {
    let sessions = crate::commands::claude::get_project_sessions(project_id.clone())
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(format!("Project '{}' not found", project_id))
            } else {
                ApiError::Internal(e)
            }
        })?;
    
    Ok(Json(success(sessions)))
}