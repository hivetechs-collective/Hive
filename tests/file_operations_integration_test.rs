// Integration tests for file operations without requiring API keys
use hive_ai::consensus::{
    FileOperationExecutor, ExecutorConfig, SmartDecisionEngine,
    OperationIntelligenceCoordinator, operation_analysis::{
        AutoAcceptMode, OperationContext as ConsensusOperationContext,
        OperationAnalysis, UnifiedScore, OperationGroups,
        ComponentScores, ScoringFactors,
    },
    smart_decision_engine::UserPreferences,
    stages::file_aware_curator::FileOperation,
};
use hive_ai::ai_helpers::{
    KnowledgeIndexer, ContextRetriever, PatternRecognizer,
    QualityAnalyzer, KnowledgeSynthesizer,
};
use hive_ai::memory::{MemoryManager, MemoryConfig};
use hive_ai::core::database::Database;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;
use tokio;
use std::fs;
use uuid::Uuid;

async fn create_test_executor_with_tempdir() -> (FileOperationExecutor, TempDir) {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    
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
    let user_prefs = UserPreferences {
        confidence_threshold: 70.0,
        risk_tolerance: 0.4,
        require_confirmation_for_deletions: false, // Allow deletions in tests
        require_confirmation_for_mass_updates: false,
        trusted_paths: vec![temp_dir.path().to_path_buf()],
        blocked_paths: vec![],
        custom_rules: vec![],
    };
    
    let decision_engine = SmartDecisionEngine::new(
        AutoAcceptMode::Balanced,
        user_prefs,
        None,
    );
    
    // Create executor config for real execution
    let executor_config = ExecutorConfig {
        create_backups: true,
        validate_syntax: false,
        dry_run_mode: false, // Real execution
        max_file_size: 10 * 1024 * 1024,
        allowed_extensions: vec![
            "rs", "js", "ts", "py", "txt", "md", "toml", "json"
        ].into_iter().map(String::from).collect(),
        forbidden_paths: vec![], // No forbidden paths for tests
        stop_on_error: true,
    };
    
    let executor = FileOperationExecutor::new(
        executor_config,
        decision_engine,
        intelligence_coordinator,
    );
    
    (executor, temp_dir)
}

#[tokio::test]
async fn test_parse_and_execute_create_operation() {
    let (executor, temp_dir) = create_test_executor_with_tempdir().await;
    let repo_path = temp_dir.path();
    
    let curator_response = r#"
I'll create a simple hello world program for you:

```CREATE:hello.rs
fn main() {
    println!("Hello, World!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert_eq!(2 + 2, 4);
    }
}
```

This creates a basic Rust program with a test.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: repo_path.to_path_buf(),
        user_question: "Create a hello world program".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Execute operations
    let result = executor.parse_and_execute_curator_response(
        curator_response,
        &context,
    ).await;
    
    assert!(result.is_ok(), "Execution should succeed: {:?}", result);
    let summary = result.unwrap();
    
    // Verify execution summary
    assert_eq!(summary.total_operations, 1);
    assert_eq!(summary.successful_operations, 1);
    assert_eq!(summary.failed_operations, 0);
    
    // Verify file was created
    let created_file = repo_path.join("hello.rs");
    assert!(created_file.exists(), "File should be created");
    
    let content = fs::read_to_string(created_file).unwrap();
    assert!(content.contains("Hello, World!"));
    assert!(content.contains("fn main()"));
    assert!(content.contains("#[test]"));
}

#[tokio::test]
async fn test_parse_and_execute_update_operation() {
    let (executor, temp_dir) = create_test_executor_with_tempdir().await;
    let repo_path = temp_dir.path();
    
    // Create initial file
    let file_path = repo_path.join("config.toml");
    fs::write(&file_path, "[server]\nport = 8080\n").unwrap();
    
    let curator_response = r#"
I'll update the configuration file to add database settings:

```UPDATE:config.toml
[server]
port = 8080
host = "0.0.0.0"

[database]
url = "sqlite://app.db"
max_connections = 10

[logging]
level = "info"
```

This adds database configuration and logging settings.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: repo_path.to_path_buf(),
        user_question: "Add database config".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Execute operations
    let result = executor.parse_and_execute_curator_response(
        curator_response,
        &context,
    ).await;
    
    assert!(result.is_ok(), "Execution should succeed: {:?}", result);
    let summary = result.unwrap();
    
    assert_eq!(summary.successful_operations, 1);
    
    // Verify file was updated
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("[database]"));
    assert!(content.contains("sqlite://app.db"));
    assert!(content.contains("[logging]"));
}

#[tokio::test]
async fn test_parse_and_execute_multiple_operations() {
    let (executor, temp_dir) = create_test_executor_with_tempdir().await;
    let repo_path = temp_dir.path();
    
    // Create src directory
    fs::create_dir(repo_path.join("src")).unwrap();
    
    let curator_response = r#"
I'll create a module structure for you:

```CREATE:src/lib.rs
pub mod models;
pub mod utils;

pub use models::User;
```

```CREATE:src/models/mod.rs
pub mod user;

pub use user::User;
```

```CREATE:src/models/user.rs
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}
```

```CREATE:src/utils/mod.rs
pub mod helpers;
```

This creates a basic module structure with a User model.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: repo_path.to_path_buf(),
        user_question: "Create module structure".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Execute operations
    let result = executor.parse_and_execute_curator_response(
        curator_response,
        &context,
    ).await;
    
    assert!(result.is_ok(), "Execution should succeed: {:?}", result);
    let summary = result.unwrap();
    
    assert_eq!(summary.total_operations, 4);
    assert_eq!(summary.successful_operations, 4);
    
    // Verify files were created
    assert!(repo_path.join("src/lib.rs").exists());
    assert!(repo_path.join("src/models/mod.rs").exists());
    assert!(repo_path.join("src/models/user.rs").exists());
    assert!(repo_path.join("src/utils/mod.rs").exists());
    
    // Verify content
    let user_content = fs::read_to_string(repo_path.join("src/models/user.rs")).unwrap();
    assert!(user_content.contains("struct User"));
    assert!(user_content.contains("impl User"));
}

#[tokio::test]
async fn test_parse_and_execute_delete_operation() {
    let (executor, temp_dir) = create_test_executor_with_tempdir().await;
    let repo_path = temp_dir.path();
    
    // Create file to delete
    let file_path = repo_path.join("old_file.txt");
    fs::write(&file_path, "This file will be deleted").unwrap();
    
    let curator_response = r#"
I'll remove the old file as requested:

```DELETE:old_file.txt
```

And create a new one to replace it:

```CREATE:new_file.txt
This is the new content that replaces the old file.
```

The old file has been removed and replaced with the new one.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: repo_path.to_path_buf(),
        user_question: "Delete old file and create new one".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Execute operations
    let result = executor.parse_and_execute_curator_response(
        curator_response,
        &context,
    ).await;
    
    assert!(result.is_ok(), "Execution should succeed: {:?}", result);
    let summary = result.unwrap();
    
    assert_eq!(summary.total_operations, 2);
    assert_eq!(summary.successful_operations, 2);
    
    // Verify old file was deleted
    assert!(!file_path.exists(), "Old file should be deleted");
    
    // Verify new file was created
    let new_file = repo_path.join("new_file.txt");
    assert!(new_file.exists(), "New file should be created");
    
    let content = fs::read_to_string(new_file).unwrap();
    assert!(content.contains("new content"));
}

#[tokio::test]
async fn test_natural_language_parsing() {
    let (executor, temp_dir) = create_test_executor_with_tempdir().await;
    let repo_path = temp_dir.path();
    
    let curator_response = r#"
I'll create a new file called 'utils.js' with some helper functions:

```javascript
// Utility functions for the application

export function formatDate(date) {
    return new Intl.DateTimeFormat('en-US').format(date);
}

export function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}
```

This file should be placed in your project directory and provides common utility functions.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: repo_path.to_path_buf(),
        user_question: "Create utility functions".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Execute operations
    let result = executor.parse_and_execute_curator_response(
        curator_response,
        &context,
    ).await;
    
    assert!(result.is_ok(), "Execution should succeed: {:?}", result);
    let summary = result.unwrap();
    
    // Should parse the natural language and create the file
    assert_eq!(summary.successful_operations, 1);
    
    // File should be created as utils.js
    let utils_file = repo_path.join("utils.js");
    assert!(utils_file.exists(), "Utils file should be created");
    
    let content = fs::read_to_string(utils_file).unwrap();
    assert!(content.contains("formatDate"));
    assert!(content.contains("debounce"));
}

#[tokio::test]
async fn test_operation_with_nested_directories() {
    let (executor, temp_dir) = create_test_executor_with_tempdir().await;
    let repo_path = temp_dir.path();
    
    let curator_response = r#"
I'll create the authentication module in the proper directory structure:

```CREATE:src/auth/handlers/login.rs
use crate::models::User;
use crate::auth::tokens::create_token;

pub async fn handle_login(email: &str, password: &str) -> Result<String, LoginError> {
    // Validate credentials
    let user = validate_user(email, password).await?;
    
    // Create JWT token
    let token = create_token(&user)?;
    
    Ok(token)
}

#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
    UserNotFound,
    TokenCreationFailed,
}
```

This creates a login handler in a nested directory structure.
    "#;
    
    let context = ConsensusOperationContext {
        repository_path: repo_path.to_path_buf(),
        user_question: "Create login handler".to_string(),
        consensus_response: curator_response.to_string(),
        timestamp: SystemTime::now(),
        session_id: Uuid::new_v4().to_string(),
        git_commit: None,
    };
    
    // Execute operations
    let result = executor.parse_and_execute_curator_response(
        curator_response,
        &context,
    ).await;
    
    assert!(result.is_ok(), "Execution should succeed: {:?}", result);
    let summary = result.unwrap();
    
    assert_eq!(summary.successful_operations, 1);
    
    // Verify nested directories were created
    assert!(repo_path.join("src/auth/handlers").exists(), "Nested directories should be created");
    
    // Verify file exists
    let file_path = repo_path.join("src/auth/handlers/login.rs");
    assert!(file_path.exists(), "File should be created in nested directory");
    
    let content = fs::read_to_string(file_path).unwrap();
    assert!(content.contains("handle_login"));
    assert!(content.contains("LoginError"));
}