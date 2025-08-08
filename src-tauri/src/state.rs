use serde::{Deserialize, Serialize};

/// Application state for Tauri
pub struct AppState {
    /// Consensus engine (placeholder for now)
    pub consensus_engine: Option<String>,
    
    /// Analytics data
    pub analytics: AnalyticsData,
    
    /// Application settings
    pub settings: AppSettings,
    
    /// Current project information
    pub current_project: Option<ProjectInfo>,
}

impl AppState {
    pub fn new() -> Self {
        // For now, create a simple state without complex initialization
        Self {
            consensus_engine: Some("Ready".to_string()),
            analytics: AnalyticsData::default(),
            settings: AppSettings::default(),
            current_project: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsData {
    pub total_queries: u64,
    pub total_cost: f64,
    pub success_rate: f64,
    pub avg_response_time: f64,
    pub queries_trend: f64,
    pub cost_trend: f64,
    pub today_total_cost: f64,
    pub today_query_count: u64,
    pub week_total_cost: f64,
    pub week_query_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub auto_save: bool,
    pub auto_accept_operations: bool,
    pub show_cost_warnings: bool,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            auto_save: true,
            auto_accept_operations: false,
            show_cost_warnings: true,
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

impl AppSettings {
    pub async fn load() -> anyhow::Result<Self> {
        // Try to load from database or config file
        // For now, just return defaults
        Ok(Self::default())
    }
    
    pub async fn save(&self) -> anyhow::Result<()> {
        // Save to database or config file
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub file_count: usize,
    pub total_lines: usize,
}