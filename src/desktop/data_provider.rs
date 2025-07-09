//! Data provider for desktop application
//! Fetches real system state for UI components

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::core::{
    database::{DatabaseManager, get_database},
    config::get_config,
};
use crate::providers::openrouter::cost::{CostTracker, BudgetConfig};
use crate::consensus::engine::ConsensusEngine;

/// System data provider that fetches real state
pub struct SystemDataProvider {
    database: Arc<RwLock<Option<Arc<DatabaseManager>>>>,
    cost_tracker: Arc<RwLock<Option<CostTracker>>>,
    consensus_engine: Arc<RwLock<Option<ConsensusEngine>>>,
}

impl SystemDataProvider {
    /// Create a new system data provider
    pub fn new() -> Self {
        Self {
            database: Arc::new(RwLock::new(None)),
            cost_tracker: Arc::new(RwLock::new(None)),
            consensus_engine: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the data provider
    pub async fn initialize(&self) -> Result<()> {
        // Initialize database connection
        match get_database().await {
            Ok(db) => {
                let mut database = self.database.write().await;
                *database = Some(db);
            }
            Err(e) => {
                tracing::warn!("Failed to open database: {}", e);
            }
        }

        // Initialize cost tracker with default budget config
        let budget_config = BudgetConfig {
            daily_limit: None,  // TODO: Add budget configuration to config file
            monthly_limit: None,
            per_request_limit: None,
            alert_threshold: 0.8,
            enforce_limits: false,
        };
        
        let mut cost_tracker = self.cost_tracker.write().await;
        *cost_tracker = Some(CostTracker::new(budget_config));

        // Note: ConsensusEngine will be initialized when needed
        
        Ok(())
    }

    /// Get conversation count from database
    pub async fn get_conversation_count(&self) -> usize {
        let database = self.database.read().await;
        
        if let Some(db) = database.as_ref() {
            match db.get_statistics().await {
                Ok(stats) => stats.conversation_count as usize,
                Err(e) => {
                    tracing::debug!("Failed to get conversation count: {}", e);
                    0
                }
            }
        } else {
            0
        }
    }

    /// Get total cost spent
    pub async fn get_total_cost(&self) -> f64 {
        let cost_tracker = self.cost_tracker.read().await;
        
        if let Some(tracker) = cost_tracker.as_ref() {
            match tracker.get_budget_status().await {
                Ok(status) => status.monthly_spent as f64,
                Err(e) => {
                    tracing::debug!("Failed to get cost status: {}", e);
                    // Try to get from database as fallback
                    self.get_cost_from_database().await.unwrap_or(0.0)
                }
            }
        } else {
            // Try to get from database as fallback
            self.get_cost_from_database().await.unwrap_or(0.0)
        }
    }

    /// Get context usage percentage
    pub async fn get_context_usage(&self) -> u8 {
        // Get from active consensus engine if available
        let consensus = self.consensus_engine.read().await;
        
        if let Some(engine) = consensus.as_ref() {
            // If consensus engine is active, get its context usage
            engine.get_context_usage_percentage() as u8
        } else {
            // Otherwise, estimate based on current conversation
            self.estimate_context_usage().await
        }
    }
    
    /// Estimate context usage based on conversation history
    async fn estimate_context_usage(&self) -> u8 {
        let database = self.database.read().await;
        
        if let Some(db) = database.as_ref() {
            // Get approximate token count from current conversation
            match db.get_connection() {
                Ok(conn) => {
                    // Get the most recent conversation's messages
                    let result: Result<(i64, Option<String>), rusqlite::Error> = conn.query_row(
                        "SELECT 
                            COALESCE(message_count, 0),
                            messages
                         FROM conversations 
                         WHERE datetime(updated_at) > datetime('now', '-1 hour')
                         ORDER BY updated_at DESC
                         LIMIT 1",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?))
                    );
                    
                    match result {
                        Ok((message_count, messages_json)) => {
                            // More accurate estimation based on actual content
                            let estimated_tokens = if let Some(json) = messages_json {
                                // Rough estimation: 1 token per 4 characters
                                json.len() / 4
                            } else {
                                // Fallback: assume average 150 tokens per message
                                (message_count * 150) as usize
                            };
                            
                            // Get actual model context window from config
                            let context_window = self.get_model_context_window().await.unwrap_or(8192);
                            
                            let usage = ((estimated_tokens as f64 / context_window as f64) * 100.0) as u8;
                            usage.min(100)
                        }
                        Err(_) => 15, // Default baseline usage
                    }
                }
                Err(_) => 0,
            }
        } else {
            0
        }
    }

    /// Check API connectivity
    pub async fn check_api_connectivity(&self) -> ConnectionStatus {
        // Check if we can reach OpenRouter API with proper authentication
        let config = match get_config().await {
            Ok(config) => config,
            Err(_) => return ConnectionStatus::Disconnected,
        };
        
        let api_key = match config.openrouter.as_ref().and_then(|or| or.api_key.as_ref()) {
            Some(key) => key,
            None => return ConnectionStatus::Disconnected,
        };
        
        let client = reqwest::Client::new();
        let url = "https://openrouter.ai/api/v1/models";
        
        match client.get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/hivetechs/hive")
            .header("X-Title", "Hive AI")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => ConnectionStatus::Connected,
            Ok(response) => {
                tracing::debug!("API connectivity check failed with status: {}", response.status());
                ConnectionStatus::Disconnected
            },
            Err(e) => {
                tracing::debug!("API connectivity check failed with error: {}", e);
                ConnectionStatus::Disconnected
            },
        }
    }

    /// Get current active model
    pub async fn get_current_model(&self) -> Option<String> {
        let config = match get_config().await {
            Ok(config) => config,
            Err(_) => return None,
        };

        // Return the generator model from consensus config
        Some(config.consensus.models.generator)
    }

    /// Get database size in MB
    pub async fn get_database_size_mb(&self) -> f64 {
        let database = self.database.read().await;
        
        if let Some(db) = database.as_ref() {
            match db.get_statistics().await {
                Ok(stats) => stats.database_size_bytes as f64 / 1024.0 / 1024.0,
                Err(_) => 0.0,
            }
        } else {
            0.0
        }
    }

    /// Get daily cost spent
    pub async fn get_daily_cost(&self) -> f64 {
        let cost_tracker = self.cost_tracker.read().await;
        
        if let Some(tracker) = cost_tracker.as_ref() {
            match tracker.get_budget_status().await {
                Ok(status) => status.daily_spent as f64,
                Err(_) => 0.0,
            }
        } else {
            0.0
        }
    }
    
    /// Set the active consensus engine reference
    pub async fn set_consensus_engine(&self, engine: Option<ConsensusEngine>) {
        let mut consensus = self.consensus_engine.write().await;
        *consensus = engine;
    }
    
    /// Get cost from database as fallback
    async fn get_cost_from_database(&self) -> Result<f64> {
        let database = self.database.read().await;
        
        if let Some(db) = database.as_ref() {
            match db.get_connection() {
                Ok(conn) => {
                    // Get total cost from consensus_costs table
                    let total_cost: f64 = conn.query_row(
                        "SELECT COALESCE(SUM(total_cost), 0.0) FROM consensus_costs 
                         WHERE datetime(created_at) > datetime('now', '-30 days')",
                        [],
                        |row| row.get(0)
                    ).unwrap_or(0.0);
                    
                    Ok(total_cost)
                }
                Err(_) => Ok(0.0),
            }
        } else {
            Ok(0.0)
        }
    }
    
    /// Get the context window size for the current model
    async fn get_model_context_window(&self) -> Result<usize> {
        let config = get_config().await?;
        
        // Map common models to their context windows
        let model = config.consensus.models.generator.as_str();
        let context_window = match model {
            model if model.contains("gpt-4") => 128000,
            model if model.contains("gpt-3.5") => 16385,
            model if model.contains("claude-3") => 200000,
            model if model.contains("claude-2") => 100000,
            model if model.contains("mistral") => 32768,
            model if model.contains("mixtral") => 32768,
            model if model.contains("llama") => 8192,
            _ => 8192, // Default fallback
        };
        
        Ok(context_window)
    }
}

/// Connection status enum matching the desktop state
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
}

/// Background task to periodically update system state with channel
pub async fn start_system_state_updater_with_channel(
    data_provider: Arc<SystemDataProvider>,
    tx: tokio::sync::mpsc::Sender<SystemState>,
) {
    loop {
        // Fetch current system state
        let system_state = SystemState {
            conversation_count: data_provider.get_conversation_count().await,
            total_cost: data_provider.get_total_cost().await,
            context_usage: data_provider.get_context_usage().await,
            connection_status: data_provider.check_api_connectivity().await,
            current_model: data_provider.get_current_model().await,
            database_size_mb: data_provider.get_database_size_mb().await,
            daily_cost: data_provider.get_daily_cost().await,
        };

        // Send the state update
        if tx.send(system_state).await.is_err() {
            // Receiver dropped, exit the loop
            break;
        }

        // Wait before next update
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

/// System state snapshot
#[derive(Clone, Debug)]
pub struct SystemState {
    pub conversation_count: usize,
    pub total_cost: f64,
    pub context_usage: u8,
    pub connection_status: ConnectionStatus,
    pub current_model: Option<String>,
    pub database_size_mb: f64,
    pub daily_cost: f64,
}