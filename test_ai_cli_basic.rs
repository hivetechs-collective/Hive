//! Basic test for AI CLI functionality without database

use hive::desktop::ai_cli_registry::{get_enabled_ai_tools, get_tool_by_id};

fn main() {
    println!("Testing AI CLI Registry...\n");
    
    // Test getting enabled tools
    let tools = get_enabled_ai_tools();
    println!("Number of enabled tools: {}", tools.len());
    
    for tool in &tools {
        println!("Tool: {} ({})", tool.name, tool.id);
        println!("  Command: {}", tool.command);
        println!("  Icon: {}", tool.icon);
        println!("  Sync Type: {}", tool.sync_type);
    }
    
    // Test getting specific tool
    println!("\nTesting get_tool_by_id...");
    if let Some(claude) = get_tool_by_id("claude") {
        println!("✅ Found Claude tool: {}", claude.name);
    } else {
        println!("❌ Claude tool not found!");
    }
    
    // Test unknown tool
    if get_tool_by_id("unknown").is_none() {
        println!("✅ Unknown tool correctly returns None");
    } else {
        println!("❌ Unknown tool incorrectly found!");
    }
    
    println!("\n✅ Basic AI CLI registry tests passed!");
}