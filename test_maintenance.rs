use hive_ai::consensus::maintenance::TemplateMaintenanceManager;
use hive_ai::core::database_simple::Database;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔧 Running model maintenance test...");

    // Get API key
    let api_key = std::env::var("OPENROUTER_API_KEY")?;

    // Get database
    let db_path = hive_ai::core::config::get_hive_config_dir().join("hive-ai.db");
    let db = Arc::new(Database::new(&db_path, Default::default()).await?);

    // Run maintenance
    let mut maintenance = TemplateMaintenanceManager::new(db, Some(api_key));
    let report = maintenance.run_maintenance().await?;

    println!("✅ Maintenance complete!");
    println!("   - Models synced: {}", report.models_synced);
    println!("   - Invalid profiles: {:?}", report.invalid_profiles);
    println!("   - Migrated profiles: {:?}", report.migrated_profiles);

    if !report.errors.is_empty() {
        println!("⚠️  Errors encountered:");
        for error in report.errors {
            println!("   - {}", error);
        }
    }

    Ok(())
}