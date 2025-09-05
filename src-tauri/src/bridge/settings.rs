// Settings and API key management bridge

use serde::{Deserialize, Serialize};
use hive_ai::core::api_keys::ApiKeyManager;
use hive_ai::core::database::get_database;

// Simple result type for Tauri
type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub theme: String,
    pub auto_save: bool,
    pub auto_accept: bool,
    pub terminal_font_size: u32,
    pub editor_font_size: u32,
    pub show_welcome: bool,
    pub enable_telemetry: bool,
    pub max_cost_per_session: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStatus {
    pub openrouter: ApiKeyState,
    pub anthropic: ApiKeyState,
    pub hive: ApiKeyState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyState {
    pub configured: bool,
    pub valid: bool,
    pub provider: String,
    pub last_validated: Option<String>,
}

/// Get current settings configuration
#[tauri::command]
pub async fn get_settings() -> Result<SettingsConfig> {
    // Load from database or config file
    // For now, return defaults
    Ok(SettingsConfig {
        theme: "dark".to_string(),
        auto_save: true,
        auto_accept: false,
        terminal_font_size: 13,
        editor_font_size: 14,
        show_welcome: true,
        enable_telemetry: false,
        max_cost_per_session: Some(10.0),
    })
}

/// Save settings configuration
#[tauri::command]
pub async fn save_settings(config: SettingsConfig) -> Result<()> {
    // Save to database or config file
    tracing::info!("Saving settings: {:?}", config);
    
    // TODO: Persist to database
    
    Ok(())
}

/// Get API key configuration status
#[tauri::command]
pub async fn get_api_keys_status() -> Result<ApiKeyStatus> {
    // Check which API keys are configured
    let openrouter_configured = ApiKeyManager::get_openrouter_key().await.is_ok();
    let anthropic_configured = ApiKeyManager::get_anthropic_key().await.is_ok();
    
    Ok(ApiKeyStatus {
        openrouter: ApiKeyState {
            configured: openrouter_configured,
            valid: openrouter_configured, // TODO: Actually validate
            provider: "OpenRouter".to_string(),
            last_validated: None,
        },
        anthropic: ApiKeyState {
            configured: anthropic_configured,
            valid: anthropic_configured, // TODO: Actually validate
            provider: "Anthropic".to_string(),
            last_validated: None,
        },
        hive: ApiKeyState {
            configured: false, // Legacy field
            valid: false,
            provider: "Hive".to_string(),
            last_validated: None,
        },
    })
}

/// Save an API key
#[tauri::command]
pub async fn save_api_key_secure(provider: String, api_key: String) -> Result<()> {
    // Validate and save based on provider
    match provider.to_lowercase().as_str() {
        "openrouter" => {
            // Validate format
            ApiKeyManager::validate_openrouter_format(&api_key)
                .map_err(|e| format!("Invalid OpenRouter key: {}", e))?;
            
            // Test the key
            let is_valid = ApiKeyManager::test_openrouter_key(&api_key)
                .await
                .map_err(|e| format!("Failed to validate key: {}", e))?;
            
            if !is_valid {
                return Err("API key validation failed".to_string());
            }
            
            // Save to database
            ApiKeyManager::save_to_database(Some(&api_key), None, None)
                .await
                .map_err(|e| format!("Failed to save key: {}", e))?;
        }
        "anthropic" => {
            // Validate format
            ApiKeyManager::validate_anthropic_format(&api_key)
                .map_err(|e| format!("Invalid Anthropic key: {}", e))?;
            
            // Test the key
            let is_valid = ApiKeyManager::test_anthropic_key(&api_key)
                .await
                .map_err(|e| format!("Failed to validate key: {}", e))?;
            
            if !is_valid {
                return Err("API key validation failed".to_string());
            }
            
            // Save to database
            ApiKeyManager::save_to_database(None, None, Some(&api_key))
                .await
                .map_err(|e| format!("Failed to save key: {}", e))?;
        }
        _ => {
            return Err(format!("Unknown provider: {}", provider));
        }
    }
    
    Ok(())
}

/// Validate an API key without saving
#[tauri::command]
pub async fn validate_api_key(provider: String, api_key: String) -> Result<bool> {
    match provider.to_lowercase().as_str() {
        "openrouter" => {
            ApiKeyManager::validate_openrouter_format(&api_key)
                .map_err(|e| format!("Invalid format: {}", e))?;
            
            ApiKeyManager::test_openrouter_key(&api_key)
                .await
                .map_err(|e| format!("Validation failed: {}", e))
        }
        "anthropic" => {
            ApiKeyManager::validate_anthropic_format(&api_key)
                .map_err(|e| format!("Invalid format: {}", e))?;
            
            ApiKeyManager::test_anthropic_key(&api_key)
                .await
                .map_err(|e| format!("Validation failed: {}", e))
        }
        _ => Err(format!("Unknown provider: {}", provider))
    }
}

/// Remove an API key
#[tauri::command]
pub async fn remove_api_key(provider: String) -> Result<()> {
    // Remove from database
    match provider.to_lowercase().as_str() {
        "openrouter" => {
            // Clear OpenRouter key
            ApiKeyManager::save_to_database(Some(""), None, None)
                .await
                .map_err(|e| format!("Failed to remove key: {}", e))?;
        }
        "anthropic" => {
            // Clear Anthropic key
            ApiKeyManager::save_to_database(None, None, Some(""))
                .await
                .map_err(|e| format!("Failed to remove key: {}", e))?;
        }
        _ => {
            return Err(format!("Unknown provider: {}", provider));
        }
    }
    
    Ok(())
}

/// Get usage statistics for API keys
#[tauri::command]
pub async fn get_api_usage() -> Result<serde_json::Value> {
    // Return usage statistics from database
    // For now, return mock data
    Ok(serde_json::json!({
        "openrouter": {
            "requests_today": 42,
            "cost_today": 0.85,
            "requests_month": 1337,
            "cost_month": 25.50,
        },
        "anthropic": {
            "requests_today": 15,
            "cost_today": 0.30,
            "requests_month": 450,
            "cost_month": 12.75,
        }
    }))
}