//! Claude Authentication Module
//! 
//! Handles OAuth PKCE flow for Claude Pro/Max subscribers and API key authentication

use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use url::Url;

/// Claude OAuth client ID
const CLAUDE_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// Claude OAuth endpoints
const CLAUDE_AUTH_ENDPOINT: &str = "https://claude.ai/oauth/authorize";
const CLAUDE_TOKEN_ENDPOINT: &str = "https://claude.ai/oauth/token";
const CLAUDE_API_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// API key authentication (standard)
    ApiKey(String),
    /// OAuth authentication (Pro/Max subscribers)
    OAuth(OAuthCredentials),
}

/// OAuth credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
    pub token_type: String,
}

/// OAuth token response
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: Option<String>,
    scope: Option<String>,
}

/// PKCE (Proof Key for Code Exchange) parameters
struct PkceParams {
    code_verifier: String,
    code_challenge: String,
}

/// Claude authentication manager
pub struct ClaudeAuth {
    client: Client,
    auth_type: RwLock<Option<AuthType>>,
    redirect_uri: String,
}

impl ClaudeAuth {
    /// Create a new authentication manager
    pub fn new(redirect_uri: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            auth_type: RwLock::new(None),
            redirect_uri,
        }
    }

    /// Generate PKCE parameters
    fn generate_pkce() -> PkceParams {
        // Generate a random code verifier (43-128 characters)
        let mut rng = rand::thread_rng();
        let code_verifier: String = (0..64)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                    .chars()
                    .nth(idx)
                    .unwrap()
            })
            .collect();

        // Generate code challenge (SHA256 of verifier)
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        PkceParams {
            code_verifier,
            code_challenge,
        }
    }

    /// Generate OAuth authorization URL
    pub fn get_auth_url(&self, state: &str) -> Result<(String, String)> {
        let pkce = Self::generate_pkce();
        
        let mut auth_url = Url::parse(CLAUDE_AUTH_ENDPOINT)?;
        auth_url.query_pairs_mut()
            .append_pair("client_id", CLAUDE_CLIENT_ID)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("scope", "messages:write messages:read")
            .append_pair("state", state)
            .append_pair("code_challenge", &pkce.code_challenge)
            .append_pair("code_challenge_method", "S256");

        Ok((auth_url.to_string(), pkce.code_verifier))
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<OAuthCredentials> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
            ("client_id", CLAUDE_CLIENT_ID),
            ("code_verifier", code_verifier),
        ];

        let response = self.client
            .post(CLAUDE_TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange authorization code")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Token exchange failed: {}", error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() + token_response.expires_in;

        let credentials = OAuthCredentials {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at,
            token_type: token_response.token_type,
        };

        // Store the credentials
        *self.auth_type.write().await = Some(AuthType::OAuth(credentials.clone()));

        Ok(credentials)
    }

    /// Refresh OAuth token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthCredentials> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", CLAUDE_CLIENT_ID),
        ];

        let response = self.client
            .post(CLAUDE_TOKEN_ENDPOINT)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Token refresh failed: {}", error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() + token_response.expires_in;

        let credentials = OAuthCredentials {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.or(Some(refresh_token.to_string())),
            expires_at,
            token_type: token_response.token_type,
        };

        // Update stored credentials
        *self.auth_type.write().await = Some(AuthType::OAuth(credentials.clone()));

        Ok(credentials)
    }

    /// Set API key authentication
    pub async fn set_api_key(&self, api_key: String) {
        *self.auth_type.write().await = Some(AuthType::ApiKey(api_key));
        info!("✅ API key authentication configured");
    }

    /// Set OAuth authentication
    pub async fn set_oauth(&self, credentials: OAuthCredentials) {
        *self.auth_type.write().await = Some(AuthType::OAuth(credentials));
        info!("✅ OAuth authentication configured");
    }

    /// Get current authentication type
    pub async fn get_auth_type(&self) -> Option<AuthType> {
        self.auth_type.read().await.clone()
    }

    /// Get authorization header for API requests
    pub async fn get_auth_header(&self) -> Result<String> {
        let auth_guard = self.auth_type.read().await;
        let auth_type = auth_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication configured"))?;

        match auth_type {
            AuthType::ApiKey(key) => Ok(key.clone()),
            AuthType::OAuth(credentials) => {
                // Check if token is expired
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs();
                
                if now >= credentials.expires_at {
                    // Clone the refresh token before dropping the guard
                    let refresh_token = credentials.refresh_token.clone();
                    drop(auth_guard); // Release read lock
                    
                    // Try to refresh token
                    if let Some(refresh_token) = refresh_token {
                        info!("🔄 Refreshing expired OAuth token...");
                        let new_credentials = self.refresh_token(&refresh_token).await?;
                        return Ok(new_credentials.access_token);
                    } else {
                        return Err(anyhow::anyhow!("OAuth token expired and no refresh token available"));
                    }
                }
                
                Ok(credentials.access_token.clone())
            }
        }
    }

    /// Check if authentication is configured
    pub async fn is_authenticated(&self) -> bool {
        self.auth_type.read().await.is_some()
    }

    /// Load credentials from secure storage
    pub async fn load_from_storage() -> Result<AuthType> {
        // TODO: Implement keychain storage for different platforms
        // For now, we'll use the database
        use crate::desktop::simple_db;
        
        // Try to load OAuth credentials first
        if let Ok(Some(oauth_json)) = simple_db::get_config("claude_oauth_credentials") {
            if let Ok(credentials) = serde_json::from_str::<OAuthCredentials>(&oauth_json) {
                return Ok(AuthType::OAuth(credentials));
            }
        }
        
        // Fall back to API key
        if let Ok(Some(api_key)) = simple_db::get_config("anthropic_api_key") {
            if !api_key.is_empty() {
                return Ok(AuthType::ApiKey(api_key));
            }
        }
        
        Err(anyhow::anyhow!("No Claude authentication found"))
    }

    /// Save credentials to secure storage
    pub async fn save_to_storage(auth_type: &AuthType) -> Result<()> {
        use crate::desktop::simple_db;
        
        match auth_type {
            AuthType::ApiKey(key) => {
                simple_db::save_config("anthropic_api_key", key)
                    .map_err(|e| anyhow::anyhow!("Failed to save API key: {}", e))?;
            }
            AuthType::OAuth(credentials) => {
                let json = serde_json::to_string(credentials)?;
                simple_db::save_config("claude_oauth_credentials", &json)
                    .map_err(|e| anyhow::anyhow!("Failed to save OAuth credentials: {}", e))?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = ClaudeAuth::generate_pkce();
        assert_eq!(pkce.code_verifier.len(), 64);
        assert!(!pkce.code_challenge.is_empty());
        
        // Verify challenge is base64url encoded
        assert!(URL_SAFE_NO_PAD.decode(&pkce.code_challenge).is_ok());
    }

    #[tokio::test]
    async fn test_auth_url_generation() {
        let auth = ClaudeAuth::new("http://localhost:8080/callback".to_string());
        let (url, verifier) = auth.get_auth_url("test_state").unwrap();
        
        assert!(url.contains(CLAUDE_CLIENT_ID));
        assert!(url.contains("code_challenge"));
        assert!(url.contains("state=test_state"));
        assert_eq!(verifier.len(), 64);
    }
}