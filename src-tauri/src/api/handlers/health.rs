use crate::api::{ApiError, ApiResult, AppState};
use crate::api::response::success;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub services: ServiceStatus,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceStatus {
    pub database: String,
    pub checkpoint_manager: String,
    pub process_registry: String,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Service health status", body = HealthResponse),
    ),
    tag = "health"
)]
pub async fn health_check_handler(
    State(app_state): State<AppState>,
) -> ApiResult<Json<crate::api::ApiResponse<HealthResponse>>> {
    // Check database connectivity
    let db_status = match app_state.agent_db.0.lock() {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };
    
    // Check checkpoint manager
    let checkpoint_status = {
        let active_count = app_state.checkpoint_state.active_count().await;
        if active_count >= 0 {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        }
    };
    
    // Check process registry
    let process_status = match app_state.process_registry.0.get_running_agent_processes() {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };
    
    let overall_status = if db_status == "healthy" 
        && checkpoint_status == "healthy" 
        && process_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };
    
    let health = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        services: ServiceStatus {
            database: db_status,
            checkpoint_manager: checkpoint_status,
            process_registry: process_status,
        },
    };
    
    Ok(Json(success(health)))
}