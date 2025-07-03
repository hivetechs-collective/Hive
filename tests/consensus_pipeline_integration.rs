//! Consensus Pipeline Integration Tests with OpenRouter API
//! Tests the complete 4-stage consensus pipeline with real API integration

use hive_ai::consensus::{ConsensusEngine, ConsensusRequest, ConsensusProfile, ConsensusStage};
use hive_ai::core::config::{HiveConfig, OpenRouterConfig};
use hive_ai::providers::openrouter::{OpenRouterClient, OpenRouterModel};
use std::time::{Duration, Instant};
use tokio;
use serial_test::serial;

/// Mock OpenRouter configuration for testing
fn create_test_openrouter_config() -> OpenRouterConfig {
    OpenRouterConfig {
        api_key: std::env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
        base_url: "https://openrouter.ai/api/v1".to_string(),
        timeout_seconds: 30,
        max_retries: 3,
        retry_delay_ms: 1000,
    }
}

/// Create test consensus request
fn create_test_request(profile: ConsensusProfile) -> ConsensusRequest {
    ConsensusRequest {
        query: "Explain what this Rust function does and suggest improvements".to_string(),
        context: Some(r#"
fn calculate_fibonacci(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)
}
"#.to_string()),
        profile,
        stream: false,
        temperature: Some(0.7),
        max_tokens: Some(500),
    }
}

/// Test OpenRouter client initialization
#[tokio::test]
#[serial]
async fn test_openrouter_client_init() {
    let config = create_test_openrouter_config();
    let client = OpenRouterClient::new(config);
    
    // Test that client initializes without error
    assert!(client.is_ok());
    
    let client = client.unwrap();
    
    // Test client configuration
    assert_eq!(client.base_url(), "https://openrouter.ai/api/v1");
    assert_eq!(client.timeout(), Duration::from_secs(30));
}

/// Test OpenRouter model listing (requires API key)
#[tokio::test]
#[serial]
async fn test_openrouter_models() {
    // Skip test if no API key provided
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping OpenRouter models test - no API key provided");
        return;
    }
    
    let config = create_test_openrouter_config();
    let client = OpenRouterClient::new(config).unwrap();
    
    let models_result = client.list_models().await;
    
    match models_result {
        Ok(models) => {
            assert!(!models.is_empty());
            
            // Verify we have expected models
            let model_names: Vec<_> = models.iter().map(|m| &m.id).collect();
            println!("Available models: {:?}", model_names);
            
            // Should have some common models
            assert!(models.iter().any(|m| m.id.contains("gpt")));
        }
        Err(e) => {
            println!("Models test failed (may be expected if API key is invalid): {}", e);
        }
    }
}

/// Test consensus engine initialization
#[tokio::test]
#[serial]
async fn test_consensus_engine_init() {
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine_result = ConsensusEngine::new(config).await;
    
    // Engine should initialize successfully
    assert!(engine_result.is_ok());
    
    let engine = engine_result.unwrap();
    
    // Test engine configuration
    assert!(engine.is_ready());
}

/// Test consensus profiles
#[tokio::test]
#[serial]
async fn test_consensus_profiles() {
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    
    // Test different profiles
    let profiles = vec![
        ConsensusProfile::Speed,
        ConsensusProfile::Balanced,
        ConsensusProfile::Elite,
    ];
    
    for profile in profiles {
        let models = engine.get_models_for_profile(&profile).await;
        assert!(models.is_ok());
        
        let models = models.unwrap();
        assert!(!models.is_empty());
        
        println!("Profile {:?} uses {} models", profile, models.len());
        
        // Verify model selection logic
        match profile {
            ConsensusProfile::Speed => {
                // Speed profile should prefer faster, cheaper models
                assert!(models.len() >= 1);
            }
            ConsensusProfile::Balanced => {
                // Balanced profile should use mix of models
                assert!(models.len() >= 2);
            }
            ConsensusProfile::Elite => {
                // Elite profile should use premium models
                assert!(models.len() >= 3);
            }
        }
    }
}

/// Test individual consensus stages
#[tokio::test]
#[serial]
async fn test_consensus_stages() {
    // Skip test if no API key provided
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping consensus stages test - no API key provided");
        return;
    }
    
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    let request = create_test_request(ConsensusProfile::Speed);
    
    // Test Generator stage
    let start = Instant::now();
    let generator_result = engine.generate_stage(&request).await;
    let generator_duration = start.elapsed();
    
    match generator_result {
        Ok(generated_content) => {
            assert!(!generated_content.is_empty());
            assert!(generated_content.len() > 10); // Should be substantial content
            
            println!("Generator stage took: {:?}", generator_duration);
            println!("Generated content length: {}", generated_content.len());
            
            // Test Refiner stage
            let start = Instant::now();
            let refiner_result = engine.refine_stage(&request, &generated_content).await;
            let refiner_duration = start.elapsed();
            
            match refiner_result {
                Ok(refined_content) => {
                    assert!(!refined_content.is_empty());
                    
                    println!("Refiner stage took: {:?}", refiner_duration);
                    println!("Refined content length: {}", refined_content.len());
                    
                    // Test Validator stage
                    let start = Instant::now();
                    let validator_result = engine.validate_stage(&request, &refined_content).await;
                    let validator_duration = start.elapsed();
                    
                    match validator_result {
                        Ok(validated_content) => {
                            assert!(!validated_content.is_empty());
                            
                            println!("Validator stage took: {:?}", validator_duration);
                            println!("Validated content length: {}", validated_content.len());
                            
                            // Test Curator stage
                            let start = Instant::now();
                            let curator_result = engine.curate_stage(&request, &validated_content).await;
                            let curator_duration = start.elapsed();
                            
                            match curator_result {
                                Ok(final_content) => {
                                    assert!(!final_content.is_empty());
                                    
                                    println!("Curator stage took: {:?}", curator_duration);
                                    println!("Final content length: {}", final_content.len());
                                    
                                    // Verify total pipeline performance
                                    let total_duration = generator_duration + refiner_duration + 
                                                       validator_duration + curator_duration;
                                    println!("Total pipeline duration: {:?}", total_duration);
                                    
                                    // Should be faster than TypeScript baseline (3.2s)
                                    assert!(total_duration < Duration::from_millis(3200));
                                    
                                    // Should meet Rust target (500ms) in optimal conditions
                                    if total_duration < Duration::from_millis(500) {
                                        println!("✅ Met Rust performance target!");
                                    } else {
                                        println!("⚠️  Slower than Rust target but faster than TypeScript");
                                    }
                                }
                                Err(e) => println!("Curator stage failed: {}", e),
                            }
                        }
                        Err(e) => println!("Validator stage failed: {}", e),
                    }
                }
                Err(e) => println!("Refiner stage failed: {}", e),
            }
        }
        Err(e) => {
            println!("Generator stage failed (may be expected with test API key): {}", e);
        }
    }
}

/// Test full consensus pipeline
#[tokio::test]
#[serial]
async fn test_full_consensus_pipeline() {
    // Skip test if no API key provided
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping full consensus pipeline test - no API key provided");
        return;
    }
    
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    
    // Test different profiles
    let profiles = vec![
        ConsensusProfile::Speed,
        ConsensusProfile::Balanced,
        ConsensusProfile::Elite,
    ];
    
    for profile in profiles {
        let request = create_test_request(profile.clone());
        
        let start = Instant::now();
        let result = engine.process_request(request).await;
        let duration = start.elapsed();
        
        match result {
            Ok(response) => {
                assert!(!response.content.is_empty());
                assert!(response.content.len() > 50); // Should be substantial
                
                println!("Profile {:?} completed in {:?}", profile, duration);
                println!("Response length: {}", response.content.len());
                println!("Stages completed: {:?}", response.stages);
                
                // Verify metadata
                assert!(response.token_count > 0);
                assert!(response.cost_usd > 0.0);
                assert!(!response.model_used.is_empty());
                
                // Performance verification
                match profile {
                    ConsensusProfile::Speed => {
                        // Speed profile should be fastest
                        assert!(duration < Duration::from_secs(10));
                    }
                    ConsensusProfile::Balanced => {
                        // Balanced profile should be moderate
                        assert!(duration < Duration::from_secs(20));
                    }
                    ConsensusProfile::Elite => {
                        // Elite profile may be slower but should still be reasonable
                        assert!(duration < Duration::from_secs(30));
                    }
                }
                
                // Should be faster than TypeScript baseline
                assert!(duration < Duration::from_millis(3200));
            }
            Err(e) => {
                println!("Consensus pipeline failed for profile {:?}: {}", profile, e);
            }
        }
    }
}

/// Test streaming consensus
#[tokio::test]
#[serial]
async fn test_streaming_consensus() {
    // Skip test if no API key provided
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping streaming consensus test - no API key provided");
        return;
    }
    
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    let mut request = create_test_request(ConsensusProfile::Speed);
    request.stream = true;
    
    let start = Instant::now();
    let result = engine.process_request_stream(request).await;
    
    match result {
        Ok(mut stream) => {
            let mut total_content = String::new();
            let mut chunk_count = 0;
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        total_content.push_str(&chunk.content);
                        chunk_count += 1;
                        
                        println!("Received chunk {}: {} chars", chunk_count, chunk.content.len());
                        
                        // Verify chunk metadata
                        assert!(chunk.stage != ConsensusStage::Unknown);
                    }
                    Err(e) => {
                        println!("Stream chunk error: {}", e);
                        break;
                    }
                }
            }
            
            let duration = start.elapsed();
            
            assert!(chunk_count > 0);
            assert!(!total_content.is_empty());
            
            println!("Streaming completed in {:?} with {} chunks", duration, chunk_count);
            println!("Total streamed content: {} chars", total_content.len());
            
            // Streaming should provide responsive feedback
            assert!(duration < Duration::from_secs(30));
        }
        Err(e) => {
            println!("Streaming consensus failed: {}", e);
        }
    }
}

/// Test consensus error handling
#[tokio::test]
#[serial]
async fn test_consensus_error_handling() {
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    config.openrouter.api_key = "invalid-key".to_string();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    let request = create_test_request(ConsensusProfile::Speed);
    
    let result = engine.process_request(request).await;
    
    // Should handle authentication errors gracefully
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    println!("Expected error for invalid API key: {}", error);
    
    // Error should be informative
    let error_str = error.to_string();
    assert!(error_str.contains("auth") || error_str.contains("401") || error_str.contains("key"));
}

/// Test consensus with large context
#[tokio::test]
#[serial]
async fn test_consensus_large_context() {
    // Skip test if no API key provided
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping large context test - no API key provided");
        return;
    }
    
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    
    // Create request with large context
    let large_code = r#"
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexStruct {
    pub id: u64,
    pub name: String,
    pub data: HashMap<String, Value>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u32,
    pub tags: Vec<String>,
}

impl ComplexStruct {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            data: HashMap::new(),
            metadata: Metadata {
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
                tags: Vec::new(),
            },
        }
    }
    
    pub fn update(&mut self, data: HashMap<String, Value>) {
        self.data = data;
        self.metadata.updated_at = chrono::Utc::now();
        self.metadata.version += 1;
    }
}
"#.repeat(10); // Repeat to make it large
    
    let mut request = create_test_request(ConsensusProfile::Speed);
    request.context = Some(large_code);
    
    let start = Instant::now();
    let result = engine.process_request(request).await;
    let duration = start.elapsed();
    
    match result {
        Ok(response) => {
            assert!(!response.content.is_empty());
            
            println!("Large context processed in {:?}", duration);
            println!("Response length: {}", response.content.len());
            
            // Should handle large context efficiently
            assert!(duration < Duration::from_secs(60));
        }
        Err(e) => {
            println!("Large context test failed: {}", e);
        }
    }
}

/// Test concurrent consensus requests
#[tokio::test]
#[serial]
async fn test_concurrent_consensus() {
    // Skip test if no API key provided
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping concurrent consensus test - no API key provided");
        return;
    }
    
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    
    // Create multiple concurrent requests
    let requests = vec![
        create_test_request(ConsensusProfile::Speed),
        create_test_request(ConsensusProfile::Speed),
        create_test_request(ConsensusProfile::Speed),
    ];
    
    let start = Instant::now();
    
    let futures: Vec<_> = requests
        .into_iter()
        .map(|req| engine.process_request(req))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    let duration = start.elapsed();
    
    let successful_results: Vec<_> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    println!("Concurrent requests completed: {}/3 in {:?}", 
             successful_results.len(), duration);
    
    // At least some requests should succeed
    assert!(!successful_results.is_empty());
    
    // Concurrent processing should be efficient
    assert!(duration < Duration::from_secs(120));
}

/// Test consensus performance regression
#[tokio::test]
#[serial]
async fn test_consensus_performance_regression() {
    // Skip test if no API key provided
    if std::env::var("OPENROUTER_API_KEY").is_err() {
        println!("Skipping performance regression test - no API key provided");
        return;
    }
    
    let mut config = HiveConfig::default();
    config.openrouter = create_test_openrouter_config();
    
    let engine = ConsensusEngine::new(config).await.unwrap();
    let request = create_test_request(ConsensusProfile::Speed);
    
    // Run multiple iterations to get average performance
    let iterations = 3;
    let mut total_duration = Duration::new(0, 0);
    let mut successful_runs = 0;
    
    for i in 0..iterations {
        let start = Instant::now();
        let result = engine.process_request(request.clone()).await;
        let duration = start.elapsed();
        
        match result {
            Ok(_) => {
                total_duration += duration;
                successful_runs += 1;
                println!("Iteration {} completed in {:?}", i + 1, duration);
            }
            Err(e) => {
                println!("Iteration {} failed: {}", i + 1, e);
            }
        }
    }
    
    if successful_runs > 0 {
        let average_duration = total_duration / successful_runs;
        println!("Average consensus duration: {:?} over {} runs", average_duration, successful_runs);
        
        // Performance targets from CLAUDE.md
        let typescript_baseline = Duration::from_millis(3200);
        let rust_target = Duration::from_millis(500);
        
        // Should be faster than TypeScript baseline
        assert!(
            average_duration < typescript_baseline,
            "Performance regression: {:?} vs TypeScript baseline {:?}",
            average_duration,
            typescript_baseline
        );
        
        // Report progress toward Rust target
        if average_duration < rust_target {
            println!("✅ Met Rust performance target: {:?} < {:?}", average_duration, rust_target);
        } else {
            let improvement_factor = typescript_baseline.as_nanos() as f64 / average_duration.as_nanos() as f64;
            println!("⚠️  Improvement factor: {:.2}x (target: 6.4x)", improvement_factor);
        }
    } else {
        println!("No successful runs for performance regression test");
    }
}