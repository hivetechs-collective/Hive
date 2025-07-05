//! Direct test of quickstart database setup
//! 
//! This bypasses migrations to test the schema creation directly

use anyhow::Result;
use std::path::PathBuf;
use rusqlite::{Connection, params};

fn create_schema(conn: &Connection) -> Result<()> {
    // Create consensus_profiles table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS consensus_profiles (
            id TEXT PRIMARY KEY,
            profile_name TEXT NOT NULL UNIQUE,
            pipeline_profile_id TEXT,
            generator_model TEXT NOT NULL,
            refiner_model TEXT NOT NULL,
            validator_model TEXT NOT NULL,
            curator_model TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    // Create consensus_settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS consensus_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Testing quickstart database setup (direct)...\n");
    
    // Create test database
    let db_path = PathBuf::from("/tmp/hive_quickstart_test.db");
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
    }
    
    // Open connection
    let conn = Connection::open(&db_path)?;
    
    // Create schema
    println!("📊 Creating database schema...");
    create_schema(&conn)?;
    
    // Create default profiles
    println!("🎯 Creating default consensus profiles...");
    
    let profiles = vec![
        (
            "Consensus_Elite",
            "anthropic/claude-3-opus-20240229",
            "openai/gpt-4o",
            "anthropic/claude-3-opus-20240229",
            "openai/gpt-4o",
        ),
        (
            "Consensus_Balanced",
            "anthropic/claude-3-5-sonnet-20241022",
            "openai/gpt-4-turbo",
            "anthropic/claude-3-opus-20240229",
            "openai/gpt-4o",
        ),
        (
            "Consensus_Speed",
            "anthropic/claude-3-haiku-20240307",
            "openai/gpt-3.5-turbo",
            "anthropic/claude-3-haiku-20240307",
            "openai/gpt-3.5-turbo",
        ),
        (
            "Consensus_Cost",
            "meta-llama/llama-3.2-3b-instruct",
            "mistralai/mistral-7b-instruct",
            "meta-llama/llama-3.2-3b-instruct",
            "mistralai/mistral-7b-instruct",
        ),
    ];
    
    for (name, generator, refiner, validator, curator) in &profiles {
        conn.execute(
            "INSERT INTO consensus_profiles (
                id, profile_name, generator_model, refiner_model, 
                validator_model, curator_model
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &uuid::Uuid::new_v4().to_string(),
                name,
                generator,
                refiner,
                validator,
                curator,
            ],
        )?;
        println!("  ✅ Created profile: {}", name);
    }
    
    // Set default active profile
    conn.execute(
        "INSERT OR REPLACE INTO consensus_settings (key, value) 
         VALUES ('active_profile', 'Consensus_Balanced')",
        [],
    )?;
    println!("\n✅ Set Consensus_Balanced as active profile");
    
    // Verify profiles
    println!("\n📋 Verifying profiles:");
    {
        let mut stmt = conn.prepare(
            "SELECT profile_name, generator_model, refiner_model 
             FROM consensus_profiles 
             ORDER BY profile_name"
        )?;
        
        let profile_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        
        for profile in profile_iter {
            let (name, gen, ref_model) = profile?;
            println!("  • {}", name);
            println!("    - Generator: {}", gen);
            println!("    - Refiner: {}", ref_model);
        }
    }
    
    // Check active profile
    use rusqlite::OptionalExtension;
    let active: Option<String> = conn.query_row(
        "SELECT value FROM consensus_settings WHERE key = 'active_profile'",
        [],
        |row| row.get(0),
    ).optional()?;
    
    if let Some(active_name) = active {
        println!("\n🎯 Active profile: {}", active_name);
    }
    
    println!("\n🎉 Quickstart database test completed successfully!");
    
    // Clean up
    drop(conn);
    std::fs::remove_file(&db_path)?;
    
    Ok(())
}