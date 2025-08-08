use crate::state::{AppState, AnalyticsData};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_analytics_data(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AnalyticsData, String> {
    let app_state = state.lock().await;
    Ok(app_state.analytics.clone())
}

#[tauri::command]
pub async fn get_cost_breakdown(
    period: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement cost breakdown by period
    // For now, return mock data
    Ok(serde_json::json!({
        "period": period,
        "total": 0.0,
        "breakdown": []
    }))
}

#[tauri::command]
pub async fn export_analytics(
    format: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let app_state = state.lock().await;
    
    match format.as_str() {
        "json" => {
            serde_json::to_string_pretty(&app_state.analytics)
                .map_err(|e| format!("Failed to serialize analytics: {}", e))
        }
        "csv" => {
            // TODO: Implement CSV export
            Ok("CSV export not yet implemented".to_string())
        }
        _ => Err(format!("Unsupported export format: {}", format))
    }
}