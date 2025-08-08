use crate::state::{AppState, Profile, CancellationToken};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use std::time::Instant;
use anyhow::Result;

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

/// Run consensus and return the final result
#[tauri::command]
pub async fn run_consensus(
    query: String,
    state: State<'_, Arc<AppState>>,
) -> Result<ConsensusResult, String> {
    let start = Instant::now();
    
    // Check if API key is configured
    let has_api_key = {
        let keys = state.api_keys.read();
        keys.openrouter_key.is_some()
    };
    
    if !has_api_key {
        return Err("OpenRouter API key not configured. Please add it in Settings.".to_string());
    }
    
    // Get active profile
    let profile_id = state.active_profile.read()
        .clone()
        .unwrap_or_else(|| "default".to_string());
    
    // For now, simulate consensus with threading
    // In production, this would spawn the actual consensus engine
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    
    let duration_ms = start.elapsed().as_millis() as u64;
    let response = format!(
        "Processing query: '{}'\n\nUsing profile: {}\n\nThis is a simulated consensus response. The actual consensus engine will provide sophisticated 4-stage AI processing here.",
        query, profile_id
    );
    
    // Record conversation to database
    let cost = 0.0023;
    let tokens = 512;
    
    if let Err(e) = state.record_conversation(&query, &response, &profile_id, cost, tokens, duration_ms) {
        tracing::error!("Failed to record conversation: {}", e);
    }
    
    Ok(ConsensusResult {
        result: response,
        total_cost: cost,
        total_tokens: tokens,
        duration_ms,
    })
}

/// Run consensus with streaming updates
#[tauri::command]
pub async fn run_consensus_streaming(
    query: String,
    window: Window,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Check if API key is configured
    let has_api_key = {
        let keys = state.api_keys.read();
        keys.openrouter_key.is_some()
    };
    
    if !has_api_key {
        return Err("OpenRouter API key not configured. Please add it in Settings.".to_string());
    }
    
    // Create cancellation token
    let cancel_token = CancellationToken::new();
    {
        let mut token = state.cancellation_token.lock();
        *token = Some(cancel_token.clone());
    }
    
    // Get active profile
    let profile_id = state.active_profile.read()
        .clone()
        .unwrap_or_else(|| "default".to_string());
    
    let state_clone = state.inner().clone();
    
    // Spawn a task to handle streaming on a separate thread
    tokio::spawn(async move {
        let start = Instant::now();
        let stages = ["Generator", "Refiner", "Validator", "Curator"];
        let mut total_cost = 0.0;
        let mut total_tokens = 0u32;
        
        for stage in stages {
            // Check cancellation
            if cancel_token.is_cancelled() {
                let _ = window.emit("consensus-cancelled", "User cancelled operation");
                return;
            }
            
            // Emit start progress
            let progress = ConsensusProgress {
                stage: stage.to_string(),
                progress: 0,
                tokens: 0,
                cost: 0.0,
                message: format!("Starting {}", stage),
            };
            let _ = window.emit("consensus-progress", progress);
            
            // Simulate processing on separate thread to avoid blocking
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
            
            // Check cancellation again
            if cancel_token.is_cancelled() {
                let _ = window.emit("consensus-cancelled", "User cancelled operation");
                return;
            }
            
            // Calculate stage results
            let stage_tokens = 1000;
            let stage_cost = 0.01;
            total_tokens += stage_tokens;
            total_cost += stage_cost;
            
            // Emit completion progress
            let progress = ConsensusProgress {
                stage: stage.to_string(),
                progress: 100,
                tokens: stage_tokens as usize,
                cost: stage_cost,
                message: format!("{} completed", stage),
            };
            let _ = window.emit("consensus-progress", progress);
        }
        
        // Generate final response
        let duration_ms = start.elapsed().as_millis() as u64;
        let response = format!(
            "Consensus complete for: '{}'\n\nProfile: {}\n\nThis is a streaming consensus response with all 4 stages processed.",
            query, profile_id
        );
        
        // Record to database
        if let Err(e) = state_clone.record_conversation(&query, &response, &profile_id, total_cost, total_tokens, duration_ms) {
            tracing::error!("Failed to record conversation: {}", e);
        }
        
        // Emit final result
        let final_result = ConsensusResult {
            result: response,
            total_cost,
            total_tokens,
            duration_ms,
        };
        let _ = window.emit("consensus-complete", final_result);
    });
    
    Ok(())
}

/// Cancel ongoing consensus operation
#[tauri::command]
pub async fn cancel_consensus(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut token = state.cancellation_token.lock();
    if let Some(cancel_token) = token.as_ref() {
        cancel_token.cancel();
        *token = None;
        Ok(())
    } else {
        Err("No consensus operation to cancel".to_string())
    }
}

/// Get current consensus status
#[tauri::command]
pub async fn get_consensus_status(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let token = state.cancellation_token.lock();
    if token.is_some() {
        Ok("running".to_string())
    } else {
        Ok("idle".to_string())
    }
}

/// Get all consensus profiles
#[tauri::command]
pub async fn get_profiles(state: State<'_, Arc<AppState>>) -> Result<Vec<Profile>, String> {
    Ok(state.profiles.read().clone())
}

/// Get active profile ID
#[tauri::command]
pub async fn get_active_profile(state: State<'_, Arc<AppState>>) -> Result<Option<String>, String> {
    Ok(state.active_profile.read().clone())
}

/// Set active profile
#[tauri::command]
pub async fn set_active_profile(
    profile_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Verify profile exists
    let profiles = state.profiles.read();
    if !profiles.iter().any(|p| p.id == profile_id) {
        return Err(format!("Profile '{}' not found", profile_id));
    }
    drop(profiles);
    
    // Update active profile
    *state.active_profile.write() = Some(profile_id);
    Ok(())
}

/// Save or update a profile
#[tauri::command]
pub async fn save_profile(
    profile: Profile,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.save_profile(&profile)
        .map_err(|e| format!("Failed to save profile: {}", e))
}

/// Delete a profile
#[tauri::command]
pub async fn delete_profile(
    profile_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Don't allow deleting the default profile
    if profile_id == "default" {
        return Err("Cannot delete the default profile".to_string());
    }
    
    // Remove from database
    let db = state.database.lock();
    db.execute("DELETE FROM profiles WHERE id = ?1", [&profile_id])
        .map_err(|e| format!("Failed to delete profile: {}", e))?;
    drop(db);
    
    // Remove from memory
    let mut profiles = state.profiles.write();
    profiles.retain(|p| p.id != profile_id);
    
    // If this was the active profile, switch to default
    let mut active = state.active_profile.write();
    if active.as_ref() == Some(&profile_id) {
        *active = Some("default".to_string());
    }
    
    Ok(())
}