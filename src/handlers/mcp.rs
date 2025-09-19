use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::error::Result;
use crate::models::mcp::{
    AddMCPServerRequest, ConnectionTestResult, ImportResult, MCPServer, MCPServerResult,
    UpdateMCPServerRequest,
};
use crate::services::MCPService;

pub fn mcp_router() -> Router<Arc<MCPService>> {
    Router::new()
        .route("/mcp/servers", get(list_servers))
        .route("/mcp/servers", post(add_server))
        .route("/mcp/servers/:name", get(get_server))
        .route("/mcp/servers/:name", put(update_server))
        .route("/mcp/servers/:name", delete(remove_server))
        .route("/mcp/servers/:name/test", post(test_connection))
        .route("/mcp/import", post(import_from_claude_desktop))
}

/// List all MCP servers
#[utoipa::path(
    get,
    path = "/api/mcp/servers",
    responses(
        (status = 200, description = "List of MCP servers", body = [MCPServer])
    ),
    tag = "mcp"
)]
async fn list_servers(State(mcp): State<Arc<MCPService>>) -> Result<Json<Vec<MCPServer>>> {
    let servers = mcp.list_servers().await?;
    Ok(Json(servers))
}

/// Get a specific MCP server
#[utoipa::path(
    get,
    path = "/api/mcp/servers/{name}",
    params(
        ("name" = String, Path, description = "MCP server name")
    ),
    responses(
        (status = 200, description = "MCP server details", body = MCPServer),
        (status = 404, description = "Server not found")
    ),
    tag = "mcp"
)]
async fn get_server(
    Path(name): Path<String>,
    State(mcp): State<Arc<MCPService>>,
) -> Result<Json<MCPServer>> {
    let server = mcp.get_server(&name).await?;
    Ok(Json(server))
}

/// Add a new MCP server
#[utoipa::path(
    post,
    path = "/api/mcp/servers",
    request_body = AddMCPServerRequest,
    responses(
        (status = 201, description = "Server added successfully", body = MCPServerResult),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "mcp"
)]
async fn add_server(
    State(mcp): State<Arc<MCPService>>,
    Json(request): Json<AddMCPServerRequest>,
) -> Result<(StatusCode, Json<MCPServerResult>)> {
    let result = mcp.add_server(request).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

/// Update an existing MCP server
#[utoipa::path(
    put,
    path = "/api/mcp/servers/{name}",
    params(
        ("name" = String, Path, description = "MCP server name")
    ),
    request_body = UpdateMCPServerRequest,
    responses(
        (status = 200, description = "Server updated successfully", body = MCPServerResult),
        (status = 404, description = "Server not found"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "mcp"
)]
async fn update_server(
    Path(name): Path<String>,
    State(mcp): State<Arc<MCPService>>,
    Json(request): Json<UpdateMCPServerRequest>,
) -> Result<Json<MCPServerResult>> {
    let result = mcp.update_server(&name, request).await?;
    Ok(Json(result))
}

/// Remove an MCP server
#[utoipa::path(
    delete,
    path = "/api/mcp/servers/{name}",
    params(
        ("name" = String, Path, description = "MCP server name")
    ),
    responses(
        (status = 200, description = "Server removed successfully", body = MCPServerResult),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "mcp"
)]
async fn remove_server(
    Path(name): Path<String>,
    State(mcp): State<Arc<MCPService>>,
) -> Result<Json<MCPServerResult>> {
    let result = mcp.remove_server(&name).await?;
    Ok(Json(result))
}

/// Test connection to an MCP server
#[utoipa::path(
    post,
    path = "/api/mcp/servers/{name}/test",
    params(
        ("name" = String, Path, description = "MCP server name")
    ),
    responses(
        (status = 200, description = "Connection test result", body = ConnectionTestResult),
        (status = 404, description = "Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "mcp"
)]
async fn test_connection(
    Path(name): Path<String>,
    State(mcp): State<Arc<MCPService>>,
) -> Result<Json<ConnectionTestResult>> {
    let result = mcp.test_connection(&name).await?;
    Ok(Json(result))
}

/// Import MCP servers from Claude Desktop configuration
#[utoipa::path(
    post,
    path = "/api/mcp/import",
    responses(
        (status = 200, description = "Import completed", body = ImportResult),
        (status = 500, description = "Internal server error")
    ),
    tag = "mcp"
)]
async fn import_from_claude_desktop(
    State(mcp): State<Arc<MCPService>>,
) -> Result<Json<ImportResult>> {
    let result = mcp.import_from_claude_desktop().await?;
    Ok(Json(result))
}