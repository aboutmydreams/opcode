use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum AppError {
    NotFound { resource: String, id: String },
    InvalidInput { field: String, message: String },
    DatabaseError(rusqlite::Error),
    ClaudeError(String),
    AgentError(String),
    McpError(String),
    InternalError(String),
    Unauthorized,
    Forbidden,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound { resource, id } => {
                write!(f, "{} with id {} not found", resource, id)
            }
            AppError::InvalidInput { field, message } => {
                write!(f, "Invalid input for field '{}': {}", field, message)
            }
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::ClaudeError(e) => write!(f, "Claude error: {}", e),
            AppError::AgentError(e) => write!(f, "Agent error: {}", e),
            AppError::McpError(e) => write!(f, "MCP error: {}", e),
            AppError::InternalError(e) => write!(f, "Internal error: {}", e),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Forbidden => write!(f, "Forbidden"),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            AppError::NotFound { resource, id } => (
                StatusCode::NOT_FOUND,
                format!("{}_NOT_FOUND", resource.to_uppercase()),
                self.to_string(),
                Some(serde_json::json!({ "resource": resource, "id": id })),
            ),
            AppError::InvalidInput { field, .. } => (
                StatusCode::BAD_REQUEST,
                "INVALID_INPUT".to_string(),
                self.to_string(),
                Some(serde_json::json!({ "field": field })),
            ),
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR".to_string(),
                "Database operation failed".to_string(),
                None,
            ),
            AppError::ClaudeError(_) => (
                StatusCode::BAD_GATEWAY,
                "CLAUDE_ERROR".to_string(),
                self.to_string(),
                None,
            ),
            AppError::AgentError(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "AGENT_ERROR".to_string(),
                self.to_string(),
                None,
            ),
            AppError::McpError(_) => (
                StatusCode::BAD_GATEWAY,
                "MCP_ERROR".to_string(),
                self.to_string(),
                None,
            ),
            AppError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR".to_string(),
                "An internal error occurred".to_string(),
                None,
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED".to_string(),
                "Authentication required".to_string(),
                None,
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN".to_string(),
                "Access denied".to_string(),
                None,
            ),
        };

        let api_error = ApiError {
            code,
            message,
            details,
        };

        tracing::error!("API Error: {} - {}", status, self);

        (status, Json(serde_json::json!({ "error": api_error }))).into_response()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::InternalError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;