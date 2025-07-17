//! Test runner for AI Helper integration tests
//! 
//! This module provides utilities for running and validating AI helper tests.

use std::process::Command;
use anyhow::{Result, Context};

/// Run all AI helper tests
pub async fn run_all_tests() -> Result<()> {
    println!("🧪 Running AI Helper Integration Tests...");
    
    // Check if Python dependencies are available
    check_python_dependencies().await?;
    
    // Run the tests
    let output = Command::new("cargo")
        .args(&["test", "--test", "ai_helpers_e2e_test", "--", "--nocapture"])
        .output()
        .context("Failed to run tests")?;
    
    if output.status.success() {
        println!("✅ All tests passed!");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("❌ Tests failed!");
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Tests failed");
    }
    
    Ok(())
}

/// Check if Python dependencies are available
async fn check_python_dependencies() -> Result<()> {
    println!("🔍 Checking Python dependencies...");
    
    // Check Python version
    let python_output = Command::new("python3")
        .args(&["--version"])
        .output()
        .context("Python3 not found")?;
    
    if !python_output.status.success() {
        anyhow::bail!("Python3 is required but not available");
    }
    
    println!("✅ Python: {}", String::from_utf8_lossy(&python_output.stdout).trim());
    
    // Check if pip is available
    let pip_output = Command::new("pip3")
        .args(&["--version"])
        .output()
        .context("pip3 not found")?;
    
    if !pip_output.status.success() {
        anyhow::bail!("pip3 is required but not available");
    }
    
    println!("✅ pip: {}", String::from_utf8_lossy(&pip_output.stdout).trim());
    
    // Check if requirements.txt exists
    if !std::path::Path::new("python/requirements.txt").exists() {
        anyhow::bail!("python/requirements.txt not found");
    }
    
    println!("✅ Requirements file found");
    
    // Try to install dependencies if needed
    println!("📦 Installing Python dependencies...");
    let install_output = Command::new("pip3")
        .args(&["install", "-r", "python/requirements.txt", "--quiet"])
        .output()
        .context("Failed to install Python dependencies")?;
    
    if !install_output.status.success() {
        println!("⚠️  Warning: Failed to install Python dependencies");
        println!("STDERR: {}", String::from_utf8_lossy(&install_output.stderr));
        // Don't fail here, just warn
    } else {
        println!("✅ Python dependencies installed");
    }
    
    Ok(())
}

/// Run a specific test
pub async fn run_specific_test(test_name: &str) -> Result<()> {
    println!("🎯 Running specific test: {}", test_name);
    
    let output = Command::new("cargo")
        .args(&["test", test_name, "--", "--nocapture"])
        .output()
        .context("Failed to run specific test")?;
    
    if output.status.success() {
        println!("✅ Test '{}' passed!", test_name);
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("❌ Test '{}' failed!", test_name);
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Test failed");
    }
    
    Ok(())
}

/// Check system requirements
pub async fn check_system_requirements() -> Result<()> {
    println!("🔧 Checking system requirements...");
    
    // Check Rust toolchain
    let rust_output = Command::new("rustc")
        .args(&["--version"])
        .output()
        .context("Rust compiler not found")?;
    
    println!("✅ Rust: {}", String::from_utf8_lossy(&rust_output.stdout).trim());
    
    // Check Cargo
    let cargo_output = Command::new("cargo")
        .args(&["--version"])
        .output()
        .context("Cargo not found")?;
    
    println!("✅ Cargo: {}", String::from_utf8_lossy(&cargo_output.stdout).trim());
    
    // Check Python
    check_python_dependencies().await?;
    
    // Check available memory
    #[cfg(target_os = "macos")]
    {
        let memory_output = Command::new("sysctl")
            .args(&["hw.memsize"])
            .output()
            .context("Failed to check memory")?;
        
        if memory_output.status.success() {
            let memory_str = String::from_utf8_lossy(&memory_output.stdout);
            if let Some(memory_size) = memory_str.split_whitespace().nth(1) {
                if let Ok(bytes) = memory_size.parse::<u64>() {
                    let gb = bytes / (1024 * 1024 * 1024);
                    println!("✅ System Memory: {} GB", gb);
                    
                    if gb < 8 {
                        println!("⚠️  Warning: Less than 8GB RAM available. AI models may run slowly.");
                    }
                }
            }
        }
    }
    
    println!("✅ System requirements check completed");
    Ok(())
}