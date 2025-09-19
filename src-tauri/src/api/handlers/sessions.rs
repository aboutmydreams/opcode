use crate::api::{ApiError, ApiResult, AppState};
use crate::api::response::success;
use axum::extract::{Path, State};
use axum::Json;
use serde_json::Value;
use utoipa::ToSchema;

/// Load session history
#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/history/{project_id}",
    responses(
        (status = 200, description = "Session history messages", body = [Value]),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("session_id" = String, Path, description = "Session ID"),
        ("project_id" = String, Path, description = "Project ID")
    ),
    tag = "sessions"
)]
pub async fn get_session_history_handler(
    State(_app_state): State<AppState>,
    Path((session_id, project_id)): Path<(String, String)>,
) -> ApiResult<Json<crate::api::ApiResponse<Vec<Value>>>> {
    let history = crate::commands::claude::load_session_history(session_id.clone(), project_id.clone())
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::NotFound(format!("Session '{}' in project '{}' not found", session_id, project_id))
            } else {
                ApiError::Internal(e)
            }
        })?;
    
    Ok(Json(success(history)))
}