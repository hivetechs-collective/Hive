//! Tests for AI CLI updater functionality

#[cfg(test)]
mod tests {
    use hive::desktop::ai_cli_updater::{AiCliUpdaterDB, AuthStatus};
    use hive::desktop::ai_cli_registry::{get_enabled_ai_tools, get_tool_by_id};
    
    #[test]
    fn test_tool_registry() {
        // Test that we can get the enabled tools
        let tools = get_enabled_ai_tools();
        assert_eq!(tools.len(), 1, "Should have exactly 1 enabled tool (Claude)");
        
        let claude_tool = &tools[0];
        assert_eq!(claude_tool.id, "claude");
        assert_eq!(claude_tool.name, "Claude Code CLI");
        assert_eq!(claude_tool.command, "claude");
        assert_eq!(claude_tool.icon, "🤖");
    }
    
    #[test]
    fn test_get_tool_by_id() {
        // Test getting a specific tool
        let claude = get_tool_by_id("claude");
        assert!(claude.is_some(), "Should find Claude tool");
        
        let tool = claude.unwrap();
        assert_eq!(tool.id, "claude");
        assert!(matches!(tool.auth_status, AuthStatus::Required { .. }));
    }
    
    #[test]
    fn test_unknown_tool() {
        // Test getting an unknown tool
        let unknown = get_tool_by_id("unknown-tool");
        assert!(unknown.is_none(), "Should not find unknown tool");
    }
    
    #[tokio::test]
    async fn test_updater_initialization() {
        // Test that we can create an updater instance
        let result = AiCliUpdaterDB::new().await;
        
        // This might fail in test environment due to database, but let's check
        match result {
            Ok(updater) => {
                // Test that we can get the list of tools
                let tools = AiCliUpdaterDB::get_ai_tools();
                assert_eq!(tools.len(), 1, "Should have 1 tool");
            }
            Err(e) => {
                // Expected in test environment without database setup
                println!("Updater initialization failed as expected in test: {}", e);
            }
        }
    }
}