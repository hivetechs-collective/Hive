//! Security system demonstration
//! 
//! This example shows how the Claude Code-style trust system works in Hive AI

use anyhow::Result;
use hive_ai::core::{
    initialize_security, get_security_context, check_read_access, 
    check_write_access, check_directory_access, TrustLevel
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize security with interactive mode
    println!("🔒 Initializing Hive AI Security System...\n");
    initialize_security(None, true)?;
    
    // Get security context
    let context = get_security_context()?;
    
    // Demo 1: Try to read a file in an untrusted directory
    println!("📄 Demo 1: Reading file from untrusted directory");
    println!("=========================================");
    
    let test_file = PathBuf::from("/tmp/test_file.txt");
    match check_read_access(&test_file) {
        Ok(()) => println!("✅ Read access granted to: {}", test_file.display()),
        Err(e) => println!("❌ Read access denied: {}", e),
    }
    println!();
    
    // Demo 2: Try to write to a file
    println!("✏️  Demo 2: Writing to file");
    println!("=========================");
    
    let write_file = PathBuf::from("/tmp/output.txt");
    match check_write_access(&write_file) {
        Ok(()) => println!("✅ Write access granted to: {}", write_file.display()),
        Err(e) => println!("❌ Write access denied: {}", e),
    }
    println!();
    
    // Demo 3: List directory contents
    println!("📁 Demo 3: Listing directory");
    println!("===========================");
    
    let dir_path = PathBuf::from("/tmp");
    match check_directory_access(&dir_path) {
        Ok(()) => println!("✅ Directory access granted to: {}", dir_path.display()),
        Err(e) => println!("❌ Directory access denied: {}", e),
    }
    println!();
    
    // Demo 4: Show trusted paths
    println!("🛡️  Demo 4: Currently trusted paths");
    println!("==================================");
    
    let trusted_paths = context.get_trusted_paths()?;
    if trusted_paths.is_empty() {
        println!("No paths are currently trusted.");
    } else {
        for (path, level) in trusted_paths {
            let icon = match level {
                TrustLevel::Trusted => "✅",
                TrustLevel::Temporary => "⏱️",
                TrustLevel::Untrusted => "❌",
            };
            println!("{} {} - {}", icon, path.display(), level);
        }
    }
    println!();
    
    // Demo 5: Security event log
    println!("📋 Demo 5: Recent security events");
    println!("================================");
    
    if let Some((path, _)) = trusted_paths.first() {
        let events = context.get_events_for_path(path)?;
        for event in events.iter().take(5) {
            println!("[{}] {} - {} ({})", 
                event.timestamp.format("%H:%M:%S"),
                event.event_type,
                event.details,
                if event.allowed { "allowed" } else { "denied" }
            );
        }
    } else {
        println!("No security events to display.");
    }
    
    println!("\n✨ Security demo complete!");
    Ok(())
}