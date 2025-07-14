// Shell Integration Commands for Hive AI
// Professional shell integration management

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

use crate::core::config::Config;
use crate::shell::{ShellIntegration, ShellIntegrationStatus, ShellType};

/// Shell integration subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum ShellCommands {
    /// Install shell integration for specified shell(s)
    Install {
        /// Target shell(s) to install integration for
        #[arg(value_enum)]
        shell: Option<ShellSelection>,

        /// Force installation even if already installed
        #[arg(long)]
        force: bool,
    },

    /// Setup PATH and environment variables automatically
    Setup,

    /// Show shell integration status for all shells
    Status {
        /// Show detailed status information
        #[arg(long)]
        detailed: bool,

        /// Output format
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,
    },

    /// Generate and install completion files
    Completions {
        /// Target shell for completions
        #[arg(value_enum)]
        shell: Option<ShellSelection>,

        /// Output directory for completion files
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Install completions for current shell
        #[arg(long)]
        install: bool,
    },

    /// Remove shell integration with optional config preservation
    Uninstall {
        /// Target shell(s) to uninstall integration from
        #[arg(value_enum)]
        shell: Option<ShellSelection>,

        /// Preserve configuration files
        #[arg(long)]
        preserve_config: bool,

        /// Validate uninstallation completeness
        #[arg(long)]
        validate: bool,
    },
}

/// Shell selection options
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ShellSelection {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
    All,
}

impl From<ShellSelection> for Option<ShellType> {
    fn from(selection: ShellSelection) -> Self {
        match selection {
            ShellSelection::Bash => Some(ShellType::Bash),
            ShellSelection::Zsh => Some(ShellType::Zsh),
            ShellSelection::Fish => Some(ShellType::Fish),
            ShellSelection::Powershell => Some(ShellType::PowerShell),
            ShellSelection::Elvish => Some(ShellType::Elvish),
            ShellSelection::All => None,
        }
    }
}

/// Output format options
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Table,
    Yaml,
}

/// Handle shell integration commands
pub fn handle_shell(cmd: ShellCommands, config: Config) -> Result<()> {
    let shell_integration = ShellIntegration::new(config);

    match cmd {
        ShellCommands::Install { shell, force } => handle_install(shell_integration, shell, force),
        ShellCommands::Setup => handle_setup(shell_integration),
        ShellCommands::Status { detailed, format } => {
            handle_status(shell_integration, detailed, format)
        }
        ShellCommands::Completions {
            shell,
            output,
            install,
        } => handle_completions(shell_integration, shell, output, install),
        ShellCommands::Uninstall {
            shell,
            preserve_config,
            validate,
        } => handle_uninstall(shell_integration, shell, preserve_config, validate),
    }
}

/// Handle install command
fn handle_install(
    shell_integration: ShellIntegration,
    shell: Option<ShellSelection>,
    force: bool,
) -> Result<()> {
    match shell {
        Some(ShellSelection::All) | None => {
            shell_integration.install_all(force)?;
        }
        Some(shell_sel) => {
            if let Some(shell_type) = shell_sel.into() {
                shell_integration.install_for_shell(shell_type, force)?;
            }
        }
    }

    Ok(())
}

/// Handle setup command
fn handle_setup(shell_integration: ShellIntegration) -> Result<()> {
    println!("🔧 Setting up Hive AI environment...");

    // Setup environment first
    let setup = shell_integration.get_setup();
    setup.setup_environment()?;

    // Detect shells and setup PATH
    let shells = shell_integration.detect_shells()?;

    if shells.is_empty() {
        println!("⚠️  No supported shells detected");
        println!("💡 Manual setup may be required");
        return Ok(());
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
        if let Err(e) = setup.configure_path(shell) {
            eprintln!(
                "Warning: Failed to configure PATH for {}: {}",
                shell.as_str(),
                e
            );
        } else {
            println!("✅ Configured PATH for {}", shell.as_str());
        }
    }

    // Verify installation
    if setup.verify_installation()? {
        println!("🎉 Environment setup completed successfully!");
        println!("   Please restart your terminal or source your shell configuration");
    } else {
        println!("⚠️  Setup completed but verification failed");
        println!("   Manual configuration may be required");
    }

    Ok(())
}

/// Handle status command
fn handle_status(
    shell_integration: ShellIntegration,
    detailed: bool,
    format: Option<OutputFormat>,
) -> Result<()> {
    let statuses = shell_integration.get_all_shell_status()?;

    if statuses.is_empty() {
        println!("ℹ️  No supported shells detected");
        return Ok(());
    }

    match format.unwrap_or(OutputFormat::Text) {
        OutputFormat::Text => display_status_text(&statuses, detailed),
        OutputFormat::Table => display_status_table(&statuses, detailed),
        OutputFormat::Json => display_status_json(&statuses, detailed)?,
        OutputFormat::Yaml => display_status_yaml(&statuses, detailed)?,
    }

    Ok(())
}

/// Display status in text format
fn display_status_text(statuses: &[ShellIntegrationStatus], detailed: bool) {
    println!("🐚 Shell Integration Status");
    println!("==========================");

    for status in statuses {
        let shell_icon = match status.shell {
            ShellType::Bash => "🟫",
            ShellType::Zsh => "🟦",
            ShellType::Fish => "🐟",
            ShellType::PowerShell => "💙",
            ShellType::Elvish => "🟣",
        };

        println!(
            "\n{} {} Shell",
            shell_icon,
            status.shell.as_str().to_uppercase()
        );
        println!(
            "   Completions: {}",
            if status.completions_installed {
                "✅"
            } else {
                "❌"
            }
        );
        println!(
            "   PATH:        {}",
            if status.path_configured { "✅" } else { "❌" }
        );
        println!(
            "   Hooks:       {}",
            if status.hooks_installed { "✅" } else { "❌" }
        );
        println!(
            "   Aliases:     {}",
            if status.aliases_configured {
                "✅"
            } else {
                "❌"
            }
        );

        if detailed {
            if let Some(config_path) = &status.config_location {
                println!("   Config:      {}", config_path.display());
            } else {
                println!("   Config:      Not found");
            }
        }
    }

    // Overall status summary
    let total_shells = statuses.len();
    let fully_integrated = statuses
        .iter()
        .filter(|s| {
            s.completions_installed
                && s.path_configured
                && s.hooks_installed
                && s.aliases_configured
        })
        .count();

    println!("\n📊 Summary");
    println!("   Shells detected: {}", total_shells);
    println!("   Fully integrated: {}", fully_integrated);

    if fully_integrated == total_shells {
        println!("   Status: 🎉 All shells fully integrated!");
    } else if fully_integrated > 0 {
        println!("   Status: ⚠️  Partial integration detected");
        println!("   💡 Run 'hive shell install' to complete setup");
    } else {
        println!("   Status: ❌ No shell integration found");
        println!("   💡 Run 'hive shell install' to get started");
    }
}

/// Display status in table format
fn display_status_table(statuses: &[ShellIntegrationStatus], detailed: bool) {
    println!("┌─────────────┬─────────────┬──────┬───────┬─────────┐");
    println!("│ Shell       │ Completions │ PATH │ Hooks │ Aliases │");
    println!("├─────────────┼─────────────┼──────┼───────┼─────────┤");

    for status in statuses {
        println!(
            "│ {:11} │ {:11} │ {:4} │ {:5} │ {:7} │",
            status.shell.as_str(),
            if status.completions_installed {
                "✅"
            } else {
                "❌"
            },
            if status.path_configured { "✅" } else { "❌" },
            if status.hooks_installed { "✅" } else { "❌" },
            if status.aliases_configured {
                "✅"
            } else {
                "❌"
            }
        );
    }

    println!("└─────────────┴─────────────┴──────┴───────┴─────────┘");

    if detailed {
        println!("\nConfiguration Files:");
        for status in statuses {
            if let Some(config_path) = &status.config_location {
                println!("  {}: {}", status.shell.as_str(), config_path.display());
            }
        }
    }
}

/// Display status in JSON format
fn display_status_json(statuses: &[ShellIntegrationStatus], _detailed: bool) -> Result<()> {
    let json =
        serde_json::to_string_pretty(statuses).context("Failed to serialize status to JSON")?;
    println!("{}", json);
    Ok(())
}

/// Display status in YAML format
fn display_status_yaml(statuses: &[ShellIntegrationStatus], _detailed: bool) -> Result<()> {
    let yaml = serde_yaml::to_string(statuses).context("Failed to serialize status to YAML")?;
    println!("{}", yaml);
    Ok(())
}

/// Handle completions command
fn handle_completions(
    shell_integration: ShellIntegration,
    shell: Option<ShellSelection>,
    output: Option<PathBuf>,
    install: bool,
) -> Result<()> {
    let completions = shell_integration.get_completions();

    if install {
        // Install completions for current shell or all shells
        match shell {
            Some(ShellSelection::All) | None => {
                let shells = shell_integration.detect_shells()?;
                for shell_type in shells {
                    completions.install(shell_type)?;
                    println!("✅ Installed {} completions", shell_type.as_str());
                }
            }
            Some(shell_sel) => {
                if let Some(shell_type) = shell_sel.into() {
                    completions.install(shell_type)?;
                    println!("✅ Installed {} completions", shell_type.as_str());
                }
            }
        }
        return Ok(());
    }

    if let Some(output_dir) = output {
        // Generate completion files to directory
        shell_integration.generate_completion_files(&output_dir)?;
        println!("✅ Generated completion files in: {}", output_dir.display());
    } else {
        // Show completion file locations
        println!("🐚 Shell Completion Locations");
        println!("=============================");

        let shells = if let Some(shell_sel) = shell {
            if let Some(shell_type) = shell_sel.into() {
                vec![shell_type]
            } else {
                shell_integration.detect_shells()?
            }
        } else {
            shell_integration.detect_shells()?
        };

        for shell_type in shells {
            let installed = completions.is_installed(shell_type)?;
            let status = if installed {
                "✅ Installed"
            } else {
                "❌ Not installed"
            };

            println!("{}: {}", shell_type.as_str(), status);

            if let Ok(comp_dir) = completions.get_completion_directory(shell_type) {
                let comp_file = comp_dir.join(shell_type.completion_filename());
                println!("   File: {}", comp_file.display());
            }
        }
    }

    Ok(())
}

/// Handle uninstall command
fn handle_uninstall(
    shell_integration: ShellIntegration,
    shell: Option<ShellSelection>,
    preserve_config: bool,
    validate: bool,
) -> Result<()> {
    let uninstall = shell_integration.get_uninstall();

    match shell {
        Some(ShellSelection::All) | None => {
            uninstall.uninstall_all(preserve_config)?;
        }
        Some(shell_sel) => {
            if let Some(shell_type) = shell_sel.into() {
                uninstall.uninstall_from_shell(shell_type, preserve_config)?;
                println!("✅ Uninstalled {} integration", shell_type.as_str());
            }
        }
    }

    if validate {
        println!("\n🔍 Validating uninstallation...");
        let is_clean = uninstall.validate_uninstall()?;

        if is_clean {
            println!("✅ Uninstallation validation passed");
        } else {
            println!("⚠️  Uninstallation validation found issues");
            return Err(anyhow::anyhow!("Uninstallation incomplete"));
        }
    }

    Ok(())
}

/// Generate manual installation instructions
pub fn generate_manual_instructions(shell: ShellType, config: Config) -> Result<String> {
    let shell_integration = ShellIntegration::new(config);
    let setup = shell_integration.get_setup();
    setup.generate_manual_instructions(shell)
}

/// Check if shell integration is properly installed
pub fn verify_installation(config: Config) -> Result<bool> {
    let shell_integration = ShellIntegration::new(config);
    let setup = shell_integration.get_setup();
    setup.verify_installation()
}

/// Get shell integration report
pub fn get_integration_report(config: Config) -> Result<String> {
    let shell_integration = ShellIntegration::new(config);
    let statuses = shell_integration.get_all_shell_status()?;

    let mut report = String::new();
    report.push_str("# Hive AI Shell Integration Report\n\n");

    for status in &statuses {
        report.push_str(&format!("## {} Shell\n", status.shell.as_str()));
        report.push_str(&format!(
            "- Completions: {}\n",
            if status.completions_installed {
                "✅"
            } else {
                "❌"
            }
        ));
        report.push_str(&format!(
            "- PATH: {}\n",
            if status.path_configured { "✅" } else { "❌" }
        ));
        report.push_str(&format!(
            "- Hooks: {}\n",
            if status.hooks_installed { "✅" } else { "❌" }
        ));
        report.push_str(&format!(
            "- Aliases: {}\n",
            if status.aliases_configured {
                "✅"
            } else {
                "❌"
            }
        ));

        if let Some(config_path) = &status.config_location {
            report.push_str(&format!("- Config: {}\n", config_path.display()));
        }

        report.push('\n');
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_selection_conversion() {
        assert_eq!(ShellSelection::Bash.into(), Some(ShellType::Bash));
        assert_eq!(ShellSelection::Zsh.into(), Some(ShellType::Zsh));
        assert_eq!(ShellSelection::Fish.into(), Some(ShellType::Fish));
        assert_eq!(
            ShellSelection::Powershell.into(),
            Some(ShellType::PowerShell)
        );
        assert_eq!(ShellSelection::Elvish.into(), Some(ShellType::Elvish));

        let all_selection: Option<ShellType> = ShellSelection::All.into();
        assert_eq!(all_selection, None);
    }

    #[test]
    fn test_output_format_variants() {
        // Test that all output format variants exist
        let _text = OutputFormat::Text;
        let _json = OutputFormat::Json;
        let _table = OutputFormat::Table;
        let _yaml = OutputFormat::Yaml;
    }
}
