use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MCPServer {
    /// Server name/identifier
    pub name: String,
    /// Transport type: "stdio" or "sse"
    pub transport: String,
    /// Command to execute (for stdio)
    pub command: Option<String>,
    /// Command arguments (for stdio) 
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// URL endpoint (for SSE)
    pub url: Option<String>,
    /// Configuration scope: "local", "project", or "user"
    pub scope: String,
    /// Whether the server is currently active
    pub is_active: bool,
    /// Server status
    pub status: ServerStatus,
}

/// Server status information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerStatus {
    /// Whether the server is running
    pub running: bool,
    /// Last error message if any
    pub error: Option<String>,
    /// Last checked timestamp
    pub last_checked: Option<u64>,
}

/// Request to add a new MCP server
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMCPServerRequest {
    /// Server name/identifier
    pub name: String,
    /// Transport type: "stdio" or "sse"
    pub transport: String,
    /// Command to execute (for stdio)
    pub command: Option<String>,
    /// Command arguments (for stdio)
    pub args: Option<Vec<String>>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// URL endpoint (for SSE)
    pub url: Option<String>,
    /// Configuration scope: "local", "project", or "user"
    pub scope: Option<String>,
}

/// Request to update an MCP server
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMCPServerRequest {
    /// Transport type: "stdio" or "sse"
    pub transport: Option<String>,
    /// Command to execute (for stdio)
    pub command: Option<String>,
    /// Command arguments (for stdio)
    pub args: Option<Vec<String>>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// URL endpoint (for SSE)
    pub url: Option<String>,
    /// Configuration scope: "local", "project", or "user"
    pub scope: Option<String>,
    /// Whether the server is currently active
    pub is_active: Option<bool>,
}

/// Result of adding/updating a server
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MCPServerResult {
    pub success: bool,
    pub message: String,
    pub server_name: Option<String>,
}

/// Import result for multiple servers
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportResult {
    pub imported_count: u32,
    pub failed_count: u32,
    pub servers: Vec<ImportServerResult>,
}

/// Result for individual server import
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportServerResult {
    pub name: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Connection test result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub response_time_ms: Option<u64>,
    pub details: Option<serde_json::Value>,
}

/// MCP configuration for project scope (.mcp.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPProjectConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, MCPServerConfig>,
}

/// Individual server configuration in .mcp.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}