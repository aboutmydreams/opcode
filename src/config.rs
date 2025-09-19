use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub claude: ClaudeConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout: u64, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub max_connections: u32,
    pub connection_timeout: u64, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub binary_path: Option<PathBuf>,
    pub projects_dir: PathBuf,
    pub max_concurrent_sessions: usize,
    pub session_timeout: u64, // seconds
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            binary_path: None,
            projects_dir: dirs::home_dir()
                .unwrap_or_default()
                .join(".claude")
                .join("projects"),
            max_concurrent_sessions: 10,
            session_timeout: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub jwt_secret: String,
    pub token_expiry: u64, // seconds
    pub api_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<PathBuf>,
    pub json_format: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                max_connections: 1000,
                request_timeout: 30,
            },
            database: DatabaseConfig {
                path: dirs::home_dir()
                    .unwrap_or_default()
                    .join(".opcode")
                    .join("api-server.db"),
                max_connections: 10,
                connection_timeout: 30,
            },
            claude: ClaudeConfig {
                binary_path: None,
                projects_dir: dirs::home_dir()
                    .unwrap_or_default()
                    .join(".claude")
                    .join("projects"),
                max_concurrent_sessions: 10,
                session_timeout: 3600,
            },
            auth: AuthConfig {
                enabled: false,
                jwt_secret: "your-secret-key-change-in-production".to_string(),
                token_expiry: 86400, // 24 hours
                api_keys: vec![],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
                json_format: false,
            },
        }
    }
}

impl AppConfig {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path).required(false))
            .add_source(config::Environment::with_prefix("OPCODE"))
            .build()?;

        let config = settings.try_deserialize()?;
        Ok(config)
    }

    pub fn load() -> anyhow::Result<Self> {
        // Try to load from multiple locations
        let config_paths = [
            "./config.yaml",
            "./config.yml", 
            "~/.opcode/config.yaml",
            "/etc/opcode/config.yaml",
        ];

        for path in &config_paths {
            if let Ok(config) = Self::from_file(path) {
                tracing::info!("Loaded configuration from: {}", path);
                return Ok(config);
            }
        }

        tracing::info!("No configuration file found, using defaults");
        Ok(Self::default())
    }
}