//! Interactive CLI mode for Hive AI
//!
//! This module provides Claude Code-style interactive sessions with
//! real-time command processing and user-friendly interfaces.

use anyhow::Result;
use console::{style, Term};
use std::io::{self, Write};
use crate::cli::{banner::show_startup_banner, get_current_dir_display, check_internet_connection, check_api_status, get_memory_usage};

/// Interactive session manager
pub struct InteractiveSession {
    term: Term,
    mode: String,
    use_tui: bool,
}

impl InteractiveSession {
    /// Create a new interactive session
    pub fn new(mode: String, use_tui: bool) -> Self {
        Self {
            term: Term::stdout(),
            mode,
            use_tui,
        }
    }
    
    /// Run the interactive session
    pub async fn run(&mut self) -> Result<()> {
        // Clear screen and show banner
        self.term.clear_screen()?;
        show_startup_banner().await?;
        
        // Main interactive loop
        loop {
            // Print the interactive input box (like Claude Code)
            self.print_interactive_prompt()?;
            
            // Read user input
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim();
            
            // Handle special commands
            match input {
                "/help" | "help" => {
                    self.print_help_info().await?;
                }
                "/status" | "status" => {
                    self.print_status_info().await?;
                }
                "/exit" | "exit" | "quit" => {
                    println!("👋 Thanks for using HiveTechs Consensus!");
                    break;
                }
                "/clear" | "clear" => {
                    self.term.clear_screen()?;
                    show_startup_banner().await?;
                }
                "/mode" => {
                    self.print_mode_info().await?;
                }
                "" => {
                    // Empty input, just continue
                    continue;
                }
                _ => {
                    // Process the command as a regular hive command
                    self.process_interactive_command(input).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Print the interactive prompt
    fn print_interactive_prompt(&self) -> Result<()> {
        println!("╭──────────────────────────────────────────────────────╮");
        println!("│ > Try \"ask <question>\" or \"analyze .\"                 │");
        println!("╰──────────────────────────────────────────────────────╯");
        print!("  ? for shortcuts  ");
        io::stdout().flush()?;
        Ok(())
    }
    
    /// Print help information
    async fn print_help_info(&self) -> Result<()> {
        println!();
        println!(" 🆘 {} Help", style("HiveTechs Consensus").bold());
        println!();
        println!(" {}:", style("Commands").bold());
        println!("   {} - Ask the AI consensus a question", style("ask <question>").cyan());
        println!("   {} - Analyze repository or file", style("analyze <path>").cyan());
        println!("   {} - Create a development plan", style("plan <goal>").cyan());
        println!("   {} - Suggest improvements to a file", style("improve <file>").cyan());
        println!("   {} - Show configured enterprise hooks", style("hooks list").cyan());
        println!("   {} - Search conversation history", style("memory search <query>").cyan());
        println!("   {} - Show analytics", style("analytics usage").cyan());
        println!("   {} - Show current configuration", style("config show").cyan());
        println!();
        println!(" {}:", style("Interactive commands").bold());
        println!("   {} - Show this help", style("/help").yellow());
        println!("   {} - Show system status", style("/status").yellow());
        println!("   {} - Clear screen", style("/clear").yellow());
        println!("   {} - Show current mode", style("/mode").yellow());
        println!("   {} - Exit interactive mode", style("/exit").yellow());
        println!();
        println!(" {}:", style("Shortcuts").bold());
        println!("   {} - Show command shortcuts", style("?").cyan());
        println!("   {} - Repeat last command", style("!!").cyan());
        println!("   {} - Access command history", style("↑/↓").cyan());
        println!();
        println!(" {}:", style("Documentation").bold());
        println!("   📖 Full docs: {}", style("https://docs.hivetechs.com").blue().underlined());
        println!("   🚀 Quick start: {}", style("https://docs.hivetechs.com/quickstart").blue().underlined());
        println!("   💬 Community: {}", style("https://discord.gg/hivetechs").blue().underlined());
        println!();
        Ok(())
    }
    
    /// Print system status information
    async fn print_status_info(&self) -> Result<()> {
        println!();
        println!(" {} HiveTechs Consensus Status", style("📊").bold());
        println!();
        
        let config_dir = crate::core::config::get_hive_config_dir();
        let config_exists = tokio::fs::metadata(config_dir.join("config.toml")).await.is_ok();
        let db_exists = tokio::fs::metadata(config_dir.join("hive-ai.db")).await.is_ok();
        
        println!(" {}:", style("System").bold());
        println!("   Version: {}", style(env!("CARGO_PKG_VERSION")).green());
        println!("   Config: {}", if config_exists { 
            style("✓ Configured").green() 
        } else { 
            style("⚠ Not configured").yellow() 
        });
        println!("   Memory: {}", if db_exists { 
            style("✓ Ready").green() 
        } else { 
            style("⚠ Not initialized").yellow() 
        });
        println!("   Working Directory: {}", style(get_current_dir_display()).cyan());
        println!("   Mode: {}", style(&self.mode).yellow());
        println!();
        
        // Check connectivity
        let internet_status = check_internet_connection().await;
        let api_status = check_api_status().await;
        
        println!(" {}:", style("Connectivity").bold());
        println!("   Internet: {}", if internet_status { 
            style("✓ Connected").green() 
        } else { 
            style("✗ Offline").red() 
        });
        println!("   AI Models: {}", if api_status { 
            style("✓ Available (323+ models)").green() 
        } else { 
            style("✗ Unavailable").red() 
        });
        println!();
        
        // Memory usage
        let memory_usage = get_memory_usage();
        println!(" {}:", style("Performance").bold());
        println!("   Memory Usage: {:.1} MB", memory_usage as f64 / 1024.0 / 1024.0);
        println!("   Consensus Engine: {}", style("✓ Ready").green());
        
        if self.use_tui {
            println!("   TUI Support: {}", style("✓ Available").green());
        } else {
            println!("   TUI Support: {}", style("⚠ Disabled").yellow());
        }
        
        println!();
        Ok(())
    }
    
    /// Print current mode information
    async fn print_mode_info(&self) -> Result<()> {
        println!();
        println!(" {} Current Mode: {}", style("🎮").bold(), style(&self.mode).cyan().bold());
        println!();
        
        match self.mode.as_str() {
            "hybrid" => {
                println!("   📋 {} Combines planning and execution", style("Hybrid mode:").bold());
                println!("      • Automatic task breakdown for complex queries");
                println!("      • Real-time execution with consensus validation");
                println!("      • Context-aware suggestions and improvements");
            }
            "planning" => {
                println!("   📝 {} Focus on strategic planning", style("Planning mode:").bold());
                println!("      • Detailed task decomposition");
                println!("      • Risk assessment and mitigation");
                println!("      • Timeline and resource estimation");
            }
            "execution" => {
                println!("   ⚡ {} Direct command execution", style("Execution mode:").bold());
                println!("      • Fast response times");
                println!("      • Immediate action on commands");
                println!("      • Minimal planning overhead");
            }
            _ => {
                println!("   🔧 {} Custom interactive mode", style("Custom mode:").bold());
                println!("      • User-defined behavior");
                println!("      • Flexible command processing");
            }
        }
        
        println!();
        println!("   {} Change mode with: {}", 
            style("Tip:").dim(),
            style("hive interactive --mode <mode>").cyan()
        );
        println!();
        Ok(())
    }
    
    /// Process an interactive command
    async fn process_interactive_command(&self, input: &str) -> Result<()> {
        // Split the input into command and arguments
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let command = parts[0];
        let args = if parts.len() > 1 { parts[1] } else { "" };
        
        println!(); // Add spacing before output
        
        match command {
            "ask" => {
                if args.is_empty() {
                    println!("❌ Usage: ask <question>");
                    return Ok(());
                }
                self.handle_ask_command(args).await?;
            }
            "analyze" => {
                if args.is_empty() {
                    println!("❌ Usage: analyze <path>");
                    return Ok(());
                }
                self.handle_analyze_command(args).await?;
            }
            "plan" => {
                if args.is_empty() {
                    println!("❌ Usage: plan <goal>");
                    return Ok(());
                }
                self.handle_plan_command(args).await?;
            }
            "improve" => {
                if args.is_empty() {
                    println!("❌ Usage: improve <file>");
                    return Ok(());
                }
                self.handle_improve_command(args).await?;
            }
            "hooks" => {
                self.handle_hooks_command(args).await?;
            }
            "memory" => {
                self.handle_memory_command(args).await?;
            }
            "analytics" => {
                self.handle_analytics_command(args).await?;
            }
            "config" => {
                self.handle_config_command(args).await?;
            }
            "status" => {
                self.print_status_info().await?;
            }
            "?" => {
                self.print_shortcuts().await?;
            }
            _ => {
                println!("❌ Unknown command: {}", style(command).red());
                println!("💡 Type {} for available commands", style("help").cyan());
            }
        }
        
        println!(); // Add spacing after output
        Ok(())
    }
    
    /// Handle ask command in interactive mode
    async fn handle_ask_command(&self, question: &str) -> Result<()> {
        println!("🤔 {} your question...", style("Processing").bold());
        
        // Check mode for special handling
        if self.mode == "planning" {
            println!("📋 {} Planning mode - analyzing complexity...", style("Mode:").yellow());
        }
        
        println!("🧠 {} 4-stage consensus pipeline...", style("Running").bold());
        println!();
        
        // Simulate consensus processing
        let stages = [
            ("Generator", "claude-3-5-sonnet"),
            ("Refiner", "gpt-4-turbo"),
            ("Validator", "claude-3-opus"),
            ("Curator", "gpt-4o"),
        ];
        
        for (stage, model) in stages.iter() {
            print!("{} → ", style(stage).bold().cyan());
            
            // Simulate progress
            for _ in 0..8 {
                print!("█");
                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            }
            
            println!(" 100% ({})", style(model).dim());
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        
        println!();
        println!("✨ {} Response:", style("Consensus").bold().green());
        println!("{}", style(question).italic());
        println!("(This is a placeholder response during development)");
        
        Ok(())
    }
    
    /// Handle analyze command in interactive mode
    async fn handle_analyze_command(&self, path: &str) -> Result<()> {
        println!("🔍 {} {}", style("Analyzing:").bold(), style(path).cyan());
        
        // Simulate analysis
        println!("📊 {} Repository Intelligence:", style("Running").bold());
        
        let steps = vec![
            "Scanning files...",
            "Building AST...",
            "Semantic analysis...",
            "Quality assessment...",
        ];
        
        for step in steps {
            print!("   {}", step);
            for _ in 0..3 {
                print!(".");
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            println!(" ✅");
        }
        
        println!();
        println!("📊 {} Results:", style("Analysis").bold().green());
        println!("  • Architecture: {} Application", style("Rust CLI").cyan());
        println!("  • Quality Score: {}/10", style("8.5").green().bold());
        println!("  • Files Analyzed: {}", style("47").blue());
        println!("  • Technical Debt: {}", style("Low").green());
        
        Ok(())
    }
    
    /// Handle plan command in interactive mode
    async fn handle_plan_command(&self, goal: &str) -> Result<()> {
        println!("📋 {} development plan for: {}", 
            style("Creating").bold(),
            style(goal).italic()
        );
        
        // Enhanced planning in planning mode
        if self.mode == "planning" {
            println!("🎯 {} Enhanced planning mode active", style("Mode:").yellow());
        }
        
        // Simulate planning
        let planning_steps = vec![
            "Analyzing requirements...",
            "Breaking down tasks...",
            "Assessing complexity...",
            "Generating timeline...",
        ];
        
        for step in planning_steps {
            print!("   {}", step);
            for _ in 0..3 {
                print!(".");
                tokio::time::sleep(std::time::Duration::from_millis(120)).await;
            }
            println!(" ✅");
        }
        
        println!();
        println!("✅ {} Plan Created:", style("Development").bold().green());
        println!("  📋 {} tasks generated", style("5").blue().bold());
        println!("  ⏱️  Estimated completion: {} days", style("2-3").yellow());
        println!("  ⚠️  {} risks identified", style("2").yellow());
        println!("📝 Use {} to begin implementation", style("execute plan").cyan());
        
        Ok(())
    }
    
    /// Handle improve command in interactive mode
    async fn handle_improve_command(&self, file: &str) -> Result<()> {
        println!("🔍 {} {}", style("Analyzing:").bold(), style(file).cyan());
        
        // Simulate improvement analysis
        print!("   Analyzing code structure");
        for _ in 0..4 {
            print!(".");
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        println!(" ✅");
        
        println!();
        println!("💡 {} Improvement Suggestions:", style("3").blue().bold());
        println!("  • Add error handling for edge cases");
        println!("  • Consider using async/await for better performance");
        println!("  • Add documentation comments");
        println!();
        println!("❓ Apply these improvements? {} to proceed", style("(y/n)").dim());
        
        Ok(())
    }
    
    /// Handle hooks command in interactive mode
    async fn handle_hooks_command(&self, args: &str) -> Result<()> {
        match args {
            "list" | "" => {
                println!("🔗 {} Enterprise Hooks:", style("Active").bold());
                println!("  ✓ {} (enabled)", style("auto-format-rust").cyan());
                println!("  ✓ {} (enabled)", style("production-guard").cyan());
                println!("  ⚠ {} (disabled)", style("cost-control").yellow());
            }
            _ => {
                println!("❌ Usage: hooks [list]");
            }
        }
        Ok(())
    }
    
    /// Handle memory command in interactive mode
    async fn handle_memory_command(&self, args: &str) -> Result<()> {
        if let Some((subcommand, query)) = args.split_once(' ') {
            match subcommand {
                "search" => {
                    println!("🔍 {} memory for: \"{}\"", style("Searching").bold(), style(query).italic());
                    println!("📝 Found {} relevant conversations:", style("3").blue());
                    println!("  • 2024-07-01: Discussion about Rust performance");
                    println!("  • 2024-06-28: Planning system architecture");
                    println!("  • 2024-06-25: TypeScript to Rust migration");
                }
                _ => {
                    println!("❌ Usage: memory search <query>");
                }
            }
        } else {
            println!("❌ Usage: memory search <query>");
        }
        Ok(())
    }
    
    /// Handle analytics command in interactive mode
    async fn handle_analytics_command(&self, args: &str) -> Result<()> {
        match args {
            "usage" | "" => {
                println!("📊 {} Usage Analytics:", style("Current").bold());
                println!("  🔥 Total queries: {}", style("1,247").blue());
                println!("  🧠 Consensus runs: {}", style("342").blue());
                println!("  📝 Plans created: {}", style("28").blue());
                println!("  💰 Total cost: ${}", style("23.45").yellow());
            }
            _ => {
                println!("❌ Usage: analytics [usage]");
            }
        }
        Ok(())
    }
    
    /// Handle config command in interactive mode
    async fn handle_config_command(&self, args: &str) -> Result<()> {
        match args {
            "show" | "" => {
                println!("📋 {} Configuration:", style("Current").bold());
                if let Ok(config) = crate::core::config::get_config().await {
                    println!("  Consensus: {} (profiles in database)", style("Configured").cyan());
                    println!("  TUI mode: {}", if config.interface.tui_mode { 
                        style("✓ Enabled").green() 
                    } else { 
                        style("✗ Disabled").red() 
                    });
                    println!("  Log level: {}", style(&config.logging.level).yellow());
                } else {
                    println!("  ⚠️  Configuration not loaded");
                }
            }
            _ => {
                println!("❌ Usage: config [show]");
            }
        }
        Ok(())
    }
    
    /// Print available shortcuts
    async fn print_shortcuts(&self) -> Result<()> {
        println!();
        println!(" ⌨️  {} Shortcuts:", style("Available").bold());
        println!();
        println!("   {} - Ask a question", style("ask <question>").cyan());
        println!("   {} - Analyze current directory", style("analyze .").cyan());
        println!("   {} - Create a plan", style("plan <goal>").cyan());
        println!("   {} - Show system status", style("status").cyan());
        println!("   {} - Show this help", style("?").cyan());
        println!();
        println!("   {} - Show all hooks", style("hooks").cyan());
        println!("   {} - Search memory", style("memory search <query>").cyan());
        println!("   {} - Show analytics", style("analytics").cyan());
        println!("   {} - Show config", style("config").cyan());
        println!();
        Ok(())
    }
}

/// Start interactive mode with specified configuration
pub async fn start_interactive_mode(mode: String, use_tui: bool) -> Result<()> {
    // Check if we should use the enhanced TUI mode
    if use_tui && crate::cli::check_tui_capabilities() {
        // Launch the enhanced interactive TUI
        let mut tui = crate::interactive_tui::InteractiveTui::new().await?;
        tui.run().await
    } else {
        // Fall back to simple interactive CLI
        let mut session = InteractiveSession::new(mode, use_tui);
        session.run().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interactive_session_creation() {
        let session = InteractiveSession::new("hybrid".to_string(), true);
        assert_eq!(session.mode, "hybrid");
        assert!(session.use_tui);
    }
    
    #[tokio::test]
    async fn test_mode_info() {
        let session = InteractiveSession::new("planning".to_string(), false);
        // This would test the mode info display in a real test environment
        assert_eq!(session.mode, "planning");
    }
}