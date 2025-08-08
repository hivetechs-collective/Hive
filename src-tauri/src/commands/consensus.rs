use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub result: String,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub duration_ms: u64,
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
pub struct ConsensusProfile {
    pub id: String,
    pub name: String,
    pub generator_model: String,
    pub refiner_model: String,
    pub validator_model: String,
    pub curator_model: String,
}

/// Run consensus and return the final result
#[tauri::command]
pub async fn run_consensus(
    query: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ConsensusResult, String> {
    let start = std::time::Instant::now();
    
    // For now, simulate consensus with a simple response
    // This will be replaced with actual consensus engine later
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    
    let duration_ms = start.elapsed().as_millis() as u64;
    
    // Update analytics
    {
        let mut app_state = state.lock().await;
        app_state.analytics.total_queries += 1;
        app_state.analytics.today_query_count += 1;
    }
    
    Ok(ConsensusResult {
        result: format!("Processing query: '{}'\n\nThis is a simulated consensus response. The actual consensus engine will provide sophisticated 4-stage AI processing here.", query),
        total_cost: 0.0023,
        total_tokens: 512,
        duration_ms,
    })
}

/// Run consensus with streaming updates
#[tauri::command]
pub async fn run_consensus_streaming(
    query: String,
    window: Window,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    // Spawn a task to handle streaming events
    tokio::spawn(async move {
        // Emit stage progress events
        for stage in ["Generator", "Refiner", "Validator", "Curator"] {
            let progress = ConsensusProgress {
                stage: stage.to_string(),
                progress: 0,
                tokens: 0,
                cost: 0.0,
                message: format!("Starting {}", stage),
            };
            let _ = window.emit("consensus-progress", progress);
            
            // Simulate progress
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            let progress = ConsensusProgress {
                stage: stage.to_string(),
                progress: 100,
                tokens: 1000,
                cost: 0.01,
                message: format!("{} completed", stage),
            };
            let _ = window.emit("consensus-progress", progress);
        }
        
        // Emit final result
        let final_result = ConsensusResult {
            result: format!("Streaming consensus result for: '{}'", query),
            total_cost: 0.04,
            total_tokens: 4000,
            duration_ms: 2000,
        };
        let _ = window.emit("consensus-complete", final_result);
    });
    
    Ok(())
}

/// Cancel the currently running consensus
#[tauri::command]
pub async fn cancel_consensus(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    // TODO: Implement cancellation in ConsensusEngine
    Ok(())
}

/// Get the current consensus status
#[tauri::command]
pub async fn get_consensus_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<bool, String> {
    let app_state = state.lock().await;
    
    // Return whether consensus engine is available
    Ok(app_state.consensus_engine.is_some())
}

/// Get the active consensus profile
#[tauri::command]
pub async fn get_active_profile(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ConsensusProfile, String> {
    // Return a default profile for now
    Ok(ConsensusProfile {
        id: "precision".to_string(),
        name: "Precision Architect".to_string(),
        generator_model: "claude-3-opus".to_string(),
        refiner_model: "gpt-4-turbo".to_string(),
        validator_model: "claude-3-sonnet".to_string(),
        curator_model: "gemini-pro".to_string(),
    })
}

/// Set the active consensus profile
#[tauri::command]
pub async fn set_active_profile(
    _profile_name: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    // For now, just return success
    Ok(())
}

/// Get all available consensus profiles
#[tauri::command]
pub async fn get_profiles(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<ConsensusProfile>, String> {
    // Return sample profiles
    Ok(vec![
        ConsensusProfile {
            id: "lightning".to_string(),
            name: "Lightning Fast".to_string(),
            generator_model: "gpt-3.5-turbo".to_string(),
            refiner_model: "claude-instant".to_string(),
            validator_model: "gemini-flash".to_string(),
            curator_model: "gpt-3.5-turbo".to_string(),
        },
        ConsensusProfile {
            id: "precision".to_string(),
            name: "Precision Architect".to_string(),
            generator_model: "claude-3-opus".to_string(),
            refiner_model: "gpt-4-turbo".to_string(),
            validator_model: "claude-3-sonnet".to_string(),
            curator_model: "gemini-pro".to_string(),
        },
        ConsensusProfile {
            id: "budget".to_string(),
            name: "Budget Optimizer".to_string(),
            generator_model: "mistral-7b".to_string(),
            refiner_model: "llama-2-70b".to_string(),
            validator_model: "mixtral-8x7b".to_string(),
            curator_model: "mistral-7b".to_string(),
        },
    ])
}