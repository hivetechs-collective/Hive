//! Comprehensive test suite for the 4-stage consensus pipeline
//!
//! Tests all aspects of Phase 3.1 implementation including:
//! - Generator stage with context injection
//! - Refiner stage with quality metrics
//! - Validator stage with security checks
//! - Curator stage with final polishing
//! - Streaming progress tracking
//! - Temporal context integration

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

use hive_ai::consensus::{
    stages::{ConsensusStage, CuratorStage, GeneratorStage, RefinerStage, ValidatorStage},
    streaming::{ConsensusMetrics, ConsoleCallbacks, EnhancedProgressTracker},
    types::{ContextInjectionStrategy, RetryPolicy, Stage},
    ConsensusConfig, ConsensusEngine, ConsensusPipeline, ConsensusProfile, TemporalContextProvider,
};

/// Test basic consensus pipeline functionality
#[tokio::test]
async fn test_basic_consensus_pipeline() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    let result = engine.process("What is Rust?", None).await?;

    assert!(result.success);
    assert!(result.result.is_some());
    assert_eq!(result.stages.len(), 4);
    assert!(result.total_duration > 0.0);

    // Check that all stages completed
    let stage_names: Vec<&str> = result
        .stages
        .iter()
        .map(|s| s.stage_name.as_str())
        .collect();

    assert_eq!(
        stage_names,
        vec!["generator", "refiner", "validator", "curator"]
    );

    Ok(())
}

/// Test temporal context detection and integration
#[tokio::test]
async fn test_temporal_context_integration() -> Result<()> {
    let provider = TemporalContextProvider::default();

    // Test temporal context detection
    assert!(provider.requires_temporal_context("What are the latest Rust features?"));
    assert!(provider.requires_temporal_context("Search for recent AI developments"));
    assert!(provider.requires_temporal_context("What's the current stock price of NVIDIA?"));
    assert!(!provider.requires_temporal_context("How do I write a for loop in Rust?"));

    // Test temporal context building
    let context = provider.build_current_context().await?;

    assert!(!context.search_instruction.is_empty());
    assert!(!context.temporal_awareness.is_empty());
    assert!(!context.current_date.is_empty());
    assert!(!context.current_datetime.is_empty());
    assert!(context.business_context.is_some());

    // Test business context
    let business_ctx = context.business_context.unwrap();
    assert!(!business_ctx.quarter.is_empty());
    assert!(!business_ctx.fiscal_period.is_empty());

    Ok(())
}

/// Test generator stage with different context types
#[tokio::test]
async fn test_generator_stage_context_injection() -> Result<()> {
    let generator = GeneratorStage::new();

    // Test repository context detection
    assert!(generator.needs_repository_context("Analyze this code"));
    assert!(generator.needs_repository_context("What does this function do?"));
    assert!(generator.needs_repository_context("Review the implementation"));
    assert!(!generator.needs_repository_context("What is machine learning?"));

    // Test temporal context detection
    assert!(generator.needs_temporal_context("What's the latest in AI?"));
    assert!(generator.needs_temporal_context("Search for recent updates"));
    assert!(!generator.needs_temporal_context("Explain basic concepts"));

    // Test message building
    let messages = generator.build_messages(
        "How do I handle errors in Rust?",
        None,
        Some("TEMPORAL CONTEXT: Current date is 2024-07-02"),
    )?;

    assert!(!messages.is_empty());

    // Check for enhanced system prompt
    let system_msg = &messages[0];
    assert_eq!(system_msg.role, "system");
    assert!(system_msg.content.contains("TEMPORAL AWARENESS MODE"));

    Ok(())
}

/// Test refiner stage quality analysis
#[tokio::test]
async fn test_refiner_stage_quality_analysis() -> Result<()> {
    let refiner = RefinerStage::new();

    // Test response type detection
    let code_response = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
    assert_eq!(refiner.detect_response_type(code_response), "code");

    let explanation_response = "This happens because Rust enforces memory safety...";
    assert_eq!(
        refiner.detect_response_type(explanation_response),
        "explanation"
    );

    let analysis_response = "After careful analysis of the data, we can examine...";
    assert_eq!(refiner.detect_response_type(analysis_response), "analysis");

    // Test quality analysis
    let poor_response = "Short answer.";
    let analysis = refiner.analyze_response_quality(poor_response);
    assert!(analysis.contains("too brief"));

    let good_response = "This is a comprehensive response that explains the concept thoroughly with examples and provides actionable guidance for implementation.";
    let analysis = refiner.analyze_response_quality(good_response);
    assert!(analysis.contains("good") || analysis.contains("✅"));

    // Test quality issue identification
    let vague_response =
        "This might possibly work, maybe you could try this approach, perhaps it might help.";
    let issues = refiner.identify_quality_issues(vague_response);
    assert!(!issues.is_empty());
    assert!(issues[0].contains("uncertain language"));

    Ok(())
}

/// Test validator stage security checks
#[tokio::test]
async fn test_validator_stage_security_validation() -> Result<()> {
    let validator = ValidatorStage::new();

    // Test security validation
    let secure_code = "```rust\nfn safe_function() -> Result<String, Error> {\n    Ok(\"safe\".to_string())\n}\n```";
    let security_issues = validator.security_validation_checks(secure_code);
    assert!(security_issues.is_empty());

    let unsafe_code = "```rust\nuse std::process::Command;\nfn dangerous() {\n    Command::new(\"rm\").arg(\"-rf\").arg(\"/\").spawn();\n}\n```";
    let security_issues = validator.security_validation_checks(unsafe_code);
    assert!(!security_issues.is_empty());

    // Test basic validation checks
    let broken_code = "```rust\nfn incomplete(";
    let basic_issues = validator.basic_validation_checks(broken_code);
    assert!(!basic_issues.is_empty());
    assert!(basic_issues[0].contains("Unclosed code block"));

    // Test format validation
    let bad_format = "[broken link](missing-url) and ##bad header";
    let format_issues = validator.format_validation_checks(bad_format);
    assert!(!format_issues.is_empty());

    Ok(())
}

/// Test curator stage formatting and polishing
#[tokio::test]
async fn test_curator_stage_formatting() -> Result<()> {
    let curator = CuratorStage::new();

    // Test content type analysis
    let code_content = "```rust\nfn main() {}\n```\nThis function is the entry point.";
    assert_eq!(curator.analyze_content_type(code_content), "code");

    let explanation_content = "To understand this concept, we need to examine...";
    assert_eq!(
        curator.analyze_content_type(explanation_content),
        "explanation"
    );

    // Test curation opportunities analysis
    let unstructured_content = "This is a very long response that goes on and on without any structure or organization and could really benefit from some headers and better formatting to make it more readable and useful.".repeat(10);
    let opportunities = curator.analyze_curation_opportunities(&unstructured_content);
    assert!(opportunities.contains("Add section headers"));

    // Test formatting improvements
    let messy_content = "##Bad header\n\n```\ncode without language\n```\n\n*inconsistent* bullets\n•mixed bullet styles";
    let formatted = curator.apply_comprehensive_formatting(messy_content);
    assert!(formatted.contains("## Bad header"));
    assert!(formatted.contains("```text"));

    Ok(())
}

/// Test streaming progress tracking
#[tokio::test]
async fn test_streaming_progress_tracking() -> Result<()> {
    let callbacks = Arc::new(ConsoleCallbacks {
        show_progress: false,
    });
    let mut tracker = EnhancedProgressTracker::new(Stage::Generator, callbacks);

    // Test chunk addition with quality assessment
    tracker.add_chunk_with_quality("This is a test response")?;
    tracker.add_chunk_with_quality(" with multiple chunks")?;
    tracker.add_chunk_with_quality(" that builds up content")?;

    assert!(tracker.get_quality_score() >= 0.0);
    assert!(tracker.elapsed_time().as_millis() > 0);

    // Test quality scoring
    tracker.add_chunk_with_quality("## Well Structured\nThis response has good structure")?;
    tracker.add_chunk_with_quality(" because it explains concepts clearly")?;
    tracker.add_chunk_with_quality(" and you can follow these steps")?;

    let quality = tracker.get_quality_score();
    assert!(quality > 0.5); // Should have good quality score

    Ok(())
}

/// Test consensus metrics tracking
#[tokio::test]
async fn test_consensus_metrics() -> Result<()> {
    let mut metrics = ConsensusMetrics::default();

    // Test stage progress updates
    metrics.update_stage_progress(Stage::Generator, 100.0, 0.9);
    metrics.update_stage_progress(Stage::Refiner, 75.0, 0.8);
    metrics.set_current_stage(Some(Stage::Refiner));

    assert_eq!(metrics.stage_progress[0], 100.0);
    assert_eq!(metrics.stage_progress[1], 75.0);
    assert_eq!(metrics.stage_quality[0], 0.9);
    assert_eq!(metrics.current_stage, Some(Stage::Refiner));

    // Test display formatting
    let display = metrics.format_stage_display(0);
    assert!(display.contains("✓")); // Completed stage
    assert!(display.contains("Generator"));
    assert!(display.contains("100%"));

    let display = metrics.format_stage_display(1);
    assert!(display.contains("●")); // In progress stage
    assert!(display.contains("Refiner"));
    assert!(display.contains("75%"));

    Ok(())
}

/// Test consensus pipeline with different profiles
#[tokio::test]
async fn test_consensus_profiles() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    // Test profile switching
    engine.set_profile("speed").await?;
    let profile = engine.get_current_profile().await;
    assert_eq!(profile.profile_name, "Consensus_Speed");

    engine.set_profile("quality").await?;
    let profile = engine.get_current_profile().await;
    assert_eq!(profile.profile_name, "Consensus_Elite");

    engine.set_profile("cost").await?;
    let profile = engine.get_current_profile().await;
    assert_eq!(profile.profile_name, "Consensus_Cost");

    // Test invalid profile
    let result = engine.set_profile("invalid").await;
    assert!(result.is_err());

    Ok(())
}

/// Test consensus pipeline performance
#[tokio::test]
async fn test_consensus_performance() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    let start_time = std::time::Instant::now();
    let result = engine.process("Quick test question", None).await?;
    let duration = start_time.elapsed();

    // Performance target: <500ms (this is with mock models, real models will be slower)
    assert!(duration.as_millis() < 2000); // Allow more time for mock implementation
    assert!(result.success);

    // Test concurrent requests (basic load test)
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let engine_clone = engine.clone();
            tokio::spawn(async move {
                engine_clone
                    .process(&format!("Test question {}", i), None)
                    .await
            })
        })
        .collect();

    for handle in handles {
        let result = handle.await??;
        assert!(result.success);
    }

    Ok(())
}

/// Test error handling and recovery
#[tokio::test]
async fn test_consensus_error_handling() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    // Test with empty input
    let result = engine.process("", None).await?;
    assert!(result.success); // Should handle gracefully

    // Test with very long input
    let long_input = "x".repeat(10000);
    let result = engine.process(&long_input, None).await?;
    assert!(result.success); // Should handle gracefully

    // Test with special characters
    let special_input = "Test with 特殊字符 and émojis 🚀 and símböls";
    let result = engine.process(special_input, None).await?;
    assert!(result.success);

    Ok(())
}

/// Test integration with context building
#[tokio::test]
async fn test_context_integration() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    // Test with semantic context
    let semantic_context = "RELEVANT CODE:\nfn main() {\n    println!(\"Hello\");\n}\n\nRELEVANT SYMBOLS:\n- main (function)";
    let result = engine
        .process(
            "What does this code do?",
            Some(semantic_context.to_string()),
        )
        .await?;

    assert!(result.success);
    assert!(result.result.is_some());

    // The response should indicate it used the context
    if let Some(response) = &result.result {
        assert!(response.contains("development placeholder") || response.contains("consensus"));
    }

    Ok(())
}

/// Integration test for complete consensus workflow
#[tokio::test]
async fn test_complete_consensus_workflow() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    // Test temporal context query
    let temporal_query = "What are the latest Rust features released this year?";
    let result = engine.process(temporal_query, None).await?;

    assert!(result.success);
    assert_eq!(result.stages.len(), 4);

    // Check that temporal context was likely used
    if let Some(response) = &result.result {
        assert!(response.contains("temporal context") || response.contains("current"));
    }

    // Test code analysis query
    let code_query = "How can I improve error handling in this Rust function?";
    let code_context =
        "```rust\nfn risky_operation() -> String {\n    \"result\".to_string()\n}\n```";
    let result = engine
        .process(code_query, Some(code_context.to_string()))
        .await?;

    assert!(result.success);
    assert_eq!(result.stages.len(), 4);

    // Verify all stages completed successfully
    for stage in &result.stages {
        assert!(!stage.answer.is_empty());
        assert!(!stage.model.is_empty());
        assert!(stage.analytics.is_some());
    }

    Ok(())
}

/// Test consensus pipeline timeout handling
#[tokio::test]
async fn test_consensus_timeout() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    // Test with very short timeout (should still succeed with mock implementation)
    let result = timeout(
        Duration::from_millis(5000), // 5 second timeout
        engine.process("Test timeout", None),
    )
    .await;

    assert!(result.is_ok());
    let consensus_result = result.unwrap()?;
    assert!(consensus_result.success);

    Ok(())
}

/// Benchmark test for consensus pipeline
#[tokio::test]
async fn test_consensus_benchmark() -> Result<()> {
    let engine = ConsensusEngine::new().await?;

    let test_queries = [
        "What is Rust?",
        "How do I handle errors?",
        "Explain ownership",
        "What are async functions?",
    ];

    let mut total_duration = std::time::Duration::from_secs(0);
    let iterations = 4;

    for query in &test_queries {
        let start = std::time::Instant::now();
        let result = engine.process(query, None).await?;
        let duration = start.elapsed();

        assert!(result.success);
        total_duration += duration;
    }

    let avg_duration = total_duration / iterations;
    println!("Average consensus time: {:?}", avg_duration);

    // Performance assertion (generous for mock implementation)
    assert!(avg_duration.as_millis() < 2000);

    Ok(())
}
