//! Hive AI CLI - Interactive codebase intelligence

use anyhow::Result;
use hive_ai::initialize;
use hive_ai::cli::{args::Cli, commands::handle_command, should_launch_tui, banner::show_startup_banner, init_cli};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    
    // Initialize CLI system
    init_cli().await?;
    
    // Check for TUI mode first (if no command specified)
    if cli.command.is_none() {
        if should_launch_tui().await {
            return launch_tui().await;
        } else {
            // Show startup banner and exit (Claude Code style)
            show_startup_banner().await?;
            return Ok(());
        }
    }
    
    // Initialize Hive core
    initialize().await?;
    
    // Set up logging based on verbosity
    setup_logging(&cli)?;
    
    // Handle the command
    if let Some(command) = cli.command {
        handle_command(command).await?;
    }
    
    Ok(())
}

/// Set up logging based on CLI arguments
fn setup_logging(cli: &Cli) -> Result<()> {
    let level = cli.verbosity();
    
    std::env::set_var("RUST_LOG", match level {
        tracing::Level::TRACE => "trace",
        tracing::Level::DEBUG => "debug", 
        tracing::Level::INFO => "info",
        tracing::Level::WARN => "warn",
        tracing::Level::ERROR => "error",
    });
    
    Ok(())
}

/// Launch TUI mode
async fn launch_tui() -> Result<()> {
    println!("🚀 Launching HiveTechs Consensus TUI...");
    
    // Check for required dependencies
    if !check_tui_dependencies() {
        println!("❌ TUI dependencies not available in this build");
        println!("💡 Install with: cargo install hive-ai --features tui");
        return Ok(());
    }
    
    // Initialize and run professional TUI
    match hive_ai::run_professional_tui().await {
        Ok(()) => {
            println!("👋 Thanks for using HiveTechs Consensus!");
        }
        Err(e) => {
            eprintln!("❌ TUI error: {}", e);
            eprintln!("💡 Falling back to CLI mode...");
            show_startup_banner().await?;
        }
    }
    
    Ok(())
}

/// Check if TUI dependencies are available
fn check_tui_dependencies() -> bool {
    // In real implementation, this would check if TUI features are compiled in
    true
}

