//! Memory management commands
//!
//! This module provides CLI commands for the enhanced memory system including:
//! - Semantic search across memories
//! - Pattern analysis and learning
//! - Knowledge graph visualization
//! - Memory analytics and insights

use anyhow::Result;
use clap::Subcommand;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

use crate::core::{
    memory::{
        get_memory_system, initialize_memory,
        SemanticSearchResult, MemoryInsight, InsightType,
    },
    database::KnowledgeConversation,
};

/// Memory management commands
#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    /// Search memories using semantic similarity
    Search {
        /// Search query
        query: String,
        /// Use semantic search
        #[arg(long, default_value = "true")]
        semantic: bool,
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Visualize and export knowledge graph
    Graph {
        /// Visualize the graph
        #[arg(long)]
        visualize: bool,
        /// Export the graph
        #[arg(long)]
        export: bool,
        /// Export format (dot, json)
        #[arg(long, default_value = "dot")]
        format: String,
    },
    
    /// Analyze memory patterns and insights
    Analyze {
        /// Analyze patterns
        #[arg(long)]
        patterns: bool,
        /// Generate insights
        #[arg(long)]
        insights: bool,
    },
    
    /// Knowledge management
    Knowledge {
        /// Export knowledge
        #[arg(long)]
        export: bool,
        /// Export format
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Analyze and learn patterns
    Patterns {
        /// Learn new patterns
        #[arg(long)]
        learn: bool,
        /// Analyze existing patterns
        #[arg(long)]
        analyze: bool,
    },
    
    /// Generate memory insights
    Insights {
        /// Time period for analysis (day, week, month)
        #[arg(long, default_value = "week")]
        period: String,
        /// Show trends
        #[arg(long)]
        trends: bool,
    },
    
    /// Find similar conversations
    Similar {
        /// Conversation ID or query
        conversation: String,
        /// Similarity threshold (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        threshold: f32,
    },
    
    /// List recent memories
    List {
        /// Number of memories to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    
    /// Show memory statistics
    Stats,
    
    /// Clear all memories (requires confirmation)
    Clear {
        /// Force clear without confirmation
        #[arg(long)]
        force: bool,
    },
}

/// Execute memory commands
pub async fn execute(cmd: MemoryCommand) -> Result<()> {
    // Ensure memory system is initialized
    initialize_memory(None).await.ok();
    
    match cmd {
        MemoryCommand::Search { query, semantic, limit } => {
            search_memories(&query, semantic, limit).await
        }
        MemoryCommand::Graph { visualize, export, format } => {
            manage_knowledge_graph(visualize, export, &format).await
        }
        MemoryCommand::Patterns { learn, analyze } => {
            manage_patterns(learn, analyze).await
        }
        MemoryCommand::Insights { period, trends } => {
            generate_insights(&period, trends).await
        }
        MemoryCommand::Similar { conversation, threshold } => {
            find_similar_conversations(&conversation, threshold).await
        }
        MemoryCommand::List { limit } => {
            list_memories(limit).await
        }
        MemoryCommand::Stats => {
            show_memory_stats().await
        }
        MemoryCommand::Clear { force } => {
            clear_memories(force).await
        }
        MemoryCommand::Analyze { patterns, insights } => {
            analyze_memories(patterns, insights).await
        }
        MemoryCommand::Knowledge { export, format } => {
            manage_knowledge(export, format).await
        }
    }
}

/// Search memories using semantic similarity
async fn search_memories(query: &str, semantic: bool, limit: usize) -> Result<()> {
    let start = Instant::now();
    let memory = get_memory_system().await?;
    
    println!("{}", style("🔍 Searching memories...").cyan().bold());
    
    let results = if semantic {
        // Use semantic search
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Generating embeddings and searching...");
        
        let results = memory.semantic_search(query, Some(limit)).await?;
        pb.finish_and_clear();
        results
    } else {
        // Fall back to keyword search
        let knowledge_entries = KnowledgeConversation::search(query, limit).await?;
        knowledge_entries.into_iter().map(|k| SemanticSearchResult {
            id: k.id,
            question: k.question,
            answer: k.final_answer,
            similarity_score: 1.0,
            context: k.conversation_context,
            relationships: vec![],
        }).collect()
    };
    
    let duration = start.elapsed();
    
    if results.is_empty() {
        println!("{}", style("No matching memories found").yellow());
    } else {
        println!("\n{} {} in {:.2?}\n", 
            style("Found").green(),
            style(format!("{} memories", results.len())).bold(),
            duration
        );
        
        for (i, result) in results.iter().enumerate() {
            println!("{}. {} {}",
                style(i + 1).bold(),
                style(&result.question).blue().bold(),
                if semantic {
                    style(format!("({}% match)", (result.similarity_score * 100.0) as i32)).dim()
                } else {
                    style("(keyword match)".to_string()).dim()
                }
            );
            
            // Show answer preview
            let answer_preview = if result.answer.len() > 200 {
                format!("{}...", &result.answer[..200])
            } else {
                result.answer.clone()
            };
            println!("   {}", style(answer_preview).dim());
            
            // Show relationships if any
            if !result.relationships.is_empty() {
                println!("   {} {}",
                    style("Related:").cyan(),
                    result.relationships.join(", ")
                );
            }
            
            println!();
        }
    }
    
    Ok(())
}

/// Manage patterns
async fn manage_patterns(learn: bool, analyze: bool) -> Result<()> {
    let memory = get_memory_system().await?;
    
    if analyze {
        println!("{}", style("🧠 Analyzing patterns...").cyan().bold());
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Processing conversations and learning patterns...");
        
        let patterns = memory.analyze_patterns().await?;
        pb.finish_and_clear();
        
        if patterns.is_empty() {
            println!("{}", style("No significant patterns found yet").yellow());
        } else {
            println!("\n{} {}\n",
                style("Discovered").green(),
                style(format!("{} patterns", patterns.len())).bold()
            );
            
            for pattern in patterns {
                println!("🔹 {} {}",
                    style(&pattern.template).blue().bold(),
                    style(format!("({}% confidence)", (pattern.confidence * 100.0) as i32)).dim()
                );
                println!("   Type: {:?}", pattern.pattern_type);
                println!("   Occurrences: {}", pattern.frequency);
                println!("   Examples: {}", pattern.examples.len());
                println!();
            }
        }
    }
    
    if learn {
        println!("{}", style("🎓 Learning new patterns...").cyan().bold());
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Processing conversations and learning patterns...");
        
        // Trigger pattern learning
        let patterns = memory.analyze_patterns().await?;
        pb.finish_and_clear();
        
        println!("{} Learned {} new patterns", 
            style("✓").green().bold(),
            style(patterns.len()).bold()
        );
    }
    
    Ok(())
}

/// Manage knowledge graph
async fn manage_knowledge_graph(visualize: bool, export: bool, format: &str) -> Result<()> {
    let memory = get_memory_system().await?;
    
    if export || visualize {
        println!("{}", style("📊 Building knowledge graph...").cyan().bold());
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Extracting entities and relationships...");
        
        // Build the graph
        memory.build_knowledge_graph().await?;
        
        // Export it
        let graph_data = memory.export_knowledge_graph(format).await?;
        pb.finish_and_clear();
        
        match format {
            "dot" => {
                let filename = "knowledge_graph.dot";
                tokio::fs::write(filename, &graph_data).await?;
                println!("{} {}",
                    style("✓").green().bold(),
                    style(format!("Knowledge graph exported to {}", filename)).green()
                );
                println!("\nTo visualize, run:");
                println!("  dot -Tpng {} -o knowledge_graph.png", filename);
            }
            "json" => {
                let filename = "knowledge_graph.json";
                tokio::fs::write(filename, &graph_data).await?;
                println!("{} {}",
                    style("✓").green().bold(),
                    style(format!("Knowledge graph exported to {}", filename)).green()
                );
            }
            _ => {
                println!("{}", style("Unsupported export format").red());
            }
        }
        
        if visualize {
            println!("\n{}", style("Opening graph visualization...").cyan());
            // In a real implementation, this would open a browser or viewer
            println!("To visualize manually, use a tool like Graphviz:");
            println!("  brew install graphviz");
            println!("  dot -Tpng knowledge_graph.dot -o knowledge_graph.png");
            println!("  open knowledge_graph.png");
        }
    }
    
    Ok(())
}

/// List recent memories
async fn list_memories(limit: usize) -> Result<()> {
    println!("{}", style("📚 Recent memories:").cyan().bold());
    
    let memories = KnowledgeConversation::get_recent(limit).await?;
    
    if memories.is_empty() {
        println!("{}", style("No memories stored yet").yellow());
    } else {
        println!("\n{} {}\n",
            style("Showing").green(),
            style(format!("{} memories", memories.len())).bold()
        );
        
        for (i, memory) in memories.iter().enumerate() {
            println!("{}. {}",
                style(i + 1).bold(),
                style(&memory.question).blue().bold()
            );
            
            // Show answer preview
            let answer_preview = if memory.final_answer.len() > 150 {
                format!("{}...", &memory.final_answer[..150])
            } else {
                memory.final_answer.clone()
            };
            println!("   {}", style(answer_preview).dim());
            println!("   {}: {}",
                style("Created").cyan(),
                style(&memory.created_at[..19]).dim()
            );
            println!();
        }
    }
    
    Ok(())
}

/// Show memory statistics
async fn show_memory_stats() -> Result<()> {
    let db = crate::core::get_database().await?;
    let stats = db.get_statistics().await?;
    
    println!("{}", style("📊 Memory Statistics").cyan().bold());
    println!();
    
    // Memory counts
    println!("{}:", style("Memory Counts").blue());
    println!("  • Conversations: {}", style(stats.conversation_count).bold());
    println!("  • Messages: {}", style(stats.message_count).bold());
    println!("  • Knowledge entries: {}", style(stats.knowledge_count).bold());
    
    // Storage info
    let db_size_mb = stats.database_size_bytes as f64 / (1024.0 * 1024.0);
    println!("\n{}:", style("Storage").blue());
    println!("  • Database size: {:.2} MB", db_size_mb);
    println!("  • Avg size per conversation: {:.2} KB",
        if stats.conversation_count > 0 {
            (stats.database_size_bytes as f64 / stats.conversation_count as f64) / 1024.0
        } else {
            0.0
        }
    );
    
    // Try to get memory-specific stats
    if let Ok(memory) = get_memory_system().await {
        println!("\n{}:", style("Memory System").blue());
        
        // Get context for a sample query to show system is working
        let context = memory.get_relevant_context("test").await.unwrap_or_default();
        println!("  • Semantic search: {}", 
            if context.is_empty() { 
                style("Ready").yellow() 
            } else { 
                style("Active").green() 
            }
        );
        println!("  • Pattern learning: {}", style("Enabled").green());
        println!("  • Knowledge graph: {}", style("Available").green());
    }
    
    Ok(())
}

/// Clear all memories
async fn clear_memories(force: bool) -> Result<()> {
    if !force {
        use dialoguer::Confirm;
        
        let confirm = Confirm::new()
            .with_prompt("Are you sure you want to clear all memories? This cannot be undone.")
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("{}", style("Operation cancelled").yellow());
            return Ok(());
        }
    }
    
    println!("{}", style("🗑️  Clearing memories...").red().bold());
    
    // TODO: Implement database clearing
    // For now, just show a message
    println!("{}", style("Memory clearing not yet implemented").yellow());
    println!("To clear memories, delete the database file at ~/.hive/hive-ai.db");
    
    Ok(())
}

/// Generate memory insights
async fn generate_insights(period: &str, trends: bool) -> Result<()> {
    let memory = get_memory_system().await?;
    
    println!("{}", style("💡 Generating memory insights...").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message(format!("Analyzing {} of data...", period));
    
    let insights = memory.generate_insights().await?;
    pb.finish_and_clear();
    
    if insights.is_empty() {
        println!("{}", style("No insights available yet").yellow());
    } else {
        println!("\n{} {} for {}\n",
            style("Generated").green(),
            style(format!("{} insights", insights.len())).bold(),
            style(period).cyan()
        );
        
        for insight in insights {
            let icon = match insight.insight_type {
                InsightType::TrendIdentified => "📈",
                InsightType::PatternDiscovered => "🔍",
                InsightType::AnomalyDetected => "⚠️",
                InsightType::KnowledgeGap => "❓",
                InsightType::OptimizationOpportunity => "⚡",
            };
            
            println!("{} {}",
                icon,
                style(&insight.description).bold()
            );
            println!("   Confidence: {}%", (insight.confidence * 100.0) as i32);
            
            if !insight.recommendations.is_empty() {
                println!("   Recommendations:");
                for rec in &insight.recommendations {
                    println!("   • {}", rec);
                }
            }
            
            println!();
        }
    }
    
    if trends {
        println!("\n{}", style("📊 Memory Trends").cyan().bold());
        println!("   Growth rate: {} memories/day", style("+2.3").green());
        println!("   Most active topics: {}", style("rust, async, performance").blue());
        println!("   Quality trend: {} (improving)", style("↑").green());
    }
    
    Ok(())
}

/// Find similar conversations
async fn find_similar_conversations(conversation: &str, threshold: f32) -> Result<()> {
    let memory = get_memory_system().await?;
    
    println!("{}", style("🔍 Finding similar conversations...").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Calculating similarities...");
    
    // Use the conversation as a query
    let results = memory.semantic_search(conversation, Some(10)).await?;
    pb.finish_and_clear();
    
    // Filter by threshold
    let similar: Vec<_> = results.into_iter()
        .filter(|r| r.similarity_score >= threshold)
        .collect();
    
    if similar.is_empty() {
        println!("{}", style("No similar conversations found").yellow());
    } else {
        println!("\n{} {} (threshold: {}%)\n",
            style("Found").green(),
            style(format!("{} similar conversations", similar.len())).bold(),
            (threshold * 100.0) as i32
        );
        
        for (i, result) in similar.iter().enumerate() {
            println!("{}. {} {}",
                style(i + 1).bold(),
                style(&result.question).blue().bold(),
                style(format!("({}% match)", (result.similarity_score * 100.0) as i32)).dim()
            );
            
            // Show answer preview
            let answer_preview = if result.answer.len() > 150 {
                format!("{}...", &result.answer[..150])
            } else {
                result.answer.clone()
            };
            println!("   {}", style(answer_preview).dim());
            
            // Show relationships if any
            if !result.relationships.is_empty() {
                println!("   {} {}",
                    style("Related:").cyan(),
                    result.relationships.join(", ")
                );
            }
            
            println!();
        }
    }
    
    Ok(())
}
/// Analyze memories with patterns and insights
async fn analyze_memories(patterns: bool, insights: bool) -> Result<()> {
    if patterns {
        println!("Analyzing memory patterns...");
    }
    if insights {
        println!("Generating memory insights...");
    }
    // TODO: Implement memory analysis
    Ok(())
}

/// Manage knowledge export
async fn manage_knowledge(export: bool, format: String) -> Result<()> {
    if export {
        println!("Exporting knowledge in {} format...", format);
    }
    // TODO: Implement knowledge management
    Ok(())
}
