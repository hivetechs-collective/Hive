// Shell Completions Module
// Professional completion scripts for all supported shells

use anyhow::{Context, Result};
use std::path::PathBuf;

pub mod bash;
pub mod fish;
pub mod powershell;
pub mod zsh;

use super::{utils, ShellType};
use crate::core::config::Config;

/// Shell completions manager
pub struct ShellCompletions {
    config: Config,
}

impl ShellCompletions {
    /// Create new completions manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Install completion for a specific shell
    pub fn install(&self, shell: ShellType) -> Result<()> {
        let completion_dir = self.get_completion_directory(shell)?;
        let completion_file = completion_dir.join(shell.completion_filename());

        // Create completion directory if it doesn't exist
        utils::create_directory_safe(&completion_dir)?;

        // Generate and write completion content
        let content = self.generate_completion_content(shell)?;
        std::fs::write(&completion_file, content).with_context(|| {
            format!(
                "Failed to write completion file: {}",
                completion_file.display()
            )
        })?;

        // Set appropriate permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&completion_file)?.permissions();
            perms.set_mode(0o644);
            std::fs::set_permissions(&completion_file, perms)?;
        }

        tracing::info!(
            "Installed {} completions to {}",
            shell.as_str(),
            completion_file.display()
        );
        Ok(())
    }

    /// Check if completions are installed for a shell
    pub fn is_installed(&self, shell: ShellType) -> Result<bool> {
        let completion_dir = self.get_completion_directory(shell)?;
        let completion_file = completion_dir.join(shell.completion_filename());
        Ok(completion_file.exists())
    }

    /// Generate completion content for a shell
    fn generate_completion_content(&self, shell: ShellType) -> Result<String> {
        match shell {
            ShellType::Bash => Ok(bash::generate_bash_completions()),
            ShellType::Zsh => Ok(zsh::generate_zsh_completions()),
            ShellType::Fish => Ok(fish::generate_fish_completions()),
            ShellType::PowerShell => Ok(powershell::generate_powershell_completions()),
            ShellType::Elvish => Ok(self.generate_elvish_completions()),
        }
    }

    /// Get completion directory for a shell
    pub fn get_completion_directory(&self, shell: ShellType) -> Result<PathBuf> {
        match shell {
            ShellType::Bash => self.get_bash_completion_dir(),
            ShellType::Zsh => self.get_zsh_completion_dir(),
            ShellType::Fish => self.get_fish_completion_dir(),
            ShellType::PowerShell => self.get_powershell_completion_dir(),
            ShellType::Elvish => self.get_elvish_completion_dir(),
        }
    }

    /// Get bash completion directory
    fn get_bash_completion_dir(&self) -> Result<PathBuf> {
        // Try user-specific directory first
        if let Ok(home) = std::env::var("HOME") {
            let user_dir = PathBuf::from(&home).join(".bash_completion.d");
            if user_dir.exists() || utils::create_directory_safe(&user_dir).is_ok() {
                return Ok(user_dir);
            }

            // Try .local/share/bash-completion/completions
            let local_dir = PathBuf::from(&home)
                .join(".local")
                .join("share")
                .join("bash-completion")
                .join("completions");
            if local_dir.exists() || utils::create_directory_safe(&local_dir).is_ok() {
                return Ok(local_dir);
            }
        }

        // Try system directories
        for dir in &[
            "/etc/bash_completion.d",
            "/usr/local/etc/bash_completion.d",
            "/usr/share/bash-completion/completions",
        ] {
            let path = PathBuf::from(dir);
            if path.exists()
                && path
                    .metadata()
                    .map_or(false, |m| !m.permissions().readonly())
            {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!(
            "No suitable bash completion directory found"
        ))
    }

    /// Get zsh completion directory
    fn get_zsh_completion_dir(&self) -> Result<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            // Check common zsh completion directories in order of preference
            let candidate_dirs = [
                format!("{}/.zsh/completions", home),
                format!("{}/.oh-my-zsh/completions", home),
                format!("{}/.config/zsh/completions", home),
                format!("{}/.local/share/zsh/site-functions", home),
            ];

            for dir_str in &candidate_dirs {
                let dir = PathBuf::from(dir_str);
                if dir.exists() || utils::create_directory_safe(&dir).is_ok() {
                    return Ok(dir);
                }
            }
        }

        // Try system directories
        for dir in &[
            "/usr/local/share/zsh/site-functions",
            "/usr/share/zsh/site-functions",
        ] {
            let path = PathBuf::from(dir);
            if path.exists()
                && path
                    .metadata()
                    .map_or(false, |m| !m.permissions().readonly())
            {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!(
            "No suitable zsh completion directory found"
        ))
    }

    /// Get fish completion directory
    fn get_fish_completion_dir(&self) -> Result<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            let config_home =
                std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

            let fish_dir = PathBuf::from(config_home).join("fish").join("completions");

            if fish_dir.exists() || utils::create_directory_safe(&fish_dir).is_ok() {
                return Ok(fish_dir);
            }
        }

        // Try system directory
        let system_dir = PathBuf::from("/usr/share/fish/completions");
        if system_dir.exists()
            && system_dir
                .metadata()
                .map_or(false, |m| !m.permissions().readonly())
        {
            return Ok(system_dir);
        }

        Err(anyhow::anyhow!(
            "No suitable fish completion directory found"
        ))
    }

    /// Get PowerShell completion directory
    fn get_powershell_completion_dir(&self) -> Result<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            let profile_dir = if cfg!(windows) {
                PathBuf::from(home).join("Documents").join("PowerShell")
            } else {
                PathBuf::from(home).join(".config").join("powershell")
            };

            if profile_dir.exists() || utils::create_directory_safe(&profile_dir).is_ok() {
                return Ok(profile_dir);
            }
        }

        Err(anyhow::anyhow!(
            "No suitable PowerShell completion directory found"
        ))
    }

    /// Get Elvish completion directory
    fn get_elvish_completion_dir(&self) -> Result<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            let elvish_dir = PathBuf::from(home).join(".elvish").join("lib");
            if elvish_dir.exists() || utils::create_directory_safe(&elvish_dir).is_ok() {
                return Ok(elvish_dir);
            }
        }

        Err(anyhow::anyhow!(
            "No suitable Elvish completion directory found"
        ))
    }

    /// Generate Elvish completions (basic implementation)
    fn generate_elvish_completions(&self) -> String {
        r#"# Elvish completion for hive
# Generated by Hive AI

edit:completion:arg-completer[hive] = [@words]{
    fn spaces [n]{ repeat $n ' ' | str:join '' }
    fn cand [text desc]{
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    command = 'hive'
    for word $words[1:-1] {
        if (str:has-prefix $word '-') {
            continue
        }
        command = $command';'$word
    }
    completions = [
        &'hive'= {
            cand analyze 'Analyze and understand any repository'
            cand ask 'Ask the AI consensus a question'
            cand consensus 'Run 4-stage consensus analysis'
            cand plan 'Enter planning mode for complex tasks'
            cand execute 'Execute a previously created plan'
            cand improve 'Apply AI-suggested improvements to files'
            cand search 'Search for symbols in the codebase'
            cand memory 'Manage long-term memory and conversations'
            cand analytics 'Generate comprehensive analytics reports'
            cand tool 'Execute tools and tool chains'
            cand serve 'Start IDE integration servers'
            cand index 'Build semantic indices for fast search'
            cand config 'Manage configuration settings'
            cand trust 'Manage directory trust and security settings'
            cand hooks 'Manage enterprise hooks and automation'
            cand interactive 'Start interactive mode'
            cand tui 'Launch full TUI interface'
            cand status 'Show system status and health'
            cand completion 'Generate shell completions'
            cand self-update 'Self-update Hive AI binary'
        }
        &'hive;completion'= {
            cand bash 'Generate bash completions'
            cand zsh 'Generate zsh completions'
            cand fish 'Generate fish completions'
            cand powershell 'Generate PowerShell completions'
            cand elvish 'Generate Elvish completions'
        }
        &'hive;config'= {
            cand show 'Show current configuration'
            cand set 'Set a configuration value'
            cand get 'Get a configuration value'
            cand validate 'Validate configuration'
            cand reset 'Reset configuration to defaults'
            cand edit 'Edit configuration in default editor'
        }
        &'hive;memory'= {
            cand search 'Search conversation history'
            cand stats 'Show memory statistics'
            cand export 'Export conversation history'
            cand import 'Import conversation history'
            cand clear 'Clear memory'
            cand knowledge 'Manage knowledge graph'
        }
        &'hive;trust'= {
            cand list 'List all trusted directories'
            cand add 'Add a directory to trusted paths'
            cand remove 'Remove a directory from trusted paths'
            cand clear 'Clear all trusted paths'
            cand check 'Check trust status of a directory'
            cand security 'Manage security configuration'
            cand import 'Import trust settings from file'
            cand export 'Export trust settings to file'
        }
    ]
    $completions[$command]
}
"#
        .to_string()
    }

    /// Generate all completion files for distribution
    pub fn generate_all_files(&self, output_dir: &PathBuf) -> Result<()> {
        utils::create_directory_safe(output_dir)?;

        let shells = [
            ShellType::Bash,
            ShellType::Zsh,
            ShellType::Fish,
            ShellType::PowerShell,
            ShellType::Elvish,
        ];

        for shell in &shells {
            let content = self.generate_completion_content(*shell)?;
            let filename = shell.completion_filename();
            let file_path = output_dir.join(filename);

            std::fs::write(&file_path, content).with_context(|| {
                format!("Failed to write completion file: {}", file_path.display())
            })?;

            println!(
                "Generated {} completion: {}",
                shell.as_str(),
                file_path.display()
            );
        }

        Ok(())
    }

    /// Remove completions for a shell
    pub fn uninstall(&self, shell: ShellType) -> Result<()> {
        if let Ok(completion_dir) = self.get_completion_directory(shell) {
            let completion_file = completion_dir.join(shell.completion_filename());
            if completion_file.exists() {
                std::fs::remove_file(&completion_file).with_context(|| {
                    format!(
                        "Failed to remove completion file: {}",
                        completion_file.display()
                    )
                })?;
                tracing::info!(
                    "Removed {} completions from {}",
                    shell.as_str(),
                    completion_file.display()
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_completion_content() {
        let config = Config::default();
        let completions = ShellCompletions::new(config);

        assert!(completions
            .generate_completion_content(ShellType::Bash)
            .is_ok());
        assert!(completions
            .generate_completion_content(ShellType::Zsh)
            .is_ok());
        assert!(completions
            .generate_completion_content(ShellType::Fish)
            .is_ok());
        assert!(completions
            .generate_completion_content(ShellType::PowerShell)
            .is_ok());
    }

    #[test]
    fn test_generate_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        let completions = ShellCompletions::new(config);

        assert!(completions
            .generate_all_files(&temp_dir.path().to_path_buf())
            .is_ok());

        // Check that files were created
        assert!(temp_dir.path().join("hive").exists()); // bash
        assert!(temp_dir.path().join("_hive").exists()); // zsh
        assert!(temp_dir.path().join("hive.fish").exists()); // fish
        assert!(temp_dir.path().join("hive.ps1").exists()); // powershell
        assert!(temp_dir.path().join("hive.elv").exists()); // elvish
    }
}
