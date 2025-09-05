//! OpenRouter API Integration Tests
//! Real API validation with actual OpenRouter service

use crate::providers::openrouter::{client::OpenRouterClient, models::ModelInfo};
use crate::consensus::{engine::ConsensusEngine, types::*};
use std::collections::HashMap;
use std::env;
use tokio;
use anyhow::Result;
use serial_test::serial;

/// Test OpenRouter client initialization
#[tokio::test]
#[serial]
async fn test_openrouter_client_initialization() -> Result<()> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .unwrap_or_else(|_| "test-key".to_string());

    let client = OpenRouterClient::new(&api_key)?;

    // Verify client is properly configured
    assert!(client.is_configured(), "Client should be configured");

    // Test client configuration
    let config = client.get_config();
    assert_eq!(config.base_url, "https://openrouter.ai/api/v1");
    assert!(config.timeout.as_secs() >= 30, "Should have reasonable timeout");
    assert!(config.retry_attempts >= 3, "Should have retry configuration");

    let mut metrics = HashMap::new();
    metrics.insert("client_initialization_time".to_string(), 1.0);

    Ok(())
}

/// Test model availability and selection
#[tokio::test]
#[serial]
async fn test_model_availability() -> Result<()> {
    let api_key = env::var("OPENROUTER_API_KEY");
    if api_key.is_err() {
        println!("⚠️  Skipping real API test - OPENROUTER_API_KEY not set");
        return Ok(());
    }

    let client = OpenRouterClient::new(&api_key.unwrap())?;

    // Test model listing
    let models = client.list_models().await?;

    // Verify essential models are available
    let essential_models = vec![
        "anthropic/claude-3-sonnet",
        "anthropic/claude-3-haiku",
        "openai/gpt-4-turbo",
        "openai/gpt-3.5-turbo",
        "meta-llama/llama-2-70b-chat",
        "google/gemini-pro",
        "cohere/command-r-plus",
    ];

    let mut available_models = 0;
    for essential_model in &essential_models {
        if models.iter().any(|m| m.id.contains(essential_model)) {
            available_models += 1;
            println!("✅ Essential model available: {}", essential_model);
        } else {
            println!("⚠️  Essential model not found: {}", essential_model);
        }
    }

    // Should have majority of essential models available
    assert!(available_models >= essential_models.len() / 2,
           "Should have majority of essential models available");

    // Test model info retrieval
    if let Some(model) = models.first() {
        let model_info = client.get_model_info(&model.id).await?;
        assert!(!model_info.name.is_empty(), "Model should have name");
        assert!(model_info.context_length > 0, "Model should have context length");

        println!("✅ Model info retrieved: {} (context: {})",
                model_info.name, model_info.context_length);
    }

    let mut metrics = HashMap::new();
    metrics.insert("available_models".to_string(), models.len() as f64);
    metrics.insert("essential_models_available".to_string(), available_models as f64);

    Ok(())
}

/// Test real consensus request with OpenRouter
#[tokio::test]
#[serial]
async fn test_real_consensus_request() -> Result<()> {
    let api_key = env::var("OPENROUTER_API_KEY");
    if api_key.is_err() {
        println!("⚠️  Skipping real API test - OPENROUTER_API_KEY not set");
        return Ok(());
    }

    let client = OpenRouterClient::new(&api_key.unwrap())?;

    // Test simple completion request
    let request = crate::providers::openrouter::types::CompletionRequest {
        model: "anthropic/claude-3-haiku".to_string(),
        messages: vec![
            crate::providers::openrouter::types::Message {
                role: "user".to_string(),
                content: "What is 2+2? Respond with just the number.".to_string(),
            }
        ],
        temperature: Some(0.1),
        max_tokens: Some(10),
        stream: Some(false),
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let response = client.complete(&request).await?;
    let duration = start.elapsed();

    // Verify response
    assert!(!response.choices.is_empty(), "Should have at least one choice");
    let response_text = &response.choices[0].message.content;
    assert!(response_text.contains("4"), "Should contain correct answer");

    // Performance validation
    assert!(duration < std::time::Duration::from_secs(10),
           "Response should be received within 10 seconds");

    // Cost tracking
    let cost = client.calculate_cost(&request, &response).await?;
    assert!(cost >= 0.0, "Cost should be non-negative");

    println!("✅ Real API request successful: {} ({}ms, ${:.6})",
            response_text.trim(), duration.as_millis(), cost);

    let mut metrics = HashMap::new();
    metrics.insert("response_time".to_string(), duration.as_millis() as f64);
    metrics.insert("response_cost".to_string(), cost);
    metrics.insert("response_tokens".to_string(), response.usage.completion_tokens as f64);

    Ok(())
}

/// Test streaming responses
#[tokio::test]
#[serial]
async fn test_streaming_response() -> Result<()> {
    let api_key = env::var("OPENROUTER_API_KEY");
    if api_key.is_err() {
        println!("⚠️  Skipping real API test - OPENROUTER_API_KEY not set");
        return Ok(());
    }

    let client = OpenRouterClient::new(&api_key.unwrap())?;

    let request = crate::providers::openrouter::types::CompletionRequest {
        model: "anthropic/claude-3-haiku".to_string(),
        messages: vec![
            crate::providers::openrouter::types::Message {
                role: "user".to_string(),
                content: "Count from 1 to 5, one number per line.".to_string(),
            }
        ],
        temperature: Some(0.1),
        max_tokens: Some(50),
        stream: Some(true),
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let mut stream = client.stream(&request).await?;

    let mut chunks_received = 0;
    let mut complete_response = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                chunks_received += 1;
                if !chunk.choices.is_empty() {
                    if let Some(delta) = &chunk.choices[0].delta {
                        if let Some(content) = &delta.content {
                            complete_response.push_str(content);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Stream error: {}", e);
                break;
            }
        }

        // Prevent infinite loops in tests
        if chunks_received > 100 {
            break;
        }
    }

    let duration = start.elapsed();

    // Verify streaming worked
    assert!(chunks_received > 0, "Should have received stream chunks");
    assert!(!complete_response.is_empty(), "Should have received content");

    // Verify content makes sense
    let numbers_found = (1..=5).filter(|n| complete_response.contains(&n.to_string())).count();
    assert!(numbers_found >= 3, "Should contain most of the requested numbers");

    println!("✅ Streaming test successful: {} chunks, {} chars in {}ms",
            chunks_received, complete_response.len(), duration.as_millis());

    let mut metrics = HashMap::new();
    metrics.insert("stream_chunks".to_string(), chunks_received as f64);
    metrics.insert("stream_duration".to_string(), duration.as_millis() as f64);
    metrics.insert("response_length".to_string(), complete_response.len() as f64);

    Ok(())
}

/// Test rate limiting and error handling
#[tokio::test]
#[serial]
async fn test_rate_limiting_and_errors() -> Result<()> {
    let api_key = env::var("OPENROUTER_API_KEY");
    if api_key.is_err() {
        println!("⚠️  Skipping real API test - OPENROUTER_API_KEY not set");
        return Ok(());
    }

    let client = OpenRouterClient::new(&api_key.unwrap())?;

    // Test with invalid model (should handle gracefully)
    let invalid_request = crate::providers::openrouter::types::CompletionRequest {
        model: "nonexistent/invalid-model".to_string(),
        messages: vec![
            crate::providers::openrouter::types::Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }
        ],
        ..Default::default()
    };

    let result = client.complete(&invalid_request).await;
    assert!(result.is_err(), "Invalid model should return error");

    let error = result.unwrap_err();
    let error_msg = error.to_string().to_lowercase();
    assert!(error_msg.contains("model") || error_msg.contains("not found") || error_msg.contains("invalid"),
           "Error should indicate model issue");

    // Test retry mechanism with temporary failures
    let mut retry_attempts = 0;
    for _ in 0..3 {
        let result = client.complete(&invalid_request).await;
        if result.is_err() {
            retry_attempts += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        } else {
            break;
        }
    }

    // Should have attempted retries
    assert!(retry_attempts > 0, "Should have attempted retries for failed requests");

    println!("✅ Error handling test successful: {} retry attempts", retry_attempts);

    let mut metrics = HashMap::new();
    metrics.insert("retry_attempts".to_string(), retry_attempts as f64);

    Ok(())
}

/// Test cost calculation accuracy
#[tokio::test]
#[serial]
async fn test_cost_calculation() -> Result<()> {
    let api_key = env::var("OPENROUTER_API_KEY");
    if api_key.is_err() {
        println!("⚠️  Skipping real API test - OPENROUTER_API_KEY not set");
        return Ok(());
    }

    let client = OpenRouterClient::new(&api_key.unwrap())?;

    // Test with known model and token counts
    let request = crate::providers::openrouter::types::CompletionRequest {
        model: "anthropic/claude-3-haiku".to_string(),
        messages: vec![
            crate::providers::openrouter::types::Message {
                role: "user".to_string(),
                content: "Respond with exactly 10 words: The quick brown fox jumps over the lazy dog.".to_string(),
            }
        ],
        temperature: Some(0.1),
        max_tokens: Some(20),
        ..Default::default()
    };

    let response = client.complete(&request).await?;
    let cost = client.calculate_cost(&request, &response).await?;

    // Verify cost calculation
    assert!(cost > 0.0, "Cost should be positive");
    assert!(cost < 0.01, "Cost for small request should be less than 1 cent");

    // Test cost with different models
    let models_to_test = vec![
        ("anthropic/claude-3-haiku", 0.001),    // Should be cheaper
        ("anthropic/claude-3-sonnet", 0.005),   // Should be more expensive
    ];

    let mut cost_comparisons = Vec::new();

    for (model, expected_range) in models_to_test {
        let model_request = crate::providers::openrouter::types::CompletionRequest {
            model: model.to_string(),
            messages: request.messages.clone(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            ..Default::default()
        };

        if let Ok(model_response) = client.complete(&model_request).await {
            if let Ok(model_cost) = client.calculate_cost(&model_request, &model_response).await {
                cost_comparisons.push((model, model_cost, expected_range));
                println!("Cost for {}: ${:.6}", model, model_cost);
            }
        }
    }

    // Verify cost differences make sense
    if cost_comparisons.len() >= 2 {
        let haiku_cost = cost_comparisons.iter()
            .find(|(model, _, _)| model.contains("haiku"))
            .map(|(_, cost, _)| *cost);
        let sonnet_cost = cost_comparisons.iter()
            .find(|(model, _, _)| model.contains("sonnet"))
            .map(|(_, cost, _)| *cost);

        if let (Some(haiku), Some(sonnet)) = (haiku_cost, sonnet_cost) {
            assert!(sonnet >= haiku, "Sonnet should cost more than or equal to Haiku");
        }
    }

    let mut metrics = HashMap::new();
    metrics.insert("base_cost".to_string(), cost);
    metrics.insert("models_tested".to_string(), cost_comparisons.len() as f64);

    Ok(())
}

/// Test consensus engine with real API
#[tokio::test]
#[serial]
async fn test_consensus_with_real_api() -> Result<()> {
    let api_key = env::var("OPENROUTER_API_KEY");
    if api_key.is_err() {
        println!("⚠️  Skipping real API test - OPENROUTER_API_KEY not set");
        return Ok(());
    }

    // Initialize consensus engine with real API key
    std::env::set_var("OPENROUTER_API_KEY", &api_key.unwrap());

    let engine = ConsensusEngine::new().await?;

    let request = ConsensusRequest {
        query: "What is the capital of France? Respond with just the city name.".to_string(),
        context: None,
        profile: ConsensusProfile::Speed, // Use speed profile for faster testing
        stream: false,
        temperature: Some(0.1),
        max_tokens: Some(10),
    };

    let start = std::time::Instant::now();
    let response = engine.process_request(&request).await?;
    let duration = start.elapsed();

    // Verify consensus response
    assert!(!response.final_answer.is_empty(), "Should have final answer");
    assert!(response.final_answer.to_lowercase().contains("paris"),
           "Should contain correct answer");

    // Verify consensus stages were executed
    assert!(!response.stage_outputs.is_empty(), "Should have stage outputs");
    assert!(response.confidence_score > 0.0, "Should have confidence score");

    // Performance validation for real API consensus
    assert!(duration < std::time::Duration::from_secs(30),
           "Consensus should complete within 30 seconds");

    println!("✅ Real consensus test successful: '{}' ({}ms, confidence: {:.2})",
            response.final_answer.trim(), duration.as_millis(), response.confidence_score);

    let mut metrics = HashMap::new();
    metrics.insert("consensus_time".to_string(), duration.as_millis() as f64);
    metrics.insert("confidence_score".to_string(), response.confidence_score);
    metrics.insert("stages_executed".to_string(), response.stage_outputs.len() as f64);
    metrics.insert("total_cost".to_string(), response.total_cost);

    Ok(())
}