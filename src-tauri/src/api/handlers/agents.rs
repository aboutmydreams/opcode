use crate::api::{ApiError, ApiResult, AppState};
use crate::api::response::{success, success_with_message, success_message};
use crate::commands::agents::{Agent, AgentRun, AgentRunWithMetrics};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Json, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAgentRequest {
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: Option<String>,
    pub enable_file_read: Option<bool>,
    pub enable_file_write: Option<bool>,
    pub enable_network: Option<bool>,
    pub hooks: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteAgentRequest {
    pub project_path: String,
    pub task: String,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AgentQueryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// List all agents
#[utoipa::path(
    get,
    path = "/api/agents",
    responses(
        (status = 200, description = "List of agents", body = [Agent]),
        (status = 500, description = "Internal server error")
    ),
    tag = "agents"
)]
pub async fn list_agents_handler(
    State(app_state): State<AppState>,
    Query(params): Query<AgentQueryParams>,
) -> ApiResult<Json<crate::api::ApiResponse<Vec<Agent>>>> {
    let db = &app_state.agent_db;
    
    // Direct database access instead of using Tauri commands
    let conn = db.0.lock().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let mut stmt = conn
        .prepare("SELECT id, name, icon, system_prompt, default_task, model, enable_file_read, enable_file_write, enable_network, hooks, created_at, updated_at FROM agents ORDER BY created_at DESC")
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let agents = stmt
        .query_map([], |row| {
            Ok(Agent {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                system_prompt: row.get(3)?,
                default_task: row.get(4)?,
                model: row.get::<_, String>(5).unwrap_or_else(|_| "sonnet".to_string()),
                enable_file_read: row.get::<_, bool>(6).unwrap_or(true),
                enable_file_write: row.get::<_, bool>(7).unwrap_or(true),
                enable_network: row.get::<_, bool>(8).unwrap_or(false),
                hooks: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    // Apply pagination if requested
    let agents = if let (Some(page), Some(limit)) = (params.page, params.limit) {
        let start = (page.saturating_sub(1) * limit) as usize;
        let end = (start + limit as usize).min(agents.len());
        agents.get(start..end).unwrap_or(&[]).to_vec()
    } else {
        agents
    };
    
    Ok(Json(success(agents)))
}

/// Get a specific agent by ID
#[utoipa::path(
    get,
    path = "/api/agents/{id}",
    responses(
        (status = 200, description = "Agent details", body = Agent),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i64, Path, description = "Agent ID")
    ),
    tag = "agents"
)]
pub async fn get_agent_handler(
    State(app_state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<crate::api::ApiResponse<Agent>>> {
    let db = &app_state.agent_db;
    let conn = db.0.lock().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let agent = conn
        .query_row(
            "SELECT id, name, icon, system_prompt, default_task, model, enable_file_read, enable_file_write, enable_network, hooks, created_at, updated_at FROM agents WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Agent {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    system_prompt: row.get(3)?,
                    default_task: row.get(4)?,
                    model: row.get::<_, String>(5).unwrap_or_else(|_| "sonnet".to_string()),
                    enable_file_read: row.get::<_, bool>(6).unwrap_or(true),
                    enable_file_write: row.get::<_, bool>(7).unwrap_or(true),
                    enable_network: row.get::<_, bool>(8).unwrap_or(false),
                    hooks: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| {
            match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    ApiError::NotFound(format!("Agent with ID {} not found", id))
                }
                _ => ApiError::DatabaseError(e.to_string())
            }
        })?;
    
    Ok(Json(success(agent)))
}

/// Create a new agent
#[utoipa::path(
    post,
    path = "/api/agents",
    request_body = CreateAgentRequest,
    responses(
        (status = 201, description = "Agent created successfully", body = Agent),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    ),
    tag = "agents"
)]
pub async fn create_agent_handler(
    State(app_state): State<AppState>,
    Json(request): Json<CreateAgentRequest>,
) -> ApiResult<Response> {
    let db = &app_state.agent_db;
    let conn = db.0.lock().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let model = request.model.unwrap_or_else(|| "sonnet".to_string());
    let enable_file_read = request.enable_file_read.unwrap_or(true);
    let enable_file_write = request.enable_file_write.unwrap_or(true);
    let enable_network = request.enable_network.unwrap_or(false);

    conn.execute(
        "INSERT INTO agents (name, icon, system_prompt, default_task, model, enable_file_read, enable_file_write, enable_network, hooks) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![request.name, request.icon, request.system_prompt, request.default_task, model, enable_file_read, enable_file_write, enable_network, request.hooks],
    )
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let id = conn.last_insert_rowid();

    // Fetch the created agent
    let agent = conn
        .query_row(
            "SELECT id, name, icon, system_prompt, default_task, model, enable_file_read, enable_file_write, enable_network, hooks, created_at, updated_at FROM agents WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Agent {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    system_prompt: row.get(3)?,
                    default_task: row.get(4)?,
                    model: row.get(5)?,
                    enable_file_read: row.get(6)?,
                    enable_file_write: row.get(7)?,
                    enable_network: row.get(8)?,
                    hooks: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let response = success_with_message(agent, "Agent created successfully".to_string());
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// Execute an agent (Not implemented for HTTP API)
#[utoipa::path(
    post,
    path = "/api/agents/{id}/execute",
    request_body = ExecuteAgentRequest,
    responses(
        (status = 501, description = "Not implemented - use desktop application for agent execution"),
        (status = 404, description = "Agent not found"),
    ),
    params(
        ("id" = i64, Path, description = "Agent ID")
    ),
    tag = "agents"
)]
pub async fn execute_agent_handler(
    State(_app_state): State<AppState>,
    Path(_agent_id): Path<i64>,
    Json(_request): Json<ExecuteAgentRequest>,
) -> ApiResult<Response> {
    Err(ApiError::Internal(
        "Agent execution via HTTP API is not yet implemented. Please use the desktop application.".to_string()
    ))
}

/// List agent runs with metrics
#[utoipa::path(
    get,
    path = "/api/agents/{id}/runs",
    responses(
        (status = 200, description = "List of agent runs", body = [AgentRun]),
        (status = 404, description = "Agent not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i64, Path, description = "Agent ID")
    ),
    tag = "agents"
)]
pub async fn list_agent_runs_handler(
    State(app_state): State<AppState>,
    Path(agent_id): Path<i64>,
) -> ApiResult<Json<crate::api::ApiResponse<Vec<AgentRun>>>> {
    let db = &app_state.agent_db;
    let conn = db.0.lock().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, pid, process_started_at, created_at, completed_at 
         FROM agent_runs WHERE agent_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let runs = stmt
        .query_map(rusqlite::params![agent_id], |row| {
            Ok(AgentRun {
                id: Some(row.get(0)?),
                agent_id: row.get(1)?,
                agent_name: row.get(2)?,
                agent_icon: row.get(3)?,
                task: row.get(4)?,
                model: row.get(5)?,
                project_path: row.get(6)?,
                session_id: row.get(7)?,
                status: row.get::<_, String>(8).unwrap_or_else(|_| "pending".to_string()),
                pid: row.get::<_, Option<i64>>(9).ok().flatten().map(|p| p as u32),
                process_started_at: row.get(10)?,
                created_at: row.get(11)?,
                completed_at: row.get(12)?,
            })
        })
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    Ok(Json(success(runs)))
}

/// List all agent runs
#[utoipa::path(
    get,
    path = "/api/agents/runs",
    responses(
        (status = 200, description = "List of all agent runs", body = [AgentRun]),
        (status = 500, description = "Internal server error")
    ),
    tag = "agents"
)]
pub async fn list_all_agent_runs_handler(
    State(app_state): State<AppState>,
) -> ApiResult<Json<crate::api::ApiResponse<Vec<AgentRun>>>> {
    let db = &app_state.agent_db;
    let conn = db.0.lock().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, pid, process_started_at, created_at, completed_at 
         FROM agent_runs ORDER BY created_at DESC"
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let runs = stmt
        .query_map([], |row| {
            Ok(AgentRun {
                id: Some(row.get(0)?),
                agent_id: row.get(1)?,
                agent_name: row.get(2)?,
                agent_icon: row.get(3)?,
                task: row.get(4)?,
                model: row.get(5)?,
                project_path: row.get(6)?,
                session_id: row.get(7)?,
                status: row.get::<_, String>(8).unwrap_or_else(|_| "pending".to_string()),
                pid: row.get::<_, Option<i64>>(9).ok().flatten().map(|p| p as u32),
                process_started_at: row.get(10)?,
                created_at: row.get(11)?,
                completed_at: row.get(12)?,
            })
        })
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    Ok(Json(success(runs)))
}