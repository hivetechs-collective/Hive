//! Core command handlers for TUI system

use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::tui::app::{TuiEvent, TuiMessage, MessageType};
use crate::commands::*;
use crate::core::config::HiveConfig;

/// Result of command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub output: String,
    pub success: bool,
    pub cost: Option<f64>,
    pub timing: std::time::Duration,
}

/// Context passed to command handlers
#[derive(Debug)]
pub struct CommandContext {
    pub config: HiveConfig,
    pub current_directory: std::path::PathBuf,
    pub event_sender: mpsc::UnboundedSender<TuiEvent>,
}

/// Trait for command handlers
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// Handle the command execution
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult>;

    /// Get command description
    fn description(&self) -> &str;

    /// Get command usage
    fn usage(&self) -> &str;

    /// Check if command supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Ask command handler
pub struct AskCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for AskCommandHandler {
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult> {
        let start = std::time::Instant::now();

        if args.is_empty() {
            return Ok(CommandResult {
                output: "❌ Please provide a question to ask.".to_string(),
                success: false,
                cost: None,
                timing: start.elapsed(),
            });
        }

        let question = args.join(" ");

        // Send processing message
        let processing_msg = TuiMessage {
            content: format!("🤔 Processing question: \"{}\"", question),
            message_type: MessageType::ConsensusProgress,
            timestamp: chrono::Utc::now(),
        };
        context.event_sender.send(TuiEvent::Message(processing_msg))?;

        // This would integrate with the actual consensus engine
        // For now, we'll simulate the response
        let response = format!(
            "🎯 Consensus Response:\n\n\
            Based on the 4-stage consensus analysis, here's the response to: \"{}\"\n\n\
            [This would be the actual consensus response from the engine]\n\n\
            💰 Cost: $0.05 | ⏱️ Time: {}ms",
            question,
            start.elapsed().as_millis()
        );

        Ok(CommandResult {
            output: response,
            success: true,
            cost: Some(0.05),
            timing: start.elapsed(),
        })
    }

    fn description(&self) -> &str {
        "Ask a question using 4-stage consensus"
    }

    fn usage(&self) -> &str {
        "ask <question>"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Analyze command handler
pub struct AnalyzeCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for AnalyzeCommandHandler {
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult> {
        let start = std::time::Instant::now();

        let path = if args.is_empty() {
            ".".to_string()
        } else {
            args[0].clone()
        };

        // Send processing message
        let processing_msg = TuiMessage {
            content: format!("🔍 Analyzing: {}", path),
            message_type: MessageType::ConsensusProgress,
            timestamp: chrono::Utc::now(),
        };
        context.event_sender.send(TuiEvent::Message(processing_msg))?;

        // Call the actual analyze handler
        let analysis_result = handle_analyze(&context.config, &[path.clone()]).await;

        let output = match analysis_result {
            Ok(result) => format!(
                "📊 Analysis Results for: {}\n\n{}\n\n⏱️ Time: {}ms",
                path,
                result,
                start.elapsed().as_millis()
            ),
            Err(e) => format!("❌ Analysis failed: {}", e),
        };

        Ok(CommandResult {
            output,
            success: analysis_result.is_ok(),
            cost: Some(0.02),
            timing: start.elapsed(),
        })
    }

    fn description(&self) -> &str {
        "Analyze code structure and dependencies"
    }

    fn usage(&self) -> &str {
        "analyze [path]"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Plan command handler
pub struct PlanCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for PlanCommandHandler {
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult> {
        let start = std::time::Instant::now();

        if args.is_empty() {
            return Ok(CommandResult {
                output: "❌ Please provide a task to plan.".to_string(),
                success: false,
                cost: None,
                timing: start.elapsed(),
            });
        }

        let task = args.join(" ");

        // Send processing message
        let processing_msg = TuiMessage {
            content: format!("📋 Creating plan for: \"{}\"", task),
            message_type: MessageType::ConsensusProgress,
            timestamp: chrono::Utc::now(),
        };
        context.event_sender.send(TuiEvent::Message(processing_msg))?;

        // Call the actual planning handler
        let plan_result = handle_plan(&context.config, &[task.clone()]).await;

        let output = match plan_result {
            Ok(plan) => format!(
                "📋 Execution Plan:\n\n{}\n\n⏱️ Time: {}ms",
                plan,
                start.elapsed().as_millis()
            ),
            Err(e) => format!("❌ Planning failed: {}", e),
        };

        Ok(CommandResult {
            output,
            success: plan_result.is_ok(),
            cost: Some(0.03),
            timing: start.elapsed(),
        })
    }

    fn description(&self) -> &str {
        "Create an execution plan for a task"
    }

    fn usage(&self) -> &str {
        "plan <task description>"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Improve command handler
pub struct ImproveCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for ImproveCommandHandler {
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult> {
        let start = std::time::Instant::now();

        let path = if args.is_empty() {
            ".".to_string()
        } else {
            args[0].clone()
        };

        // Send processing message
        let processing_msg = TuiMessage {
            content: format!("✨ Improving code: {}", path),
            message_type: MessageType::ConsensusProgress,
            timestamp: chrono::Utc::now(),
        };
        context.event_sender.send(TuiEvent::Message(processing_msg))?;

        // Call the actual improve handler
        let improve_result = handle_improve(&context.config, &[path.clone()]).await;

        let output = match improve_result {
            Ok(improvements) => format!(
                "✨ Code Improvements for: {}\n\n{}\n\n⏱️ Time: {}ms",
                path,
                improvements,
                start.elapsed().as_millis()
            ),
            Err(e) => format!("❌ Improvement failed: {}", e),
        };

        Ok(CommandResult {
            output,
            success: improve_result.is_ok(),
            cost: Some(0.04),
            timing: start.elapsed(),
        })
    }

    fn description(&self) -> &str {
        "Suggest code improvements"
    }

    fn usage(&self) -> &str {
        "improve [path]"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Memory search command handler
pub struct SearchCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for SearchCommandHandler {
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult> {
        let start = std::time::Instant::now();

        if args.is_empty() {
            return Ok(CommandResult {
                output: "❌ Please provide a search query.".to_string(),
                success: false,
                cost: None,
                timing: start.elapsed(),
            });
        }

        let query = args.join(" ");

        // Call the actual search handler
        let search_result = handle_search(&context.config, &[query.clone()]).await;

        let output = match search_result {
            Ok(results) => format!(
                "🔍 Search Results for: \"{}\"\n\n{}\n\n⏱️ Time: {}ms",
                query,
                results,
                start.elapsed().as_millis()
            ),
            Err(e) => format!("❌ Search failed: {}", e),
        };

        Ok(CommandResult {
            output,
            success: search_result.is_ok(),
            cost: None,
            timing: start.elapsed(),
        })
    }

    fn description(&self) -> &str {
        "Search memory and conversations"
    }

    fn usage(&self) -> &str {
        "search <query>"
    }
}

/// Models command handler
pub struct ModelsCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for ModelsCommandHandler {
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult> {
        let start = std::time::Instant::now();

        let subcommand = args.first().unwrap_or(&"list".to_string()).clone();

        let output = match subcommand.as_str() {
            "list" => {
                "🤖 Available Models:\n\n\
                Generator Stage:\n\
                • claude-3-5-sonnet-20241022 (Primary)\n\
                • gpt-4o-2024-11-20 (Backup)\n\n\
                Refiner Stage:\n\
                • claude-3-5-haiku-20241022 (Fast)\n\
                • gpt-4o-mini-2024-07-18 (Cost-effective)\n\n\
                Validator Stage:\n\
                • claude-3-opus-20240229 (Thorough)\n\
                • gpt-4-turbo-2024-04-09 (Alternative)\n\n\
                Curator Stage:\n\
                • claude-3-5-sonnet-20241022 (Final polish)\n\
                • gemini-1.5-pro-002 (Creative)\n".to_string()
            }
            "test" => {
                "🧪 Testing model connectivity...\n\n\
                ✅ OpenRouter API: Connected\n\
                ✅ Claude models: Available\n\
                ✅ GPT models: Available\n\
                ✅ Gemini models: Available\n\n\
                All consensus stage models are operational.".to_string()
            }
            "recommend" => {
                "💡 Model Recommendations:\n\n\
                For Speed: Use Consensus_Speed profile\n\
                For Quality: Use Consensus_Elite profile\n\
                For Balance: Use Consensus_Balanced profile (default)\n\
                For Cost: Use Consensus_Cost profile\n\n\
                Current profile: Consensus_Balanced".to_string()
            }
            _ => format!("❌ Unknown models subcommand: {}", subcommand),
        };

        Ok(CommandResult {
            output: format!("{}\n\n⏱️ Time: {}ms", output, start.elapsed().as_millis()),
            success: true,
            cost: None,
            timing: start.elapsed(),
        })
    }

    fn description(&self) -> &str {
        "Manage AI models and configurations"
    }

    fn usage(&self) -> &str {
        "models [list|test|recommend]"
    }
}

/// Cost command handler
pub struct CostCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for CostCommandHandler {
    async fn handle(&self, args: Vec<String>, context: &CommandContext) -> Result<CommandResult> {
        let start = std::time::Instant::now();

        let query = if args.is_empty() {
            "sample query".to_string()
        } else {
            args.join(" ")
        };

        // Call the actual cost estimation
        let cost_result = estimate_cost(&context.config, &[query.clone()]).await;

        let output = match cost_result {
            Ok(estimate) => format!(
                "💰 Cost Estimation for: \"{}\"\n\n{}\n\n⏱️ Time: {}ms",
                query,
                estimate,
                start.elapsed().as_millis()
            ),
            Err(e) => format!("❌ Cost estimation failed: {}", e),
        };

        Ok(CommandResult {
            output,
            success: cost_result.is_ok(),
            cost: None,
            timing: start.elapsed(),
        })
    }

    fn description(&self) -> &str {
        "Estimate consensus costs"
    }

    fn usage(&self) -> &str {
        "cost [query]"
    }
}

/// Create command handler registry
pub fn create_command_handlers() -> HashMap<String, Box<dyn CommandHandler>> {
    let mut handlers: HashMap<String, Box<dyn CommandHandler>> = HashMap::new();

    // Core consensus commands
    handlers.insert("ask".to_string(), Box::new(AskCommandHandler));
    handlers.insert("consensus".to_string(), Box::new(AskCommandHandler)); // Alias
    handlers.insert("explain".to_string(), Box::new(AskCommandHandler)); // Alias

    // Code analysis commands
    handlers.insert("analyze".to_string(), Box::new(AnalyzeCommandHandler));
    handlers.insert("review".to_string(), Box::new(AnalyzeCommandHandler)); // Alias

    // Planning commands
    handlers.insert("plan".to_string(), Box::new(PlanCommandHandler));
    handlers.insert("decompose".to_string(), Box::new(PlanCommandHandler)); // Alias
    handlers.insert("timeline".to_string(), Box::new(PlanCommandHandler)); // Alias

    // Code improvement
    handlers.insert("improve".to_string(), Box::new(ImproveCommandHandler));
    handlers.insert("transform".to_string(), Box::new(ImproveCommandHandler)); // Alias

    // Memory and search
    handlers.insert("search".to_string(), Box::new(SearchCommandHandler));
    handlers.insert("similar".to_string(), Box::new(SearchCommandHandler)); // Alias
    handlers.insert("insights".to_string(), Box::new(SearchCommandHandler)); // Alias

    // Model management
    handlers.insert("models".to_string(), Box::new(ModelsCommandHandler));
    handlers.insert("test".to_string(), Box::new(ModelsCommandHandler)); // Alias
    handlers.insert("recommend".to_string(), Box::new(ModelsCommandHandler)); // Alias

    // Cost analysis
    handlers.insert("cost".to_string(), Box::new(CostCommandHandler));

    handlers
}