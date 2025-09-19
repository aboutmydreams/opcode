use anyhow::Result;
use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::{
    agent::{Agent, CreateAgentRequest},
    claude::{SessionRecord, CreateProjectRequest, UpdateProjectRequest, Project},
};

pub struct DatabaseService {
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseService {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        
        let service = DatabaseService {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        service.init_database()?;
        Ok(service)
    }

    fn get_db_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".claude").join("opcode.db"))
    }

    fn init_database(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        // Create agents table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                icon TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                default_task TEXT,
                model TEXT NOT NULL,
                hooks TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create agent_runs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id INTEGER NOT NULL,
                agent_name TEXT NOT NULL,
                agent_icon TEXT NOT NULL,
                task TEXT NOT NULL,
                model TEXT NOT NULL,
                project_path TEXT NOT NULL,
                session_id TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT NOT NULL DEFAULT 'running',
                output TEXT,
                FOREIGN KEY (agent_id) REFERENCES agents (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create mcp_servers table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcp_servers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                command TEXT NOT NULL,
                args TEXT NOT NULL,
                env TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create slash_commands table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS slash_commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                command TEXT NOT NULL,
                description TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create projects table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_accessed_at DATETIME
            )",
            [],
        )?;

        Ok(())
    }

    // Agent operations
    pub fn create_agent(&self, request: CreateAgentRequest) -> Result<Agent> {
        let conn = self.connection.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO agents (name, icon, system_prompt, default_task, model, 
             hooks, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                request.name,
                request.icon,
                request.system_prompt,
                request.default_task,
                request.model,
                request.hooks,
                now,
                now
            ],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Agent {
            id: Some(id),
            name: request.name,
            icon: request.icon,
            system_prompt: request.system_prompt,
            default_task: request.default_task,
            model: request.model,
            hooks: request.hooks,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn get_agents(&self) -> Result<Vec<Agent>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, system_prompt, default_task, model,
             hooks, created_at, updated_at FROM agents ORDER BY created_at DESC"
        )?;

        let agents = stmt.query_map([], |row| {
            Ok(Agent {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                system_prompt: row.get(3)?,
                default_task: row.get(4)?,
                model: row.get(5)?,
                hooks: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>()?;

        Ok(agents)
    }

    pub fn get_agent(&self, id: i64) -> Result<Option<Agent>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, system_prompt, default_task, model,
             hooks, created_at, updated_at FROM agents WHERE id = ?1"
        )?;

        let agent = stmt.query_row([id], |row| {
            Ok(Agent {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                system_prompt: row.get(3)?,
                default_task: row.get(4)?,
                model: row.get(5)?,
                hooks: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }).optional()?;

        Ok(agent)
    }

    pub fn delete_agent(&self, id: i64) -> Result<bool> {
        let conn = self.connection.lock().unwrap();
        let affected = conn.execute("DELETE FROM agents WHERE id = ?1", [id])?;
        Ok(affected > 0)
    }

    // Session operations
    pub fn create_session_record(&self, session_id: &str, task: &str, project_path: &str, model: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        
        // Create a dummy agent first if it doesn't exist (agent_id = 1)
        let _ = conn.execute(
            "INSERT OR IGNORE INTO agents (id, name, icon, system_prompt, model, created_at, updated_at)
             VALUES (1, 'Claude Code', '🤖', 'You are Claude Code CLI assistant', 'claude-3-5-sonnet-20241022', ?1, ?2)",
            params![now, now],
        );
        
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, agent_icon, task, model, project_path, session_id, created_at)
             VALUES (1, 'Claude Code', '🤖', ?1, ?2, ?3, ?4, ?5)",
            params![task, model, project_path, session_id, now],
        )?;

        Ok(())
    }

    pub fn update_session_status(&self, session_id: &str, status: &str, output: Option<&str>) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        
        if let Some(output) = output {
            conn.execute(
                "UPDATE agent_runs SET status = ?1, output = ?2 WHERE session_id = ?3",
                params![status, output, session_id],
            )?;
        } else {
            conn.execute(
                "UPDATE agent_runs SET status = ?1 WHERE session_id = ?2",
                params![status, session_id],
            )?;
        }

        Ok(())
    }

    pub fn get_sessions(&self, project_path: Option<&str>) -> Result<Vec<SessionRecord>> {
        let conn = self.connection.lock().unwrap();
        
        let mut sessions = Vec::new();
        
        if let Some(path) = project_path {
            let mut stmt = conn.prepare(
                "SELECT id, task, model, project_path, session_id, created_at, status, output 
                 FROM agent_runs WHERE project_path = ? ORDER BY created_at DESC"
            )?;
            
            let session_iter = stmt.query_map([path], |row| {
                Ok(SessionRecord {
                    id: row.get(0)?,
                    task: row.get(1)?,
                    model: row.get(2)?,
                    project_path: row.get(3)?,
                    session_id: row.get(4)?,
                    created_at: row.get(5)?,
                    status: row.get(6)?,
                    output: row.get(7)?,
                })
            })?;
            
            for session in session_iter {
                sessions.push(session?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, task, model, project_path, session_id, created_at, status, output 
                 FROM agent_runs ORDER BY created_at DESC"
            )?;
            
            let session_iter = stmt.query_map([], |row| {
                Ok(SessionRecord {
                    id: row.get(0)?,
                    task: row.get(1)?,
                    model: row.get(2)?,
                    project_path: row.get(3)?,
                    session_id: row.get(4)?,
                    created_at: row.get(5)?,
                    status: row.get(6)?,
                    output: row.get(7)?,
                })
            })?;
            
            for session in session_iter {
                sessions.push(session?);
            }
        }
        
        Ok(sessions)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, task, model, project_path, session_id, created_at, status, output 
             FROM agent_runs WHERE session_id = ?1"
        )?;

        let session = stmt.query_row([session_id], |row| {
            Ok(SessionRecord {
                id: row.get(0)?,
                task: row.get(1)?,
                model: row.get(2)?,
                project_path: row.get(3)?,
                session_id: row.get(4)?,
                created_at: row.get(5)?,
                status: row.get(6)?,
                output: row.get(7)?,
            })
        }).optional()?;

        Ok(session)
    }

    // Project operations
    pub fn create_project(&self, request: CreateProjectRequest) -> Result<Project> {
        let conn = self.connection.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        
        // Generate project ID using base64 encoding of the path (to match Claude's convention)
        let project_id = base64::engine::general_purpose::STANDARD.encode(&request.path);
        
        // Validate that the parent directory exists
        let parent_path = std::path::Path::new(&request.path).parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid path: no parent directory"))?;
        
        if !parent_path.exists() {
            return Err(anyhow::anyhow!("Parent directory does not exist: {}", parent_path.display()));
        }
        
        // Insert into database
        conn.execute(
            "INSERT INTO projects (id, name, path, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                project_id,
                request.name,
                request.path,
                request.description,
                now,
                now
            ],
        )?;
        
        // Create the actual project directory
        let project_path = std::path::Path::new(&request.path);
        if !project_path.exists() {
            std::fs::create_dir_all(&project_path)?;
        }
        
        // Create the Claude project directory structure
        self.create_claude_project_directory(&project_id, &request.path)?;
        
        Ok(Project {
            id: project_id,
            path: request.path,
            sessions: vec![], // New project has no sessions
            created_at: chrono::Utc::now().timestamp() as u64,
            most_recent_session: None,
        })
    }
    
    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, path, description, created_at, updated_at, last_accessed_at 
             FROM projects ORDER BY updated_at DESC"
        )?;
        
        let project_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let path: String = row.get(2)?;
            let created_at_str: String = row.get(4)?;
            
            // Parse created_at to timestamp
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.timestamp() as u64)
                .unwrap_or(0);
            
            Ok((id, path, created_at))
        })?;
        
        let mut projects = Vec::new();
        for result in project_iter {
            let (id, path, created_at) = result?;
            
            // Get sessions for this project (this will be empty for database-managed projects)
            // But we still check the Claude directory structure for compatibility
            let sessions = self.get_project_sessions(&id)?;
            let most_recent_session = self.get_most_recent_session_time(&id)?;
            
            projects.push(Project {
                id,
                path,
                sessions,
                created_at,
                most_recent_session,
            });
        }
        
        Ok(projects)
    }
    
    pub fn update_project(&self, project_id: &str, request: UpdateProjectRequest) -> Result<Option<Project>> {
        let conn = self.connection.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        
        // First check if project exists
        let existing: Option<(String, String, String)> = conn.query_row(
            "SELECT id, name, path FROM projects WHERE id = ?1",
            params![project_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        ).optional()?;
        
        let (id, current_name, path) = match existing {
            Some(project) => project,
            None => return Ok(None), // Project not found
        };
        
        // Build update query dynamically based on provided fields
        let mut update_parts = Vec::new();
        let mut update_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        
        let name = request.name.as_ref().unwrap_or(&current_name);
        let description = request.description.as_deref().unwrap_or("");
        
        update_parts.push("name = ?1");
        update_params.push(name);
        
        update_parts.push("description = ?2");
        update_params.push(&description);
        
        update_parts.push("updated_at = ?3");
        update_params.push(&now);
        
        // Add project_id as the last parameter for WHERE clause
        update_params.push(&project_id);
        
        let update_query = format!(
            "UPDATE projects SET {} WHERE id = ?{}",
            update_parts.join(", "),
            update_params.len()
        );
        
        conn.execute(&update_query, &update_params[..])?;
        
        // Return updated project
        let sessions = self.get_project_sessions(&id)?;
        let most_recent_session = self.get_most_recent_session_time(&id)?;
        let created_at = chrono::Utc::now().timestamp() as u64; // This should be fetched from DB in real implementation
        
        Ok(Some(Project {
            id,
            path,
            sessions,
            created_at,
            most_recent_session,
        }))
    }
    
    pub fn delete_project(&self, project_id: &str) -> Result<bool> {
        let conn = self.connection.lock().unwrap();
        
        // First get the project path before deleting from database
        let project_path: Option<String> = conn.query_row(
            "SELECT path FROM projects WHERE id = ?1",
            params![project_id],
            |row| Ok(row.get(0)?)
        ).optional()?;
        
        let rows_affected = conn.execute(
            "DELETE FROM projects WHERE id = ?1",
            params![project_id],
        )?;
        
        // Remove the Claude project directory and actual project directory if they exist
        if rows_affected > 0 {
            self.remove_claude_project_directory(project_id)?;
            
            // Also remove the actual project directory if it exists
            if let Some(path) = project_path {
                let project_dir = std::path::Path::new(&path);
                if project_dir.exists() && project_dir.is_dir() {
                    std::fs::remove_dir_all(&project_dir)?;
                }
            }
        }
        
        Ok(rows_affected > 0)
    }
    
    fn create_claude_project_directory(&self, project_id: &str, _project_path: &str) -> Result<()> {
        let claude_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".claude")
            .join("projects")
            .join(project_id);
        
        std::fs::create_dir_all(&claude_dir)?;
        Ok(())
    }
    
    fn remove_claude_project_directory(&self, project_id: &str) -> Result<()> {
        let claude_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".claude")
            .join("projects")
            .join(project_id);
        
        if claude_dir.exists() {
            std::fs::remove_dir_all(&claude_dir)?;
        }
        Ok(())
    }
    
    fn get_project_sessions(&self, project_id: &str) -> Result<Vec<String>> {
        let claude_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".claude")
            .join("projects")
            .join(project_id);
        
        let mut sessions = Vec::new();
        
        if claude_dir.exists() {
            for entry in std::fs::read_dir(&claude_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                    if let Some(session_name) = path.file_stem().and_then(|n| n.to_str()) {
                        sessions.push(session_name.to_string());
                    }
                }
            }
        }
        
        Ok(sessions)
    }
    
    fn get_most_recent_session_time(&self, project_id: &str) -> Result<Option<u64>> {
        let claude_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".claude")
            .join("projects")
            .join(project_id);
        
        if !claude_dir.exists() {
            return Ok(None);
        }
        
        let mut most_recent = None;
        
        for entry in std::fs::read_dir(&claude_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                if let Ok(metadata) = path.metadata() {
                    if let Ok(created) = metadata.created() {
                        if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                            let timestamp = duration.as_secs();
                            if most_recent.is_none() || most_recent.unwrap() < timestamp {
                                most_recent = Some(timestamp);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(most_recent)
    }
}