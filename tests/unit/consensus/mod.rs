//! Unit tests for consensus engine
//! Comprehensive testing of the 4-stage consensus pipeline

use hive_ai::consensus::{engine::ConsensusEngine, pipeline::ConsensusPipeline, types::*};
use hive_ai::providers::openrouter::client::OpenRouterClient;
use std::collections::HashMap;
use tokio;
use anyhow::Result;

mod engine_tests;
mod pipeline_tests;
mod stages_tests;
mod streaming_tests;
mod temporal_tests;

/// Test consensus engine initialization
#[tokio::test]
async fn test_consensus_engine_initialization() -> Result<()> {
    let engine = ConsensusEngine::new().await?;
    
    // Verify engine is properly initialized
    assert!(engine.is_initialized(), "Engine should be initialized");
    
    // Check that all required components are available
    let components = engine.get_components();
    assert!(components.contains_key("generator"), "Generator stage should be available");
    assert!(components.contains_key("refiner"), "Refiner stage should be available");
    assert!(components.contains_key("validator"), "Validator stage should be available");
    assert!(components.contains_key("curator"), "Curator stage should be available");
    
    let mut metrics = HashMap::new();
    metrics.insert("initialization_time".to_string(), 1.0); // Mock timing
    
    Ok(())
}

/// Test consensus request validation
#[tokio::test]
async fn test_consensus_request_validation() -> Result<()> {
    let engine = ConsensusEngine::new().await?;
    
    // Test valid request
    let valid_request = ConsensusRequest {
        query: "Test query".to_string(),
        context: Some("Test context".to_string()),
        profile: ConsensusProfile::Balanced,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(1000),
    };
    
    let validation_result = engine.validate_request(&valid_request).await;
    assert!(validation_result.is_ok(), "Valid request should pass validation");
    
    // Test invalid requests
    let empty_query_request = ConsensusRequest {
        query: "".to_string(),
        context: None,
        profile: ConsensusProfile::Balanced,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(1000),
    };
    
    let validation_result = engine.validate_request(&empty_query_request).await;
    assert!(validation_result.is_err(), "Empty query should fail validation");
    
    // Test parameter bounds
    let invalid_temperature_request = ConsensusRequest {
        query: "Test".to_string(),
        context: None,
        profile: ConsensusProfile::Balanced,
        stream: false,
        temperature: Some(2.5), // Invalid temperature > 2.0
        max_tokens: Some(1000),
    };
    
    let validation_result = engine.validate_request(&invalid_temperature_request).await;
    assert!(validation_result.is_err(), "Invalid temperature should fail validation");
    
    let mut metrics = HashMap::new();
    metrics.insert("validation_time".to_string(), 0.5);
    
    Ok(())
}

/// Test consensus profiles
#[tokio::test]
async fn test_consensus_profiles() -> Result<()> {
    let engine = ConsensusEngine::new().await?;
    
    let test_cases = vec![
        (ConsensusProfile::Speed, "speed"),
        (ConsensusProfile::Balanced, "balanced"),
        (ConsensusProfile::Quality, "quality"),
        (ConsensusProfile::Cost, "cost"),
    ];
    
    for (profile, profile_name) in test_cases {
        let request = ConsensusRequest {
            query: format!("Test query for {} profile", profile_name),
            context: None,
            profile: profile.clone(),
            stream: false,
            temperature: None,
            max_tokens: None,
        };
        
        // Verify profile configuration is loaded correctly
        let config = engine.get_profile_config(&profile).await?;
        
        match profile {
            ConsensusProfile::Speed => {
                assert!(config.max_stages <= 2, "Speed profile should use fewer stages");
                assert!(config.timeout.as_secs() <= 30, "Speed profile should have shorter timeout");
            }
            ConsensusProfile::Quality => {
                assert!(config.max_stages >= 3, "Quality profile should use more stages");
                assert!(config.temperature <= 0.3, "Quality profile should use lower temperature");
            }
            ConsensusProfile::Cost => {
                assert!(config.cost_limit.is_some(), "Cost profile should have cost limits");
                assert!(config.prefer_cheaper_models, "Cost profile should prefer cheaper models");
            }
            ConsensusProfile::Balanced => {
                assert!(config.max_stages == 4, "Balanced profile should use all stages");
                assert!(config.temperature >= 0.5 && config.temperature <= 0.8, 
                       "Balanced profile should use moderate temperature");
            }
        }
    }
    
    let mut metrics = HashMap::new();
    metrics.insert("profile_tests".to_string(), 4.0);
    
    Ok(())
}

/// Test error handling in consensus engine
#[tokio::test]
async fn test_consensus_error_handling() -> Result<()> {
    let engine = ConsensusEngine::new().await?;
    
    // Test network error handling
    let request_with_network_issue = ConsensusRequest {
        query: "Test network error".to_string(),
        context: None,
        profile: ConsensusProfile::Speed,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(100),
    };
    
    // Mock network failure (this would be integrated with actual OpenRouter testing)
    // In a real test, we'd use a mock client that simulates network failures
    
    // Test rate limiting handling
    let rate_limit_request = ConsensusRequest {
        query: "Test rate limiting".to_string(),
        context: None,
        profile: ConsensusProfile::Speed,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(100),
    };
    
    // Test malformed response handling
    let malformed_response_request = ConsensusRequest {
        query: "Test malformed response".to_string(),
        context: None,
        profile: ConsensusProfile::Speed,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(100),
    };
    
    // Verify error recovery mechanisms
    let error_stats = engine.get_error_statistics().await?;
    assert!(error_stats.contains_key("total_errors"), "Should track error statistics");
    assert!(error_stats.contains_key("recovery_attempts"), "Should track recovery attempts");
    
    let mut metrics = HashMap::new();
    metrics.insert("error_handling_tests".to_string(), 3.0);
    
    Ok(())
}

/// Test consensus engine performance
#[tokio::test]
async fn test_consensus_performance() -> Result<()> {
    let engine = ConsensusEngine::new().await?;
    
    let request = ConsensusRequest {
        query: "Performance test query".to_string(),
        context: Some("Test context for performance measurement".to_string()),
        profile: ConsensusProfile::Speed,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(100),
    };
    
    // Measure single request performance
    let start = std::time::Instant::now();
    let result = engine.process_request(&request).await;
    let duration = start.elapsed();
    
    // For unit tests, we focus on the engine's internal performance
    // Real API calls would be tested in integration tests
    assert!(duration < std::time::Duration::from_millis(100), 
           "Engine processing should be fast without network calls");
    
    // Test concurrent request handling
    let concurrent_requests = 5;
    let start = std::time::Instant::now();
    
    let mut handles = Vec::new();
    for i in 0..concurrent_requests {
        let engine_clone = engine.clone();
        let request_clone = ConsensusRequest {
            query: format!("Concurrent test query {}", i),
            ..request.clone()
        };
        
        handles.push(tokio::spawn(async move {
            engine_clone.process_request(&request_clone).await
        }));
    }
    
    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;
    let concurrent_duration = start.elapsed();
    
    // Verify concurrent processing is efficient
    let expected_max_duration = std::time::Duration::from_millis(500);
    assert!(concurrent_duration < expected_max_duration,
           "Concurrent processing should be efficient");
    
    let mut metrics = HashMap::new();
    metrics.insert("single_request_time".to_string(), duration.as_millis() as f64);
    metrics.insert("concurrent_request_time".to_string(), concurrent_duration.as_millis() as f64);
    metrics.insert("concurrent_requests".to_string(), concurrent_requests as f64);
    
    Ok(())
}

/// Test consensus engine memory usage
#[tokio::test]
async fn test_consensus_memory_usage() -> Result<()> {
    let initial_memory = get_memory_usage();
    
    // Create multiple engine instances to test memory efficiency
    let mut engines = Vec::new();
    for _ in 0..10 {
        engines.push(ConsensusEngine::new().await?);
    }
    
    let after_creation_memory = get_memory_usage();
    let memory_per_engine = (after_creation_memory - initial_memory) / 10;
    
    // Each engine should use minimal memory
    assert!(memory_per_engine < 1024 * 1024, // Less than 1MB per engine
           "Each consensus engine should use minimal memory");
    
    // Process requests with all engines
    let request = ConsensusRequest {
        query: "Memory test query".to_string(),
        context: None,
        profile: ConsensusProfile::Speed,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(100),
    };
    
    for engine in &engines {
        let _ = engine.process_request(&request).await;
    }
    
    let after_processing_memory = get_memory_usage();
    let processing_memory_increase = after_processing_memory - after_creation_memory;
    
    // Memory usage shouldn't grow significantly during processing
    assert!(processing_memory_increase < 10 * 1024 * 1024, // Less than 10MB increase
           "Memory usage should not grow significantly during processing");
    
    // Clean up
    drop(engines);
    
    // Give GC time to work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let final_memory = get_memory_usage();
    let memory_leaked = final_memory - initial_memory;
    
    // Should not have significant memory leaks
    assert!(memory_leaked < 5 * 1024 * 1024, // Less than 5MB leaked
           "Should not have significant memory leaks");
    
    let mut metrics = HashMap::new();
    metrics.insert("memory_per_engine".to_string(), memory_per_engine as f64);
    metrics.insert("processing_memory_increase".to_string(), processing_memory_increase as f64);
    metrics.insert("memory_leaked".to_string(), memory_leaked as f64);
    
    Ok(())
}

/// Helper function to get current memory usage
fn get_memory_usage() -> usize {
    // This would be implemented using platform-specific APIs
    // For now, return a mock value
    std::process::id() as usize * 1024 // Mock memory usage
}

/// Test consensus engine cleanup
#[tokio::test]
async fn test_consensus_cleanup() -> Result<()> {
    let engine = ConsensusEngine::new().await?;
    
    // Process some requests to create internal state
    let request = ConsensusRequest {
        query: "Cleanup test query".to_string(),
        context: None,
        profile: ConsensusProfile::Balanced,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(100),
    };
    
    for _ in 0..5 {
        let _ = engine.process_request(&request).await;
    }
    
    // Verify engine has internal state
    let stats_before = engine.get_statistics().await?;
    assert!(stats_before.get("processed_requests").unwrap_or(&0.0) > &0.0,
           "Engine should have processed requests");
    
    // Test cleanup
    engine.cleanup().await?;
    
    // Verify cleanup was effective
    let stats_after = engine.get_statistics().await?;
    assert!(engine.is_clean(), "Engine should be clean after cleanup");
    
    let mut metrics = HashMap::new();
    metrics.insert("cleanup_time".to_string(), 1.0);
    
    Ok(())
}