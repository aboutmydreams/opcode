pub mod agent;
pub mod claude;
pub mod mcp;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Storage usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StorageUsage {
    pub total_size_bytes: u64,
    pub total_files: usize,
    pub projects_count: usize,
    pub sessions_count: usize,
}

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpServer {
    pub id: Option<i64>,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create/update MCP server
#[derive(Debug, Deserialize, ToSchema)]
pub struct McpServerRequest {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub enabled: bool,
}

/// Slash command configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SlashCommand {
    pub id: Option<i64>,
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Request to create/update slash command
#[derive(Debug, Deserialize, ToSchema)]
pub struct SlashCommandRequest {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub enabled: bool,
}