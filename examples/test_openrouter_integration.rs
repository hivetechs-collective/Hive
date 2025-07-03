//! Test OpenRouter Integration with Consensus Pipeline
//! 
//! This example demonstrates the real OpenRouter API integration
//! working with the 4-stage consensus pipeline.

use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_level(true)
        .with_target(false)
        .init();

    println!("🧪 Testing OpenRouter Integration with Consensus Pipeline");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Check for API key
    if env::var("OPENROUTER_API_KEY").is_err() {
        println!("❌ OPENROUTER_API_KEY environment variable not set");
        println!("   Set your OpenRouter API key to test real integration:");
        println!("   export OPENROUTER_API_KEY=\"your-key-here\"");
        println!();
        println!("🔄 Testing without API key (fallback mode)...");
        return test_fallback_mode().await;
    }

    println!("✅ OpenRouter API key found");
    println!("🚀 Testing real API integration...");
    println!();

    // Test real consensus integration
    test_real_integration().await
}

async fn test_real_integration() -> Result<()> {
    use hive_ai::consensus::ConsensusEngine;

    println!("1️⃣ Creating consensus engine...");
    let engine = ConsensusEngine::new().await?;
    println!("   ✅ Engine created successfully");

    println!();
    println!("2️⃣ Testing profile switching...");
    
    // Test different profiles
    let profiles = vec!["balanced", "speed", "quality", "cost"];
    for profile in &profiles {
        match engine.set_profile(profile).await {
            Ok(()) => {
                let current = engine.get_current_profile().await;
                println!("   ✅ Profile '{}': {}", profile, current.profile_name);
                println!("      Generator: {}", current.generator_model);
                println!("      Curator: {}", current.curator_model);
            }
            Err(e) => {
                println!("   ❌ Profile '{}' failed: {}", profile, e);
            }
        }
    }

    println!();
    println!("3️⃣ Testing simple consensus query...");
    engine.set_profile("balanced").await?;
    
    let test_query = "What is the capital of France?";
    println!("   Query: '{}'", test_query);
    
    match engine.process(test_query, None).await {
        Ok(result) => {
            println!("   ✅ Consensus completed successfully!");
            println!("   Duration: {:.2}s", result.total_duration);
            println!("   Cost: ${:.4}", result.total_cost);
            println!("   Stages: {}", result.stages.len());
            
            if let Some(final_result) = &result.result {
                let preview = if final_result.len() > 100 {
                    format!("{}...", &final_result[..100])
                } else {
                    final_result.clone()
                };
                println!("   Result: {}", preview);
            }
            
            // Verify all 4 stages ran
            if result.stages.len() == 4 {
                println!("   ✅ All 4 stages completed");
                for (i, stage) in result.stages.iter().enumerate() {
                    println!("      {}. {}: {}", i + 1, stage.stage_name, stage.model);
                }
            } else {
                println!("   ⚠️ Expected 4 stages, got {}", result.stages.len());
            }
        }
        Err(e) => {
            println!("   ❌ Consensus failed: {}", e);
            println!("   This might be due to:");
            println!("      • Invalid API key");
            println!("      • Insufficient credits");
            println!("      • Network connectivity issues");
            println!("      • OpenRouter service issues");
            return Err(e);
        }
    }

    println!();
    println!("4️⃣ Testing with semantic context...");
    
    let context = Some("You are analyzing a simple geography question.".to_string());
    let contextual_query = "Is Paris the largest city in France by population?";
    
    match engine.process(contextual_query, context).await {
        Ok(result) => {
            println!("   ✅ Contextual consensus completed!");
            println!("   Duration: {:.2}s", result.total_duration);
            println!("   Cost: ${:.4}", result.total_cost);
        }
        Err(e) => {
            println!("   ❌ Contextual consensus failed: {}", e);
        }
    }

    println!();
    println!("🎉 OpenRouter integration test complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

async fn test_fallback_mode() -> Result<()> {
    use hive_ai::consensus::ConsensusEngine;

    println!("1️⃣ Creating consensus engine (fallback mode)...");
    let engine = ConsensusEngine::new().await?;
    println!("   ✅ Engine created successfully");

    println!();
    println!("2️⃣ Testing fallback consensus...");
    
    let test_query = "What is the capital of France?";
    println!("   Query: '{}'", test_query);
    
    match engine.process(test_query, None).await {
        Ok(result) => {
            println!("   ✅ Fallback consensus completed!");
            println!("   Duration: {:.2}s", result.total_duration);
            println!("   Stages: {}", result.stages.len());
            
            if let Some(final_result) = &result.result {
                let preview = if final_result.len() > 100 {
                    format!("{}...", &final_result[..100])
                } else {
                    final_result.clone()
                };
                println!("   Result: {}", preview);
            }
        }
        Err(e) => {
            println!("   ❌ Fallback consensus failed: {}", e);
            return Err(e);
        }
    }

    println!();
    println!("🎉 Fallback mode test complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}