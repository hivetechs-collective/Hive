use hive_ai::consensus::engine::ConsensusEngine;
use hive_ai::consensus::streaming::{StreamingCallbacks, ProgressInfo};
use hive_ai::consensus::types::{Stage, StageResult};
use anyhow::Result;
use std::sync::Arc;

struct TestStreamingCallbacks;

impl StreamingCallbacks for TestStreamingCallbacks {
    fn on_stage_start(&self, stage: Stage, model: &str) -> Result<()> {
        println!("\n[TEST] Stage {} starting with model: {}", stage.display_name(), model);
        Ok(())
    }
    
    fn on_stage_chunk(&self, stage: Stage, chunk: &str, total_content: &str) -> Result<()> {
        println!("[TEST] Stage {} chunk: '{}'", stage.display_name(), chunk);
        println!("[TEST] Total content length: {}", total_content.len());
        Ok(())
    }
    
    fn on_stage_progress(&self, stage: Stage, progress: ProgressInfo) -> Result<()> {
        println!("[TEST] Stage {} progress: {}% ({} tokens)", 
                 stage.display_name(), 
                 (progress.percentage * 100.0) as u8,
                 progress.tokens);
        Ok(())
    }
    
    fn on_stage_complete(&self, stage: Stage, _result: &StageResult) -> Result<()> {
        println!("[TEST] Stage {} complete", stage.display_name());
        Ok(())
    }
    
    fn on_error(&self, stage: Stage, error: &anyhow::Error) -> Result<()> {
        println!("[TEST] Stage {} error: {}", stage.display_name(), error);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("Starting streaming test...");
    
    // Create database
    use hive_ai::core::database::{DatabaseManager, DatabaseConfig};
    let db = Arc::new(DatabaseManager::new(DatabaseConfig::default()).await?);
    
    // Create consensus engine
    let engine = match ConsensusEngine::new(Some(db)).await {
        Ok(e) => e,
        Err(e) => {
            println!("Failed to create engine: {}", e);
            println!("Make sure you have:");
            println!("1. OpenRouter API key configured");
            println!("2. Consensus profiles in database");
            return Err(e);
        }
    };
    
    // Create test callbacks
    let callbacks = Arc::new(TestStreamingCallbacks);
    
    // Process a simple query with streaming
    println!("\nProcessing query with streaming callbacks...");
    match engine.process_with_callbacks(
        "What is 2+2?", 
        None,
        callbacks
    ).await {
        Ok(result) => {
            println!("\nFinal result: {:?}", result.result);
            println!("Total cost: ${:.4}", result.total_cost);
            println!("Total duration: {:.2}s", result.total_duration);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    
    Ok(())
}