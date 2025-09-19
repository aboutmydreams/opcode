use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// CC Agent stored in the database
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Agent {
    pub id: Option<i64>,
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub hooks: Option<String>, // JSON string of hooks configuration
    pub created_at: String,
    pub updated_at: String,
}

/// Agent execution run
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AgentRun {
    pub id: Option<i64>,
    pub agent_id: i64,
    pub agent_name: String,
    pub agent_icon: String,
    pub task: String,
    pub model: String,
    pub project_path: String,
    pub session_id: String, // UUID session ID from Claude Code
    pub created_at: String,
    pub status: AgentRunStatus,
    pub output: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum AgentRunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Request to create a new agent
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAgentRequest {
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub hooks: Option<String>,
}

/// Request to run an agent
#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct RunAgentRequest {
    pub agent_id: i64,
    pub task: String,
    pub project_path: String,
    pub model: Option<String>, // Optional model override
}

/// Request to update an existing agent
#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAgentRequest {
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub hooks: Option<String>,
}

/// Agent run with metrics and real-time data
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AgentRunWithMetrics {
    pub id: Option<i64>,
    pub agent_id: i64,
    pub agent_name: String,
    pub agent_icon: String,
    pub task: String,
    pub model: String,
    pub project_path: String,
    pub session_id: String,
    pub status: String,
    pub pid: Option<i64>,
    pub duration_ms: Option<i64>,
    pub total_tokens: Option<i64>,
    pub process_started_at: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub output: Option<String>, // Real-time JSONL content
}

/// Agent export format for import/export
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AgentExport {
    pub version: i32,
    pub exported_at: String,
    pub agent: AgentExportData,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AgentExportData {
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub hooks: Option<String>,
}

/// GitHub agent file information
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GitHubAgentFile {
    pub name: String,
    pub path: String,
    pub download_url: String,
    pub size: i64,
    pub sha: String,
}

/// Claude installation information
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ClaudeInstallation {
    pub path: String,
    pub version: Option<String>,
    pub source: String,
    pub installation_type: String,
}