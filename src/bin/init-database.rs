use anyhow::Result;
use hive_ai::core::database::{
    get_database, initialize_database, ConsensusProfile, DatabaseManager,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("🐝 Initializing Hive database...");

    // Initialize the database
    match initialize_database(None).await {
        Ok(_) => {
            println!("✅ Database initialized successfully at ~/.hive/hive-ai.db");

            // Try to get the database to verify it's working
            match get_database().await {
                Ok(db) => {
                    println!("✅ Database connection verified");

                    // Run migrations
                    println!("📦 Running database migrations...");
                    let conn = db.get_connection()?;

                    // The migrations should run automatically on init
                    println!("✅ Migrations completed");

                    // Create default profiles if they don't exist
                    println!("👤 Setting up default consensus profiles...");
                    setup_default_profiles(db).await?;

                    println!("🎉 Database initialization complete!");
                }
                Err(e) => {
                    println!("❌ Failed to verify database connection: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to initialize database: {}", e);
            println!("Please check permissions for ~/.hive directory");
        }
    }

    Ok(())
}

async fn setup_default_profiles(_db: Arc<DatabaseManager>) -> Result<()> {
    // Check if profiles already exist
    if let Ok(profiles) = ConsensusProfile::list_all().await {
        if !profiles.is_empty() {
            println!("📋 Found {} existing profiles", profiles.len());
            return Ok(());
        }
    }

    // Create default profiles
    let profiles = vec![
        (
            "balanced-performer",
            "anthropic/claude-3-5-sonnet-20241022",
            "anthropic/claude-3-5-sonnet-20241022",
            "anthropic/claude-3-5-sonnet-20241022",
            "anthropic/claude-3-5-sonnet-20241022",
        ),
        (
            "lightning-fast",
            "anthropic/claude-3-5-haiku-20241022",
            "anthropic/claude-3-5-haiku-20241022",
            "anthropic/claude-3-5-haiku-20241022",
            "anthropic/claude-3-5-haiku-20241022",
        ),
        (
            "deep-researcher",
            "anthropic/claude-3-5-sonnet-20241022",
            "openai/gpt-4o-2024-11-20",
            "google/gemini-2.0-flash-thinking-exp-1219:free",
            "anthropic/claude-3-5-sonnet-20241022",
        ),
    ];

    for (name, gen, ref_, val, cur) in profiles {
        match ConsensusProfile::create(
            name.to_string(),
            gen.to_string(),
            ref_.to_string(),
            val.to_string(),
            cur.to_string(),
        )
        .await
        {
            Ok(_) => println!("✅ Created profile: {}", name),
            Err(e) => println!("⚠️ Failed to create profile {}: {}", name, e),
        }
    }

    Ok(())
}
