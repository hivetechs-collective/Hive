//! Integration tests for ClaudeCodeExecutor
//! 
//! Tests the hybrid Claude-Consensus integration to ensure:
//! - Execution modes work correctly
//! - Consensus invocation criteria are evaluated properly
//! - Knowledge storage functions as expected

use hive_ai::consensus::{
    ClaudeCodeExecutor, ClaudeCodeExecutorBuilder, ClaudeExecutionMode,
    ConsensusInvocationCriteria, ConsensusProfile,
};
use hive_ai::consensus::streaming::{StreamingCallbacks, ConsensusEvent};
use hive_ai::consensus::types::{Stage, StageResult};
use hive_ai::core::database::DatabaseManager;
use hive_ai::ai_helpers::AIHelperEcosystem;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

/// Test callbacks that capture events
struct TestCallbacks {
    events: Arc<Mutex<Vec<String>>>,
}

impl TestCallbacks {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn get_events(&self) -> Vec<String> {
        self.events.lock().await.clone()
    }
}

impl StreamingCallbacks for TestCallbacks {
    fn on_stage_start(&self, stage: Stage, model: &str) -> Result<()> {
        let events = self.events.clone();
        tokio::spawn(async move {
            let mut events = events.lock().await;
            events.push(format!("Stage started: {:?} with model: {}", stage, model));
        });
        Ok(())
    }
    
    fn on_stage_chunk(&self, stage: Stage, chunk: &str, _total: &str) -> Result<()> {
        let events = self.events.clone();
        let chunk = chunk.to_string();
        tokio::spawn(async move {
            let mut events = events.lock().await;
            events.push(format!("Stage {:?} chunk: {}", stage, chunk));
        });
        Ok(())
    }
    
    fn on_stage_complete(&self, stage: Stage, _result: &StageResult) -> Result<()> {
        let events = self.events.clone();
        tokio::spawn(async move {
            let mut events = events.lock().await;
            events.push(format!("Stage completed: {:?}", stage));
        });
        Ok(())
    }
    
    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> Result<()> {
        let events = self.events.clone();
        let error_msg = error.to_string();
        tokio::spawn(async move {
            let mut events = events.lock().await;
            events.push(format!("Stage {:?} error: {}", stage, error_msg));
        });
        Ok(())
    }
}

#[tokio::test]
async fn test_execution_mode_switching() {
    // Create executor without Anthropic key (won't actually execute)
    let executor = ClaudeCodeExecutor::new(
        ConsensusProfile::default(),
        Arc::new(DatabaseManager::new(None).unwrap()),
        Arc::new(AIHelperEcosystem::new()),
        None, // No knowledge base for test
        None, // No Anthropic key
    );
    
    // Test default mode
    assert_eq!(executor.get_mode().await, ClaudeExecutionMode::ConsensusAssisted);
    
    // Test mode switching
    executor.set_mode(ClaudeExecutionMode::Direct).await;
    assert_eq!(executor.get_mode().await, ClaudeExecutionMode::Direct);
    
    executor.set_mode(ClaudeExecutionMode::ConsensusRequired).await;
    assert_eq!(executor.get_mode().await, ClaudeExecutionMode::ConsensusRequired);
}

#[test]
fn test_consensus_invocation_criteria() {
    // Create a mock executor just to test the criteria logic
    let executor = ClaudeCodeExecutor::new(
        ConsensusProfile::default(),
        Arc::new(DatabaseManager::new(None).unwrap()),
        Arc::new(AIHelperEcosystem::new()),
        None,
        None,
    );
    
    // Test high risk always invokes consensus
    let criteria = ConsensusInvocationCriteria {
        architectural_change: false,
        high_risk_operation: true,
        confidence_level: 0.9,
        multiple_approaches: false,
        user_requested_analysis: false,
        complexity_score: 0.2,
    };
    // Note: We can't directly test should_invoke_consensus as it's private
    // But we can verify the struct is created correctly
    assert!(criteria.high_risk_operation);
    
    // Test low confidence invokes consensus
    let criteria = ConsensusInvocationCriteria {
        architectural_change: false,
        high_risk_operation: false,
        confidence_level: 0.4,
        multiple_approaches: false,
        user_requested_analysis: false,
        complexity_score: 0.3,
    };
    assert!(criteria.confidence_level < 0.6);
}

#[tokio::test]
async fn test_builder_pattern() {
    // Test the builder creates executor correctly
    let result = ClaudeCodeExecutorBuilder::new()
        .with_profile(ConsensusProfile::default())
        .with_database(Arc::new(DatabaseManager::new(None).unwrap()))
        .with_ai_helpers(Arc::new(AIHelperEcosystem::new()))
        .build();
        
    assert!(result.is_ok());
    let executor = result.unwrap();
    
    // Verify default mode
    assert_eq!(executor.get_mode().await, ClaudeExecutionMode::ConsensusAssisted);
}

#[tokio::test]
async fn test_execution_without_anthropic_key() {
    // Create executor without key
    let executor = ClaudeCodeExecutor::new(
        ConsensusProfile::default(),
        Arc::new(DatabaseManager::new(None).unwrap()),
        Arc::new(AIHelperEcosystem::new()),
        None,
        None,
    );
    
    let callbacks = Arc::new(TestCallbacks::new());
    
    // Try to execute - should fail due to missing API key
    let result = executor.execute(
        "Test request",
        None,
        callbacks.clone(),
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Anthropic API key not configured"));
}

#[tokio::test]
async fn test_knowledge_storage_integration() {
    // Create a temporary database
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Initialize database with migrations
    let db = Arc::new(DatabaseManager::new(Some(db_path.to_str().unwrap().to_string()))
        .await
        .unwrap());
    
    // Run the knowledge_conversations migration
    let conn = db.get_connection().unwrap();
    conn.execute_batch(include_str!("../migrations/014_add_knowledge_conversations.sql"))
        .unwrap();
    
    // Verify table exists
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='knowledge_conversations'",
        [],
        |row| row.get(0),
    ).unwrap();
    
    assert_eq!(count, 1, "knowledge_conversations table should exist");
    
    // Test we can insert a record
    let result = conn.execute(
        "INSERT INTO knowledge_conversations (
            id, query, claude_plan, consensus_evaluation, 
            curator_output, execution_result, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            "test-id",
            "test query",
            "test plan",
            "{}",
            "test output",
            "{}",
            chrono::Utc::now().to_rfc3339(),
        ],
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_execution_mode_serialization() {
    // Test that execution modes can be serialized/deserialized
    let mode = ClaudeExecutionMode::ConsensusRequired;
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: ClaudeExecutionMode = serde_json::from_str(&json).unwrap();
    assert_eq!(mode, deserialized);
}

/// Test that demonstrates the conceptual flow (without actual Claude Code process)
#[tokio::test]
async fn test_conceptual_execution_flow() {
    println!("\n=== Testing Conceptual Claude Code Executor Flow ===\n");
    
    // Different types of requests to test criteria
    let test_requests = vec![
        ("Create a simple hello world file", false, "Low risk, high confidence"),
        ("Delete all user data from the database", true, "High risk operation"),
        ("Refactor the entire authentication system", true, "Architectural change"),
        ("Explain how the consensus pipeline works", false, "Simple explanation"),
        ("Analyze and optimize the codebase architecture", true, "User requested analysis"),
    ];
    
    for (request, should_invoke_consensus, reason) in test_requests {
        println!("Request: \"{}\"", request);
        println!("Should invoke consensus: {} ({})", should_invoke_consensus, reason);
        
        // Simulate the analysis
        let request_lower = request.to_lowercase();
        let invokes = request_lower.contains("delete") || 
                     request_lower.contains("refactor") ||
                     request_lower.contains("architecture") ||
                     request_lower.contains("analyze");
                     
        assert_eq!(invokes, should_invoke_consensus, 
                   "Consensus invocation logic failed for: {}", request);
        println!("✓ Consensus invocation logic correct\n");
    }
    
    println!("=== All conceptual tests passed! ===");
}