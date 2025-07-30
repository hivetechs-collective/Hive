//! Simple test to see what's happening with consensus storage

use anyhow::Result;
use hive::consensus::engine::ConsensusEngine;
use hive::core::database::{get_database, initialize_database, DatabaseConfig};
use std::path::PathBuf;
use rusqlite::OptionalExtension;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,hive::consensus::pipeline=debug")
        .init();
    
    println!("🐝 Simple Consensus Test");
    println!("========================\n");
    
    // Initialize database
    let db_config = DatabaseConfig {
        path: PathBuf::from("~/.hive/hive-ai.db"),
        ..Default::default()
    };
    
    let _ = initialize_database(Some(db_config)).await;
    let db = get_database().await?;
    
    // Create consensus engine
    println!("Creating consensus engine...");
    let engine = ConsensusEngine::new(Some(db.clone())).await?;
    
    // Run consensus
    println!("\nRunning consensus...");
    match engine.process_with_user("What is 2+2?", None, None).await {
        Ok(result) => {
            println!("\n✅ Consensus completed!");
            println!("Success: {}", result.success);
            println!("Total cost: ${:.8}", result.total_cost);
            println!("Conversation ID: {}", result.conversation_id);
            
            // Check if conversation was stored
            let conn = db.get_connection()?;
            let stored: Option<(String, f64, i32, i32)> = conn.query_row(
                "SELECT id, total_cost, total_tokens_input, total_tokens_output 
                 FROM conversations 
                 WHERE id = ?1",
                [&result.conversation_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            ).optional()?;
            
            match stored {
                Some((id, cost, input, output)) => {
                    println!("\n💾 Conversation stored in database:");
                    println!("ID: {}", id);
                    println!("Cost: ${:.8}", cost);
                    println!("Input tokens: {}", input);
                    println!("Output tokens: {}", output);
                    
                    if cost > 0.0 {
                        println!("\n✅ SUCCESS: Cost tracking is working!");
                    } else {
                        println!("\n❌ FAIL: Cost is 0 in database");
                    }
                }
                None => {
                    println!("\n❌ FAIL: Conversation not found in database!");
                }
            }
        }
        Err(e) => {
            println!("\n❌ Consensus failed: {}", e);
        }
    }
    
    Ok(())
}