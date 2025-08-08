use crate::state::{AppSettings, AppState};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AppSettings, String> {
    let app_state = state.lock().await;
    Ok(app_state.settings.clone())
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.lock().await;
    app_state.settings = settings;
    
    // TODO: Save settings to persistent storage
    Ok(())
}

#[tauri::command]
pub async fn get_api_key_status() -> Result<bool, String> {
    // For now, return true to indicate API keys are configured
    Ok(true)
}

#[tauri::command]
pub async fn set_api_key(
    _key: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    // TODO: Save the API key and reinitialize consensus
    Ok(())
}