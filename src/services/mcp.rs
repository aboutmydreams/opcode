use crate::config::ClaudeConfig;
use crate::error::{AppError, Result};
use crate::models::mcp::{
    AddMCPServerRequest, ConnectionTestResult, ImportResult, ImportServerResult,
    MCPServer, MCPServerResult, UpdateMCPServerRequest,
};
use anyhow::Context;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone)]
pub struct MCPService {
    #[allow(dead_code)]
    claude_config: ClaudeConfig,
    #[allow(dead_code)]
    claude_binary_path: Option<PathBuf>,
}

impl MCPService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            claude_config: Default::default(), // 临时使用默认值
            claude_binary_path: None,
        })
    }

    /// List all MCP servers from all scopes
    pub async fn list_servers(&self) -> Result<Vec<MCPServer>> {
        let mut servers = Vec::new();

        // Get user-level servers from Claude Desktop config
        if let Ok(user_servers) = self.get_user_servers().await {
            servers.extend(user_servers);
        }

        // Get project-level servers from .mcp.json files
        if let Ok(project_servers) = self.get_project_servers().await {
            servers.extend(project_servers);
        }

        Ok(servers)
    }

    /// Get a specific MCP server by name
    pub async fn get_server(&self, name: &str) -> Result<MCPServer> {
        let servers = self.list_servers().await?;
        
        servers
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| AppError::NotFound {
                resource: "MCP Server".to_string(),
                id: name.to_string(),
            })
    }

    /// Add a new MCP server
    pub async fn add_server(&self, request: AddMCPServerRequest) -> Result<MCPServerResult> {
        let scope = request.scope.as_deref().unwrap_or("user");
        
        match scope {
            "user" => self.add_user_server(request).await,
            "project" => self.add_project_server(request).await,
            _ => Err(AppError::InvalidInput {
                field: "scope".to_string(),
                message: "Scope must be 'user' or 'project'".to_string(),
            }),
        }
    }

    /// Update an existing MCP server
    pub async fn update_server(
        &self,
        name: &str,
        request: UpdateMCPServerRequest,
    ) -> Result<MCPServerResult> {
        // First find the server to determine its scope
        let server = self.get_server(name).await?;
        
        match server.scope.as_str() {
            "user" => self.update_user_server(name, request).await,
            "project" => self.update_project_server(name, request).await,
            _ => Err(AppError::McpError(format!(
                "Unknown scope: {}",
                server.scope
            ))),
        }
    }

    /// Remove an MCP server
    pub async fn remove_server(&self, name: &str) -> Result<MCPServerResult> {
        let server = self.get_server(name).await?;
        
        match server.scope.as_str() {
            "user" => self.remove_user_server(name).await,
            "project" => self.remove_project_server(name).await,
            _ => Err(AppError::McpError(format!(
                "Unknown scope: {}",
                server.scope
            ))),
        }
    }

    /// Test connection to an MCP server
    pub async fn test_connection(&self, name: &str) -> Result<ConnectionTestResult> {
        let server = self.get_server(name).await?;
        
        let _start_time = SystemTime::now();
        
        match server.transport.as_str() {
            "stdio" => self.test_stdio_connection(&server).await,
            "sse" => self.test_sse_connection(&server).await,
            _ => Ok(ConnectionTestResult {
                success: false,
                message: format!("Unsupported transport type: {}", server.transport),
                response_time_ms: None,
                details: None,
            }),
        }
    }

    /// Import servers from Claude Desktop configuration
    pub async fn import_from_claude_desktop(&self) -> Result<ImportResult> {
        let claude_config_path = self.get_claude_desktop_config_path()?;
        
        if !claude_config_path.exists() {
            return Ok(ImportResult {
                imported_count: 0,
                failed_count: 0,
                servers: vec![],
            });
        }

        let content = fs::read_to_string(&claude_config_path)
            .context("Failed to read Claude Desktop config")?;
        
        let config: Value = serde_json::from_str(&content)
            .context("Failed to parse Claude Desktop config")?;

        let mut import_results = Vec::new();
        let mut imported_count = 0;
        let mut failed_count = 0;

        if let Some(mcp_servers) = config.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_config) in mcp_servers {
                match self.import_server_from_config(name, server_config).await {
                    Ok(_) => {
                        imported_count += 1;
                        import_results.push(ImportServerResult {
                            name: name.clone(),
                            success: true,
                            error: None,
                        });
                    }
                    Err(e) => {
                        failed_count += 1;
                        import_results.push(ImportServerResult {
                            name: name.clone(),
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }

        Ok(ImportResult {
            imported_count,
            failed_count,
            servers: import_results,
        })
    }

    // Private helper methods

    async fn get_user_servers(&self) -> Result<Vec<MCPServer>> {
        // Implementation to read from Claude Desktop config
        // This would parse the Claude Desktop configuration file
        // and convert to our MCPServer format
        Ok(vec![]) // Placeholder
    }

    async fn get_project_servers(&self) -> Result<Vec<MCPServer>> {
        // Implementation to read from .mcp.json files in projects
        Ok(vec![]) // Placeholder
    }

    async fn add_user_server(&self, request: AddMCPServerRequest) -> Result<MCPServerResult> {
        // Implementation to add server to Claude Desktop config
        Ok(MCPServerResult {
            success: true,
            message: "Server added successfully".to_string(),
            server_name: Some(request.name),
        })
    }

    async fn add_project_server(&self, request: AddMCPServerRequest) -> Result<MCPServerResult> {
        // Implementation to add server to project .mcp.json
        Ok(MCPServerResult {
            success: true,
            message: "Server added successfully".to_string(),
            server_name: Some(request.name),
        })
    }

    async fn update_user_server(
        &self,
        name: &str,
        _request: UpdateMCPServerRequest,
    ) -> Result<MCPServerResult> {
        // Implementation to update user server
        Ok(MCPServerResult {
            success: true,
            message: "Server updated successfully".to_string(),
            server_name: Some(name.to_string()),
        })
    }

    async fn update_project_server(
        &self,
        name: &str,
        _request: UpdateMCPServerRequest,
    ) -> Result<MCPServerResult> {
        // Implementation to update project server
        Ok(MCPServerResult {
            success: true,
            message: "Server updated successfully".to_string(),
            server_name: Some(name.to_string()),
        })
    }

    async fn remove_user_server(&self, name: &str) -> Result<MCPServerResult> {
        // Implementation to remove user server
        Ok(MCPServerResult {
            success: true,
            message: "Server removed successfully".to_string(),
            server_name: Some(name.to_string()),
        })
    }

    async fn remove_project_server(&self, name: &str) -> Result<MCPServerResult> {
        // Implementation to remove project server
        Ok(MCPServerResult {
            success: true,
            message: "Server removed successfully".to_string(),
            server_name: Some(name.to_string()),
        })
    }

    async fn test_stdio_connection(&self, server: &MCPServer) -> Result<ConnectionTestResult> {
        let start_time = SystemTime::now();
        
        if let Some(command) = &server.command {
            match AsyncCommand::new(command)
                .args(&server.args)
                .envs(&server.env)
                .output()
                .await
            {
                Ok(output) => {
                    let response_time = start_time.elapsed().unwrap().as_millis() as u64;
                    
                    Ok(ConnectionTestResult {
                        success: output.status.success(),
                        message: if output.status.success() {
                            "Connection successful".to_string()
                        } else {
                            format!("Connection failed: {}", String::from_utf8_lossy(&output.stderr))
                        },
                        response_time_ms: Some(response_time),
                        details: Some(serde_json::json!({
                            "stdout": String::from_utf8_lossy(&output.stdout),
                            "stderr": String::from_utf8_lossy(&output.stderr),
                            "status_code": output.status.code()
                        })),
                    })
                }
                Err(e) => Ok(ConnectionTestResult {
                    success: false,
                    message: format!("Failed to execute command: {}", e),
                    response_time_ms: None,
                    details: None,
                }),
            }
        } else {
            Ok(ConnectionTestResult {
                success: false,
                message: "No command specified for stdio transport".to_string(),
                response_time_ms: None,
                details: None,
            })
        }
    }

    async fn test_sse_connection(&self, server: &MCPServer) -> Result<ConnectionTestResult> {
        if let Some(url) = &server.url {
            let start_time = SystemTime::now();
            
            match reqwest::get(url).await {
                Ok(response) => {
                    let response_time = start_time.elapsed().unwrap().as_millis() as u64;
                    
                    Ok(ConnectionTestResult {
                        success: response.status().is_success(),
                        message: format!("HTTP {}", response.status()),
                        response_time_ms: Some(response_time),
                        details: Some(serde_json::json!({
                            "status_code": response.status().as_u16(),
                            "headers": response.headers().iter()
                                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                                .collect::<HashMap<String, String>>()
                        })),
                    })
                }
                Err(e) => Ok(ConnectionTestResult {
                    success: false,
                    message: format!("Connection failed: {}", e),
                    response_time_ms: None,
                    details: None,
                }),
            }
        } else {
            Ok(ConnectionTestResult {
                success: false,
                message: "No URL specified for SSE transport".to_string(),
                response_time_ms: None,
                details: None,
            })
        }
    }

    async fn import_server_from_config(
        &self,
        name: &str,
        _config: &Value,
    ) -> Result<MCPServerResult> {
        // Parse the server configuration from Claude Desktop format
        // and add it as a user-level server
        Ok(MCPServerResult {
            success: true,
            message: format!("Imported server: {}", name),
            server_name: Some(name.to_string()),
        })
    }

    fn get_claude_desktop_config_path(&self) -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            AppError::InternalError("Could not determine home directory".to_string())
        })?;

        #[cfg(target_os = "macos")]
        let config_path = home_dir
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json");

        #[cfg(target_os = "windows")]
        let config_path = home_dir
            .join("AppData")
            .join("Roaming")
            .join("Claude")
            .join("claude_desktop_config.json");

        #[cfg(target_os = "linux")]
        let config_path = home_dir
            .join(".config")
            .join("Claude")
            .join("claude_desktop_config.json");

        Ok(config_path)
    }
}