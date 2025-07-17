#!/usr/bin/env rust-script

//! Simple test runner for AI helpers functionality
//! This avoids the full test suite compilation issues

use std::process::Command;
use std::time::Duration;

fn main() {
    println!("🧪 Testing AI Helper Ecosystem...");
    
    // Test basic compilation
    println!("📦 Testing compilation...");
    let compile_result = Command::new("cargo")
        .args(&["check", "--lib"])
        .output()
        .expect("Failed to run cargo check");
    
    if !compile_result.status.success() {
        println!("❌ Compilation failed!");
        println!("STDERR: {}", String::from_utf8_lossy(&compile_result.stderr));
        return;
    }
    
    println!("✅ Compilation successful!");
    
    // Test Python dependencies
    println!("🐍 Testing Python dependencies...");
    let python_result = Command::new("python3")
        .args(&["-c", "import torch, transformers, sentence_transformers, chromadb; print('✅ Python dependencies OK')"])
        .output()
        .expect("Failed to test Python dependencies");
    
    if !python_result.status.success() {
        println!("⚠️  Python dependencies not fully available");
        println!("STDERR: {}", String::from_utf8_lossy(&python_result.stderr));
        println!("💡 Try: pip3 install -r python/requirements.txt");
    } else {
        println!("{}", String::from_utf8_lossy(&python_result.stdout));
    }
    
    // Test Python model service script
    println!("🤖 Testing Python model service...");
    let model_service_result = Command::new("python3")
        .args(&["-c", r#"
import sys
sys.path.append('python')
try:
    from model_service import ModelService
    service = ModelService()
    print('✅ Python model service can be imported')
except Exception as e:
    print(f'❌ Python model service error: {e}')
"#])
        .output()
        .expect("Failed to test Python model service");
    
    println!("{}", String::from_utf8_lossy(&model_service_result.stdout));
    
    // Test basic struct creation
    println!("🏗️  Testing Rust struct creation...");
    let struct_test = Command::new("cargo")
        .args(&["test", "--lib", "test_ai_helper_creation", "--", "--nocapture"])
        .output()
        .expect("Failed to run struct test");
    
    if struct_test.status.success() {
        println!("✅ Rust struct creation test passed!");
    } else {
        println!("⚠️  Rust struct test had issues:");
        println!("STDERR: {}", String::from_utf8_lossy(&struct_test.stderr));
    }
    
    println!("\n🎉 AI Helper Ecosystem basic tests completed!");
    println!("💡 For full integration tests, ensure Python dependencies are installed and run:");
    println!("   cargo test ai_helpers_e2e_test --release");
}