//! End-to-end integration tests for AI Helper Ecosystem
//! 
//! This test suite validates that all AI helpers work together correctly
//! and that the complete flow from curator output to processed knowledge works.

use std::sync::Arc;
use anyhow::Result;
use tokio::time::{timeout, Duration};

use crate::ai_helpers::{
    AIHelperEcosystem, 
    IndexedKnowledge, 
    ProcessedKnowledge,
    HelperStats,
    PatternType,
    InsightType,
    Severity,
};
use crate::core::database::DatabaseManager;

/// Test the complete AI helper ecosystem flow
#[tokio::test]
async fn test_ai_helper_ecosystem_e2e() -> Result<()> {
    // Initialize database
    let db = Arc::new(DatabaseManager::new_in_memory().await?);
    
    // Create AI helper ecosystem
    let helpers = timeout(
        Duration::from_secs(30),
        AIHelperEcosystem::new(db.clone())
    ).await??;
    
    // Test data - simulate curator output
    let curator_output = r#"
    The Repository Intelligence system provides deep codebase analysis through multiple layers:
    
    1. **Semantic Analysis**: Uses CodeBERT to understand code relationships
    2. **Pattern Recognition**: Identifies recurring patterns using UniXcoder
    3. **Quality Assessment**: Evaluates code quality with CodeT5+
    4. **Knowledge Synthesis**: Combines insights using local LLMs
    
    This approach enables the consensus system to have complete understanding of the
    current repository context, allowing for more accurate and context-aware responses.
    
    The system maintains a persistent knowledge base that grows with each interaction,
    creating a comprehensive understanding of the project structure and patterns.
    "#;
    
    let source_question = "How does the Repository Intelligence system work?";
    let conversation_id = "test_conversation_123";
    
    println!("🧪 Testing AI Helper Ecosystem end-to-end flow...");
    
    // Process curator output through all helpers
    let start_time = std::time::Instant::now();
    let processed = helpers
        .process_curator_output(curator_output, source_question, conversation_id)
        .await?;
    let processing_time = start_time.elapsed();
    
    println!("✅ Processing completed in {:?}", processing_time);
    
    // Validate the processed knowledge
    validate_processed_knowledge(&processed)?;
    
    // Test stage-specific context preparation
    test_stage_contexts(&helpers, source_question).await?;
    
    // Test batch processing
    test_batch_processing(&helpers).await?;
    
    // Test system statistics
    test_system_statistics(&helpers).await?;
    
    // Test performance under load
    test_performance_load(&helpers).await?;
    
    println!("🎉 All end-to-end tests passed successfully!");
    
    Ok(())
}

/// Validate the structure and content of processed knowledge
fn validate_processed_knowledge(processed: &ProcessedKnowledge) -> Result<()> {
    println!("🔍 Validating processed knowledge structure...");
    
    // Check indexed knowledge
    assert!(!processed.indexed.id.is_empty(), "Indexed knowledge should have ID");
    assert!(!processed.indexed.content.is_empty(), "Indexed knowledge should have content");
    assert!(!processed.indexed.embedding.is_empty(), "Indexed knowledge should have embedding");
    assert!(processed.indexed.embedding.len() > 100, "Embedding should be substantial");
    
    // Check patterns
    println!("📊 Found {} patterns", processed.patterns.len());
    for pattern in &processed.patterns {
        assert!(!pattern.description.is_empty(), "Pattern should have description");
        assert!(pattern.confidence > 0.0, "Pattern should have positive confidence");
        assert!(pattern.confidence <= 1.0, "Pattern confidence should be <= 1.0");
    }
    
    // Check quality report
    let quality = &processed.quality;
    assert!(quality.overall_score >= 0.0, "Overall score should be non-negative");
    assert!(quality.overall_score <= 1.0, "Overall score should be <= 1.0");
    assert!(quality.consistency_score >= 0.0, "Consistency score should be non-negative");
    assert!(quality.completeness_score >= 0.0, "Completeness score should be non-negative");
    assert!(quality.accuracy_score >= 0.0, "Accuracy score should be non-negative");
    
    println!("📈 Quality scores: Overall={:.2}, Consistency={:.2}, Completeness={:.2}, Accuracy={:.2}",
             quality.overall_score, quality.consistency_score, 
             quality.completeness_score, quality.accuracy_score);
    
    // Check insights
    println!("💡 Found {} insights", processed.insights.len());
    for insight in &processed.insights {
        assert!(!insight.content.is_empty(), "Insight should have content");
        assert!(insight.confidence > 0.0, "Insight should have positive confidence");
        assert!(!insight.supporting_facts.is_empty(), "Insight should have supporting facts");
    }
    
    println!("✅ Processed knowledge validation passed");
    Ok(())
}

/// Test stage-specific context preparation
async fn test_stage_contexts(helpers: &AIHelperEcosystem, question: &str) -> Result<()> {
    println!("🎭 Testing stage-specific context preparation...");
    
    use crate::consensus::types::Stage;
    
    let stages = [Stage::Generator, Stage::Refiner, Stage::Validator, Stage::Curator];
    
    for stage in stages {
        let context = helpers
            .prepare_stage_context(question, stage, 1000)
            .await?;
        
        // Validate stage context
        assert_eq!(context.stage, stage, "Context should match requested stage");
        assert!(!context.relevant_facts.is_empty(), "Context should have relevant facts");
        
        // Stage-specific validations
        match stage {
            Stage::Generator => {
                assert!(context.custom_guidance.is_some(), "Generator should have custom guidance");
                if let Some(guidance) = &context.custom_guidance {
                    assert!(guidance.contains("comprehensive"), "Generator guidance should mention comprehensive approach");
                }
            }
            Stage::Refiner => {
                if let Some(guidance) = &context.custom_guidance {
                    assert!(guidance.contains("improving"), "Refiner guidance should mention improving");
                }
            }
            Stage::Validator => {
                if let Some(guidance) = &context.custom_guidance {
                    assert!(guidance.contains("critical"), "Validator guidance should mention being critical");
                }
            }
            Stage::Curator => {
                if let Some(guidance) = &context.custom_guidance {
                    assert!(guidance.contains("synthesizing"), "Curator guidance should mention synthesizing");
                }
            }
        }
        
        println!("✅ Stage {:?} context preparation passed", stage);
    }
    
    Ok(())
}

/// Test batch processing functionality
async fn test_batch_processing(helpers: &AIHelperEcosystem) -> Result<()> {
    println!("📦 Testing batch processing...");
    
    let batch_data = vec![
        (
            "First response about code structure".to_string(),
            "How is the code organized?".to_string(),
            "batch_conv_1".to_string(),
        ),
        (
            "Second response about testing patterns".to_string(),
            "What testing patterns are used?".to_string(),
            "batch_conv_2".to_string(),
        ),
        (
            "Third response about performance optimization".to_string(),
            "How to optimize performance?".to_string(),
            "batch_conv_3".to_string(),
        ),
    ];
    
    let start_time = std::time::Instant::now();
    let batch_results = helpers.process_batch(batch_data.clone()).await?;
    let batch_time = start_time.elapsed();
    
    println!("✅ Batch processing completed in {:?}", batch_time);
    
    // Validate batch results
    assert_eq!(batch_results.len(), batch_data.len(), "Batch results should match input size");
    
    for (i, result) in batch_results.iter().enumerate() {
        assert!(!result.indexed.id.is_empty(), "Batch result {} should have ID", i);
        assert!(!result.indexed.content.is_empty(), "Batch result {} should have content", i);
        assert!(!result.indexed.embedding.is_empty(), "Batch result {} should have embedding", i);
    }
    
    println!("✅ Batch processing validation passed");
    Ok(())
}

/// Test system statistics
async fn test_system_statistics(helpers: &AIHelperEcosystem) -> Result<()> {
    println!("📊 Testing system statistics...");
    
    let stats = helpers.get_stats().await;
    
    // Validate statistics
    assert!(stats.total_facts > 0, "Should have processed some facts");
    assert!(stats.parallel_speedup > 0.0, "Should have positive parallel speedup");
    assert!(stats.tasks_completed > 0, "Should have completed some tasks");
    
    println!("📈 System Statistics:");
    println!("  Total facts: {}", stats.total_facts);
    println!("  Total patterns: {}", stats.total_patterns);
    println!("  Vector store size: {}", stats.vector_store_size);
    println!("  Parallel speedup: {:.2}x", stats.parallel_speedup);
    println!("  Tasks completed: {}", stats.tasks_completed);
    println!("  Tasks failed: {}", stats.tasks_failed);
    
    if let Some(perf_stats) = &stats.performance_stats {
        println!("  Performance operations: {}", perf_stats.total_operations);
        println!("  Success rate: {:.2}%", perf_stats.success_rate * 100.0);
    }
    
    println!("✅ System statistics validation passed");
    Ok(())
}

/// Test performance under load
async fn test_performance_load(helpers: &AIHelperEcosystem) -> Result<()> {
    println!("🚀 Testing performance under load...");
    
    let load_test_data = (0..10)
        .map(|i| {
            (
                format!("Load test response {} with substantial content about various topics", i),
                format!("Load test question {}?", i),
                format!("load_test_conv_{}", i),
            )
        })
        .collect::<Vec<_>>();
    
    let start_time = std::time::Instant::now();
    let load_results = helpers.process_batch(load_test_data.clone()).await?;
    let load_time = start_time.elapsed();
    
    println!("✅ Load test completed in {:?}", load_time);
    println!("📊 Throughput: {:.2} items/second", 
             load_test_data.len() as f64 / load_time.as_secs_f64());
    
    // Validate load test results
    assert_eq!(load_results.len(), load_test_data.len(), "Load test results should match input");
    
    // Check that quality didn't degrade under load
    let avg_quality: f64 = load_results.iter()
        .map(|r| r.quality.overall_score)
        .sum::<f64>() / load_results.len() as f64;
    
    assert!(avg_quality > 0.5, "Average quality should remain reasonable under load");
    
    println!("📈 Average quality under load: {:.2}", avg_quality);
    println!("✅ Performance load test passed");
    
    Ok(())
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    println!("🚨 Testing error handling and recovery...");
    
    // Initialize database
    let db = Arc::new(DatabaseManager::new_in_memory().await?);
    
    // Create AI helper ecosystem
    let helpers = AIHelperEcosystem::new(db.clone()).await?;
    
    // Test with invalid input
    let invalid_output = "";
    let result = helpers
        .process_curator_output(invalid_output, "test question", "test_conv")
        .await;
    
    // Should either handle gracefully or return appropriate error
    match result {
        Ok(processed) => {
            println!("✅ Empty input handled gracefully");
            assert!(processed.indexed.content.is_empty() || processed.indexed.content == "");
        }
        Err(e) => {
            println!("✅ Empty input properly rejected: {}", e);
        }
    }
    
    // Test with extremely long input
    let long_output = "a".repeat(100000);
    let result = helpers
        .process_curator_output(&long_output, "test question", "test_conv")
        .await;
    
    // Should handle without crashing
    assert!(result.is_ok(), "Should handle long input gracefully");
    
    println!("✅ Error handling tests passed");
    Ok(())
}

/// Test concurrent access
#[tokio::test]
async fn test_concurrent_access() -> Result<()> {
    println!("🔄 Testing concurrent access...");
    
    // Initialize database
    let db = Arc::new(DatabaseManager::new_in_memory().await?);
    
    // Create AI helper ecosystem
    let helpers = Arc::new(AIHelperEcosystem::new(db.clone()).await?);
    
    // Create multiple concurrent tasks
    let mut tasks = Vec::new();
    
    for i in 0..5 {
        let helpers_clone = helpers.clone();
        let task = tokio::spawn(async move {
            let output = format!("Concurrent test output {}", i);
            let question = format!("Concurrent test question {}?", i);
            let conv_id = format!("concurrent_conv_{}", i);
            
            helpers_clone
                .process_curator_output(&output, &question, &conv_id)
                .await
        });
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let results = futures::future::try_join_all(tasks).await?;
    
    // Validate all results
    for (i, result) in results.into_iter().enumerate() {
        let processed = result?;
        assert!(!processed.indexed.id.is_empty(), "Concurrent result {} should have ID", i);
        assert!(!processed.indexed.content.is_empty(), "Concurrent result {} should have content", i);
    }
    
    println!("✅ Concurrent access test passed");
    Ok(())
}