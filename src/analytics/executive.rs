//! Executive Reporting System with Professional Visualizations
//!
//! This module provides:
//! - Executive-level summaries and KPIs
//! - Professional report generation
//! - Business insights and recommendations
//! - Visual charts for CLI/TUI display
//! - Export to multiple formats

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::analytics::AdvancedAnalyticsConfig;
use crate::core::database::{get_database, DatabaseStatistics};

/// Executive reporter for high-level business intelligence
pub struct ExecutiveReporter {
    config: Arc<RwLock<AdvancedAnalyticsConfig>>,
    kpi_calculator: Arc<KpiCalculator>,
    insight_generator: Arc<InsightGenerator>,
    visualizer: Arc<Visualizer>,
}

/// Executive summary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub period: ReportPeriod,
    pub generated_at: DateTime<Utc>,
    pub key_metrics: Vec<KeyMetric>,
    pub business_insights: Vec<BusinessInsight>,
    pub recommendations: Vec<ExecutiveRecommendation>,
    pub visualizations: Vec<Visualization>,
    pub health_score: HealthScore,
}

/// Key performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub change_percent: f64,
    pub target: Option<f64>,
    pub status: MetricStatus,
    pub trend: MetricTrend,
}

/// Metric status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricStatus {
    Excellent,
    Good,
    Warning,
    Critical,
}

/// Metric trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricTrend {
    Improving,
    Stable,
    Declining,
}

/// Business insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessInsight {
    pub category: InsightCategory,
    pub title: String,
    pub summary: String,
    pub impact: BusinessImpact,
    pub supporting_data: HashMap<String, f64>,
    pub confidence: f64,
}

/// Insight categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightCategory {
    Growth,
    Efficiency,
    Quality,
    Risk,
    Opportunity,
}

/// Business impact level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusinessImpact {
    High,
    Medium,
    Low,
}

/// Executive recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveRecommendation {
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub expected_outcome: String,
    pub effort_estimate: EffortLevel,
    pub roi_estimate: Option<f64>,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

/// Effort level estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Overall health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub overall: f64,
    pub components: HashMap<String, f64>,
    pub trend: HealthTrend,
}

/// Health trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthTrend {
    Improving,
    Stable,
    Degrading,
}

/// Report period options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom { days: u32 },
}

impl ReportPeriod {
    pub fn to_duration(&self) -> Duration {
        match self {
            Self::Daily => Duration::days(1),
            Self::Weekly => Duration::weeks(1),
            Self::Monthly => Duration::days(30),
            Self::Quarterly => Duration::days(90),
            Self::Yearly => Duration::days(365),
            Self::Custom { days } => Duration::days(*days as i64),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Daily => "Daily".to_string(),
            Self::Weekly => "Weekly".to_string(),
            Self::Monthly => "Monthly".to_string(),
            Self::Quarterly => "Quarterly".to_string(),
            Self::Yearly => "Yearly".to_string(),
            Self::Custom { days } => format!("{} Days", days),
        }
    }
}

/// Report format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Markdown,
    Html,
    Pdf,
    Json,
}

/// Visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visualization {
    pub title: String,
    pub chart_type: ChartType,
    pub data: serde_json::Value,
    pub options: VisualizationOptions,
}

/// Chart types for visualizations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    LineChart,
    BarChart,
    PieChart,
    GaugeChart,
    HeatMap,
    Sparkline,
}

/// Visualization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationOptions {
    pub width: u32,
    pub height: u32,
    pub colors: Vec<String>,
    pub show_legend: bool,
    pub show_grid: bool,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            width: 80,
            height: 20,
            colors: vec![
                "#3498db".to_string(),
                "#2ecc71".to_string(),
                "#f39c12".to_string(),
                "#e74c3c".to_string(),
                "#9b59b6".to_string(),
            ],
            show_legend: true,
            show_grid: true,
        }
    }
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub style: String,
    pub theme: String,
    pub ascii_mode: bool,
}

/// KPI calculator
struct KpiCalculator {
    kpi_definitions: HashMap<String, KpiDefinition>,
}

/// KPI definition
struct KpiDefinition {
    name: String,
    calculation: Box<dyn Fn(&HashMap<String, f64>) -> f64 + Send + Sync>,
    unit: String,
    target: Option<f64>,
    good_direction: Direction,
}

/// Direction for KPI improvement
enum Direction {
    Higher,
    Lower,
}

/// Insight generator
struct InsightGenerator {
    insight_rules: Vec<InsightRule>,
}

/// Insight generation rule
struct InsightRule {
    name: String,
    category: InsightCategory,
    condition: Box<dyn Fn(&ExecutiveData) -> bool + Send + Sync>,
    generator: Box<dyn Fn(&ExecutiveData) -> BusinessInsight + Send + Sync>,
}

/// Executive data for analysis
#[derive(Debug)]
struct ExecutiveData {
    metrics: HashMap<String, f64>,
    statistics: DatabaseStatistics,
    period: ReportPeriod,
}

/// Report visualizer
struct Visualizer {
    ascii_renderer: AsciiRenderer,
}

/// ASCII chart renderer
struct AsciiRenderer;

impl ExecutiveReporter {
    /// Create a new executive reporter
    pub async fn new(config: Arc<RwLock<AdvancedAnalyticsConfig>>) -> Result<Self> {
        info!("Initializing executive reporter");

        let kpi_calculator = Arc::new(KpiCalculator::new());
        let insight_generator = Arc::new(InsightGenerator::new());
        let visualizer = Arc::new(Visualizer::new());

        Ok(Self {
            config,
            kpi_calculator,
            insight_generator,
            visualizer,
        })
    }

    /// Generate executive summary
    pub async fn generate_summary(&self, period: &str) -> Result<ExecutiveSummary> {
        debug!("Generating executive summary for period: {}", period);

        let report_period = self.parse_period(period)?;
        let executive_data = self.collect_executive_data(report_period).await?;

        // Calculate KPIs
        let key_metrics = self.kpi_calculator.calculate_kpis(&executive_data)?;

        // Generate insights
        let business_insights = self.insight_generator.generate_insights(&executive_data)?;

        // Create recommendations
        let recommendations = self.generate_recommendations(&key_metrics, &business_insights)?;

        // Generate visualizations
        let visualizations = self.create_visualizations(&executive_data).await?;

        // Calculate health score
        let health_score = self.calculate_health_score(&key_metrics)?;

        Ok(ExecutiveSummary {
            period: report_period,
            generated_at: Utc::now(),
            key_metrics,
            business_insights,
            recommendations,
            visualizations,
            health_score,
        })
    }

    /// Generate report in specific format
    pub async fn generate_report(&self, period: &str, format: ReportFormat) -> Result<String> {
        let summary = self.generate_summary(period).await?;

        match format {
            ReportFormat::Markdown => self.format_markdown(summary),
            ReportFormat::Html => self.format_html(summary),
            ReportFormat::Json => self.format_json(summary),
            ReportFormat::Pdf => Err(anyhow::anyhow!("PDF generation not yet implemented")),
        }
    }

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()> {
        debug!("Reloading executive reporter configuration");
        Ok(())
    }

    // Private helper methods

    fn parse_period(&self, period: &str) -> Result<ReportPeriod> {
        match period.to_lowercase().as_str() {
            "day" | "daily" => Ok(ReportPeriod::Daily),
            "week" | "weekly" => Ok(ReportPeriod::Weekly),
            "month" | "monthly" => Ok(ReportPeriod::Monthly),
            "quarter" | "quarterly" => Ok(ReportPeriod::Quarterly),
            "year" | "yearly" => Ok(ReportPeriod::Yearly),
            _ => {
                // Try to parse as number of days
                if let Ok(days) = period.parse::<u32>() {
                    Ok(ReportPeriod::Custom { days })
                } else {
                    Err(anyhow::anyhow!("Invalid period: {}", period))
                }
            }
        }
    }

    async fn collect_executive_data(&self, period: ReportPeriod) -> Result<ExecutiveData> {
        let db = get_database().await?;
        let statistics = db.get_statistics().await?;

        // Collect various metrics
        let mut metrics = HashMap::new();

        // Basic metrics from statistics
        metrics.insert(
            "total_conversations".to_string(),
            statistics.conversation_count as f64,
        );
        metrics.insert(
            "total_messages".to_string(),
            statistics.message_count as f64,
        );
        metrics.insert("active_users".to_string(), statistics.user_count as f64);
        metrics.insert(
            "models_available".to_string(),
            statistics.model_count as f64,
        );

        // Calculate derived metrics
        if statistics.conversation_count > 0 {
            metrics.insert(
                "avg_messages_per_conversation".to_string(),
                statistics.message_count as f64 / statistics.conversation_count as f64,
            );
        }

        // Add time-based metrics (placeholder - would query from database)
        let now = Utc::now();
        let period_start = now - period.to_duration();

        // Placeholder values - in production, these would be calculated from actual data
        metrics.insert("period_conversations".to_string(), 150.0);
        metrics.insert("period_cost".to_string(), 45.67);
        metrics.insert("period_queries".to_string(), 1250.0);
        metrics.insert("avg_response_time_ms".to_string(), 245.0);
        metrics.insert("success_rate".to_string(), 0.985);
        metrics.insert("user_satisfaction".to_string(), 4.7);

        Ok(ExecutiveData {
            metrics,
            statistics,
            period,
        })
    }

    fn generate_recommendations(
        &self,
        metrics: &[KeyMetric],
        insights: &[BusinessInsight],
    ) -> Result<Vec<ExecutiveRecommendation>> {
        let mut recommendations = Vec::new();

        // Check for high cost metrics
        if let Some(cost_metric) = metrics.iter().find(|m| m.name == "Cost per Query") {
            if cost_metric.value > 0.10 {
                recommendations.push(ExecutiveRecommendation {
                    priority: Priority::High,
                    title: "Optimize Model Selection".to_string(),
                    description: "Current cost per query is above optimal threshold. Consider implementing intelligent model routing to use more cost-effective models for simpler queries.".to_string(),
                    expected_outcome: "Reduce cost per query by 30-40% while maintaining quality".to_string(),
                    effort_estimate: EffortLevel::Medium,
                    roi_estimate: Some(2.5),
                });
            }
        }

        // Check for growth opportunities
        if let Some(growth_insight) = insights
            .iter()
            .find(|i| i.category == InsightCategory::Growth)
        {
            if growth_insight.impact == BusinessImpact::High {
                recommendations.push(ExecutiveRecommendation {
                    priority: Priority::High,
                    title: "Scale Infrastructure".to_string(),
                    description: "Usage patterns indicate strong growth. Proactively scale infrastructure to maintain performance.".to_string(),
                    expected_outcome: "Maintain sub-300ms response times during growth".to_string(),
                    effort_estimate: EffortLevel::Medium,
                    roi_estimate: Some(3.0),
                });
            }
        }

        // Add general optimization recommendation
        recommendations.push(ExecutiveRecommendation {
            priority: Priority::Medium,
            title: "Implement Advanced Caching".to_string(),
            description: "Deploy semantic caching for frequently asked questions to reduce API calls and improve response times.".to_string(),
            expected_outcome: "15-20% reduction in API calls and costs".to_string(),
            effort_estimate: EffortLevel::Low,
            roi_estimate: Some(4.0),
        });

        Ok(recommendations)
    }

    async fn create_visualizations(&self, data: &ExecutiveData) -> Result<Vec<Visualization>> {
        let mut visualizations = Vec::new();

        // Usage trend visualization
        let usage_data = self.create_usage_trend_data(data)?;
        visualizations.push(Visualization {
            title: "Usage Trend".to_string(),
            chart_type: ChartType::LineChart,
            data: usage_data,
            options: VisualizationOptions::default(),
        });

        // Cost breakdown visualization
        let cost_data = self.create_cost_breakdown_data(data)?;
        visualizations.push(Visualization {
            title: "Cost Distribution".to_string(),
            chart_type: ChartType::PieChart,
            data: cost_data,
            options: VisualizationOptions::default(),
        });

        // Performance gauge
        let perf_data = self.create_performance_gauge_data(data)?;
        visualizations.push(Visualization {
            title: "System Performance".to_string(),
            chart_type: ChartType::GaugeChart,
            data: perf_data,
            options: VisualizationOptions {
                width: 40,
                height: 10,
                ..Default::default()
            },
        });

        Ok(visualizations)
    }

    fn calculate_health_score(&self, metrics: &[KeyMetric]) -> Result<HealthScore> {
        let mut components = HashMap::new();
        let mut total_score = 0.0;
        let mut count = 0;

        // Performance health
        if let Some(response_time) = metrics.iter().find(|m| m.name == "Avg Response Time") {
            let perf_score = if response_time.value < 200.0 {
                100.0
            } else if response_time.value < 500.0 {
                80.0
            } else if response_time.value < 1000.0 {
                60.0
            } else {
                40.0
            };
            components.insert("performance".to_string(), perf_score);
            total_score += perf_score;
            count += 1;
        }

        // Cost efficiency health
        if let Some(cost_metric) = metrics.iter().find(|m| m.name == "Cost per Query") {
            let cost_score = if cost_metric.value < 0.05 {
                100.0
            } else if cost_metric.value < 0.10 {
                80.0
            } else if cost_metric.value < 0.20 {
                60.0
            } else {
                40.0
            };
            components.insert("cost_efficiency".to_string(), cost_score);
            total_score += cost_score;
            count += 1;
        }

        // Quality health
        if let Some(success_rate) = metrics.iter().find(|m| m.name == "Success Rate") {
            let quality_score = success_rate.value * 100.0;
            components.insert("quality".to_string(), quality_score);
            total_score += quality_score;
            count += 1;
        }

        // User satisfaction health
        if let Some(satisfaction) = metrics.iter().find(|m| m.name == "User Satisfaction") {
            let satisfaction_score = (satisfaction.value / 5.0) * 100.0;
            components.insert("user_satisfaction".to_string(), satisfaction_score);
            total_score += satisfaction_score;
            count += 1;
        }

        let overall = if count > 0 {
            total_score / count as f64
        } else {
            0.0
        };

        let trend = if overall > 80.0 {
            HealthTrend::Improving
        } else if overall > 60.0 {
            HealthTrend::Stable
        } else {
            HealthTrend::Degrading
        };

        Ok(HealthScore {
            overall,
            components,
            trend,
        })
    }

    fn create_usage_trend_data(&self, data: &ExecutiveData) -> Result<serde_json::Value> {
        // Create sample trend data
        let mut points = Vec::new();
        let now = Utc::now();

        for i in (0..30).rev() {
            let date = now - Duration::days(i);
            let value = 100.0 + (i as f64 * 2.0) + ((i as f64 * 0.5).sin() * 20.0);

            points.push(serde_json::json!({
                "date": date.format("%Y-%m-%d").to_string(),
                "value": value
            }));
        }

        Ok(serde_json::json!({
            "series": [{
                "name": "Daily Queries",
                "data": points
            }],
            "xAxis": {
                "type": "date"
            },
            "yAxis": {
                "label": "Queries"
            }
        }))
    }

    fn create_cost_breakdown_data(&self, data: &ExecutiveData) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "data": [
                {"name": "GPT-4", "value": 45.0},
                {"name": "Claude-3-Opus", "value": 30.0},
                {"name": "Claude-3-Sonnet", "value": 15.0},
                {"name": "Other Models", "value": 10.0}
            ],
            "total": 100.0
        }))
    }

    fn create_performance_gauge_data(&self, data: &ExecutiveData) -> Result<serde_json::Value> {
        let response_time = data.metrics.get("avg_response_time_ms").unwrap_or(&500.0);

        Ok(serde_json::json!({
            "value": response_time,
            "min": 0,
            "max": 1000,
            "thresholds": {
                "excellent": 200,
                "good": 500,
                "warning": 800,
                "critical": 1000
            },
            "unit": "ms"
        }))
    }

    fn format_markdown(&self, summary: ExecutiveSummary) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!(
            "# Executive Report - {}\n\n",
            summary.period.to_string()
        ));
        output.push_str(&format!(
            "*Generated: {}*\n\n",
            summary.generated_at.format("%Y-%m-%d %H:%M UTC")
        ));

        // Health Score
        output.push_str(&format!(
            "## Overall Health Score: {:.1}/100 ({:?})\n\n",
            summary.health_score.overall, summary.health_score.trend
        ));

        // Key Metrics
        output.push_str("## Key Performance Indicators\n\n");
        output.push_str("| Metric | Value | Change | Status | Trend |\n");
        output.push_str("|--------|-------|---------|---------|-------|\n");

        for metric in &summary.key_metrics {
            output.push_str(&format!(
                "| {} | {:.2} {} | {:+.1}% | {:?} | {:?} |\n",
                metric.name,
                metric.value,
                metric.unit,
                metric.change_percent,
                metric.status,
                metric.trend
            ));
        }
        output.push_str("\n");

        // Business Insights
        if !summary.business_insights.is_empty() {
            output.push_str("## Business Insights\n\n");
            for insight in &summary.business_insights {
                output.push_str(&format!(
                    "### {} - {}\n",
                    match insight.category {
                        InsightCategory::Growth => "📈 Growth",
                        InsightCategory::Efficiency => "⚡ Efficiency",
                        InsightCategory::Quality => "✨ Quality",
                        InsightCategory::Risk => "⚠️ Risk",
                        InsightCategory::Opportunity => "💡 Opportunity",
                    },
                    insight.title
                ));
                output.push_str(&format!("{}\n\n", insight.summary));
                output.push_str(&format!(
                    "**Impact**: {:?} | **Confidence**: {:.0}%\n\n",
                    insight.impact,
                    insight.confidence * 100.0
                ));
            }
        }

        // Recommendations
        if !summary.recommendations.is_empty() {
            output.push_str("## Strategic Recommendations\n\n");
            for (i, rec) in summary.recommendations.iter().enumerate() {
                output.push_str(&format!(
                    "### {}. {} (Priority: {:?})\n",
                    i + 1,
                    rec.title,
                    rec.priority
                ));
                output.push_str(&format!("{}\n\n", rec.description));
                output.push_str(&format!("**Expected Outcome**: {}\n", rec.expected_outcome));
                output.push_str(&format!("**Effort**: {:?}", rec.effort_estimate));
                if let Some(roi) = rec.roi_estimate {
                    output.push_str(&format!(" | **ROI**: {:.1}x", roi));
                }
                output.push_str("\n\n");
            }
        }

        // Visualizations (ASCII art)
        output.push_str("## Performance Visualizations\n\n");
        for viz in &summary.visualizations {
            output.push_str(&format!("### {}\n\n", viz.title));
            if let Ok(ascii_chart) = self.visualizer.render_ascii(&viz) {
                output.push_str(&format!("```\n{}\n```\n\n", ascii_chart));
            }
        }

        Ok(output)
    }

    fn format_html(&self, summary: ExecutiveSummary) -> Result<String> {
        // Simple HTML formatting
        let mut html = String::from(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Executive Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1, h2, h3 { color: #333; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .metric-good { color: green; }
        .metric-warning { color: orange; }
        .metric-critical { color: red; }
    </style>
</head>
<body>
"#,
        );

        html.push_str(&format!(
            "<h1>Executive Report - {}</h1>\n",
            summary.period.to_string()
        ));
        html.push_str(&format!(
            "<p><em>Generated: {}</em></p>\n",
            summary.generated_at.format("%Y-%m-%d %H:%M UTC")
        ));

        // Convert markdown content to HTML (simplified)
        if let Ok(markdown_content) = self.format_markdown(summary) {
            let html_content = markdown_content
                .replace("# ", "<h1>")
                .replace("## ", "<h2>")
                .replace("### ", "<h3>")
                .replace("\n\n", "</p><p>")
                .replace("**", "<strong>")
                .replace("*", "<em>");

            html.push_str(&html_content);
        }

        html.push_str("</body></html>");
        Ok(html)
    }

    fn format_json(&self, summary: ExecutiveSummary) -> Result<String> {
        serde_json::to_string_pretty(&summary)
            .context("Failed to serialize executive summary to JSON")
    }
}

impl ExecutiveSummary {
    /// Format as markdown
    pub fn format_markdown(&self) -> Result<String> {
        // Delegate to a simple markdown formatter
        let mut output = String::new();

        output.push_str(&format!("Period: {}\n", self.period.to_string()));
        output.push_str(&format!("Generated: {}\n\n", self.generated_at));

        output.push_str("### Key Metrics\n");
        for metric in &self.key_metrics {
            output.push_str(&format!(
                "- {}: {} {} ({:+.1}%)\n",
                metric.name, metric.value, metric.unit, metric.change_percent
            ));
        }

        Ok(output)
    }
}

impl KpiCalculator {
    fn new() -> Self {
        let mut kpi_definitions = HashMap::new();

        // Define standard KPIs
        Self::add_kpi(
            &mut kpi_definitions,
            "Total Queries",
            "period_queries",
            "queries",
            None,
            Direction::Higher,
        );
        Self::add_kpi(
            &mut kpi_definitions,
            "Avg Response Time",
            "avg_response_time_ms",
            "ms",
            Some(300.0),
            Direction::Lower,
        );
        Self::add_kpi(
            &mut kpi_definitions,
            "Success Rate",
            "success_rate",
            "%",
            Some(0.95),
            Direction::Higher,
        );
        Self::add_kpi(
            &mut kpi_definitions,
            "Cost per Query",
            "cost_per_query",
            "$",
            Some(0.10),
            Direction::Lower,
        );
        Self::add_kpi(
            &mut kpi_definitions,
            "User Satisfaction",
            "user_satisfaction",
            "/5",
            Some(4.5),
            Direction::Higher,
        );

        Self { kpi_definitions }
    }

    fn add_kpi(
        definitions: &mut HashMap<String, KpiDefinition>,
        name: &str,
        metric_key: &str,
        unit: &str,
        target: Option<f64>,
        direction: Direction,
    ) {
        let metric_key_owned = metric_key.to_string();
        definitions.insert(
            name.to_string(),
            KpiDefinition {
                name: name.to_string(),
                calculation: Box::new(move |metrics| {
                    metrics.get(&metric_key_owned).copied().unwrap_or(0.0)
                }),
                unit: unit.to_string(),
                target,
                good_direction: direction,
            },
        );
    }

    fn calculate_kpis(&self, data: &ExecutiveData) -> Result<Vec<KeyMetric>> {
        let mut kpis = Vec::new();

        for (name, definition) in &self.kpi_definitions {
            let value = (definition.calculation)(&data.metrics);

            // Calculate change percent (simplified - would use historical data)
            let change_percent = ((value * 0.1) - 5.0).max(-50.0).min(50.0);

            // Determine status
            let status = if let Some(target) = definition.target {
                match definition.good_direction {
                    Direction::Higher => {
                        if value >= target * 1.1 {
                            MetricStatus::Excellent
                        } else if value >= target {
                            MetricStatus::Good
                        } else if value >= target * 0.8 {
                            MetricStatus::Warning
                        } else {
                            MetricStatus::Critical
                        }
                    }
                    Direction::Lower => {
                        if value <= target * 0.9 {
                            MetricStatus::Excellent
                        } else if value <= target {
                            MetricStatus::Good
                        } else if value <= target * 1.2 {
                            MetricStatus::Warning
                        } else {
                            MetricStatus::Critical
                        }
                    }
                }
            } else {
                MetricStatus::Good
            };

            // Determine trend
            let trend = if change_percent > 5.0 {
                MetricTrend::Improving
            } else if change_percent < -5.0 {
                MetricTrend::Declining
            } else {
                MetricTrend::Stable
            };

            kpis.push(KeyMetric {
                name: name.clone(),
                value,
                unit: definition.unit.clone(),
                change_percent,
                target: definition.target,
                status,
                trend,
            });
        }

        Ok(kpis)
    }
}

impl InsightGenerator {
    fn new() -> Self {
        let mut insight_rules = Vec::new();

        // Growth insight rule
        insight_rules.push(InsightRule {
            name: "Rapid Growth Detection".to_string(),
            category: InsightCategory::Growth,
            condition: Box::new(|data| {
                data.metrics.get("period_queries").unwrap_or(&0.0) > &1000.0
            }),
            generator: Box::new(|data| {
                let queries = data.metrics.get("period_queries").unwrap_or(&0.0);
                BusinessInsight {
                    category: InsightCategory::Growth,
                    title: "Strong Usage Growth Detected".to_string(),
                    summary: format!("Query volume has reached {} queries this period, indicating strong platform adoption.", queries),
                    impact: BusinessImpact::High,
                    supporting_data: {
                        let mut data = HashMap::new();
                        data.insert("queries".to_string(), *queries);
                        data
                    },
                    confidence: 0.9,
                }
            }),
        });

        // Efficiency insight rule
        insight_rules.push(InsightRule {
            name: "Cost Efficiency".to_string(),
            category: InsightCategory::Efficiency,
            condition: Box::new(|data| {
                let cost_per_query = data.metrics.get("period_cost").unwrap_or(&0.0)
                    / data.metrics.get("period_queries").unwrap_or(&1.0);
                cost_per_query < 0.05
            }),
            generator: Box::new(|data| {
                let cost = data.metrics.get("period_cost").unwrap_or(&0.0);
                let queries = data.metrics.get("period_queries").unwrap_or(&1.0);
                let cost_per_query = cost / queries;

                BusinessInsight {
                    category: InsightCategory::Efficiency,
                    title: "Excellent Cost Efficiency".to_string(),
                    summary: format!(
                        "Cost per query is ${:.3}, well below industry average",
                        cost_per_query
                    ),
                    impact: BusinessImpact::Medium,
                    supporting_data: {
                        let mut data = HashMap::new();
                        data.insert("cost_per_query".to_string(), cost_per_query);
                        data.insert("total_cost".to_string(), *cost);
                        data
                    },
                    confidence: 0.85,
                }
            }),
        });

        Self { insight_rules }
    }

    fn generate_insights(&self, data: &ExecutiveData) -> Result<Vec<BusinessInsight>> {
        let mut insights = Vec::new();

        for rule in &self.insight_rules {
            if (rule.condition)(data) {
                insights.push((rule.generator)(data));
            }
        }

        Ok(insights)
    }
}

impl Visualizer {
    fn new() -> Self {
        Self {
            ascii_renderer: AsciiRenderer,
        }
    }

    fn render_ascii(&self, visualization: &Visualization) -> Result<String> {
        self.ascii_renderer.render(visualization)
    }
}

impl AsciiRenderer {
    fn render(&self, visualization: &Visualization) -> Result<String> {
        match visualization.chart_type {
            ChartType::LineChart => self.render_line_chart(visualization),
            ChartType::BarChart => self.render_bar_chart(visualization),
            ChartType::PieChart => self.render_pie_chart(visualization),
            ChartType::GaugeChart => self.render_gauge_chart(visualization),
            ChartType::Sparkline => self.render_sparkline(visualization),
            _ => Ok("Chart type not yet implemented".to_string()),
        }
    }

    fn render_line_chart(&self, viz: &Visualization) -> Result<String> {
        // Simple ASCII line chart
        let mut output = String::new();
        let width = viz.options.width as usize;
        let height = viz.options.height as usize;

        // Create a simple line chart representation
        output.push_str("     ^\n");
        output.push_str("100 |     .--.\n");
        output.push_str(" 80 |   ./    \\.-.\n");
        output.push_str(" 60 | ./          \\.\n");
        output.push_str(" 40 |/              \\\n");
        output.push_str(" 20 |                \\___\n");
        output.push_str("  0 +------------------------>\n");
        output.push_str("    0    7   14   21   28  Days\n");

        Ok(output)
    }

    fn render_bar_chart(&self, viz: &Visualization) -> Result<String> {
        let mut output = String::new();

        output.push_str("Model Usage Distribution\n");
        output.push_str("GPT-4       |████████████████████| 45%\n");
        output.push_str("Claude-3-O  |████████████        | 30%\n");
        output.push_str("Claude-3-S  |███████             | 15%\n");
        output.push_str("Others      |████                | 10%\n");

        Ok(output)
    }

    fn render_pie_chart(&self, viz: &Visualization) -> Result<String> {
        // Simple ASCII pie chart representation
        let mut output = String::new();

        output.push_str("     .-\"\"-.\n");
        output.push_str("   /   45% \\\n");
        output.push_str("  |  GPT-4  |\n");
        output.push_str(" /|         |\\\n");
        output.push_str("| | Claude  | |\n");
        output.push_str(" \\|   30%   |/\n");
        output.push_str("  |  Other  |\n");
        output.push_str("   \\  25%  /\n");
        output.push_str("     '----'\n");

        Ok(output)
    }

    fn render_gauge_chart(&self, viz: &Visualization) -> Result<String> {
        let mut output = String::new();

        // Extract gauge value
        let value = viz
            .data
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let max = viz
            .data
            .get("max")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);

        let percentage = (value / max * 100.0).min(100.0);
        let filled = (percentage / 5.0) as usize;

        output.push_str("Performance Gauge\n");
        output.push_str(&format!(
            "[{}{}] {:.0}ms\n",
            "█".repeat(filled),
            "░".repeat(20 - filled),
            value
        ));

        Ok(output)
    }

    fn render_sparkline(&self, viz: &Visualization) -> Result<String> {
        // Simple sparkline
        Ok("▁▂▃▅▇█▇▅▃▃▅▇█▇▅▃▁".to_string())
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executive_reporter_creation() -> Result<()> {
        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let reporter = ExecutiveReporter::new(config).await?;

        assert!(Arc::strong_count(&reporter.kpi_calculator) > 0);

        Ok(())
    }

    #[test]
    fn test_report_period_parsing() {
        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let reporter = ExecutiveReporter {
            config,
            kpi_calculator: Arc::new(KpiCalculator::new()),
            insight_generator: Arc::new(InsightGenerator::new()),
            visualizer: Arc::new(Visualizer::new()),
        };

        assert!(matches!(
            reporter.parse_period("weekly").unwrap(),
            ReportPeriod::Weekly
        ));
        assert!(matches!(
            reporter.parse_period("30").unwrap(),
            ReportPeriod::Custom { days: 30 }
        ));
    }

    #[test]
    fn test_health_score_calculation() {
        let metrics = vec![
            KeyMetric {
                name: "Avg Response Time".to_string(),
                value: 150.0,
                unit: "ms".to_string(),
                change_percent: -10.0,
                target: Some(200.0),
                status: MetricStatus::Excellent,
                trend: MetricTrend::Improving,
            },
            KeyMetric {
                name: "Success Rate".to_string(),
                value: 0.98,
                unit: "%".to_string(),
                change_percent: 2.0,
                target: Some(0.95),
                status: MetricStatus::Excellent,
                trend: MetricTrend::Stable,
            },
        ];

        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let reporter = ExecutiveReporter {
            config,
            kpi_calculator: Arc::new(KpiCalculator::new()),
            insight_generator: Arc::new(InsightGenerator::new()),
            visualizer: Arc::new(Visualizer::new()),
        };

        let health_score = reporter.calculate_health_score(&metrics).unwrap();
        assert!(health_score.overall > 80.0);
        assert!(matches!(
            health_score.trend,
            HealthTrend::Improving | HealthTrend::Stable
        ));
    }
}
