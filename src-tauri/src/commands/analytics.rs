use crate::state::{AppState, AnalyticsData};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    pub id: String,
    pub query: String,
    pub response: String,
    pub profile_id: String,
    pub cost: f64,
    pub tokens: u32,
    pub duration_ms: u64,
    pub created_at: String,
}

#[tauri::command]
pub async fn get_analytics_data(
    state: State<'_, Arc<AppState>>,
) -> Result<AnalyticsData, String> {
    Ok(state.analytics_data.read().clone())
}

#[tauri::command]
pub async fn get_cost_breakdown(
    period: String,
    state: State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let analytics = state.analytics_data.read();
    
    let breakdown = match period.as_str() {
        "today" => {
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let today_queries = analytics.queries_by_day.get(&today).unwrap_or(&0);
            serde_json::json!({
                "period": "today",
                "total_cost": analytics.total_cost / analytics.total_queries.max(1) as f64 * *today_queries as f64,
                "query_count": today_queries,
                "breakdown": analytics.cost_by_model.clone()
            })
        },
        "week" => {
            // Calculate week stats
            let mut week_queries = 0u32;
            let mut week_cost = 0.0f64;
            
            for i in 0..7 {
                let date = chrono::Utc::now()
                    .checked_sub_signed(chrono::Duration::days(i))
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string();
                if let Some(count) = analytics.queries_by_day.get(&date) {
                    week_queries += count;
                }
            }
            
            if analytics.total_queries > 0 {
                week_cost = analytics.total_cost / analytics.total_queries as f64 * week_queries as f64;
            }
            
            serde_json::json!({
                "period": "week",
                "total_cost": week_cost,
                "query_count": week_queries,
                "breakdown": analytics.cost_by_model.clone()
            })
        },
        "month" => {
            serde_json::json!({
                "period": "month",
                "total_cost": analytics.total_cost,
                "query_count": analytics.total_queries,
                "breakdown": analytics.cost_by_model.clone()
            })
        },
        _ => {
            return Err(format!("Invalid period: {}", period));
        }
    };
    
    Ok(breakdown)
}

#[tauri::command]
pub async fn export_analytics(
    format: String,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let analytics = state.analytics_data.read();
    
    match format.as_str() {
        "json" => {
            serde_json::to_string_pretty(&*analytics)
                .map_err(|e| format!("Failed to serialize analytics: {}", e))
        }
        "csv" => {
            // Generate CSV format
            let mut csv = String::from("Date,Queries,Cost\n");
            for (date, count) in &analytics.queries_by_day {
                let day_cost = if analytics.total_queries > 0 {
                    analytics.total_cost / analytics.total_queries as f64 * *count as f64
                } else {
                    0.0
                };
                csv.push_str(&format!("{},{},{:.4}\n", date, count, day_cost));
            }
            Ok(csv)
        }
        _ => Err(format!("Unsupported export format: {}", format))
    }
}

#[tauri::command]
pub async fn get_conversation_history(
    limit: Option<u32>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ConversationHistory>, String> {
    let db = state.database.lock();
    let limit = limit.unwrap_or(50);
    
    let mut stmt = db.prepare(
        "SELECT id, query, response, profile_id, cost, tokens, duration_ms, created_at 
         FROM conversations 
         ORDER BY created_at DESC 
         LIMIT ?1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let conversations = stmt.query_map([limit], |row| {
        Ok(ConversationHistory {
            id: row.get(0)?,
            query: row.get(1)?,
            response: row.get(2)?,
            profile_id: row.get(3)?,
            cost: row.get(4)?,
            tokens: row.get::<_, i64>(5)? as u32,
            duration_ms: row.get::<_, i64>(6)? as u64,
            created_at: row.get(7)?,
        })
    }).map_err(|e| format!("Failed to query conversations: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect conversations: {}", e))?;
    
    Ok(conversations)
}

#[tauri::command]
pub async fn clear_conversation_history(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let db = state.database.lock();
    db.execute("DELETE FROM conversations", [])
        .map_err(|e| format!("Failed to clear history: {}", e))?;
    
    // Reset analytics
    let mut analytics = state.analytics_data.write();
    analytics.total_queries = 0;
    analytics.total_cost = 0.0;
    analytics.total_tokens = 0;
    analytics.queries_by_day.clear();
    analytics.cost_by_model.clear();
    
    Ok(())
}

#[tauri::command]
pub async fn get_model_usage_stats(
    state: State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let analytics = state.analytics_data.read();
    
    // Calculate model usage from cost breakdown
    let total_cost: f64 = analytics.cost_by_model.values().sum();
    let mut model_percentages = serde_json::Map::new();
    
    for (model, cost) in &analytics.cost_by_model {
        let percentage = if total_cost > 0.0 {
            (cost / total_cost * 100.0) as u32
        } else {
            0
        };
        model_percentages.insert(model.clone(), serde_json::json!(percentage));
    }
    
    Ok(serde_json::json!({
        "total_models_used": analytics.cost_by_model.len(),
        "model_percentages": model_percentages,
        "most_used_model": analytics.cost_by_model
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(model, _)| model.clone())
            .unwrap_or_else(|| "None".to_string())
    }))
}