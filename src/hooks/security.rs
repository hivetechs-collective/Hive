//! Hook Security - Security validation and policies for hooks

use super::{registry::HookAction, Hook};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

/// Security policy for a hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    #[serde(default)]
    pub require_approval: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_message: Option<String>,

    #[serde(default)]
    pub allowed_commands: Vec<String>,

    #[serde(default)]
    pub allowed_domains: Vec<String>,

    #[serde(default)]
    pub blocked_domains: Vec<String>,

    #[serde(default)]
    pub allowed_script_languages: Vec<String>,

    #[serde(default = "default_max_execution_time")]
    pub max_execution_time: u64, // seconds

    #[serde(default = "default_stop_on_error")]
    pub stop_on_error: bool,

    #[serde(default)]
    pub sandbox_mode: bool,

    #[serde(default)]
    pub allow_network: bool,

    #[serde(default)]
    pub allow_file_system: bool,

    #[serde(default)]
    pub max_memory_mb: Option<u64>,

    #[serde(default)]
    pub required_permissions: Vec<String>,
}

fn default_max_execution_time() -> u64 {
    300 // 5 minutes
}

fn default_stop_on_error() -> bool {
    true
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            require_approval: false,
            approval_message: None,
            allowed_commands: vec![
                "cargo".to_string(),
                "rustfmt".to_string(),
                "clippy".to_string(),
                "git".to_string(),
                "npm".to_string(),
                "yarn".to_string(),
                "prettier".to_string(),
                "eslint".to_string(),
                "black".to_string(),
                "pytest".to_string(),
                "go".to_string(),
                "gofmt".to_string(),
            ],
            allowed_domains: vec![
                "api.github.com".to_string(),
                "api.gitlab.com".to_string(),
                "openrouter.ai".to_string(),
            ],
            blocked_domains: vec![],
            allowed_script_languages: vec![
                "bash".to_string(),
                "sh".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "ruby".to_string(),
            ],
            max_execution_time: default_max_execution_time(),
            stop_on_error: default_stop_on_error(),
            sandbox_mode: false,
            allow_network: true,
            allow_file_system: true,
            max_memory_mb: None,
            required_permissions: vec![],
        }
    }
}

/// Validates hook security policies
pub struct HookSecurityValidator {
    dangerous_commands: HashSet<String>,
    dangerous_patterns: Vec<Regex>,
    max_action_count: usize,
}

impl HookSecurityValidator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            dangerous_commands: Self::get_dangerous_commands(),
            dangerous_patterns: Self::compile_dangerous_patterns()?,
            max_action_count: 50,
        })
    }

    /// Validate a hook's security
    pub fn validate_hook(&self, hook: &Hook) -> Result<()> {
        // Check action count
        if hook.actions.len() > self.max_action_count {
            return Err(anyhow!(
                "Hook has too many actions ({} > {})",
                hook.actions.len(),
                self.max_action_count
            ));
        }

        // Validate each action
        for action in &hook.actions {
            self.validate_action(action, &hook.security)?;
        }

        // Validate security policy
        self.validate_security_policy(&hook.security)?;

        Ok(())
    }

    /// Validate a single action
    fn validate_action(&self, action: &HookAction, policy: &SecurityPolicy) -> Result<()> {
        match action {
            HookAction::Command { command, args, .. } => {
                self.validate_command_action(command, args, policy)?;
            }
            HookAction::Script { language, content } => {
                self.validate_script_action(language, content, policy)?;
            }
            HookAction::HttpRequest { url, .. } => {
                self.validate_http_action(url, policy)?;
            }
            _ => {} // Other actions are safe
        }

        Ok(())
    }

    /// Validate command action
    fn validate_command_action(
        &self,
        command: &str,
        args: &[String],
        policy: &SecurityPolicy,
    ) -> Result<()> {
        // Check if command is dangerous
        if self.dangerous_commands.contains(command) {
            return Err(anyhow!(
                "Command '{}' is not allowed for security reasons",
                command
            ));
        }

        // Check if command is in allowed list
        if !policy.allowed_commands.is_empty()
            && !policy.allowed_commands.contains(&command.to_string())
        {
            return Err(anyhow!(
                "Command '{}' is not in the allowed commands list",
                command
            ));
        }

        // Check for dangerous patterns in arguments
        let full_command = format!("{} {}", command, args.join(" "));
        for pattern in &self.dangerous_patterns {
            if pattern.is_match(&full_command) {
                return Err(anyhow!(
                    "Command contains dangerous pattern: {}",
                    full_command
                ));
            }
        }

        Ok(())
    }

    /// Validate script action
    fn validate_script_action(
        &self,
        language: &str,
        content: &str,
        policy: &SecurityPolicy,
    ) -> Result<()> {
        // Check if language is allowed
        if !policy.allowed_script_languages.is_empty()
            && !policy
                .allowed_script_languages
                .contains(&language.to_string())
        {
            return Err(anyhow!("Script language '{}' is not allowed", language));
        }

        // Check for dangerous patterns in script
        for pattern in &self.dangerous_patterns {
            if pattern.is_match(content) {
                return Err(anyhow!("Script contains dangerous pattern"));
            }
        }

        // Language-specific checks
        match language {
            "bash" | "sh" => self.validate_shell_script(content)?,
            "python" => self.validate_python_script(content)?,
            "javascript" | "js" => self.validate_javascript_script(content)?,
            _ => {}
        }

        Ok(())
    }

    /// Validate HTTP action
    fn validate_http_action(&self, url: &str, policy: &SecurityPolicy) -> Result<()> {
        if !policy.allow_network {
            return Err(anyhow!("Network access is not allowed for this hook"));
        }

        // Parse URL
        let parsed_url = url::Url::parse(url).map_err(|_| anyhow!("Invalid URL: {}", url))?;

        // Check scheme
        if parsed_url.scheme() != "https" && parsed_url.scheme() != "http" {
            return Err(anyhow!("Only HTTP/HTTPS URLs are allowed"));
        }

        // Check domain
        if let Some(host) = parsed_url.host_str() {
            // Check blocked domains
            if policy.blocked_domains.iter().any(|d| host.contains(d)) {
                return Err(anyhow!("Domain '{}' is blocked", host));
            }

            // Check allowed domains if specified
            if !policy.allowed_domains.is_empty()
                && !policy.allowed_domains.iter().any(|d| host.contains(d))
            {
                return Err(anyhow!("Domain '{}' is not in allowed domains list", host));
            }
        }

        Ok(())
    }

    /// Validate security policy itself
    fn validate_security_policy(&self, policy: &SecurityPolicy) -> Result<()> {
        // Check execution time
        if policy.max_execution_time == 0 || policy.max_execution_time > 3600 {
            return Err(anyhow!(
                "Invalid max_execution_time: {} (must be between 1 and 3600 seconds)",
                policy.max_execution_time
            ));
        }

        // Check memory limit
        if let Some(memory_mb) = policy.max_memory_mb {
            if memory_mb == 0 || memory_mb > 8192 {
                return Err(anyhow!(
                    "Invalid max_memory_mb: {} (must be between 1 and 8192)",
                    memory_mb
                ));
            }
        }

        Ok(())
    }

    /// Validate a command for execution
    pub fn validate_command(&self, command: &str) -> Result<()> {
        if self.dangerous_commands.contains(command) {
            return Err(anyhow!(
                "Command '{}' is not allowed for security reasons",
                command
            ));
        }
        Ok(())
    }

    /// Validate a script language
    pub fn validate_script_language(&self, language: &str) -> Result<()> {
        let allowed_languages = ["bash", "sh", "python", "javascript", "js", "ruby"];
        if !allowed_languages.contains(&language) {
            return Err(anyhow!("Script language '{}' is not supported", language));
        }
        Ok(())
    }

    /// Validate a URL
    pub fn validate_url(&self, url: &str) -> Result<()> {
        let parsed = url::Url::parse(url)?;
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(anyhow!("Only HTTP/HTTPS URLs are allowed"));
        }
        Ok(())
    }

    /// Get list of dangerous commands
    fn get_dangerous_commands() -> HashSet<String> {
        [
            "rm", "del", "rmdir", "format", "dd", "mkfs", "fdisk", "chmod", "chown", "chgrp",
            "kill", "killall", "pkill", "shutdown", "reboot", "halt", "passwd", "useradd",
            "userdel", "sudo", "su", "doas", "nc", "netcat", "ncat", "curl",
            "wget", // Allow these but validate URLs
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Compile dangerous patterns
    fn compile_dangerous_patterns() -> Result<Vec<Regex>> {
        let patterns = [
            r"rm\s+-rf\s+/",
            r":\(\)\{\s*:\|:&\s*\};:", // Fork bomb (escaped braces)
            r">\s*/dev/sd[a-z]",
            r"dd\s+if=/dev/zero",
            r"/etc/passwd",
            r"/etc/shadow",
            r"base64\s+-d.*sh",
            r"eval\s*\(",
            r"exec\s*\(",
        ];

        patterns
            .iter()
            .map(|p| Regex::new(p).map_err(Into::into))
            .collect()
    }

    /// Validate shell script
    fn validate_shell_script(&self, content: &str) -> Result<()> {
        // Check for dangerous shell constructs
        let dangerous_constructs = ["eval ", "source /dev/stdin", "bash -c", "sh -c"];

        for construct in &dangerous_constructs {
            if content.contains(construct) {
                return Err(anyhow!(
                    "Script contains dangerous construct: {}",
                    construct
                ));
            }
        }

        Ok(())
    }

    /// Validate Python script
    fn validate_python_script(&self, content: &str) -> Result<()> {
        // Check for dangerous Python constructs
        let dangerous_constructs = [
            "eval(",
            "exec(",
            "__import__",
            "compile(",
            "open('/etc/passwd'",
            "subprocess.call(['rm'",
        ];

        for construct in &dangerous_constructs {
            if content.contains(construct) {
                return Err(anyhow!(
                    "Script contains dangerous construct: {}",
                    construct
                ));
            }
        }

        Ok(())
    }

    /// Validate JavaScript script
    fn validate_javascript_script(&self, content: &str) -> Result<()> {
        // Check for dangerous JavaScript constructs
        let dangerous_constructs = [
            "eval(",
            "Function(",
            "require('child_process')",
            "exec(",
            "spawn(",
        ];

        for construct in &dangerous_constructs {
            if content.contains(construct) {
                return Err(anyhow!(
                    "Script contains dangerous construct: {}",
                    construct
                ));
            }
        }

        Ok(())
    }
}

/// Security context for hook execution
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: Option<String>,
    pub session_id: String,
    pub permissions: HashSet<String>,
    pub trusted_paths: HashSet<String>,
}

impl SecurityContext {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission) || self.permissions.contains("admin")
    }

    pub fn is_path_trusted(&self, path: &str) -> bool {
        self.trusted_paths
            .iter()
            .any(|trusted| path.starts_with(trusted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_command_detection() {
        let validator = HookSecurityValidator::new().unwrap();

        assert!(validator.validate_command("rm").is_err());
        assert!(validator.validate_command("echo").is_ok());
        assert!(validator.validate_command("sudo").is_err());
        assert!(validator.validate_command("cargo").is_ok());
    }

    #[test]
    fn test_url_validation() {
        let validator = HookSecurityValidator::new().unwrap();

        assert!(validator.validate_url("https://api.github.com").is_ok());
        assert!(validator.validate_url("http://localhost:8080").is_ok());
        assert!(validator.validate_url("ftp://example.com").is_err());
        assert!(validator.validate_url("file:///etc/passwd").is_err());
    }
}
