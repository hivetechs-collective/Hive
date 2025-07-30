//! Test consensus cost tracking

use anyhow::Result;
use hive::consensus::engine::ConsensusEngine;
use hive::core::database::{get_database, initialize_database, DatabaseConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    println!("🐝 Testing Consensus Cost Tracking");
    println!("==================================\n");
    
    // Initialize database
    let db_config = DatabaseConfig {
        path: PathBuf::from("~/.hive/hive-ai.db"),
        ..Default::default()
    };
    
    // Initialize database if not already initialized
    let _ = initialize_database(Some(db_config)).await;
    
    // Get database
    let db = get_database().await?;
    
    // Count conversations before
    let count_before: i64 = {
        let conn = db.get_connection()?;
        conn.query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))?
    };
    println!("Conversations before: {}", count_before);
    
    // Create consensus engine
    println!("\nCreating consensus engine...");
    let engine = ConsensusEngine::new(Some(db.clone())).await?;
    
    // Run consensus
    println!("\nRunning consensus for: 'What is 2+2?'");
    let result = engine.process_with_user("What is 2+2?", None, None).await?;
    
    println!("\nConsensus completed!");
    println!("Success: {}", result.success);
    println!("Total cost: ${:.8}", result.total_cost);
    println!("Total duration: {:.2}s", result.total_duration);
    
    // Count conversations after
    let count_after: i64 = {
        let conn = db.get_connection()?;
        conn.query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))?
    };
    println!("\nConversations after: {}", count_after);
    
    // Get the latest conversation
    let (id, cost, input_tokens, output_tokens): (String, f64, i32, i32) = {
        let conn = db.get_connection()?;
        conn.query_row(
            "SELECT id, total_cost, total_tokens_input, total_tokens_output 
             FROM conversations 
             ORDER BY created_at DESC 
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        )?
    };
    
    println!("\nLatest conversation:");
    println!("ID: {}", id);
    println!("Total cost: ${:.8}", cost);
    println!("Input tokens: {}", input_tokens);
    println!("Output tokens: {}", output_tokens);
    
    if cost > 0.0 {
        println!("\n✅ Cost tracking is working correctly!");
    } else {
        println!("\n❌ Cost is still 0 - check the consensus pipeline");
    }
    
    Ok(())
}