use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Path,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::{
    collections::HashMap,
    process::Stdio,
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{broadcast, Mutex},
};
use tracing::{error, info, warn};
use uuid::Uuid;

/// WebSocket manager to handle Claude Code sessions
#[derive(Clone)]
pub struct WebSocketManager {
    /// Active WebSocket sessions by session ID
    sessions: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    /// Running Claude processes by session ID
    processes: Arc<Mutex<HashMap<String, tokio::process::Child>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new WebSocket session
    pub async fn register_session(&self, session_id: String) -> broadcast::Receiver<String> {
        let (tx, rx) = broadcast::channel(1000);
        self.sessions.lock().await.insert(session_id, tx);
        rx
    }

    /// Send message to a specific session
    pub async fn send_to_session(&self, session_id: &str, message: String) {
        if let Some(tx) = self.sessions.lock().await.get(session_id) {
            let _ = tx.send(message);
        }
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) {
        self.sessions.lock().await.remove(session_id);
        if let Some(mut process) = self.processes.lock().await.remove(session_id) {
            let _ = process.kill().await;
        }
    }

    /// Store a running process for a session
    pub async fn store_process(&self, session_id: String, process: tokio::process::Child) {
        self.processes.lock().await.insert(session_id, process);
    }

    /// Cancel a running process for a session
    pub async fn cancel_process(&self, session_id: &str) -> bool {
        if let Some(mut process) = self.processes.lock().await.remove(session_id) {
            match process.kill().await {
                Ok(_) => {
                    info!("Successfully killed process for session: {}", session_id);
                    true
                }
                Err(e) => {
                    error!("Failed to kill process for session {}: {}", session_id, e);
                    false
                }
            }
        } else {
            warn!("No process found for session: {}", session_id);
            false
        }
    }
}

/// Request to execute Claude Code
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub project_path: String,
    pub prompt: String,
    pub model: String,
    pub session_type: SessionType,
    pub session_id: Option<String>, // For resume operations
}

/// Type of Claude Code session
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    New,      // Start new session with -p
    Continue, // Continue existing session with -c
    Resume,   // Resume specific session with --resume
}

/// Query parameters for WebSocket connection
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub session_id: String,
}

/// WebSocket routes
pub fn websocket_router() -> Router<Arc<WebSocketManager>> {
    Router::new()
        .route("/ws/claude/:session_id", get(claude_websocket_handler))
        .route("/claude/execute", axum::routing::post(execute_claude_code))
        .route("/claude/cancel/:session_id", axum::routing::post(cancel_execution))
}

/// WebSocket handler for Claude Code sessions
pub async fn claude_websocket_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    info!("WebSocket connection established for session: {}", session_id);
    
    ws.on_upgrade(move |socket| handle_websocket(socket, session_id, manager))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, session_id: String, manager: Arc<WebSocketManager>) {
    let mut receiver = manager.register_session(session_id.clone()).await;
    let (mut sender, mut receiver_ws) = socket.split();

    // Task to forward messages from broadcast to WebSocket
    let session_id_clone = session_id.clone();
    let forward_task = tokio::spawn(async move {
        while let Ok(message) = receiver.recv().await {
            if sender.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
        info!("WebSocket sender task ended for session: {}", session_id_clone);
    });

    // Task to handle incoming WebSocket messages (if needed)
    let session_id_clone = session_id.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver_ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Handle incoming commands if needed
                    info!("Received WebSocket message for session {}: {}", session_id_clone, text);
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket closed for session: {}", session_id_clone);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error for session {}: {}", session_id_clone, e);
                    break;
                }
                _ => {}
            }
        }
        info!("WebSocket receiver task ended for session: {}", session_id_clone);
    });

    // Wait for either task to complete
    tokio::select! {
        _ = forward_task => {},
        _ = receive_task => {},
    }

    // Cleanup
    manager.remove_session(&session_id).await;
    info!("WebSocket connection cleaned up for session: {}", session_id);
}

/// Execute Claude Code command
pub async fn execute_claude_code(
    State(manager): State<Arc<WebSocketManager>>,
    axum::extract::Json(request): axum::extract::Json<ExecuteRequest>,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    info!("Executing Claude Code: {:?}", request);

    // Generate session ID if not provided
    let session_id = match request.session_type {
        SessionType::Resume => request.session_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        _ => Uuid::new_v4().to_string(),
    };

    // Find Claude binary
    let claude_path = find_claude_binary().map_err(|e| {
        error!("Failed to find Claude binary: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Build command arguments
    let mut args = Vec::new();
    match request.session_type {
        SessionType::New => {
            args.push("-p".to_string());
            args.push(request.prompt.clone());
        }
        SessionType::Continue => {
            args.push("-c".to_string());
            args.push("-p".to_string());
            args.push(request.prompt.clone());
        }
        SessionType::Resume => {
            args.push("--resume".to_string());
            args.push(session_id.clone());
            args.push("-p".to_string());
            args.push(request.prompt.clone());
        }
    }

    args.extend([
        "--model".to_string(),
        request.model.clone(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--dangerously-skip-permissions".to_string(),
    ]);

    // Create and spawn command
    let mut cmd = Command::new(claude_path);
    cmd.args(args)
        .current_dir(&request.project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        error!("Failed to spawn Claude process: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get stdout and stderr
    let stdout = child.stdout.take().ok_or_else(|| {
        error!("Failed to get stdout");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        error!("Failed to get stderr");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let pid = child.id().unwrap_or(0);
    info!("Spawned Claude process with PID: {} for session: {}", pid, session_id);

    // Store the process
    manager.store_process(session_id.clone(), child).await;

    // Spawn tasks to read stdout and stderr
    let manager_clone = manager.clone();
    let session_id_clone = session_id.clone();
    tokio::spawn(async move {
        let mut stdout_reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            manager_clone.send_to_session(&session_id_clone, line).await;
        }
        info!("Stdout reading completed for session: {}", session_id_clone);
    });

    let manager_clone = manager.clone();
    let session_id_clone = session_id.clone();
    tokio::spawn(async move {
        let mut stderr_reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            let error_msg = serde_json::json!({
                "type": "error",
                "message": line
            });
            manager_clone.send_to_session(&session_id_clone, error_msg.to_string()).await;
        }
        info!("Stderr reading completed for session: {}", session_id_clone);
    });

    // Wait for process completion in background
    let manager_clone = manager.clone();
    let session_id_clone = session_id.clone();
    tokio::spawn(async move {
        // Remove from our process map and wait for completion
        if let Some(mut process) = manager_clone.processes.lock().await.remove(&session_id_clone) {
            match process.wait().await {
                Ok(status) => {
                    let completion_msg = serde_json::json!({
                        "type": "complete",
                        "success": status.success(),
                        "code": status.code()
                    });
                    manager_clone.send_to_session(&session_id_clone, completion_msg.to_string()).await;
                    info!("Claude process completed with status: {} for session: {}", status, session_id_clone);
                }
                Err(e) => {
                    let error_msg = serde_json::json!({
                        "type": "error",
                        "message": format!("Process error: {}", e)
                    });
                    manager_clone.send_to_session(&session_id_clone, error_msg.to_string()).await;
                    error!("Claude process error for session {}: {}", session_id_clone, e);
                }
            }
        }
    });

    Ok(axum::response::Json(serde_json::json!({
        "session_id": session_id,
        "status": "started",
        "websocket_url": format!("/ws/claude/{}", session_id)
    })))
}

/// Cancel Claude Code execution
pub async fn cancel_execution(
    Path(session_id): Path<String>,
    State(manager): State<Arc<WebSocketManager>>,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    info!("Cancelling execution for session: {}", session_id);

    let cancelled = manager.cancel_process(&session_id).await;

    // Send cancellation message to WebSocket clients
    let cancel_msg = serde_json::json!({
        "type": "cancelled",
        "session_id": session_id
    });
    manager.send_to_session(&session_id, cancel_msg.to_string()).await;

    Ok(axum::response::Json(serde_json::json!({
        "session_id": session_id,
        "cancelled": cancelled
    })))
}

/// Find Claude binary (similar to the Tauri version)
fn find_claude_binary() -> Result<String, String> {
    // Try to find claude binary in PATH
    if let Ok(path) = which::which("claude") {
        return Ok(path.to_string_lossy().to_string());
    }

    // Try common installation locations
    let common_paths = vec![
        "/usr/local/bin/claude",
        "/opt/homebrew/bin/claude",
        "~/.local/bin/claude",
    ];

    for path in common_paths {
        let expanded_path = if path.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                home.join(&path[2..])
            } else {
                continue;
            }
        } else {
            std::path::PathBuf::from(path)
        };

        if expanded_path.exists() {
            return Ok(expanded_path.to_string_lossy().to_string());
        }
    }

    Err("Claude binary not found. Please install Claude Code CLI.".to_string())
}