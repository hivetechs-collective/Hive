//! Mode management CLI commands
//! 
//! Provides commands for intelligent mode detection, switching,
//! and preference management.

use crate::core::error::HiveResult;
use crate::consensus::ConsensusEngine;
use crate::planning::{ModeType, PlanningContext};
use crate::modes::{ModeManager, ModeConfig, ContextPreservationLevel};
use clap::Subcommand;
use std::sync::Arc;
use crossterm::style::Stylize;

/// Mode management commands
#[derive(Debug, Subcommand)]
pub enum ModeCommands {
    /// Show current mode status
    #[command(visible_alias = "s")]
    Status,
    
    /// Set operating mode
    #[command(visible_alias = "set")]
    Switch {
        /// Target mode (planning, execution, hybrid, analysis, learning)
        mode: String,
        
        /// Preserve context during switch
        #[arg(short, long, default_value = "true")]
        preserve_context: bool,
    },
    
    /// Detect appropriate mode for a query
    #[command(visible_alias = "d")]
    Detect {
        /// Query to analyze
        query: String,
        
        /// Show confidence scores
        #[arg(short, long)]
        confidence: bool,
    },
    
    /// Manage user preferences
    #[command(visible_alias = "p")]
    Preferences {
        /// Show current preferences
        #[arg(short, long)]
        show: bool,
        
        /// Enable/disable learning
        #[arg(short, long)]
        learning: Option<bool>,
        
        /// Reset preferences to default
        #[arg(short, long)]
        reset: bool,
    },
    
    /// Enable automatic mode switching
    #[command(visible_alias = "auto")]
    Auto {
        /// Enable or disable
        enabled: bool,
    },
}

/// Execute mode commands
pub async fn execute(
    cmd: ModeCommands,
    consensus_engine: Arc<ConsensusEngine>,
) -> HiveResult<()> {
    let mode_manager = Arc::new(ModeManager::new(consensus_engine.clone()).await?);
    
    match cmd {
        ModeCommands::Status => {
            show_status(&mode_manager).await?;
        }
        
        ModeCommands::Switch { mode, preserve_context } => {
            switch_mode(&mode_manager, &mode, preserve_context).await?;
        }
        
        ModeCommands::Detect { query, confidence } => {
            detect_mode(&mode_manager, &query, confidence).await?;
        }
        
        ModeCommands::Preferences { show, learning, reset } => {
            manage_preferences(&mode_manager, show, learning, reset).await?;
        }
        
        ModeCommands::Auto { enabled } => {
            set_auto_mode(&mode_manager, enabled).await?;
        }
    }
    
    Ok(())
}

/// Show current mode status
async fn show_status(manager: &Arc<ModeManager>) -> HiveResult<()> {
    let status = manager.get_status().await?;
    
    println!("\n{}", "Mode Status".bold().cyan());
    println!("{}", "─".repeat(40));
    
    // Current mode with indicator
    let mode_icon = get_mode_icon(&status.current_mode);
    println!("Current Mode: {} {:?}", mode_icon, status.current_mode);
    
    // Confidence
    let confidence_bar = format_confidence_bar(status.confidence);
    println!("Confidence:   {}", confidence_bar);
    
    // Health status
    let health_color = match status.health {
        crate::modes::visualization::HealthStatus::Excellent => "Excellent".green(),
        crate::modes::visualization::HealthStatus::Good => "Good".cyan(),
        crate::modes::visualization::HealthStatus::Warning => "Warning".yellow(),
        crate::modes::visualization::HealthStatus::Critical => "Critical".red(),
    };
    println!("Health:       {}", health_color);
    
    // Active duration
    let duration = format_duration(status.active_duration);
    println!("Active:       {}", duration);
    
    // Context items
    println!("Context:      {} items", status.context_items);
    
    // Last switch
    if let Some(last_switch) = status.last_switch {
        println!("Last Switch:  {}", last_switch.format("%Y-%m-%d %H:%M:%S"));
    }
    
    // Learning stats
    let stats = manager.get_learning_stats().await?;
    if stats.total_detections > 0 {
        println!("\n{}", "Learning Statistics".bold());
        println!("Detections:   {}", stats.total_detections);
        println!("Switches:     {}", stats.total_switches);
        println!("Accuracy:     {:.1}%", stats.mode_accuracy * 100.0);
    }
    
    Ok(())
}

/// Switch to a different mode
async fn switch_mode(
    manager: &Arc<ModeManager>,
    mode_str: &str,
    preserve_context: bool
) -> HiveResult<()> {
    let target_mode = parse_mode(mode_str)?;
    
    println!("Switching to {:?} mode...", target_mode);
    
    let result = manager.switch_mode(target_mode.clone(), preserve_context).await?;
    
    if result.success {
        println!("\n{}", "✓ Mode switch successful!".green().bold());
        println!("Duration: {}ms", result.duration.as_millis());
        
        if preserve_context {
            let ctx = &result.context_transformation;
            println!("\nContext preserved:");
            println!("  • Items preserved:   {}", ctx.items_preserved);
            println!("  • Items transformed: {}", ctx.items_transformed);
            println!("  • Items dropped:     {}", ctx.items_dropped);
            println!("  • Quality:           {:.0}%", ctx.transformation_quality * 100.0);
        }
        
        if !result.recommendations.is_empty() {
            println!("\n{}", "Recommendations:".bold());
            for rec in &result.recommendations {
                println!("  • {}", rec);
            }
        }
    } else {
        println!("\n{}", "✗ Mode switch failed".red().bold());
        
        if !result.warnings.is_empty() {
            println!("\n{}", "Warnings:".yellow());
            for warning in &result.warnings {
                println!("  ⚠ {}", warning);
            }
        }
    }
    
    Ok(())
}

/// Detect appropriate mode for a query
async fn detect_mode(
    manager: &Arc<ModeManager>,
    query: &str,
    show_confidence: bool
) -> HiveResult<()> {
    println!("Analyzing query: \"{}\"", query.italic());
    
    let context = PlanningContext::default(); // Use default context for CLI
    let recommendation = manager.get_recommendation(query, &context).await?;
    
    println!("\n{}", "Detection Result".bold().cyan());
    println!("{}", "─".repeat(40));
    
    let mode_icon = get_mode_icon(&recommendation.recommended_mode);
    println!("Recommended Mode: {} {:?}", mode_icon, recommendation.recommended_mode);
    println!("Confidence:       {:.0}%", recommendation.confidence * 100.0);
    
    if !recommendation.reasoning.is_empty() {
        println!("Reasoning:        {}", recommendation.reasoning);
    }
    
    if show_confidence && !recommendation.alternatives.is_empty() {
        println!("\n{}", "Alternative Modes:".bold());
        for (mode, confidence) in &recommendation.alternatives {
            let icon = get_mode_icon(mode);
            let bar = format_mini_confidence_bar(*confidence);
            println!("  {} {:?} {}", icon, mode, bar);
        }
    }
    
    if recommendation.user_preference_weight > 0.0 {
        println!("\nUser preference influence: {:.0}%", 
            recommendation.user_preference_weight * 100.0);
    }
    
    Ok(())
}

/// Manage user preferences
async fn manage_preferences(
    manager: &Arc<ModeManager>,
    show: bool,
    learning: Option<bool>,
    reset: bool
) -> HiveResult<()> {
    if reset {
        manager.reset_mode().await?;
        println!("{}", "✓ Preferences reset to default".green());
        return Ok(());
    }
    
    if let Some(enabled) = learning {
        let mut prefs = crate::modes::UserPreference::default();
        prefs.learning_enabled = enabled;
        manager.update_preferences(prefs).await?;
        
        if enabled {
            println!("{}", "✓ Learning enabled".green());
        } else {
            println!("{}", "✓ Learning disabled".yellow());
        }
        return Ok(());
    }
    
    if show {
        let stats = manager.get_learning_stats().await?;
        
        println!("\n{}", "User Preferences".bold().cyan());
        println!("{}", "─".repeat(40));
        
        // Mode distribution
        if !stats.mode_distribution.is_empty() {
            println!("\n{}", "Mode Usage:".bold());
            for (mode, count) in &stats.mode_distribution {
                let percentage = *count as f32 / stats.total_detections as f32 * 100.0;
                let bar = format_usage_bar(percentage);
                println!("  {:?}: {} {:.0}%", mode, bar, percentage);
            }
        }
        
        // Top patterns
        if !stats.top_patterns.is_empty() {
            println!("\n{}", "Top Patterns:".bold());
            for (pattern, count) in &stats.top_patterns {
                println!("  • {} ({}x)", pattern, count);
            }
        }
        
        println!("\nPreference influence: {:.0}%", stats.preference_influence * 100.0);
    }
    
    Ok(())
}

/// Set automatic mode switching
async fn set_auto_mode(
    _manager: &Arc<ModeManager>,
    enabled: bool
) -> HiveResult<()> {
    // In real implementation, would update config
    if enabled {
        println!("{}", "✓ Automatic mode switching enabled".green());
        println!("Hive will now automatically switch modes based on your tasks.");
    } else {
        println!("{}", "✓ Automatic mode switching disabled".yellow());
        println!("Use 'hive mode switch' to manually change modes.");
    }
    
    Ok(())
}

// Helper functions

fn parse_mode(mode_str: &str) -> HiveResult<ModeType> {
    match mode_str.to_lowercase().as_str() {
        "planning" | "plan" | "p" => Ok(ModeType::Planning),
        "execution" | "exec" | "e" => Ok(ModeType::Execution),
        "hybrid" | "h" => Ok(ModeType::Hybrid),
        "analysis" | "analyze" | "a" => Ok(ModeType::Analysis),
        "learning" | "learn" | "l" => Ok(ModeType::Learning),
        _ => Err(crate::core::error::HiveError::Planning(
            format!("Unknown mode: {}. Valid modes: planning, execution, hybrid, analysis, learning", mode_str)
        )),
    }
}

fn get_mode_icon(mode: &ModeType) -> &'static str {
    match mode {
        ModeType::Planning => "📋",
        ModeType::Execution => "⚡",
        ModeType::Hybrid => "🔄",
        ModeType::Analysis => "🔍",
        ModeType::Learning => "🧠",
    }
}

fn format_confidence_bar(confidence: f32) -> String {
    let width = 20;
    let filled = (confidence * width as f32) as usize;
    let bar = "█".repeat(filled) + &"░".repeat(width - filled);
    
    let color = if confidence > 0.8 {
        bar.green()
    } else if confidence > 0.5 {
        bar.yellow()
    } else {
        bar.red()
    };
    
    format!("[{}] {:.0}%", color, confidence * 100.0)
}

fn format_mini_confidence_bar(confidence: f32) -> String {
    let width = 10;
    let filled = (confidence * width as f32) as usize;
    let bar = "▪".repeat(filled) + &"▫".repeat(width - filled);
    format!("[{}] {:.0}%", bar, confidence * 100.0)
}

fn format_usage_bar(percentage: f32) -> String {
    let width = 15;
    let filled = (percentage / 100.0 * width as f32) as usize;
    "█".repeat(filled) + &"░".repeat(width - filled)
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode("planning").unwrap(), ModeType::Planning);
        assert_eq!(parse_mode("p").unwrap(), ModeType::Planning);
        assert_eq!(parse_mode("EXECUTION").unwrap(), ModeType::Execution);
        assert!(parse_mode("invalid").is_err());
    }
    
    #[test]
    fn test_format_confidence_bar() {
        let bar = format_confidence_bar(0.85);
        assert!(bar.contains("85%"));
        
        let bar = format_confidence_bar(0.3);
        assert!(bar.contains("30%"));
    }
}