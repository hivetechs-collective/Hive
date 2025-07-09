//! Consensus pipeline testing and verification commands
//! 
//! Provides CLI commands for testing the 4-stage consensus pipeline
//! including temporal context, streaming progress, and quality validation.

use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::consensus::{ConsensusEngine, ConsensusRequest, TemporalContextProvider};
use crate::consensus::streaming::{ConsensusMetrics, EnhancedProgressTracker, StreamingCallbacks};
use crate::consensus::types::{ConsensusConfig, ConsensusProfile, Stage};
use crate::core::context::ContextBuilder;
use crate::core::semantic::SemanticIndex;

/// Handle consensus testing command
pub async fn handle_consensus_test(
    input: &str,
    profile: Option<&str>,
    show_progress: bool,
    show_context: bool,
    verify_stages: bool,
) -> Result<()> {
    println!("🤖 Testing 4-Stage Consensus Pipeline");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let start_time = Instant::now();
    
    // Create consensus engine with database
    use crate::core::database::{DatabaseManager, DatabaseConfig};
    let db = Arc::new(DatabaseManager::new(DatabaseConfig::default()).await?);
    let engine = ConsensusEngine::new(Some(db)).await
        .context("Failed to create consensus engine")?;
    
    // Set profile if specified
    if let Some(profile_name) = profile {
        engine.set_profile(profile_name).await
            .with_context(|| format!("Failed to set profile: {}", profile_name))?;
    }
    
    let current_profile = engine.get_current_profile().await;
    println!("📋 Profile: {}", current_profile.profile_name);
    println!("🎯 Input: {}", input);
    println!();
    
    // Build context if requested
    let context = if show_context {
        println!("🧠 Building semantic context...");
        let cwd = std::env::current_dir()?;
        build_test_context(&cwd, input).await.ok()
    } else {
        None
    };
    
    if let Some(ctx) = &context {
        println!("✅ Context built: {} snippets, {} symbols", 
                ctx.code_snippets.len(), ctx.symbols.len());
        if show_context {
            display_context_summary(ctx);
        }
        println!();
    }
    
    // Test consensus pipeline
    let result = if show_progress {
        test_with_streaming_progress(&engine, input, context.as_ref()).await?
    } else {
        test_basic_consensus(&engine, input, context.as_ref()).await?
    };
    
    // Display results
    display_consensus_results(&result, verify_stages, start_time.elapsed())?;
    
    Ok(())
}

/// Handle temporal context testing
pub async fn handle_temporal_test(query: &str) -> Result<()> {
    println!("🕒 Testing Temporal Context Detection");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let provider = TemporalContextProvider::default();
    
    // Test detection
    let requires_temporal = provider.requires_temporal_context(query);
    println!("📝 Query: {}", query);
    println!("🎯 Requires temporal context: {}", 
             if requires_temporal { "✅ Yes" } else { "❌ No" });
    
    if requires_temporal {
        println!("\n⏰ Building temporal context...");
        let context = provider.build_current_context().await?;
        
        println!("📅 Current date: {}", context.current_datetime);
        println!("🔍 Search instruction:");
        println!("   {}", context.search_instruction);
        println!("⚡ Temporal awareness:");
        println!("   {}", context.temporal_awareness);
        
        if let Some(business_ctx) = &context.business_context {
            println!("💼 Business context:");
            println!("   Business day: {}", business_ctx.is_business_day);
            println!("   Market hours: {}", business_ctx.is_market_hours);
            println!("   Quarter: {}", business_ctx.quarter);
        }
    }
    
    Ok(())
}

/// Handle consensus metrics display
pub async fn handle_consensus_metrics() -> Result<()> {
    println!("📊 Consensus Pipeline Metrics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Create sample metrics for display
    let mut metrics = ConsensusMetrics::default();
    
    // Simulate completed stages
    metrics.update_stage_progress(Stage::Generator, 100.0, 0.92);
    metrics.update_stage_progress(Stage::Refiner, 100.0, 0.88);
    metrics.update_stage_progress(Stage::Validator, 100.0, 0.95);
    metrics.update_stage_progress(Stage::Curator, 75.0, 0.85);
    metrics.set_current_stage(Some(Stage::Curator));
    
    // Display metrics
    println!("🚀 Overall Progress: {:.1}%", metrics.overall_progress);
    println!("📈 Stage Details:");
    
    for i in 0..4 {
        println!("   {}", metrics.format_stage_display(i));
    }
    
    println!("\n💰 Estimated cost: ${:.4}", metrics.estimated_cost);
    println!("🔢 Total tokens: {}", metrics.total_tokens);
    
    if let Some(current) = metrics.current_stage {
        println!("⚡ Current stage: {}", current.display_name());
    }
    
    Ok(())
}

/// Handle stage prompts display
pub async fn handle_stage_prompts(stage: Option<&str>) -> Result<()> {
    use crate::consensus::types::StagePrompts;
    use crate::consensus::stages::{GeneratorStage, RefinerStage, ValidatorStage, CuratorStage, ConsensusStage};
    
    println!("📝 Consensus Stage Prompts");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    match stage {
        Some("generator") => {
            println!("🎯 Generator Stage System Prompt:");
            println!("{}", StagePrompts::generator_system());
            
            let generator = GeneratorStage::new();
            let sample_question = "How do I implement error handling in Rust?";
            println!("\n📋 Enhanced Prompt Example:");
            println!("{}", generator.enhance_system_prompt_with_context(sample_question));
        },
        Some("refiner") => {
            println!("🔧 Refiner Stage System Prompt:");
            println!("{}", StagePrompts::refiner_system());
        },
        Some("validator") => {
            println!("🔍 Validator Stage System Prompt:");
            println!("{}", StagePrompts::validator_system());
        },
        Some("curator") => {
            println!("✨ Curator Stage System Prompt:");
            println!("{}", StagePrompts::curator_system());
        },
        _ => {
            println!("🎯 All Stage Prompts:\n");
            
            println!("1. GENERATOR:");
            println!("{}\n", StagePrompts::generator_system());
            
            println!("2. REFINER:");
            println!("{}\n", StagePrompts::refiner_system());
            
            println!("3. VALIDATOR:");
            println!("{}\n", StagePrompts::validator_system());
            
            println!("4. CURATOR:");
            println!("{}\n", StagePrompts::curator_system());
        }
    }
    
    Ok(())
}

/// Handle consensus benchmark
pub async fn handle_consensus_benchmark(iterations: usize) -> Result<()> {
    println!("⚡ Consensus Pipeline Benchmark");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔢 Iterations: {}", iterations);
    println!();
    
    let engine = ConsensusEngine::new(None).await?;
    
    let test_queries = [
        "What is Rust?",
        "How do I handle errors in async Rust?",
        "Explain ownership and borrowing",
        "What are the latest Rust features in 2024?",
    ];
    
    let mut total_duration = std::time::Duration::from_secs(0);
    let mut results = Vec::new();
    
    for (i, query) in test_queries.iter().cycle().take(iterations).enumerate() {
        print!("Running test {}/{}: {} ... ", i + 1, iterations, query);
        
        let start = Instant::now();
        let result = engine.process(query, None).await?;
        let duration = start.elapsed();
        total_duration += duration;
        
        results.push((query, duration, result.success));
        println!("{:.2}s {}", duration.as_secs_f64(), if result.success { "✅" } else { "❌" });
    }
    
    // Display benchmark results
    println!("\n📊 Benchmark Results:");
    println!("⏱️  Total time: {:.2}s", total_duration.as_secs_f64());
    println!("📈 Average time: {:.2}s", total_duration.as_secs_f64() / iterations as f64);
    println!("✅ Success rate: {:.1}%", 
             results.iter().filter(|(_, _, success)| *success).count() as f32 / iterations as f32 * 100.0);
    
    // Performance analysis
    let avg_ms = total_duration.as_millis() / iterations as u128;
    if avg_ms < 500 {
        println!("🚀 Performance: Excellent (target: <500ms)");
    } else if avg_ms < 1000 {
        println!("⚡ Performance: Good (target: <500ms)");
    } else {
        println!("⚠️  Performance: Needs optimization (target: <500ms)");
    }
    
    Ok(())
}

/// Test consensus with streaming progress
async fn test_with_streaming_progress(
    engine: &ConsensusEngine,
    input: &str,
    context: Option<&crate::core::context::QueryContext>,
) -> Result<crate::consensus::types::ConsensusResult> {
    println!("🔄 Running consensus with streaming progress...");
    println!();
    
    // Convert context to string if available
    let context_str = context.map(|ctx| format_context_for_consensus(ctx));
    
    let result = engine.process(input, context_str).await
        .context("Consensus pipeline failed")?;
    
    println!("\n✅ Consensus completed successfully!");
    Ok(result)
}

/// Test basic consensus without streaming
async fn test_basic_consensus(
    engine: &ConsensusEngine,
    input: &str, 
    context: Option<&crate::core::context::QueryContext>,
) -> Result<crate::consensus::types::ConsensusResult> {
    println!("⚡ Running basic consensus...");
    
    let context_str = context.map(|ctx| format_context_for_consensus(ctx));
    
    let result = engine.process(input, context_str).await
        .context("Consensus pipeline failed")?;
    
    println!("✅ Consensus completed!");
    Ok(result)
}

/// Build test context for consensus
async fn build_test_context(
    path: &Path,
    query: &str,
) -> Result<crate::core::context::QueryContext> {
    // This would integrate with the actual semantic index from Phase 2
    // For now, return a mock context
    use crate::core::context::{QueryContext, CodeSnippet, ContextSymbol, FileSummary, ProjectInfo, Documentation};
    use crate::core::ast::SymbolKind;
    use crate::core::Language;
    use std::collections::HashMap;
    
    Ok(QueryContext {
        code_snippets: vec![
            CodeSnippet {
                file: path.join("src/main.rs"),
                start_line: 1,
                end_line: 10,
                content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
                language: Language::Rust,
                relevance: 0.8,
                reason: "Main entry point".to_string(),
            }
        ],
        symbols: vec![
            ContextSymbol {
                name: "main".to_string(),
                kind: SymbolKind::Function,
                file: path.join("src/main.rs"),
                signature: Some("fn main()".to_string()),
                documentation: Some("Entry point function".to_string()),
                related: vec![],
            }
        ],
        file_summaries: vec![
            FileSummary {
                path: path.join("src/main.rs"),
                description: "Main application entry point".to_string(),
                exports: vec!["main".to_string()],
                dependencies: vec![],
            }
        ],
        project_info: ProjectInfo {
            name: "test-project".to_string(),
            project_type: "Rust".to_string(),
            technologies: vec!["Rust".to_string()],
            structure: "Simple Rust project with main.rs".to_string(),
        },
        documentation: vec![],
        total_tokens: 50,
    })
}

/// Display context summary
fn display_context_summary(context: &crate::core::context::QueryContext) {
    println!("📋 Context Summary:");
    if !context.code_snippets.is_empty() {
        println!("  📄 Code snippets: {}", context.code_snippets.len());
        for snippet in context.code_snippets.iter().take(3) {
            println!("    • {} (lines {}-{})", 
                     snippet.file.display(), snippet.start_line, snippet.end_line);
        }
    }
    
    if !context.symbols.is_empty() {
        println!("  🔧 Symbols: {}", context.symbols.len());
        for symbol in context.symbols.iter().take(3) {
            println!("    • {} ({:?})", symbol.name, symbol.kind);
        }
    }
}

/// Format context for consensus
fn format_context_for_consensus(context: &crate::core::context::QueryContext) -> String {
    let mut formatted = String::new();
    
    if !context.code_snippets.is_empty() {
        formatted.push_str("RELEVANT CODE:\n");
        for snippet in context.code_snippets.iter().take(3) {
            formatted.push_str(&format!(
                "File: {} (lines {}-{})\n```{}\n{}\n```\n\n",
                snippet.file.display(),
                snippet.start_line,
                snippet.end_line,
                snippet.language.as_str(),
                snippet.content
            ));
        }
    }
    
    if !context.symbols.is_empty() {
        formatted.push_str("RELEVANT SYMBOLS:\n");
        for symbol in context.symbols.iter().take(5) {
            formatted.push_str(&format!("Symbol: {} ({:?})\n", symbol.name, symbol.kind));
        }
        formatted.push('\n');
    }
    
    formatted
}

/// Display consensus results
fn display_consensus_results(
    result: &crate::consensus::types::ConsensusResult,
    verify_stages: bool,
    total_duration: std::time::Duration,
) -> Result<()> {
    println!("\n🎯 Consensus Results");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("✅ Success: {}", result.success);
    println!("⏱️  Total duration: {:.2}s", total_duration.as_secs_f64());
    println!("💰 Total cost: ${:.4}", result.total_cost);
    println!("🔢 Stages completed: {}", result.stages.len());
    
    if verify_stages {
        println!("\n📋 Stage Verification:");
        for (i, stage_result) in result.stages.iter().enumerate() {
            let stage_name = match i {
                0 => "Generator",
                1 => "Refiner",
                2 => "Validator", 
                3 => "Curator",
                _ => "Unknown",
            };
            
            println!("  {}. {} ({})", i + 1, stage_name, stage_result.model);
            if let Some(analytics) = &stage_result.analytics {
                println!("     ⏱️  Duration: {:.2}s", analytics.duration);
                println!("     💰 Cost: ${:.4}", analytics.cost);
                println!("     ⭐ Quality: {:.1}%", analytics.quality_score * 100.0);
            }
            println!("     📝 Response length: {} chars", stage_result.answer.len());
        }
    }
    
    if let Some(final_result) = &result.result {
        println!("\n🎯 Final Response:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{}", final_result);
    }
    
    // Performance assessment
    let avg_stage_time = total_duration.as_millis() / result.stages.len() as u128;
    if total_duration.as_millis() < 500 {
        println!("\n🚀 Performance: Excellent (target: <500ms)");
    } else if total_duration.as_millis() < 1000 {
        println!("⚡ Performance: Good (target: <500ms)");
    } else {
        println!("⚠️  Performance: Needs optimization (target: <500ms)");
    }
    
    Ok(())
}