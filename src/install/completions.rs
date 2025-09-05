//! Shell completion installation and management

use anyhow::{Context, Result};
use clap::CommandFactory;
use clap_complete::shells::{Bash, Fish, PowerShell, Zsh};
use clap_complete::{Generator, Shell};
use std::fs;
use std::path::PathBuf;
use tokio::fs as afs;

/// Shell completion manager
#[derive(Debug, Clone)]
pub struct CompletionManager {
    /// Base completion directory
    pub completion_dir: PathBuf,
}

impl CompletionManager {
    /// Create a new completion manager
    pub fn new(completion_dir: PathBuf) -> Result<Self> {
        Ok(Self { completion_dir })
    }

    /// Install completions for specified shells
    pub async fn install(&self, shells: &[String]) -> Result<()> {
        println!("🔧 Installing shell completions...");

        for shell_name in shells {
            if let Err(e) = self.install_for_shell(shell_name).await {
                eprintln!(
                    "⚠️  Warning: Failed to install completion for {}: {}",
                    shell_name, e
                );
            } else {
                println!("✅ Installed completion for {}", shell_name);
            }
        }

        Ok(())
    }

    /// Install completion for a specific shell
    async fn install_for_shell(&self, shell_name: &str) -> Result<()> {
        let shell = parse_shell(shell_name)?;
        let completion_content = self.generate_completion(shell)?;

        match shell {
            Shell::Bash => {
                let completion_file = self.get_bash_completion_path()?;
                self.write_completion_file(&completion_file, &completion_content)
                    .await?;
            }
            Shell::Zsh => {
                let completion_file = self.get_zsh_completion_path()?;
                self.write_completion_file(&completion_file, &completion_content)
                    .await?;
            }
            Shell::Fish => {
                let completion_file = self.get_fish_completion_path()?;
                self.write_completion_file(&completion_file, &completion_content)
                    .await?;
            }
            Shell::PowerShell => {
                let completion_file = self.get_powershell_completion_path()?;
                self.write_completion_file(&completion_file, &completion_content)
                    .await?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported shell: {}", shell_name));
            }
        }

        Ok(())
    }

    /// Generate completion content for a shell
    fn generate_completion(&self, shell: Shell) -> Result<String> {
        let mut cmd = crate::cli::Cli::command();
        let mut buf = Vec::new();

        match shell {
            Shell::Bash => {
                clap_complete::generate(Bash, &mut cmd, "hive", &mut buf);
            }
            Shell::Zsh => {
                clap_complete::generate(Zsh, &mut cmd, "hive", &mut buf);
            }
            Shell::Fish => {
                clap_complete::generate(Fish, &mut cmd, "hive", &mut buf);
            }
            Shell::PowerShell => {
                clap_complete::generate(PowerShell, &mut cmd, "hive", &mut buf);
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported shell"));
            }
        }

        Ok(String::from_utf8(buf)?)
    }

    /// Write completion file with proper permissions
    async fn write_completion_file(&self, path: &PathBuf, content: &str) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            afs::create_dir_all(parent).await?;
        }

        // Write completion file
        afs::write(path, content).await?;

        // Set readable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o644); // rw-r--r--
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Get bash completion file path
    fn get_bash_completion_path(&self) -> Result<PathBuf> {
        #[cfg(unix)]
        {
            // Try system-wide locations first
            let system_paths = vec![
                "/usr/local/share/bash-completion/completions/hive",
                "/usr/share/bash-completion/completions/hive",
                "/etc/bash_completion.d/hive",
            ];

            for path in system_paths {
                let path = PathBuf::from(path);
                if let Some(parent) = path.parent() {
                    if parent.exists() {
                        return Ok(path);
                    }
                }
            }

            // Fallback to user directory
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join(".bash_completion.d/hive"))
        }

        #[cfg(windows)]
        {
            // Windows doesn't have standard bash completion
            Ok(self.completion_dir.join("hive.bash"))
        }
    }

    /// Get zsh completion file path
    fn get_zsh_completion_path(&self) -> Result<PathBuf> {
        #[cfg(unix)]
        {
            // Try system-wide locations first
            let system_paths = vec![
                "/usr/local/share/zsh/site-functions/_hive",
                "/usr/share/zsh/site-functions/_hive",
                "/usr/share/zsh/vendor-completions/_hive",
            ];

            for path in system_paths {
                let path = PathBuf::from(path);
                if let Some(parent) = path.parent() {
                    if parent.exists() {
                        return Ok(path);
                    }
                }
            }

            // Fallback to user directory
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join(".zsh/completions/_hive"))
        }

        #[cfg(windows)]
        {
            Ok(self.completion_dir.join("_hive"))
        }
    }

    /// Get fish completion file path
    fn get_fish_completion_path(&self) -> Result<PathBuf> {
        #[cfg(unix)]
        {
            // Try system-wide locations first
            let system_paths = vec![
                "/usr/local/share/fish/completions/hive.fish",
                "/usr/share/fish/completions/hive.fish",
            ];

            for path in system_paths {
                let path = PathBuf::from(path);
                if let Some(parent) = path.parent() {
                    if parent.exists() {
                        return Ok(path);
                    }
                }
            }

            // Fallback to user directory
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join(".config/fish/completions/hive.fish"))
        }

        #[cfg(windows)]
        {
            Ok(self.completion_dir.join("hive.fish"))
        }
    }

    /// Get PowerShell completion file path
    fn get_powershell_completion_path(&self) -> Result<PathBuf> {
        #[cfg(windows)]
        {
            // Use Documents/PowerShell/Scripts
            let documents = std::env::var("USERPROFILE")?;
            Ok(PathBuf::from(documents).join("Documents/PowerShell/Scripts/hive-completion.ps1"))
        }

        #[cfg(unix)]
        {
            Ok(self.completion_dir.join("hive.ps1"))
        }
    }

    /// Remove all completions
    pub async fn remove(&self) -> Result<()> {
        println!("🗑️  Removing shell completions...");

        let shells = vec!["bash", "zsh", "fish", "powershell"];

        for shell in shells {
            if let Err(e) = self.remove_for_shell(shell).await {
                eprintln!(
                    "⚠️  Warning: Failed to remove completion for {}: {}",
                    shell, e
                );
            } else {
                println!("✅ Removed completion for {}", shell);
            }
        }

        Ok(())
    }

    /// Remove completion for a specific shell
    async fn remove_for_shell(&self, shell_name: &str) -> Result<()> {
        let completion_file = match shell_name {
            "bash" => self.get_bash_completion_path()?,
            "zsh" => self.get_zsh_completion_path()?,
            "fish" => self.get_fish_completion_path()?,
            "powershell" => self.get_powershell_completion_path()?,
            _ => return Err(anyhow::anyhow!("Unknown shell: {}", shell_name)),
        };

        if completion_file.exists() {
            afs::remove_file(&completion_file).await?;
        }

        Ok(())
    }

    /// Check which completions are installed
    pub async fn check_installed(&self) -> Result<Vec<String>> {
        let mut installed = Vec::new();
        let shells = vec!["bash", "zsh", "fish", "powershell"];

        for shell in shells {
            let completion_file = match shell {
                "bash" => self.get_bash_completion_path()?,
                "zsh" => self.get_zsh_completion_path()?,
                "fish" => self.get_fish_completion_path()?,
                "powershell" => self.get_powershell_completion_path()?,
                _ => continue,
            };

            if completion_file.exists() {
                installed.push(shell.to_string());
            }
        }

        Ok(installed)
    }

    /// Generate completion setup instructions
    pub fn generate_setup_instructions(&self) -> Result<CompletionInstructions> {
        Ok(CompletionInstructions {
            bash: self.generate_bash_instructions()?,
            zsh: self.generate_zsh_instructions()?,
            fish: self.generate_fish_instructions()?,
            powershell: self.generate_powershell_instructions()?,
        })
    }

    fn generate_bash_instructions(&self) -> Result<String> {
        let completion_file = self.get_bash_completion_path()?;
        Ok(format!(
            r#"# Bash completion for Hive AI
# Add this to your ~/.bashrc or ~/.bash_profile:

if [ -f "{}" ]; then
    source "{}"
fi

# Or enable bash completion globally:
sudo cp {} /usr/local/share/bash-completion/completions/hive
"#,
            completion_file.display(),
            completion_file.display(),
            completion_file.display()
        ))
    }

    fn generate_zsh_instructions(&self) -> Result<String> {
        let completion_file = self.get_zsh_completion_path()?;
        Ok(format!(
            r#"# Zsh completion for Hive AI
# Add this to your ~/.zshrc:

fpath=(~/.zsh/completions $fpath)
autoload -U compinit
compinit

# Or copy to system directory:
sudo cp {} /usr/local/share/zsh/site-functions/_hive
"#,
            completion_file.display()
        ))
    }

    fn generate_fish_instructions(&self) -> Result<String> {
        let completion_file = self.get_fish_completion_path()?;
        Ok(format!(
            r#"# Fish completion for Hive AI
# Completion file installed at: {}

# Fish will automatically load completions from:
# - ~/.config/fish/completions/
# - /usr/local/share/fish/completions/
# - /usr/share/fish/completions/

# No additional configuration needed for Fish shell.
"#,
            completion_file.display()
        ))
    }

    fn generate_powershell_instructions(&self) -> Result<String> {
        let completion_file = self.get_powershell_completion_path()?;
        Ok(format!(
            r#"# PowerShell completion for Hive AI
# Add this to your PowerShell profile:

if (Test-Path "{}") {{
    . "{}"
}}

# To find your profile location, run:
# $PROFILE
"#,
            completion_file.display(),
            completion_file.display()
        ))
    }
}

/// Shell completion setup instructions
#[derive(Debug, Clone)]
pub struct CompletionInstructions {
    pub bash: String,
    pub zsh: String,
    pub fish: String,
    pub powershell: String,
}

/// Parse shell name to Shell enum
fn parse_shell(shell_name: &str) -> Result<Shell> {
    match shell_name.to_lowercase().as_str() {
        "bash" => Ok(Shell::Bash),
        "zsh" => Ok(Shell::Zsh),
        "fish" => Ok(Shell::Fish),
        "powershell" | "pwsh" => Ok(Shell::PowerShell),
        _ => Err(anyhow::anyhow!("Unsupported shell: {}", shell_name)),
    }
}

/// Detect available shells on the system
pub fn detect_available_shells() -> Vec<String> {
    let mut shells = Vec::new();

    // Check for common shells
    let shell_paths = vec![
        ("/bin/bash", "bash"),
        ("/usr/bin/bash", "bash"),
        ("/bin/zsh", "zsh"),
        ("/usr/bin/zsh", "zsh"),
        ("/usr/local/bin/zsh", "zsh"),
        ("/usr/bin/fish", "fish"),
        ("/usr/local/bin/fish", "fish"),
    ];

    for (path, name) in shell_paths {
        if PathBuf::from(path).exists() {
            if !shells.contains(&name.to_string()) {
                shells.push(name.to_string());
            }
        }
    }

    // Check for PowerShell on Windows
    #[cfg(windows)]
    {
        if std::env::var("PWSH").is_ok() || std::env::var("POWERSHELL").is_ok() {
            shells.push("powershell".to_string());
        }
    }

    // Check SHELL environment variable
    if let Ok(shell_path) = std::env::var("SHELL") {
        if let Some(shell_name) = PathBuf::from(shell_path).file_name() {
            if let Some(name) = shell_name.to_str() {
                let name = name.to_lowercase();
                if !shells.contains(&name) {
                    shells.push(name);
                }
            }
        }
    }

    shells
}

// Re-export for CLI usage is handled in main.rs
