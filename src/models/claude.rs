use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a project in the ~/.claude/projects directory
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Project {
    /// The project ID (derived from the directory name)
    pub id: String,
    /// The original project path (decoded from the directory name)
    pub path: String,
    /// List of session IDs (JSONL file names without extension)
    pub sessions: Vec<String>,
    /// Unix timestamp when the project directory was created
    pub created_at: u64,
    /// Unix timestamp of the most recent session (if any)
    pub most_recent_session: Option<u64>,
}

/// Represents a session with its metadata
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Session {
    /// The session ID (UUID)
    pub id: String,
    /// The project ID this session belongs to
    pub project_id: String,
    /// The project path
    pub project_path: String,
    /// Unix timestamp when the session was created
    pub created_at: u64,
    /// Number of messages in the session
    pub message_count: usize,
    /// Size of the session file in bytes
    pub file_size: u64,
}

/// Request to start a Claude session
#[derive(Debug, Deserialize, ToSchema)]
pub struct StartSessionRequest {
    pub project_path: String,
    pub prompt: String,
    pub model: Option<String>,
    pub session_type: Option<String>,
    pub session_id: Option<String>,
    pub additional_args: Option<Vec<String>>,
}

/// Request to create a new project
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

/// Request to update an existing project
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request to execute a command in Claude session
#[allow(dead_code)]
#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteCommandRequest {
    pub session_id: String,
    pub command: String,
}

/// Session record from database
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionRecord {
    pub id: i64,
    pub task: String,
    pub model: String,
    pub project_path: String,
    pub session_id: String,
    pub created_at: String,
    pub status: String,
    pub output: Option<String>,
}