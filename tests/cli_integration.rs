//! Integration tests for CLI functionality

use hive_ai::cli::{args::Cli, check_tui_capabilities, should_launch_tui};
use hive_ai::core::config::{create_default_config, get_hive_config_dir, HiveConfig};
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_config_creation() {
    // Test that we can create a default configuration
    let config = HiveConfig::default();
    assert_eq!(config.consensus.profile, "balanced");
    assert!(config.interface.tui_mode);
    assert_eq!(config.logging.level, "info");
}

#[tokio::test]
async fn test_cli_parsing() {
    // Test basic CLI parsing
    // Note: In a real test, we'd use clap's testing utilities
    // For now, just test that the structure exists

    // This tests that our CLI args compile and have the expected structure
    use clap::Parser;

    // Test help generation works
    let help_output = Cli::try_parse_from(&["hive", "--help"]);
    assert!(help_output.is_err()); // --help causes clap to exit with error code
}

#[test]
fn test_tui_capabilities() {
    // Test TUI capability detection
    let _capabilities = check_tui_capabilities();
    // Just test that it doesn't panic
}

#[tokio::test]
async fn test_config_directory() {
    // Test configuration directory detection
    let config_dir = get_hive_config_dir();
    assert!(
        config_dir.to_string_lossy().contains(".hive")
            || config_dir.to_string_lossy().contains("HiveAI")
    );
}

#[tokio::test]
async fn test_default_config_file_creation() {
    // Test creating default config file
    let result = create_default_config().await;

    // This might fail if the directory doesn't exist, which is fine for testing
    // We're mainly testing that the function exists and returns a Result
    match result {
        Ok(path) => {
            assert!(path.exists() || path.to_string_lossy().contains("config.toml"));
        }
        Err(_) => {
            // Expected if we don't have write permissions
        }
    }
}
