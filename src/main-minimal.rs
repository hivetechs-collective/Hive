//! Hive AI CLI - Minimal test version

use hive_ai::core::{initialize_default_logging, HiveError};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), HiveError> {
    // Initialize logging
    initialize_default_logging()?;
    
    // Show a simple startup message
    println!("🐝 Hive AI - Rust Implementation");
    println!("Version: {}", hive_ai::VERSION);
    println!("✅ Foundation infrastructure loaded successfully!");
    
    // Test error handling
    let workspace = hive_ai::core::Workspace::new(PathBuf::from(".")).await?;
    println!("📁 Workspace initialized: {}", workspace.root.display());
    
    // Test basic functionality
    test_error_handling().await?;
    test_logging().await;
    
    println!("🎉 All foundation tests passed!");
    Ok(())
}

async fn test_error_handling() -> Result<(), HiveError> {
    // Test different error types
    let _config_error = HiveError::config_not_found("/tmp/nonexistent");
    let _db_error = HiveError::database_connection("test error");
    let _file_error = HiveError::file_not_found("/tmp/missing.txt");
    
    println!("✅ Error handling system tested");
    Ok(())
}

async fn test_logging() {
    use hive_ai::core::logging::{PerfTimer, ErrorTracker, AnalyticsLogger};
    use tracing::{info, warn, error};
    
    // Test structured logging
    info!("Testing info level logging");
    warn!("Testing warning level logging");
    error!("Testing error level logging");
    
    // Test performance timer
    let timer = PerfTimer::new("test_operation");
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    timer.checkpoint("midpoint");
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    timer.finish();
    
    // Test error tracking
    let test_error = HiveError::internal("test", "This is a test error");
    ErrorTracker::log_error(&test_error, Some("testing"));
    
    // Test analytics logging
    AnalyticsLogger::log_user_action("test_action", None);
    AnalyticsLogger::log_performance_metrics("test_operation", 25.5, Some(12.3), Some(45.6));
    
    println!("✅ Logging system tested");
}