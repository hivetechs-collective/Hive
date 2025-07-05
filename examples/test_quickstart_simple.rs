//! Simple test for quickstart functionality
//! 
//! This tests the basic database setup without complex queries

use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up clean test database
    let db_path = PathBuf::from("/tmp/hive_test.db");
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
    }
    
    println!("🚀 Testing quickstart database setup...\n");
    
    // Set database path
    std::env::set_var("HIVE_DB_PATH", db_path.to_str().unwrap());
    
    // Initialize database with custom config
    use hive_ai::core::database::{DatabaseConfig, DatabaseManager};
    
    let config = DatabaseConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    
    println!("📊 Creating database at {:?}", db_path);
    let db = DatabaseManager::new(config).await?;
    
    // Get connection and check tables
    let conn = db.get_connection()?;
    
    // Check if consensus_profiles table exists
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='consensus_profiles'",
        [],
        |row| Ok(row.get::<_, i32>(0)? > 0),
    )?;
    
    println!("✅ consensus_profiles table exists: {}", table_exists);
    
    // Create default profiles
    use hive_ai::database::schema::create_default_profiles;
    println!("\n🎯 Creating default consensus profiles...");
    create_default_profiles(&conn)?;
    
    // Count profiles
    let profile_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM consensus_profiles",
        [],
        |row| row.get(0),
    )?;
    
    println!("✅ Created {} consensus profiles", profile_count);
    
    // List profiles
    println!("\n📋 Available profiles:");
    {
        let mut stmt = conn.prepare("SELECT profile_name, generator_model FROM consensus_profiles ORDER BY profile_name")?;
        let profiles = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
            ))
        })?;
        
        for profile in profiles {
            let (name, model) = profile?;
            println!("  • {} (Generator: {})", name, model);
        }
    } // stmt is dropped here
    
    // Check active profile
    use rusqlite::OptionalExtension;
    let active_profile: Option<String> = conn.query_row(
        "SELECT value FROM consensus_settings WHERE key = 'active_profile'",
        [],
        |row| row.get(0),
    ).optional()?;
    
    if let Some(active) = active_profile {
        println!("\n✅ Active profile: {}", active);
    }
    
    println!("\n🎉 Quickstart test completed successfully!");
    
    // Clean up
    drop(conn);
    drop(db);
    std::fs::remove_file(&db_path)?;
    
    Ok(())
}