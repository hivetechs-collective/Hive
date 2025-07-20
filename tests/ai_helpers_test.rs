// Unit tests for AI Helpers
use hive_ai::ai_helpers::{
    KnowledgeIndexer, IndexedOperation, OperationSimilarity, OperationSuccessPrediction,
    ContextRetriever, OperationContextAnalysis, OperationPrecedent,
    PatternRecognizer, SafetyPatternAnalysis, OperationSafetyPattern,
    QualityAnalyzer, QualityMetrics, OperationRiskAssessment,
    KnowledgeSynthesizer,
    ChromaVectorStore,
    ParallelProcessor, ParallelConfig,
    PythonModelService, PythonModelConfig,
    KnowledgeIndexerScore, ContextRetrieverScore, PatternRecognizerScore,
    QualityAnalyzerScore, KnowledgeSynthesizerScore,
};
use hive_ai::consensus::operation_intelligence::{OperationContext, OperationOutcome};
use hive_ai::consensus::stages::file_aware_curator::FileOperation;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Helper to create test operation context
fn create_test_context() -> OperationContext {
    OperationContext {
        repository_path: PathBuf::from("/test/repo"),
        git_commit: Some("abc123".to_string()),
        source_question: "Test question".to_string(),
        related_files: vec![PathBuf::from("src/main.rs")],
        project_metadata: HashMap::new(),
    }
}

/// Helper to create test operations
fn create_test_operations() -> Vec<FileOperation> {
    vec![
        FileOperation::Create {
            path: PathBuf::from("src/new_file.rs"),
            content: "pub fn hello() { println!(\"Hello!\"); }".to_string(),
        },
        FileOperation::Update {
            path: PathBuf::from("src/main.rs"),
            content: "fn main() { hello(); }".to_string(),
        },
    ]
}

/// Helper to create a successful operation outcome
fn create_successful_outcome() -> OperationOutcome {
    OperationOutcome {
        success: true,
        error_message: None,
        execution_time: std::time::Duration::from_millis(100),
        rollback_required: false,
        user_feedback: None,
        post_operation_quality: None,
    }
}

#[cfg(test)]
mod knowledge_indexer_tests {
    use super::*;

    async fn create_test_indexer() -> KnowledgeIndexer {
        let vector_store = Arc::new(ChromaVectorStore::new().await.unwrap());
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        KnowledgeIndexer::new(vector_store, python_service).await.unwrap()
    }

    #[tokio::test]
    async fn test_knowledge_indexer_creation() {
        let _indexer = create_test_indexer().await;
        // Successfully created means test passes
    }

    #[tokio::test]
    async fn test_index_curator_output() {
        let indexer = create_test_indexer().await;
        
        let curator_output = "The function calculates the sum of two numbers.";
        let source_question = "What does this function do?";
        let conversation_id = "test-conv-1";

        let result = indexer.index_output(
            curator_output,
            source_question,
            conversation_id,
        ).await;
        
        assert!(result.is_ok());
        let indexed = result.unwrap();
        assert!(!indexed.id.is_empty());
        assert_eq!(indexed.content, curator_output);
    }

    #[tokio::test]
    async fn test_index_operation_outcome() {
        let indexer = create_test_indexer().await;
        
        let operation = FileOperation::Create {
            path: PathBuf::from("test.rs"),
            content: "fn test() {}".to_string(),
        };
        let context = create_test_context();
        let outcome = create_successful_outcome();

        let result = indexer.index_operation_outcome(
            &operation,
            &context,
            &outcome,
            None,
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_similar_operations() {
        let indexer = create_test_indexer().await;
        
        // First index an operation
        let operation = FileOperation::Create {
            path: PathBuf::from("test.rs"),
            content: "fn hello() {}".to_string(),
        };
        let context = create_test_context();
        let outcome = create_successful_outcome();
        
        indexer.index_operation_outcome(&operation, &context, &outcome, None).await.unwrap();

        // Now search for similar
        let similar_op = FileOperation::Create {
            path: PathBuf::from("test2.rs"),
            content: "fn hello_world() {}".to_string(),
        };
        
        let results = indexer.find_similar_operations(&similar_op, &context, 5).await.unwrap();
        
        // Should find at least our indexed operation
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_predict_operation_success() {
        let indexer = create_test_indexer().await;
        
        let operation = FileOperation::Create {
            path: PathBuf::from("src/lib.rs"),
            content: "pub mod test;".to_string(),
        };
        let context = create_test_context();

        let prediction = indexer.predict_operation_success(&operation, &context).await.unwrap();
        
        assert!(prediction.success_probability >= 0.0 && prediction.success_probability <= 1.0);
    }
}

#[cfg(test)]
mod context_retriever_tests {
    use super::*;

    async fn create_test_retriever() -> ContextRetriever {
        let vector_store = Arc::new(ChromaVectorStore::new().await.unwrap());
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        ContextRetriever::new(vector_store, python_service).await.unwrap()
    }

    #[tokio::test]
    async fn test_context_retriever_creation() {
        let _retriever = create_test_retriever().await;
    }

    #[tokio::test]
    async fn test_analyze_operation_context() {
        let retriever = create_test_retriever().await;
        
        let operations = create_test_operations();
        let context = create_test_context();

        let analysis = retriever.analyze_operation_context(&operations, &context).await.unwrap();
        
        assert!(!analysis.relevant_precedents.is_empty() || analysis.success_rate_analysis.is_some());
    }

    #[tokio::test]
    async fn test_get_generator_context() {
        let retriever = create_test_retriever().await;
        
        let question = "How do I implement a binary search tree?";
        let context_limit = 1000;

        let stage_context = retriever.get_generator_context(question, context_limit).await.unwrap();
        
        assert!(!stage_context.relevant_facts.is_empty() || stage_context.custom_guidance.is_some());
    }
}

#[cfg(test)]
mod pattern_recognizer_tests {
    use super::*;

    async fn create_test_recognizer() -> PatternRecognizer {
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        PatternRecognizer::new(python_service).await.unwrap()
    }

    #[tokio::test]
    async fn test_pattern_recognizer_creation() {
        let _recognizer = create_test_recognizer().await;
    }

    #[tokio::test]
    async fn test_analyze_operation_safety() {
        let recognizer = create_test_recognizer().await;
        
        let operations = create_test_operations();
        let context = create_test_context();

        let analysis = recognizer.analyze_operation_safety(&operations, &context).await.unwrap();
        
        assert!(analysis.overall_safety_score >= 0.0 && analysis.overall_safety_score <= 1.0);
    }

    #[tokio::test]
    async fn test_dangerous_pattern_detection() {
        let recognizer = create_test_recognizer().await;
        
        // Create a potentially dangerous operation
        let operations = vec![
            FileOperation::Delete {
                path: PathBuf::from("/etc/important.conf"),
            }
        ];
        let context = create_test_context();

        let analysis = recognizer.analyze_operation_safety(&operations, &context).await.unwrap();
        
        // Should detect danger
        assert!(analysis.overall_safety_score < 0.5);
        assert!(!analysis.detected_patterns.is_empty());
    }
}

#[cfg(test)]
mod quality_analyzer_tests {
    use super::*;

    async fn create_test_analyzer() -> QualityAnalyzer {
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        QualityAnalyzer::new(python_service).await.unwrap()
    }

    #[tokio::test]
    async fn test_quality_analyzer_creation() {
        let _analyzer = create_test_analyzer().await;
    }

    #[tokio::test]
    async fn test_analyze_code_quality() {
        let analyzer = create_test_analyzer().await;
        
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("src/good_code.rs"),
                content: r#"
/// Well-documented function
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_sum() {
        assert_eq!(calculate_sum(2, 3), 5);
    }
}
"#.to_string(),
            }
        ];

        let metrics = analyzer.analyze_code_quality(&operations).await.unwrap();
        
        // Good code should have high quality score
        assert!(metrics.overall_quality_score > 0.7);
    }

    #[tokio::test]
    async fn test_assess_operation_risk() {
        let analyzer = create_test_analyzer().await;
        
        let operations = create_test_operations();
        let context = create_test_context();

        let assessment = analyzer.assess_operation_risk(&operations, &context).await.unwrap();
        
        assert!(assessment.overall_risk_score >= 0.0 && assessment.overall_risk_score <= 1.0);
    }
}

#[cfg(test)]
mod knowledge_synthesizer_tests {
    use super::*;

    async fn create_test_synthesizer() -> KnowledgeSynthesizer {
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        KnowledgeSynthesizer::new(python_service).await.unwrap()
    }

    #[tokio::test]
    async fn test_knowledge_synthesizer_creation() {
        let _synthesizer = create_test_synthesizer().await;
    }

    #[tokio::test]
    async fn test_synthesize_insights() {
        let synthesizer = create_test_synthesizer().await;
        
        let curator_output = "This function implements a binary search algorithm";
        let question = "How does binary search work?";
        
        let synthesis = synthesizer.synthesize_insights(
            curator_output,
            question,
            "test-conv",
        ).await.unwrap();
        
        assert!(!synthesis.key_insights.is_empty());
        assert!(!synthesis.recommendations.is_empty());
    }
}

#[cfg(test)]
mod parallel_processor_tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_processor_creation() {
        let config = ParallelConfig::default();
        let _processor = ParallelProcessor::new(config);
    }

    #[tokio::test]
    async fn test_process_parallel() {
        let config = ParallelConfig {
            max_concurrent_tasks: 4,
            enable_monitoring: true,
            timeout_seconds: 30,
        };
        let processor = ParallelProcessor::new(config);
        
        // Create mock helpers
        let vector_store = Arc::new(ChromaVectorStore::new().await.unwrap());
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        let recognizer = Arc::new(PatternRecognizer::new(python_service.clone()).await.unwrap());
        let analyzer = Arc::new(QualityAnalyzer::new(python_service.clone()).await.unwrap());
        let synthesizer = Arc::new(KnowledgeSynthesizer::new(python_service).await.unwrap());
        
        let indexed = hive_ai::ai_helpers::IndexedKnowledge {
            id: "test".to_string(),
            content: "Test content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: serde_json::json!({}),
        };
        
        let result = processor.process_parallel(
            &indexed,
            "curator output",
            recognizer,
            analyzer,
            synthesizer,
        ).await.unwrap();
        
        assert!(result.parallel_speedup > 0.0);
    }
}

#[cfg(test)]
mod score_tests {
    use super::*;

    #[test]
    fn test_knowledge_indexer_score() {
        let score = KnowledgeIndexerScore {
            indexed_facts: 10,
            embeddings_generated: 10,
            similarity_scores: vec![0.8, 0.7, 0.9],
            knowledge_coverage: 0.85,
        };
        
        let helper_score = score.to_helper_score();
        assert!(helper_score.confidence > 80.0);
        assert!(helper_score.risk < 20.0);
    }

    #[test]
    fn test_context_retriever_score() {
        let score = ContextRetrieverScore {
            relevant_contexts: 5,
            context_quality: 0.9,
            retrieval_accuracy: 0.95,
            context_diversity: 0.8,
        };
        
        let helper_score = score.to_helper_score();
        assert!(helper_score.confidence > 85.0);
    }

    #[test]
    fn test_pattern_recognizer_score() {
        let score = PatternRecognizerScore {
            patterns_detected: 3,
            pattern_confidence: 0.85,
            safety_score: 0.9,
            anti_patterns_found: 0,
        };
        
        let helper_score = score.to_helper_score();
        assert!(helper_score.risk < 15.0);
    }

    #[test]
    fn test_quality_analyzer_score() {
        let score = QualityAnalyzerScore {
            code_quality: 0.8,
            documentation_quality: 0.9,
            test_coverage: 0.75,
            complexity_score: 0.85,
        };
        
        let helper_score = score.to_helper_score();
        assert!(helper_score.confidence > 75.0);
    }

    #[test]
    fn test_knowledge_synthesizer_score() {
        let score = KnowledgeSynthesizerScore {
            insights_generated: 4,
            synthesis_quality: 0.9,
            coherence_score: 0.95,
            recommendation_relevance: 0.85,
        };
        
        let helper_score = score.to_helper_score();
        assert!(helper_score.confidence > 85.0);
    }
}

#[cfg(test)]
mod vector_store_tests {
    use super::*;

    #[tokio::test]
    async fn test_chroma_vector_store_creation() {
        let store = ChromaVectorStore::new().await;
        assert!(store.is_ok());
    }

    #[tokio::test]
    async fn test_vector_store_operations() {
        let store = ChromaVectorStore::new().await.unwrap();
        
        // Add some entries
        let id1 = "test1";
        let embedding1 = vec![0.1, 0.2, 0.3];
        let metadata1 = serde_json::json!({
            "type": "function",
            "language": "rust"
        });
        
        store.add(id1, &embedding1, &metadata1).await.unwrap();
        
        // Search for similar
        let query = vec![0.15, 0.25, 0.35];
        let results = store.search(&query, 5).await.unwrap();
        
        // Should find our entry
        assert!(!results.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_helpers_working_together() {
        // Create shared resources
        let vector_store = Arc::new(ChromaVectorStore::new().await.unwrap());
        let python_config = PythonModelConfig::default();
        let python_service = Arc::new(PythonModelService::new(python_config).await.unwrap());
        
        // Initialize helpers
        let indexer = KnowledgeIndexer::new(
            vector_store.clone(),
            python_service.clone(),
        ).await.unwrap();
        
        let retriever = ContextRetriever::new(
            vector_store.clone(),
            python_service.clone(),
        ).await.unwrap();
        
        let recognizer = PatternRecognizer::new(
            python_service.clone(),
        ).await.unwrap();
        
        let analyzer = QualityAnalyzer::new(
            python_service.clone(),
        ).await.unwrap();
        
        let synthesizer = KnowledgeSynthesizer::new(
            python_service,
        ).await.unwrap();
        
        // Test scenario: Index some knowledge and analyze operations
        let curator_output = "The binary search algorithm has O(log n) time complexity";
        let question = "What is the time complexity of binary search?";
        let conversation_id = "test-conv";
        
        // Index the knowledge
        let indexed = indexer.index_output(
            curator_output,
            question,
            conversation_id,
        ).await.unwrap();
        
        // Create test operations
        let operations = vec![
            FileOperation::Create {
                path: PathBuf::from("src/binary_search.rs"),
                content: r#"
pub fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();
    
    while left < right {
        let mid = left + (right - left) / 2;
        match arr[mid].cmp(target) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }
    None
}
"#.to_string(),
            }
        ];
        let context = create_test_context();
        
        // Analyze with all helpers
        let context_analysis = retriever.analyze_operation_context(&operations, &context).await.unwrap();
        let safety_analysis = recognizer.analyze_operation_safety(&operations, &context).await.unwrap();
        let quality_metrics = analyzer.analyze_code_quality(&operations).await.unwrap();
        let risk_assessment = analyzer.assess_operation_risk(&operations, &context).await.unwrap();
        
        // Synthesize insights
        let synthesis = synthesizer.synthesize_insights(
            &indexed.content,
            question,
            conversation_id,
        ).await.unwrap();
        
        // Verify we got comprehensive analysis
        assert!(safety_analysis.overall_safety_score > 0.8); // Binary search is safe
        assert!(quality_metrics.overall_quality_score > 0.7); // Well-written code
        assert!(risk_assessment.overall_risk_score < 0.3); // Low risk
        assert!(!synthesis.key_insights.is_empty());
    }
}