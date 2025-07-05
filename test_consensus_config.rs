#!/usr/bin/env rust-script
//! Tests the consensus engine configuration loading
//! 
//! Run with: rustc test_consensus_config.rs && ./test_consensus_config

use std::env;
use std::path::PathBuf;

fn main() {
    println!("Testing consensus engine configuration loading...\n");
    
    // Test 1: Check if config file exists
    let config_dir = dirs::home_dir()
        .map(|h| h.join(".hive"))
        .unwrap_or_else(|| PathBuf::from(".hive"));
    let config_path = config_dir.join("config.toml");
    
    println!("✓ Config directory: {}", config_dir.display());
    println!("✓ Config file path: {}", config_path.display());
    
    if config_path.exists() {
        println!("✓ Config file exists");
        
        // Test 2: Read and check for OpenRouter API key
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            if contents.contains("[openrouter]") && contents.contains("api_key") {
                println!("✓ OpenRouter configuration found in config file");
            } else {
                println!("✗ OpenRouter configuration not found in config file");
                println!("  Run 'hive setup' to configure API keys");
            }
        }
    } else {
        println!("✗ Config file does not exist");
        println!("  Run 'hive setup' to create configuration");
    }
    
    // Test 3: Check environment variable (should not be needed anymore)
    if env::var("OPENROUTER_API_KEY").is_ok() {
        println!("\n⚠️  Warning: OPENROUTER_API_KEY environment variable is set");
        println!("  The consensus engine now uses the config file instead");
    } else {
        println!("\n✓ No OPENROUTER_API_KEY environment variable (correct)");
    }
    
    println!("\nConfiguration test complete!");
}