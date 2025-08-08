// Bridge module - Thin wrappers connecting Tauri to existing Hive functionality
// NO business logic here - just calls to existing code

// Include sub-modules from bridge directory
#[path = "bridge/profiles.rs"]
pub mod profiles;
#[path = "bridge/settings.rs"]
pub mod settings;
#[path = "bridge/git.rs"]
pub mod git;

use tauri::{State, Window, Emitter};
use serde::{Deserialize, Serialize};

// Re-export from main hive crate
use hive_ai::core::{
    database::{get_database, initialize_database, DatabaseConfig},
    api_keys::ApiKeyManager,
    profiles::ConsensusProfile,
    analytics::AnalyticsEngine,
};
use hive_ai::consensus::{
    engine::ConsensusEngine,
    streaming::StreamingCallbacks,
    types::{ConsensusConfig, ConsensusResult as HiveConsensusResult},
};
// Terminal PTY is in desktop module
use hive_ai::desktop::terminal_pty::PtyProcess;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::spawn;
use once_cell::sync::Lazy;

// Simple result type for Tauri
type Result<T> = std::result::Result<T, String>;

// Terminal management
pub static TERMINALS: Lazy<Arc<Mutex<HashMap<String, TerminalInstance>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct TerminalInstance {
    pub pty: Arc<PtyProcess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub id: String,
    pub title: String,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStatusInfo {
    pub openrouter_configured: bool,
    pub anthropic_configured: bool,
    pub hive_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProgress {
    pub stage: String,
    pub progress: u8,
    pub tokens: usize,
    pub cost: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub result: String,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub duration_ms: u64,
}

/// Initialize the database on app start
pub async fn init_database() -> Result<()> {
    initialize_database(Some(DatabaseConfig::default()))
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))
}

/// Run consensus using the existing engine
#[tauri::command]
pub async fn run_consensus(query: String) -> Result<ConsensusResult> {
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    let engine = ConsensusEngine::new(Some(db))
        .await
        .map_err(|e| format!("Failed to create consensus engine: {}", e))?;
    
    let result = engine.process(&query, None)
        .await
        .map_err(|e| format!("Consensus failed: {}", e))?;
    
    Ok(ConsensusResult {
        result: result.result.unwrap_or_else(|| result.error.unwrap_or_default()),
        total_cost: result.total_cost,
        total_tokens: result.stages.iter()
            .filter_map(|s| s.usage.as_ref())
            .map(|u| u.total_tokens)
            .sum(),
        duration_ms: (result.total_duration * 1000.0) as u64,
    })
}

/// Run consensus with streaming using existing callbacks
#[tauri::command]
pub async fn run_consensus_streaming(query: String, window: Window) -> Result<()> {
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    let engine = ConsensusEngine::new(Some(db))
        .await
        .map_err(|e| format!("Failed to create consensus engine: {}", e))?;
    
    // Create Tauri-specific callbacks that emit to window
    let callbacks = Arc::new(TauriStreamingCallbacks::new(window));
    
    engine.process_with_callbacks(&query, None, callbacks, None)
        .await
        .map_err(|e| format!("Streaming consensus failed: {}", e))?;
    
    Ok(())
}

/// Get profiles from database
#[tauri::command]
pub async fn get_profiles() -> Result<Vec<String>> {
    // For now, return a list of available profile names
    // The actual profile system is in ConsensusProfile struct
    Ok(vec![
        "speed".to_string(),
        "balanced".to_string(),
        "quality".to_string(),
        "consensus".to_string(),
    ])
}

/// Get analytics data from existing engine
#[tauri::command]
pub async fn get_analytics_data() -> Result<serde_json::Value> {
    // Analytics engine needs a config, not a database
    let config = hive_ai::core::analytics::AnalyticsConfig::default();
    let engine = AnalyticsEngine::new(config)
        .await
        .map_err(|e| format!("Failed to create analytics engine: {}", e))?;
    
    // For now return basic stats until we implement the full analytics
    // In the future we can call engine methods to get real data
    Ok(serde_json::json!({
        "total_queries": 0,
        "total_cost": 0.0,
        "average_response_time": 0,
    }))
}

/// Get API key status from existing manager
#[tauri::command]
pub async fn get_api_key_status() -> Result<ApiKeyStatusInfo> {
    // Check for OpenRouter key
    let openrouter_configured = ApiKeyManager::get_openrouter_key().await.is_ok();
    
    // Check for Anthropic key
    let anthropic_configured = ApiKeyManager::get_anthropic_key().await.is_ok();
    
    Ok(ApiKeyStatusInfo {
        openrouter_configured,
        anthropic_configured,
        hive_configured: false, // Legacy field
    })
}

/// Save API key using existing manager
#[tauri::command]
pub async fn save_api_key(key_type: String, key_value: String) -> Result<()> {
    // Validate and save based on key type
    match key_type.as_str() {
        "openrouter" => {
            ApiKeyManager::validate_openrouter_format(&key_value)
                .map_err(|e| format!("Invalid OpenRouter key: {}", e))?;
            ApiKeyManager::save_to_database(Some(&key_value), None, None)
                .await
                .map_err(|e| format!("Failed to save OpenRouter key: {}", e))?;
        }
        "anthropic" => {
            ApiKeyManager::validate_anthropic_format(&key_value)
                .map_err(|e| format!("Invalid Anthropic key: {}", e))?;
            ApiKeyManager::save_to_database(None, None, Some(&key_value))
                .await
                .map_err(|e| format!("Failed to save Anthropic key: {}", e))?;
        }
        _ => return Err(format!("Unknown key type: {}", key_type))
    }
    
    Ok(())
}

// Tauri-specific streaming callbacks that emit events to the window
struct TauriStreamingCallbacks {
    window: Window,
}

impl TauriStreamingCallbacks {
    fn new(window: Window) -> Self {
        Self { window }
    }
}

impl StreamingCallbacks for TauriStreamingCallbacks {
    fn on_stage_start(&self, stage: hive_ai::consensus::types::Stage, model: &str) -> anyhow::Result<()> {
        let progress = ConsensusProgress {
            stage: format!("{:?}", stage),
            progress: 0,
            tokens: 0,
            cost: 0.0,
            message: format!("Starting {:?} with {}", stage, model),
        };
        let _ = self.window.emit("consensus-progress", progress);
        Ok(())
    }
    
    fn on_stage_chunk(&self, stage: hive_ai::consensus::types::Stage, chunk: &str, _total_content: &str) -> anyhow::Result<()> {
        let _ = self.window.emit("consensus-token", serde_json::json!({
            "stage": format!("{:?}", stage),
            "chunk": chunk
        }));
        Ok(())
    }
    
    fn on_stage_progress(&self, stage: hive_ai::consensus::types::Stage, progress: hive_ai::consensus::streaming::ProgressInfo) -> anyhow::Result<()> {
        let consensus_progress = ConsensusProgress {
            stage: format!("{:?}", stage),
            progress: progress.percentage as u8,
            tokens: progress.tokens as usize,
            cost: 0.0,
            message: format!("{:?} progress: {:.0}%", stage, progress.percentage),
        };
        let _ = self.window.emit("consensus-progress", consensus_progress);
        Ok(())
    }
    
    fn on_stage_complete(&self, stage: hive_ai::consensus::types::Stage, result: &hive_ai::consensus::types::StageResult) -> anyhow::Result<()> {
        let tokens = result.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
        let cost = result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0);
        
        let progress = ConsensusProgress {
            stage: format!("{:?}", stage),
            progress: 100,
            tokens: tokens as usize,
            cost,
            message: format!("{:?} complete", stage),
        };
        let _ = self.window.emit("consensus-progress", progress);
        Ok(())
    }
    
    fn on_error(&self, stage: hive_ai::consensus::types::Stage, error: &anyhow::Error) -> anyhow::Result<()> {
        let _ = self.window.emit("consensus-error", serde_json::json!({
            "stage": format!("{:?}", stage),
            "error": error.to_string()
        }));
        Ok(())
    }
}

// Terminal Commands - Bridge to existing PTY implementation

/// Create a new terminal using existing PTY implementation
#[tauri::command]
pub async fn create_terminal(
    title: String,
    rows: u16,
    cols: u16,
    window: Window,
) -> Result<TerminalInfo> {
    let id = uuid::Uuid::new_v4().to_string();
    
    // Use home directory as default
    let working_dir = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());
    
    // Get shell command
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    
    // Create PTY process using our existing implementation
    let (pty, mut rx) = PtyProcess::spawn(&shell, &[], &working_dir)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;
    
    let pty = Arc::new(pty);
    let terminal_id = id.clone();
    let window_clone = window.clone();
    
    // Spawn output handler that emits to frontend
    let pty_clone = pty.clone();
    spawn(async move {
        while let Some(output) = rx.recv().await {
            let _ = window_clone.emit("terminal-output", serde_json::json!({
                "id": terminal_id,
                "data": output
            }));
            
            // Check if process is still running
            if !pty_clone.is_running().await {
                let _ = window_clone.emit("terminal-closed", terminal_id.clone());
                break;
            }
        }
    });
    
    // Store terminal instance
    let mut terminals = TERMINALS.lock().await;
    terminals.insert(id.clone(), TerminalInstance {
        pty,
    });
    
    Ok(TerminalInfo {
        id,
        title,
        rows,
        cols,
    })
}

/// Write data to terminal
#[tauri::command]
pub async fn write_to_terminal(
    terminal_id: String,
    data: String,
) -> Result<()> {
    let terminals = TERMINALS.lock().await;
    
    if let Some(terminal) = terminals.get(&terminal_id) {
        terminal.pty.write(&data).await
            .map_err(|e| format!("Failed to write to terminal: {}", e))
    } else {
        Err(format!("Terminal {} not found", terminal_id))
    }
}

/// Resize terminal (placeholder - PTY resize not yet implemented in our PtyProcess)
#[tauri::command]
pub async fn resize_terminal(
    _terminal_id: String,
    _rows: u16,
    _cols: u16,
) -> Result<()> {
    // TODO: Add resize support to PtyProcess if needed
    Ok(())
}

/// Close terminal
#[tauri::command]
pub async fn close_terminal(
    terminal_id: String,
) -> Result<()> {
    let mut terminals = TERMINALS.lock().await;
    
    if let Some(terminal) = terminals.remove(&terminal_id) {
        terminal.pty.kill().await
            .map_err(|e| format!("Failed to close terminal: {}", e))?;
    }
    
    Ok(())
}