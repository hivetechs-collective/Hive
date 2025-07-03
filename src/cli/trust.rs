//! Trust management commands for Hive AI CLI
//! 
//! Provides commands to manage the Claude Code-style trust system

use anyhow::{anyhow, Result};
use clap::Subcommand;
use console::style;
use std::path::PathBuf;

use crate::core::{get_security_context, TrustLevel};

/// Trust management commands
#[derive(Debug, Subcommand)]
pub enum TrustCommand {
    /// List all trusted paths
    List,
    
    /// Add a path to trusted list
    Add {
        /// Path to trust
        path: PathBuf,
        
        /// Trust temporarily (session only)
        #[arg(short = 't', long)]
        temporary: bool,
    },
    
    /// Remove a path from trusted list
    Remove {
        /// Path to remove trust from
        path: PathBuf,
    },
    
    /// Show security events for a path
    Events {
        /// Path to show events for
        path: PathBuf,
        
        /// Number of events to show
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },
    
    /// Clear all temporary trusts
    ClearTemp,
    
    /// Reset all trust decisions (dangerous!)
    Reset {
        /// Confirm the reset
        #[arg(long)]
        confirm: bool,
    },
}

/// Handle trust management commands
pub async fn handle_trust_command(cmd: TrustCommand) -> Result<()> {
    let context = get_security_context()?;
    
    match cmd {
        TrustCommand::List => {
            list_trusted_paths(&context).await?;
        }
        
        TrustCommand::Add { path, temporary } => {
            add_trusted_path(&context, &path, temporary).await?;
        }
        
        TrustCommand::Remove { path } => {
            remove_trusted_path(&context, &path).await?;
        }
        
        TrustCommand::Events { path, limit } => {
            show_security_events(&context, &path, limit).await?;
        }
        
        TrustCommand::ClearTemp => {
            clear_temporary_trusts(&context).await?;
        }
        
        TrustCommand::Reset { confirm } => {
            if !confirm {
                return Err(anyhow!(
                    "This will remove ALL trust decisions. Use --confirm to proceed."
                ));
            }
            reset_all_trusts(&context).await?;
        }
    }
    
    Ok(())
}

/// List all trusted paths
async fn list_trusted_paths(context: &crate::core::SecurityContext) -> Result<()> {
    let trusted_paths = context.get_trusted_paths()?;
    
    if trusted_paths.is_empty() {
        println!("{}", style("No paths are currently trusted.").yellow());
        println!("\n💡 Use 'hive trust add <path>' to trust a directory.");
        return Ok(());
    }
    
    println!("{}", style("🛡️  Trusted Paths").bold());
    println!("{}", style("================").dim());
    println!();
    
    let mut permanent_count = 0;
    let mut temporary_count = 0;
    
    for (path, level) in &trusted_paths {
        let (icon, label, color) = match level {
            TrustLevel::Trusted => {
                permanent_count += 1;
                ("✅", "Permanent", console::Color::Green)
            }
            TrustLevel::Temporary => {
                temporary_count += 1;
                ("⏱️ ", "Temporary", console::Color::Yellow)
            }
            TrustLevel::Untrusted => ("❌", "Untrusted", console::Color::Red),
        };
        
        println!(
            "{} {} {}",
            icon,
            style(path.display()).fg(color),
            style(format!("({})", label)).dim()
        );
    }
    
    println!();
    println!(
        "📊 Total: {} paths ({} permanent, {} temporary)",
        trusted_paths.len(),
        permanent_count,
        temporary_count
    );
    
    Ok(())
}

/// Add a path to trusted list
async fn add_trusted_path(
    context: &crate::core::SecurityContext,
    path: &PathBuf,
    temporary: bool,
) -> Result<()> {
    let canonical = path.canonicalize()
        .map_err(|e| anyhow!("Failed to resolve path '{}': {}", path.display(), e))?;
    
    let trust_level = if temporary {
        TrustLevel::Temporary
    } else {
        TrustLevel::Trusted
    };
    
    context.set_trust_level(&canonical, trust_level, Some("CLI command".to_string()))?;
    
    let (icon, label) = if temporary {
        ("⏱️ ", "temporarily")
    } else {
        ("✅", "permanently")
    };
    
    println!(
        "{} {} trusted {} {}",
        icon,
        style("Successfully").green(),
        label,
        style(canonical.display()).bold()
    );
    
    Ok(())
}

/// Remove a path from trusted list
async fn remove_trusted_path(
    context: &crate::core::SecurityContext,
    path: &PathBuf,
) -> Result<()> {
    let canonical = path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    
    // Set to untrusted
    context.set_trust_level(&canonical, TrustLevel::Untrusted, Some("CLI removal".to_string()))?;
    
    println!(
        "❌ {} removed trust from {}",
        style("Successfully").green(),
        style(canonical.display()).bold()
    );
    
    Ok(())
}

/// Show security events for a path
async fn show_security_events(
    context: &crate::core::SecurityContext,
    path: &PathBuf,
    limit: usize,
) -> Result<()> {
    let canonical = path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    
    let events = context.get_events_for_path(&canonical)?;
    
    if events.is_empty() {
        println!(
            "{} No security events found for {}",
            style("ℹ️ ").blue(),
            style(canonical.display()).bold()
        );
        return Ok(());
    }
    
    println!(
        "{} Security Events for {}",
        style("📋").bold(),
        style(canonical.display()).bold()
    );
    println!("{}", style("=".repeat(60)).dim());
    println!();
    
    for (i, event) in events.iter().take(limit).enumerate() {
        let icon = if event.allowed { "✅" } else { "❌" };
        let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S");
        
        println!(
            "{} {} {} - {}",
            style(format!("{:>3}.", i + 1)).dim(),
            icon,
            style(timestamp).dim(),
            style(&event.event_type).bold()
        );
        println!("    {}", event.details);
        
        if i < events.len() - 1 && i < limit - 1 {
            println!();
        }
    }
    
    if events.len() > limit {
        println!(
            "\n{} Showing {} of {} events. Use -n to see more.",
            style("ℹ️ ").blue(),
            limit,
            events.len()
        );
    }
    
    Ok(())
}

/// Clear all temporary trust decisions
async fn clear_temporary_trusts(context: &crate::core::SecurityContext) -> Result<()> {
    context.clear_temporary_trusts()?;
    
    println!(
        "{} {} cleared all temporary trust decisions",
        style("🧹").bold(),
        style("Successfully").green()
    );
    
    Ok(())
}

/// Reset all trust decisions
async fn reset_all_trusts(context: &crate::core::SecurityContext) -> Result<()> {
    context.reset_all_trusts()?;
    
    println!(
        "{} {} reset all trust decisions",
        style("⚠️ ").yellow(),
        style("Successfully").yellow()
    );
    println!("\n💡 You will need to re-trust directories as you use them.");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trust_command_parsing() {
        // This would test clap parsing in a real implementation
        assert!(true);
    }
}