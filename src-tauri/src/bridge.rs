// Bridge module - Thin wrappers connecting Tauri to existing Hive functionality
// NO business logic here - just calls to existing code

use tauri::{State, Window, Emitter};
use serde::{Deserialize, Serialize};

// Re-export from main hive crate
use hive_ai::core::{
    database::{get_database, initialize_database, DatabaseConfig},
    api_keys::ApiKeyManager,
    profiles::ProfileManager,
};
use hive_ai::consensus::{
    engine::ConsensusEngine,
    streaming::StreamingCallbacks,
    types::{ConsensusConfig, ConsensusResult as HiveConsensusResult},
};
use hive_ai::analytics::AnalyticsEngine;

// Simple result type for Tauri
type Result<T> = std::result::Result<T, String>;

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
    let engine = ConsensusEngine::from_database()
        .await
        .map_err(|e| format!("Failed to get consensus engine: {}", e))?;
    
    let result = engine.run(query.clone())
        .await
        .map_err(|e| format!("Consensus failed: {}", e))?;
    
    Ok(ConsensusResult {
        result: result.final_output,
        total_cost: result.total_cost,
        total_tokens: result.total_tokens,
        duration_ms: result.duration_ms,
    })
}

/// Run consensus with streaming using existing callbacks
#[tauri::command]
pub async fn run_consensus_streaming(query: String, window: Window) -> Result<()> {
    let engine = ConsensusEngine::from_database()
        .await
        .map_err(|e| format!("Failed to get consensus engine: {}", e))?;
    
    // Create Tauri-specific callbacks that emit to window
    let callbacks = TauriStreamingCallbacks::new(window);
    
    engine.run_with_callbacks(query, Box::new(callbacks))
        .await
        .map_err(|e| format!("Streaming consensus failed: {}", e))?;
    
    Ok(())
}

/// Get profiles from existing profile manager
#[tauri::command]
pub async fn get_profiles() -> Result<Vec<hive_ai::core::profiles::Profile>> {
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    ProfileManager::new(db)
        .get_all_profiles()
        .await
        .map_err(|e| format!("Failed to get profiles: {}", e))
}

/// Get analytics data from existing engine
#[tauri::command]
pub async fn get_analytics_data() -> Result<hive_ai::analytics::AnalyticsData> {
    let engine = AnalyticsEngine::from_database()
        .await
        .map_err(|e| format!("Failed to get analytics engine: {}", e))?;
    
    engine.get_comprehensive_stats()
        .await
        .map_err(|e| format!("Failed to get analytics: {}", e))
}

/// Get API key status from existing manager
#[tauri::command]
pub async fn get_api_key_status() -> Result<hive_ai::core::api_keys::ApiKeyStatus> {
    ApiKeyManager::get_status()
        .await
        .map_err(|e| format!("Failed to get API key status: {}", e))
}

/// Save API key using existing manager
#[tauri::command]
pub async fn save_api_key(key_type: String, key_value: String) -> Result<()> {
    ApiKeyManager::save_key(&key_type, &key_value)
        .await
        .map_err(|e| format!("Failed to save API key: {}", e))
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
    fn on_stage_start(&self, stage: &str, model: &str) {
        let progress = ConsensusProgress {
            stage: stage.to_string(),
            progress: 0,
            tokens: 0,
            cost: 0.0,
            message: format!("Starting {} with {}", stage, model),
        };
        let _ = self.window.emit("consensus-progress", progress);
    }
    
    fn on_stage_progress(&self, stage: &str, progress: u8, tokens: usize, cost: f64) {
        let progress = ConsensusProgress {
            stage: stage.to_string(),
            progress,
            tokens,
            cost,
            message: format!("{} in progress", stage),
        };
        let _ = self.window.emit("consensus-progress", progress);
    }
    
    fn on_stage_complete(&self, stage: &str, tokens: usize, cost: f64) {
        let progress = ConsensusProgress {
            stage: stage.to_string(),
            progress: 100,
            tokens,
            cost,
            message: format!("{} complete", stage),
        };
        let _ = self.window.emit("consensus-progress", progress);
    }
    
    fn on_streaming_token(&self, token: &str) {
        let _ = self.window.emit("consensus-token", token);
    }
    
    fn on_error(&self, error: &str) {
        let _ = self.window.emit("consensus-error", error);
    }
}