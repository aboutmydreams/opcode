use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl ApiResponse<()> {
    pub fn success_no_data() -> Self {
        Self {
            success: true,
            data: None,
            message: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn success_with_message_only(message: String) -> Self {
        Self {
            success: true,
            data: None,
            message: Some(message),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

// Convenience function for simple success responses
pub fn success<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse::success(data)
}

// Convenience function for success with message
pub fn success_with_message<T: Serialize>(data: T, message: impl Into<String>) -> ApiResponse<T> {
    ApiResponse::success_with_message(data, message.into())
}

// Convenience function for no-data success
pub fn success_no_data() -> ApiResponse<()> {
    ApiResponse::success_no_data()
}

// Convenience function for message-only success
pub fn success_message(message: impl Into<String>) -> ApiResponse<()> {
    ApiResponse::success_with_message_only(message.into())
}