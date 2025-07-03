//! Analyze command implementation with repository intelligence
//!
//! This module implements the `hive analyze` command for comprehensive
//! repository analysis including architecture detection and quality scoring.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::time::Instant;
use console::style;
use tracing::{info, debug};

use crate::analysis::{
    symbol_index::SymbolIndexer,
    dependency::DependencyAnalyzer,
    repository_intelligence::{
        RepositoryAnalyzer, ArchitecturePattern, Severity, Priority, Impact, Effort,
        ArchitectureInfo, QualityReport, SecurityReport, PerformanceReport, TechnicalDebtReport,
        Recommendation
    },
};
use crate::core::database::DatabaseManager;
use std::sync::Arc;

/// Analyze a codebase and return analysis results
pub async fn analyze_codebase(path: &Path) -> Result<serde_json::Value> {
    let db = Arc::new(DatabaseManager::default().await?);
    let symbol_indexer = Arc::new(SymbolIndexer::new(db.clone()).await?);
    let dependency_analyzer = Arc::new(DependencyAnalyzer::new().await?);
    let repository_analyzer = RepositoryAnalyzer::new(
        symbol_indexer.clone(),
        dependency_analyzer.clone(),
    ).await?;
    
    let analysis = repository_analyzer.analyze_repository(path).await?;
    
    // Convert to JSON for MCP integration
    let result = serde_json::json!({
        "architecture": {
            "primary_pattern": format!("{:?}", analysis.architecture.primary_pattern),
            "confidence": analysis.architecture.confidence,
            "adherence_score": analysis.architecture.adherence_score,
            "components": analysis.architecture.components.len()
        },
        "quality": {
            "overall_score": analysis.quality.overall_score,
            "issues_count": analysis.quality.issues.len(),
            "test_coverage": analysis.quality.metrics.test_coverage,
            "maintainability_index": analysis.quality.metrics.maintainability_index
        },
        "security": {
            "vulnerability_count": analysis.security.vulnerability_count,
            "risk_score": analysis.security.risk_score
        },
        "technical_debt": {
            "total_hours": analysis.technical_debt.total_debt_hours,
            "estimated_cost": analysis.technical_debt.estimated_cost,
            "debt_ratio": analysis.technical_debt.debt_ratio
        }
    });
    
    Ok(result)
}

/// Handle the analyze command
pub async fn handle_analyze(
    target: PathBuf,
    depth: String,
    quality: bool,
    architecture: bool,
    dependencies: bool,
    output_format: Option<String>,
) -> Result<()> {
    let start = Instant::now();
    
    println!("🔍 {} {}...", 
        style("Analyzing").bold().cyan(),
        style(target.display()).yellow()
    );
    
    // Initialize components
    let db = Arc::new(DatabaseManager::default().await?);
    let symbol_indexer = Arc::new(SymbolIndexer::new(db.clone()).await?);
    let dependency_analyzer = Arc::new(DependencyAnalyzer::new().await?);
    let repository_analyzer = RepositoryAnalyzer::new(
        symbol_indexer.clone(),
        dependency_analyzer.clone(),
    ).await?;
    
    // Perform analysis based on depth
    match depth.as_str() {
        "comprehensive" => {
            // Full repository analysis
            let analysis = repository_analyzer.analyze_repository(&target).await?;
            
            let elapsed = start.elapsed();
            
            println!("\n✅ {} complete in {:.2}s", 
                style("Analysis").green().bold(),
                elapsed.as_secs_f64()
            );
            
            // Display architecture information
            display_architecture_info(&analysis.architecture);
            
            // Display quality report
            display_quality_report(&analysis.quality);
            
            // Display security summary
            if analysis.security.vulnerability_count > 0 {
                display_security_summary(&analysis.security);
            }
            
            // Display performance summary
            if !analysis.performance.hotspots.is_empty() {
                display_performance_summary(&analysis.performance);
            }
            
            // Display technical debt
            display_technical_debt(&analysis.technical_debt);
            
            // Display top recommendations
            display_recommendations(&analysis.recommendations);
        }
        "quick" => {
            // Quick analysis - architecture and quality only
            println!("⚡ Running quick analysis...");
            
            let analysis = repository_analyzer.analyze_repository(&target).await?;
            
            println!("\n📊 {} Results:", style("Quick Analysis").bold());
            println!("  Architecture: {} (confidence: {:.0}%)",
                style(format!("{:?}", analysis.architecture.primary_pattern)).cyan(),
                analysis.architecture.confidence * 100.0
            );
            println!("  Quality Score: {}",
                format_quality_score(analysis.quality.overall_score)
            );
            println!("  Files Analyzed: {}", 
                style(analysis.quality.hotspots.len()).yellow()
            );
        }
        _ => {
            // Default analysis
            let analysis = repository_analyzer.analyze_repository(&target).await?;
            
            println!("\n📊 {} Summary:", style("Repository").bold());
            println!("  Architecture: {:?}", analysis.architecture.primary_pattern);
            println!("  Quality: {:.1}/10", analysis.quality.overall_score);
            println!("  Issues: {}", analysis.quality.issues.len());
            println!("  Tech Debt: ${:.0}", analysis.technical_debt.estimated_cost);
        }
    }
    
    // Show quality score if requested
    if quality && depth != "comprehensive" {
        let analysis = repository_analyzer.analyze_repository(&target).await?;
        println!("\n🏆 {}: {}",
            style("Quality Score").bold(),
            format_quality_score(analysis.quality.overall_score)
        );
    }
    
    Ok(())
}

/// Display architecture information
fn display_architecture_info(architecture: &crate::analysis::repository_intelligence::ArchitectureInfo) {
    println!("\n🏗️  {} {}", style("Architecture:").bold().cyan(), style("━".repeat(50)).dim());
    
    println!("  Primary Pattern: {} (confidence: {:.0}%)",
        style(format!("{:?}", architecture.primary_pattern)).green().bold(),
        architecture.confidence * 100.0
    );
    
    if architecture.detected_patterns.len() > 1 {
        println!("  Also detected:");
        for pattern in &architecture.detected_patterns[1..] {
            println!("    • {:?}", pattern);
        }
    }
    
    println!("  Adherence Score: {}",
        format_adherence_score(architecture.adherence_score)
    );
    
    // Show layers if detected
    if !architecture.layers.is_empty() {
        println!("\n  Layers detected:");
        for layer in &architecture.layers {
            println!("    📁 {} ({} modules)", 
                style(&layer.name).cyan(),
                layer.modules.len()
            );
            
            if !layer.violations.is_empty() {
                println!("       ⚠️  {} violations found", 
                    style(layer.violations.len()).red()
                );
            }
        }
    }
    
    // Show components
    if !architecture.components.is_empty() {
        println!("\n  Major Components:");
        for (i, component) in architecture.components.iter().take(5).enumerate() {
            println!("    {}. {} ({:?}) - {} modules",
                i + 1,
                style(&component.name).yellow(),
                component.kind,
                component.modules.len()
            );
        }
    }
}

/// Display quality report
fn display_quality_report(quality: &crate::analysis::repository_intelligence::QualityReport) {
    println!("\n📊 {} {}", style("Code Quality:").bold().cyan(), style("━".repeat(50)).dim());
    
    println!("  Overall Score: {}",
        format_quality_score(quality.overall_score)
    );
    
    // Display metrics
    println!("\n  Key Metrics:");
    println!("    • Maintainability Index: {:.1}", quality.metrics.maintainability_index);
    println!("    • Avg Complexity: {:.1}", quality.metrics.cyclomatic_complexity);
    println!("    • Test Coverage: {:.0}%", quality.metrics.test_coverage * 100.0);
    println!("    • Documentation: {:.0}%", quality.metrics.documentation_coverage * 100.0);
    println!("    • Code Duplication: {:.1}%", quality.metrics.duplication_ratio * 100.0);
    
    // Display top issues
    if !quality.issues.is_empty() {
        println!("\n  Top Issues:");
        let critical_issues = quality.issues.iter()
            .filter(|i| i.severity == Severity::Critical || i.severity == Severity::High)
            .take(5);
            
        for issue in critical_issues {
            let severity_icon = match issue.severity {
                Severity::Critical => "🔴",
                Severity::High => "🟠",
                Severity::Medium => "🟡",
                Severity::Low => "🟢",
            };
            
            println!("    {} {:?}: {}",
                severity_icon,
                issue.kind,
                issue.message
            );
        }
        
        let total_issues = quality.issues.len();
        if total_issues > 5 {
            println!("    ... and {} more issues", total_issues - 5);
        }
    }
    
    // Display hotspots
    if !quality.hotspots.is_empty() {
        println!("\n  Quality Hotspots:");
        for (i, hotspot) in quality.hotspots.iter().take(3).enumerate() {
            println!("    {}. {} (score: {:.1}, {} issues)",
                i + 1,
                style(hotspot.file.display()).red(),
                hotspot.score,
                hotspot.issues_count
            );
        }
    }
}

/// Display security summary
fn display_security_summary(security: &crate::analysis::repository_intelligence::SecurityReport) {
    println!("\n🔒 {} {}", style("Security:").bold().cyan(), style("━".repeat(50)).dim());
    
    println!("  Vulnerabilities: {} (Risk Score: {:.1})",
        style(security.vulnerability_count).red().bold(),
        security.risk_score
    );
    
    // Group by severity
    let critical = security.vulnerabilities.iter().filter(|v| v.severity == Severity::Critical).count();
    let high = security.vulnerabilities.iter().filter(|v| v.severity == Severity::High).count();
    let medium = security.vulnerabilities.iter().filter(|v| v.severity == Severity::Medium).count();
    
    if critical > 0 || high > 0 {
        println!("    🔴 Critical: {} | 🟠 High: {} | 🟡 Medium: {}",
            critical, high, medium
        );
    }
    
    // Show top vulnerabilities
    for vuln in security.vulnerabilities.iter().take(3) {
        println!("    • {:?} at {}:{}",
            vuln.kind,
            vuln.location.display(),
            vuln.line
        );
    }
}

/// Display performance summary
fn display_performance_summary(performance: &crate::analysis::repository_intelligence::PerformanceReport) {
    println!("\n⚡ {} {}", style("Performance:").bold().cyan(), style("━".repeat(50)).dim());
    
    println!("  Hotspots: {} | Bottlenecks: {} | Opportunities: {}",
        style(performance.hotspots.len()).yellow(),
        style(performance.bottlenecks.len()).red(),
        style(performance.optimization_opportunities.len()).green()
    );
    
    // Show top hotspots
    if !performance.hotspots.is_empty() {
        println!("\n  Top Performance Issues:");
        for (i, hotspot) in performance.hotspots.iter().take(3).enumerate() {
            println!("    {}. {} in {} - {:?}",
                i + 1,
                style(&hotspot.function).red(),
                hotspot.location.display(),
                hotspot.issue_kind
            );
        }
    }
}

/// Display technical debt
fn display_technical_debt(debt: &crate::analysis::repository_intelligence::TechnicalDebtReport) {
    println!("\n💰 {} {}", style("Technical Debt:").bold().cyan(), style("━".repeat(50)).dim());
    
    println!("  Total Debt: {} hours (${:.0} @ $100/hr)",
        style(format!("{:.0}", debt.total_debt_hours)).red().bold(),
        debt.estimated_cost
    );
    
    println!("  Debt Ratio: {:.1}% of development effort",
        debt.debt_ratio * 100.0
    );
    
    // Show debt by category
    if !debt.debt_by_category.is_empty() {
        println!("\n  By Category:");
        let mut categories: Vec<_> = debt.debt_by_category.iter().collect();
        categories.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        for (category, hours) in categories.iter().take(5) {
            let percentage = (*hours / debt.total_debt_hours) * 100.0;
            println!("    • {}: {:.0} hours ({:.0}%)",
                category,
                hours,
                percentage
            );
        }
    }
}

/// Display recommendations
fn display_recommendations(recommendations: &[crate::analysis::repository_intelligence::Recommendation]) {
    if recommendations.is_empty() {
        return;
    }
    
    println!("\n💡 {} {}", style("Top Recommendations:").bold().cyan(), style("━".repeat(50)).dim());
    
    for (i, rec) in recommendations.iter().take(5).enumerate() {
        let priority_icon = match rec.priority {
            Priority::Urgent => "🚨",
            Priority::High => "⚠️",
            Priority::Medium => "📌",
            Priority::Low => "💭",
        };
        
        let impact_color = match rec.impact {
            Impact::Major => console::Color::Red,
            Impact::Significant => console::Color::Yellow,
            Impact::Moderate => console::Color::Blue,
            Impact::Minimal => console::Color::Green,
        };
        
        println!("\n  {}. {} {}",
            i + 1,
            priority_icon,
            style(&rec.title).bold()
        );
        
        println!("     {}", rec.description);
        println!("     Impact: {} | Effort: {:?}",
            style(format!("{:?}", rec.impact)).fg(impact_color),
            rec.effort
        );
        
        if !rec.specific_actions.is_empty() {
            println!("     Actions:");
            for action in &rec.specific_actions {
                println!("       ✓ {}", action);
            }
        }
    }
}

/// Format quality score with color
fn format_quality_score(score: f32) -> String {
    let color = if score >= 8.0 {
        console::Color::Green
    } else if score >= 6.0 {
        console::Color::Yellow
    } else if score >= 4.0 {
        console::Color::Magenta
    } else {
        console::Color::Red
    };
    
    let emoji = if score >= 8.0 {
        "🌟"
    } else if score >= 6.0 {
        "✨"
    } else if score >= 4.0 {
        "⚡"
    } else {
        "⚠️"
    };
    
    format!("{} {}", emoji, style(format!("{:.1}/10", score)).fg(color).bold())
}

/// Format adherence score
fn format_adherence_score(score: f32) -> String {
    let percentage = score * 100.0;
    let color = if percentage >= 80.0 {
        console::Color::Green
    } else if percentage >= 60.0 {
        console::Color::Yellow
    } else {
        console::Color::Red
    };
    
    style(format!("{:.0}%", percentage)).fg(color).to_string()
}

/// Handle dependency analysis command
pub async fn handle_dependency_analysis(
    path: PathBuf,
    detect_cycles: bool,
    visualize: bool,
) -> Result<()> {
    println!("🕸️  {} dependencies...", style("Analyzing").bold().cyan());
    
    let analyzer = DependencyAnalyzer::new().await?;
    let analysis = analyzer.analyze_project(&path).await?;
    
    // Display statistics
    println!("\n📊 Dependency Statistics:");
    println!("  Total Modules: {}", style(analysis.stats.total_modules).green());
    println!("  Total Dependencies: {}", style(analysis.stats.total_dependencies).yellow());
    println!("  External Dependencies: {}", style(analysis.stats.external_dependencies).blue());
    println!("  Max Dependency Depth: {}", analysis.stats.max_depth);
    println!("  Avg Dependencies/Module: {:.1}", analysis.stats.avg_dependencies_per_module);
    
    // Show circular dependencies if requested
    if detect_cycles {
        if analysis.circular_dependencies.is_empty() {
            println!("\n✅ {} detected!", style("No circular dependencies").green().bold());
        } else {
            println!("\n⚠️  {} circular dependencies:", 
                style(analysis.circular_dependencies.len()).red().bold()
            );
            
            for (i, cycle) in analysis.circular_dependencies.iter().enumerate() {
                println!("\n  Cycle {}:", i + 1);
                for (j, module) in cycle.modules.iter().enumerate() {
                    if j > 0 {
                        println!("    ↓");
                    }
                    println!("    {}", style(module.display()).yellow());
                }
                println!("    ↓");
                println!("    {} (cycle)", style(cycle.modules[0].display()).yellow());
                
                println!("  Severity: {:?}", cycle.severity);
                println!("  Suggested breaking points:");
                for point in &cycle.breaking_points {
                    println!("    • {}", point.display());
                }
            }
        }
    }
    
    // Show most depended on modules
    if !analysis.stats.most_depended_on.is_empty() {
        println!("\n🎯 Most Depended On:");
        for (module, count) in analysis.stats.most_depended_on.iter().take(5) {
            println!("  {} - {} dependents", 
                style(module.display()).cyan(),
                style(count).yellow()
            );
        }
    }
    
    // Generate visualization if requested
    if visualize {
        println!("\n📈 Dependency Graph (DOT format):");
        println!("{}", analysis.visualization_dot);
        println!("\n💡 Tip: Save to file and visualize with: dot -Tpng deps.dot -o deps.png");
    }
    
    Ok(())
}

/// Handle find circular dependencies command
pub async fn handle_find_circular_deps(path: PathBuf) -> Result<()> {
    println!("🔄 {} circular dependencies...", style("Detecting").bold().cyan());
    
    let analyzer = DependencyAnalyzer::new().await?;
    let analysis = analyzer.analyze_project(&path).await?;
    
    if analysis.circular_dependencies.is_empty() {
        println!("\n✅ {} circular dependencies found!", style("No").green().bold());
        println!("🎉 Your codebase has a clean dependency structure!");
    } else {
        println!("\n⚠️  Found {} circular {}:",
            style(analysis.circular_dependencies.len()).red().bold(),
            if analysis.circular_dependencies.len() == 1 { "dependency" } else { "dependencies" }
        );
        
        for (i, cycle) in analysis.circular_dependencies.iter().enumerate() {
            println!("\n{}. Cycle with {} modules (Severity: {:?}):",
                i + 1,
                cycle.modules.len(),
                cycle.severity
            );
            
            // Show the cycle
            for (j, module) in cycle.modules.iter().enumerate() {
                let arrow = if j == 0 { "┌─" } else if j == cycle.modules.len() - 1 { "└─" } else { "├─" };
                println!("  {} {}", arrow, style(module.display()).yellow());
            }
            println!("  └─→ {} (back to start)", style(cycle.modules[0].display()).yellow());
            
            // Show suggested fixes
            if !cycle.breaking_points.is_empty() {
                println!("\n  💡 Suggested refactoring points:");
                for point in &cycle.breaking_points {
                    println!("     • Extract interface from {}", style(point.display()).green());
                }
            }
        }
        
        // Summary
        let critical = analysis.circular_dependencies.iter()
            .filter(|c| matches!(c.severity, crate::analysis::dependency::Severity::Critical))
            .count();
        let high = analysis.circular_dependencies.iter()
            .filter(|c| matches!(c.severity, crate::analysis::dependency::Severity::High))
            .count();
            
        if critical > 0 || high > 0 {
            println!("\n⚠️  {} {} and {} {} severity cycles need immediate attention!",
                style(critical).red().bold(),
                "critical",
                style(high).yellow().bold(),
                "high"
            );
        }
    }
    
    Ok(())
}