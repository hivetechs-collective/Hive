//! Command-line tool to fix database schema mismatches

use hive_ai::core::db_migration::{fix_schema_mismatches, needs_migration};
use hive_ai::core::database::{get_database, initialize_database, DatabaseConfig};
use std::process;
use std::ops::DerefMut;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    println!("🐝 Hive Database Schema Migration Tool");
    println!("=====================================\n");
    
    // Initialize database if needed
    let db_config = DatabaseConfig {
        path: PathBuf::from("~/.hive/hive-ai.db"),
        ..Default::default()
    };
    
    // Try to initialize, ignore if already initialized
    let _ = initialize_database(Some(db_config)).await;
    
    // Get database connection
    let db = match get_database().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            process::exit(1);
        }
    };
    
    // Get a raw connection from the database manager
    let mut conn = match db.get_connection() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("❌ Failed to get database connection: {}", e);
            process::exit(1);
        }
    };
    
    // Check if migration is needed
    match needs_migration(conn.deref_mut()) {
        Ok(true) => {
            println!("📊 Schema migration needed. Starting migration...\n");
        }
        Ok(false) => {
            println!("✅ Database schema is already up to date!");
            return;
        }
        Err(e) => {
            eprintln!("❌ Failed to check schema status: {}", e);
            process::exit(1);
        }
    }
    
    // Run migration
    match fix_schema_mismatches(conn.deref_mut()) {
        Ok(()) => {
            println!("\n🎉 Migration completed successfully!");
            println!("Your database is now ready to store consensus costs properly.");
        }
        Err(e) => {
            eprintln!("\n❌ Migration failed: {}", e);
            process::exit(1);
        }
    }
    
    // Verify the fix
    println!("\n📊 Verifying database schema...");
    match needs_migration(conn.deref_mut()) {
        Ok(false) => {
            println!("✅ Verification successful - schema is correct!");
        }
        Ok(true) => {
            eprintln!("⚠️  Warning: Schema still needs migration");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ Verification failed: {}", e);
            process::exit(1);
        }
    }
}