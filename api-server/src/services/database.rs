use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::{
    agent::{Agent, CreateAgentRequest},
    claude::SessionRecord,
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
}