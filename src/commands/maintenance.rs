use crate::consensus::maintenance::TemplateMaintenanceManager;
use crate::core::database_simple::Database;
use anyhow::Result;
use std::sync::Arc;

/// Run model maintenance to sync with OpenRouter
pub async fn run_maintenance_command() -> Result<()> {
    println!("🔧 Running model maintenance...");
    
    // Get API key
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .ok()
        .or_else(|| {
            // Try to read from config
            let config_path = crate::core::config::get_hive_config_dir().join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).ok()?;
                let config: toml::Value = toml::from_str(&content).ok()?;
                config.get("openrouter_api_key")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        });
    
    if api_key.is_none() {
        anyhow::bail!("OpenRouter API key not found. Set OPENROUTER_API_KEY environment variable or add to config.toml");
    }
    
    // Get database
    let db_path = crate::core::config::get_hive_config_dir().join("hive-ai.db");
    let db_config = crate::core::database_simple::DatabaseConfig {
        path: db_path,
        enable_wal: true,
        enable_foreign_keys: true,
    };
    let db = Arc::new(Database::open(db_config).await?);
    
    // Run maintenance
    let mut maintenance = TemplateMaintenanceManager::new(db, api_key);
    let report = maintenance.run_maintenance().await?;
    
    println!("✅ Maintenance complete!");
    println!("   - Models synced: {}", report.models_synced);
    println!("   - Invalid profiles: {}", report.invalid_profiles.len());
    println!("   - Migrated profiles: {}", report.migrated_profiles.len());
    
    if !report.errors.is_empty() {
        println!("⚠️  Errors encountered:");
        for error in report.errors {
            println!("   - {}", error);
        }
    }
    
    Ok(())
}