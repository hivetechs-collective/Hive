// Shell Setup Module for Hive AI
// Automatic PATH configuration and environment setup

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;

use crate::core::config::Config;
use super::{ShellType, utils};

/// Shell setup manager for PATH and environment configuration
pub struct ShellSetup {
    config: Config,
}

impl ShellSetup {
    /// Create new shell setup manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Configure PATH for a specific shell
    pub fn configure_path(&self, shell: ShellType) -> Result<()> {
        let binary_path = utils::get_binary_path()?;
        let binary_dir = binary_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine binary directory"))?;

        let shell_config = self.get_shell_config_file(shell)?;
        
        // Backup the config file before modification
        if shell_config.exists() {
            utils::backup_file(&shell_config)?;
        }

        // Check if PATH is already configured
        if self.is_path_configured(shell)? {
            tracing::info!("PATH already configured for {}", shell.as_str());
            return Ok(());
        }

        // Add PATH configuration to shell config
        self.add_path_to_shell_config(shell, &shell_config, binary_dir)?;
        
        tracing::info!("Configured PATH for {} in {}", shell.as_str(), shell_config.display());
        Ok(())
    }

    /// Check if PATH is already configured for a shell
    pub fn is_path_configured(&self, shell: ShellType) -> Result<bool> {
        let binary_path = utils::get_binary_path()?;
        let binary_dir = binary_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine binary directory"))?;

        // Check if binary is in PATH
        if let Ok(which_output) = std::process::Command::new("which")
            .arg("hive")
            .output()
        {
            if which_output.status.success() {
                let which_path = String::from_utf8_lossy(&which_output.stdout);
                let which_path = which_path.trim();
                if which_path == binary_path.to_string_lossy() {
                    return Ok(true);
                }
            }
        }

        // Check shell config file for PATH entries
        if let Ok(shell_config) = self.get_shell_config_file(shell) {
            if shell_config.exists() {
                let content = read_to_string(&shell_config)?;
                let binary_dir_str = binary_dir.to_string_lossy();
                
                // Look for PATH exports containing our binary directory
                let path_patterns = [
                    format!("export PATH=\"{}:$PATH\"", binary_dir_str),
                    format!("export PATH='{}:$PATH'", binary_dir_str),
                    format!("export PATH={}:$PATH", binary_dir_str),
                    format!("$env:PATH = \"{};$env:PATH\"", binary_dir_str), // PowerShell
                    format!("set -gx PATH {} $PATH", binary_dir_str), // Fish
                ];

                for pattern in &path_patterns {
                    if content.contains(pattern) {
                        return Ok(true);
                    }
                }

                // Also check for Hive-specific markers
                if content.contains("# Hive AI PATH configuration") {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Get shell configuration file path
    fn get_shell_config_file(&self, shell: ShellType) -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not found")?;
        
        let config_file = match shell {
            ShellType::Bash => {
                // Prefer .bashrc, fall back to .bash_profile
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
                let fish_config = PathBuf::from(config_home).join("fish").join("config.fish");
                
                // Create fish config directory if it doesn't exist
                if let Some(parent) = fish_config.parent() {
                    utils::create_directory_safe(&parent.to_path_buf())?;
                }
                
                fish_config
            }
            ShellType::PowerShell => {
                if cfg!(windows) {
                    PathBuf::from(&home).join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1")
                } else {
                    PathBuf::from(&home).join(".config").join("powershell").join("Microsoft.PowerShell_profile.ps1")
                }
            }
            ShellType::Elvish => PathBuf::from(&home).join(".elvish").join("rc.elv"),
        };

        Ok(config_file)
    }

    /// Add PATH configuration to shell config file
    fn add_path_to_shell_config(&self, shell: ShellType, config_file: &PathBuf, binary_dir: &std::path::Path) -> Result<()> {
        // Create config file if it doesn't exist
        if !config_file.exists() {
            if let Some(parent) = config_file.parent() {
                utils::create_directory_safe(&parent.to_path_buf())?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(config_file)
            .with_context(|| format!("Failed to open config file: {}", config_file.display()))?;

        let binary_dir_str = binary_dir.to_string_lossy();
        let path_config = self.generate_path_config(shell, &binary_dir_str);

        writeln!(file, "\n{}", path_config)?;
        
        Ok(())
    }

    /// Generate PATH configuration for a specific shell
    fn generate_path_config(&self, shell: ShellType, binary_dir: &str) -> String {
        match shell {
            ShellType::Bash | ShellType::Zsh => {
                format!(r#"# Hive AI PATH configuration
# Added by Hive AI Shell Integration
if [[ ":$PATH:" != *":{}"* ]]; then
    export PATH="{}:$PATH"
fi

# Hive AI environment variables
export HIVE_CONFIG_DIR="$HOME/.hive"
export HIVE_LOG_LEVEL="info"

# Enable Hive AI shell integration
if command -v hive >/dev/null 2>&1; then
    # Context detection for better UX
    export HIVE_AUTO_DETECT_CONTEXT="true"
    
    # Optional: Auto-launch TUI for large terminals
    # export HIVE_AUTO_TUI="true"
fi"#, binary_dir, binary_dir)
            }
            ShellType::Fish => {
                format!(r#"# Hive AI PATH configuration
# Added by Hive AI Shell Integration
if not contains {} $PATH
    set -gx PATH {} $PATH
end

# Hive AI environment variables
set -gx HIVE_CONFIG_DIR "$HOME/.hive"
set -gx HIVE_LOG_LEVEL "info"

# Enable Hive AI shell integration
if command -v hive >/dev/null 2>&1
    # Context detection for better UX
    set -gx HIVE_AUTO_DETECT_CONTEXT "true"
    
    # Optional: Auto-launch TUI for large terminals
    # set -gx HIVE_AUTO_TUI "true"
end"#, binary_dir, binary_dir)
            }
            ShellType::PowerShell => {
                format!(r#"# Hive AI PATH configuration
# Added by Hive AI Shell Integration
$HiveBinaryDir = "{}"
if ($env:PATH -notlike "*$HiveBinaryDir*") {{
    $env:PATH = "$HiveBinaryDir;$env:PATH"
}}

# Hive AI environment variables
$env:HIVE_CONFIG_DIR = "$env:USERPROFILE\.hive"
$env:HIVE_LOG_LEVEL = "info"

# Enable Hive AI shell integration
if (Get-Command hive -ErrorAction SilentlyContinue) {{
    # Context detection for better UX
    $env:HIVE_AUTO_DETECT_CONTEXT = "true"
    
    # Optional: Auto-launch TUI for large terminals
    # $env:HIVE_AUTO_TUI = "true"
}}"#, binary_dir)
            }
            ShellType::Elvish => {
                format!(r#"# Hive AI PATH configuration
# Added by Hive AI Shell Integration
if (not (has-external hive)) {{
    set paths = [{}] $paths
}}

# Hive AI environment variables
set-env HIVE_CONFIG_DIR $E:HOME/.hive
set-env HIVE_LOG_LEVEL info

# Enable Hive AI shell integration
if (has-external hive) {{
    # Context detection for better UX
    set-env HIVE_AUTO_DETECT_CONTEXT true
    
    # Optional: Auto-launch TUI for large terminals
    # set-env HIVE_AUTO_TUI true
}}"#, binary_dir)
            }
        }
    }

    /// Setup environment variables for all shells
    pub fn setup_environment(&self) -> Result<()> {
        let hive_config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".hive");

        // Create Hive configuration directory
        utils::create_directory_safe(&hive_config_dir)?;

        // Create cache directory
        let cache_dir = hive_config_dir.join("cache");
        utils::create_directory_safe(&cache_dir)?;

        // Create logs directory
        let logs_dir = hive_config_dir.join("logs");
        utils::create_directory_safe(&logs_dir)?;

        // Set appropriate permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hive_config_dir)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hive_config_dir, perms)?;
        }

        tracing::info!("Environment setup completed");
        tracing::info!("Config directory: {}", hive_config_dir.display());

        Ok(())
    }

    /// Remove PATH configuration for a shell
    pub fn remove_path_configuration(&self, shell: ShellType) -> Result<()> {
        let shell_config = self.get_shell_config_file(shell)?;
        
        if !shell_config.exists() {
            return Ok(());
        }

        // Backup before modification
        utils::backup_file(&shell_config)?;

        let content = read_to_string(&shell_config)?;
        let mut lines: Vec<&str> = content.lines().collect();
        
        // Remove Hive AI configuration block
        let mut in_hive_block = false;
        let mut filtered_lines = Vec::new();
        
        for line in lines {
            if line.contains("# Hive AI PATH configuration") {
                in_hive_block = true;
                continue;
            }
            
            if in_hive_block {
                // End of block detection (empty line or different comment)
                if line.trim().is_empty() && 
                   filtered_lines.last().map_or(false, |l: &&str| l.trim().is_empty()) {
                    in_hive_block = false;
                }
                continue;
            }
            
            filtered_lines.push(line);
        }

        // Write cleaned content back
        std::fs::write(&shell_config, filtered_lines.join("\n"))?;
        
        tracing::info!("Removed PATH configuration for {} from {}", shell.as_str(), shell_config.display());
        Ok(())
    }

    /// Get current PATH entries
    pub fn get_current_path(&self) -> Vec<PathBuf> {
        if let Ok(path_env) = std::env::var("PATH") {
            let separator = if cfg!(windows) { ";" } else { ":" };
            path_env.split(separator)
                   .map(PathBuf::from)
                   .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if Hive AI binary is accessible
    pub fn verify_installation(&self) -> Result<bool> {
        match std::process::Command::new("hive").arg("--version").output() {
            Ok(output) => {
                let success = output.status.success();
                if success {
                    let version = String::from_utf8_lossy(&output.stdout);
                    tracing::info!("Hive AI installation verified: {}", version.trim());
                } else {
                    tracing::warn!("Hive AI binary found but failed to execute");
                }
                Ok(success)
            }
            Err(e) => {
                tracing::warn!("Failed to execute hive binary: {}", e);
                Ok(false)
            }
        }
    }

    /// Generate installation instructions for manual setup
    pub fn generate_manual_instructions(&self, shell: ShellType) -> Result<String> {
        let binary_path = utils::get_binary_path()?;
        let binary_dir = binary_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine binary directory"))?;
        
        let shell_config = self.get_shell_config_file(shell)?;
        let path_config = self.generate_path_config(shell, &binary_dir.to_string_lossy());

        Ok(format!(r#"Manual Installation Instructions for {}

1. Add the following to your shell configuration file ({}):

{}

2. Reload your shell configuration:
   {}

3. Verify installation:
   hive --version

4. Test completions:
   hive <TAB>

Note: You may need to restart your terminal or start a new shell session for changes to take effect.
"#, 
            shell.as_str(),
            shell_config.display(),
            path_config,
            match shell {
                ShellType::Bash => format!("source {}", shell_config.display()),
                ShellType::Zsh => format!("source {}", shell_config.display()),
                ShellType::Fish => format!("source {}", shell_config.display()),
                ShellType::PowerShell => format!("& {}", shell_config.display()),
                ShellType::Elvish => format!("use {}", shell_config.display()),
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    #[test]
    fn test_path_config_generation() {
        let config = Config::default();
        let setup = ShellSetup::new(config);
        
        let bash_config = setup.generate_path_config(ShellType::Bash, "/usr/local/bin");
        assert!(bash_config.contains("export PATH="));
        assert!(bash_config.contains("HIVE_CONFIG_DIR"));
        
        let fish_config = setup.generate_path_config(ShellType::Fish, "/usr/local/bin");
        assert!(fish_config.contains("set -gx PATH"));
        assert!(fish_config.contains("HIVE_CONFIG_DIR"));
        
        let ps_config = setup.generate_path_config(ShellType::PowerShell, "/usr/local/bin");
        assert!(ps_config.contains("$env:PATH"));
        assert!(ps_config.contains("HIVE_CONFIG_DIR"));
    }

    #[test]
    fn test_environment_setup() {
        let temp_dir = TempDir::new().unwrap();
        
        // Set temporary HOME
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let config = Config::default();
        let setup = ShellSetup::new(config);
        
        assert!(setup.setup_environment().is_ok());
        
        let hive_dir = temp_dir.path().join(".hive");
        assert!(hive_dir.exists());
        assert!(hive_dir.join("cache").exists());
        assert!(hive_dir.join("logs").exists());
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn test_get_current_path() {
        let config = Config::default();
        let setup = ShellSetup::new(config);
        
        let path_entries = setup.get_current_path();
        assert!(!path_entries.is_empty());
        
        // Should contain common system paths
        let path_strings: Vec<String> = path_entries.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        
        let has_bin = path_strings.iter().any(|p| p.contains("bin"));
        assert!(has_bin, "PATH should contain at least one 'bin' directory");
    }

    #[test]
    fn test_manual_instructions_generation() {
        let config = Config::default();
        let setup = ShellSetup::new(config);
        
        let instructions = setup.generate_manual_instructions(ShellType::Bash);
        assert!(instructions.is_ok());
        
        let instructions_text = instructions.unwrap();
        assert!(instructions_text.contains("Manual Installation Instructions"));
        assert!(instructions_text.contains("source"));
        assert!(instructions_text.contains("hive --version"));
    }
}