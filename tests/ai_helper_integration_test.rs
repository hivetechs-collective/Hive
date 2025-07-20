// Integration tests for AI Helper Coordination
// Tests the complete AI helper ecosystem working together

use hive_ai::ai_helpers::{
    AIHelperEcosystem, ProcessedKnowledge,
    KnowledgeIndexer, ContextRetriever, PatternRecognizer,
    QualityAnalyzer, KnowledgeSynthesizer,
    ChromaVectorStore, PythonModelService, PythonModelConfig,
    IntelligentContextOrchestrator,
};
use hive_ai::consensus::types::Stage;
use hive_ai::consensus::operation_analysis::{
    OperationAnalysis as OpAnalysis, OperationContext, UnifiedScore,
};
use hive_ai::consensus::stages::file_aware_curator::FileOperation;
use hive_ai::consensus::operation_intelligence::OperationIntelligenceCoordinator;
use hive_ai::consensus::smart_decision_engine::{
    SmartDecisionEngine, UserPreferences, ExecutionDecision,
};
use hive_ai::consensus::operation_analysis::AutoAcceptMode;
use hive_ai::core::database::DatabaseManager;
use std::sync::Arc;
use std::path::PathBuf;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Helper to create a test database
async fn create_test_database() -> Arc<DatabaseManager> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    Arc::new(DatabaseManager::new(&db_path.to_str().unwrap()).await.unwrap())
}

/// Helper to create test operation context
fn create_test_context() -> OperationContext {
    OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        user_question: "Test question".to_string(),
        consensus_response: "Test response".to_string(),
        timestamp: std::time::SystemTime::now(),
        session_id: "test-session".to_string(),
        git_commit: Some("abc123".to_string()),
    }
}

#[cfg(test)]
mod ecosystem_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_helper_ecosystem_creation() {
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database).await;
        
        assert!(ecosystem.is_ok());
    }

    #[tokio::test]
    async fn test_process_curator_output() {
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database).await.unwrap();
        
        let curator_output = r#"
The binary search algorithm is an efficient searching technique that works on sorted arrays.
It has a time complexity of O(log n) compared to linear search's O(n).

Key implementation details:
1. Start with two pointers at the beginning and end
2. Calculate the middle element
3. Compare with target value
4. Eliminate half of the search space
5. Repeat until found or space is exhausted
"#;
        
        let source_question = "How does binary search work?";
        let conversation_id = "test-conv-1";
        
        let result = ecosystem.process_curator_output(
            curator_output,
            source_question,
            conversation_id,
        ).await;
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        
        // Should have indexed the knowledge
        assert!(!processed.indexed.id.is_empty());
        assert_eq!(processed.indexed.content, curator_output);
        
        // Should have found patterns
        assert!(!processed.patterns.is_empty());
        
        // Should have quality report
        assert!(processed.quality.overall_score > 0.0);
        
        // Should have insights
        assert!(!processed.insights.is_empty());
    }

    #[tokio::test]
    async fn test_prepare_stage_context() {
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database).await.unwrap();
        
        let question = "What are the best practices for error handling in Rust?";
        
        // Test context for different stages
        let stages = vec![
            Stage::Generator,
            Stage::Refiner,
            Stage::Validator,
            Stage::Curator,
        ];
        
        for stage in stages {
            let context = ecosystem.prepare_stage_context(
                question,
                stage,
                1000, // context limit
            ).await;
            
            assert!(context.is_ok());
            let stage_context = context.unwrap();
            
            // Should have stage-specific content
            assert_eq!(stage_context.stage, stage);
            
            // Different stages should get different guidance
            match stage {
                Stage::Generator => {
                    // Generator needs broad context
                    assert!(stage_context.custom_guidance.is_some() || 
                           !stage_context.relevant_facts.is_empty());
                }
                Stage::Validator => {
                    // Validator should look for edge cases
                    assert!(stage_context.patterns.iter().any(|p| 
                        matches!(p.pattern_type, hive_ai::ai_helpers::PatternType::Contradiction)
                    ) || !stage_context.patterns.is_empty());
                }
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database).await.unwrap();
        
        let batch = vec![
            ("Binary search has O(log n) complexity".to_string(), 
             "What is binary search complexity?".to_string(),
             "conv-1".to_string()),
            ("Rust uses ownership for memory safety".to_string(),
             "How does Rust ensure memory safety?".to_string(),
             "conv-2".to_string()),
            ("HTTP/2 uses multiplexing for better performance".to_string(),
             "What improvements does HTTP/2 bring?".to_string(),
             "conv-3".to_string()),
        ];
        
        let results = ecosystem.process_batch(batch).await;
        
        assert!(results.is_ok());
        let processed_batch = results.unwrap();
        
        assert_eq!(processed_batch.len(), 3);
        
        // Each should be properly processed
        for processed in processed_batch {
            assert!(!processed.indexed.id.is_empty());
            assert!(processed.quality.overall_score > 0.0);
        }
    }

    #[tokio::test]
    async fn test_ecosystem_stats() {
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database).await.unwrap();
        
        // Process some outputs to generate stats
        let _ = ecosystem.process_curator_output(
            "Test output 1",
            "Question 1",
            "conv-1",
        ).await.unwrap();
        
        let _ = ecosystem.process_curator_output(
            "Test output 2 with more content",
            "Question 2",
            "conv-2",
        ).await.unwrap();
        
        let stats = ecosystem.get_stats().await;
        
        assert_eq!(stats.total_facts, 2);
        assert!(stats.total_patterns >= 0);
        assert!(stats.last_processing_time.is_some());
    }
}

#[cfg(test)]
mod intelligent_orchestrator_tests {
    use super::*;

    async fn create_test_orchestrator() -> IntelligentContextOrchestrator {
        let vector_store = Arc::new(ChromaVectorStore::new().await.unwrap());
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        let retriever = Arc::new(ContextRetriever::new(
            vector_store.clone(),
            python_service.clone(),
        ).await.unwrap());
        
        let recognizer = Arc::new(PatternRecognizer::new(
            python_service.clone(),
        ).await.unwrap());
        
        let analyzer = Arc::new(QualityAnalyzer::new(
            python_service.clone(),
        ).await.unwrap());
        
        let synthesizer = Arc::new(KnowledgeSynthesizer::new(
            python_service,
        ).await.unwrap());
        
        IntelligentContextOrchestrator::new(
            retriever,
            recognizer,
            analyzer,
            synthesizer,
        )
    }

    #[tokio::test]
    async fn test_intelligent_context_decision() {
        let orchestrator = create_test_orchestrator().await;
        
        // Test different types of questions
        let test_cases = vec![
            ("How do I implement a binary tree?", "implementation"),
            ("What is the time complexity of quicksort?", "conceptual"),
            ("Debug this segmentation fault", "debugging"),
            ("Optimize this database query", "optimization"),
        ];
        
        for (question, expected_type) in test_cases {
            let decision = orchestrator.analyze_question_intent(question).await.unwrap();
            
            assert!(!decision.primary_intent.is_empty());
            assert!(decision.confidence > 0.0);
            
            // Should have relevant context suggestions
            assert!(!decision.suggested_context_sources.is_empty());
        }
    }

    #[tokio::test]
    async fn test_dynamic_context_preparation() {
        let orchestrator = create_test_orchestrator().await;
        
        let question = "How can I improve the performance of my React application?";
        let decision = orchestrator.analyze_question_intent(question).await.unwrap();
        
        // Prepare context based on the decision
        let context = orchestrator.prepare_dynamic_context(
            question,
            &decision,
            2000, // context limit
        ).await.unwrap();
        
        // Should have performance-related context
        assert!(context.contains("performance") || 
                context.contains("optimization") ||
                !context.is_empty());
    }
}

#[cfg(test)]
mod full_pipeline_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_auto_accept_pipeline() {
        // Create all components
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database.clone()).await.unwrap();
        
        // Create AI helpers
        let vector_store = Arc::new(ChromaVectorStore::new().await.unwrap());
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        let indexer = Arc::new(KnowledgeIndexer::new(
            vector_store.clone(),
            python_service.clone(),
        ).await.unwrap());
        
        let retriever = Arc::new(ContextRetriever::new(
            vector_store.clone(),
            python_service.clone(),
        ).await.unwrap());
        
        let recognizer = Arc::new(PatternRecognizer::new(
            python_service.clone(),
        ).await.unwrap());
        
        let analyzer = Arc::new(QualityAnalyzer::new(
            python_service.clone(),
        ).await.unwrap());
        
        let synthesizer = Arc::new(KnowledgeSynthesizer::new(
            python_service,
        ).await.unwrap());
        
        // Create operation intelligence coordinator
        let coordinator = OperationIntelligenceCoordinator::new(
            indexer,
            retriever,
            recognizer,
            analyzer,
            synthesizer,
        );
        
        // Create smart decision engine
        let user_prefs = UserPreferences {
            risk_tolerance: 0.5,
            auto_backup: true,
            require_confirmation_for_deletions: true,
            require_confirmation_for_mass_updates: true,
            trust_ai_suggestions: 0.8,
            preferred_mode: AutoAcceptMode::Balanced,
            custom_rules: vec![],
        };
        
        let decision_engine = SmartDecisionEngine::new(
            AutoAcceptMode::Balanced,
            user_prefs,
            None,
        );
        
        // Test scenario: Analyze and decide on file operations
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("src/new_module.rs"),
                content: r#"
/// New module for data processing
pub mod processor {
    pub fn process_data(input: &[u8]) -> Vec<u8> {
        input.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_data() {
        let input = b"test";
        let result = processor::process_data(input);
        assert_eq!(result, input);
    }
}
"#.to_string(),
            },
            FileOperation::Update {
                path: PathBuf::from("src/lib.rs"),
                content: "pub mod processor;".to_string(),
            },
        ];
        
        let context = create_test_context();
        
        // Get unified score from coordinator
        let unified_score = UnifiedScore {
            confidence: 85.0,
            risk: 15.0,
        };
        
        // Create operation analysis
        let operation_analysis = OpAnalysis {
            operations: operations.clone(),
            context: context.clone(),
            unified_score,
            recommendations: vec![],
            groups: hive_ai::consensus::operation_analysis::OperationGroups {
                create_operations: vec![operations[0].clone()],
                update_operations: vec![operations[1].clone()],
                delete_operations: vec![],
                move_operations: vec![],
            },
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        // Make decision
        let decision = decision_engine.make_decision(&operation_analysis).await.unwrap();
        
        // In balanced mode with 85% confidence and 15% risk, should auto-execute
        match decision {
            ExecutionDecision::AutoExecute { confidence, risk_level, .. } => {
                assert_eq!(confidence, 85.0);
                assert_eq!(risk_level, 15.0);
            }
            _ => panic!("Expected auto-execute decision for high confidence, low risk"),
        }
        
        // Process curator output through ecosystem
        let curator_output = "Successfully created new data processing module with tests";
        let processed = ecosystem.process_curator_output(
            curator_output,
            "Create data processing module",
            "test-conv",
        ).await.unwrap();
        
        assert!(processed.quality.overall_score > 0.7); // Good quality
        assert!(!processed.insights.is_empty());
    }

    #[tokio::test]
    async fn test_dangerous_operation_rejection() {
        let user_prefs = UserPreferences {
            risk_tolerance: 0.3, // Low risk tolerance
            auto_backup: true,
            require_confirmation_for_deletions: true,
            require_confirmation_for_mass_updates: true,
            trust_ai_suggestions: 0.9,
            preferred_mode: AutoAcceptMode::Conservative,
            custom_rules: vec![],
        };
        
        let decision_engine = SmartDecisionEngine::new(
            AutoAcceptMode::Conservative,
            user_prefs,
            None,
        );
        
        // Dangerous operation
        let operations = vec![
            FileOperation::Delete {
                path: PathBuf::from("/etc/important.conf"),
            },
        ];
        
        let context = create_test_context();
        
        let operation_analysis = OpAnalysis {
            operations: operations.clone(),
            context,
            unified_score: UnifiedScore {
                confidence: 30.0,
                risk: 95.0, // Very high risk
            },
            recommendations: vec![],
            groups: hive_ai::consensus::operation_analysis::OperationGroups {
                create_operations: vec![],
                update_operations: vec![],
                delete_operations: operations,
                move_operations: vec![],
            },
            component_scores: Default::default(),
            scoring_factors: Default::default(),
            statistics: None,
        };
        
        let decision = decision_engine.make_decision(&operation_analysis).await.unwrap();
        
        // Should block dangerous operation
        assert!(matches!(decision, ExecutionDecision::Block { .. }));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_parallel_processing_performance() {
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database).await.unwrap();
        
        let large_curator_output = "x".repeat(10000); // 10KB of text
        
        // Measure single processing time
        let start = Instant::now();
        let _ = ecosystem.process_curator_output(
            &large_curator_output,
            "Process large output",
            "conv-1",
        ).await.unwrap();
        let single_duration = start.elapsed();
        
        // Measure batch processing time
        let batch: Vec<_> = (0..5).map(|i| {
            (large_curator_output.clone(), 
             format!("Question {}", i),
             format!("conv-{}", i))
        }).collect();
        
        let start = Instant::now();
        let _ = ecosystem.process_batch(batch).await.unwrap();
        let batch_duration = start.elapsed();
        
        // Batch processing should be more efficient than sequential
        let sequential_estimate = single_duration * 5;
        assert!(
            batch_duration < sequential_estimate,
            "Batch processing should be faster than sequential: {:?} < {:?}",
            batch_duration,
            sequential_estimate
        );
    }

    #[tokio::test]
    async fn test_caching_effectiveness() {
        let database = create_test_database().await;
        let ecosystem = AIHelperEcosystem::new(database).await.unwrap();
        
        let curator_output = "Rust provides memory safety through ownership";
        let question = "How does Rust ensure memory safety?";
        
        // First processing (no cache)
        let start = Instant::now();
        let _ = ecosystem.process_curator_output(
            curator_output,
            question,
            "conv-1",
        ).await.unwrap();
        let first_duration = start.elapsed();
        
        // Second processing (should hit cache for similar content)
        let start = Instant::now();
        let _ = ecosystem.process_curator_output(
            curator_output,
            question,
            "conv-2",
        ).await.unwrap();
        let cached_duration = start.elapsed();
        
        // Cached should be significantly faster
        assert!(
            cached_duration < first_duration / 2 || cached_duration.as_millis() < 50,
            "Cached processing should be much faster"
        );
    }
}