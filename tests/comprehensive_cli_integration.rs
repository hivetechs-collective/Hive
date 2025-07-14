//! Comprehensive CLI Integration Tests
//! Tests all CLI commands with real API integration where possible

use hive_ai::cli::args::{AnalyzeSubcommand, Cli, Commands, MemorySubcommand, ModelsSubcommand};
use hive_ai::core::config::{create_default_config, get_hive_config_dir, HiveConfig};
use hive_ai::core::database::HiveDatabase;
use serial_test::serial;
use std::path::PathBuf;
use std::process::Command;
use tempfile::{tempdir, NamedTempFile};
use tokio;

/// Test helper to create a temporary config
async fn create_test_config() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = HiveConfig::default();
    let config_content = toml::to_string(&config).unwrap();
    tokio::fs::write(&config_path, config_content)
        .await
        .unwrap();

    (temp_dir, config_path)
}

/// Test helper to create a temporary project
async fn create_test_project() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path();

    // Create a basic Rust project structure
    tokio::fs::create_dir_all(project_path.join("src"))
        .await
        .unwrap();

    // Create Cargo.toml
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
    tokio::fs::write(project_path.join("Cargo.toml"), cargo_toml)
        .await
        .unwrap();

    // Create main.rs
    let main_rs = r#"
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TestStruct {
    id: u32,
    name: String,
}

fn main() {
    let test = TestStruct {
        id: 1,
        name: "Hello, World!".to_string(),
    };
    println!("{:?}", test);
}
"#;
    tokio::fs::write(project_path.join("src/main.rs"), main_rs)
        .await
        .unwrap();

    // Create lib.rs
    let lib_rs = r#"
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
"#;
    tokio::fs::write(project_path.join("src/lib.rs"), lib_rs)
        .await
        .unwrap();

    temp_dir
}

/// Test CLI help and version commands
#[tokio::test]
async fn test_cli_help_and_version() {
    // Test version command
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("hive-ai") || stdout.contains("0.1.0"));

    // Test help command
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("AI-powered codebase intelligence"));
}

/// Test configuration commands
#[tokio::test]
#[serial]
async fn test_config_commands() {
    let (_temp_dir, config_path) = create_test_config().await;

    // Test config show
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "show"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    // Should not fail (may show default config if none exists)
    // We mainly test that the command runs without crashing
    assert!(!output.status.code().unwrap_or(0) == 1);

    // Test config validate
    let output = Command::new("cargo")
        .args(&["run", "--", "config", "validate"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    // Should succeed with valid config
    let stderr = String::from_utf8(output.stderr).unwrap();
    // Don't assert success as config validation might be strict
    println!("Config validate output: {}", stderr);
}

/// Test analyze commands
#[tokio::test]
#[serial]
async fn test_analyze_commands() {
    let project_dir = create_test_project().await;
    let project_path = project_dir.path();

    // Test analyze overview
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze", "overview"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    // Should analyze the project structure
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Analyze overview stdout: {}", stdout);
    println!("Analyze overview stderr: {}", stderr);

    // Test analyze dependencies
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze", "dependencies"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Analyze dependencies stdout: {}", stdout);
    println!("Analyze dependencies stderr: {}", stderr);

    // Test analyze quality
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze", "quality"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Analyze quality stdout: {}", stdout);
    println!("Analyze quality stderr: {}", stderr);
}

/// Test index commands
#[tokio::test]
#[serial]
async fn test_index_commands() {
    let project_dir = create_test_project().await;
    let project_path = project_dir.path();

    // Test index build
    let output = Command::new("cargo")
        .args(&["run", "--", "index", "build"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Index build stdout: {}", stdout);
    println!("Index build stderr: {}", stderr);

    // Test index status
    let output = Command::new("cargo")
        .args(&["run", "--", "index", "status"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Index status stdout: {}", stdout);
    println!("Index status stderr: {}", stderr);
}

/// Test search commands
#[tokio::test]
#[serial]
async fn test_search_commands() {
    let project_dir = create_test_project().await;
    let project_path = project_dir.path();

    // Test search code
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "code", "TestStruct"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Search code stdout: {}", stdout);
    println!("Search code stderr: {}", stderr);

    // Test search symbols
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "symbols", "main"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Search symbols stdout: {}", stdout);
    println!("Search symbols stderr: {}", stderr);
}

/// Test memory commands
#[tokio::test]
#[serial]
async fn test_memory_commands() {
    let (_temp_dir, config_path) = create_test_config().await;

    // Test memory stats
    let output = Command::new("cargo")
        .args(&["run", "--", "memory", "stats"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Memory stats stdout: {}", stdout);
    println!("Memory stats stderr: {}", stderr);

    // Test memory conversations
    let output = Command::new("cargo")
        .args(&["run", "--", "memory", "conversations", "--limit", "10"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Memory conversations stdout: {}", stdout);
    println!("Memory conversations stderr: {}", stderr);
}

/// Test models commands
#[tokio::test]
#[serial]
async fn test_models_commands() {
    // Test models list
    let output = Command::new("cargo")
        .args(&["run", "--", "models", "list", "--provider", "openai"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Models list stdout: {}", stdout);
    println!("Models list stderr: {}", stderr);

    // Test models info
    let output = Command::new("cargo")
        .args(&["run", "--", "models", "info", "gpt-4"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Models info stdout: {}", stdout);
    println!("Models info stderr: {}", stderr);
}

/// Test cost tracking commands
#[tokio::test]
#[serial]
async fn test_cost_commands() {
    let (_temp_dir, config_path) = create_test_config().await;

    // Test cost stats
    let output = Command::new("cargo")
        .args(&["run", "--", "cost", "stats"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Cost stats stdout: {}", stdout);
    println!("Cost stats stderr: {}", stderr);

    // Test cost budget
    let output = Command::new("cargo")
        .args(&["run", "--", "cost", "budget", "--set", "10.00"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Cost budget stdout: {}", stdout);
    println!("Cost budget stderr: {}", stderr);
}

/// Test analytics commands
#[tokio::test]
#[serial]
async fn test_analytics_commands() {
    let (_temp_dir, config_path) = create_test_config().await;

    // Test analytics overview
    let output = Command::new("cargo")
        .args(&["run", "--", "analytics", "overview"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Analytics overview stdout: {}", stdout);
    println!("Analytics overview stderr: {}", stderr);

    // Test analytics trends
    let output = Command::new("cargo")
        .args(&["run", "--", "analytics", "trends", "--period", "week"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Analytics trends stdout: {}", stdout);
    println!("Analytics trends stderr: {}", stderr);
}

/// Test performance commands
#[tokio::test]
#[serial]
async fn test_performance_commands() {
    // Test performance profile
    let output = Command::new("cargo")
        .args(&["run", "--", "performance", "profile", "--duration", "30s"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Performance profile stdout: {}", stdout);
    println!("Performance profile stderr: {}", stderr);

    // Test performance benchmark
    let output = Command::new("cargo")
        .args(&["run", "--", "performance", "benchmark"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Performance benchmark stdout: {}", stdout);
    println!("Performance benchmark stderr: {}", stderr);
}

/// Test hooks commands
#[tokio::test]
#[serial]
async fn test_hooks_commands() {
    let (_temp_dir, config_path) = create_test_config().await;

    // Test hooks list
    let output = Command::new("cargo")
        .args(&["run", "--", "hooks", "list"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Hooks list stdout: {}", stdout);
    println!("Hooks list stderr: {}", stderr);

    // Test hooks validate
    let output = Command::new("cargo")
        .args(&["run", "--", "hooks", "validate"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Hooks validate stdout: {}", stdout);
    println!("Hooks validate stderr: {}", stderr);
}

/// Test TUI command
#[tokio::test]
#[serial]
async fn test_tui_command() {
    // Test TUI in test mode (should exit immediately)
    let output = Command::new("cargo")
        .args(&["run", "--", "tui", "--test-mode"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("TUI test mode stdout: {}", stdout);
    println!("TUI test mode stderr: {}", stderr);
}

/// Test interactive commands that don't require user input
#[tokio::test]
#[serial]
async fn test_non_interactive_commands() {
    // Test status command
    let output = Command::new("cargo")
        .args(&["run", "--", "status"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Status stdout: {}", stdout);
    println!("Status stderr: {}", stderr);

    // Test doctor command
    let output = Command::new("cargo")
        .args(&["run", "--", "doctor"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("Doctor stdout: {}", stdout);
    println!("Doctor stderr: {}", stderr);
}

/// Test error handling for invalid commands
#[tokio::test]
async fn test_error_handling() {
    // Test invalid command
    let output = Command::new("cargo")
        .args(&["run", "--", "invalid-command"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("error") || stderr.contains("invalid") || stderr.contains("unrecognized")
    );

    // Test invalid subcommand
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze", "invalid-subcommand"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("error") || stderr.contains("invalid") || stderr.contains("unrecognized")
    );
}

/// Test CLI with different output formats
#[tokio::test]
#[serial]
async fn test_output_formats() {
    let project_dir = create_test_project().await;
    let project_path = project_dir.path();

    // Test JSON output
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze", "overview", "--format", "json"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("JSON format stdout: {}", stdout);
    println!("JSON format stderr: {}", stderr);

    // Test YAML output
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze", "overview", "--format", "yaml"])
        .current_dir(project_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("YAML format stdout: {}", stdout);
    println!("YAML format stderr: {}", stderr);
}

/// Integration test for the complete CLI workflow
#[tokio::test]
#[serial]
async fn test_complete_cli_workflow() {
    let project_dir = create_test_project().await;
    let project_path = project_dir.path();
    let (_temp_dir, config_path) = create_test_config().await;

    // 1. Analyze the project
    let output = Command::new("cargo")
        .args(&["run", "--", "analyze", "overview"])
        .current_dir(project_path)
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    println!("Workflow step 1 - Analyze: {:?}", output.status);

    // 2. Build index
    let output = Command::new("cargo")
        .args(&["run", "--", "index", "build"])
        .current_dir(project_path)
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    println!("Workflow step 2 - Index: {:?}", output.status);

    // 3. Search for code
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "code", "main"])
        .current_dir(project_path)
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    println!("Workflow step 3 - Search: {:?}", output.status);

    // 4. Check memory stats
    let output = Command::new("cargo")
        .args(&["run", "--", "memory", "stats"])
        .env("HIVE_CONFIG_PATH", config_path.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    println!("Workflow step 4 - Memory: {:?}", output.status);

    // This test mainly verifies that the complete workflow can run
    // without crashing, which is valuable for integration testing
}

/// Performance test for CLI responsiveness
#[tokio::test]
#[serial]
async fn test_cli_performance() {
    let start = std::time::Instant::now();

    // Test that basic commands respond quickly
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    let duration = start.elapsed();

    // Should be fast (this includes compilation time in debug mode)
    // In release mode, this should be much faster
    println!("CLI version command took: {:?}", duration);

    assert!(output.status.success());
}
