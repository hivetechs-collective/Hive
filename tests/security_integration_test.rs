//! Integration tests for the security and trust management system

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::TempDir;
use hive_ai::{
    TrustManager, TrustLevel, TrustDecision, SecurityPolicy,
    SecureFileAccess, FileOperations, SecurityConfig, SecurityConfigManager
};

/// Test trust manager creation and basic functionality
#[tokio::test]
async fn test_trust_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let policy = SecurityPolicy::default();
    
    let trust_manager = TrustManager::new(temp_dir.path(), policy).await;
    assert!(trust_manager.is_ok());
    
    let manager = trust_manager.unwrap();
    assert_eq!(manager.get_trust_entries().len(), 0);
}

/// Test trust persistence across manager instances
#[tokio::test]
async fn test_trust_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let policy = SecurityPolicy::default();
    let test_path = temp_dir.path().join("test_project");
    tokio::fs::create_dir(&test_path).await.unwrap();
    
    // Create first manager and add trust
    {
        let mut manager = TrustManager::new(temp_dir.path(), policy.clone()).await.unwrap();
        let trust_entry = hive_ai::core::security::TrustEntry {
            path: test_path.clone(),
            level: TrustLevel::Trusted,
            timestamp: chrono::Utc::now(),
            reason: Some("Test trust".to_string()),
        };
        manager.add_trust_entry(trust_entry).await.unwrap();
        assert!(manager.is_path_trusted(&test_path));
    }
    
    // Create second manager and verify persistence
    {
        let manager = TrustManager::new(temp_dir.path(), policy).await.unwrap();
        assert!(manager.is_path_trusted(&test_path));
        assert_eq!(manager.get_trust_entries().len(), 1);
    }
}

/// Test secure file access integration
#[tokio::test]
async fn test_secure_file_access() {
    let temp_dir = TempDir::new().unwrap();
    let policy = SecurityPolicy::default();
    let trust_manager = Arc::new(Mutex::new(
        TrustManager::new(temp_dir.path(), policy.clone()).await.unwrap()
    ));
    
    let file_access = SecureFileAccess::new(trust_manager.clone(), policy);
    
    // Test file existence check (should work without trust)
    let test_file = temp_dir.path().join("test.txt");
    let exists = file_access.file_exists(&test_file).await.unwrap();
    assert!(!exists);
    
    // Create the test file
    tokio::fs::write(&test_file, "test content").await.unwrap();
    let exists = file_access.file_exists(&test_file).await.unwrap();
    assert!(exists);
}

/// Test security configuration management
#[tokio::test]
async fn test_security_config_manager() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SecurityConfigManager::new(temp_dir.path()).await;
    assert!(manager.is_ok());
    
    let mut config_manager = manager.unwrap();
    let config = config_manager.get_config();
    
    // Test default configuration
    assert!(config.policy.trust_prompts_enabled);
    assert!(config.audit_config.enabled);
    
    // Test configuration update
    let mut new_config = config.clone();
    new_config.policy.max_file_size = 12345;
    config_manager.update_config(new_config).await.unwrap();
    
    assert_eq!(config_manager.get_config().policy.max_file_size, 12345);
}

/// Test configuration validation
#[tokio::test]
async fn test_config_validation() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = SecurityConfigManager::new(temp_dir.path()).await.unwrap();
    
    // Create a configuration with warnings
    let mut config = SecurityConfig::default();
    config.policy.trust_prompts_enabled = false; // This should generate a warning
    manager.update_config(config).await.unwrap();
    
    let warnings = manager.validate_config().unwrap();
    assert!(!warnings.is_empty());
    assert!(warnings[0].contains("Trust prompts are disabled"));
}

/// Test JSON import/export functionality
#[tokio::test]
async fn test_config_json_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = SecurityConfigManager::new(temp_dir.path()).await.unwrap();
    
    // Export configuration
    let json = manager.export_as_json().unwrap();
    assert!(!json.is_empty());
    
    // Modify and import
    let mut config: SecurityConfig = serde_json::from_str(&json).unwrap();
    config.policy.max_file_size = 54321;
    let modified_json = serde_json::to_string(&config).unwrap();
    
    manager.import_from_json(&modified_json).await.unwrap();
    assert_eq!(manager.get_config().policy.max_file_size, 54321);
}

/// Test production vs development configuration profiles
#[tokio::test]
async fn test_configuration_profiles() {
    use hive_ai::core::security_config::SecurityConfigUtils;
    
    let prod_config = SecurityConfigUtils::production_config();
    let dev_config = SecurityConfigUtils::development_config();
    let enterprise_config = SecurityConfigUtils::enterprise_config();
    
    // Production should be more restrictive
    assert!(prod_config.policy.sandbox_mode);
    assert_eq!(prod_config.trust_settings.trust_expiry_hours, 24);
    assert!(prod_config.execution.enable_sandbox);
    
    // Development should be more permissive
    assert!(!dev_config.policy.sandbox_mode);
    assert_eq!(dev_config.trust_settings.trust_expiry_hours, 0);
    assert!(!dev_config.execution.enable_sandbox);
    
    // Enterprise should have additional security features
    assert!(enterprise_config.enterprise.enabled);
    assert!(enterprise_config.enterprise.require_mfa);
    assert!(enterprise_config.audit_config.include_content_hashes);
}

/// Test file operations convenience functions
#[tokio::test]
async fn test_file_operations_convenience() {
    let temp_dir = TempDir::new().unwrap();
    let policy = SecurityPolicy::default();
    let trust_manager = Arc::new(Mutex::new(
        TrustManager::new(temp_dir.path(), policy).await.unwrap()
    ));
    
    let test_file = temp_dir.path().join("convenience_test.txt");
    let test_content = "This is test content for convenience functions";
    
    // These would normally require trust, but in test environment we can mock it
    // In real usage, these would trigger trust dialogs
    
    // Test that the convenience functions exist and can be called
    let file_access = FileOperations::new_with_trust_manager(trust_manager.clone()).await;
    assert!(file_access.file_exists(&test_file).await.unwrap() == false);
}

/// Test security policy defaults
#[test]
fn test_security_policy_defaults() {
    let policy = SecurityPolicy::default();
    
    assert!(policy.trust_prompts_enabled);
    assert!(!policy.sandbox_mode);
    assert_eq!(policy.max_file_size, 10 * 1024 * 1024); // 10MB
    assert!(policy.audit_logging);
    
    // Check that essential commands are allowed
    assert!(policy.allowed_commands.contains("cargo"));
    assert!(policy.allowed_commands.contains("git"));
    assert!(policy.allowed_commands.contains("hive"));
    
    // Check that essential domains are allowed
    assert!(policy.allowed_domains.contains("openrouter.ai"));
    assert!(policy.allowed_domains.contains("workers.dev"));
}

/// Test that trust entries have proper metadata
#[tokio::test]
async fn test_trust_entry_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let policy = SecurityPolicy::default();
    let mut manager = TrustManager::new(temp_dir.path(), policy).await.unwrap();
    
    let test_path = temp_dir.path().join("metadata_test");
    tokio::fs::create_dir(&test_path).await.unwrap();
    
    let trust_entry = hive_ai::core::security::TrustEntry {
        path: test_path.clone(),
        level: TrustLevel::Trusted,
        timestamp: chrono::Utc::now(),
        reason: Some("Test with metadata".to_string()),
    };
    
    manager.add_trust_entry(trust_entry).await.unwrap();
    
    let entries = manager.get_trust_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].level, TrustLevel::Trusted);
    assert_eq!(entries[0].reason, Some("Test with metadata".to_string()));
    assert!(entries[0].timestamp.timestamp() > 0);
}