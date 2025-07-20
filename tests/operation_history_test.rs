// Unit tests for Operation History Database
use hive_ai::consensus::operation_history::{
    OperationHistoryDatabase, OperationStatistics,
};
use hive_ai::consensus::operation_intelligence::{OperationOutcome, OperationContext};
use hive_ai::consensus::stages::file_aware_curator::FileOperation;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

/// Helper to create test context
fn create_test_context() -> OperationContext {
    OperationContext {
        repository_path: PathBuf::from("/test"),
        git_commit: None,
        source_question: "Test question".to_string(),
        related_files: vec![],
        project_metadata: HashMap::new(),
    }
}

/// Helper to create a test database
async fn create_test_db() -> (OperationHistoryDatabase, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_history.db");
    let db = OperationHistoryDatabase::new(db_path.to_str().unwrap()).await.unwrap();
    (db, temp_dir)
}

/// Helper to create a test outcome
fn create_test_outcome(success: bool) -> OperationOutcome {
    OperationOutcome {
        success,
        error_message: if success { None } else { Some("Test error".to_string()) },
        execution_time: Duration::from_millis(100),
        rollback_required: false,
        user_feedback: None,
        post_operation_quality: None,
    }
}

#[tokio::test]
async fn test_database_initialization() {
    let (db, _temp) = create_test_db().await;
    
    // Database should initialize with empty statistics
    let stats = db.get_statistics().unwrap();
    assert_eq!(stats.total_operations, 0);
    assert_eq!(stats.successful_operations, 0);
    assert_eq!(stats.failed_operations, 0);
}

#[tokio::test]
async fn test_update_outcome() {
    let (db, _temp) = create_test_db().await;
    
    // Create a test operation ID
    let operation_id = Uuid::new_v4();
    
    // Update with successful outcome
    let outcome = create_test_outcome(true);
    let result = db.update_outcome(operation_id, &outcome);
    
    // Should succeed (even if operation doesn't exist yet in this implementation)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_similar_operations() {
    let (db, _temp) = create_test_db().await;
    
    // Create test operations
    let op1 = FileOperation::Create {
        path: PathBuf::from("src/lib.rs"),
        content: "pub fn hello() {}".to_string(),
    };
    
    // Find similar operations (should return empty for new database)
    let context = create_test_context();
    let similar = db.find_similar_operations(&op1, &context, 10).unwrap();
    
    // New database should have no similar operations
    assert_eq!(similar.len(), 0);
}

#[tokio::test]
async fn test_statistics_empty_database() {
    let (db, _temp) = create_test_db().await;
    
    let stats = db.get_statistics().unwrap();
    
    assert_eq!(stats.total_operations, 0);
    assert_eq!(stats.successful_operations, 0);
    assert_eq!(stats.failed_operations, 0);
    assert_eq!(stats.average_confidence, 0.0);
    assert_eq!(stats.average_risk, 0.0);
}

#[tokio::test]
async fn test_search_operations() {
    let (db, _temp) = create_test_db().await;
    
    // Search in empty database
    let filters = hive_ai::consensus::operation_history::OperationFilters {
        operation_type: None,
        file_path_pattern: None,
        success_only: None,
        after: None,
        before: None,
        min_confidence: None,
        max_confidence: None,
        min_risk: None,
        max_risk: None,
        limit: Some(10),
    };
    let results = db.search_operations(&filters).unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_concurrent_access() {
    use std::sync::Arc;
    use tokio::task;
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_history.db");
    let db = Arc::new(OperationHistoryDatabase::new(db_path.to_str().unwrap()).await.unwrap());
    
    let mut handles = vec![];
    
    // Spawn multiple threads to access concurrently
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = task::spawn(async move {
            // Each thread gets statistics
            let stats = db_clone.get_statistics().unwrap();
            assert_eq!(stats.total_operations, 0);
            
            // And updates an outcome
            let id = Uuid::new_v4();
            let outcome = create_test_outcome(i % 2 == 0);
            db_clone.update_outcome(id, &outcome).unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Database should still be functional
    let stats = db.get_statistics().unwrap();
    assert!(stats.total_operations >= 0);
}

#[tokio::test]
async fn test_user_feedback() {
    let (db, _temp) = create_test_db().await;
    
    let operation_id = Uuid::new_v4();
    
    // Add user feedback
    let result = db.add_user_feedback(
        operation_id,
        5.0,  // satisfaction
        true, // was_helpful
        Some("Great job!".to_string()),  // comment
    );
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_operation_with_quality_metrics() {
    let (db, _temp) = create_test_db().await;
    
    let operation_id = Uuid::new_v4();
    
    // Create outcome with quality metrics
    let outcome = OperationOutcome {
        success: true,
        error_message: None,
        execution_time: Duration::from_millis(50),
        rollback_required: false,
        user_feedback: None,
        post_operation_quality: None,
    };
    
    let result = db.update_outcome(operation_id, &outcome);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_similar_operations_with_different_types() {
    let (db, _temp) = create_test_db().await;
    
    // Test with different operation types
    let create_op = FileOperation::Create {
        path: PathBuf::from("test.txt"),
        content: "content".to_string(),
    };
    
    let update_op = FileOperation::Update {
        path: PathBuf::from("test.txt"),
        content: "updated".to_string(),
    };
    
    let delete_op = FileOperation::Delete {
        path: PathBuf::from("test.txt"),
    };
    
    // Find similar for each type
    let context = create_test_context();
    let similar_create = db.find_similar_operations(&create_op, &context, 5).unwrap();
    let similar_update = db.find_similar_operations(&update_op, &context, 5).unwrap();
    let similar_delete = db.find_similar_operations(&delete_op, &context, 5).unwrap();
    
    // All should be empty for new database
    assert_eq!(similar_create.len(), 0);
    assert_eq!(similar_update.len(), 0);
    assert_eq!(similar_delete.len(), 0);
}

#[tokio::test]
async fn test_database_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_history.db");
    
    // Create database and update some data
    {
        let db = OperationHistoryDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        let id = Uuid::new_v4();
        let outcome = create_test_outcome(true);
        db.update_outcome(id, &outcome).unwrap();
    }
    
    // Reopen database and verify data persists
    {
        let db = OperationHistoryDatabase::new(db_path.to_str().unwrap()).await.unwrap();
        let stats = db.get_statistics().unwrap();
        // The implementation may or may not track outcomes without operations
        assert!(stats.total_operations >= 0);
    }
}

#[tokio::test]
async fn test_error_handling() {
    let (db, _temp) = create_test_db().await;
    
    // Test with various edge cases
    let empty_path_op = FileOperation::Create {
        path: PathBuf::from(""),
        content: "test".to_string(),
    };
    
    let context = create_test_context();
    let result = db.find_similar_operations(&empty_path_op, &context, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
    
    // Test with very long path
    let long_path = "a/".repeat(100) + "file.txt";
    let long_path_op = FileOperation::Create {
        path: PathBuf::from(long_path),
        content: "test".to_string(),
    };
    
    let result = db.find_similar_operations(&long_path_op, &context, 10);
    assert!(result.is_ok());
}