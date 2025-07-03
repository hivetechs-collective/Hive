//! Analytics commands for metrics and insights
//!
//! This module provides CLI commands for:
//! - Real-time metrics collection
//! - Cost analysis and optimization
//! - Performance dashboards
//! - Business intelligence reports

use anyhow::{Context, Result};
use clap::Subcommand;
use console::{style, Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

use crate::{
    core::analytics::{
        get_analytics_engine, initialize_analytics, AnalyticsConfig,
        AggregationPeriod, ReportType, TrendDirection, InsightCategory,
        ImpactLevel, DashboardWidget,
    },
    analytics::{
        get_advanced_analytics, initialize_advanced_analytics,
        AdvancedAnalyticsConfig, ForecastHorizon,
    },
};

/// Analytics commands
#[derive(Debug, Subcommand)]
pub enum AnalyticsCommand {
    /// Generate analytics report
    Report {
        /// Report type
        #[arg(long, value_enum, default_value = "executive")]
        r#type: ReportTypeArg,
        /// Time period
        #[arg(long, value_enum, default_value = "week")]
        period: PeriodArg,
        /// Output format
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    
    /// Analyze trends
    Trends {
        /// Specific metric to analyze
        metric: Option<String>,
        /// Time period
        #[arg(long, value_enum, default_value = "month")]
        period: PeriodArg,
        /// Enable prediction
        #[arg(long)]
        predict: bool,
        /// Prediction horizon (days)
        #[arg(long, default_value = "7")]
        horizon: u32,
    },
    
    /// Cost analysis and optimization
    Cost {
        /// Show cost breakdown
        #[arg(long)]
        breakdown: bool,
        /// Generate optimization recommendations
        #[arg(long)]
        optimize: bool,
        /// Time period
        #[arg(long, value_enum, default_value = "week")]
        period: PeriodArg,
    },
    
    /// Performance analytics
    Performance {
        /// Show model performance
        #[arg(long)]
        models: bool,
        /// Detailed analysis
        #[arg(long)]
        detailed: bool,
    },
    
    /// Analytics overview
    Overview {
        /// Time period
        #[arg(long, default_value = "week")]
        period: String,
        /// Output format
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    
    /// Real-time dashboard
    Dashboard {
        /// Enable live updates
        #[arg(long)]
        live: bool,
        /// Refresh interval (seconds)
        #[arg(long, default_value = "5")]
        refresh: u64,
    },
    
    /// Generate comprehensive analytics
    Generate {
        /// Include all analytics
        #[arg(long)]
        comprehensive: bool,
    },
    
    /// Collect metrics (usually runs automatically)
    Collect,
}

/// Report type argument
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ReportTypeArg {
    Executive,
    Operational,
    Financial,
    Performance,
}

/// Period argument
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum PeriodArg {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl From<ReportTypeArg> for ReportType {
    fn from(arg: ReportTypeArg) -> Self {
        match arg {
            ReportTypeArg::Executive => ReportType::Executive,
            ReportTypeArg::Operational => ReportType::Operational,
            ReportTypeArg::Financial => ReportType::Financial,
            ReportTypeArg::Performance => ReportType::Performance,
        }
    }
}

impl From<PeriodArg> for AggregationPeriod {
    fn from(arg: PeriodArg) -> Self {
        match arg {
            PeriodArg::Day => AggregationPeriod::Day,
            PeriodArg::Week => AggregationPeriod::Week,
            PeriodArg::Month => AggregationPeriod::Month,
            PeriodArg::Quarter => AggregationPeriod::Quarter,
            PeriodArg::Year => AggregationPeriod::Year,
        }
    }
}

/// Execute analytics commands
pub async fn execute(cmd: AnalyticsCommand) -> Result<()> {
    // Ensure both analytics engines are initialized
    initialize_analytics(None).await.ok();
    initialize_advanced_analytics(None).await.ok();
    
    match cmd {
        AnalyticsCommand::Report { r#type, period, format } => {
            generate_report(r#type.into(), period.into(), &format).await
        }
        AnalyticsCommand::Trends { metric, period, predict, horizon } => {
            analyze_trends_advanced(metric.as_deref(), period.into(), predict, horizon).await
        }
        AnalyticsCommand::Cost { breakdown, optimize, period } => {
            analyze_costs_advanced(breakdown, optimize, period.into()).await
        }
        AnalyticsCommand::Performance { models, detailed } => {
            show_performance_analytics(models, detailed).await
        }
        AnalyticsCommand::Overview { period, format } => {
            show_analytics_overview(&period, &format).await
        }
        AnalyticsCommand::Dashboard { live, refresh } => {
            show_dashboard_advanced(live, refresh).await
        }
        AnalyticsCommand::Generate { comprehensive } => {
            generate_analytics(comprehensive).await
        }
        AnalyticsCommand::Collect => {
            collect_metrics().await
        }
    }
}

/// Generate analytics report
async fn generate_report(
    report_type: ReportType,
    period: AggregationPeriod,
    format: &str,
) -> Result<()> {
    let engine = get_analytics_engine().await?;
    
    println!("{}", style("📊 Generating report...").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Collecting metrics and generating report...");
    
    let report = engine.generate_report(report_type, period).await?;
    pb.finish_and_clear();
    
    match format {
        "markdown" | "md" => {
            println!("\n{}", report);
        }
        "html" => {
            let filename = format!("report_{:?}_{}.html", report_type, Utc::now().format("%Y%m%d"));
            let html = format!(
                "<!DOCTYPE html><html><head><title>Hive AI Report</title></head><body>{}</body></html>",
                markdown_to_html(&report)
            );
            tokio::fs::write(&filename, html).await?;
            println!("{} Report saved to {}", style("✓").green().bold(), filename);
        }
        _ => {
            println!("{}", style("Unsupported format. Using markdown.").yellow());
            println!("\n{}", report);
        }
    }
    
    Ok(())
}

/// Analyze trends
async fn analyze_trends(metric: Option<&str>, period: AggregationPeriod) -> Result<()> {
    let engine = get_analytics_engine().await?;
    
    println!("{}", style("📈 Analyzing trends...").cyan().bold());
    
    // For now, generate a report that includes trends
    let report = engine.generate_report(ReportType::Performance, period).await?;
    
    // Extract and display trends section
    if let Some(trends_start) = report.find("## Trends") {
        if let Some(trends_end) = report[trends_start..].find("\n## ") {
            let trends_section = &report[trends_start..trends_start + trends_end];
            println!("\n{}", trends_section);
        } else {
            let trends_section = &report[trends_start..];
            println!("\n{}", trends_section);
        }
    } else {
        println!("{}", style("No trend data available").yellow());
    }
    
    Ok(())
}

/// Analyze costs
async fn analyze_costs(breakdown: bool, optimize: bool, period: AggregationPeriod) -> Result<()> {
    let engine = get_analytics_engine().await?;
    
    println!("{}", style("💰 Analyzing costs...").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Calculating costs and analyzing usage patterns...");
    
    let cost_metrics = engine.analyze_costs().await?;
    pb.finish_and_clear();
    
    // Show summary
    println!("\n{}", style("Cost Summary").blue().bold());
    println!("Total cost: ${:.2}", cost_metrics.total_cost);
    println!("Average per query: ${:.4}", cost_metrics.average_cost_per_query);
    
    if breakdown {
        // Show cost breakdown by model
        if !cost_metrics.cost_by_model.is_empty() {
            println!("\n{}", style("Cost by Model").blue().bold());
            let mut models: Vec<_> = cost_metrics.cost_by_model.iter().collect();
            models.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
            
            for (model, cost) in models {
                let percentage = (cost / cost_metrics.total_cost) * 100.0;
                println!("  {} ${:.2} ({:.1}%)",
                    style(model).cyan(),
                    cost,
                    percentage
                );
            }
        }
        
        // Show cost by user
        if !cost_metrics.cost_by_user.is_empty() {
            println!("\n{}", style("Top Users by Cost").blue().bold());
            let mut users: Vec<_> = cost_metrics.cost_by_user.iter().collect();
            users.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
            users.truncate(5);
            
            for (user, cost) in users {
                let percentage = (cost / cost_metrics.total_cost) * 100.0;
                println!("  {} ${:.2} ({:.1}%)",
                    style(user).cyan(),
                    cost,
                    percentage
                );
            }
        }
    }
    
    if optimize {
        println!("\n{}", style("💡 Cost Optimization Recommendations").yellow().bold());
        
        let recommendations = engine.get_cost_optimizations().await?;
        
        if recommendations.is_empty() {
            println!("{}", style("No optimization opportunities found. Costs are well managed!").green());
        } else {
            for (i, rec) in recommendations.iter().enumerate() {
                println!("{}. {}", i + 1, rec);
            }
        }
    }
    
    Ok(())
}

/// Show real-time dashboard
async fn show_dashboard(real_time: bool) -> Result<()> {
    let engine = get_analytics_engine().await?;
    let term = Term::stdout();
    
    println!("{}", style("📊 Analytics Dashboard").cyan().bold());
    
    if real_time {
        println!("{}", style("Real-time mode enabled. Press Ctrl+C to exit.").dim());
    }
    
    loop {
        let dashboard = engine.generate_dashboard().await?;
        
        // Clear screen for real-time update
        if real_time {
            term.clear_screen()?;
        }
        
        // Display header
        println!("{}", style(&dashboard.title).blue().bold());
        println!("Last updated: {}", dashboard.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("{}", style("─".repeat(80)).dim());
        
        // Display widgets
        for widget in &dashboard.widgets {
            display_widget(widget)?;
            println!();
        }
        
        if !real_time {
            break;
        }
        
        // Wait before next update
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    
    Ok(())
}

/// Display a dashboard widget
fn display_widget(widget: &DashboardWidget) -> Result<()> {
    println!("{}", style(&widget.title).green().bold());
    
    match widget.widget_type.as_str() {
        "line_chart" => {
            // Simple ASCII line chart
            if let Ok(chart_data) = serde_json::from_value::<serde_json::Value>(widget.data.clone()) {
                println!("  📈 Time series data");
                // TODO: Implement actual ASCII chart rendering
            }
        }
        "gauge" => {
            if let Ok(gauge_data) = serde_json::from_value::<HashMap<String, serde_json::Value>>(widget.data.clone()) {
                if let Some(value) = gauge_data.get("value").and_then(|v| v.as_f64()) {
                    let unit = gauge_data.get("unit").and_then(|v| v.as_str()).unwrap_or("");
                    let max = gauge_data.get("max").and_then(|v| v.as_f64()).unwrap_or(100.0);
                    
                    let percentage = (value / max * 100.0).min(100.0);
                    let filled = (percentage / 5.0) as usize;
                    let bar = "█".repeat(filled) + &"░".repeat(20 - filled);
                    
                    println!("  {} {:.1}{} ({:.0}%)", bar, value, unit, percentage);
                }
            }
        }
        "pie_chart" => {
            if let Ok(pie_data) = serde_json::from_value::<HashMap<String, serde_json::Value>>(widget.data.clone()) {
                if let Some(data) = pie_data.get("data").and_then(|v| v.as_array()) {
                    for item in data {
                        if let Some(obj) = item.as_object() {
                            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                            let value = obj.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("  • {}: ${:.2}", name, value);
                        }
                    }
                }
            }
        }
        "status_grid" => {
            if let Ok(status_data) = serde_json::from_value::<HashMap<String, serde_json::Value>>(widget.data.clone()) {
                if let Some(components) = status_data.get("components").and_then(|v| v.as_array()) {
                    for component in components {
                        if let Some(obj) = component.as_object() {
                            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                            let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
                            let icon = match status {
                                "healthy" => "✅",
                                "warning" => "⚠️",
                                "critical" => "❌",
                                _ => "❓",
                            };
                            println!("  {} {}", icon, name);
                        }
                    }
                }
            }
        }
        _ => {
            println!("  {}", style("Widget type not yet implemented").dim());
        }
    }
    
    Ok(())
}

/// Generate comprehensive analytics
async fn generate_analytics(comprehensive: bool) -> Result<()> {
    let engine = get_analytics_engine().await?;
    
    println!("{}", style("🔄 Generating analytics...").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    
    // Collect metrics
    pb.set_message("Collecting metrics...");
    engine.collect_metrics().await?;
    
    // Generate reports
    if comprehensive {
        pb.set_message("Generating executive report...");
        let exec_report = engine.generate_report(ReportType::Executive, AggregationPeriod::Month).await?;
        
        pb.set_message("Generating performance report...");
        let perf_report = engine.generate_report(ReportType::Performance, AggregationPeriod::Week).await?;
        
        pb.set_message("Analyzing costs...");
        let cost_metrics = engine.analyze_costs().await?;
        let cost_opts = engine.get_cost_optimizations().await?;
        
        pb.finish_and_clear();
        
        // Save comprehensive report
        let filename = format!("analytics_comprehensive_{}.md", Utc::now().format("%Y%m%d"));
        let mut full_report = String::new();
        
        full_report.push_str("# Comprehensive Analytics Report\n\n");
        full_report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        full_report.push_str(&exec_report);
        full_report.push_str("\n\n---\n\n");
        full_report.push_str(&perf_report);
        full_report.push_str("\n\n---\n\n");
        
        full_report.push_str("## Cost Analysis\n\n");
        full_report.push_str(&format!("Total Cost: ${:.2}\n", cost_metrics.total_cost));
        full_report.push_str(&format!("Average per Query: ${:.4}\n\n", cost_metrics.average_cost_per_query));
        
        if !cost_opts.is_empty() {
            full_report.push_str("### Optimization Recommendations\n\n");
            for opt in cost_opts {
                full_report.push_str(&format!("- {}\n", opt));
            }
        }
        
        tokio::fs::write(&filename, full_report).await?;
        
        println!("{} Comprehensive report saved to {}",
            style("✓").green().bold(),
            style(&filename).cyan()
        );
    } else {
        pb.finish_and_clear();
        println!("{} Metrics collected successfully", style("✓").green().bold());
    }
    
    Ok(())
}

/// Collect metrics
async fn collect_metrics() -> Result<()> {
    let engine = get_analytics_engine().await?;
    
    println!("{}", style("📊 Collecting metrics...").cyan().bold());
    
    engine.collect_metrics().await?;
    
    println!("{} Metrics collected successfully", style("✓").green().bold());
    
    Ok(())
}

/// Simple markdown to HTML conversion
fn markdown_to_html(markdown: &str) -> String {
    markdown
        .replace("# ", "<h1>")
        .replace("\n## ", "</h1>\n<h2>")
        .replace("\n### ", "</h2>\n<h3>")
        .replace("\n", "<br>\n")
        .replace("**", "<strong>")
        .replace("- ", "• ")
        + "</h3>"
}

/// Analyze trends with advanced ML predictions
async fn analyze_trends_advanced(
    metric: Option<&str>,
    period: AggregationPeriod,
    predict: bool,
    horizon_days: u32,
) -> Result<()> {
    let engine = get_advanced_analytics().await?;
    
    println!("{}", style("📈 Analyzing trends with ML...").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    
    if let Some(metric_name) = metric {
        pb.set_message(&format!("Analyzing trend for metric: {}", metric_name));
        
        let horizon = if predict {
            Some(ForecastHorizon::Custom(horizon_days as i64))
        } else {
            None
        };
        
        let prediction = engine.trends().analyze_metric(metric_name, horizon).await?;
        pb.finish_and_clear();
        
        println!("\n{}", style(&format!("Trend Analysis: {}", metric_name)).blue().bold());
        println!("Direction: {:?}", prediction.trend_direction);
        println!("Model Accuracy: {:.1}%", prediction.model_accuracy * 100.0);
        
        if predict && !prediction.predictions.is_empty() {
            println!("\n{}", style("Predictions:").green().bold());
            for (i, point) in prediction.predictions.iter().take(5).enumerate() {
                println!("  Day {}: {:.2} (±{:.2})",
                    i + 1,
                    point.predicted_value,
                    point.upper_bound - point.predicted_value
                );
            }
        }
    } else {
        pb.set_message("Analyzing all trends...");
        let summary = engine.trends().get_current_trends().await?;
        pb.finish_and_clear();
        
        println!("\n{}", summary.format_markdown()?);
    }
    
    Ok(())
}

/// Analyze costs with advanced intelligence
async fn analyze_costs_advanced(
    breakdown: bool,
    optimize: bool,
    period: AggregationPeriod,
) -> Result<()> {
    let engine = get_advanced_analytics().await?;
    
    println!("{}", style("💰 Analyzing costs with intelligence...").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Performing cost analysis...");
    
    let insights = engine.costs().get_insights().await?;
    pb.finish_and_clear();
    
    println!("\n{}", insights.format_markdown()?);
    
    if breakdown {
        println!("\n{}", style("Detailed Cost Breakdown").blue().bold());
        
        // Model efficiency analysis
        if !insights.efficiency_metrics.model_efficiency.is_empty() {
            println!("\n{}", style("Model Efficiency:").green());
            for (model, efficiency) in &insights.efficiency_metrics.model_efficiency {
                println!("  {} - Score: {:.1}%, Cost Effectiveness: {:.1}",
                    style(model).cyan(),
                    efficiency.efficiency_score,
                    efficiency.cost_effectiveness
                );
            }
        }
    }
    
    if optimize {
        println!("\n{}", style("💡 Advanced Optimization Strategy").yellow().bold());
        
        for (i, opt) in insights.optimization_opportunities.iter().enumerate() {
            println!("\n{}. {} (Priority: {:?})",
                i + 1,
                style(&opt.title).bold(),
                opt.priority
            );
            println!("   {}", opt.description);
            println!("   Potential Savings: ${:.2}", opt.potential_savings);
            println!("   Implementation Effort: {:?}", opt.implementation_effort);
            
            if !opt.action_items.is_empty() {
                println!("   Action Items:");
                for action in &opt.action_items {
                    println!("   - {}", action);
                }
            }
        }
    }
    
    Ok(())
}

/// Show performance analytics
async fn show_performance_analytics(models: bool, detailed: bool) -> Result<()> {
    let engine = get_advanced_analytics().await?;
    
    println!("{}", style("⚡ Performance Analytics").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Analyzing performance metrics...");
    
    let metrics = engine.performance().get_metrics().await?;
    pb.finish_and_clear();
    
    println!("\n{}", metrics.format_markdown()?);
    
    if models && !metrics.model_performance.is_empty() {
        println!("\n{}", style("Model Performance Details").blue().bold());
        
        for (model, perf) in &metrics.model_performance {
            println!("\n{}", style(model).green().bold());
            println!("  Requests: {}", perf.total_requests);
            println!("  Success Rate: {:.1}%", perf.success_rate * 100.0);
            println!("  Avg Latency: {:.0}ms", perf.avg_latency_ms);
            println!("  P95 Latency: {:.0}ms", perf.p95_latency_ms);
            println!("  Tokens/sec: {:.1}", perf.tokens_per_second);
            
            if detailed && !perf.stage_performance.is_empty() {
                println!("  Stage Performance:");
                for (stage, stage_perf) in &perf.stage_performance {
                    println!("    - {}: {:.0}ms", stage, stage_perf.avg_duration_ms);
                }
            }
        }
    }
    
    Ok(())
}

/// Show analytics overview
async fn show_analytics_overview(period: &str, format: &str) -> Result<()> {
    let engine = get_advanced_analytics().await?;
    
    println!("{}", style("📊 Analytics Overview").cyan().bold());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message("Generating comprehensive overview...");
    
    let overview = engine.overview(period, format).await?;
    pb.finish_and_clear();
    
    match format {
        "json" => {
            println!("{}", overview);
        }
        _ => {
            println!("\n{}", overview);
        }
    }
    
    Ok(())
}

/// Show advanced dashboard
async fn show_dashboard_advanced(live: bool, refresh_secs: u64) -> Result<()> {
    let engine = get_advanced_analytics().await?;
    let term = Term::stdout();
    
    println!("{}", style("📊 Advanced Analytics Dashboard").cyan().bold());
    
    if live {
        println!("{}", style("Live mode enabled. Press Ctrl+C to exit.").dim());
        
        // Subscribe to real-time updates
        let mut receiver = engine.dashboard().subscribe().await;
        
        loop {
            // Clear screen for live update
            term.clear_screen()?;
            
            // Render dashboard
            let (width, height) = term.size();
            let ascii_dashboard = engine.dashboard().render_ascii(
                width as usize,
                height.saturating_sub(2) as usize
            ).await?;
            
            println!("{}", ascii_dashboard);
            
            // Wait for next update or timeout
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(refresh_secs)) => {},
                Ok(metric) = receiver.recv() => {
                    // Handle real-time metric update
                    println!("\n{} {}: {:.2} {}",
                        style("▶").cyan(),
                        metric.name,
                        metric.value,
                        metric.unit
                    );
                }
            }
        }
    } else {
        // Single render
        let (width, height) = term.size();
        let ascii_dashboard = engine.dashboard().render_ascii(
            width as usize,
            height.saturating_sub(2) as usize
        ).await?;
        
        println!("{}", ascii_dashboard);
    }
    
    Ok(())
}