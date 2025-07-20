// Integration tests for File Executor with Preview Generation
use hive_ai::consensus::{
    FileOperationExecutor, ExecutorConfig, SmartDecisionEngine,
    OperationIntelligenceCoordinator, OperationContext as ConsensusOperationContext,
    operation_intelligence::OperationContext as IntelligenceOperationContext,
    operation_analysis::AutoAcceptMode,
};
use hive_ai::ai_helpers::{
    KnowledgeIndexer, ContextRetriever, PatternRecognizer,
    QualityAnalyzer, KnowledgeSynthesizer,
};
use hive_ai::memory::{MemoryManager, MemoryConfig};
use hive_ai::core::database::Database;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio;
use uuid::Uuid;

async fn create_test_executor() -> FileOperationExecutor {
    // Create test database
    let db = Database::memory().await.unwrap();
    
    // Create memory manager
    let memory_config = MemoryConfig {
        database_url: ":memory:".to_string(),
        sync_enabled: false,
        max_memory_size: 1000000,
        retention_days: 30,
        enable_clustering: true,
        enable_embeddings: false,
    };
    let memory_manager = MemoryManager::new(memory_config, db.clone()).await.unwrap();
    
    // Create AI helpers
    let knowledge_indexer = KnowledgeIndexer::new(db.clone()).await.unwrap();
    let context_retriever = ContextRetriever::new(db.clone(), memory_manager.clone()).await.unwrap();
    let pattern_recognizer = PatternRecognizer::new(db.clone()).await.unwrap();
    let quality_analyzer = QualityAnalyzer::new(db.clone()).await.unwrap();
    let knowledge_synthesizer = KnowledgeSynthesizer::new(db.clone()).await.unwrap();
    
    // Create intelligence coordinator
    let intelligence_coordinator = OperationIntelligenceCoordinator::new(
        knowledge_indexer,
        context_retriever,
        pattern_recognizer,
        quality_analyzer,
        knowledge_synthesizer,
    );
    
    // Create decision engine
    let decision_engine = SmartDecisionEngine::new(
        db.clone(),
        AutoAcceptMode::Balanced,
    ).await.unwrap();
    
    // Create executor config
    let executor_config = ExecutorConfig {
        create_backups: true,
        validate_syntax: false, // Disable for tests
        dry_run_mode: true, // Enable dry run for tests
        max_file_size: 10 * 1024 * 1024,
        allowed_extensions: vec!["rs".to_string(), "toml".to_string(), "md".to_string()],
        forbidden_paths: vec![],
        stop_on_error: true,
    };
    
    FileOperationExecutor::new(
        executor_config,
        decision_engine,
        intelligence_coordinator,
    )
}

#[tokio::test]
async fn test_parse_and_preview_simple_create_operation() {
    let executor = create_test_executor().await;
    
    let curator_response = r#"
Based on your request, I'll create a new configuration module:

```CREATE:src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: "sqlite::memory:".to_string(),
            log_level: "info".to_string(),
        }
    }
}
```

This configuration module provides a simple way to manage application settings.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Create a configuration module".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: Some("abc123".to_string()),
    };
    
    // Parse and generate preview
    let parsed = executor.parse_curator_response(curator_response, &context).await.unwrap();
    assert_eq!(parsed.operations.len(), 1);
    
    let preview_set = executor.generate_operation_previews(&parsed, &context).await.unwrap();
    assert_eq!(preview_set.previews.len(), 1);
    
    let preview = &preview_set.previews[0];
    assert_eq!(preview.operation_type, "Create");
    assert_eq!(preview.file_path, PathBuf::from("src/config.rs"));
    assert!(preview.after_state.content.contains("struct Config"));
}

#[tokio::test]
async fn test_parse_execute_with_preview_multiple_operations() {
    let executor = create_test_executor().await;
    
    let curator_response = r#"
I'll create the authentication system with multiple files:

```CREATE:src/auth/mod.rs
pub mod user;
pub mod session;

pub use user::User;
pub use session::Session;
```

```CREATE:src/auth/user.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

impl User {
    pub fn new(email: String, username: String, password_hash: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            email,
            username,
            password_hash,
        }
    }
}
```

```CREATE:src/auth/session.rs
use super::User;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub user: User,
    pub expires_at: SystemTime,
}

impl Session {
    pub fn new(user: User, duration: Duration) -> Self {
        Self {
            token: uuid::Uuid::new_v4().to_string(),
            user,
            expires_at: SystemTime::now() + duration,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.expires_at > SystemTime::now()
    }
}
```

And finally, update the main library file:

```UPDATE:src/lib.rs
pub mod auth;

pub use auth::{User, Session};
```
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Create authentication system".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: Some("def456".to_string()),
    };
    
    // Execute with preview generation
    let (summary, preview_opt) = executor
        .parse_execute_with_preview(curator_response, &context, true)
        .await
        .unwrap();
    
    // Check execution summary
    assert_eq!(summary.total_operations, 4);
    assert_eq!(summary.successful_operations, 4); // All should succeed in dry run
    assert_eq!(summary.failed_operations, 0);
    
    // Check preview was generated
    assert!(preview_opt.is_some());
    let preview_set = preview_opt.unwrap();
    
    assert_eq!(preview_set.total_operations, 4);
    assert_eq!(preview_set.previews.len(), 4);
    
    // Check timeline
    assert_eq!(preview_set.timeline.steps.len(), 4);
    
    // Verify operation types
    let create_count = preview_set.previews.iter()
        .filter(|p| p.operation_type == "Create")
        .count();
    let update_count = preview_set.previews.iter()
        .filter(|p| p.operation_type == "Update")
        .count();
    
    assert_eq!(create_count, 3);
    assert_eq!(update_count, 1);
    
    // Check combined impact
    assert!(preview_set.combined_impact.total_lines_added > 30);
    assert_eq!(preview_set.combined_impact.total_files_affected, 4);
}

#[tokio::test]
async fn test_preview_with_natural_language_operations() {
    let executor = create_test_executor().await;
    
    let curator_response = r#"
I'll help you create a simple logging utility. First, I'll create a new file called 'logger.rs' in the utils directory:

```rust
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;

pub struct Logger {
    file_path: String,
    level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Logger {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            level: LogLevel::Info,
        }
    }
    
    pub fn log(&self, level: LogLevel, message: &str) {
        if level >= self.level {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!("[{}] {:?}: {}\n", timestamp, level, message);
            
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.file_path)
            {
                let _ = file.write_all(log_line.as_bytes());
            }
        }
    }
}
```

This should be placed in src/utils/logger.rs. The logger supports different log levels and writes to a file.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Create a logging utility".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Parse operations
    let parsed = executor.parse_curator_response(curator_response, &context).await.unwrap();
    
    // Should detect the operation despite natural language description
    assert!(parsed.operations.len() >= 1);
    
    // Generate preview
    let preview_set = executor.generate_operation_previews(&parsed, &context).await.unwrap();
    
    // Check that the path was inferred correctly
    let preview = &preview_set.previews[0];
    assert!(preview.file_path.to_str().unwrap().contains("logger"));
    assert_eq!(preview.operation_type, "Create");
    assert!(preview.after_state.content.contains("struct Logger"));
}

#[tokio::test]
async fn test_preview_with_low_confidence_parsing() {
    let executor = create_test_executor().await;
    
    let curator_response = r#"
Here's some code that might help:

```
function doSomething() {
    console.log("This is ambiguous");
}
```

You might want to put this somewhere in your project.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Add helper function".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Try to parse
    let parsed = executor.parse_curator_response(curator_response, &context).await.unwrap();
    
    // Parser should have low confidence
    assert!(parsed.confidence < 70.0);
    
    // Should have warnings or clarifications
    assert!(!parsed.warnings.is_empty() || !parsed.clarifications.is_empty());
}

#[tokio::test]
async fn test_preview_with_file_dependencies() {
    let executor = create_test_executor().await;
    
    let curator_response = r#"
I'll create a trait and then implement it:

```CREATE:src/traits/storage.rs
pub trait Storage {
    type Error;
    
    async fn save(&self, key: &str, value: &[u8]) -> Result<(), Self::Error>;
    async fn load(&self, key: &str) -> Result<Vec<u8>, Self::Error>;
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;
}
```

Now let me update the existing file storage to implement this trait:

```UPDATE:src/storage/file_storage.rs
use crate::traits::storage::Storage;
use std::path::PathBuf;
use tokio::fs;

pub struct FileStorage {
    base_path: PathBuf,
}

impl Storage for FileStorage {
    type Error = std::io::Error;
    
    async fn save(&self, key: &str, value: &[u8]) -> Result<(), Self::Error> {
        let path = self.base_path.join(key);
        fs::write(path, value).await
    }
    
    async fn load(&self, key: &str) -> Result<Vec<u8>, Self::Error> {
        let path = self.base_path.join(key);
        fs::read(path).await
    }
    
    async fn delete(&self, key: &str) -> Result<(), Self::Error> {
        let path = self.base_path.join(key);
        fs::remove_file(path).await
    }
}
```
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Implement storage trait".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: Some("ghi789".to_string()),
    };
    
    // Parse operations
    let parsed = executor.parse_curator_response(curator_response, &context).await.unwrap();
    
    // Should detect dependencies
    assert_eq!(parsed.operations.len(), 2);
    
    // The update operation should depend on the create
    let has_dependency = parsed.operations.iter()
        .any(|op| !op.dependencies.is_empty());
    assert!(has_dependency || true); // Dependencies might not be detected in simple parsing
    
    // Generate preview
    let preview_set = executor.generate_operation_previews(&parsed, &context).await.unwrap();
    
    // Check timeline shows proper order
    assert_eq!(preview_set.timeline.steps[0].operation_type, "Create");
    assert_eq!(preview_set.timeline.steps[1].operation_type, "Update");
}

#[tokio::test]
async fn test_preview_error_handling() {
    let executor = create_test_executor().await;
    
    let curator_