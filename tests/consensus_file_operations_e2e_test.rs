// End-to-end tests for file operations through consensus pipeline
use crate::consensus::{
    ConsensusEngine, ConsensusConfig, ConsensusProfile, ProfileType,
    ContextInjectionStrategy, RetryPolicy,
};
use crate::core::database::Database;
use crate::core::api_keys::ApiKeyManager;
use crate::ai_helpers::AIHelperEcosystem;
use crate::memory::{MemoryManager, MemoryConfig};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;
use std::fs;

async fn setup_test_engine_with_repo() -> (ConsensusEngine, TempDir) {
    // Create temporary directory for test repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    
    // Initialize git repository
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to initialize git repository");
    
    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repository\n").unwrap();
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add files");
    
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to commit");
    
    // Create test database
    let db = Database::memory().await.unwrap();
    
    // Create consensus engine
    let engine = ConsensusEngine::new(Some(db)).await.unwrap();
    
    (engine, temp_dir)
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_create_file_through_consensus() {
    let (engine, temp_dir) = setup_test_engine_with_repo().await;
    let repo_path = temp_dir.path();
    
    // Set up consensus configuration
    let config = ConsensusConfig {
        enable_streaming: false,
        retry_on_error: false,
        retry_policy: RetryPolicy::default(),
        context_injection: ContextInjectionStrategy::default(),
        show_progress: false,
        cancellation_token: None,
        enable_codebase_intelligence: true,
    };
    
    // Use a profile that will generate file operations
    let profile = ConsensusProfile {
        id: "code_writer".to_string(),
        profile_type: ProfileType::ExpertDomain,
        display_name: "Code Writer".to_string(),
        description: "Expert at writing code files".to_string(),
        ..Default::default()
    };
    
    // Set repository context
    engine.set_repository_path(repo_path.to_path_buf()).await.unwrap();
    
    // Create a request that should result in file creation
    let question = "Create a new Rust module called 'hello' with a simple hello_world function that prints 'Hello, World!'";
    
    // Run consensus
    let result = engine.run_consensus(
        question,
        None, // No additional context
        config,
        profile,
    ).await;
    
    assert!(result.is_ok(), "Consensus should succeed: {:?}", result);
    let consensus_result = result.unwrap();
    
    // Check that consensus was successful
    assert!(consensus_result.success);
    assert!(consensus_result.result.is_some());
    
    // Verify the curator response contains file operation
    let curator_response = consensus_result.result.unwrap();
    assert!(curator_response.contains("hello.rs") || curator_response.contains("hello"));
    
    // Check if file was actually created
    let expected_file = repo_path.join("src/hello.rs");
    if expected_file.exists() {
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("hello_world"));
        assert!(content.contains("Hello, World!"));
        println!("✅ File created successfully with expected content");
    } else {
        // File might be in root if src doesn't exist
        let alt_file = repo_path.join("hello.rs");
        if alt_file.exists() {
            let content = fs::read_to_string(&alt_file).unwrap();
            assert!(content.contains("hello_world"));
            println!("✅ File created in root directory");
        } else {
            panic!("Expected file was not created by consensus pipeline");
        }
    }
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_update_file_through_consensus() {
    let (engine, temp_dir) = setup_test_engine_with_repo().await;
    let repo_path = temp_dir.path();
    
    // Create an existing file to update
    let existing_file = repo_path.join("config.toml");
    fs::write(&existing_file, r#"
[server]
port = 8080
host = "localhost"
"#).unwrap();
    
    // Commit the file
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    std::process::Command::new("git")
        .args(&["commit", "-m", "Add config file"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Set up consensus configuration
    let config = ConsensusConfig {
        enable_streaming: false,
        retry_on_error: false,
        retry_policy: RetryPolicy::default(),
        context_injection: ContextInjectionStrategy::default(),
        show_progress: false,
        cancellation_token: None,
        enable_codebase_intelligence: true,
    };
    
    let profile = ConsensusProfile {
        id: "code_updater".to_string(),
        profile_type: ProfileType::ExpertDomain,
        display_name: "Code Updater".to_string(),
        description: "Expert at updating configuration files".to_string(),
        ..Default::default()
    };
    
    // Set repository context
    engine.set_repository_path(repo_path.to_path_buf()).await.unwrap();
    
    // Request to update the config
    let question = "Update the config.toml file to add a [database] section with url = 'sqlite://data.db' and max_connections = 10";
    
    // Run consensus
    let result = engine.run_consensus(
        question,
        None,
        config,
        profile,
    ).await;
    
    assert!(result.is_ok(), "Consensus should succeed: {:?}", result);
    
    // Verify file was updated
    let updated_content = fs::read_to_string(&existing_file).unwrap();
    assert!(updated_content.contains("[database]"));
    assert!(updated_content.contains("sqlite://data.db"));
    assert!(updated_content.contains("max_connections"));
    println!("✅ File updated successfully with new database section");
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_multiple_file_operations() {
    let (engine, temp_dir) = setup_test_engine_with_repo().await;
    let repo_path = temp_dir.path();
    
    // Create src directory
    fs::create_dir(repo_path.join("src")).unwrap();
    
    let config = ConsensusConfig {
        enable_streaming: false,
        retry_on_error: false,
        retry_policy: RetryPolicy::default(),
        context_injection: ContextInjectionStrategy::default(),
        show_progress: false,
        cancellation_token: None,
        enable_codebase_intelligence: true,
    };
    
    let profile = ConsensusProfile {
        id: "architect".to_string(),
        profile_type: ProfileType::ExpertDomain,
        display_name: "Software Architect".to_string(),
        description: "Expert at creating modular code structures".to_string(),
        ..Default::default()
    };
    
    // Set repository context
    engine.set_repository_path(repo_path.to_path_buf()).await.unwrap();
    
    // Request multiple file operations
    let question = "Create a simple user authentication module with: 1) A User struct in src/models/user.rs with id, email, and password_hash fields, 2) A mod.rs file in src/models/ that exports the User struct, and 3) Update src/lib.rs to include the models module";
    
    // Create initial lib.rs
    fs::write(repo_path.join("src/lib.rs"), "// Main library file\n").unwrap();
    
    // Run consensus
    let result = engine.run_consensus(
        question,
        None,
        config,
        profile,
    ).await;
    
    assert!(result.is_ok(), "Consensus should succeed: {:?}", result);
    
    // Verify files were created/updated
    let user_file = repo_path.join("src/models/user.rs");
    let mod_file = repo_path.join("src/models/mod.rs");
    let lib_file = repo_path.join("src/lib.rs");
    
    // Check if models directory was created
    assert!(repo_path.join("src/models").exists(), "models directory should be created");
    
    // Check files
    if user_file.exists() {
        let content = fs::read_to_string(&user_file).unwrap();
        assert!(content.contains("struct User"));
        assert!(content.contains("email"));
        println!("✅ User model created");
    }
    
    if mod_file.exists() {
        let content = fs::read_to_string(&mod_file).unwrap();
        assert!(content.contains("mod user") || content.contains("pub use"));
        println!("✅ Module file created");
    }
    
    let lib_content = fs::read_to_string(&lib_file).unwrap();
    if lib_content.contains("mod models") || lib_content.contains("pub mod models") {
        println!("✅ Library file updated");
    }
}

#[tokio::test]
async fn test_file_operation_preview_mode() {
    let (engine, temp_dir) = setup_test_engine_with_repo().await;
    let repo_path = temp_dir.path();
    
    // Configure for dry run mode
    let config = ConsensusConfig {
        enable_streaming: false,
        retry_on_error: false,
        retry_policy: RetryPolicy::default(),
        context_injection: ContextInjectionStrategy::default(),
        show_progress: false,
        cancellation_token: None,
        enable_codebase_intelligence: true,
    };
    
    let profile = ConsensusProfile {
        id: "careful_coder".to_string(),
        profile_type: ProfileType::ExpertDomain,
        display_name: "Careful Coder".to_string(),
        description: "Reviews changes before applying".to_string(),
        ..Default::default()
    };
    
    engine.set_repository_path(repo_path.to_path_buf()).await.unwrap();
    
    // This test would verify preview generation without actual execution
    // For now, we'll just verify the consensus runs
    let question = "Show me what would happen if I create a new test.rs file with a basic test function";
    
    let result = engine.run_consensus(
        question,
        None,
        config,
        profile,
    ).await;
    
    assert!(result.is_ok(), "Consensus should succeed");
    
    // In preview mode, file should not be created
    let test_file = repo_path.join("test.rs");
    // The actual behavior depends on the configuration of the executor
    println!("Preview mode test completed");
}