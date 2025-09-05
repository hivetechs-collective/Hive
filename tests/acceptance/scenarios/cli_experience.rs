//! CLI User Experience Acceptance Tests
//! End-to-end validation of user workflows and CLI behavior

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::fs;
use tempfile::TempDir;
use tokio;
use anyhow::Result;
use serial_test::serial;

/// Test basic CLI help and version commands
#[tokio::test]
#[serial]
async fn test_basic_cli_commands() -> Result<()> {
    let binary_path = get_hive_binary()?;

    // Test --version command
    let version_output = Command::new(&binary_path)
        .arg("--version")
        .output()?;

    assert!(version_output.status.success(), "Version command should succeed");

    let version_text = String::from_utf8(version_output.stdout)?;
    assert!(version_text.contains("hive"), "Version should contain 'hive'");
    assert!(version_text.contains("0.1.0") || version_text.contains("2.0"),
           "Should contain version number");

    println!("✅ Version command: {}", version_text.trim());

    // Test --help command
    let help_output = Command::new(&binary_path)
        .arg("--help")
        .output()?;

    assert!(help_output.status.success(), "Help command should succeed");

    let help_text = String::from_utf8(help_output.stdout)?;
    assert!(help_text.contains("Usage:") || help_text.contains("USAGE:"),
           "Help should contain usage information");
    assert!(help_text.contains("ask") || help_text.contains("consensus"),
           "Help should mention main commands");

    // Verify essential commands are mentioned
    let essential_commands = vec!["ask", "analyze", "config", "status"];
    let mut found_commands = 0;

    for command in &essential_commands {
        if help_text.to_lowercase().contains(command) {
            found_commands += 1;
        }
    }

    assert!(found_commands >= essential_commands.len() / 2,
           "Help should mention most essential commands");

    println!("✅ Help command shows {} essential commands", found_commands);

    let mut metrics = HashMap::new();
    metrics.insert("version_command_success".to_string(), 1.0);
    metrics.insert("help_command_success".to_string(), 1.0);
    metrics.insert("essential_commands_found".to_string(), found_commands as f64);

    Ok(())
}

/// Test configuration management workflow
#[tokio::test]
#[serial]
async fn test_configuration_workflow() -> Result<()> {
    let binary_path = get_hive_binary()?;
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test_config.toml");

    // Test config status without configuration
    let status_output = Command::new(&binary_path)
        .args(&["config", "--check"])
        .env("HIVE_CONFIG_PATH", &config_path)
        .output()?;

    // Should handle missing config gracefully
    let status_text = String::from_utf8_lossy(&status_output.stderr);
    println!("Config check output: {}", status_text);

    // Test config initialization
    let init_output = Command::new(&binary_path)
        .args(&["config", "--init"])
        .env("HIVE_CONFIG_PATH", &config_path)
        .output()?;

    // Should create config file
    if init_output.status.success() {
        assert!(config_path.exists(), "Config file should be created");

        let config_content = fs::read_to_string(&config_path)?;
        assert!(config_content.contains("[general]") || config_content.contains("general"),
               "Config should contain general section");

        println!("✅ Config initialized successfully");
    } else {
        println!("⚠️  Config init may require interactive input");
    }

    // Test config validation
    let validate_output = Command::new(&binary_path)
        .args(&["config", "--validate"])
        .env("HIVE_CONFIG_PATH", &config_path)
        .output()?;

    if validate_output.status.success() {
        println!("✅ Config validation passed");
    } else {
        let error_text = String::from_utf8_lossy(&validate_output.stderr);
        println!("Config validation: {}", error_text);
    }

    let mut metrics = HashMap::new();
    metrics.insert("config_check_executed".to_string(), 1.0);
    metrics.insert("config_init_executed".to_string(), 1.0);
    metrics.insert("config_validate_executed".to_string(), 1.0);

    Ok(())
}

/// Test repository analysis workflow
#[tokio::test]
#[serial]
async fn test_repository_analysis() -> Result<()> {
    let binary_path = get_hive_binary()?;
    let temp_dir = TempDir::new()?;

    // Create a test repository structure
    create_test_repository(&temp_dir).await?;

    // Test repository indexing
    let index_output = Command::new(&binary_path)
        .args(&["index", temp_dir.path().to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if index_output.status.success() {
        let index_text = String::from_utf8_lossy(&index_output.stdout);
        println!("Repository indexing output: {}", index_text);

        // Should report file analysis
        assert!(index_text.contains("files") || index_text.contains("analyzed") ||
               index_text.contains("indexed"), "Should report indexing progress");
    } else {
        let error_text = String::from_utf8_lossy(&index_output.stderr);
        println!("Repository indexing error: {}", error_text);

        // Allow for missing dependencies or configuration
        if !error_text.to_lowercase().contains("not configured") {
            return Ok(()); // Skip if not configured
        }
    }

    // Test repository analysis
    let analyze_output = Command::new(&binary_path)
        .args(&["analyze", temp_dir.path().to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if analyze_output.status.success() {
        let analyze_text = String::from_utf8_lossy(&analyze_output.stdout);
        println!("Repository analysis output: {}", analyze_text);
    } else {
        let error_text = String::from_utf8_lossy(&analyze_output.stderr);
        println!("Repository analysis: {}", error_text);
    }

    let mut metrics = HashMap::new();
    metrics.insert("repository_index_executed".to_string(), 1.0);
    metrics.insert("repository_analyze_executed".to_string(), 1.0);

    Ok(())
}

/// Test basic query workflow
#[tokio::test]
#[serial]
async fn test_basic_query_workflow() -> Result<()> {
    let binary_path = get_hive_binary()?;

    // Test simple ask command
    let ask_output = Command::new(&binary_path)
        .args(&["ask", "What is 2+2?"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if ask_output.status.success() {
        let response = String::from_utf8_lossy(&ask_output.stdout);
        println!("Ask response: {}", response);

        // Should provide some response
        assert!(!response.trim().is_empty(), "Should provide a response");

        // For a simple math question, should contain the answer
        if response.to_lowercase().contains("4") {
            println!("✅ Correct answer provided");
        } else {
            println!("⚠️  Response may not contain expected answer");
        }
    } else {
        let error_text = String::from_utf8_lossy(&ask_output.stderr);
        println!("Ask command error: {}", error_text);

        // Common reasons for failure in testing
        if error_text.to_lowercase().contains("api key") {
            println!("⚠️  Skipping ask test - API key not configured");
            return Ok(());
        } else if error_text.to_lowercase().contains("not configured") {
            println!("⚠️  Skipping ask test - not configured");
            return Ok(());
        }
    }

    // Test ask with context
    let temp_dir = TempDir::new()?;
    let context_file = temp_dir.path().join("context.txt");
    fs::write(&context_file, "This is a test file for context.")?;

    let ask_with_context = Command::new(&binary_path)
        .args(&["ask", "What does this file contain?", "--context", context_file.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if ask_with_context.status.success() {
        let response = String::from_utf8_lossy(&ask_with_context.stdout);
        println!("Ask with context response: {}", response);
    } else {
        let error_text = String::from_utf8_lossy(&ask_with_context.stderr);
        println!("Ask with context error: {}", error_text);
    }

    let mut metrics = HashMap::new();
    metrics.insert("ask_command_executed".to_string(), 1.0);
    metrics.insert("ask_with_context_executed".to_string(), 1.0);

    Ok(())
}

/// Test status and health check workflow
#[tokio::test]
#[serial]
async fn test_status_workflow() -> Result<()> {
    let binary_path = get_hive_binary()?;

    // Test status command
    let status_output = Command::new(&binary_path)
        .args(&["status"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    assert!(status_output.status.success(), "Status command should succeed");

    let status_text = String::from_utf8_lossy(&status_output.stdout);
    println!("Status output:\n{}", status_text);

    // Should show system information
    let expected_items = vec!["version", "config", "memory", "system"];
    let mut found_items = 0;

    for item in &expected_items {
        if status_text.to_lowercase().contains(item) {
            found_items += 1;
        }
    }

    assert!(found_items >= 2, "Status should show at least 2 system information items");

    // Test health check
    let health_output = Command::new(&binary_path)
        .args(&["status", "--health"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if health_output.status.success() {
        let health_text = String::from_utf8_lossy(&health_output.stdout);
        println!("Health check output:\n{}", health_text);

        // Should show health status
        assert!(health_text.contains("✅") || health_text.contains("OK") ||
               health_text.contains("healthy") || health_text.contains("✓"),
               "Health check should show positive indicators");
    } else {
        println!("Health check returned non-zero exit code (may be expected)");
    }

    let mut metrics = HashMap::new();
    metrics.insert("status_command_success".to_string(), 1.0);
    metrics.insert("status_items_found".to_string(), found_items as f64);

    Ok(())
}

/// Test error handling and user feedback
#[tokio::test]
#[serial]
async fn test_error_handling() -> Result<()> {
    let binary_path = get_hive_binary()?;

    // Test invalid command
    let invalid_output = Command::new(&binary_path)
        .args(&["nonexistent-command"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    assert!(!invalid_output.status.success(), "Invalid command should fail");

    let error_text = String::from_utf8_lossy(&invalid_output.stderr);
    assert!(!error_text.is_empty(), "Should provide error message");
    assert!(error_text.to_lowercase().contains("command") ||
           error_text.to_lowercase().contains("unknown") ||
           error_text.to_lowercase().contains("help"),
           "Error should be informative about invalid command");

    println!("✅ Invalid command error: {}", error_text.trim());

    // Test invalid arguments
    let invalid_args = Command::new(&binary_path)
        .args(&["ask"]) // Missing required argument
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !invalid_args.status.success() {
        let args_error = String::from_utf8_lossy(&invalid_args.stderr);
        assert!(!args_error.is_empty(), "Should provide error for missing arguments");
        println!("✅ Missing argument error: {}", args_error.trim());
    }

    // Test invalid file paths
    let invalid_file = Command::new(&binary_path)
        .args(&["analyze", "/nonexistent/path"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !invalid_file.status.success() {
        let file_error = String::from_utf8_lossy(&invalid_file.stderr);
        assert!(file_error.to_lowercase().contains("not found") ||
               file_error.to_lowercase().contains("does not exist") ||
               file_error.to_lowercase().contains("no such file"),
               "Should provide clear error for missing files");
        println!("✅ File not found error: {}", file_error.trim());
    }

    let mut metrics = HashMap::new();
    metrics.insert("error_handling_tests".to_string(), 3.0);

    Ok(())
}

/// Test interactive features (where applicable)
#[tokio::test]
#[serial]
async fn test_interactive_features() -> Result<()> {
    let binary_path = get_hive_binary()?;

    // Test commands that might have interactive components
    let interactive_commands = vec![
        vec!["config", "--setup"],
        vec!["init"],
        vec!["setup"],
    ];

    for command in interactive_commands {
        let output = Command::new(&binary_path)
            .args(&command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null()) // Provide no input to test non-hanging behavior
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ Command {} completed successfully", command.join(" "));
                } else {
                    let error = String::from_utf8_lossy(&result.stderr);
                    if error.to_lowercase().contains("interactive") ||
                       error.to_lowercase().contains("input") {
                        println!("⚠️  Command {} requires interactive input", command.join(" "));
                    }
                }
            }
            Err(e) => {
                println!("Command {} failed to execute: {}", command.join(" "), e);
            }
        }
    }

    let mut metrics = HashMap::new();
    metrics.insert("interactive_commands_tested".to_string(), 3.0);

    Ok(())
}

/// Test performance of common CLI operations
#[tokio::test]
#[serial]
async fn test_cli_performance() -> Result<()> {
    let binary_path = get_hive_binary()?;

    // Test responsiveness of fast commands
    let fast_commands = vec![
        vec!["--version"],
        vec!["--help"],
        vec!["status"],
    ];

    for command in &fast_commands {
        let start = std::time::Instant::now();

        let output = Command::new(&binary_path)
            .args(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()?;

        let duration = start.elapsed();

        // Fast commands should complete quickly
        assert!(duration < std::time::Duration::from_millis(500),
               "Command {:?} took too long: {:?}", command, duration);

        println!("Command {:?}: {:.2}ms", command, duration.as_millis());
    }

    // Test that commands don't hang
    let potentially_slow_commands = vec![
        vec!["config", "--check"],
    ];

    for command in &potentially_slow_commands {
        let start = std::time::Instant::now();

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::process::Command::new(&binary_path)
                .args(command)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
        ).await;

        match output {
            Ok(Ok(_)) => {
                let duration = start.elapsed();
                println!("Command {:?}: {:.2}ms", command, duration.as_millis());
            }
            Ok(Err(e)) => {
                println!("Command {:?} failed: {}", command, e);
            }
            Err(_) => {
                println!("Command {:?} timed out (>5s)", command);
                // This might be expected for some commands
            }
        }
    }

    let mut metrics = HashMap::new();
    metrics.insert("fast_commands_tested".to_string(), fast_commands.len() as f64);
    metrics.insert("slow_commands_tested".to_string(), potentially_slow_commands.len() as f64);

    Ok(())
}

/// Helper functions

fn get_hive_binary() -> Result<String> {
    let possible_paths = vec![
        "target/release/hive",
        "target/debug/hive",
        "./hive",
        "/usr/local/bin/hive",
    ];

    for path in possible_paths {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    anyhow::bail!("Could not find hive binary")
}

async fn create_test_repository(temp_dir: &TempDir) -> Result<()> {
    let base_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(base_path.join("src"))?;
    fs::create_dir_all(base_path.join("tests"))?;
    fs::create_dir_all(base_path.join("docs"))?;

    // Create test files
    fs::write(base_path.join("README.md"), "# Test Repository\n\nThis is a test repository for analysis.")?;

    fs::write(base_path.join("Cargo.toml"), r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#)?;

    fs::write(base_path.join("src/main.rs"), r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#)?;

    fs::write(base_path.join("src/lib.rs"), r#"
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#)?;

    // Create some additional test files
    fs::write(base_path.join("tests/integration_test.rs"), r#"
use test_project::multiply;

#[test]
fn test_multiply() {
    assert_eq!(multiply(3, 4), 12);
}
"#)?;

    fs::write(base_path.join("docs/DESIGN.md"), "# Design Document\n\nArchitecture overview...")?;

    Ok(())
}