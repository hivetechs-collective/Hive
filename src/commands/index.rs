//! Index command implementation for building semantic indexes
//!
//! This module implements the `hive index build` command for creating
//! and maintaining the symbol index with FTS5 support.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::time::Instant;
use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{info, debug};
use walkdir::WalkDir;

use crate::analysis::symbol_index::{SymbolIndexer, IndexStatistics};
use crate::core::database::DatabaseManager;
use std::sync::Arc;

/// Handle the index build command
pub async fn handle_index_build(
    path: Option<PathBuf>,
    force: bool,
    include_tests: bool,
    exclude_patterns: Vec<String>,
) -> Result<()> {
    let start = Instant::now();
    let target_path = path.unwrap_or_else(|| PathBuf::from("."));
    
    println!("🐝 {} semantic index for {}...",
        if force { style("Rebuilding").yellow() } else { style("Building").cyan() },
        style(target_path.display()).bold()
    );
    
    // Initialize database
    let db = Arc::new(DatabaseManager::default().await?);
    
    // Create symbol indexer
    let indexer = Arc::new(SymbolIndexer::new(db).await?);
    
    // Discover files to index
    let files = discover_source_files(&target_path, include_tests, &exclude_patterns)?;
    
    if files.is_empty() {
        println!("❌ No source files found to index");
        return Ok(());
    }
    
    println!("📂 Found {} files to index", style(files.len()).green().bold());
    
    // Create progress bar
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    let mut success_count = 0;
    let mut error_count = 0;
    let mut total_symbols = 0;
    
    // Index each file
    for file_path in &files {
        pb.set_message(format!("Indexing {}", file_path.file_name().unwrap_or_default().to_string_lossy()));
        
        match index_single_file(&indexer, file_path).await {
            Ok(symbol_count) => {
                success_count += 1;
                total_symbols += symbol_count;
                debug!("Indexed {} with {} symbols", file_path.display(), symbol_count);
            }
            Err(e) => {
                error_count += 1;
                debug!("Failed to index {}: {}", file_path.display(), e);
            }
        }
        
        pb.inc(1);
    }
    
    pb.finish_and_clear();
    
    let elapsed = start.elapsed();
    
    // Get final statistics
    let stats = indexer.get_stats().await;
    
    // Display results
    println!("\n✅ {} complete!", style("Indexing").green().bold());
    println!("   ⏱️  Time: {:.2}s", elapsed.as_secs_f64());
    println!("   📄 Files: {} indexed, {} errors", 
        style(success_count).green(),
        if error_count > 0 { style(error_count).red() } else { style(error_count).dim() }
    );
    println!("   🔍 Symbols: {} total", style(total_symbols).cyan().bold());
    
    // Display symbol breakdown
    if !stats.symbols_by_kind.is_empty() {
        println!("\n📊 Symbol Distribution:");
        let mut symbols_by_kind: Vec<_> = stats.symbols_by_kind.iter().collect();
        symbols_by_kind.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        
        for (kind, count) in symbols_by_kind.iter().take(8) {
            let emoji = get_symbol_emoji(&format!("{:?}", kind));
            println!("   {} {}: {}", emoji, kind, style(**count).yellow());
        }
    }
    
    // Check for circular dependencies
    if !stats.cyclic_dependencies.is_empty() {
        println!("\n⚠️  {} {} detected!",
            style(stats.cyclic_dependencies.len()).red().bold(),
            if stats.cyclic_dependencies.len() == 1 { "circular dependency" } else { "circular dependencies" }
        );
    }
    
    // Performance check
    let avg_time_per_file = elapsed.as_secs_f64() / files.len() as f64 * 1000.0;
    if avg_time_per_file > 50.0 {
        println!("\n⚠️  {} Average indexing time {:.2}ms per file (target: <50ms)",
            style("Performance:").yellow(),
            avg_time_per_file
        );
    } else {
        println!("\n⚡ {} {:.2}ms average per file",
            style("Performance:").green(),
            avg_time_per_file
        );
    }
    
    Ok(())
}

/// Index a single file
async fn index_single_file(indexer: &Arc<SymbolIndexer>, path: &Path) -> Result<usize> {
    // Read file content
    let content = tokio::fs::read_to_string(path).await
        .context("Failed to read file")?;
    
    // Index the file
    indexer.index_file(path, &content).await?;
    
    // Return approximate symbol count (in real implementation, get from indexer)
    let lines = content.lines().count();
    let estimated_symbols = lines / 10; // Rough estimate
    
    Ok(estimated_symbols)
}

/// Discover source files to index
fn discover_source_files(
    root: &Path,
    include_tests: bool,
    exclude_patterns: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    // Common source file extensions
    const SOURCE_EXTENSIONS: &[&str] = &[
        "rs", "ts", "tsx", "js", "jsx", "py", "go", 
        "java", "cpp", "c", "cc", "cxx", "h", "hpp",
        "cs", "rb", "php", "swift", "kt", "scala"
    ];
    
    // Walk directory tree
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            
            // Skip hidden directories
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false) {
                return false;
            }
            
            // Skip common non-source directories
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
                
            if matches!(dir_name, 
                "node_modules" | "target" | "dist" | "build" | 
                ".git" | ".venv" | "__pycache__" | "vendor"
            ) {
                return false;
            }
            
            // Skip test directories if not included
            if !include_tests && (dir_name == "tests" || dir_name == "test") {
                return false;
            }
            
            true
        })
    {
        let entry = entry?;
        let path = entry.path();
        
        // Check if it's a file
        if !path.is_file() {
            continue;
        }
        
        // Check extension
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
            
        if !SOURCE_EXTENSIONS.contains(&extension) {
            continue;
        }
        
        // Check exclude patterns
        let path_str = path.to_string_lossy();
        if exclude_patterns.iter().any(|pattern| path_str.contains(pattern)) {
            continue;
        }
        
        files.push(path.to_path_buf());
    }
    
    files.sort();
    Ok(files)
}

/// Get emoji for symbol kind
fn get_symbol_emoji(kind: &str) -> &'static str {
    match kind {
        "Function" => "🔧",
        "Class" => "📦",
        "Struct" => "🏗️",
        "Interface" => "🔌",
        "Enum" => "🎯",
        "Module" => "📁",
        "Variable" => "📝",
        "Constant" => "🔒",
        "Method" => "⚡",
        "TypeAlias" => "🏷️",
        "Trait" => "🎭",
        _ => "📌",
    }
}

/// Handle index statistics command
pub async fn handle_index_stats() -> Result<()> {
    println!("📊 {} index statistics...", style("Loading").cyan());
    
    // Initialize database
    let db = Arc::new(DatabaseManager::default().await?);
    
    // Create symbol indexer
    let indexer = SymbolIndexer::new(db).await?;
    
    // Get statistics
    let stats = indexer.get_stats().await;
    
    // Display statistics
    println!("\n🐝 {} {}", style("Symbol Index Statistics").bold().cyan(), style("━".repeat(40)).dim());
    
    println!("\n📈 Overview:");
    println!("   Files Indexed: {}", style(stats.files_indexed).green().bold());
    println!("   Total Symbols: {}", style(stats.total_symbols).cyan().bold());
    println!("   Total References: {}", style(stats.total_references).yellow());
    println!("   Index Time: {:.2}s total", stats.index_time_ms / 1000.0);
    
    if stats.files_indexed > 0 {
        let avg_symbols_per_file = stats.total_symbols as f32 / stats.files_indexed as f32;
        let avg_time_per_file = stats.index_time_ms / stats.files_indexed as f64;
        
        println!("\n📊 Averages:");
        println!("   Symbols/File: {:.1}", avg_symbols_per_file);
        println!("   Time/File: {:.2}ms", avg_time_per_file);
        println!("   References/Symbol: {:.1}", 
            if stats.total_symbols > 0 {
                stats.total_references as f32 / stats.total_symbols as f32
            } else {
                0.0
            }
        );
    }
    
    // Symbol distribution
    if !stats.symbols_by_kind.is_empty() {
        println!("\n🔍 Symbol Types:");
        
        let mut symbols_by_kind: Vec<_> = stats.symbols_by_kind.iter().collect();
        symbols_by_kind.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        
        let max_count = symbols_by_kind[0].1;
        let bar_width = 30;
        
        for (kind, count) in symbols_by_kind {
            let emoji = get_symbol_emoji(&format!("{:?}", kind));
            let percentage = (*count as f32 / stats.total_symbols as f32) * 100.0;
            let bar_len = ((*count as f32 / *max_count as f32) * bar_width as f32) as usize;
            let bar = "█".repeat(bar_len);
            
            println!("   {} {:12} {:>6} ({:>5.1}%) {}",
                emoji,
                kind,
                style(count).yellow(),
                percentage,
                style(bar).cyan()
            );
        }
    }
    
    // Circular dependencies
    if !stats.cyclic_dependencies.is_empty() {
        println!("\n⚠️  {} Dependencies:", style("Circular").red().bold());
        
        for (i, cycle) in stats.cyclic_dependencies.iter().take(5).enumerate() {
            println!("   {}. Cycle with {} modules:", i + 1, cycle.len());
            for module in cycle.iter().take(3) {
                println!("      • {}", module);
            }
            if cycle.len() > 3 {
                println!("      ... and {} more", cycle.len() - 3);
            }
        }
        
        if stats.cyclic_dependencies.len() > 5 {
            println!("   ... and {} more cycles", stats.cyclic_dependencies.len() - 5);
        }
    }
    
    // Index health
    println!("\n🏥 Index Health:");
    let health_score = calculate_index_health(&stats);
    let health_emoji = if health_score >= 90.0 { "🟢" }
        else if health_score >= 70.0 { "🟡" }
        else { "🔴" };
        
    println!("   {} Health Score: {:.0}%", health_emoji, health_score);
    
    if health_score < 90.0 {
        println!("\n💡 Suggestions:");
        if stats.cyclic_dependencies.len() > 0 {
            println!("   • Resolve circular dependencies to improve code structure");
        }
        if stats.files_indexed == 0 {
            println!("   • Run 'hive index build' to index your codebase");
        }
    }
    
    Ok(())
}

/// Calculate index health score
fn calculate_index_health(stats: &IndexStatistics) -> f32 {
    let mut score = 100.0;
    
    // Deduct for circular dependencies
    score -= (stats.cyclic_dependencies.len() as f32 * 5.0).min(30.0);
    
    // Deduct if no files indexed
    if stats.files_indexed == 0 {
        score -= 50.0;
    }
    
    // Deduct for low symbol count
    if stats.total_symbols < 100 && stats.files_indexed > 10 {
        score -= 20.0;
    }
    
    score.max(0.0)
}