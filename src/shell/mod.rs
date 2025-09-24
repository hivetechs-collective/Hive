// Shell Integration Module for Hive AI
// Comprehensive shell integration and global deployment

use anyhow::{Context, Result};
use std::path::PathBuf;

pub mod completions;
pub mod hooks;
pub mod setup;
pub mod uninstall;

use crate::core::config::Config;
use serde::{Deserialize, Serialize};

/// Shell types supported by Hive AI
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl ShellType {
    /// Parse shell type from string
    pub fn from_string(shell: &str) -> Option<Self> {
        match shell.to_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "powershell" | "pwsh" => Some(Self::PowerShell),
            "elvish" => Some(Self::Elvish),
            _ => None,
        }
    }

    /// Get shell name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::PowerShell => "powershell",
            Self::Elvish => "elvish",
        }
    }

    /// Get completion file extension
    pub fn completion_filename(&self) -> &'static str {
        match self {
            Self::Bash => "hive",
            Self::Zsh => "_hive",
            Self::Fish => "hive.fish",
            Self::PowerShell => "hive.ps1",
            Self::Elvish => "hive.elv",
        }
    }
}

/// Shell integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellIntegrationStatus {
    pub shell: ShellType,
    pub completions_installed: bool,
    pub path_configured: bool,
    pub hooks_installed: bool,
    pub aliases_configured: bool,
    pub config_location: Option<PathBuf>,
}

/// Main shell integration manager
pub struct ShellIntegration {
    config: Config,
}

impl ShellIntegration {
    /// Create new shell integration manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Detect available shells on the system
    pub fn detect_shells(&self) -> Result<Vec<ShellType>> {
        let mut shells = Vec::new();

        // Check for shell executables in PATH
        let shell_commands = [
            ("bash", ShellType::Bash),
            ("zsh", ShellType::Zsh),
            ("fish", ShellType::Fish),
            ("pwsh", ShellType::PowerShell),
            ("powershell", ShellType::PowerShell),
            ("elvish", ShellType::Elvish),
        ];

        for (command, shell_type) in &shell_commands {
            if self.command_exists(command) {
                if !shells.contains(shell_type) {
                    shells.push(*shell_type);
                }
            }
        }

        Ok(shells)
    }

    /// Check if a command exists in PATH
    fn command_exists(&self, command: &str) -> bool {
        std::process::Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get integration status for a specific shell
    pub fn get_shell_status(&self, shell: ShellType) -> Result<ShellIntegrationStatus> {
        let completions = self.get_completions();
        let setup = self.get_setup();
        let hooks = self.get_hooks();

        Ok(ShellIntegrationStatus {
            shell,
            completions_installed: completions.is_installed(shell)?,
            path_configured: setup.is_path_configured(shell)?,
            hooks_installed: hooks.are_installed(shell)?,
            aliases_configured: hooks.are_aliases_configured(shell)?,
            config_location: self.get_shell_config_location(shell),
        })
    }

    /// Get shell configuration file location
    fn get_shell_config_location(&self, shell: ShellType) -> Option<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            let config_file = match shell {
                ShellType::Bash => ".bashrc",
                ShellType::Zsh => ".zshrc",
                ShellType::Fish => ".config/fish/config.fish",
                ShellType::PowerShell => {
                    if cfg!(windows) {
                        "Documents/PowerShell/Microsoft.PowerShell_profile.ps1"
                    } else {
                        ".config/powershell/Microsoft.PowerShell_profile.ps1"
                    }
                }
                ShellType::Elvish => ".elvish/rc.elv",
            };

            let path = PathBuf::from(home).join(config_file);
            if path.exists() {
                Some(path)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get status for all detected shells
    pub fn get_all_shell_status(&self) -> Result<Vec<ShellIntegrationStatus>> {
        let shells = self.detect_shells()?;
        let mut statuses = Vec::new();

        for shell in shells {
            match self.get_shell_status(shell) {
                Ok(status) => statuses.push(status),
                Err(e) => {
                    tracing::warn!("Failed to get status for {}: {}", shell.as_str(), e);
                }
            }
        }

        Ok(statuses)
    }

    /// Install shell integration for a specific shell
    pub fn install_for_shell(&self, shell: ShellType, force: bool) -> Result<()> {
        println!(
            "Installing Hive AI shell integration for {}...",
            shell.as_str()
        );

        let completions = self.get_completions();
        let setup = self.get_setup();
        let hooks = self.get_hooks();

        // Install completions
        if force || !completions.is_installed(shell)? {
            completions.install(shell)?;
            println!("✅ Installed {} completions", shell.as_str());
        } else {
            println!("⏭️  {} completions already installed", shell.as_str());
        }

        // Setup PATH
        if force || !setup.is_path_configured(shell)? {
            setup.configure_path(shell)?;
            println!("✅ Configured PATH for {}", shell.as_str());
        } else {
            println!("⏭️  PATH already configured for {}", shell.as_str());
        }

        // Install hooks and aliases
        if force || !hooks.are_installed(shell)? {
            hooks.install(shell)?;
            println!("✅ Installed hooks and aliases for {}", shell.as_str());
        } else {
            println!(
                "⏭️  Hooks and aliases already installed for {}",
                shell.as_str()
            );
        }

        println!("🎉 Shell integration complete for {}!", shell.as_str());
        println!(
            "   Please restart your shell or run: source ~/.{}",
            match shell {
                ShellType::Bash => "bashrc",
                ShellType::Zsh => "zshrc",
                ShellType::Fish => "config/fish/config.fish",
                _ => "profile",
            }
        );

        Ok(())
    }

    /// Install shell integration for all detected shells
    pub fn install_all(&self, force: bool) -> Result<()> {
        let shells = self.detect_shells()?;

        if shells.is_empty() {
            return Err(anyhow::anyhow!("No supported shells detected"));
        }

        println!(
            "Detected shells: {}",
            shells
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        for shell in shells {
            if let Err(e) = self.install_for_shell(shell, force) {
                eprintln!("Warning: Failed to install for {}: {}", shell.as_str(), e);
            }
        }

        println!("\n🎉 Shell integration setup complete!");
        self.print_post_install_instructions()?;

        Ok(())
    }

    /// Print post-installation instructions
    fn print_post_install_instructions(&self) -> Result<()> {
        println!("\n📋 Post-Installation Instructions:");
        println!("   1. Restart your terminal or source your shell configuration");
        println!("   2. Test completions: hive <TAB>");
        println!("   3. Try aliases: ha (analyze), hq (ask), hp (plan)");
        println!("   4. Check status: hive shell status");
        println!("\n🔗 Quick Commands:");
        println!("   hive analyze .     # Analyze current directory");
        println!("   hive ask \"help\"     # Ask a question");
        println!("   hive plan \"task\"    # Create a plan");
        println!("   hive status        # Show system status");

        Ok(())
    }

    /// Uninstall shell integration
    pub fn uninstall(&self, preserve_config: bool) -> Result<()> {
        let uninstall = self.get_uninstall();
        uninstall.uninstall_all(preserve_config)?;
        Ok(())
    }

    /// Generate completion files for distribution
    pub fn generate_completion_files(&self, output_dir: &PathBuf) -> Result<()> {
        let completions = self.get_completions();
        completions.generate_all_files(output_dir)?;
        Ok(())
    }
}

// Convenient accessor methods
impl ShellIntegration {
    pub fn get_completions(&self) -> completions::ShellCompletions {
        completions::ShellCompletions::new(self.config.clone())
    }

    pub fn get_setup(&self) -> setup::ShellSetup {
        setup::ShellSetup::new(self.config.clone())
    }

    pub fn get_hooks(&self) -> hooks::ShellHooks {
        hooks::ShellHooks::new(self.config.clone())
    }

    pub fn get_uninstall(&self) -> uninstall::ShellUninstall {
        uninstall::ShellUninstall::new(self.config.clone())
    }
}

/// Utility functions for shell integration
pub mod utils {
    use super::*;

    /// Get the binary path for Hive AI
    pub fn get_binary_path() -> Result<PathBuf> {
        std::env::current_exe().context("Failed to get current executable path")
    }

    /// Check if running as administrator/root
    pub fn is_elevated() -> bool {
        #[cfg(unix)]
        {
            // Use std::os::unix for uid checking
            use std::os::unix::fs::MetadataExt;
            std::env::var("USER").map(|u| u == "root").unwrap_or(false)
        }

        #[cfg(windows)]
        {
            // For Windows, we'd need to check if running as administrator
            // This is a simplified check
            std::env::var("USERNAME")
                .map(|u| u.to_lowercase() == "administrator")
                .unwrap_or(false)
        }
    }

    /// Create directory with proper permissions
    pub fn create_directory_safe(path: &PathBuf) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        }
        Ok(())
    }

    /// Backup a file before modification
    pub fn backup_file(file_path: &PathBuf) -> Result<PathBuf> {
        let backup_path = file_path.with_extension(format!(
            "{}.hive-backup",
            file_path.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));

        if file_path.exists() {
            std::fs::copy(file_path, &backup_path)
                .with_context(|| format!("Failed to backup file: {}", file_path.display()))?;
        }

        Ok(backup_path)
    }

    /// Restore a file from backup
    pub fn restore_file(file_path: &PathBuf, backup_path: &PathBuf) -> Result<()> {
        if backup_path.exists() {
            std::fs::copy(backup_path, file_path)
                .with_context(|| format!("Failed to restore file: {}", file_path.display()))?;
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shell_type_parsing() {
        assert_eq!(ShellType::from_string("bash"), Some(ShellType::Bash));
        assert_eq!(ShellType::from_string("zsh"), Some(ShellType::Zsh));
        assert_eq!(ShellType::from_string("fish"), Some(ShellType::Fish));
        assert_eq!(
            ShellType::from_string("powershell"),
            Some(ShellType::PowerShell)
        );
        assert_eq!(ShellType::from_string("invalid"), None);
    }

    #[test]
    fn test_completion_filename() {
        assert_eq!(ShellType::Bash.completion_filename(), "hive");
        assert_eq!(ShellType::Zsh.completion_filename(), "_hive");
        assert_eq!(ShellType::Fish.completion_filename(), "hive.fish");
        assert_eq!(ShellType::PowerShell.completion_filename(), "hive.ps1");
    }

    #[test]
    fn test_utils_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test");

        assert!(utils::create_directory_safe(&test_dir).is_ok());
        assert!(test_dir.exists());
    }
}
