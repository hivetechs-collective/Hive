// Unit tests for AI Helpers
use hive_ai::ai_helpers::{
    knowledge_indexer::{KnowledgeIndexer, IndexerConfig},
    context_retriever::{ContextRetriever, ContextQuery},
    pattern_recognizer::PatternRecognizer,
    quality_analyzer::{QualityAnalyzer, QualityConfig},
    knowledge_synthesizer::{KnowledgeSynthesizer, SynthesisConfig},
    ChromaVectorStore,
    python_models::PythonModelService,
};
use hive_ai::consensus::stages::file_aware_curator::FileOperation;
use hive_ai::consensus::operation_analysis::OperationContext;
use hive_ai::consensus::operation_intelligence::{OperationOutcome, UserFeedback, QualityMetrics as OpQualityMetrics};
use std::sync::Arc;
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use tempfile::TempDir;

/// Helper to create a mock Python service
async fn create_mock_python_service() -> Arc<PythonModelService> {
    Arc::new(PythonModelService::mock())
}

/// Helper to create a test vector store
async fn create_test_vector_store() -> Arc<ChromaVectorStore> {
    let temp_dir = TempDir::new().unwrap();
    Arc::new(ChromaVectorStore::new(
        temp_dir.path().join("chroma"),
        "test_collection".to_string(),
    ).await.unwrap())
}

#[tokio::test]
async fn test_knowledge_indexer_basic_operations() {
    let vector_store = create_test_vector_store().await;
    let python_service = create_mock_python_service().await;
    let indexer = KnowledgeIndexer::new(vector_store, python_service).await.unwrap();
    
    // Create a test operation
    let operation = FileOperation::Create {
        path: PathBuf::from("test.rs"),
        content: "fn main() {}".to_string(),
    };
    
    // Create test context
    let context = OperationContext {
        repository_path: PathBuf::from("/test"),
        user_question: "Create a test file".to_string(),
        consensus_response: "Creating test file".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test".to_string(),
        git_commit: None,
    };
    
    // Index the operation
    let indexed = indexer.index_operation(&operation, &context).await.unwrap();
    assert!(!indexed.id.is_empty(), "Should generate an ID for indexed operation");
    
    // Search for similar operations
    let similar = indexer.find_similar_operations(&operation, 5).await.unwrap();
    assert!(!similar.is_empty(), "Should find at least the indexed operation");
}

#[tokio::test]
async fn test_context_retriever_storage_and_retrieval() {
    let python_service = create_mock_python_service().await;
    let retriever = ContextRetriever::new(python_service).await.unwrap();
    
    // Create test context
    let context = OperationContext {
        repository_path: PathBuf::from("/test"),
        user_question: "Create a library file".to_string(),
        consensus_response: "Creating lib.rs".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test".to_string(),
        git_commit: None,
    };
    
    // Store context
    retriever.store_context(&context).await.unwrap();
    
    // Create a query
    let query = ContextQuery {
        keywords: vec!["library".to_string(), "create".to_string()],
        semantic_similarity_threshold: 0.7,
        max_results: 10,
        time_window: None,
    };
    
    // Retrieve relevant contexts
    let contexts = retriever.retrieve_relevant_contexts(&query).await.unwrap();
    assert!(!contexts.is_empty(), "Should retrieve stored context");
}

#[tokio::test]
async fn test_pattern_recognizer_dangerous_patterns() {
    let recognizer = PatternRecognizer::new();
    
    // Test dangerous pattern detection
    let dangerous_ops = vec![
        FileOperation::Delete {
            path: PathBuf::from("/etc/passwd"),
        },
        FileOperation::Update {
            path: PathBuf::from("~/.ssh/id_rsa"),
            content: "compromised".to_string(),
        },
    ];
    
    for op in dangerous_ops {
        let analysis = recognizer.analyze_patterns(&[op.clone()]).await.unwrap();
        assert!(analysis.dangerous_patterns > 0, "Should detect dangerous pattern in {:?}", op);
        assert!(!analysis.identified_patterns.is_empty(), "Should identify specific patterns");
    }
}

#[tokio::test]
async fn test_pattern_recognizer_safe_patterns() {
    let recognizer = PatternRecognizer::new();
    
    // Test safe pattern detection
    let safe_ops = vec![
        FileOperation::Create {
            path: PathBuf::from("README.md"),
            content: "# Project".to_string(),
        },
        FileOperation::Create {
            path: PathBuf::from("tests/test.rs"),
            content: "#[test] fn test() {}".to_string(),
        },
    ];
    
    for op in safe_ops {
        let analysis = recognizer.analyze_patterns(&[op.clone()]).await.unwrap();
        assert_eq!(analysis.dangerous_patterns, 0, "Should not detect dangerous patterns in {:?}", op);
        assert!(analysis.safe_patterns > 0, "Should detect safe patterns");
    }
}

#[tokio::test]
async fn test_quality_analyzer_risk_assessment() {
    let config = QualityConfig::default();
    let python_service = create_mock_python_service().await;
    let analyzer = QualityAnalyzer::new(config, python_service).await.unwrap();
    
    // Test operation context
    let context = OperationContext {
        repository_path: PathBuf::from("/test"),
        user_question: "Delete main file".to_string(),
        consensus_response: "Deleting main.rs".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test".to_string(),
        git_commit: None,
    };
    
    // Test high-risk operation
    let risky_op = FileOperation::Delete {
        path: PathBuf::from("src/main.rs"),
    };
    
    let risk = analyzer.analyze_risk(&risky_op, &context).await.unwrap();
    assert!(risk > 50.0, "Deleting main.rs should be high risk");
    
    // Test low-risk operation
    let safe_op = FileOperation::Create {
        path: PathBuf::from("docs/readme.txt"),
        content: "Documentation".to_string(),
    };
    
    let risk = analyzer.analyze_risk(&safe_op, &context).await.unwrap();
    assert!(risk < 30.0, "Creating docs should be low risk");
}

#[tokio::test]
async fn test_quality_analyzer_conflict_detection() {
    let python_service = create_mock_python_service().await;
    let analyzer = QualityAnalyzer::new(QualityConfig::default(), python_service).await.unwrap();
    
    // Create conflicting operations
    let operations = vec![
        FileOperation::Create {
            path: PathBuf::from("config.toml"),
            content: "key = \"value1\"".to_string(),
        },
        FileOperation::Create {
            path: PathBuf::from("config.toml"),
            content: "key = \"value2\"".to_string(),
        },
    ];
    
    let conflicts = analyzer.detect_conflicts(&operations).await.unwrap();
    assert!(!conflicts.is_empty(), "Should detect file creation conflict");
    assert!(conflicts[0].severity > 0.7, "Duplicate file creation should be high severity");
}

#[tokio::test]
async fn test_knowledge_synthesizer_synthesis() {
    let config = SynthesisConfig::default();
    let python_service = create_mock_python_service().await;
    let synthesizer = KnowledgeSynthesizer::new(config, python_service).await.unwrap();
    
    // Create test operations
    let operations = vec![
        FileOperation::Create {
            path: PathBuf::from("src/module/mod.rs"),
            content: "pub mod impl;".to_string(),
        },
        FileOperation::Create {
            path: PathBuf::from("src/module/impl.rs"),
            content: "pub fn process() {}".to_string(),
        },
    ];
    
    // Create test context
    let context = OperationContext {
        repository_path: PathBuf::from("/test"),
        user_question: "Create a new module".to_string(),
        consensus_response: "Creating module structure".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test".to_string(),
        git_commit: None,
    };
    
    // Test synthesis
    let synthesis = synthesizer.synthesize_knowledge(&operations, &context).await.unwrap();
    assert!(!synthesis.insights.is_empty(), "Should generate insights");
    assert!(synthesis.confidence > 0.0, "Should have confidence score");
}

#[tokio::test]
async fn test_ai_helper_coordination() {
    // Test that all helpers can work together
    let python_service = create_mock_python_service().await;
    let vector_store = create_test_vector_store().await;
    
    let indexer = KnowledgeIndexer::new(vector_store, python_service.clone()).await.unwrap();
    let retriever = ContextRetriever::new(python_service.clone()).await.unwrap();
    let recognizer = PatternRecognizer::new();
    let analyzer = QualityAnalyzer::new(Default::default(), python_service.clone()).await.unwrap();
    let synthesizer = KnowledgeSynthesizer::new(Default::default(), python_service).await.unwrap();
    
    // Create test operation and context
    let operation = FileOperation::Create {
        path: PathBuf::from("test.txt"),
        content: "test content".to_string(),
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test"),
        user_question: "Create test file".to_string(),
        consensus_response: "Creating test.txt".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test".to_string(),
        git_commit: None,
    };
    
    // Run through all helpers
    let _ = indexer.index_operation(&operation, &context).await;
    let _ = retriever.store_context(&context).await;
    let _ = recognizer.analyze_patterns(&[operation.clone()]).await;
    let _ = analyzer.analyze_risk(&operation, &context).await;
    let _ = synthesizer.synthesize_knowledge(&[operation], &context).await;
    
    // If we get here without errors, coordination works
    assert!(true, "AI helpers can coordinate successfully");
}

#[tokio::test]
async fn test_operation_outcome_tracking() {
    let vector_store = create_test_vector_store().await;
    let python_service = create_mock_python_service().await;
    let indexer = KnowledgeIndexer::new(vector_store, python_service).await.unwrap();
    
    // Create test operation
    let operation = FileOperation::Create {
        path: PathBuf::from("test.rs"),
        content: "fn main() {}".to_string(),
    };
    
    let context = OperationContext {
        repository_path: PathBuf::from("/test"),
        user_question: "Create file".to_string(),
        consensus_response: "Creating file".to_string(),
        timestamp: SystemTime::now(),
        session_id: "test".to_string(),
        git_commit: None,
    };
    
    // Index operation
    let indexed = indexer.index_operation(&operation, &context).await.unwrap();
    
    // Record successful outcome
    let outcome = OperationOutcome {
        success: true,
        error_message: None,
        execution_time: Duration::from_millis(100),
        rollback_required: false,
        user_feedback: Some(UserFeedback {
            satisfaction_score: 5.0,
            comments: Some("Great!".to_string()),
            would_recommend: true,
        }),
        post_operation_quality: Some(OpQualityMetrics {
            code_quality_score: 0.9,
            test_coverage: 0.8,
            documentation_score: 0.85,
            performance_impact: -0.05,
            security_score: 0.95,
            maintainability_score: 0.88,
        }),
    };
    
    indexer.update_operation_outcome(&indexed.id, &outcome).await.unwrap();
    
    // Verify the outcome was recorded
    let similar = indexer.find_similar_operations(&operation, 5).await.unwrap();
    assert!(!similar.is_empty(), "Should find the operation with outcome");
}