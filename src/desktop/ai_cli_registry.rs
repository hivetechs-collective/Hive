//! AI CLI tool registry and configurations
//!
//! This module defines all available AI CLI tools and their configurations.

use crate::desktop::ai_cli_updater::{
    AuthStatus, CliToolConfig, InstallMethod, InstallScript, Platform, ToolDependency,
};

/// Get the complete list of AI CLI tools
pub fn get_all_ai_tools() -> Vec<CliToolConfig> {
    vec![
        // Tier 1: Primary tool - Claude CLI
        CliToolConfig {
            id: "claude".to_string(),
            name: "Claude Code CLI".to_string(),
            command: "claude".to_string(),
            icon: "🤖".to_string(),
            install_method: InstallMethod::Npm,
            version_check: "claude --version".to_string(),
            install_script: InstallScript::Npm("@anthropic/claude-cli".to_string()),
            platforms: vec![Platform::MacOS, Platform::Linux, Platform::Windows],
            dependencies: vec![],
            auth_status: AuthStatus::Required {
                instructions: "Run 'claude login' after installation".to_string()
            },
            sync_type: "claude_cli_update".to_string(),
        },
        
        // Tier 2: Popular tools
        CliToolConfig {
            id: "gh-copilot".to_string(),
            name: "GitHub Copilot CLI".to_string(),
            command: "gh".to_string(), // The actual command is "gh copilot"
            icon: "🐙".to_string(),
            install_method: InstallMethod::GhExtension,
            version_check: "gh copilot --version".to_string(),
            install_script: InstallScript::GhExtension("github/gh-copilot".to_string()),
            platforms: vec![Platform::MacOS, Platform::Linux, Platform::Windows],
            dependencies: vec![
                ToolDependency {
                    tool: "gh".to_string(),
                    check_command: "gh --version".to_string(),
                    install_hint: "Install GitHub CLI first: brew install gh (macOS) or see https://cli.github.com".to_string(),
                }
            ],
            auth_status: AuthStatus::Required {
                instructions: "Run 'gh auth login' first, then 'gh extension install github/gh-copilot'".to_string()
            },
            sync_type: "gh_copilot_cli_update".to_string(),
        },
        
        CliToolConfig {
            id: "openai".to_string(),
            name: "OpenAI CLI".to_string(),
            command: "openai".to_string(),
            icon: "🌟".to_string(),
            install_method: InstallMethod::Pipx,
            version_check: "openai --version".to_string(),
            install_script: InstallScript::Pipx("openai-cli".to_string()),
            platforms: vec![Platform::MacOS, Platform::Linux, Platform::Windows],
            dependencies: vec![],
            auth_status: AuthStatus::Required {
                instructions: "Set OPENAI_API_KEY environment variable".to_string()
            },
            sync_type: "openai_cli_update".to_string(),
        },
        
        // Additional tools can be added here as we expand support
    ]
}

/// Get only the enabled tools (for initial rollout, just Claude)
pub fn get_enabled_ai_tools() -> Vec<CliToolConfig> {
    get_all_ai_tools()
        .into_iter()
        .filter(|tool| tool.id == "claude") // Start with just Claude
        .collect()
}

/// Get tool by ID
pub fn get_tool_by_id(tool_id: &str) -> Option<CliToolConfig> {
    get_all_ai_tools()
        .into_iter()
        .find(|tool| tool.id == tool_id)
}

/// Check if a tool is currently enabled
pub fn is_tool_enabled(tool_id: &str) -> bool {
    get_enabled_ai_tools().iter().any(|tool| tool.id == tool_id)
}
