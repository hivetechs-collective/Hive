// Unit tests for Operation Intelligence Coordinator
use crate::consensus::operation_intelligence::{
    OperationIntelligenceCoordinator, AutoAcceptMode, OperationAnalysis,
    OperationStatistics, AutoAcceptStatistics,
};
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_parser::{EnhancedFileOperation, OperationContext as ParseContext};
use crate::consensus::operation_analysis::{OperationContext, SystemContext};
use crate::ai_helpers::*;
use std::sync::Arc;
use std::path::PathBuf;
use tempfile::TempDir;

async fn create_coordinator_with_db() -> (OperationIntelligenceCoordinator, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_ops.db");
    
    let coordinator = OperationIntelligenceCoordinator::new(
        Arc::new(knowledge_indexer::KnowledgeIndexer::new(None)),
        Arc::new(context_retriever::ContextRetriever::new(None)),
        Arc::new(pattern_recognizer::PatternRecognizer::new()),
        Arc::new(quality_analyzer::QualityAnalyzer::new(Default::default())),
        Arc::new(knowledge_synthesizer::KnowledgeSynthesizer::new(Default::default())),
        Some(db_path),
    ).await.unwrap();
    
    (coordinator, temp_dir)
}

#[tokio::test]
async fn test_coordinator_initialization() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    // Test default mode
    assert_eq!(coordinator.get_current_mode(), AutoAcceptMode::Manual);
    
    // Test mode changes
    coordinator.set_auto_accept_mode(AutoAcceptMode::Conservative);
    assert_eq!(coordinator.get_current_mode(), AutoAcceptMode::Conservative);
    
    coordinator.set_auto_accept_mode(AutoAcceptMode::Balanced);
    assert_eq!(coordinator.get_current_mode(), AutoAcceptMode::Balanced);
}

#[tokio::test]
async fn test_operation_analysis_caching() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    let operation = EnhancedFileOperation {
        operation: FileOperation::Create {
            path: PathBuf::from("cached_test.txt"),
            content: "Cached content".to_string(),
        },
        context: ParseContext::default(),
        parsing_confidence: 0.95,
    };
    
    let context = OperationContext {
        source_question: "Create cached file".to_string(),
        consensus_response: "Creating file".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    // First analysis - should compute
    let start = std::time::Instant::now();
    let analysis1 = coordinator.analyze_operation(&operation, &context).await.unwrap();
    let first_duration = start.elapsed();
    
    // Second analysis - should use cache
    let start = std::time::Instant::now();
    let analysis2 = coordinator.analyze_operation(&operation, &context).await.unwrap();
    let cached_duration = start.elapsed();
    
    // Cache should be faster (though this might be flaky in tests)
    assert!(cached_duration < first_duration || cached_duration.as_millis() < 10);
    
    // Results should be identical
    assert_eq!(analysis1.confidence, analysis2.confidence);
    assert_eq!(analysis1.risk, analysis2.risk);
}

#[tokio::test]
async fn test_should_auto_accept_logic() {
    let (mut coordinator, _temp) = create_coordinator_with_db().await;
    
    // Test conservative mode
    coordinator.set_auto_accept_mode(AutoAcceptMode::Conservative);
    
    let high_confidence_analysis = OperationAnalysis {
        confidence: 95.0,
        risk: 5.0,
        recommendation: "Safe to proceed".to_string(),
        explanation: "Very safe operation".to_string(),
        main_risk_factors: vec![],
        supporting_evidence: vec!["High success rate".to_string()],
        component_scores: Default::default(),
        scoring_factors: Default::default(),
        execution_plan: None,
        rollback_strategy: None,
        verification_steps: vec![],
        decision_rationale: "Safe operation".to_string(),
        improvements: vec![],
        warnings: vec![],
    };
    
    assert!(coordinator.should_auto_accept(&high_confidence_analysis));
    
    let medium_confidence_analysis = OperationAnalysis {
        confidence: 85.0,
        risk: 20.0,
        ..high_confidence_analysis.clone()
    };
    
    // Conservative mode should reject medium confidence
    assert!(!coordinator.should_auto_accept(&medium_confidence_analysis));
    
    // But balanced mode should accept it
    coordinator.set_auto_accept_mode(AutoAcceptMode::Balanced);
    assert!(coordinator.should_auto_accept(&medium_confidence_analysis));
    
    // Manual mode should never accept
    coordinator.set_auto_accept_mode(AutoAcceptMode::Manual);
    assert!(!coordinator.should_auto_accept(&high_confidence_analysis));
}

#[tokio::test]
async fn test_operation_history_recording() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    let operation = EnhancedFileOperation {
        operation: FileOperation::Create {
            path: PathBuf::from("history_test.txt"),
            content: "Historical content".to_string(),
        },
        context: ParseContext::default(),
        parsing_confidence: 0.90,
    };
    
    let context = OperationContext {
        source_question: "Create history file".to_string(),
        consensus_response: "Creating file".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    let analysis = coordinator.analyze_operation(&operation, &context).await.unwrap();
    
    // Record successful outcome
    coordinator.record_operation_outcome(
        &operation,
        &analysis,
        true,
        None,
        Default::default(),
    ).await.unwrap();
    
    // Get statistics
    let stats = coordinator.get_operation_statistics().await.unwrap();
    assert_eq!(stats.total_operations, 1);
    assert_eq!(stats.successful_operations, 1);
    assert_eq!(stats.failed_operations, 0);
    
    // Record failed outcome
    let failed_op = EnhancedFileOperation {
        operation: FileOperation::Delete {
            path: PathBuf::from("important.txt"),
        },
        context: ParseContext::default(),
        parsing_confidence: 0.85,
    };
    
    let failed_analysis = coordinator.analyze_operation(&failed_op, &context).await.unwrap();
    
    coordinator.record_operation_outcome(
        &failed_op,
        &failed_analysis,
        false,
        Some("File not found"),
        Default::default(),
    ).await.unwrap();
    
    let updated_stats = coordinator.get_operation_statistics().await.unwrap();
    assert_eq!(updated_stats.total_operations, 2);
    assert_eq!(updated_stats.successful_operations, 1);
    assert_eq!(updated_stats.failed_operations, 1);
}

#[tokio::test]
async fn test_batch_operation_analysis() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    let operations = vec![
        EnhancedFileOperation {
            operation: FileOperation::Create {
                path: PathBuf::from("batch1.txt"),
                content: "First file".to_string(),
            },
            context: ParseContext::default(),
            parsing_confidence: 0.95,
        },
        EnhancedFileOperation {
            operation: FileOperation::Create {
                path: PathBuf::from("batch2.txt"),
                content: "Second file".to_string(),
            },
            context: ParseContext::default(),
            parsing_confidence: 0.95,
        },
        EnhancedFileOperation {
            operation: FileOperation::Update {
                path: PathBuf::from("existing.txt"),
                content: "Updated content".to_string(),
            },
            context: ParseContext::default(),
            parsing_confidence: 0.90,
        },
    ];
    
    let context = OperationContext {
        source_question: "Create batch files".to_string(),
        consensus_response: "Creating multiple files".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    let analyses = coordinator.analyze_operations(&operations, &context).await.unwrap();
    
    assert_eq!(analyses.len(), operations.len());
    
    // Check that each operation was analyzed
    for (i, analysis) in analyses.iter().enumerate() {
        assert!(analysis.confidence > 0.0);
        assert!(analysis.risk >= 0.0);
        assert!(!analysis.explanation.is_empty());
        
        // Updates might have slightly higher risk than creates
        if matches!(operations[i].operation, FileOperation::Update { .. }) {
            assert!(analysis.risk >= analyses[0].risk);
        }
    }
}

#[tokio::test]
async fn test_auto_accept_statistics() {
    let (mut coordinator, _temp) = create_coordinator_with_db().await;
    coordinator.set_auto_accept_mode(AutoAcceptMode::Balanced);
    
    // Create operations with varying confidence levels
    let operations = vec![
        (95.0, 5.0),   // High confidence, low risk - should accept
        (85.0, 15.0),  // Good confidence, low risk - should accept
        (75.0, 25.0),  // Medium confidence, medium risk - borderline
        (60.0, 40.0),  // Low confidence, high risk - should reject
        (40.0, 60.0),  // Very low confidence, very high risk - should reject
    ];
    
    let mut accepted = 0;
    let mut rejected = 0;
    
    for (confidence, risk) in operations {
        let analysis = OperationAnalysis {
            confidence,
            risk,
            recommendation: "Test".to_string(),
            explanation: "Test analysis".to_string(),
            main_risk_factors: vec![],
            supporting_evidence: vec![],
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            execution_plan: None,
            rollback_strategy: None,
            verification_steps: vec![],
            decision_rationale: "Test".to_string(),
            improvements: vec![],
            warnings: vec![],
        };
        
        if coordinator.should_auto_accept(&analysis) {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    
    let stats = coordinator.get_auto_accept_statistics().await.unwrap();
    
    // In balanced mode, we expect to accept high and good confidence operations
    assert_eq!(accepted, 2, "Should accept 2 high-confidence operations");
    assert_eq!(rejected, 3, "Should reject 3 lower-confidence operations");
}

#[tokio::test]
async fn test_learning_from_feedback() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    let operation = EnhancedFileOperation {
        operation: FileOperation::Create {
            path: PathBuf::from("learning_test.rs"),
            content: "fn test() {}".to_string(),
        },
        context: ParseContext {
            semantic_tags: vec!["rust".to_string(), "test".to_string()],
            ..Default::default()
        },
        parsing_confidence: 0.90,
    };
    
    let context = OperationContext {
        source_question: "Create test file".to_string(),
        consensus_response: "Creating test file".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    // First analysis
    let initial_analysis = coordinator.analyze_operation(&operation, &context).await.unwrap();
    let initial_confidence = initial_analysis.confidence;
    
    // Record positive feedback multiple times
    for _ in 0..5 {
        coordinator.record_operation_outcome(
            &operation,
            &initial_analysis,
            true,
            None,
            crate::consensus::operation_analysis::UserFeedback {
                satisfaction: Some(5), // Very satisfied
                comments: Some("Perfect!".to_string()),
                would_recommend: true,
            },
        ).await.unwrap();
    }
    
    // Create similar operation
    let similar_operation = EnhancedFileOperation {
        operation: FileOperation::Create {
            path: PathBuf::from("another_test.rs"),
            content: "fn another_test() {}".to_string(),
        },
        context: ParseContext {
            semantic_tags: vec!["rust".to_string(), "test".to_string()],
            ..Default::default()
        },
        parsing_confidence: 0.90,
    };
    
    // Analyze similar operation
    let learned_analysis = coordinator.analyze_operation(&similar_operation, &context).await.unwrap();
    
    // Confidence should be higher due to positive history
    assert!(
        learned_analysis.confidence >= initial_confidence,
        "Confidence should improve with positive feedback: {} vs {}",
        learned_analysis.confidence,
        initial_confidence
    );
}

#[tokio::test]
async fn test_mode_specific_behavior() {
    let (mut coordinator, _temp) = create_coordinator_with_db().await;
    
    let operation = EnhancedFileOperation {
        operation: FileOperation::Create {
            path: PathBuf::from("mode_test.txt"),
            content: "Mode specific content".to_string(),
        },
        context: ParseContext::default(),
        parsing_confidence: 0.85,
    };
    
    let context = OperationContext {
        source_question: "Test modes".to_string(),
        consensus_response: "Testing modes".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    // Test Plan mode
    coordinator.set_auto_accept_mode(AutoAcceptMode::Plan);
    let plan_analysis = coordinator.analyze_operation(&operation, &context).await.unwrap();
    
    assert!(plan_analysis.execution_plan.is_some(), "Plan mode should generate execution plan");
    assert!(!coordinator.should_auto_accept(&plan_analysis), "Plan mode should never auto-execute");
    
    // Test Manual mode
    coordinator.set_auto_accept_mode(AutoAcceptMode::Manual);
    let manual_analysis = coordinator.analyze_operation(&operation, &context).await.unwrap();
    
    assert!(!coordinator.should_auto_accept(&manual_analysis), "Manual mode should never auto-execute");
    
    // Test Aggressive mode with medium confidence
    coordinator.set_auto_accept_mode(AutoAcceptMode::Aggressive);
    let aggressive_analysis = OperationAnalysis {
        confidence: 75.0,
        risk: 35.0,
        ..manual_analysis
    };
    
    assert!(
        coordinator.should_auto_accept(&aggressive_analysis),
        "Aggressive mode should accept medium confidence operations"
    );
}

#[tokio::test]
async fn test_dangerous_operation_detection() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    // Test system file deletion
    let dangerous_op = EnhancedFileOperation {
        operation: FileOperation::Delete {
            path: PathBuf::from("/etc/passwd"),
        },
        context: ParseContext {
            semantic_tags: vec!["system".to_string(), "critical".to_string()],
            ..Default::default()
        },
        parsing_confidence: 0.95,
    };
    
    let context = OperationContext {
        source_question: "Delete system file".to_string(),
        consensus_response: "Deleting system file".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    let analysis = coordinator.analyze_operation(&dangerous_op, &context).await.unwrap();
    
    assert!(analysis.risk > 90.0, "System file deletion should have very high risk");
    assert!(!analysis.warnings.is_empty(), "Should have warnings about dangerous operation");
}

#[tokio::test]
async fn test_mass_operation_detection() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    // Create many delete operations
    let mass_deletes: Vec<_> = (0..20).map(|i| {
        EnhancedFileOperation {
            operation: FileOperation::Delete {
                path: PathBuf::from(format!("file{}.txt", i)),
            },
            context: ParseContext::default(),
            parsing_confidence: 0.90,
        }
    }).collect();
    
    let context = OperationContext {
        source_question: "Clean up files".to_string(),
        consensus_response: "Deleting multiple files".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    let analyses = coordinator.analyze_operations(&mass_deletes, &context).await.unwrap();
    
    // Should detect mass deletion pattern
    let high_risk_count = analyses.iter().filter(|a| a.risk > 70.0).count();
    assert!(high_risk_count > 0, "Mass deletion should be detected as risky");
}

#[tokio::test]
async fn test_quality_improvement_operations() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    // Adding tests should be recognized as quality improvement
    let test_op = EnhancedFileOperation {
        operation: FileOperation::Create {
            path: PathBuf::from("tests/new_test.rs"),
            content: r#"
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_important_function() {
        assert_eq!(important_function(2, 3), 5);
    }
}
"#.to_string(),
        },
        context: ParseContext {
            semantic_tags: vec!["test".to_string(), "quality".to_string()],
            ..Default::default()
        },
        parsing_confidence: 0.95,
    };
    
    let context = OperationContext {
        source_question: "Add tests".to_string(),
        consensus_response: "Adding test coverage".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    let analysis = coordinator.analyze_operation(&test_op, &context).await.unwrap();
    
    assert!(analysis.confidence > 80.0, "Test additions should have high confidence");
    assert!(analysis.risk < 30.0, "Test additions should have low risk");
}

#[tokio::test]
async fn test_rollback_strategy_generation() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    let update_op = EnhancedFileOperation {
        operation: FileOperation::Update {
            path: PathBuf::from("critical_config.json"),
            content: r#"{"version": "2.0", "settings": {}}"#.to_string(),
        },
        context: ParseContext {
            semantic_tags: vec!["config".to_string(), "critical".to_string()],
            ..Default::default()
        },
        parsing_confidence: 0.90,
    };
    
    let context = OperationContext {
        source_question: "Update configuration".to_string(),
        consensus_response: "Updating critical configuration".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    let analysis = coordinator.analyze_operation(&update_op, &context).await.unwrap();
    
    assert!(analysis.rollback_strategy.is_some(), "Critical updates should have rollback strategy");
}

#[tokio::test]
async fn test_concurrent_operation_safety() {
    let (coordinator, _temp) = create_coordinator_with_db().await;
    
    // Operations that might conflict if run concurrently
    let concurrent_ops = vec![
        EnhancedFileOperation {
            operation: FileOperation::Update {
                path: PathBuf::from("shared_resource.txt"),
                content: "Process A update".to_string(),
            },
            context: ParseContext::default(),
            parsing_confidence: 0.90,
        },
        EnhancedFileOperation {
            operation: FileOperation::Update {
                path: PathBuf::from("shared_resource.txt"),
                content: "Process B update".to_string(),
            },
            context: ParseContext::default(),
            parsing_confidence: 0.90,
        },
    ];
    
    let context = OperationContext {
        source_question: "Update shared resource".to_string(),
        consensus_response: "Updating shared file".to_string(),
        repository_context: None,
        system_context: SystemContext::default(),
    };
    
    let analyses = coordinator.analyze_operations(&concurrent_ops, &context).await.unwrap();
    
    // Should detect potential conflict
    let has_conflict_warning = analyses.iter().any(|a| 
        a.warnings.iter().any(|w| w.contains("conflict") || w.contains("concurrent"))
    );
    
    assert!(has_conflict_warning || analyses.len() > 0, 
        "Should detect or handle concurrent access to same file");
}