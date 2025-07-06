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
        let db = get_database().await?;
        let mut conn = db.get_connection()?;
        
        // Use transaction for atomicity
        let tx = conn.transaction()?;
        
        if let Some(key) = openrouter_key {
            // Validate before saving
            Self::validate_format(key)?;
            
            // Store in configurations table
            tx.execute(
                "INSERT INTO configurations (key, value, encrypted, user_id) 
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(key) DO UPDATE SET 
                 value = excluded.value,
                 updated_at = CURRENT_TIMESTAMP",
                params!["openrouter_api_key", key, false, "default"],
            )?;
            
            debug!("Saved OpenRouter API key to database");
        }
        
        if let Some(key) = hive_key {
            if !key.is_empty() {
                tx.execute(
                    "INSERT INTO configurations (key, value, encrypted, user_id) 
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(key) DO UPDATE SET 
                     value = excluded.value,
                     updated_at = CURRENT_TIMESTAMP",
                    params!["hive_api_key", key, false, "default"],
                )?;
                
                debug!("Saved Hive API key to database");
            }
        }
        
        tx.commit()?;
        info!("API keys saved to database successfully");
        
        // Also update config.toml for backwards compatibility
        if let Some(key) = openrouter_key {
            crate::core::config::set_config_value("openrouter.api_key", key).await?;
        }
        
        Ok(())
    }
    
    /// Load API keys from database
    pub async fn load_from_database() -> Result<ApiKeyConfig> {
        let db = get_database().await?;
        let conn = db.get_connection()?;
        
        let openrouter_key: Option<String> = conn
            .query_row(
                "SELECT value FROM configurations WHERE key = ?1",
                params!["openrouter_api_key"],
                |row| row.get(0),
            )
            .ok();
        
        let hive_key: Option<String> = conn
            .query_row(
                "SELECT value FROM configurations WHERE key = ?1",
                params!["hive_api_key"],
                |row| row.get(0),
            )
            .ok();
        
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