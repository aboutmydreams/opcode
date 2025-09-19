use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use walkdir::WalkDir;

pub fn storage_router() -> Router {
    Router::new()
        .route("/storage/usage", get(get_storage_usage))
}

/// Get storage usage statistics
#[utoipa::path(
    get,
    path = "/api/storage/usage",
    responses(
        (status = 200, description = "Storage usage statistics", body = serde_json::Value)
    )
)]
async fn get_storage_usage() -> Result<Json<serde_json::Value>, StatusCode> {
    // Calculate storage usage from ~/.claude directory
    let claude_dir = match dirs::home_dir() {
        Some(home) => home.join(".claude"),
        None => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if !claude_dir.exists() {
        return Ok(Json(serde_json::json!({
            "total_size_bytes": 0,
            "total_files": 0,
            "projects_count": 0,
            "sessions_count": 0
        })));
    }

    let mut total_size = 0u64;
    let mut total_files = 0usize;
    let mut projects_count = 0usize;
    let mut sessions_count = 0usize;

    // Walk through the .claude directory
    if let Ok(walker) = WalkDir::new(&claude_dir).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in walker {
            if entry.file_type().is_file() {
                total_files += 1;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
                
                // Count sessions (JSONL files)
                if entry.path().extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                    sessions_count += 1;
                }
            } else if entry.file_type().is_dir() {
                // Count projects (directories in projects/)
                if let Some(parent) = entry.path().parent() {
                    if parent.file_name().and_then(|name| name.to_str()) == Some("projects") {
                        projects_count += 1;
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "total_size_bytes": total_size,
        "total_files": total_files,
        "projects_count": projects_count,
        "sessions_count": sessions_count
    })))
}