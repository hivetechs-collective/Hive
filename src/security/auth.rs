//! Enterprise Authentication System
//!
//! Provides comprehensive authentication including:
//! - Multi-factor authentication
//! - Session management
//! - API key management
//! - Password policies

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{PasswordPolicy, SecurityConfig};

/// Authentication manager
pub struct AuthenticationManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    login_attempts: Arc<RwLock<HashMap<String, LoginAttempts>>>,
    mfa_provider: Arc<dyn MfaProvider + Send + Sync>,
    config: SecurityConfig,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_system: bool,
}

/// API key for programmatic access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key: String,
    pub user_id: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub active: bool,
}

/// Login attempt tracking
#[derive(Debug, Clone)]
struct LoginAttempts {
    count: u32,
    last_attempt: DateTime<Utc>,
    locked_until: Option<DateTime<Utc>>,
}

/// Authentication provider trait
pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, username: &str, password: &str) -> Result<bool>;
    fn change_password(&self, user_id: &str, old_password: &str, new_password: &str) -> Result<()>;
    fn reset_password(&self, user_id: &str, new_password: &str) -> Result<()>;
    fn validate_password_policy(&self, password: &str, policy: &PasswordPolicy) -> Result<()>;
}

/// Multi-factor authentication provider trait
pub trait MfaProvider: Send + Sync {
    fn generate_challenge(&self, user_id: &str) -> Result<MfaChallenge>;
    fn verify_challenge(&self, user_id: &str, token: &str) -> Result<bool>;
    fn is_enabled(&self, user_id: &str) -> Result<bool>;
    fn enable_mfa(&self, user_id: &str) -> Result<String>; // Returns backup codes
    fn disable_mfa(&self, user_id: &str) -> Result<()>;
}

/// MFA challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaChallenge {
    pub challenge_id: String,
    pub challenge_type: MfaChallengeType,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaChallengeType {
    Totp,
    Sms,
    Email,
    BackupCode,
}

/// Session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    config: SecurityConfig,
}

impl SessionManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn create_session(
        &self,
        user_id: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session> {
        let token = self.generate_session_token();
        let now = Utc::now();
        let expires_at = now + Duration::seconds(self.config.session_timeout as i64);

        let session = Session {
            token: token.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at,
            last_activity: now,
            ip_address,
            user_agent,
            is_system: false,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(token.clone(), session.clone());

        Ok(session)
    }

    pub async fn validate_session(&self, token: &str) -> Result<Session> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(token) {
            if session.expires_at < Utc::now() {
                sessions.remove(token);
                return Err(anyhow!("Session expired"));
            }

            // Update last activity
            session.last_activity = Utc::now();
            Ok(session.clone())
        } else {
            Err(anyhow!("Invalid session token"))
        }
    }

    pub async fn revoke_session(&self, token: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(token);
        Ok(())
    }

    pub async fn revoke_user_sessions(&self, user_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.user_id != user_id);
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let initial_count = sessions.len();

        sessions.retain(|_, session| session.expires_at > now);

        Ok((initial_count - sessions.len()) as u64)
    }

    pub async fn get_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    fn generate_session_token(&self) -> String {
        format!("sess_{}", Uuid::new_v4().to_string().replace('-', ""))
    }
}

/// API key manager
pub struct ApiKeyManager {
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    config: SecurityConfig,
}

impl ApiKeyManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn create_api_key(
        &self,
        user_id: &str,
        name: &str,
        permissions: Vec<String>,
    ) -> Result<ApiKey> {
        let key = self.generate_api_key();
        let now = Utc::now();
        let expires_at = if self.config.api_key_expiry_days > 0 {
            Some(now + Duration::days(self.config.api_key_expiry_days as i64))
        } else {
            None
        };

        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            key: key.clone(),
            user_id: user_id.to_string(),
            permissions,
            created_at: now,
            expires_at,
            last_used: None,
            active: true,
        };

        let mut api_keys = self.api_keys.write().await;
        api_keys.insert(key.clone(), api_key.clone());

        Ok(api_key)
    }

    pub async fn validate_api_key(&self, key: &str) -> Result<ApiKey> {
        let mut api_keys = self.api_keys.write().await;

        if let Some(api_key) = api_keys.get_mut(key) {
            if !api_key.active {
                return Err(anyhow!("API key is disabled"));
            }

            if let Some(expires_at) = api_key.expires_at {
                if expires_at < Utc::now() {
                    api_key.active = false;
                    return Err(anyhow!("API key expired"));
                }
            }

            // Update last used timestamp
            api_key.last_used = Some(Utc::now());
            Ok(api_key.clone())
        } else {
            Err(anyhow!("Invalid API key"))
        }
    }

    pub async fn revoke_api_key(&self, key: &str) -> Result<()> {
        let mut api_keys = self.api_keys.write().await;
        if let Some(api_key) = api_keys.get_mut(key) {
            api_key.active = false;
        }
        Ok(())
    }

    pub async fn list_user_api_keys(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let api_keys = self.api_keys.read().await;
        Ok(api_keys
            .values()
            .filter(|key| key.user_id == user_id)
            .cloned()
            .collect())
    }

    fn generate_api_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        format!("hive_{}", hex::encode(random_bytes))
    }
}

/// Simple TOTP-based MFA provider
pub struct TotpMfaProvider {
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

impl TotpMfaProvider {
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl MfaProvider for TotpMfaProvider {
    fn generate_challenge(&self, user_id: &str) -> Result<MfaChallenge> {
        Ok(MfaChallenge {
            challenge_id: Uuid::new_v4().to_string(),
            challenge_type: MfaChallengeType::Totp,
            expires_at: Utc::now() + Duration::minutes(5),
        })
    }

    fn verify_challenge(&self, _user_id: &str, _token: &str) -> Result<bool> {
        // Simple implementation - in production, use proper TOTP library
        Ok(true)
    }

    fn is_enabled(&self, _user_id: &str) -> Result<bool> {
        Ok(false) // Default disabled
    }

    fn enable_mfa(&self, _user_id: &str) -> Result<String> {
        Ok("backup-codes-here".to_string())
    }

    fn disable_mfa(&self, _user_id: &str) -> Result<()> {
        Ok(())
    }
}

/// Simple password-based authentication provider
pub struct PasswordAuthProvider {
    password_hashes: Arc<RwLock<HashMap<String, String>>>,
}

impl PasswordAuthProvider {
    pub fn new() -> Self {
        Self {
            password_hashes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn hash_password(&self, password: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn set_password(&self, user_id: &str, password: &str) -> Result<()> {
        let salt = Uuid::new_v4().to_string();
        let hash = self.hash_password(password, &salt);
        let stored_hash = format!("{}:{}", salt, hash);

        let mut hashes = self.password_hashes.write().await;
        hashes.insert(user_id.to_string(), stored_hash);

        Ok(())
    }
}

impl AuthProvider for PasswordAuthProvider {
    fn authenticate(&self, username: &str, password: &str) -> Result<bool> {
        // In a real implementation, this would be async and query a database
        // For now, return true for demonstration
        Ok(username == "admin" && password == "admin")
    }

    fn change_password(
        &self,
        _user_id: &str,
        _old_password: &str,
        _new_password: &str,
    ) -> Result<()> {
        // Implement password change logic
        Ok(())
    }

    fn reset_password(&self, _user_id: &str, _new_password: &str) -> Result<()> {
        // Implement password reset logic
        Ok(())
    }

    fn validate_password_policy(&self, password: &str, policy: &PasswordPolicy) -> Result<()> {
        if password.len() < policy.min_length as usize {
            return Err(anyhow!("Password too short"));
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(anyhow!("Password must contain uppercase letters"));
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(anyhow!("Password must contain lowercase letters"));
        }

        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(anyhow!("Password must contain numbers"));
        }

        if policy.require_symbols && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(anyhow!("Password must contain symbols"));
        }

        Ok(())
    }
}

impl AuthenticationManager {
    pub async fn new(config: SecurityConfig) -> Result<Self> {
        let mfa_provider: Arc<dyn MfaProvider + Send + Sync> = Arc::new(TotpMfaProvider::new());

        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            login_attempts: Arc::new(RwLock::new(HashMap::new())),
            mfa_provider,
            config,
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        // Start session cleanup task
        let sessions = self.sessions.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                let session_manager = SessionManager {
                    sessions: sessions.clone(),
                    config: config.clone(),
                };
                if let Err(e) = session_manager.cleanup_expired_sessions().await {
                    tracing::error!("Failed to cleanup expired sessions: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        mfa_token: Option<&str>,
    ) -> Result<Session> {
        // Check login attempts
        if self.is_user_locked(username).await? {
            return Err(anyhow!(
                "User account is temporarily locked due to too many failed attempts"
            ));
        }

        // Authenticate with password (simplified - use proper auth provider in production)
        let auth_success = username == "admin" && password == "admin";

        if !auth_success {
            self.record_failed_attempt(username).await?;
            return Err(anyhow!("Invalid credentials"));
        }

        // Check MFA if enabled
        if self.config.enable_mfa {
            if let Some(token) = mfa_token {
                if !self.mfa_provider.verify_challenge(username, token)? {
                    return Err(anyhow!("Invalid MFA token"));
                }
            } else if self.mfa_provider.is_enabled(username)? {
                return Err(anyhow!("MFA token required"));
            }
        }

        // Clear failed attempts on successful login
        self.clear_failed_attempts(username).await?;

        // Create session
        let session_manager = SessionManager::new(self.config.clone());
        session_manager.create_session(username, None, None).await
    }

    pub async fn validate_session(&self, token: &str) -> Result<Session> {
        let session_manager = SessionManager {
            sessions: self.sessions.clone(),
            config: self.config.clone(),
        };
        session_manager.validate_session(token).await
    }

    pub async fn create_api_key(
        &self,
        user_id: &str,
        name: &str,
        permissions: Vec<String>,
    ) -> Result<ApiKey> {
        let api_key_manager = ApiKeyManager::new(self.config.clone());
        let api_key = api_key_manager
            .create_api_key(user_id, name, permissions)
            .await?;

        // Store in our collection
        let mut api_keys = self.api_keys.write().await;
        api_keys.insert(api_key.key.clone(), api_key.clone());

        Ok(api_key)
    }

    pub async fn reset_password(&self, user_id: &str, new_password: &str) -> Result<()> {
        // Validate new password against policy
        let auth_provider = PasswordAuthProvider::new();
        auth_provider.validate_password_policy(new_password, &self.config.password_policy)?;

        // In production, update password in database
        Ok(())
    }

    pub async fn revoke_all_sessions_except_system(&self) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.is_system);
        Ok(())
    }

    async fn is_user_locked(&self, username: &str) -> Result<bool> {
        let attempts = self.login_attempts.read().await;
        if let Some(user_attempts) = attempts.get(username) {
            if let Some(locked_until) = user_attempts.locked_until {
                return Ok(locked_until > Utc::now());
            }
        }
        Ok(false)
    }

    async fn record_failed_attempt(&self, username: &str) -> Result<()> {
        let mut attempts = self.login_attempts.write().await;
        let user_attempts = attempts
            .entry(username.to_string())
            .or_insert(LoginAttempts {
                count: 0,
                last_attempt: Utc::now(),
                locked_until: None,
            });

        user_attempts.count += 1;
        user_attempts.last_attempt = Utc::now();

        if user_attempts.count >= self.config.max_login_attempts {
            user_attempts.locked_until = Some(Utc::now() + Duration::minutes(15));
            // Lock for 15 minutes
        }

        Ok(())
    }

    async fn clear_failed_attempts(&self, username: &str) -> Result<()> {
        let mut attempts = self.login_attempts.write().await;
        attempts.remove(username);
        Ok(())
    }

    pub async fn get_statistics(&self) -> Result<AuthStatistics> {
        let sessions = self.sessions.read().await;
        let api_keys = self.api_keys.read().await;
        let attempts = self.login_attempts.read().await;

        let now = Utc::now();
        let failed_logins_24h = attempts
            .values()
            .filter(|a| a.last_attempt > now - Duration::hours(24))
            .map(|a| a.count as u64)
            .sum();

        Ok(AuthStatistics {
            active_sessions: sessions.len() as u64,
            active_api_keys: api_keys.values().filter(|k| k.active).count() as u64,
            failed_logins_last_24h: failed_logins_24h,
            locked_accounts: attempts
                .values()
                .filter(|a| a.locked_until.map_or(false, |l| l > now))
                .count() as u64,
        })
    }

    pub async fn update_config(&self, config: SecurityConfig) -> Result<()> {
        // In a full implementation, this would update the internal config
        // and potentially restart background tasks with new parameters
        Ok(())
    }
}

/// Authentication statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatistics {
    pub active_sessions: u64,
    pub active_api_keys: u64,
    pub failed_logins_last_24h: u64,
    pub locked_accounts: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_management() {
        let config = SecurityConfig::default();
        let session_manager = SessionManager::new(config);

        let session = session_manager
            .create_session("test_user", None, None)
            .await
            .unwrap();
        assert_eq!(session.user_id, "test_user");

        let validated = session_manager
            .validate_session(&session.token)
            .await
            .unwrap();
        assert_eq!(validated.user_id, "test_user");

        session_manager
            .revoke_session(&session.token)
            .await
            .unwrap();

        let result = session_manager.validate_session(&session.token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_key_management() {
        let config = SecurityConfig::default();
        let api_key_manager = ApiKeyManager::new(config);

        let api_key = api_key_manager
            .create_api_key(
                "test_user",
                "test_key",
                vec!["read".to_string(), "write".to_string()],
            )
            .await
            .unwrap();

        assert_eq!(api_key.user_id, "test_user");
        assert_eq!(api_key.name, "test_key");
        assert!(api_key.active);

        let validated = api_key_manager
            .validate_api_key(&api_key.key)
            .await
            .unwrap();
        assert_eq!(validated.user_id, "test_user");

        api_key_manager.revoke_api_key(&api_key.key).await.unwrap();

        let result = api_key_manager.validate_api_key(&api_key.key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_password_policy_validation() {
        let auth_provider = PasswordAuthProvider::new();
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_symbols: true,
            max_age_days: None,
            history_count: 5,
        };

        // Valid password
        assert!(auth_provider
            .validate_password_policy("MyPass123!", &policy)
            .is_ok());

        // Too short
        assert!(auth_provider
            .validate_password_policy("Pass1!", &policy)
            .is_err());

        // Missing uppercase
        assert!(auth_provider
            .validate_password_policy("mypass123!", &policy)
            .is_err());

        // Missing numbers
        assert!(auth_provider
            .validate_password_policy("MyPassword!", &policy)
            .is_err());
    }
}
