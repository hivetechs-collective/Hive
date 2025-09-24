//! Working Phase 1 main for Hive AI
//! This provides essential functionality that can be tested immediately

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;

mod essential_commands;

#[derive(Parser)]
#[command(name = "hive")]
#[command(about = "HiveTechs Consensus - AI-powered development assistant")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Hive in current directory
    Init {
        /// Target directory (defaults to current)
        path: Option<std::path::PathBuf>,
    },
    /// Show system status
    Status,
    /// Ask the AI consensus a question
    Ask {
        /// Question to ask
        question: String,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { path }) => {
            essential_commands::essential_init(path).await?;
        }
        Some(Commands::Status) => {
            essential_commands::essential_status().await?;
        }
        Some(Commands::Ask { question }) => {
            essential_commands::essential_ask(&question).await?;
        }
        Some(Commands::Config { command }) => {
            match command {
                ConfigCommands::Show => {
                    essential_commands::essential_config_show().await?;
                }
                ConfigCommands::Set { key, value } => {
                    essential_commands::essential_config_set(&key, &value).await?;
                }
                ConfigCommands::Get { key } => {
                    println!("Getting config key: {} (not yet implemented)", key);
                }
            }
        }
        None => {
            // Show banner
            show_banner().await?;
        }
    }

    Ok(())
}

async fn show_banner() -> Result<()> {
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ {} Welcome to HiveTechs Consensus!                     │", style("🐝").bold().yellow());
    println!("│                                                             │");
    println!("│   Phase 1 Essential Functionality Working ✅               │");
    println!("│                                                             │");
    println!("│   {} {}          │", style("cwd:").dim(), get_cwd_display());
    println!("╰─────────────────────────────────────────────────────────────╯");
    println!();

    println!(" {}:", style("Available commands").bold());
    println!("   {} - Initialize Hive in current directory", style("hive init").cyan());
    println!("   {} - Show system status", style("hive status").cyan());
    println!("   {} - Show configuration", style("hive config show").cyan());
    println!("   {} - Ask a question (demo)", style("hive ask \"<question>\"").cyan());
    println!();

    println!(" {}:", style("Phase 1 Progress").bold());
    println!("   ✅ CLI infrastructure working");
    println!("   ✅ Configuration system functional");
    println!("   ✅ Database initialization working");
    println!("   ✅ Essential commands operational");
    println!("   🚧 Full AI consensus (Phase 3)");
    println!("   🚧 TUI interface (Phase 7)");
    println!();

    Ok(())
}

fn get_cwd_display() -> String {
    std::env::current_dir()
        .map(|p| {
            let display = p.display().to_string();
            if display.len() > 40 {
                format!("...{}", &display[display.len()-37..])
            } else {
                display
            }
        })
        .unwrap_or_else(|_| "(unknown)".to_string())
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_cwd_display() {
        let display = get_cwd_display();
        assert!(!display.is_empty());
    }
}