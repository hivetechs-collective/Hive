//! Installation management commands

use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::install::{InstallationManager, InstallConfig, InstallMethod};

/// Installation command arguments
#[derive(Debug, Args)]
pub struct InstallArgs {
    #[command(subcommand)]
    pub command: InstallCommands,
}

/// Installation subcommands
#[derive(Debug, Subcommand)]
pub enum InstallCommands {
    /// Install Hive AI globally
    Install {
        /// Installation directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
        
        /// Skip shell completions
        #[arg(long)]
        no_completions: bool,
        
        /// Skip auto-updates
        #[arg(long)]
        no_auto_update: bool,
        
        /// Shells to install completions for
        #[arg(long, value_delimiter = ',')]
        shells: Option<Vec<String>>,
        
        /// Force installation even if already installed
        #[arg(long)]
        force: bool,
    },
    
    /// Uninstall Hive AI
    Uninstall {
        /// Keep configuration
        #[arg(long)]
        keep_config: bool,
        
        /// Keep cache
        #[arg(long)]
        keep_cache: bool,
    },
    
    /// Update Hive AI to latest version
    Update {
        /// Force update even if up to date
        #[arg(long)]
        force: bool,
        
        /// Check for updates only
        #[arg(long)]
        check_only: bool,
    },
    
    /// Show installation status
    Status,
    
    /// Repair installation
    Repair {
        /// Repair type
        #[arg(long, default_value = "all")]
        repair_type: String,
    },
    
    /// Validate installation
    Validate {
        /// Validation level
        #[arg(long, default_value = "standard")]
        level: String,
        
        /// Generate report
        #[arg(long)]
        report: bool,
    },
    
    /// Generate installer package
    Package {
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
        
        /// Target platform
        #[arg(long)]
        platform: Option<String>,
        
        /// Include completions
        #[arg(long, default_value = "true")]
        completions: bool,
    },
    
    /// Create NPM package
    Npm {
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
        
        /// Dry run (don't publish)
        #[arg(long)]
        dry_run: bool,
    },
}

/// Handle installation commands
pub async fn handle_install_command(args: InstallArgs) -> Result<()> {
    match args.command {
        InstallCommands::Install {
            dir,
            no_completions,
            no_auto_update,
            shells,
            force,
        } => {
            install_hive(dir, !no_completions, !no_auto_update, shells, force).await
        }
        
        InstallCommands::Uninstall { keep_config, keep_cache } => {
            uninstall_hive(keep_config, keep_cache).await
        }
        
        InstallCommands::Update { force, check_only } => {
            update_hive(force, check_only).await
        }
        
        InstallCommands::Status => {
            show_install_status().await
        }
        
        InstallCommands::Repair { repair_type } => {
            repair_installation(&repair_type).await
        }
        
        InstallCommands::Validate { level, report } => {
            validate_installation(&level, report).await
        }
        
        InstallCommands::Package { output, platform, completions } => {
            create_installer_package(output, platform, completions).await
        }
        
        InstallCommands::Npm { output, dry_run } => {
            create_npm_package(output, dry_run).await
        }
    }
}

/// Install Hive AI globally
async fn install_hive(
    dir: Option<PathBuf>,
    enable_completions: bool,
    auto_update: bool,
    shells: Option<Vec<String>>,
    force: bool,
) -> Result<()> {
    println!("🚀 Installing Hive AI globally...");
    
    let mut manager = InstallationManager::new().await?;
    
    // Check if already installed
    if manager.status.is_installed && !force {
        println!("✅ Hive AI is already installed (version {})", manager.status.version);
        println!("   Use --force to reinstall");
        return Ok(());
    }
    
    // Create installation configuration
    let mut config = InstallConfig::default();
    
    if let Some(install_dir) = dir {
        config.install_dir = install_dir;
    }
    
    config.enable_completions = enable_completions;
    config.auto_update = auto_update;
    
    if let Some(shell_list) = shells {
        config.shells = shell_list;
    }
    
    // Perform installation
    manager.install(config).await?;
    
    // Show post-installation information
    show_post_install_info(&manager).await?;
    
    Ok(())
}

/// Uninstall Hive AI
async fn uninstall_hive(keep_config: bool, keep_cache: bool) -> Result<()> {
    println!("🗑️  Uninstalling Hive AI...");
    
    let mut manager = InstallationManager::new().await?;
    
    if !manager.status.is_installed {
        println!("ℹ️  Hive AI is not installed globally");
        return Ok(());
    }
    
    // Confirm uninstallation
    println!("Are you sure you want to uninstall Hive AI? (y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if !input.trim().to_lowercase().starts_with('y') {
        println!("Uninstallation cancelled");
        return Ok(());
    }
    
    // Perform uninstallation
    manager.uninstall().await?;
    
    // Clean up optional components
    if !keep_config {
        cleanup_config(&manager).await?;
    }
    
    if !keep_cache {
        cleanup_cache(&manager).await?;
    }
    
    println!("✅ Hive AI uninstalled successfully!");
    
    if keep_config || keep_cache {
        println!("   Some data was preserved:");
        if keep_config {
            println!("   - Configuration: {}", manager.config_dir.display());
        }
        if keep_cache {
            println!("   - Cache: {}", manager.cache_dir.display());
        }
    }
    
    Ok(())
}

/// Update Hive AI
async fn update_hive(force: bool, check_only: bool) -> Result<()> {
    let mut manager = InstallationManager::new().await?;
    
    if !manager.status.is_installed {
        println!("❌ Hive AI is not installed globally");
        println!("   Run 'hive install install' to install");
        return Ok(());
    }
    
    if check_only {
        // Check for updates only
        match manager.check_for_updates().await? {
            Some(version) => {
                println!("✨ Update available: {} → {}", manager.status.version, version);
                println!("   Run 'hive install update' to install");
            }
            None => {
                println!("✅ Hive AI is up to date ({})", manager.status.version);
            }
        }
        return Ok(());
    }
    
    // Perform update
    let updated = manager.update().await?;
    
    if !updated && !force {
        println!("✅ Hive AI is already up to date!");
    }
    
    Ok(())
}

/// Show installation status
async fn show_install_status() -> Result<()> {
    let manager = InstallationManager::new().await?;
    let info = manager.get_info();
    
    println!("📊 Hive AI Installation Status");
    println!();
    
    if info.is_installed {
        println!("Status: ✅ Installed");
        println!("Version: {}", info.version);
        println!("Method: {:?}", info.method);
        println!("Installed: {}", info.installed_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Binary: {}", info.install_dir.join("hive").display());
        println!("Config: {}", info.config_dir.display());
        println!("Cache: {}", info.cache_dir.display());
        
        if !info.completions.is_empty() {
            println!("Completions: {}", info.completions.join(", "));
        }
        
        if info.auto_update {
            println!("Auto-update: ✅ Enabled");
        } else {
            println!("Auto-update: ❌ Disabled");
        }
    } else {
        println!("Status: ❌ Not installed");
        println!("To install: hive install install");
    }
    
    Ok(())
}

/// Repair installation
async fn repair_installation(repair_type: &str) -> Result<()> {
    println!("🔧 Repairing Hive AI installation...");
    
    let mut manager = InstallationManager::new().await?;
    
    match repair_type {
        "all" => {
            repair_binary(&mut manager).await?;
            repair_completions(&mut manager).await?;
            repair_configuration(&mut manager).await?;
        }
        "binary" => {
            repair_binary(&mut manager).await?;
        }
        "completions" => {
            repair_completions(&mut manager).await?;
        }
        "config" => {
            repair_configuration(&mut manager).await?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown repair type: {}", repair_type));
        }
    }
    
    println!("✅ Installation repair completed!");
    
    Ok(())
}

/// Validate installation
async fn validate_installation(level: &str, generate_report: bool) -> Result<()> {
    use crate::install::validation::{InstallationValidator, comprehensive_validate};
    
    println!("🔍 Validating Hive AI installation...");
    
    let manager = InstallationManager::new().await?;
    let result = comprehensive_validate(&manager.install_dir, &manager.config_dir).await?;
    
    if result.success {
        println!("✅ Installation validation passed!");
    } else {
        println!("❌ Installation validation failed!");
        
        if !result.errors.is_empty() {
            println!("\nErrors found:");
            for error in &result.errors {
                println!("  - {}: {}", error.code, error.message);
                if let Some(fix) = &error.fix {
                    println!("    Fix: {}", fix);
                }
            }
        }
        
        if !result.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &result.warnings {
                println!("  - {}: {}", warning.code, warning.message);
            }
        }
    }
    
    if generate_report {
        let validator = InstallationValidator::new(manager.install_dir, manager.config_dir)?;
        let report_path = validator.save_report(&result).await?;
        println!("\n📄 Validation report saved: {}", report_path.display());
    }
    
    Ok(())
}

/// Create installer package
async fn create_installer_package(
    output: PathBuf,
    platform: Option<String>,
    include_completions: bool,
) -> Result<()> {
    use crate::install::binary::InstallerBuilder;
    
    println!("📦 Creating installer package...");
    
    let current_exe = std::env::current_exe()?;
    let mut builder = InstallerBuilder::new(current_exe);
    
    builder = builder.with_completions(include_completions);
    
    let installer_path = if let Some(platform_name) = platform {
        output.join(format!("hive-installer-{}.sh", platform_name))
    } else {
        output.join("hive-installer.sh")
    };
    
    builder.build(&installer_path).await?;
    
    println!("✅ Installer package created: {}", installer_path.display());
    
    Ok(())
}

/// Create NPM package
async fn create_npm_package(output: PathBuf, dry_run: bool) -> Result<()> {
    use crate::install::npm::{create_npm_package, publish_npm_package};
    
    println!("📦 Creating NPM package...");
    
    // Create package structure
    create_npm_package(&output).await?;
    
    if dry_run {
        println!("✅ NPM package created (dry run): {}", output.display());
        println!("   Run without --dry-run to publish");
    } else {
        // Publish package
        publish_npm_package(&output, false).await?;
        println!("✅ NPM package published successfully!");
    }
    
    Ok(())
}

/// Show post-installation information
async fn show_post_install_info(manager: &InstallationManager) -> Result<()> {
    println!();
    println!("🎉 Installation completed successfully!");
    println!();
    println!("Next steps:");
    println!("  1. Restart your terminal or run: source ~/.bashrc");
    println!("  2. Verify installation: hive --version");
    println!("  3. Get started: hive quickstart");
    println!();
    
    // Show shell-specific instructions
    if !manager.status.completions.is_empty() {
        println!("Shell completions installed for: {}", manager.status.completions.join(", "));
        println!("Tab completion will be available after restarting your shell.");
        println!();
    }
    
    if manager.status.auto_update {
        println!("Auto-updates are enabled. Hive will check for updates daily.");
        println!("To disable: hive install update --no-auto-update");
        println!();
    }
    
    Ok(())
}

/// Repair binary installation
async fn repair_binary(manager: &mut InstallationManager) -> Result<()> {
    println!("🔧 Repairing binary installation...");
    
    use crate::install::binary::BinaryManager;
    let binary_manager = BinaryManager::new(manager.install_dir.clone())?;
    
    if binary_manager.is_installed() {
        // Reinstall binary
        binary_manager.uninstall().await?;
    }
    
    binary_manager.install().await?;
    
    println!("✅ Binary repaired");
    
    Ok(())
}

/// Repair shell completions
async fn repair_completions(manager: &mut InstallationManager) -> Result<()> {
    println!("🔧 Repairing shell completions...");
    
    use crate::install::completions::CompletionManager;
    let completion_manager = CompletionManager::new(manager.completion_dir.clone())?;
    
    // Remove existing completions
    completion_manager.remove().await?;
    
    // Reinstall completions
    let shells = vec!["bash".to_string(), "zsh".to_string(), "fish".to_string()];
    completion_manager.install(&shells).await?;
    
    println!("✅ Shell completions repaired");
    
    Ok(())
}

/// Repair configuration
async fn repair_configuration(manager: &mut InstallationManager) -> Result<()> {
    println!("🔧 Repairing configuration...");
    
    // Create default configuration if missing
    let config_path = manager.config_dir.join("config.toml");
    if !config_path.exists() {
        let default_config = include_str!("../../templates/default_config.toml");
        tokio::fs::write(config_path, default_config).await?;
    }
    
    println!("✅ Configuration repaired");
    
    Ok(())
}

/// Clean up configuration
async fn cleanup_config(manager: &InstallationManager) -> Result<()> {
    if manager.config_dir.exists() {
        tokio::fs::remove_dir_all(&manager.config_dir).await?;
    }
    Ok(())
}

/// Clean up cache
async fn cleanup_cache(manager: &InstallationManager) -> Result<()> {
    if manager.cache_dir.exists() {
        tokio::fs::remove_dir_all(&manager.cache_dir).await?;
    }
    Ok(())
}