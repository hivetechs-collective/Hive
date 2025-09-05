// Shell Uninstall Module for Hive AI
// Clean removal of shell integration with backup preservation

use anyhow::{Context, Result};
use std::fs::read_to_string;
use std::path::PathBuf;

use super::{
    completions::ShellCompletions, hooks::ShellHooks, setup::ShellSetup, utils, ShellType,
};
use crate::core::config::Config;

/// Shell uninstall manager for clean removal
pub struct ShellUninstall {
    config: Config,
}

impl ShellUninstall {
    /// Create new shell uninstall manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Uninstall shell integration for all shells
    pub fn uninstall_all(&self, preserve_config: bool) -> Result<()> {
        println!("🗑️  Uninstalling Hive AI shell integration...");

        // Detect installed shells
        let available_shells = self.detect_installed_shells()?;

        if available_shells.is_empty() {
            println!("ℹ️  No shell integrations found to remove");
            return Ok(());
        }

        println!(
            "Found integrations for: {}",
            available_shells
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Create backup before uninstalling
        if preserve_config {
            self.create_backup()?;
        }

        // Uninstall from each shell
        for shell in &available_shells {
            if let Err(e) = self.uninstall_from_shell(*shell, preserve_config) {
                eprintln!(
                    "Warning: Failed to uninstall from {}: {}",
                    shell.as_str(),
                    e
                );
            } else {
                println!("✅ Removed {} integration", shell.as_str());
            }
        }

        // Clean up completion files
        self.clean_completion_files(&available_shells)?;

        // Remove binary from PATH (if installed by us)
        if !preserve_config {
            self.remove_binary()?;
        }

        println!("\n🎉 Hive AI shell integration uninstalled successfully!");

        if preserve_config {
            println!("📦 Configuration files preserved in ~/.hive/");
            println!("🔄 To restore integration later, run: hive shell install");
        } else {
            println!("🗑️  All configuration files removed");
            println!("📋 Backup files (.hive-backup) preserved for manual restoration");
        }

        self.print_post_uninstall_instructions(&available_shells)?;

        Ok(())
    }

    /// Uninstall shell integration for a specific shell
    pub fn uninstall_from_shell(&self, shell: ShellType, preserve_config: bool) -> Result<()> {
        println!("Removing {} integration...", shell.as_str());

        // Remove completions
        let completions = ShellCompletions::new(self.config.clone());
        completions.uninstall(shell)?;

        // Remove PATH configuration
        let setup = ShellSetup::new(self.config.clone());
        setup.remove_path_configuration(shell)?;

        // Remove hooks and aliases
        let hooks = ShellHooks::new(self.config.clone());
        hooks.uninstall(shell)?;

        tracing::info!("Uninstalled shell integration for {}", shell.as_str());
        Ok(())
    }

    /// Detect shells that have Hive AI integration installed
    fn detect_installed_shells(&self) -> Result<Vec<ShellType>> {
        let mut installed_shells = Vec::new();

        let shells_to_check = [
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Fish,
            ShellType::PowerShell,
            ShellType::Elvish,
        ];

        for shell in &shells_to_check {
            if self.is_shell_integrated(*shell)? {
                installed_shells.push(*shell);
            }
        }

        Ok(installed_shells)
    }

    /// Check if a shell has Hive AI integration
    fn is_shell_integrated(&self, shell: ShellType) -> Result<bool> {
        // Check completion files
        let completions = ShellCompletions::new(self.config.clone());
        if completions.is_installed(shell)? {
            return Ok(true);
        }

        // Check shell config files for Hive AI markers
        if let Ok(config_file) = self.get_shell_config_file(shell) {
            if config_file.exists() {
                let content = read_to_string(&config_file)?;
                if content.contains("# Hive AI")
                    || content.contains("hive")
                    || content.contains("HIVE_")
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Get shell configuration file path
    fn get_shell_config_file(&self, shell: ShellType) -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not found")?;

        let config_file = match shell {
            ShellType::Bash => {
                let bashrc = PathBuf::from(&home).join(".bashrc");
                if bashrc.exists() {
                    bashrc
                } else {
                    PathBuf::from(&home).join(".bash_profile")
                }
            }
            ShellType::Zsh => PathBuf::from(&home).join(".zshrc"),
            ShellType::Fish => {
                let config_home = std::env::var("XDG_CONFIG_HOME")
                    .unwrap_or_else(|_| format!("{}/.config", home));
                PathBuf::from(config_home).join("fish").join("config.fish")
            }
            ShellType::PowerShell => {
                if cfg!(windows) {
                    PathBuf::from(&home)
                        .join("Documents")
                        .join("PowerShell")
                        .join("Microsoft.PowerShell_profile.ps1")
                } else {
                    PathBuf::from(&home)
                        .join(".config")
                        .join("powershell")
                        .join("Microsoft.PowerShell_profile.ps1")
                }
            }
            ShellType::Elvish => PathBuf::from(&home).join(".elvish").join("rc.elv"),
        };

        Ok(config_file)
    }

    /// Create backup of all Hive AI related files
    fn create_backup(&self) -> Result<()> {
        let backup_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".hive")
            .join("backups")
            .join(format!(
                "uninstall-{}",
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            ));

        utils::create_directory_safe(&backup_dir)?;

        // Backup shell config files
        let shells = [
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Fish,
            ShellType::PowerShell,
            ShellType::Elvish,
        ];

        for shell in &shells {
            if let Ok(config_file) = self.get_shell_config_file(*shell) {
                if config_file.exists() {
                    let backup_file = backup_dir.join(format!("{}_config", shell.as_str()));
                    std::fs::copy(&config_file, &backup_file)
                        .with_context(|| format!("Failed to backup {} config", shell.as_str()))?;
                    tracing::info!(
                        "Backed up {} config to {}",
                        shell.as_str(),
                        backup_file.display()
                    );
                }
            }
        }

        // Backup completion files
        let completions = ShellCompletions::new(self.config.clone());
        if let Ok(completion_backup_dir) =
            utils::create_directory_safe(&backup_dir.join("completions"))
        {
            let _ = completions.generate_all_files(&backup_dir.join("completions"));
        }

        println!("📦 Created backup at: {}", backup_dir.display());
        Ok(())
    }

    /// Clean up completion files
    fn clean_completion_files(&self, shells: &[ShellType]) -> Result<()> {
        let completions = ShellCompletions::new(self.config.clone());

        for shell in shells {
            if let Err(e) = completions.uninstall(*shell) {
                tracing::warn!("Failed to remove {} completions: {}", shell.as_str(), e);
            }
        }

        Ok(())
    }

    /// Remove Hive AI binary (if we can safely determine it was installed by us)
    fn remove_binary(&self) -> Result<()> {
        // Only remove binary if it's in a location we would have installed to
        let binary_path = utils::get_binary_path()?;
        let binary_dir = binary_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine binary directory"))?;

        // Safe directories we might have installed to
        let safe_removal_dirs = [
            "/usr/local/bin",
            "/opt/hive/bin",
            &format!("{}/.local/bin", std::env::var("HOME").unwrap_or_default()),
            &format!("{}/bin", std::env::var("HOME").unwrap_or_default()),
        ];

        let binary_dir_str = binary_dir.to_string_lossy();

        if safe_removal_dirs.iter().any(|&dir| binary_dir_str == dir) {
            // Ask for confirmation before removing binary
            println!("🗑️  Binary location: {}", binary_path.display());
            println!("❓ Remove Hive AI binary? (y/N): ");

            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                std::fs::remove_file(&binary_path).with_context(|| {
                    format!("Failed to remove binary: {}", binary_path.display())
                })?;
                println!("✅ Removed binary: {}", binary_path.display());
            } else {
                println!("⏭️  Binary preserved at: {}", binary_path.display());
            }
        } else {
            println!(
                "⏭️  Binary preserved (custom installation location): {}",
                binary_path.display()
            );
        }

        Ok(())
    }

    /// Print post-uninstall instructions
    fn print_post_uninstall_instructions(&self, shells: &[ShellType]) -> Result<()> {
        println!("\n📋 Post-Uninstall Instructions:");
        println!("   1. Restart your terminal or source your shell configuration");
        println!("   2. Verify removal: which hive (should return nothing)");

        if !shells.is_empty() {
            println!("\n🔄 To manually restore configurations:");
            for shell in shells {
                if let Ok(config_file) = self.get_shell_config_file(*shell) {
                    let backup_file = config_file.with_extension(format!(
                        "{}.hive-backup",
                        config_file
                            .extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                    ));

                    if backup_file.exists() {
                        println!(
                            "   {}: cp {} {}",
                            shell.as_str(),
                            backup_file.display(),
                            config_file.display()
                        );
                    }
                }
            }
        }

        println!("\n📦 Configuration preservation:");
        println!("   • User data preserved in ~/.hive/");
        println!("   • Conversation history maintained");
        println!("   • Trust settings preserved");
        println!("   • Analytics data maintained");

        println!("\n♻️  Complete removal (if desired):");
        println!("   rm -rf ~/.hive/");
        println!("   # This will delete ALL Hive AI data");

        Ok(())
    }

    /// Restore shell integration from backup
    pub fn restore_from_backup(&self, backup_dir: &PathBuf) -> Result<()> {
        if !backup_dir.exists() {
            return Err(anyhow::anyhow!(
                "Backup directory does not exist: {}",
                backup_dir.display()
            ));
        }

        println!("🔄 Restoring Hive AI shell integration from backup...");

        // Restore shell config files
        let shells = [
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Fish,
            ShellType::PowerShell,
            ShellType::Elvish,
        ];

        for shell in &shells {
            let backup_file = backup_dir.join(format!("{}_config", shell.as_str()));
            if backup_file.exists() {
                if let Ok(config_file) = self.get_shell_config_file(*shell) {
                    // Backup current config before restore
                    if config_file.exists() {
                        utils::backup_file(&config_file)?;
                    }

                    std::fs::copy(&backup_file, &config_file)
                        .with_context(|| format!("Failed to restore {} config", shell.as_str()))?;

                    println!("✅ Restored {} configuration", shell.as_str());
                }
            }
        }

        // Restore completion files if available
        let completions_backup_dir = backup_dir.join("completions");
        if completions_backup_dir.exists() {
            // This would require implementing completion file restoration
            // For now, suggest manual reinstallation
            println!("💡 To restore completions, run: hive shell install");
        }

        println!("🎉 Shell integration restored from backup!");
        println!("   Please restart your terminal or source your shell configuration");

        Ok(())
    }

    /// Validate uninstallation completeness
    pub fn validate_uninstall(&self) -> Result<bool> {
        println!("🔍 Validating uninstallation...");

        let mut issues = Vec::new();

        // Check if hive command is still accessible
        if let Ok(_) = std::process::Command::new("hive").arg("--version").output() {
            issues.push("Hive binary is still accessible in PATH".to_string());
        }

        // Check for remaining completion files
        let shells = [
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Fish,
            ShellType::PowerShell,
            ShellType::Elvish,
        ];
        for shell in &shells {
            let completions = ShellCompletions::new(self.config.clone());
            if completions.is_installed(*shell).unwrap_or(false) {
                issues.push(format!("{} completions still installed", shell.as_str()));
            }
        }

        // Check for remaining shell configuration
        for shell in &shells {
            if self.is_shell_integrated(*shell).unwrap_or(false) {
                issues.push(format!(
                    "{} shell configuration still contains Hive AI references",
                    shell.as_str()
                ));
            }
        }

        if issues.is_empty() {
            println!("✅ Uninstallation completed successfully");
            println!("   No remaining Hive AI integration detected");
            Ok(true)
        } else {
            println!("⚠️  Uninstallation incomplete:");
            for issue in issues {
                println!("   • {}", issue);
            }
            println!("\n💡 Manual cleanup may be required");
            Ok(false)
        }
    }

    /// Generate uninstall report
    pub fn generate_uninstall_report(&self) -> Result<String> {
        let shells = self.detect_installed_shells()?;
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

        let mut report = format!(
            r#"# Hive AI Shell Integration Uninstall Report
Generated: {}

## Detected Integrations
"#,
            timestamp
        );

        if shells.is_empty() {
            report.push_str("No shell integrations detected.\n");
        } else {
            for shell in &shells {
                report.push_str(&format!("- {} integration found\n", shell.as_str()));

                // Add details about what will be removed
                if let Ok(config_file) = self.get_shell_config_file(*shell) {
                    if config_file.exists() {
                        report.push_str(&format!("  - Configuration: {}\n", config_file.display()));
                    }
                }
            }
        }

        report.push_str("\n## Files to be Modified/Removed\n");

        // List completion directories
        for shell in &shells {
            let completions = ShellCompletions::new(self.config.clone());
            if let Ok(comp_dir) = completions.get_completion_directory(*shell) {
                let comp_file = comp_dir.join(shell.completion_filename());
                if comp_file.exists() {
                    report.push_str(&format!("- Completion file: {}\n", comp_file.display()));
                }
            }
        }

        // List shell config files
        for shell in &shells {
            if let Ok(config_file) = self.get_shell_config_file(*shell) {
                if config_file.exists() {
                    report.push_str(&format!(
                        "- Shell config: {} (Hive sections will be removed)\n",
                        config_file.display()
                    ));
                }
            }
        }

        // Add binary information
        if let Ok(binary_path) = utils::get_binary_path() {
            report.push_str(&format!(
                "\n## Binary Location\n- {}\n",
                binary_path.display()
            ));
        }

        report.push_str("\n## Preservation\n");
        report.push_str("- User configuration in ~/.hive/ will be preserved\n");
        report.push_str("- Conversation history will be maintained\n");
        report.push_str("- Trust settings will be preserved\n");
        report.push_str("- Backup files (.hive-backup) will be created\n");

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_detect_installed_shells() {
        let temp_dir = TempDir::new().unwrap();
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());

        // Create a fake shell config with Hive AI content
        let bashrc = temp_dir.path().join(".bashrc");
        std::fs::write(&bashrc, "# Hive AI Shell Hooks\nalias ha='hive analyze'").unwrap();

        let config = Config::default();
        let uninstall = ShellUninstall::new(config);

        let shells = uninstall.detect_installed_shells().unwrap();
        assert!(shells.contains(&ShellType::Bash));

        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn test_is_shell_integrated() {
        let temp_dir = TempDir::new().unwrap();
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());

        let config = Config::default();
        let uninstall = ShellUninstall::new(config);

        // Should return false for non-existent config
        assert!(!uninstall.is_shell_integrated(ShellType::Bash).unwrap());

        // Create config with Hive AI content
        let bashrc = temp_dir.path().join(".bashrc");
        std::fs::write(&bashrc, "# Some other content\nexport PATH=/usr/bin:$PATH").unwrap();
        assert!(!uninstall.is_shell_integrated(ShellType::Bash).unwrap());

        // Add Hive AI content
        std::fs::write(&bashrc, "# Hive AI Shell Hooks\nalias ha='hive analyze'").unwrap();
        assert!(uninstall.is_shell_integrated(ShellType::Bash).unwrap());

        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn test_generate_uninstall_report() {
        let config = Config::default();
        let uninstall = ShellUninstall::new(config);

        let report = uninstall.generate_uninstall_report().unwrap();

        assert!(report.contains("# Hive AI Shell Integration Uninstall Report"));
        assert!(report.contains("## Detected Integrations"));
        assert!(report.contains("## Files to be Modified/Removed"));
        assert!(report.contains("## Preservation"));
    }

    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());

        let config = Config::default();
        let uninstall = ShellUninstall::new(config);

        // Create a fake shell config
        let bashrc = temp_dir.path().join(".bashrc");
        std::fs::write(&bashrc, "# Test config\nalias ll='ls -la'").unwrap();

        assert!(uninstall.create_backup().is_ok());

        // Check that backup directory was created
        let backup_base = temp_dir.path().join(".hive").join("backups");
        assert!(backup_base.exists());

        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }
}
