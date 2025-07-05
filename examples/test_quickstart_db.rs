//! Test quickstart database initialization
//! 
//! This example tests that the quickstart command properly initializes
//! the database with consensus profiles.

use anyhow::Result;
use hive_ai::core::database::{initialize_database, get_database, ConsensusProfile};
use hive_ai::database::schema::create_default_profiles;
use rusqlite::OptionalExtension;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("hive=debug")
        .init();

    println!("🚀 Testing quickstart database initialization...\n");

    // Initialize database
    println!("📊 Initializing database...");
    initialize_database(None).await?;
    
    // Get database connection
    let db = get_database().await?;
    let conn = db.get_connection()?;
    
    // Create default profiles
    println!("🎯 Creating default consensus profiles...");
    create_default_profiles(&conn)?;
    
    // List all profiles
    println!("\n📋 Available consensus profiles:");
    let profiles = ConsensusProfile::list_all().await?;
    
    for profile in &profiles {
        println!("  • {} - Models:", profile.profile_name);
        println!("    - Generator: {}", profile.generator_model);
        println!("    - Refiner: {}", profile.refiner_model);
        println!("    - Validator: {}", profile.validator_model);
        println!("    - Curator: {}", profile.curator_model);
        println!();
    }
    
    // Check active profile
    let active_profile_name: Option<String> = conn
        .query_row(
            "SELECT value FROM consensus_settings WHERE key = 'active_profile'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    
    if let Some(active) = active_profile_name {
        println!("✅ Active profile: {}", active);
    } else {
        println!("❌ No active profile set");
    }
    
    // Test database statistics
    let stats = db.get_statistics().await?;
    println!("\n📊 Database statistics:");
    println!("  • Consensus profiles: {}", stats.profile_count);
    println!("  • Conversations: {}", stats.conversation_count);
    println!("  • Messages: {}", stats.message_count);
    println!("  • OpenRouter models: {}", stats.model_count);
    
    println!("\n✅ Quickstart database initialization successful!");
    
    Ok(())
}