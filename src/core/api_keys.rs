// API Key Management with Database Storage and Validation
// Matches TypeScript implementation with proper validation and security

use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::consensus::openrouter::OpenRouterClient;
use crate::core::database::get_database;

/// API key configuration stored in database
#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    pub openrouter_key: Option<String>,
    pub hive_key: Option<String>,
}

/// Validates and stores API keys in the database
pub struct ApiKeyManager;

impl ApiKeyManager {
    /// Validate OpenRouter API key format
    pub fn validate_format(api_key: &str) -> Result<()> {
        if !api_key.starts_with("sk-or-") {
            return Err(anyhow!("OpenRouter API key must start with 'sk-or-'"));
        }
        
        if api_key.len() <= 10 {
            return Err(anyhow!("OpenRouter API key is too short"));
        }
        
        Ok(())
    }
    
    /// Test OpenRouter API key with live authentication
    pub async fn test_openrouter_key(api_key: &str) -> Result<bool> {
        // First validate format
        Self::validate_format(api_key)?;
        
        debug!("Testing OpenRouter API key...");
        
        // Create HTTP client with proper headers
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        // Test against OpenRouter models endpoint
        let response = timeout(
            Duration::from_secs(30),
            client
                .get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://hivetechs.io")
                .header("X-Title", "hive-ai")
                .send()
        ).await
        .context("Timeout while testing API key")?
        .context("Failed to test API key")?;
        
        match response.status().as_u16() {
            200 => {
                info!("OpenRouter API key validated successfully");
                Ok(true)
            }
            401 | 403 => {
                warn!("OpenRouter API key authentication failed");
                Err(anyhow!("Invalid API key - authentication failed"))
            }
            402 => {
                warn!("OpenRouter API key has insufficient credits");
                Err(anyhow!("API key has insufficient credits"))
            }
            429 => {
                warn!("OpenRouter rate limit hit during validation");
                Err(anyhow!("Rate limit exceeded - please try again later"))
            }
            status => {
                warn!("Unexpected status code from OpenRouter: {}", status);
                Err(anyhow!("API validation failed with status: {}", status))
            }
        }
    }
    
    /// Save API keys to database configurations table
    pub async fn save_to_database(
        openrouter_key: Option<&str>,
        hive_key: Option<&str>,
    ) -> Result<()> {
        use crate::core::database::{initialize_database, get_database, DatabaseConfig};
        use uuid::Uuid;
        
        if let Some(key) = openrouter_key {
            // Validate before saving
            Self::validate_format(key)?;
            
            // Save to config.toml first for immediate availability
            crate::core::config::set_config_value("openrouter.api_key", key).await?;
            
            // Try to save to database as well
            if let Ok(db) = get_database().await {
                if let Ok(mut conn) = db.get_connection() {
                    // Ensure default user exists
                    let default_user_id = "default";
                    let _ = conn.execute(
                        "INSERT OR IGNORE INTO users (id, email, tier) VALUES (?1, ?2, ?3)",
                        params![default_user_id, "default@hive.ai", "FREE"],
                    );
                    
                    // Save API key to configurations table
                    let tx = conn.transaction()?;
                    tx.execute(
                        "INSERT INTO configurations (key, value, encrypted, user_id) 
                         VALUES (?1, ?2, ?3, ?4)
                         ON CONFLICT(key) DO UPDATE SET 
                         value = excluded.value,
                         updated_at = CURRENT_TIMESTAMP",
                        params!["openrouter_api_key", key, false, default_user_id],
                    )?;
                    
                    if let Some(hive_key) = hive_key {
                        if !hive_key.is_empty() {
                            tx.execute(
                                "INSERT INTO configurations (key, value, encrypted, user_id) 
                                 VALUES (?1, ?2, ?3, ?4)
                                 ON CONFLICT(key) DO UPDATE SET 
                                 value = excluded.value,
                                 updated_at = CURRENT_TIMESTAMP",
                                params!["hive_api_key", hive_key, false, default_user_id],
                            )?;
                        }
                    }
                    
                    tx.commit()?;
                    info!("Saved API keys to database configurations table");
                }
            }
        }
        
        Ok(())
    }
    
    /// Load API keys from database
    pub async fn load_from_database() -> Result<ApiKeyConfig> {
        // For now, load from config to avoid database initialization issues
        let config = crate::core::config::get_config().await?;
        
        let openrouter_key = config.openrouter
            .as_ref()
            .and_then(|or| or.api_key.clone());
            
        let hive_key = None; // TODO: Add hive key to config structure
        
        Ok(ApiKeyConfig {
            openrouter_key,
            hive_key,
        })
    }
    
    /// Check if valid API keys are configured
    pub async fn has_valid_keys() -> Result<bool> {
        let config = Self::load_from_database().await?;
        
        if let Some(key) = config.openrouter_key {
            // Validate format at minimum
            if Self::validate_format(&key).is_ok() {
                return Ok(true);
            }
        }
        
        // Check config.toml as fallback
        if let Ok(config) = crate::core::config::get_config().await {
            if let Some(openrouter) = config.openrouter {
                if let Some(key) = openrouter.api_key {
                    if Self::validate_format(&key).is_ok() {
                        // Migrate to database
                        let _ = Self::save_to_database(Some(&key), None).await;
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    /// Get OpenRouter API key from database or config
    pub async fn get_openrouter_key() -> Result<String> {
        // Try database first
        let config = Self::load_from_database().await?;
        if let Some(key) = config.openrouter_key {
            return Ok(key);
        }
        
        // Try config.toml as fallback
        if let Ok(config) = crate::core::config::get_config().await {
            if let Some(openrouter) = config.openrouter {
                if let Some(key) = openrouter.api_key {
                    // Migrate to database for next time
                    let _ = Self::save_to_database(Some(&key), None).await;
                    return Ok(key);
                }
            }
        }
        
        // Try environment variable as last resort
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            if Self::validate_format(&key).is_ok() {
                // Save to database for persistence
                let _ = Self::save_to_database(Some(&key), None).await;
                return Ok(key);
            }
        }
        
        Err(anyhow!(
            "No valid OpenRouter API key found. Please configure in Settings."
        ))
    }
    
    /// Clear API keys from database (for logout/reset)
    pub async fn clear_keys() -> Result<()> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        conn.execute(
            "DELETE FROM configurations WHERE key IN (?1, ?2)",
            params!["openrouter_api_key", "hive_api_key"],
        )?;
        
        info!("API keys cleared from database");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_format() {
        assert!(ApiKeyManager::validate_format("sk-or-abcd1234567890").is_ok());
        assert!(ApiKeyManager::validate_format("sk-or-123").is_err()); // Too short
        assert!(ApiKeyManager::validate_format("sk-invalid").is_err()); // Wrong prefix
        assert!(ApiKeyManager::validate_format("").is_err()); // Empty
    }
}