// Integration tests for the 4-stage consensus pipeline
// Verifies behavior matches TypeScript implementation

use hive::{ConsensusEngine, initialize};
use std::sync::Arc;

#[tokio::test]
async fn test_consensus_basic_query() {
    // Initialize the system
    initialize().await.expect("Failed to initialize");

    // Create consensus engine
    let db = Arc::new(hive::core::Database::memory().await.unwrap());
    let engine = ConsensusEngine::new(db).await.unwrap();

    // Test basic query
    let result = engine
        .process("What is Rust programming language?", None)
        .await
        .expect("Consensus should process successfully");

    assert!(result.success);
    assert!(result.result.is_some());
    assert_eq!(result.stages.len(), 4);

    // Verify all stages ran
    let stage_names: Vec<_> = result.stages.iter().map(|s| &s.stage_name).collect();
    assert_eq!(stage_names, vec!["generator", "refiner", "validator", "curator"]);
}

#[tokio::test]
async fn test_consensus_with_temporal_context() {
    initialize().await.expect("Failed to initialize");

    let db = Arc::new(hive::core::Database::memory().await.unwrap());
    let engine = ConsensusEngine::new(db).await.unwrap();

    // Query that should trigger temporal context
    let result = engine
        .process("What are the latest Rust features in 2024?", None)
        .await
        .expect("Consensus should process with temporal context");

    assert!(result.success);
    assert!(result.result.is_some());
    
    // The result should mention current date context
    // (In real implementation, this would be injected into prompts)
}

#[tokio::test]
async fn test_consensus_with_semantic_context() {
    initialize().await.expect("Failed to initialize");

    let db = Arc::new(hive::core::Database::memory().await.unwrap());
    let engine = ConsensusEngine::new(db).await.unwrap();

    let semantic_context = "Repository: hive-ai
Language: Rust
Framework: Tokio async runtime
Purpose: AI-powered codebase intelligence";

    let result = engine
        .process("What does this code do?", Some(semantic_context.to_string()))
        .await
        .expect("Consensus should process with semantic context");

    assert!(result.success);
    assert!(result.result.is_some());
}

#[tokio::test]
async fn test_consensus_streaming() {
    use hive::consensus::{StreamingCallbacks, Stage};
    use std::sync::atomic::{AtomicU32, Ordering};

    initialize().await.expect("Failed to initialize");

    let db = Arc::new(hive::core::Database::memory().await.unwrap());
    let engine = ConsensusEngine::new(db).await.unwrap();

    // Track streaming events
    struct TestCallbacks {
        token_count: AtomicU32,
        stages_started: AtomicU32,
        stages_completed: AtomicU32,
    }

    impl StreamingCallbacks for TestCallbacks {
        fn on_stage_start(&self, _stage: Stage, _model: &str) -> anyhow::Result<()> {
            self.stages_started.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn on_stage_chunk(&self, _stage: Stage, _chunk: &str, _total: &str) -> anyhow::Result<()> {
            self.token_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn on_stage_complete(&self, _stage: Stage, _result: &hive::consensus::types::StageResult) -> anyhow::Result<()> {
            self.stages_completed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    let callbacks = Arc::new(TestCallbacks {
        token_count: AtomicU32::new(0),
        stages_started: AtomicU32::new(0),
        stages_completed: AtomicU32::new(0),
    });

    let result = engine
        .process_with_callbacks("Explain async/await in Rust", None, callbacks.clone())
        .await
        .expect("Streaming consensus should work");

    assert!(result.success);
    assert_eq!(callbacks.stages_started.load(Ordering::SeqCst), 4);
    assert_eq!(callbacks.stages_completed.load(Ordering::SeqCst), 4);
    assert!(callbacks.token_count.load(Ordering::SeqCst) > 0);
}

#[tokio::test]
async fn test_consensus_profiles() {
    initialize().await.expect("Failed to initialize");

    let db = Arc::new(hive::core::Database::memory().await.unwrap());
    let engine = ConsensusEngine::new(db).await.unwrap();

    // Test different profiles
    let profiles = ["balanced", "speed", "quality", "cost"];
    
    for profile in &profiles {
        engine.set_profile(profile).await.expect(&format!("Should set {} profile", profile));
        
        let current = engine.get_current_profile().await;
        assert!(current.profile_name.to_lowercase().contains(profile));
    }
}

#[tokio::test]
async fn test_consensus_error_handling() {
    initialize().await.expect("Failed to initialize");

    let db = Arc::new(hive::core::Database::memory().await.unwrap());
    let engine = ConsensusEngine::new(db).await.unwrap();

    // Test with empty query (should still handle gracefully)
    let result = engine
        .process("", None)
        .await
        .expect("Should handle empty query");

    assert!(result.success); // Should still succeed with some response
}

#[test]
fn test_temporal_detection() {
    use hive::consensus::temporal::TemporalContextProvider;

    let provider = TemporalContextProvider::default();

    // Queries that should trigger temporal context
    let temporal_queries = vec![
        "What are the latest Rust features?",
        "Search for recent AI developments",
        "What's the current stock price of AAPL?",
        "Find news about OpenAI today",
        "What happened in tech this week?",
        "Show me 2024 programming trends",
    ];

    for query in temporal_queries {
        assert!(
            provider.requires_temporal_context(query),
            "Should detect temporal context for: {}",
            query
        );
    }

    // Queries that should NOT trigger temporal context
    let non_temporal_queries = vec![
        "How do I write a for loop in Rust?",
        "Explain the concept of ownership",
        "What is a HashMap?",
        "Debug this function",
    ];

    for query in non_temporal_queries {
        assert!(
            !provider.requires_temporal_context(query),
            "Should NOT detect temporal context for: {}",
            query
        );
    }
}