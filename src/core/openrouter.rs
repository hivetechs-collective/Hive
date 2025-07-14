//! OpenRouter integration for model discovery and synchronization
//!
//! This module provides comprehensive OpenRouter API integration including:
//! - Model discovery and synchronization
//! - Provider metadata management
//! - Real-time model updates and rankings
//! - Cost and performance tracking

use crate::core::{HiveError, Result, DatabaseManager};
use anyhow::Context;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// OpenRouter model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub pricing: ModelPricing,
    pub context_window: Option<u32>,
    pub architecture: Option<ModelArchitecture>,
    pub top_provider: Option<TopProvider>,
    pub per_request_limits: Option<PerRequestLimits>,
}

/// Model pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt: String,
    pub completion: String,
    pub image: Option<String>,
    pub request: Option<String>,
}

/// Model architecture details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArchitecture {
    pub modality: String,
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}

/// Top provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopProvider {
    pub max_completion_tokens: Option<u32>,
    pub is_moderated: Option<bool>,
}

/// Per-request limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerRequestLimits {
    pub prompt_tokens: Option<String>,
    pub completion_tokens: Option<String>,
}

/// OpenRouter API response for models list
#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

/// Stored model information in database
#[derive(Debug, Clone)]
pub struct StoredModel {
    pub internal_id: i64,
    pub openrouter_id: String,
    pub name: String,
    pub provider_name: String,
    pub description: Option<String>,
    pub context_window: Option<u32>,
    pub pricing_input: f64,
    pub pricing_output: f64,
    pub is_active: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Model ranking information
#[derive(Debug, Clone)]
pub struct ModelRanking {
    pub model_internal_id: i64,
    pub ranking_source: String,
    pub rank_position: u32,
    pub score: Option<f64>,
    pub category: Option<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// OpenRouter client for API interactions
pub struct OpenRouterClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterClient {
    /// Create new OpenRouter client
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("hive-rust/2.0.0")
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }

    /// Test API key connectivity
    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/auth/key", self.base_url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/hivetechs/hive")
            .header("X-Title", "Hive AI")
            .send()
            .await
            .context("Failed to connect to OpenRouter API")?;

        Ok(response.status().is_success())
    }

    /// Fetch all available models from OpenRouter
    pub async fn fetch_models(&self) -> Result<Vec<OpenRouterModel>> {
        let url = format!("{}/models", self.base_url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/hivetechs/hive")
            .header("X-Title", "Hive AI")
            .send()
            .await
            .context("Failed to fetch models from OpenRouter")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(HiveError::Internal { context: "openrouter".to_string(), message: format!("OpenRouter API error: {} - {}", status, error_text) });
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .context("Failed to parse OpenRouter models response")?;

        Ok(models_response.data)
    }

    /// Fetch model rankings (if available)
    pub async fn fetch_rankings(&self) -> Result<HashMap<String, u32>> {
        // OpenRouter doesn't have a public rankings endpoint, but we can infer
        // popularity from pricing trends and usage patterns
        // For now, return empty rankings - we'll implement this when the API is available
        Ok(HashMap::new())
    }
}

/// Model synchronization manager
pub struct ModelSyncManager {
    db: DatabaseManager,
    client: OpenRouterClient,
}

impl ModelSyncManager {
    /// Create new model sync manager
    pub fn new(db: DatabaseManager, api_key: String) -> Self {
        let client = OpenRouterClient::new(api_key);

        Self {
            db,
            client,
        }
    }

    /// Sync all models from OpenRouter
    pub async fn sync_models(&self) -> Result<ModelSyncResult> {
        println!("🔄 Syncing models from OpenRouter...");

        // Test connection first
        if !self.client.test_connection().await? {
            return Err(HiveError::AuthenticationFailed { service: "OpenRouter".to_string(), message: "Invalid OpenRouter API key".to_string() });
        }

        // Fetch models from OpenRouter
        let openrouter_models = self.client.fetch_models().await?;
        println!("📥 Fetched {} models from OpenRouter", openrouter_models.len());

        // Process and store models
        let mut sync_result = ModelSyncResult {
            total_fetched: openrouter_models.len(),
            new_models: 0,
            updated_models: 0,
            deactivated_models: 0,
            providers: std::collections::HashSet::new(),
        };

        let conn = self.db.get_connection()?;

        // Create a transaction for bulk operations
        let tx = conn.unchecked_transaction()?;

        // Get existing models for comparison
        let existing_models = self.get_existing_models(&tx)?;
        let mut current_model_ids = std::collections::HashSet::new();

        // Process each model
        for model in openrouter_models {
            current_model_ids.insert(model.id.clone());

            let provider_name = self.extract_provider_name(&model.id);
            sync_result.providers.insert(provider_name.clone());

            let pricing_input = self.parse_pricing(&model.pricing.prompt)?;
            let pricing_output = self.parse_pricing(&model.pricing.completion)?;

            match existing_models.get(&model.id) {
                Some(existing) => {
                    // Update existing model
                    let needs_update =
                        existing.name != model.name ||
                        existing.description != model.description ||
                        existing.context_window != model.context_window ||
                        (existing.pricing_input - pricing_input).abs() > f64::EPSILON ||
                        (existing.pricing_output - pricing_output).abs() > f64::EPSILON;

                    if needs_update {
                        self.update_model(&tx, &model, existing.internal_id, &provider_name, pricing_input, pricing_output)?;
                        sync_result.updated_models += 1;
                    }
                }
                None => {
                    // Insert new model
                    self.insert_model(&tx, &model, &provider_name, pricing_input, pricing_output)?;
                    sync_result.new_models += 1;
                }
            }
        }

        // Deactivate models that are no longer available
        for (model_id, stored_model) in &existing_models {
            if !current_model_ids.contains(model_id) && stored_model.is_active {
                self.deactivate_model(&tx, stored_model.internal_id)?;
                sync_result.deactivated_models += 1;
            }
        }

        // Commit transaction
        tx.commit()?;

        // Update rankings if available
        if let Ok(rankings) = self.client.fetch_rankings().await {
            self.update_rankings(&rankings).await?;
        }

        println!("✅ Model sync completed:");
        println!("   📊 Total models: {}", sync_result.total_fetched);
        println!("   🆕 New models: {}", sync_result.new_models);
        println!("   🔄 Updated models: {}", sync_result.updated_models);
        println!("   ❌ Deactivated models: {}", sync_result.deactivated_models);
        println!("   🏢 Providers: {}", sync_result.providers.len());

        Ok(sync_result)
    }

    /// Get existing models from database
    fn get_existing_models(&self, conn: &rusqlite::Connection) -> Result<HashMap<String, StoredModel>> {
        let mut stmt = conn.prepare(
            "SELECT internal_id, openrouter_id, name, provider_name, description,
                    context_window, pricing_input, pricing_output, is_active, last_updated
             FROM openrouter_models"
        )?;

        let model_iter = stmt.query_map([], |row| {
            let last_updated_str: String = row.get(9)?;
            let last_updated = chrono::DateTime::parse_from_rfc3339(&last_updated_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(9, "last_updated".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&chrono::Utc);

            Ok(StoredModel {
                internal_id: row.get(0)?,
                openrouter_id: row.get(1)?,
                name: row.get(2)?,
                provider_name: row.get(3)?,
                description: row.get(4)?,
                context_window: row.get(5)?,
                pricing_input: row.get(6)?,
                pricing_output: row.get(7)?,
                is_active: row.get(8)?,
                last_updated,
            })
        })?;

        let mut models = HashMap::new();
        for model in model_iter {
            let model = model?;
            models.insert(model.openrouter_id.clone(), model);
        }

        Ok(models)
    }

    /// Insert new model into database
    fn insert_model(
        &self,
        conn: &rusqlite::Connection,
        model: &OpenRouterModel,
        provider_name: &str,
        pricing_input: f64,
        pricing_output: f64,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO openrouter_models
             (openrouter_id, name, provider_name, description, context_window,
              pricing_input, pricing_output, is_active, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
            rusqlite::params![
                model.id,
                model.name,
                provider_name,
                model.description,
                model.context_window,
                pricing_input,
                pricing_output,
                now
            ],
        )?;

        Ok(())
    }

    /// Update existing model in database
    fn update_model(
        &self,
        conn: &rusqlite::Connection,
        model: &OpenRouterModel,
        internal_id: i64,
        provider_name: &str,
        pricing_input: f64,
        pricing_output: f64,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE openrouter_models
             SET name = ?1, provider_name = ?2, description = ?3, context_window = ?4,
                 pricing_input = ?5, pricing_output = ?6, is_active = 1, last_updated = ?7
             WHERE internal_id = ?8",
            rusqlite::params![
                model.name,
                provider_name,
                model.description,
                model.context_window,
                pricing_input,
                pricing_output,
                now,
                internal_id
            ],
        )?;

        Ok(())
    }

    /// Deactivate model in database
    fn deactivate_model(&self, conn: &rusqlite::Connection, internal_id: i64) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE openrouter_models SET is_active = 0, last_updated = ?1 WHERE internal_id = ?2",
            rusqlite::params![now, internal_id],
        )?;

        Ok(())
    }

    /// Update model rankings
    async fn update_rankings(&self, rankings: &HashMap<String, u32>) -> Result<()> {
        if rankings.is_empty() {
            return Ok(());
        }

        let conn = self.db.get_connection()?;
        let tx = conn.unchecked_transaction()?;

        // Clear existing rankings
        tx.execute("DELETE FROM model_rankings WHERE ranking_source = 'openrouter_general'", [])?;

        // Insert new rankings
        for (model_id, rank) in rankings {
            // Get internal ID for this model
            let internal_id: Option<i64> = match tx.query_row(
                "SELECT internal_id FROM openrouter_models WHERE openrouter_id = ? AND is_active = 1",
                [model_id],
                |row| row.get(0),
            ) {
                Ok(id) => Some(id),
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => return Err(HiveError::from(e)),
            };

            if let Some(internal_id) = internal_id {
                let now = chrono::Utc::now().to_rfc3339();
                tx.execute(
                    "INSERT INTO model_rankings
                     (model_internal_id, ranking_source, rank_position, last_updated)
                     VALUES (?1, 'openrouter_general', ?2, ?3)",
                    rusqlite::params![internal_id, rank, now],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Extract provider name from model ID
    fn extract_provider_name(&self, model_id: &str) -> String {
        if let Some(slash_pos) = model_id.find('/') {
            model_id[..slash_pos].to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Parse pricing string to float
    fn parse_pricing(&self, pricing_str: &str) -> Result<f64> {
        // OpenRouter pricing is typically in format like "0.000001" (per token)
        pricing_str.parse::<f64>()
            .map_err(|_| HiveError::Internal {
                context: "pricing".to_string(),
                message: format!("Invalid pricing format: {}", pricing_str)
            })
    }
}

/// Result of model synchronization
#[derive(Debug)]
pub struct ModelSyncResult {
    pub total_fetched: usize,
    pub new_models: usize,
    pub updated_models: usize,
    pub deactivated_models: usize,
    pub providers: std::collections::HashSet<String>,
}

/// Model query and filtering utilities
pub struct ModelQuery {
    db: DatabaseManager,
}

impl ModelQuery {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    /// Get all active models
    pub async fn get_all_models(&self) -> Result<Vec<StoredModel>> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT internal_id, openrouter_id, name, provider_name, description,
                    context_window, pricing_input, pricing_output, is_active, last_updated
             FROM openrouter_models
             WHERE is_active = 1
             ORDER BY provider_name, name"
        )?;

        let model_iter = stmt.query_map([], |row| {
            let last_updated_str: String = row.get(9)?;
            let last_updated = chrono::DateTime::parse_from_rfc3339(&last_updated_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(9, "last_updated".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&chrono::Utc);

            Ok(StoredModel {
                internal_id: row.get(0)?,
                openrouter_id: row.get(1)?,
                name: row.get(2)?,
                provider_name: row.get(3)?,
                description: row.get(4)?,
                context_window: row.get(5)?,
                pricing_input: row.get(6)?,
                pricing_output: row.get(7)?,
                is_active: row.get(8)?,
                last_updated,
            })
        })?;

        let mut models = Vec::new();
        for model in model_iter {
            models.push(model?);
        }

        Ok(models)
    }

    /// Get models by provider
    pub async fn get_models_by_provider(&self, provider: &str) -> Result<Vec<StoredModel>> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT internal_id, openrouter_id, name, provider_name, description,
                    context_window, pricing_input, pricing_output, is_active, last_updated
             FROM openrouter_models
             WHERE is_active = 1 AND provider_name = ?1
             ORDER BY name"
        )?;

        let model_iter = stmt.query_map([provider], |row| {
            let last_updated_str: String = row.get(9)?;
            let last_updated = chrono::DateTime::parse_from_rfc3339(&last_updated_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(9, "last_updated".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&chrono::Utc);

            Ok(StoredModel {
                internal_id: row.get(0)?,
                openrouter_id: row.get(1)?,
                name: row.get(2)?,
                provider_name: row.get(3)?,
                description: row.get(4)?,
                context_window: row.get(5)?,
                pricing_input: row.get(6)?,
                pricing_output: row.get(7)?,
                is_active: row.get(8)?,
                last_updated,
            })
        })?;

        let mut models = Vec::new();
        for model in model_iter {
            models.push(model?);
        }

        Ok(models)
    }

    /// Get all providers
    pub async fn get_providers(&self) -> Result<Vec<ProviderInfo>> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT provider_name,
                    COUNT(*) as model_count,
                    COUNT(CASE WHEN pricing_input = 0 THEN 1 END) as free_models,
                    MIN(pricing_input + pricing_output) as min_cost,
                    AVG(pricing_input + pricing_output) as avg_cost
             FROM openrouter_models
             WHERE is_active = 1
             GROUP BY provider_name
             ORDER BY provider_name"
        )?;

        let provider_iter = stmt.query_map([], |row| {
            Ok(ProviderInfo {
                name: row.get(0)?,
                model_count: row.get(1)?,
                free_models: row.get(2)?,
                min_cost: row.get(3)?,
                avg_cost: row.get(4)?,
            })
        })?;

        let mut providers = Vec::new();
        for provider in provider_iter {
            providers.push(provider?);
        }

        Ok(providers)
    }

    /// Get model by internal ID
    pub async fn get_model_by_internal_id(&self, internal_id: i64) -> Result<Option<StoredModel>> {
        let conn = self.db.get_connection()?;
        let result = match conn.query_row(
            "SELECT internal_id, openrouter_id, name, provider_name, description,
                    context_window, pricing_input, pricing_output, is_active, last_updated
             FROM openrouter_models
             WHERE internal_id = ?1",
            [internal_id],
            |row| {
                let last_updated_str: String = row.get(9)?;
                let last_updated = chrono::DateTime::parse_from_rfc3339(&last_updated_str)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(9, "last_updated".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&chrono::Utc);

                Ok(StoredModel {
                    internal_id: row.get(0)?,
                    openrouter_id: row.get(1)?,
                    name: row.get(2)?,
                    provider_name: row.get(3)?,
                    description: row.get(4)?,
                    context_window: row.get(5)?,
                    pricing_input: row.get(6)?,
                    pricing_output: row.get(7)?,
                    is_active: row.get(8)?,
                    last_updated,
                })
            }
        ) {
            Ok(model) => Some(model),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(HiveError::from(e)),
        };

        Ok(result)
    }
}

/// Provider information summary
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub model_count: u32,
    pub free_models: u32,
    pub min_cost: f64,
    pub avg_cost: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_provider_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let db = DatabaseManager::new(temp_dir.path().join("test.db")).unwrap();
        let sync_manager = ModelSyncManager::new(db, "test-key".to_string());

        assert_eq!(sync_manager.extract_provider_name("openai/gpt-4"), "openai");
        assert_eq!(sync_manager.extract_provider_name("anthropic/claude-3"), "anthropic");
        assert_eq!(sync_manager.extract_provider_name("invalid"), "unknown");
    }

    #[test]
    fn test_pricing_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let db = DatabaseManager::new(temp_dir.path().join("test.db")).unwrap();
        let sync_manager = ModelSyncManager::new(db, "test-key".to_string());

        assert_eq!(sync_manager.parse_pricing("0.000001").unwrap(), 0.000001);
        assert_eq!(sync_manager.parse_pricing("0.01").unwrap(), 0.01);
        assert!(sync_manager.parse_pricing("invalid").is_err());
    }
}