use anyhow::Result;
use base64::engine::Engine;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use uuid::Uuid;

use crate::models::claude::{Project, StartSessionRequest, SessionRecord};
use crate::services::DatabaseService;

pub struct ClaudeService {
    claude_binary_path: String,
    db_service: Arc<DatabaseService>,
}

impl ClaudeService {
    pub fn new(db_service: Arc<DatabaseService>) -> Result<Self> {
        let binary_path = Self::find_claude_binary()?;
        Ok(ClaudeService {
            claude_binary_path: binary_path,
            db_service,
        })
    }

    fn find_claude_binary() -> Result<String> {
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
                PathBuf::from(path)
            };

            if expanded_path.exists() {
                return Ok(expanded_path.to_string_lossy().to_string());
            }
        }

        Err(anyhow::anyhow!("Claude binary not found. Please install Claude Code CLI."))
    }

    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let claude_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".claude")
            .join("projects");

        if !claude_dir.exists() {
            return Ok(vec![]);
        }

        let mut projects = Vec::new();

        for entry in std::fs::read_dir(&claude_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(project_id) = path.file_name().and_then(|n| n.to_str()) {
                    // Decode project path from directory name
                    let decoded_path = base64::engine::general_purpose::STANDARD.decode(project_id)
                        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| base64::DecodeError::InvalidByte(0, 0)))
                        .unwrap_or_else(|_| project_id.to_string());

                    // Get sessions
                    let mut sessions = Vec::new();
                    let mut most_recent_session = None;

                    if let Ok(session_entries) = std::fs::read_dir(&path) {
                        for session_entry in session_entries {
                            if let Ok(session_entry) = session_entry {
                                let session_path = session_entry.path();
                                if session_path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                                    if let Some(session_name) = session_path.file_stem().and_then(|n| n.to_str()) {
                                        sessions.push(session_name.to_string());

                                        // Update most recent session
                                        if let Ok(metadata) = session_path.metadata() {
                                            if let Ok(created) = metadata.created() {
                                                if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                                                    let timestamp = duration.as_secs();
                                                    if most_recent_session.is_none() || most_recent_session.unwrap() < timestamp {
                                                        most_recent_session = Some(timestamp);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Get project creation time
                    let created_at = path.metadata()
                        .and_then(|meta| meta.created())
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
                        .map(|duration| duration.as_secs())
                        .unwrap_or(0);

                    projects.push(Project {
                        id: project_id.to_string(),
                        path: decoded_path,
                        sessions,
                        created_at,
                        most_recent_session,
                    });
                }
            }
        }

        // Sort by most recent session first, then by creation time
        projects.sort_by(|a, b| {
            match (a.most_recent_session, b.most_recent_session) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.created_at.cmp(&a.created_at),
            }
        });

        Ok(projects)
    }

    pub async fn start_session(&self, request: StartSessionRequest) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        
        let mut cmd = Command::new(&self.claude_binary_path);
        cmd.arg("--project-path")
           .arg(&request.project_path);

        if let Some(model) = &request.model {
            cmd.arg("--model").arg(model);
        }

        if let Some(args) = &request.additional_args {
            cmd.args(args);
        }

        // Start Claude process but don't wait for it to complete
        let _child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // In a real implementation, you would manage the process lifecycle
        // For now, we just return the session ID
        Ok(session_id)
    }

    // Session management methods
    pub async fn list_sessions(&self, project_path: Option<&str>) -> Result<Vec<SessionRecord>> {
        self.db_service.get_sessions(project_path)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>> {
        self.db_service.get_session(session_id)
    }
}