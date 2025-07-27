//! Simple Claude Code Integration - Beautiful GUI Preserved!
//!
//! This module provides a clean, simple integration with Claude Code CLI
//! that preserves all existing GUI functionality while adding intelligence.

use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, warn};

/// Simple Claude Code integration that preserves GUI design
pub struct SimpleClaudeIntegration {
    claude_binary: Option<String>,
}

impl SimpleClaudeIntegration {
    /// Create new integration with strict Claude Code requirement
    pub async fn new() -> Result<Self> {
        let integration = Self {
            claude_binary: None,
        };
        
        // Try to find Claude binary - this is now REQUIRED
        match integration.find_claude_binary().await {
            Ok(binary) => {
                info!("✅ Found Claude Code at: {}", binary);
                
                // Verify Claude Code is working
                if let Err(e) = integration.verify_claude_working(&binary).await {
                    error!("❌ Claude Code found but not working: {}", e);
                    return Err(anyhow::anyhow!(
                        "Claude Code installation found but not functional: {}. Please reinstall Claude Code.", e
                    ));
                }
                
                info!("✅ Claude Code verified and working");
                Ok(Self {
                    claude_binary: Some(binary),
                })
            }
            Err(_) => {
                error!("❌ Claude Code is REQUIRED but not found");
                Err(anyhow::anyhow!(
                    "Claude Code is required but not installed. Install with:\n\
                    \n\
                    📦 NPM: npm install -g @anthropic-ai/claude-code\n\
                    🌐 Download: https://claude.ai/download\n\
                    🍺 Homebrew: brew install claude-code (if available)\n\
                    \n\
                    After installation, restart Hive AI."
                ))
            }
        }
    }
    
    /// Verify Claude Code is working properly
    async fn verify_claude_working(&self, binary: &str) -> Result<()> {
        // Test basic version command
        let output = Command::new(binary)
            .arg("--version")
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("Version command failed"));
        }
        
        // Test help command to ensure Claude is responsive
        let output = Command::new(binary)
            .arg("--help")
            .output()
            .await?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("Help command failed"));
        }
        
        Ok(())
    }
    
    /// Find Claude Code binary
    async fn find_claude_binary(&self) -> Result<String> {
        // Try common locations
        let possible_paths = vec![
            "claude",
            "/usr/local/bin/claude",
            "/opt/homebrew/bin/claude",
            "~/.local/bin/claude",
        ];
        
        for path in possible_paths {
            let expanded_path = shellexpand::tilde(path).to_string();
            if let Ok(output) = Command::new(&expanded_path)
                .arg("--version")
                .output()
                .await
            {
                if output.status.success() {
                    return Ok(expanded_path);
                }
            }
        }
        
        // Try 'which' command
        if let Ok(output) = Command::new("which")
            .arg("claude")
            .output()
            .await
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(path);
                }
            }
        }
        
        Err(anyhow::anyhow!("Claude Code not found"))
    }
    
    /// Send message to Claude Code with optional memory context
    pub async fn send_message(&self, message: &str, memory_context: Option<&str>) -> Result<String> {
        let claude_binary = match &self.claude_binary {
            Some(binary) => binary,
            None => {
                return Ok(
                    "⚠️ Claude Code not found. Please install it:\n\
                    npm install -g @anthropic-ai/claude-code\n\
                    or download from: https://claude.ai/download".to_string()
                );
            }
        };
        
        // Build enhanced message with memory context if provided
        let enhanced_message = if let Some(context) = memory_context {
            format!("Context from previous conversations:\n{}\n\nUser: {}", context, message)
        } else {
            message.to_string()
        };
        
        // Execute Claude Code with the message
        self.execute_claude_command(&claude_binary, &enhanced_message).await
    }
    
    /// Execute Claude Code command and return response
    async fn execute_claude_command(&self, binary: &str, message: &str) -> Result<String> {
        info!("Sending to Claude: {}", message);
        
        // For interactive commands, handle differently
        if message.trim().starts_with('/') {
            return self.execute_interactive_command(binary, message).await;
        }
        
        // For regular messages, use ask command
        let mut child = Command::new(binary)
            .arg("ask")
            .arg(message)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;
        
        // Read output
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();
        let mut output = String::new();
        let mut error_output = String::new();
        
        // Read stdout
        while let Some(line) = stdout_reader.next_line().await? {
            output.push_str(&line);
            output.push('\n');
        }
        
        // Read stderr if needed
        while let Some(line) = stderr_reader.next_line().await? {
            error_output.push_str(&line);
            error_output.push('\n');
        }
        
        // Wait for completion
        let status = child.wait().await?;
        
        if status.success() {
            Ok(if output.is_empty() { 
                "Claude Code executed successfully.".to_string() 
            } else { 
                output.trim().to_string() 
            })
        } else {
            if !error_output.is_empty() {
                Ok(format!("Claude Code error: {}", error_output.trim()))
            } else {
                Ok(format!("Claude Code command failed with exit code: {:?}", status.code()))
            }
        }
    }
    
    /// Handle interactive commands like /login, /help, etc.
    async fn execute_interactive_command(&self, binary: &str, command: &str) -> Result<String> {
        info!("Processing slash command: {}", command);
        
        // For slash commands, we need to pass them as part of the message to Claude
        // Claude Code CLI expects: claude ask "/help" or claude ask "/login"
        // Not: claude help or claude login
        
        // Use the ask command with the slash command as the message
        let mut child = Command::new(binary)
            .arg("ask")
            .arg(command)  // Pass the slash command as-is (e.g., "/help", "/login")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;
        
        // Read output
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();
        let mut output = String::new();
        let mut error_output = String::new();
        
        // Read stdout
        while let Some(line) = stdout_reader.next_line().await? {
            output.push_str(&line);
            output.push('\n');
        }
        
        // Read stderr if needed
        while let Some(line) = stderr_reader.next_line().await? {
            error_output.push_str(&line);
            error_output.push('\n');
        }
        
        // Wait for completion
        let status = child.wait().await?;
        
        if status.success() {
            Ok(if output.is_empty() { 
                format!("Command '{}' executed successfully.", command)
            } else { 
                output.trim().to_string() 
            })
        } else {
            if !error_output.is_empty() {
                warn!("Slash command error: {}", error_output);
                Ok(format!("Command error: {}", error_output.trim()))
            } else {
                Ok(format!("Command '{}' failed", command))
            }
        }
    }
    
    /// Check if Claude Code is available
    pub fn is_available(&self) -> bool {
        self.claude_binary.is_some()
    }
    
    /// Get status message for GUI
    pub fn get_status(&self) -> String {
        if self.is_available() {
            "✅ Claude Code Ready".to_string()
        } else {
            "⚠️ Claude Code Not Found".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_integration_creation() {
        let integration = SimpleClaudeIntegration::new().await;
        assert!(integration.is_ok());
    }
}