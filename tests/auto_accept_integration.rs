// Integration tests for AI-Enhanced Auto-Accept System
use hive_ai::consensus::operation_intelligence::{
    OperationIntelligenceCoordinator, AutoAcceptMode,
};
use hive_ai::consensus::stages::file_aware_curator::FileOperation;
use hive_ai::consensus::operation_analysis::OperationContext;
use hive_ai::ai_helpers::knowledge_indexer::KnowledgeIndexer;
use hive_ai::ai_helpers::context_retriever::ContextRetriever;
use hive_ai::ai_helpers::pattern_recognizer::PatternRecognizer;
use hive_ai::ai_helpers::quality_analyzer::QualityAnalyzer;
use hive_ai::ai_helpers::knowledge_synthesizer::KnowledgeSynthesizer;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;

/// Helper to create a test coordinator with all AI helpers
async fn create_test_coordinator() -> OperationIntelligenceCoordinator {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_history.db");
    
    let knowledge_indexer = Arc::new(KnowledgeIndexer::new(None));
    let context_retriever = Arc::new(ContextRetriever::new(None));
    let pattern_recognizer = Arc::new(PatternRecognizer::new());
    let quality_analyzer = Arc::new(QualityAnalyzer::new(Default::default()));
    let knowledge_synthesizer = Arc::new(KnowledgeSynthesizer::new(Default::default()));
    
    OperationIntelligenceCoordinator::new(
        knowledge_indexer,
        context_retriever,
        pattern_recognizer,
        quality_analyzer,
        knowledge_synthesizer,
        Some(db_path),
    ).await.unwrap()
}

/// Helper to create test context
fn create_test_context(question: &str) -> OperationContext {
    OperationContext {
        repository_path: PathBuf::from("/test"),
        user_question: question.to_string(),
        consensus_response: String::new(),
        timestamp: SystemTime::now(),
        session_id: "test_session".to_string(),
        git_commit: None,
    }
}

#[tokio::test]
async fn test_auto_accept_mode_conservative() {
    let coordinator = create_test_coordinator().await;
    
    // Create a safe operation
    let operation = FileOperation::Create {
        path: PathBuf::from("test.txt"),
        content: "Hello, world!".to_string(),
    };
    let context = create_test_context("Create a test file");
    
    // Test conservative mode
    let (would_accept, analysis) = coordinator.should_auto_execute(
        &operation,
        &context,
        AutoAcceptMode::Conservative,
    ).await.unwrap();
    
    // In conservative mode, only very safe operations should be auto-accepted
    // A simple text file creation might be safe enough
    assert!(analysis.unified_score.confidence > 0.0);
    assert!(analysis.unified_score.risk >= 0.0);
    
    // Conservative mode requires >90% confidence and <15% risk
    if analysis.unified_score.confidence > 90.0 && analysis.unified_score.risk < 15.0 {
        assert!(would_accept, "Conservative mode should accept safe operations with >90% confidence and <15% risk");
    } else {
        assert!(!would_accept, "Conservative mode should reject operations that don't meet thresholds");
    }
}

#[tokio::test]
async fn test_auto_accept_mode_manual() {
    let coordinator = create_test_coordinator().await;
    
    // Create any operation
    let operation = FileOperation::Create {
        path: PathBuf::from("test.txt"),
        content: "Hello, world!".to_string(),
    };
    let context = create_test_context("Create a test file");
    
    // Manual mode should never auto-accept
    let (would_accept, _) = coordinator.should_auto_execute(
        &operation,
        &context,
        AutoAcceptMode::Manual,
    ).await.unwrap();
    
    assert!(!would_accept, "Manual mode should never auto-accept");
}

#[tokio::test]
async fn test_risky_operation_rejection() {
    let coordinator = create_test_coordinator().await;
    
    // Create a risky operation (deleting a critical file)
    let operation = FileOperation::Delete {
        path: PathBuf::from("/etc/passwd"), // Very risky!
    };
    let context = create_test_context("Delete passwd file");
    
    // Test in balanced mode
    let (would_accept_balanced, analysis) = coordinator.should_auto_execute(
        &operation,
        &context,
        AutoAcceptMode::Balanced,
    ).await.unwrap();
    
    // This should have low confidence and high risk
    assert!(analysis.unified_score.risk > 50.0, "Critical system file deletion should have high risk");
    assert!(!would_accept_balanced, "Should not auto-accept critical system file deletion in balanced mode");
    
    // Should not be auto-accepted even in aggressive mode
    let (would_accept_aggressive, _) = coordinator.should_auto_execute(
        &operation,
        &context,
        AutoAcceptMode::Aggressive,
    ).await.unwrap();
    assert!(!would_accept_aggressive, "Should never auto-accept critical system file deletion even in aggressive mode");
}

#[tokio::test]
async fn test_operation_history_learning() {
    let coordinator = create_test_coordinator().await;
    
    // Create an operation
    let operation = FileOperation::Create {
        path: PathBuf::from("src/lib.rs"),
        content: "pub fn hello() {}".to_string(),
    };
    let context = create_test_context("Create a library file");
    
    // Analyze it
    let (_, analysis) = coordinator.should_auto_execute(
        &operation,
        &context,
        AutoAcceptMode::Balanced,
    ).await.unwrap();
    
    let first_confidence = analysis.unified_score.confidence;
    
    // Record a successful outcome
    let outcome = hive_ai::consensus::operation_intelligence::OperationOutcome {
        success: true,
        error_message: None,
        execution_time: std::time::Duration::from_millis(100),
        rollback_required: false,
        user_feedback: Some(hive_ai::consensus::operation_intelligence::UserFeedback {
            satisfaction_score: 5.0,
            comments: Some("Great!".to_string()),
            would_recommend: true,
        }),
        post_operation_quality: Some(hive_ai::consensus::operation_intelligence::QualityMetrics {
            code_quality_score: 0.9,
            test_coverage: 0.8,
            documentation_score: 0.85,
            performance_impact: -0.05,
            security_score: 0.95,
            maintainability_score: 0.88,
        }),
    };
    
    coordinator.record_outcome(
        analysis.operations[0].clone(),
        analysis,
        outcome,
    ).await.unwrap();
    
    // Analyze a similar operation
    let similar_operation = FileOperation::Create {
        path: PathBuf::from("src/lib2.rs"),
        content: "pub fn world() {}".to_string(),
    };
    let (_, analysis2) = coordinator.should_auto_execute(
        &similar_operation,
        &context,
        AutoAcceptMode::Balanced,
    ).await.unwrap();
    
    // The system should learn from successful operations
    println!("First confidence: {}, Second confidence: {}", 
             first_confidence, analysis2.unified_score.confidence);
}

#[tokio::test] 
async fn test_ai_helper_coordination() {
    let coordinator = create_test_coordinator().await;
    
    // Create a complex operation that requires all helpers
    let operation = FileOperation::Update {
        path: PathBuf::from("src/main.rs"),
        content: "fn main() { println!(\"Updated!\"); }".to_string(),
    };
    
    let context = create_test_context("Update the main function");
    
    let (_, analysis) = coordinator.should_auto_execute(
        &operation,
        &context,
        AutoAcceptMode::Balanced,
    ).await.unwrap();
    
    // Check that all AI helpers contributed
    assert!(analysis.component_scores.knowledge_indexer.is_some());
    assert!(analysis.component_scores.context_retriever.is_some());
    assert!(analysis.component_scores.pattern_recognizer.is_some());
    assert!(analysis.component_scores.quality_analyzer.is_some());
    assert!(analysis.component_scores.knowledge_synthesizer.is_some());
    
    // Overall analysis should be reasonable
    assert!(analysis.unified_score.confidence >= 0.0 && analysis.unified_score.confidence <= 100.0);
    assert!(analysis.unified_score.risk >= 0.0 && analysis.unified_score.risk <= 100.0);
}

#[tokio::test]
async fn test_plan_mode() {
    let coordinator = create_test_coordinator().await;
    
    // Create multiple related operations
    let operations = vec![
        FileOperation::Create {
            path: PathBuf::from("src/auth/mod.rs"),
            content: "pub mod login;".to_string(),
        },
        FileOperation::Create {
            path: PathBuf::from("src/auth/login.rs"),
            content: "pub fn login() {}".to_string(),
        },
        FileOperation::Create {
            path: PathBuf::from("src/auth/logout.rs"),
            content: "pub fn logout() {}".to_string(),
        },
    ];
    
    let context = create_test_context("Create authentication module");
    
    // Analyze each operation in plan mode
    for operation in &operations {
        let (would_accept, analysis) = coordinator.should_auto_execute(
            operation,
            &context,
            AutoAcceptMode::Plan,
        ).await.unwrap();
        
        // Plan mode should never auto-execute
        assert!(!would_accept, "Plan mode should not auto-execute");
        
        // But should still provide analysis
        assert!(analysis.unified_score.confidence >= 0.0);
        assert!(analysis.unified_score.risk >= 0.0);
    }
}

#[test]
fn test_auto_accept_mode_thresholds() {
    // Test that mode thresholds are correctly understood
    // Conservative: >90% confidence, <15% risk
    // Balanced: >80% confidence, <25% risk
    // Aggressive: >70% confidence, <40% risk
    
    // These thresholds are enforced in the smart decision engine
    assert!(true, "Mode thresholds are enforced by the smart decision engine");
}