//! Conversation Gateway - D1 Backend Communication
//! 
//! Handles secure communication with Cloudflare D1 backend for:
//! - Pre-conversation authorization and usage validation
//! - Post-conversation verification and usage reporting
//! - License key validation and user profile fetching

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

use super::{SubscriptionInfo, SubscriptionTier, ConversationVerification};

/// Custom deserializer for handling "unlimited" strings or numbers
fn deserialize_unlimited_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct UnlimitedVisitor;

    impl<'de> Visitor<'de> for UnlimitedVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number or the string 'unlimited'")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as u32)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value == "unlimited" {
                Ok(u32::MAX) // Use max value to represent unlimited
            } else {
                value.parse().map_err(de::Error::custom)
            }
        }
    }

    deserializer.deserialize_any(UnlimitedVisitor)
}

/// Optional version of the unlimited deserializer
fn deserialize_unlimited_u32_opt<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct UnlimitedOptVisitor;

    impl<'de> Visitor<'de> for UnlimitedOptVisitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number, the string 'unlimited', or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize_unlimited_u32(deserializer).map(Some)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value as u32))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value == "unlimited" {
                Ok(Some(u32::MAX))
            } else {
                value.parse().map(Some).map_err(de::Error::custom)
            }
        }
    }

    deserializer.deserialize_option(UnlimitedOptVisitor)
}

/// Conversation authorization token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationAuthorization {
    pub conversation_token: String,
    pub question_hash: String,
    pub user_id: String,
    #[serde(deserialize_with = "deserialize_unlimited_u32_opt", default)]
    pub remaining: Option<u32>,
    #[serde(deserialize_with = "deserialize_unlimited_u32_opt", default)]
    pub limit: Option<u32>,
    pub expires_at: DateTime<Utc>,
}

/// User profile from D1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub email: String,
    pub tier: String,
    pub daily_limit: u32,
    pub features: Vec<String>,
    pub is_valid: bool,
}

/// D1 API response for pre-conversation
#[derive(Debug, Deserialize)]
struct PreConversationResponse {
    allowed: bool,
    token: Option<String>,
    conversation_token: Option<String>,
    #[serde(deserialize_with = "deserialize_unlimited_u32_opt", default)]
    remaining: Option<u32>,
    #[serde(deserialize_with = "deserialize_unlimited_u32_opt", default)]
    remaining_conversations: Option<u32>,
    limits: Option<LimitsInfo>,
    #[serde(deserialize_with = "deserialize_unlimited_u32_opt", default)]
    plan_limit: Option<u32>,
    user: Option<UserInfo>,
    user_id: Option<String>,
    email: Option<String>,
    expires_at: Option<String>,
    error: Option<String>,
    used_conversations: Option<u32>,
    plan: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LimitsInfo {
    #[serde(deserialize_with = "deserialize_unlimited_u32")]
    daily: u32,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    id: String,
    email: Option<String>,
    subscription_tier: Option<String>,
}

/// D1 API response for post-conversation
#[derive(Debug, Deserialize)]
struct PostConversationResponse {
    success: Option<bool>,
    verified: Option<bool>,
    remaining: Option<u32>,
    remaining_conversations: Option<u32>,
    error: Option<String>,
}

/// D1 API response for license validation
#[derive(Debug, Deserialize)]
struct ValidateResponse {
    valid: bool,
    status: Option<String>,
    user: Option<UserInfo>,
    user_id: Option<String>,
    email: Option<String>,
    tier: Option<String>,
    limits: Option<LimitsInfo>,
    daily_limit: Option<u32>,
    features: Option<Vec<String>>,
    usage: Option<UsageInfo>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageInfo {
    #[serde(deserialize_with = "deserialize_unlimited_u32")]
    remaining: u32,
    #[serde(deserialize_with = "deserialize_unlimited_u32")]
    limit: u32,
}

/// Gateway errors
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("{} | {} | Daily limit reached ({}/{})", user.as_deref().unwrap_or("unknown"), plan, used, limit)]
    UsageLimitExceeded { used: u32, limit: u32, plan: String, user: Option<String> },
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Gateway for secure D1 backend communication
pub struct ConversationGateway {
    api_url: String,
    client: Client,
}

impl ConversationGateway {
    pub fn new() -> Result<Self> {
        let api_url = std::env::var("HIVE_API_ENDPOINT")
            .unwrap_or_else(|_| "https://gateway.hivetechs.io".to_string());
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("hive-rust/2.0.0")
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self { api_url, client })
    }

    /// Request conversation authorization from D1 backend
    /// This is called BEFORE starting any consensus pipeline
    pub async fn request_conversation_authorization(&self, question: &str, license_key: &str) -> Result<ConversationAuthorization> {
        // Generate hash of the question for verification
        let question_hash = self.generate_question_hash(question);
        let installation_id = self.get_installation_id().await?;

        tracing::info!("Requesting conversation authorization from D1...");

        let request_data = json!({
            "license_key": license_key,
            "installation_id": installation_id,
            "conversation_request_hash": question_hash
        });

        let response = self.client
            .post(format!("{}/auth/pre-conversation", self.api_url))
            .header("Authorization", format!("Bearer {}", license_key))
            .json(&request_data)
            .send()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            
            tracing::error!("D1 authorization failed: {} - {}", status, error_text);
            
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<PreConversationResponse>(&error_text) {
                if let Some(error) = error_response.error {
                    return Err(GatewayError::AuthenticationFailed(error).into());
                }
            }
            
            // Extract user information from error response if available
            let user_info = if let Ok(error_response) = serde_json::from_str::<PreConversationResponse>(&error_text) {
                error_response.user
                    .as_ref()
                    .and_then(|u| {
                        if !u.id.is_empty() {
                            Some(u.id.clone())
                        } else {
                            u.email.clone()
                        }
                    })
                    .or(error_response.user_id)
                    .or(error_response.email)
            } else {
                None
            };
            
            // Extract a more meaningful error message from the response
            let error_msg = if error_text.contains("Invalid or inactive license") {
                if let Some(user) = user_info {
                    format!("{} | Invalid or inactive license", user)
                } else {
                    "Invalid or inactive license".to_string()
                }
            } else if error_text.contains("No active subscription") {
                if let Some(user) = user_info {
                    format!("{} | No active subscription", user)
                } else {
                    "No active subscription found".to_string()
                }
            } else {
                format!("Authorization failed with status {}", status)
            };
            
            return Err(GatewayError::AuthenticationFailed(error_msg).into());
        }

        let response_text = response.text().await
            .map_err(|e| GatewayError::InvalidResponse(e.to_string()))?;
        
        // Log the raw response
        tracing::info!("D1 raw response: {}", response_text);
        
        let result: PreConversationResponse = serde_json::from_str(&response_text)
            .map_err(|e| GatewayError::InvalidResponse(format!("Failed to parse response: {} - Raw: {}", e, response_text)))?;

        // Debug log the parsed response
        tracing::info!("D1 parsed response: allowed={}, user={:?}, user_id={:?}, email={:?}, remaining={:?}, limit={:?}", 
            result.allowed, result.user, result.user_id, result.email, result.remaining, result.plan_limit);

        if !result.allowed {
            let used = result.used_conversations.unwrap_or(0);
            let limit = result.plan_limit.unwrap_or(10);
            let plan = result.plan.unwrap_or_else(|| "FREE".to_string());
            let user = result.user
                .as_ref()
                .and_then(|u| {
                    if !u.id.is_empty() {
                        Some(u.id.clone())
                    } else {
                        u.email.clone()
                    }
                })
                .or(result.user_id.clone())
                .or(result.email.clone());
            
            return Err(GatewayError::UsageLimitExceeded { used, limit, plan, user }.into());
        }

        tracing::info!("Conversation authorized ({} remaining)", 
            result.remaining.or(result.remaining_conversations).unwrap_or(0));

        // Map D1 response format to our expected format
        Ok(ConversationAuthorization {
            conversation_token: result.token
                .or(result.conversation_token)
                .ok_or_else(|| anyhow!("No conversation token in response"))?,
            question_hash,
            user_id: result.user
                .as_ref()
                .and_then(|u| {
                    // Try id first, then email
                    if !u.id.is_empty() {
                        Some(u.id.clone())
                    } else {
                        u.email.clone()
                    }
                })
                .or(result.user_id.clone())
                .or(result.email.clone())
                .unwrap_or_else(|| "unknown".to_string()), // D1 doesn't always return user_id
            remaining: result.remaining
                .or(result.remaining_conversations), // Let it be None for unlimited
            limit: result.limits
                .map(|l| l.daily)
                .or(result.plan_limit), // Let it be None for unlimited
            expires_at: if let Some(expires) = result.expires_at {
                DateTime::parse_from_rfc3339(&expires)?
                    .with_timezone(&Utc)
            } else {
                Utc::now() + Duration::hours(1)
            },
        })
    }

    /// Report conversation completion to D1 backend
    /// This is called AFTER successful consensus pipeline completion
    pub async fn report_conversation_completion(
        &self,
        conversation_token: &str,
        conversation_id: &str,
        question_hash: &str
    ) -> Result<ConversationVerification> {
        // Generate usage proof HMAC
        let usage_proof = self.generate_usage_proof(conversation_token, conversation_id, question_hash);

        tracing::info!("Reporting conversation completion to D1...");

        let request_data = json!({
            "conversation_token": conversation_token,
            "conversation_id": conversation_id,
            "usage_proof": usage_proof,
            "timestamp": Utc::now().to_rfc3339()
        });

        let response = self.client
            .post(format!("{}/auth/post-conversation", self.api_url))
            .json(&request_data)
            .send()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::warn!("Conversation verification failed: {}", error_text);
            
            // Don't throw here - we don't want to break the user's workflow
            // The server will handle the security implications
            return Ok(ConversationVerification {
                verified: false,
                remaining_conversations: 0,
                usage_updated: false,
            });
        }

        let result: PostConversationResponse = response
            .json()
            .await
            .map_err(|e| GatewayError::InvalidResponse(e.to_string()))?;

        let remaining = result.remaining
            .or(result.remaining_conversations)
            .unwrap_or(0);
        
        tracing::info!("Conversation verified ({} remaining)", remaining);

        // Map D1 response format to our expected format
        Ok(ConversationVerification {
            verified: result.success.or(result.verified).unwrap_or(false),
            remaining_conversations: remaining,
            usage_updated: result.success.or(result.verified).unwrap_or(false),
        })
    }

    /// Validate license key against D1 backend and fetch user profile
    /// This is called during license configuration
    pub async fn validate_license_key(&self, license_key: &str) -> Result<UserProfile> {
        tracing::info!("Validating license key with D1 backend...");

        let request_data = json!({
            "client_id": "hive-tools",
            "session_token": license_key,
            "fingerprint": self.get_device_fingerprint().await?,
            "nonce": Utc::now().timestamp_millis().to_string()
        });

        let response = self.client
            .post(format!("{}/v1/session/validate", self.api_url))
            .header("Authorization", format!("Bearer {}", license_key))
            .json(&request_data)
            .send()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            
            return Err(GatewayError::AuthenticationFailed(
                format!("License validation failed: {} - {}", status, error_text)
            ).into());
        }

        let result: ValidateResponse = response
            .json()
            .await
            .map_err(|e| GatewayError::InvalidResponse(e.to_string()))?;

        if !result.valid {
            let error = result.error.unwrap_or_else(|| "Invalid license key".to_string());
            return Err(GatewayError::AuthenticationFailed(error).into());
        }

        let tier = result.user.as_ref()
            .and_then(|u| u.subscription_tier.as_ref())
            .or(result.tier.as_ref())
            .unwrap_or(&"free".to_string())
            .clone();
            
        tracing::info!("License validated - {} tier", tier);

        // Map D1 response format to our expected format
        Ok(UserProfile {
            user_id: result.user.as_ref()
                .map(|u| u.id.clone())
                .or(result.user_id)
                .ok_or_else(|| anyhow!("No user ID in response"))?,
            email: result.user.as_ref()
                .and_then(|u| u.email.clone())
                .or(result.email)
                .unwrap_or_default(),
            tier: result.user.as_ref()
                .and_then(|u| u.subscription_tier.clone())
                .or(result.tier)
                .unwrap_or_else(|| "free".to_string()),
            daily_limit: result.limits
                .map(|l| l.daily)
                .or(result.daily_limit)
                .unwrap_or(10),
            features: result.features.unwrap_or_else(|| vec!["consensus".to_string()]),
            is_valid: result.valid || result.status == Some("active".to_string()),
        })
    }

    /// Get quick usage status without full authorization
    /// Used for status displays
    pub async fn get_quick_usage_status(&self, license_key: &str) -> Result<(u32, u32)> {
        let request_data = json!({
            "client_id": "hive-tools",
            "session_token": license_key,
            "fingerprint": self.get_device_fingerprint().await?,
            "nonce": Utc::now().timestamp_millis().to_string()
        });

        let response = self.client
            .post(format!("{}/v1/session/validate", self.api_url))
            .header("Authorization", format!("Bearer {}", license_key))
            .json(&request_data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: ValidateResponse = response.json().await?;
            
            let remaining = result.usage
                .as_ref()
                .map(|u| u.remaining)
                .unwrap_or(0);
            
            let limit = result.usage
                .as_ref()
                .map(|u| u.limit)
                .or(result.limits.map(|l| l.daily))
                .or(result.daily_limit)
                .unwrap_or(10);
            
            Ok((remaining, limit))
        } else {
            Ok((0, 10)) // Default values on error
        }
    }

    /// Generate a hash of the question for verification
    fn generate_question_hash(&self, question: &str) -> String {
        // Normalize the question (trim whitespace, convert to lowercase)
        let normalized = question.trim().to_lowercase();
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate HMAC proof of conversation completion
    fn generate_usage_proof(&self, conversation_token: &str, conversation_id: &str, question_hash: &str) -> String {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        
        let payload = format!("{}:{}", conversation_id, question_hash);
        let mut mac = HmacSha256::new_from_slice(conversation_token.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    }

    /// Check if a conversation token is expired
    pub fn is_token_expired(&self, expires_at: &DateTime<Utc>) -> bool {
        Utc::now() > *expires_at
    }

    /// Get installation ID from device/machine fingerprinting
    async fn get_installation_id(&self) -> Result<String> {
        use sysinfo::System;
        
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        let system = System::new_all();
        let os_version = System::os_version().unwrap_or_default();
        
        let machine_data = json!({
            "hostname": hostname,
            "platform": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "homedir": dirs::home_dir().map(|p| p.to_string_lossy().to_string()),
            "username": username,
            "os_version": os_version,
        });
        
        let machine_string = serde_json::to_string(&machine_data)?;
        let mut hasher = Sha256::new();
        hasher.update(machine_string.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash[..16].to_string())
    }

    /// Get device fingerprint for security validation
    async fn get_device_fingerprint(&self) -> Result<String> {
        use sysinfo::System;
        
        let system = System::new_all();
        
        let device_data = json!({
            "platform": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "release": System::os_version().unwrap_or_default(),
            "cpus": system.cpus().len(),
            "memory": system.total_memory() / 1024 / 1024, // MB
        });
        
        let device_string = serde_json::to_string(&device_data)?;
        let mut hasher = Sha256::new();
        hasher.update(device_string.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash[..32].to_string())
    }
}

impl Default for ConversationGateway {
    fn default() -> Self {
        Self::new().expect("Failed to create conversation gateway")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_hash_generation() {
        let gateway = ConversationGateway::default();
        
        // Same question with different whitespace should produce same hash
        let hash1 = gateway.generate_question_hash("What is Rust?");
        let hash2 = gateway.generate_question_hash("  what is rust?  ");
        let hash3 = gateway.generate_question_hash("WHAT IS RUST?");
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
        
        // Different questions should produce different hashes
        let hash4 = gateway.generate_question_hash("What is Python?");
        assert_ne!(hash1, hash4);
    }

    #[test]
    fn test_token_expiry() {
        let gateway = ConversationGateway::default();
        
        let future_time = Utc::now() + Duration::hours(1);
        assert!(!gateway.is_token_expired(&future_time));
        
        let past_time = Utc::now() - Duration::hours(1);
        assert!(gateway.is_token_expired(&past_time));
    }
}