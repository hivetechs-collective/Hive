//! Integration tests for the security system

use hive_ai::core::{
    check_read_access, check_write_access, get_security_context, initialize_security,
    SecurityContext, TrustLevel,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_security_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_security.db");

    // Initialize security system
    initialize_security(Some(db_path), false).unwrap();

    // Verify we can get the context
    let context = get_security_context().unwrap();
    assert!(context.get_trusted_paths().unwrap().is_empty());
}

#[tokio::test]
async fn test_trust_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_security.db");
    let test_path = temp_dir.path().join("test_dir");

    // Create a new context
    {
        let context = SecurityContext::new(Some(db_path.clone()), false).unwrap();

        // Trust a path
        context
            .set_trust_level(&test_path, TrustLevel::Trusted, Some("Test".to_string()))
            .unwrap();

        // Verify it's trusted
        assert!(context.is_trusted(&test_path).unwrap());
    }

    // Create a new context to test persistence
    {
        let context = SecurityContext::new(Some(db_path), false).unwrap();

        // Verify the trust persisted
        assert!(context.is_trusted(&test_path).unwrap());
    }
}

#[tokio::test]
async fn test_parent_directory_trust_inheritance() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_security.db");

    let context = SecurityContext::new(Some(db_path), false).unwrap();

    // Trust parent directory
    context
        .set_trust_level(temp_dir.path(), TrustLevel::Trusted, None)
        .unwrap();

    // Child paths should inherit trust
    let child1 = temp_dir.path().join("subdir/file.txt");
    let child2 = temp_dir.path().join("another/deep/path/file.rs");

    assert!(context.is_trusted(&child1).unwrap());
    assert!(context.is_trusted(&child2).unwrap());
}

#[tokio::test]
async fn test_temporary_trust() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_security.db");
    let test_path = temp_dir.path().join("temp_trusted");

    let context = SecurityContext::new(Some(db_path), false).unwrap();

    // Set temporary trust
    context
        .set_trust_level(
            &test_path,
            TrustLevel::Temporary,
            Some("Temporary access".to_string()),
        )
        .unwrap();

    // Should be trusted now
    assert!(context.is_trusted(&test_path).unwrap());

    // Clear temporary trusts
    context.clear_temporary_trusts().unwrap();

    // Should no longer be trusted
    assert!(!context.is_trusted(&test_path).unwrap());
}

#[tokio::test]
async fn test_security_event_logging() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_security.db");
    let test_path = temp_dir.path().join("test_file.txt");

    let context = SecurityContext::new(Some(db_path), false).unwrap();

    // Set trust and trigger some events
    context
        .set_trust_level(
            &test_path,
            TrustLevel::Trusted,
            Some("Test logging".to_string()),
        )
        .unwrap();
    let _ = context.check_read_access(&test_path);
    let _ = context.check_write_access(&test_path);

    // Get events
    let events = context.get_events_for_path(&test_path).unwrap();

    // Should have at least the trust decision event
    assert!(!events.is_empty());

    // Check event types
    let has_trust_decision = events.iter().any(|e| {
        matches!(
            e.event_type,
            hive_ai::core::SecurityEventType::TrustDecision
        )
    });
    assert!(has_trust_decision);
}

#[tokio::test]
async fn test_untrusted_access_denial() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_security.db");
    let untrusted_path = PathBuf::from("/tmp/definitely_not_trusted_path_12345");

    let context = SecurityContext::new(Some(db_path), false).unwrap();

    // Non-interactive mode should deny access
    let result = context.check_read_access(&untrusted_path);
    assert!(result.is_err());

    let result = context.check_write_access(&untrusted_path);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trusted_paths_listing() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_security.db");

    let context = SecurityContext::new(Some(db_path), false).unwrap();

    // Add multiple trusted paths
    let path1 = temp_dir.path().join("project1");
    let path2 = temp_dir.path().join("project2");
    let path3 = temp_dir.path().join("temp_project");

    context
        .set_trust_level(&path1, TrustLevel::Trusted, None)
        .unwrap();
    context
        .set_trust_level(&path2, TrustLevel::Trusted, None)
        .unwrap();
    context
        .set_trust_level(&path3, TrustLevel::Temporary, None)
        .unwrap();

    // Get trusted paths
    let trusted = context.get_trusted_paths().unwrap();

    // Should have all 3 paths
    assert_eq!(trusted.len(), 3);

    // Check trust levels
    let permanent_count = trusted
        .iter()
        .filter(|(_, level)| *level == TrustLevel::Trusted)
        .count();
    let temporary_count = trusted
        .iter()
        .filter(|(_, level)| *level == TrustLevel::Temporary)
        .count();

    assert_eq!(permanent_count, 2);
    assert_eq!(temporary_count, 1);
}
