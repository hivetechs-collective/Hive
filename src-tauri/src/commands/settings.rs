use crate::state::{AppState, Settings, ApiKeys};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStatus {
    pub openrouter_configured: bool,
    pub hive_configured: bool,
    pub anthropic_configured: bool,
}

/// Get all settings
#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    Ok(state.settings.read().clone())
}

/// Update settings
#[tauri::command]
pub async fn update_settings(
    settings: Settings,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Save each setting to database
    state.save_setting("auto_accept", &settings.auto_accept.to_string())
        .map_err(|e| format!("Failed to save auto_accept: {}", e))?;
    state.save_setting("theme", &settings.theme)
        .map_err(|e| format!("Failed to save theme: {}", e))?;
    state.save_setting("font_size", &settings.font_size.to_string())
        .map_err(|e| format!("Failed to save font_size: {}", e))?;
    state.save_setting("show_line_numbers", &settings.show_line_numbers.to_string())
        .map_err(|e| format!("Failed to save show_line_numbers: {}", e))?;
    state.save_setting("word_wrap", &settings.word_wrap.to_string())
        .map_err(|e| format!("Failed to save word_wrap: {}", e))?;
    state.save_setting("max_tokens", &settings.max_tokens.to_string())
        .map_err(|e| format!("Failed to save max_tokens: {}", e))?;
    state.save_setting("temperature", &settings.temperature.to_string())
        .map_err(|e| format!("Failed to save temperature: {}", e))?;
    
    // Update in-memory settings
    *state.settings.write() = settings;
    
    Ok(())
}

/// Save settings to database
#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    update_settings(settings, state).await
}

/// Get API key status (whether keys are configured, not the actual keys)
#[tauri::command]
pub async fn get_api_key_status(state: State<'_, Arc<AppState>>) -> Result<ApiKeyStatus, String> {
    let keys = state.api_keys.read();
    Ok(ApiKeyStatus {
        openrouter_configured: keys.openrouter_key.is_some() && !keys.openrouter_key.as_ref().unwrap().is_empty(),
        hive_configured: keys.hive_key.is_some() && !keys.hive_key.as_ref().unwrap().is_empty(),
        anthropic_configured: keys.anthropic_key.is_some() && !keys.anthropic_key.as_ref().unwrap().is_empty(),
    })
}

/// Get all API keys (masked for security)
#[tauri::command]
pub async fn get_api_keys(state: State<'_, Arc<AppState>>) -> Result<ApiKeys, String> {
    let keys = state.api_keys.read();
    
    // Return masked versions of the keys
    Ok(ApiKeys {
        openrouter_key: keys.openrouter_key.as_ref().map(|k| {
            if k.len() > 8 {
                format!("{}...{}", &k[..4], &k[k.len()-4..])
            } else {
                "****".to_string()
            }
        }),
        hive_key: keys.hive_key.as_ref().map(|k| {
            if k.len() > 8 {
                format!("{}...{}", &k[..4], &k[k.len()-4..])
            } else {
                "****".to_string()
            }
        }),
        anthropic_key: keys.anthropic_key.as_ref().map(|k| {
            if k.len() > 8 {
                format!("{}...{}", &k[..4], &k[k.len()-4..])
            } else {
                "****".to_string()
            }
        }),
    })
}

/// Set an API key
#[tauri::command]
pub async fn set_api_key(
    key_type: String,
    key_value: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Validate key type
    match key_type.as_str() {
        "openrouter" | "hive" | "anthropic" => {},
        _ => return Err(format!("Invalid key type: {}", key_type)),
    }
    
    // Save to database and update in-memory state
    state.save_api_key(&key_type, &key_value)
        .map_err(|e| format!("Failed to save API key: {}", e))?;
    
    Ok(())
}

/// Save an API key (alias for set_api_key)
#[tauri::command]
pub async fn save_api_key(
    key_type: String,
    key_value: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    set_api_key(key_type, key_value, state).await
}

/// Clear an API key
#[tauri::command]
pub async fn clear_api_key(
    key_type: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // Save empty string to clear the key
    state.save_api_key(&key_type, "")
        .map_err(|e| format!("Failed to clear API key: {}", e))?;
    
    Ok(())
}

/// Validate an API key (test if it works)
#[tauri::command]
pub async fn validate_api_key(
    key_type: String,
    key_value: String,
) -> Result<bool, String> {
    // TODO: Implement actual validation by making a test API call
    // For now, just check if the key looks valid
    match key_type.as_str() {
        "openrouter" => {
            // OpenRouter keys typically start with "sk-or-"
            Ok(key_value.starts_with("sk-or-") && key_value.len() > 20)
        },
        "hive" => {
            // Hive keys typically start with "hive-"
            Ok(key_value.starts_with("hive-") && key_value.len() > 15)
        },
        "anthropic" => {
            // Anthropic keys typically start with "sk-ant-"
            Ok(key_value.starts_with("sk-ant-") && key_value.len() > 20)
        },
        _ => Err(format!("Unknown key type: {}", key_type))
    }
}