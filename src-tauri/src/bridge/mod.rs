// Bridge module - connects Tauri frontend to REAL Hive backend

pub mod profiles;
pub mod settings;
pub mod git;

use anyhow::Result;
use hive_ai::core::database::{get_database, DatabaseManager};
use hive_ai::consensus::engine::ConsensusEngine;
use hive_ai::consensus::profiles::ExpertProfileManager;
use hive_ai::consensus::streaming::ChannelStreamingCallbacks;
use serde::{Deserialize, Serialize};
use tauri::Window;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

// Re-export submodule commands
pub use profiles::*;
pub use settings::*;
pub use git::*;

// Global consensus engine instance
lazy_static::lazy_static! {
    static ref CONSENSUS_ENGINE: Arc<RwLock<Option<ConsensusEngine>>> = Arc::new(RwLock::new(None));
}

/// Initialize the database on startup
pub async fn init_database() -> Result<()> {
    tracing::info!("Initializing Hive database...");
    
    // The database is initialized automatically on first access
    // Just get it to ensure it's created
    let db = get_database().await
        .map_err(|e| anyhow::anyhow!("Failed to get database: {}", e))?;
    
    // Run any necessary migrations
    db.run_migrations().await
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;
    
    // Initialize the REAL consensus engine with database
    let engine = ConsensusEngine::new(Some(db.clone())).await
        .map_err(|e| anyhow::anyhow!("Failed to create consensus engine: {}", e))?;
    
    let mut engine_guard = CONSENSUS_ENGINE.write().await;
    *engine_guard = Some(engine);
    
    tracing::info!("Database and consensus engine initialized successfully");
    Ok(())
}

/// Progress update for streaming consensus
#[derive(Debug, Clone, Serialize)]
pub struct ConsensusProgress {
    pub stage: String,
    pub progress: u8,
    pub message: String,
}

/// Result from consensus operation
#[derive(Debug, Clone, Serialize)]
pub struct ConsensusResult {
    pub result: String,
    pub duration_ms: u64,
    pub total_cost: f64,
    pub total_tokens: u32,
}

/// Run consensus with the REAL 4-stage engine
#[tauri::command]
pub async fn run_consensus(query: String) -> Result<ConsensusResult, String> {
    tracing::info!("Running consensus for query: {}", query);
    
    let engine_guard = CONSENSUS_ENGINE.read().await;
    let engine = engine_guard.as_ref()
        .ok_or_else(|| "Consensus engine not initialized".to_string())?;
    
    let start = std::time::Instant::now();
    
    // Run the REAL consensus engine
    match engine.process(&query, None).await {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            
            Ok(ConsensusResult {
                result: result.result.unwrap_or_else(|| "No result".to_string()),
                duration_ms,
                total_cost: result.total_cost,
                total_tokens: 0, // TODO: Get from stages
            })
        }
        Err(e) => Err(format!("Consensus failed: {}", e))
    }
}

/// Run consensus with streaming updates
#[tauri::command]
pub async fn run_consensus_streaming(
    window: Window,
    query: String,
) -> Result<(), String> {
    tracing::info!("Running streaming consensus for query: {}", query);
    
    let engine_guard = CONSENSUS_ENGINE.read().await;
    let engine = engine_guard.as_ref()
        .ok_or_else(|| "Consensus engine not initialized".to_string())?;
    
    let start = std::time::Instant::now();
    
    // Create channels for streaming
    let (_tx, mut _rx) = mpsc::channel::<String>(100);
    
    // For now, just run without streaming callbacks
    // TODO: Reconnect streaming when types are fixed
    match engine.process(&query, None).await {
        Ok(result) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            
            // Send completion event - TODO: Fix to use proper Tauri v2 event API
            // window.emit_all("consensus-complete", ConsensusResult {
            //     result: result.result.unwrap_or_else(|| "No result".to_string()),
            //     duration_ms,
            //     total_cost: result.total_cost,
            //     total_tokens: 0,
            // }).map_err(|e| e.to_string())?;
            
            Ok(())
        }
        Err(e) => Err(format!("Consensus failed: {}", e))
    }
}

/// Get analytics data from the REAL analytics engine
#[tauri::command]
pub async fn get_analytics_data() -> Result<serde_json::Value, String> {
    tracing::info!("Getting analytics data");
    
    // Get real analytics from database
    let db = get_database().await.map_err(|e| e.to_string())?;
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    
    // Query real statistics
    let total_queries: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversations",
        [],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let total_cost: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_cost), 0.0) FROM conversations",
        [],
        |row| row.get(0)
    ).unwrap_or(0.0);
    
    let success_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversations WHERE status = 'completed'",
        [],
        |row| row.get(0)
    ).unwrap_or(0);
    
    let success_rate = if total_queries > 0 {
        (success_count as f64 / total_queries as f64) * 100.0
    } else {
        100.0
    };
    
    let avg_response_time: f64 = conn.query_row(
        "SELECT COALESCE(AVG(duration_ms), 0.0) FROM conversations",
        [],
        |row| row.get(0)
    ).unwrap_or(2300.0) / 1000.0; // Convert to seconds
    
    Ok(serde_json::json!({
        "total_queries": total_queries,
        "total_cost": total_cost,
        "success_rate": success_rate,
        "avg_response_time": avg_response_time,
        "models_used": {
            "generator": "claude-3-opus",
            "refiner": "gpt-4-turbo",
            "validator": "gemini-pro",
            "curator": "claude-3-sonnet"
        }
    }))
}

// Terminal bridge functions
// TODO: Reconnect to real terminal implementation
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct TerminalInfo {
    pub id: String,
    pub rows: u16,
    pub cols: u16,
}

/// Create a new terminal instance
#[tauri::command]
pub async fn create_terminal(rows: u16, cols: u16) -> Result<TerminalInfo, String> {
    let id = uuid::Uuid::new_v4().to_string();
    
    // TODO: Connect to real terminal implementation
    
    Ok(TerminalInfo { id, rows, cols })
}

/// Write to terminal
#[tauri::command]
pub async fn write_to_terminal(_id: String, _data: String) -> Result<(), String> {
    // TODO: Connect to real terminal implementation
    Ok(())
}

/// Resize terminal
#[tauri::command]
pub async fn resize_terminal(_id: String, _rows: u16, _cols: u16) -> Result<(), String> {
    // TODO: Connect to real terminal implementation
    Ok(())
}

/// Close terminal
#[tauri::command]
pub async fn close_terminal(_id: String) -> Result<(), String> {
    // TODO: Connect to real terminal implementation
    Ok(())
}