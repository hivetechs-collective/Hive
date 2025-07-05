//! Enterprise Security and Compliance Module
//! 
//! Provides comprehensive security features including:
//! - Multi-factor authentication
//! - Advanced audit logging
//! - Compliance reporting
//! - Session management
//! - API key management

pub mod auth;
pub mod audit;
pub mod compliance;
pub mod permissions;
pub mod rbac;
pub mod teams;
pub mod trust_dialog;

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// Re-export key types
pub use auth::{
    AuthenticationManager, AuthProvider, SessionManager, Session,
    ApiKeyManager, ApiKey, MfaProvider, MfaChallenge,
};
pub use audit::{
    EnterpriseAuditLogger, AuditEvent, AuditEventType, AuditFilter,
    AuditStatistics, ComplianceReport, RetentionPolicy,
};
pub use compliance::{
    ComplianceManager, ComplianceStandard, ComplianceRule, ComplianceViolation,
    ComplianceStatus, ComplianceReport as ComplianceReportType,
};
pub use permissions::{
    PermissionManager, ResourcePermission, PermissionScope, PermissionContext,
    PermissionInheritance, PermissionTemplate,
};
pub use rbac::{
    EnterpriseRbacManager, EnterpriseRole, EnterpriseUser, SecurityGroup,
    AccessPolicy, RoleInheritance,
};
pub use teams::{
    TeamManager, EnterpriseTeam, TeamHierarchy, TeamRole,
    TeamPermissions, TeamAccess, TeamInvitation,
};
pub use trust_dialog::{
    TrustDecision, TrustScope, TrustCondition, TrustDialogConfig, TrustDialogSystem,
};

/// Main enterprise security system
pub struct SecuritySystem {
    auth_manager: Arc<AuthenticationManager>,
    audit_logger: Arc<EnterpriseAuditLogger>,
    compliance_manager: Arc<ComplianceManager>,
    permission_manager: Arc<PermissionManager>,
    rbac_manager: Arc<EnterpriseRbacManager>,
    team_manager: Arc<TeamManager>,
    trust_dialog_system: Arc<TrustDialogSystem>,
    config: SecurityConfig,
}

/// Security system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable multi-factor authentication
    pub enable_mfa: bool,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// API key expiration in days
    pub api_key_expiry_days: u32,
    /// Maximum failed login attempts
    pub max_login_attempts: u32,
    /// Audit log retention in days
    pub audit_retention_days: u32,
    /// Compliance standards to enforce
    pub compliance_standards: Vec<String>,
    /// Enable real-time monitoring
    pub enable_monitoring: bool,
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Encryption settings
    pub encryption: EncryptionConfig,
    /// Trust dialog configuration
    pub trust_dialog: TrustDialogConfig,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub max_age_days: Option<u32>,
    pub history_count: u32,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,
    pub key_rotation_days: u32,
    pub encrypt_audit_logs: bool,
    pub encrypt_session_data: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_mfa: true,
            session_timeout: 3600, // 1 hour
            api_key_expiry_days: 90,
            max_login_attempts: 5,
            audit_retention_days: 2555, // 7 years for SOX compliance
            compliance_standards: vec![
                "SOX".to_string(),
                "GDPR".to_string(),
                "ISO27001".to_string(),
            ],
            enable_monitoring: true,
            password_policy: PasswordPolicy {
                min_length: 12,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_symbols: true,
                max_age_days: Some(90),
                history_count: 5,
            },
            encryption: EncryptionConfig {
                algorithm: "AES-256-GCM".to_string(),
                key_rotation_days: 30,
                encrypt_audit_logs: true,
                encrypt_session_data: true,
            },
            trust_dialog: TrustDialogConfig::default(),
        }
    }
}

impl SecuritySystem {
    /// Create a new enterprise security system
    pub async fn new(config: SecurityConfig, db_path: Option<std::path::PathBuf>) -> Result<Self> {
        let auth_manager = Arc::new(AuthenticationManager::new(config.clone()).await?);
        let audit_logger = Arc::new(EnterpriseAuditLogger::new(
            db_path.clone(),
            config.audit_retention_days,
        ).await?);
        let compliance_manager = Arc::new(ComplianceManager::new(
            config.compliance_standards.clone(),
            audit_logger.clone(),
        ).await?);
        let permission_manager = Arc::new(PermissionManager::new().await?);
        let rbac_manager = Arc::new(EnterpriseRbacManager::new(
            permission_manager.clone(),
            audit_logger.clone(),
        ).await?);
        let team_manager = Arc::new(TeamManager::new(
            rbac_manager.clone(),
            audit_logger.clone(),
        ).await?);
        let trust_dialog_system = Arc::new(TrustDialogSystem::new(config.trust_dialog.clone()));

        Ok(Self {
            auth_manager,
            audit_logger,
            compliance_manager,
            permission_manager,
            rbac_manager,
            team_manager,
            trust_dialog_system,
            config,
        })
    }

    /// Initialize the security system with default configurations
    pub async fn initialize(&self) -> Result<()> {
        // Initialize authentication system
        self.auth_manager.initialize().await?;
        
        // Initialize RBAC with default roles
        self.rbac_manager.initialize_default_roles().await?;
        
        // Initialize compliance monitoring
        self.compliance_manager.initialize_monitoring().await?;
        
        // Set up audit logging
        self.audit_logger.initialize().await?;
        
        // Log system initialization
        self.audit_logger.log_system_event(
            AuditEventType::SystemInit,
            "Security system initialized".to_string(),
            None,
        ).await?;

        Ok(())
    }

    // Authentication methods
    pub async fn authenticate_user(&self, username: &str, password: &str, mfa_token: Option<&str>) -> Result<Session> {
        self.auth_manager.authenticate(username, password, mfa_token).await
    }

    pub async fn create_api_key(&self, user_id: &str, name: &str, permissions: Vec<String>) -> Result<ApiKey> {
        self.auth_manager.create_api_key(user_id, name, permissions).await
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<Session> {
        self.auth_manager.validate_session(session_token).await
    }

    // RBAC methods
    pub async fn create_user(&self, user: EnterpriseUser) -> Result<()> {
        let result = self.rbac_manager.create_user(user.clone()).await;
        
        self.audit_logger.log_user_event(
            AuditEventType::UserCreated,
            &user.id,
            format!("User created: {}", user.full_name),
        ).await?;
        
        result
    }

    pub async fn assign_role(&self, user_id: &str, role_name: &str, assigner_id: &str) -> Result<()> {
        // Check if assigner has permission
        if !self.rbac_manager.check_permission(
            assigner_id,
            "manage_users",
            None,
        ).await? {
            return Err(anyhow::anyhow!("Insufficient permissions to assign roles"));
        }

        let result = self.rbac_manager.assign_role(user_id, role_name).await;
        
        self.audit_logger.log_user_event(
            AuditEventType::RoleAssigned,
            user_id,
            format!("Role '{}' assigned by {}", role_name, assigner_id),
        ).await?;
        
        result
    }

    pub async fn check_permission(&self, user_id: &str, permission: &str, resource: Option<&str>) -> Result<bool> {
        self.rbac_manager.check_permission(user_id, permission, resource).await
    }

    // Team management methods
    pub async fn create_team(&self, team: EnterpriseTeam, creator_id: &str) -> Result<()> {
        let result = self.team_manager.create_team(team.clone()).await;
        
        self.audit_logger.log_team_event(
            AuditEventType::TeamCreated,
            &team.name,
            format!("Team created by {}", creator_id),
        ).await?;
        
        result
    }

    pub async fn add_user_to_team(&self, user_id: &str, team_name: &str, adder_id: &str) -> Result<()> {
        let result = self.team_manager.add_user_to_team(user_id, team_name).await;
        
        self.audit_logger.log_team_event(
            AuditEventType::UserAddedToTeam,
            team_name,
            format!("User {} added to team by {}", user_id, adder_id),
        ).await?;
        
        result
    }

    // Audit and compliance methods
    pub async fn get_audit_logs(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>> {
        self.audit_logger.query_events(filter).await
    }

    pub async fn generate_compliance_report(&self, standard: &str) -> Result<ComplianceReportType> {
        self.compliance_manager.generate_report(standard).await
    }

    pub async fn scan_compliance_violations(&self) -> Result<Vec<ComplianceViolation>> {
        self.compliance_manager.scan_violations().await
    }

    // Security monitoring
    pub async fn get_security_metrics(&self) -> Result<SecurityMetrics> {
        let auth_stats = self.auth_manager.get_statistics().await?;
        let audit_stats = self.audit_logger.get_statistics().await?;
        let rbac_stats = self.rbac_manager.get_statistics().await?;
        let team_stats = self.team_manager.get_statistics().await?;

        Ok(SecurityMetrics {
            active_sessions: auth_stats.active_sessions,
            failed_logins_24h: auth_stats.failed_logins_last_24h,
            api_keys_active: auth_stats.active_api_keys,
            total_users: rbac_stats.total_users as u64,
            total_teams: team_stats.total_teams as u64,
            audit_events_24h: audit_stats.events_last_24h,
            compliance_violations: self.compliance_manager.get_violation_count().await?,
            last_updated: Utc::now(),
        })
    }

    // Configuration management
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    pub async fn update_config(&mut self, config: SecurityConfig) -> Result<()> {
        self.config = config.clone();
        
        // Update component configurations
        self.auth_manager.update_config(config.clone()).await?;
        self.audit_logger.update_retention_policy(config.audit_retention_days).await?;
        
        self.audit_logger.log_system_event(
            AuditEventType::ConfigChanged,
            "Security configuration updated".to_string(),
            None,
        ).await?;
        
        Ok(())
    }

    // Emergency procedures
    pub async fn emergency_lockdown(&self, reason: &str, initiator_id: &str) -> Result<()> {
        // Revoke all active sessions except system accounts
        self.auth_manager.revoke_all_sessions_except_system().await?;
        
        // Log emergency action
        self.audit_logger.log_security_event(
            AuditEventType::EmergencyLockdown,
            format!("Emergency lockdown initiated by {}: {}", initiator_id, reason),
            Some(initiator_id.to_string()),
        ).await?;
        
        Ok(())
    }

    pub async fn reset_user_password(&self, user_id: &str, new_password: &str, admin_id: &str) -> Result<()> {
        // Verify admin has permission
        if !self.rbac_manager.check_permission(
            admin_id,
            "manage_users",
            None,
        ).await? {
            return Err(anyhow::anyhow!("Insufficient permissions to reset passwords"));
        }

        let result = self.auth_manager.reset_password(user_id, new_password).await;
        
        self.audit_logger.log_user_event(
            AuditEventType::PasswordReset,
            user_id,
            format!("Password reset by admin {}", admin_id),
        ).await?;
        
        result
    }

    // Access to individual managers
    pub fn auth(&self) -> Arc<AuthenticationManager> {
        self.auth_manager.clone()
    }

    pub fn audit(&self) -> Arc<EnterpriseAuditLogger> {
        self.audit_logger.clone()
    }

    pub fn compliance(&self) -> Arc<ComplianceManager> {
        self.compliance_manager.clone()
    }

    pub fn permissions(&self) -> Arc<PermissionManager> {
        self.permission_manager.clone()
    }

    pub fn rbac(&self) -> Arc<EnterpriseRbacManager> {
        self.rbac_manager.clone()
    }

    pub fn teams(&self) -> Arc<TeamManager> {
        self.team_manager.clone()
    }

    // Trust dialog methods
    pub async fn is_directory_trusted(&self, path: &std::path::Path, operation: Option<&str>) -> Result<bool> {
        self.trust_dialog_system.is_directory_trusted(path).await
    }

    pub async fn request_directory_trust(&self, path: &std::path::Path, operation: Option<&str>) -> Result<bool> {
        let reason = operation.unwrap_or("Access files in this directory");
        self.trust_dialog_system.request_directory_trust(path, reason).await
    }

    pub async fn revoke_directory_trust(&self, path: &std::path::Path) -> Result<()> {
        self.trust_dialog_system.revoke_directory_trust(path).await
    }

    pub async fn cleanup_expired_trust(&self) -> Result<usize> {
        self.trust_dialog_system.cleanup_expired_trust().await
    }

    pub async fn list_trust_decisions(&self) -> Result<Vec<TrustDecision>> {
        self.trust_dialog_system.list_trust_decisions().await
    }
}

/// Security metrics for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub active_sessions: u64,
    pub failed_logins_24h: u64,
    pub api_keys_active: u64,
    pub total_users: u64,
    pub total_teams: u64,
    pub audit_events_24h: u64,
    pub compliance_violations: u64,
    pub last_updated: DateTime<Utc>,
}

/// Security event types for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    LoginSuccess,
    LoginFailure,
    LogoutSuccess,
    SessionExpired,
    PermissionDenied,
    UnauthorizedAccess,
    ConfigurationChange,
    EmergencyAction,
    ComplianceViolation,
    AuditLogAccess,
}

/// Security alert levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Security event for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub level: SecurityAlertLevel,
    pub message: String,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_security_system_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("security_test.db");
        
        let config = SecurityConfig::default();
        let security_system = SecuritySystem::new(config, Some(db_path)).await.unwrap();
        
        security_system.initialize().await.unwrap();
        
        // Verify default roles are created
        let roles = security_system.rbac().list_roles().await.unwrap();
        assert!(!roles.is_empty());
        
        // Verify audit logging is working
        let metrics = security_system.get_security_metrics().await.unwrap();
        assert_eq!(metrics.total_users, 0); // No users created yet
    }

    #[tokio::test]
    async fn test_user_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("security_lifecycle.db");
        
        let config = SecurityConfig::default();
        let security_system = SecuritySystem::new(config, Some(db_path)).await.unwrap();
        security_system.initialize().await.unwrap();

        // Create a test user
        let user = EnterpriseUser {
            id: "test_user".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            full_name: "Test User".to_string(),
            roles: vec![],
            teams: vec![],
            security_groups: vec![],
            active: true,
            created_at: Utc::now(),
            last_login: None,
            password_hash: None,
            mfa_enabled: false,
            account_locked: false,
            lock_reason: None,
            last_activity: None,
            failed_login_attempts: 0,
            password_expires_at: None,
            must_change_password: false,
            session_timeout: None,
            ip_restrictions: vec![],
            metadata: HashMap::new(),
        };

        security_system.create_user(user).await.unwrap();
        
        // Verify user can be retrieved
        let retrieved_user = security_system.rbac().get_user("test_user").await.unwrap();
        assert!(retrieved_user.is_some());
        assert_eq!(retrieved_user.unwrap().username, "testuser");
    }
}