// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod checkpoint;
mod claude_binary;
mod commands;
mod process;
mod api;

use checkpoint::state::CheckpointState;
use commands::agents::{
    cleanup_finished_processes, create_agent, delete_agent, execute_agent, export_agent,
    export_agent_to_file, fetch_github_agent_content, fetch_github_agents, get_agent,
    get_agent_run, get_agent_run_with_real_time_metrics, get_claude_binary_path,
    get_live_session_output, get_session_output, get_session_status, import_agent,
    import_agent_from_file, import_agent_from_github, init_database, kill_agent_session,
    list_agent_runs, list_agent_runs_with_metrics, list_agents, list_claude_installations,
    list_running_sessions, load_agent_session_history, set_claude_binary_path, stream_session_output, update_agent, AgentDb,
};
use commands::claude::{
    cancel_claude_execution, check_auto_checkpoint, check_claude_version, cleanup_old_checkpoints,
    clear_checkpoint_manager, continue_claude_code, create_checkpoint, create_project, execute_claude_code,
    find_claude_md_files, fork_from_checkpoint, get_checkpoint_diff, get_checkpoint_settings,
    get_checkpoint_state_stats, get_claude_session_output, get_claude_settings, get_home_directory, get_project_sessions,
    get_recently_modified_files, get_session_timeline, get_system_prompt, list_checkpoints,
    list_directory_contents, list_projects, list_running_claude_sessions, load_session_history,
    open_new_session, read_claude_md_file, restore_checkpoint, resume_claude_code,
    save_claude_md_file, save_claude_settings, save_system_prompt, search_files,
    track_checkpoint_message, track_session_messages, update_checkpoint_settings,
    get_hooks_config, update_hooks_config, validate_hook_command,
    ClaudeProcessState,
};
use commands::mcp::{
    mcp_add, mcp_add_from_claude_desktop, mcp_add_json, mcp_get, mcp_get_server_status, mcp_list,
    mcp_read_project_config, mcp_remove, mcp_reset_project_choices, mcp_save_project_config,
    mcp_serve, mcp_test_connection,
};

use commands::usage::{
    get_session_stats, get_usage_by_date_range, get_usage_details, get_usage_stats,
};
use commands::storage::{
    storage_list_tables, storage_read_table, storage_update_row, storage_delete_row,
    storage_insert_row, storage_execute_sql, storage_reset_database,
};
use commands::proxy::{get_proxy_settings, save_proxy_settings, apply_proxy_settings};
use std::env;
use std::sync::Mutex;
// GUI-related imports commented out for API-only build
// use std::path::PathBuf;
// use tauri::Manager;
// #[cfg(target_os = "macos")]
// use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

// GUI state structures commented out for API-only build
// #[derive(Clone)]
// pub struct InitialProjectPath(pub std::sync::Arc<Mutex<Option<PathBuf>>>);

// GUI-related commands commented out for API-only build
// #[tauri::command]
// async fn get_initial_project_path(
//     state: tauri::State<'_, InitialProjectPath>,
// ) -> Result<Option<String>, String> {
//     match state.0.lock() {
//         Ok(mut path_opt) => {
//             // Return the path and clear it (one-time use)
//             if let Some(path) = path_opt.take() {
//                 Ok(Some(path.to_string_lossy().to_string()))
//             } else {
//                 Ok(None)
//             }
//         }
//         Err(e) => Err(format!("Failed to access initial project path: {}", e)),
//     }
// }


fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Initialize logger first
    env_logger::init();
    
    // Check for different modes
    if args.len() > 1 {
        match args[1].as_str() {
            "api" | "--api" => {
                // Start HTTP API server
                handle_api_mode(&args);
                return;
            }
            _ => {
                eprintln!("Error: Unknown command: {}", args[1]);
                eprintln!("Usage: opcode api [--port PORT]    (HTTP API server mode)");
                std::process::exit(1);
            }
        }
    }
    
    // API-only mode - no GUI available
    eprintln!("Error: This build only supports API mode.");
    eprintln!("Usage: opcode api [--port PORT]    (HTTP API server mode)");
    std::process::exit(1);
}

// GUI functions commented out for API-only build
// fn handle_cli_mode(args: &[String]) { ... }
// fn run_gui_app() { ... }
// fn run_gui_app_with_project(project_path: PathBuf) { ... }

fn handle_api_mode(args: &[String]) {
    println!("🚀 Starting Opcode HTTP API server...");
    
    // Parse command line arguments for port
    let mut port = 3001u16;
    let mut i = 2; // Skip "opcode" and "api"
    
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid port number: {}", args[i + 1]);
                        std::process::exit(1);
                    });
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a port number");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                eprintln!("Usage: opcode api [--port PORT]");
                std::process::exit(1);
            }
        }
    }
    
    println!("📡 Port: {}", port);
    
    // Initialize the runtime for async operations
    let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
        eprintln!("Failed to create async runtime: {}", e);
        std::process::exit(1);
    });
    
    rt.block_on(async {
        // Initialize database and state (similar to GUI app setup)
        let app_dir = dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".opcode");
        
        std::fs::create_dir_all(&app_dir).expect("Failed to create app directory");
        
        // Initialize agents database directly
        let db_path = app_dir.join("agents.db");
        let conn = rusqlite::Connection::open(db_path)
            .expect("Failed to open database");
        
        // Create tables manually (since init_database requires AppHandle)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                icon TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                default_task TEXT,
                model TEXT NOT NULL DEFAULT 'sonnet',
                enable_file_read BOOLEAN NOT NULL DEFAULT 1,
                enable_file_write BOOLEAN NOT NULL DEFAULT 1,
                enable_network BOOLEAN NOT NULL DEFAULT 0,
                hooks TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).expect("Failed to create agents table");
        
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
                status TEXT NOT NULL DEFAULT 'pending',
                pid INTEGER,
                process_started_at TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,
                FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
            )",
            [],
        ).expect("Failed to create agent_runs table");
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).expect("Failed to create app_settings table");
        
        let agent_db = AgentDb(Mutex::new(conn));
        
        // Initialize other states
        let checkpoint_state = CheckpointState::new();
        let process_registry = ProcessRegistryState::default();
        
        // Create and start the API server
        let api_server = api::ApiServer::new(
            agent_db,
            checkpoint_state,
            process_registry,
            Some(port),
        );
        
        if let Err(e) = api_server.start().await {
            eprintln!("Failed to start API server: {}", e);
            std::process::exit(1);
        }
    });
}
