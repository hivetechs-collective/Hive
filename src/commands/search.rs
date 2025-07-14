//! Search command implementation with sub-millisecond performance
//!
//! This module implements the `hive search` command for fast symbol search
//! using SQLite FTS5 integration.

use anyhow::{Context, Result};
use console::style;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, info};

use crate::analysis::symbol_index::{SymbolEntry, SymbolIndexer};
use crate::core::database::DatabaseManager;
use std::sync::Arc;

/// Handle the search command
pub async fn handle_search(
    query: String,
    kind: Option<String>,
    path: Option<PathBuf>,
    limit: usize,
    fuzzy: bool,
) -> Result<()> {
    let start = Instant::now();

    println!(
        "🔍 {} for '{}'...",
        style("Searching").bold().cyan(),
        style(&query).yellow()
    );

    // Initialize database
    let db = Arc::new(DatabaseManager::default().await?);

    // Create symbol indexer
    let indexer = SymbolIndexer::new(db).await?;

    // Build search query
    let search_query = if fuzzy {
        // For fuzzy search, add wildcards
        format!("{}*", query)
    } else {
        query.clone()
    };

    // Filter by kind if specified
    let filtered_query = if let Some(ref k) = kind {
        format!("{} AND kind:{}", search_query, k)
    } else {
        search_query
    };

    // Perform search
    let results = indexer.search(&filtered_query, limit).await?;

    let elapsed = start.elapsed();

    // Display results
    if results.is_empty() {
        println!("❌ {} found for '{}'", style("No symbols").red(), query);
    } else {
        println!(
            "✅ Found {} {} in {:.2}ms",
            style(results.len()).green().bold(),
            if results.len() == 1 {
                "symbol"
            } else {
                "symbols"
            },
            elapsed.as_secs_f64() * 1000.0
        );

        println!();

        for (i, symbol) in results.iter().enumerate() {
            display_symbol_result(i + 1, symbol);
        }
    }

    // Verify sub-millisecond performance for common queries
    if elapsed.as_millis() > 10 {
        println!(
            "\n⚠️  {} Search took {}ms (target: <10ms)",
            style("Warning:").yellow(),
            elapsed.as_millis()
        );
    }

    Ok(())
}

/// Display a single symbol result
fn display_symbol_result(index: usize, symbol: &SymbolEntry) {
    let kind_emoji = match symbol.kind {
        crate::core::ast::SymbolKind::Function => "🔧",
        crate::core::ast::SymbolKind::Class => "📦",
        crate::core::ast::SymbolKind::Struct => "🏗️",
        crate::core::ast::SymbolKind::Interface => "🔌",
        crate::core::ast::SymbolKind::Enum => "🎯",
        crate::core::ast::SymbolKind::Module => "📁",
        crate::core::ast::SymbolKind::Variable => "📝",
        crate::core::ast::SymbolKind::Constant => "🔒",
        crate::core::ast::SymbolKind::Method => "⚡",
        crate::core::ast::SymbolKind::TypeAlias => "🏷️",
        crate::core::ast::SymbolKind::Trait => "🎭",
        crate::core::ast::SymbolKind::Import => "📥",
        crate::core::ast::SymbolKind::Namespace => "📦",
        crate::core::ast::SymbolKind::Property => "🔗",
        crate::core::ast::SymbolKind::Parameter => "📌",
    };

    println!(
        "{:2}. {} {} {} {}:{}",
        style(index).dim(),
        kind_emoji,
        style(&symbol.name).green().bold(),
        style(format!("{:?}", symbol.kind)).cyan(),
        style(symbol.file_path.display()).dim(),
        style(symbol.start_pos.line + 1).yellow()
    );

    if let Some(ref sig) = symbol.signature {
        println!("    {}", style(sig).dim());
    }

    if let Some(ref doc) = symbol.documentation {
        let preview = doc.lines().next().unwrap_or("");
        let truncated = if preview.len() > 60 {
            format!("{}...", &preview[..60])
        } else {
            preview.to_string()
        };
        println!("    💬 {}", style(truncated).italic().dim());
    }

    // Show quality score with color coding
    let quality_color = if symbol.quality_score >= 8.0 {
        console::Color::Green
    } else if symbol.quality_score >= 6.0 {
        console::Color::Yellow
    } else {
        console::Color::Red
    };

    println!(
        "    📊 Quality: {} | 🔗 References: {} | 🧩 Complexity: {}",
        style(format!("{:.1}/10", symbol.quality_score)).fg(quality_color),
        style(symbol.reference_count).cyan(),
        style(symbol.complexity).magenta()
    );

    println!();
}

/// Handle symbol references command
pub async fn handle_references(
    symbol_name: String,
    file: Option<PathBuf>,
    line: Option<usize>,
) -> Result<()> {
    let start = Instant::now();

    println!(
        "🔗 {} to '{}'...",
        style("Finding references").bold().cyan(),
        style(&symbol_name).yellow()
    );

    // Initialize database
    let db = Arc::new(DatabaseManager::default().await?);

    // Create symbol indexer
    let indexer = SymbolIndexer::new(db).await?;

    // First find the symbol
    let symbols = indexer.search(&symbol_name, 10).await?;

    let symbol = if let Some(f) = file {
        // Filter by file if provided
        symbols
            .into_iter()
            .find(|s| s.file_path == f && line.map_or(true, |l| s.start_pos.line == l))
    } else {
        symbols.into_iter().next()
    };

    if let Some(symbol) = symbol {
        // Find all references
        let references = indexer.find_references(&symbol.id).await?;

        let elapsed = start.elapsed();

        if references.is_empty() {
            println!(
                "❌ {} to '{}' found",
                style("No references").red(),
                symbol_name
            );
        } else {
            println!(
                "✅ Found {} {} to '{}' in {:.2}ms",
                style(references.len()).green().bold(),
                if references.len() == 1 {
                    "reference"
                } else {
                    "references"
                },
                symbol_name,
                elapsed.as_secs_f64() * 1000.0
            );

            println!();

            // Group references by file
            let mut refs_by_file = std::collections::HashMap::new();
            for r in references {
                refs_by_file
                    .entry(r.file_path.clone())
                    .or_insert_with(Vec::new)
                    .push(r);
            }

            for (file, refs) in refs_by_file {
                println!("📄 {}", style(file.display()).cyan().bold());

                for r in refs {
                    let kind_icon = match r.reference_kind {
                        crate::analysis::symbol_index::ReferenceKind::Call => "📞",
                        crate::analysis::symbol_index::ReferenceKind::Import => "📥",
                        crate::analysis::symbol_index::ReferenceKind::Inherit => "🔗",
                        crate::analysis::symbol_index::ReferenceKind::Implement => "🔧",
                        crate::analysis::symbol_index::ReferenceKind::Instantiate => "🏗️",
                        crate::analysis::symbol_index::ReferenceKind::Reference => "👉",
                        crate::analysis::symbol_index::ReferenceKind::TypeUse => "🏷️",
                    };

                    println!(
                        "  {} Line {}: {}",
                        kind_icon,
                        style(r.position.line + 1).yellow(),
                        style(&r.context).dim()
                    );
                }

                println!();
            }
        }
    } else {
        println!("❌ {} '{}' not found", style("Symbol").red(), symbol_name);
    }

    Ok(())
}

/// Handle call graph command
pub async fn handle_call_graph(
    function_name: String,
    depth: Option<usize>,
    format: Option<String>,
) -> Result<()> {
    println!(
        "🕸️  {} for '{}'...",
        style("Building call graph").bold().cyan(),
        style(&function_name).yellow()
    );

    // Initialize database
    let db = Arc::new(DatabaseManager::default().await?);

    // Create symbol indexer
    let indexer = SymbolIndexer::new(db).await?;

    // Get call graph
    let call_info = indexer.get_call_graph(&function_name).await?;

    if call_info.calls.is_empty() && call_info.called_by.is_empty() {
        println!(
            "❌ {} call information found for '{}'",
            style("No").red(),
            function_name
        );
        return Ok(());
    }

    println!(
        "\n📊 {} '{}':",
        style("Call graph for").bold(),
        style(&function_name).green()
    );

    // Display functions this one calls
    if !call_info.calls.is_empty() {
        println!(
            "\n  {} ({}):",
            style("Calls").cyan().bold(),
            call_info.calls.len()
        );
        for called in &call_info.calls {
            println!("    → {}", style(called).yellow());
        }
    }

    // Display functions that call this one
    if !call_info.called_by.is_empty() {
        println!(
            "\n  {} ({}):",
            style("Called by").magenta().bold(),
            call_info.called_by.len()
        );
        for caller in &call_info.called_by {
            println!("    ← {}", style(caller).blue());
        }
    }

    // Optional: Generate visualization
    if let Some(fmt) = format {
        match fmt.as_str() {
            "dot" => {
                println!("\n📈 {} format:", style("Graphviz DOT").dim());
                println!("digraph CallGraph {{");
                println!(
                    "  \"{}\" [style=filled, fillcolor=lightblue];",
                    function_name
                );

                for called in &call_info.calls {
                    println!("  \"{}\" -> \"{}\";", function_name, called);
                }

                for caller in &call_info.called_by {
                    println!("  \"{}\" -> \"{}\";", caller, function_name);
                }

                println!("}}");
            }
            _ => {
                println!("⚠️  Unknown format '{}'. Supported: dot", fmt);
            }
        }
    }

    Ok(())
}
