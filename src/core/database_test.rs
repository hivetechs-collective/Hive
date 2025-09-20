use anyhow::Result;
use rusqlite::params;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::core::database::{
    execute_transaction, get_database, initialize_database, ActivityLog, ConsensusProfile,
    Conversation, DatabaseConfig, DatabaseManager, KnowledgeConversation, Message, User,
};

/// Integration test for complete database setup with migrations
#[tokio::test]
async fn test_complete_database_setup() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");

    // Configure database
    let config = DatabaseConfig {
        path: db_path.clone(),
        max_connections: 5,
        connection_timeout: std::time::Duration::from_secs(5),
        idle_timeout: std::time::Duration::from_secs(60),
        enable_wal: true,
        enable_foreign_keys: true,
        cache_size: -1000,
        synchronous: "NORMAL".to_string(),
        journal_mode: "WAL".to_string(),
    };

    // Initialize database
    initialize_database(Some(config)).await?;

    // Verify database is working
    let db = get_database().await?;
    let health = db.health_check().await?;

    assert!(health.healthy);
    assert!(health.wal_mode_active);
    assert!(health.foreign_keys_enabled);

    // Test basic operations
    let stats = db.get_statistics().await?;
    assert_eq!(stats.conversation_count, 0); // Fresh database

    // Test vacuum operation
    db.vacuum().await?;

    // Test optimization
    db.optimize().await?;

    Ok(())
}

/// Test database connection pooling under load
#[tokio::test]
async fn test_connection_pooling_stress() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("pool_test.db");

    let config = DatabaseConfig {
        path: db_path,
        max_connections: 10,
        connection_timeout: std::time::Duration::from_secs(1),
        idle_timeout: std::time::Duration::from_secs(10),
        enable_wal: true,
        enable_foreign_keys: true,
        cache_size: -500,
        synchronous: "NORMAL".to_string(),
        journal_mode: "WAL".to_string(),
    };

    initialize_database(Some(config)).await?;
    let db = get_database().await?;

    // Create a test table
    let conn = db.get_connection()?;
    conn.execute(
        "CREATE TABLE test_table (id INTEGER PRIMARY KEY, value TEXT)",
        [],
    )?;
    drop(conn);

    // Simulate concurrent operations
    let mut handles = vec![];

    for i in 0..20 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            // Each task inserts a value and then reads it
            let conn = db_clone.get_connection()?;
            let value = format!("test_value_{}", i);

            conn.execute(
                "INSERT INTO test_table (id, value) VALUES (?1, ?2)",
                params![i, &value],
            )?;

            let result: String = conn.query_row(
                "SELECT value FROM test_table WHERE id = ?1",
                params![i],
                |row| row.get(0),
            )?;

            Ok::<_, anyhow::Error>(result == value)
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    for result in results {
        assert!(result??); // Unwrap JoinResult and then our Result<bool>
    }

    // Verify all records were inserted
    let conn = db.get_connection()?;
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM test_table", [], |row| row.get(0))?;

    assert_eq!(count, 20);

    Ok(())
}

/// Test database error handling and recovery
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("error_test.db");

    let config = DatabaseConfig {
        path: db_path,
        max_connections: 1,
        connection_timeout: std::time::Duration::from_millis(100),
        idle_timeout: std::time::Duration::from_secs(5),
        enable_wal: true,
        enable_foreign_keys: true,
        cache_size: -100,
        synchronous: "NORMAL".to_string(),
        journal_mode: "WAL".to_string(),
    };

    initialize_database(Some(config)).await?;
    let db = get_database().await?;

    // Test invalid SQL
    let conn = db.get_connection()?;
    let result = conn.execute("INVALID SQL STATEMENT", []);
    assert!(result.is_err());
    drop(conn);

    // Test that database is still functional after error
    let health = db.health_check().await?;
    assert!(health.healthy);

    // Test foreign key constraint violation (if enabled)
    let conn = db.get_connection()?;
    conn.execute("CREATE TABLE parent (id INTEGER PRIMARY KEY)", [])?;
    conn.execute(
        "CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER, FOREIGN KEY(parent_id) REFERENCES parent(id))",
        [],
    )?;

    // This should fail due to foreign key constraint
    let result = conn.execute("INSERT INTO child (id, parent_id) VALUES (1, 999)", []);
    assert!(result.is_err());

    // Database should still be healthy
    let health = db.health_check().await?;
    assert!(health.healthy);

    Ok(())
}

/// Test comprehensive model operations
#[tokio::test]
async fn test_model_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("models_test.db");

    let config = DatabaseConfig {
        path: db_path,
        ..Default::default()
    };

    initialize_database(Some(config)).await?;

    // Test User operations
    let user = User::create(
        Some("test@example.com".to_string()),
        Some("license-123".to_string()),
    )
    .await?;
    assert!(!user.id.is_empty());

    let found_user = User::find_by_id(&user.id).await?;
    assert!(found_user.is_some());
    assert_eq!(
        found_user.unwrap().email,
        Some("test@example.com".to_string())
    );

    // Test Conversation operations
    let mut conversation = Conversation::create(Some(user.id.clone()), None).await?;
    assert!(!conversation.id.is_empty());

    // Update metrics
    conversation.update_metrics(0.123, 500, 750).await?;
    assert_eq!(conversation.total_cost, 0.123);
    assert_eq!(conversation.input_tokens, 500);
    assert_eq!(conversation.output_tokens, 750);

    // Complete conversation
    conversation.complete().await?;
    assert!(conversation.end_time.is_some());

    // Test Message operations
    let msg1 = Message::create(
        conversation.id.clone(),
        "user".to_string(),
        "What is Rust?".to_string(),
        None,
        None,
    )
    .await?;

    let msg2 = Message::create(
        conversation.id.clone(),
        "assistant".to_string(),
        "Rust is a systems programming language.".to_string(),
        Some("generator".to_string()),
        Some("claude-3-opus".to_string()),
    )
    .await?;

    let messages = Message::find_by_conversation(&conversation.id).await?;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "What is Rust?");
    assert_eq!(
        messages[1].content,
        "Rust is a systems programming language."
    );

    // Test Knowledge operations
    let knowledge = KnowledgeConversation::create(
        Some(conversation.id.clone()),
        "What is Rust?".to_string(),
        "Rust is a modern systems programming language focused on safety and performance."
            .to_string(),
        "Rust documentation and common knowledge".to_string(),
    )
    .await?;

    let search_results = KnowledgeConversation::search("Rust", 10).await?;
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].id, knowledge.id);

    // Test Consensus Profile operations
    let profile = ConsensusProfile::create(
        "Test Profile".to_string(),
        "claude-3-opus-20240229".to_string(),
        "gpt-4-turbo".to_string(),
        "claude-3-sonnet-20240229".to_string(),
        "gpt-4".to_string(),
    )
    .await?;

    let found_profile = ConsensusProfile::find_by_name("Test Profile").await?;
    assert!(found_profile.is_some());
    assert_eq!(found_profile.unwrap().id, profile.id);

    let all_profiles = ConsensusProfile::list_all().await?;
    assert_eq!(all_profiles.len(), 1);

    // Test Activity Log operations
    ActivityLog::log(
        "query_start".to_string(),
        Some(conversation.id.clone()),
        Some(user.id.clone()),
        Some("generator".to_string()),
        Some(serde_json::json!({
            "query": "What is Rust?",
            "profile": "Test Profile"
        })),
    )
    .await?;

    ActivityLog::log(
        "query_complete".to_string(),
        Some(conversation.id.clone()),
        Some(user.id.clone()),
        Some("curator".to_string()),
        None,
    )
    .await?;

    let activities = ActivityLog::get_recent(10).await?;
    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].event_type, "query_complete");
    assert_eq!(activities[1].event_type, "query_start");

    Ok(())
}

/// Test transaction behavior
#[tokio::test]
async fn test_transactions() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("tx_test.db");

    let config = DatabaseConfig {
        path: db_path,
        ..Default::default()
    };

    initialize_database(Some(config)).await?;

    // Create test users
    let user1 = User::create(Some("user1@test.com".to_string()), None).await?;
    let user2 = User::create(Some("user2@test.com".to_string()), None).await?;

    // Test successful transaction
    let result = execute_transaction(|tx| {
        tx.execute(
            "UPDATE users SET tier = ?1 WHERE id = ?2",
            params!["PREMIUM", &user1.id],
        )?;
        tx.execute(
            "UPDATE users SET tier = ?1 WHERE id = ?2",
            params!["ENTERPRISE", &user2.id],
        )?;
        Ok(())
    })
    .await;
    assert!(result.is_ok());

    // Verify changes were committed
    let updated_user1 = User::find_by_id(&user1.id).await?.unwrap();
    let updated_user2 = User::find_by_id(&user2.id).await?.unwrap();
    assert_eq!(updated_user1.tier, "PREMIUM");
    assert_eq!(updated_user2.tier, "ENTERPRISE");

    // Test failed transaction (rollback)
    let result = execute_transaction(|tx| {
        tx.execute(
            "UPDATE users SET tier = ?1 WHERE id = ?2",
            params!["FREE", &user1.id],
        )?;
        // Force an error
        anyhow::bail!("Simulated transaction failure");
    })
    .await;
    assert!(result.is_err());

    // Verify rollback - tier should still be PREMIUM
    let user1_after_rollback = User::find_by_id(&user1.id).await?.unwrap();
    assert_eq!(user1_after_rollback.tier, "PREMIUM");

    Ok(())
}

/// Test database migration system
#[tokio::test]
async fn test_migration_system() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("migration_test.db");

    let config = DatabaseConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    // Initialize database (migrations will run automatically)
    initialize_database(Some(config)).await?;
    let db = get_database().await?;

    // Check that migrations table exists
    let conn = db.get_connection()?;
    let table_exists: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(table_exists, 1);

    // Check that migrations were applied
    let migration_count: i32 =
        conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })?;
    assert!(
        migration_count > 0,
        "At least one migration should be applied"
    );

    // Verify all expected tables exist
    let expected_tables = vec![
        "users",
        "configurations",
        "openrouter_providers",
        "openrouter_models",
        "pipeline_profiles",
        "consensus_profiles",
        "conversations",
        "messages",
        "usage_records",
        "budget_limits",
        "sync_metadata",
        "settings",
        "consensus_settings",
        "conversation_usage",
        "knowledge_conversations",
        "curator_truths",
        "conversation_context",
        "conversation_topics",
        "conversation_keywords",
        "improvement_patterns",
        "conversation_threads",
        "pending_sync",
        "provider_performance",
        "model_rankings",
        "consensus_metrics",
        "cost_analytics",
        "feature_usage",
        "profile_templates",
        "model_selection_history",
        "performance_metrics",
        "activity_log",
    ];

    for table in expected_tables {
        let exists: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            params![table],
            |row| row.get(0),
        )?;
        assert_eq!(exists, 1, "Table {} should exist", table);
    }

    Ok(())
}

/// Test database performance metrics
#[tokio::test]
async fn test_performance_metrics() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("perf_test.db");

    let config = DatabaseConfig {
        path: db_path,
        ..Default::default()
    };

    initialize_database(Some(config)).await?;

    // Create test data
    let user = User::create(None, None).await?;

    // Measure conversation creation performance
    let start = std::time::Instant::now();
    let mut conversation_ids = Vec::new();

    for i in 0..100 {
        let conv = Conversation::create(Some(user.id.clone()), None).await?;
        conversation_ids.push(conv.id);

        // Add some messages
        for j in 0..5 {
            Message::create(
                conv.id.clone(),
                if j % 2 == 0 { "user" } else { "assistant" }.to_string(),
                format!("Message {} in conversation {}", j, i),
                None,
                None,
            )
            .await?;
        }
    }

    let creation_time = start.elapsed();
    println!(
        "Created 100 conversations with 500 messages in {:?}",
        creation_time
    );

    // Measure query performance
    let start = std::time::Instant::now();
    for id in &conversation_ids[..10] {
        let _ = Message::find_by_conversation(id).await?;
    }
    let query_time = start.elapsed();
    println!("Queried 10 conversations in {:?}", query_time);

    // Ensure performance meets targets
    assert!(creation_time.as_millis() < 5000, "Creation should be fast");
    assert!(query_time.as_millis() < 100, "Queries should be fast");

    Ok(())
}

/// Test concurrent write operations
#[tokio::test]
async fn test_concurrent_writes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("concurrent_test.db");

    let config = DatabaseConfig {
        path: db_path,
        max_connections: 20,
        ..Default::default()
    };

    initialize_database(Some(config)).await?;

    // Create multiple users concurrently
    let mut handles = vec![];

    for i in 0..50 {
        let handle =
            tokio::spawn(
                async move { User::create(Some(format!("user{}@test.com", i)), None).await },
            );
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    // All operations should succeed
    for result in results {
        assert!(result?.is_ok());
    }

    // Verify all users were created
    let db = get_database().await?;
    let conn = db.get_connection()?;
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    assert_eq!(count, 50);

    Ok(())
}

/// Test database backup and restore functionality
#[tokio::test]
async fn test_backup_restore() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("backup_test.db");
    let backup_path = temp_dir.path().join("backup.db");

    let config = DatabaseConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    initialize_database(Some(config)).await?;

    // Create test data
    let user = User::create(Some("backup@test.com".to_string()), None).await?;
    let profile = ConsensusProfile::create(
        "Backup Profile".to_string(),
        "model1".to_string(),
        "model2".to_string(),
        "model3".to_string(),
        "model4".to_string(),
    )
    .await?;

    // Perform backup
    let db = get_database().await?;
    let conn = db.get_connection()?;

    // Use SQLite backup API
    let backup_conn = rusqlite::Connection::open(&backup_path)?;
    let backup = rusqlite::backup::Backup::new(&conn, &backup_conn)?;
    backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;
    drop(backup_conn);

    // Verify backup contains data
    let verify_conn = rusqlite::Connection::open(&backup_path)?;
    let user_count: i32 =
        verify_conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    assert_eq!(user_count, 1);

    let profile_count: i32 =
        verify_conn.query_row("SELECT COUNT(*) FROM consensus_profiles", [], |row| {
            row.get(0)
        })?;
    assert_eq!(profile_count, 1);

    Ok(())
}
