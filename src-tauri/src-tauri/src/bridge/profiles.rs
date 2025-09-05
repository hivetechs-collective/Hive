// Profile management bridge - connects frontend to existing profile system

use serde::{Deserialize, Serialize};
use hive_ai::core::profiles::{
    ConsensusProfile, ProfileCategory, ExpertLevel, 
    BudgetProfile, PerformanceProfile, ExpertTemplate
};
use hive_ai::core::database::get_database;

// Simple result type for Tauri
type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub generator_model: String,
    pub generator_temperature: f64,
    pub refiner_model: String,
    pub refiner_temperature: f64,
    pub validator_model: String,
    pub validator_temperature: f64,
    pub curator_model: String,
    pub curator_temperature: f64,
}

/// Get available profile presets
#[tauri::command]
pub async fn get_available_profiles() -> Result<Vec<ProfileInfo>> {
    // Return the standard 4 profiles that match the Dioxus version
    Ok(vec![
        ProfileInfo {
            id: "speed".to_string(),
            name: "Speed".to_string(),
            description: "Optimized for fastest response time".to_string(),
            category: "Speed".to_string(),
            is_active: false,
        },
        ProfileInfo {
            id: "balanced".to_string(),
            name: "Balanced".to_string(),
            description: "Balance between speed and quality".to_string(),
            category: "Balanced".to_string(),
            is_active: true, // Default
        },
        ProfileInfo {
            id: "quality".to_string(),
            name: "Quality".to_string(),
            description: "Maximum quality and accuracy".to_string(),
            category: "Quality".to_string(),
            is_active: false,
        },
        ProfileInfo {
            id: "consensus".to_string(),
            name: "Consensus".to_string(),
            description: "Full 4-stage consensus pipeline".to_string(),
            category: "Production".to_string(),
            is_active: false,
        },
    ])
}

/// Set the active profile for consensus
#[tauri::command]
pub async fn set_active_profile(profile_id: String) -> Result<()> {
    // Store the active profile in the database or memory
    // For now, we'll just log it
    tracing::info!("Setting active profile to: {}", profile_id);
    
    // In a real implementation, this would:
    // 1. Load the profile configuration
    // 2. Update the consensus engine to use this profile
    // 3. Persist the selection
    
    Ok(())
}

/// Get configuration for a specific profile
#[tauri::command]
pub async fn get_profile_config(profile_id: String) -> Result<ProfileConfig> {
    // Map profile IDs to model configurations
    let config = match profile_id.as_str() {
        "speed" => ProfileConfig {
            generator_model: "claude-3-haiku-20240307".to_string(),
            generator_temperature: 0.3,
            refiner_model: "claude-3-haiku-20240307".to_string(),
            refiner_temperature: 0.2,
            validator_model: "claude-3-haiku-20240307".to_string(),
            validator_temperature: 0.1,
            curator_model: "claude-3-haiku-20240307".to_string(),
            curator_temperature: 0.0,
        },
        "balanced" => ProfileConfig {
            generator_model: "claude-3-sonnet-20240229".to_string(),
            generator_temperature: 0.5,
            refiner_model: "claude-3-haiku-20240307".to_string(),
            refiner_temperature: 0.3,
            validator_model: "claude-3-haiku-20240307".to_string(),
            validator_temperature: 0.2,
            curator_model: "claude-3-sonnet-20240229".to_string(),
            curator_temperature: 0.1,
        },
        "quality" => ProfileConfig {
            generator_model: "claude-3-opus-20240229".to_string(),
            generator_temperature: 0.7,
            refiner_model: "claude-3-sonnet-20240229".to_string(),
            refiner_temperature: 0.5,
            validator_model: "claude-3-sonnet-20240229".to_string(),
            validator_temperature: 0.3,
            curator_model: "claude-3-opus-20240229".to_string(),
            curator_temperature: 0.2,
        },
        "consensus" => ProfileConfig {
            generator_model: "claude-3-opus-20240229".to_string(),
            generator_temperature: 0.8,
            refiner_model: "claude-3-opus-20240229".to_string(),
            refiner_temperature: 0.6,
            validator_model: "claude-3-opus-20240229".to_string(),
            validator_temperature: 0.4,
            curator_model: "claude-3-opus-20240229".to_string(),
            curator_temperature: 0.2,
        },
        _ => {
            return Err(format!("Unknown profile: {}", profile_id));
        }
    };
    
    Ok(config)
}

/// Get all saved custom profiles from database
#[tauri::command]
pub async fn get_custom_profiles() -> Result<Vec<ProfileInfo>> {
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    // Query the database for user-created profiles
    // For now, return empty list
    Ok(vec![])
}

/// Create a new custom profile
#[tauri::command]
pub async fn create_custom_profile(
    name: String,
    config: ProfileConfig,
) -> Result<ProfileInfo> {
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    // Save the custom profile to database
    // For now, just return a mock response
    Ok(ProfileInfo {
        id: format!("custom_{}", uuid::Uuid::new_v4()),
        name,
        description: "Custom profile".to_string(),
        category: "Custom".to_string(),
        is_active: false,
    })
}

/// Delete a custom profile
#[tauri::command]
pub async fn delete_custom_profile(profile_id: String) -> Result<()> {
    if !profile_id.starts_with("custom_") {
        return Err("Cannot delete built-in profiles".to_string());
    }
    
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    // Delete from database
    // For now, just return success
    Ok(())
}