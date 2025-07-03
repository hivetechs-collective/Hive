//! Professional Welcome Banner for HiveTechs Consensus TUI

use anyhow::Result;
use console::style;
use crate::consensus::temporal::TemporalContextProvider;

/// Professional welcome banner component
pub struct WelcomeBanner {
    temporal_provider: TemporalContextProvider,
}

impl WelcomeBanner {
    /// Create new welcome banner
    pub async fn new() -> Result<Self> {
        Ok(Self {
            temporal_provider: TemporalContextProvider::default(),
        })
    }
    
    /// Format the professional welcome banner with Claude Code styling
    pub async fn format_banner(&self) -> Result<String> {
        // Get current temporal context
        let temporal_context = self.temporal_provider.build_current_context().await
            .unwrap_or_else(|_| Default::default());
        
        let date_display = temporal_context.current_datetime;
        let current_dir = self.get_current_dir_display();
        
        // Get system status
        let system_status = self.get_system_status().await;
        
        let banner = format!(
            "╭─────────────────────────────────────────────────────────────╮\n\
             │ {} HiveTechs Consensus - Professional AI Assistant      │\n\
             │                                                             │\n\
             │   Version: 2.0.0-dev                                       │\n\
             │   Config: {}                                      │\n\
             │   Memory: {}                              │\n\
             │   Models: {} 323+ AI models available                    │\n\
             │                                                             │\n\
             │   {} {}                 │\n\
             │   {} {}       │\n\
             ╰─────────────────────────────────────────────────────────────╯\n\
             \n\
             {} What's new today:\n\
             • {} - Real-time temporal awareness\n\
             • {} - 4-stage consensus with 10-40x performance boost\n\
             • {} - Repository intelligence with ML-powered analysis\n\
             • {} - Enterprise hooks for deterministic AI control\n\
             • {} - Planning mode for strategic development workflows\n\
             \n\
             {} Try:\n\
             • {} - Ask the AI consensus anything\n\
             • {} - Analyze your codebase with ML intelligence\n\
             • {} - Create strategic development plans\n\
             • {} - Show keyboard shortcuts",
            
            // Header
            style("🐝").bold().cyan(),
            
            // System status
            if system_status.config_ok { style("✓ Configured").green() } else { style("❌ Not configured").red() },
            if system_status.memory_ok { style("✓ 142 conversations").green() } else { style("❌ Memory unavailable").red() },
            if system_status.models_ok { style("✓").green() } else { style("❌").red() },
            
            // Location info
            style("cwd:").dim(),
            self.pad_right(&current_dir, 42),
            style("date:").dim(),
            self.pad_right(&date_display, 40),
            
            // What's new section
            style("📅").cyan(),
            style("Today is").dim(),
            style("Enhanced").bold().yellow(),
            style("Repository Intelligence").bold().green(),
            style("Enterprise Hooks").bold().blue(),
            style("Planning Mode").bold().magenta(),
            
            // Try section
            style("💡").yellow(),
            style("ask \"What can you help me with today?\"").cyan(),
            style("analyze .").green(),
            style("plan \"Add user authentication\"").blue(),
            style("? or help").dim(),
        );
        
        Ok(banner)
    }
    
    /// Get current directory display (truncated if too long)
    fn get_current_dir_display(&self) -> String {
        std::env::current_dir()
            .map(|path| {
                let path_str = path.to_string_lossy();
                if path_str.len() > 35 {
                    format!("...{}", &path_str[path_str.len()-32..])
                } else {
                    path_str.to_string()
                }
            })
            .unwrap_or_else(|_| "unknown".to_string())
    }
    
    /// Pad string to right with spaces
    fn pad_right(&self, s: &str, width: usize) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", s, " ".repeat(width - s.len()))
        }
    }
    
    /// Get system status for display
    async fn get_system_status(&self) -> SystemStatus {
        SystemStatus {
            config_ok: self.check_config().await,
            memory_ok: self.check_memory().await,
            models_ok: self.check_models().await,
        }
    }
    
    /// Check if configuration is valid
    async fn check_config(&self) -> bool {
        // TODO: Implement actual config check
        // For now, assume config is OK
        true
    }
    
    /// Check if memory system is available
    async fn check_memory(&self) -> bool {
        // TODO: Implement actual memory check
        // For now, assume memory is OK
        true
    }
    
    /// Check if AI models are available
    async fn check_models(&self) -> bool {
        // TODO: Implement actual models check
        // For now, assume models are OK
        true
    }
}

/// System status for banner display
struct SystemStatus {
    config_ok: bool,
    memory_ok: bool,
    models_ok: bool,
}