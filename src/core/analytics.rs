//! Analytics engine for comprehensive metrics and insights
//!
//! This module provides:
//! - Real-time metrics collection and aggregation
//! - Performance dashboards with visualizations
//! - Cost analysis and optimization recommendations
//! - Quality trends and predictions
//! - Business intelligence reports

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::database::{get_database, ActivityLog};

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Retention period for detailed metrics (days)
    pub detailed_retention_days: u32,
    /// Retention period for aggregated metrics (days)
    pub aggregated_retention_days: u32,
    /// Real-time update interval (seconds)
    pub update_interval_secs: u64,
    /// Enable predictive analytics
    pub enable_predictions: bool,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            detailed_retention_days: 30,
            aggregated_retention_days: 365,
            update_interval_secs: 60,
            enable_predictions: true,
            enable_anomaly_detection: true,
        }
    }
}

/// Main analytics engine
#[derive(Debug)]
pub struct AnalyticsEngine {
    config: AnalyticsConfig,
    metrics_store: Arc<RwLock<MetricsStore>>,
    cost_analyzer: Arc<CostAnalyzer>,
    trend_analyzer: Arc<TrendAnalyzer>,
    dashboard_builder: Arc<DashboardBuilder>,
    report_generator: Arc<ReportGenerator>,
}

/// Stores and manages metrics data
#[derive(Debug, Default)]
struct MetricsStore {
    /// Time-series metrics data
    time_series: HashMap<String, TimeSeries>,
    /// Aggregated metrics by period
    aggregations: HashMap<AggregationPeriod, HashMap<String, f64>>,
    /// Real-time metrics
    real_time: HashMap<String, MetricValue>,
    /// Event counters
    counters: HashMap<String, u64>,
}

/// Time series data for a metric
#[derive(Debug)]
struct TimeSeries {
    name: String,
    data_points: VecDeque<DataPoint>,
    metric_type: MetricType,
    unit: String,
}

/// A single data point in a time series
#[derive(Debug, Clone)]
struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f64,
    tags: HashMap<String, String>,
}

/// Types of metrics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Rate,
}

/// Metric value with metadata
#[derive(Debug, Clone)]
struct MetricValue {
    value: f64,
    timestamp: DateTime<Utc>,
    unit: String,
}

/// Aggregation periods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggregationPeriod {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

/// Cost analysis engine
#[derive(Debug)]
struct CostAnalyzer {
    /// Model pricing information
    model_costs: HashMap<String, ModelPricing>,
    /// Cost optimization rules
    optimization_rules: Vec<OptimizationRule>,
}

/// Model pricing information
#[derive(Debug, Clone)]
struct ModelPricing {
    provider: String,
    input_cost_per_1k: f64,
    output_cost_per_1k: f64,
    minimum_cost: f64,
}

/// Cost optimization rule
struct OptimizationRule {
    name: String,
    condition: Box<dyn Fn(&CostMetrics) -> bool + Send + Sync>,
    recommendation: String,
    potential_savings: f64,
}

impl std::fmt::Debug for OptimizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptimizationRule")
            .field("name", &self.name)
            .field("recommendation", &self.recommendation)
            .field("potential_savings", &self.potential_savings)
            .field("condition", &"<function>")
            .finish()
    }
}

/// Cost metrics for analysis
#[derive(Debug, Default)]
pub struct CostMetrics {
    pub total_cost: f64,
    pub cost_by_model: HashMap<String, f64>,
    pub cost_by_user: HashMap<String, f64>,
    pub cost_by_period: HashMap<String, f64>,
    pub average_cost_per_query: f64,
}

/// Trend analysis engine
#[derive(Debug)]
struct TrendAnalyzer {
    /// Moving averages calculator
    ma_calculator: MovingAverageCalculator,
    /// Anomaly detector
    anomaly_detector: AnomalyDetector,
    /// Trend predictor
    predictor: TrendPredictor,
}

/// Moving average calculator
#[derive(Debug)]
struct MovingAverageCalculator {
    window_sizes: Vec<usize>,
}

/// Anomaly detection engine
#[derive(Debug)]
struct AnomalyDetector {
    threshold_multiplier: f64,
    min_data_points: usize,
}

/// Trend prediction engine
#[derive(Debug)]
struct TrendPredictor {
    /// Prediction horizon (days)
    horizon_days: u32,
}

/// Dashboard builder for terminal UI
#[derive(Debug)]
struct DashboardBuilder {
    /// Dashboard templates
    templates: HashMap<String, DashboardTemplate>,
}

/// Dashboard template
#[derive(Debug)]
struct DashboardTemplate {
    name: String,
    layout: DashboardLayout,
    widgets: Vec<Widget>,
}

/// Dashboard layout configuration
#[derive(Debug)]
enum DashboardLayout {
    Grid { rows: usize, cols: usize },
    Vertical,
    Horizontal,
}

/// Dashboard widget
#[derive(Debug)]
struct Widget {
    widget_type: WidgetType,
    title: String,
    position: (usize, usize),
    size: (usize, usize),
}

/// Types of dashboard widgets
#[derive(Debug)]
enum WidgetType {
    LineChart,
    BarChart,
    Gauge,
    Table,
    Sparkline,
    HeatMap,
}

/// Report generator for executive reports
#[derive(Debug)]
struct ReportGenerator {
    /// Report templates
    templates: HashMap<ReportType, ReportTemplate>,
}

/// Types of reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReportType {
    Executive,
    Operational,
    Financial,
    Performance,
    Custom,
}

/// Report template
#[derive(Debug)]
struct ReportTemplate {
    name: String,
    sections: Vec<ReportSection>,
    format: ReportFormat,
}

/// Report section
#[derive(Debug)]
struct ReportSection {
    title: String,
    content_type: ReportContentType,
    data_source: String,
}

/// Types of report content
#[derive(Debug)]
enum ReportContentType {
    Summary,
    Chart,
    Table,
    Analysis,
    Recommendations,
}

/// Report output format
#[derive(Debug)]
enum ReportFormat {
    Html,
    Markdown,
    Json,
    Pdf,
}

/// Analytics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub period: String,
    pub metrics: HashMap<String, f64>,
    pub trends: Vec<TrendInfo>,
    pub insights: Vec<AnalyticsInsight>,
}

/// Trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendInfo {
    pub metric: String,
    pub direction: TrendDirection,
    pub change_percent: f64,
    pub prediction: Option<f64>,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

/// Analytics insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsInsight {
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub impact: ImpactLevel,
    pub recommendations: Vec<String>,
}

/// Insight categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightCategory {
    Cost,
    Performance,
    Quality,
    Usage,
    Anomaly,
}

/// Impact levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Dashboard data for real-time display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub widgets: Vec<DashboardWidget>,
}

/// Dashboard widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub title: String,
    pub widget_type: String,
    pub data: serde_json::Value,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub async fn new(config: AnalyticsConfig) -> Result<Self> {
        info!("Initializing analytics engine");

        let metrics_store = Arc::new(RwLock::new(MetricsStore::default()));
        let cost_analyzer = Arc::new(CostAnalyzer::new());
        let trend_analyzer = Arc::new(TrendAnalyzer::new());
        let dashboard_builder = Arc::new(DashboardBuilder::new());
        let report_generator = Arc::new(ReportGenerator::new());

        let engine = Self {
            config,
            metrics_store,
            cost_analyzer,
            trend_analyzer,
            dashboard_builder,
            report_generator,
        };

        // Start background metrics collection
        engine.start_metrics_collection().await?;

        info!("Analytics engine initialized successfully");
        Ok(engine)
    }

    /// Collect and store metrics
    pub async fn collect_metrics(&self) -> Result<()> {
        debug!("Collecting analytics metrics");

        // Get database statistics
        let db = get_database().await?;
        let stats = db.get_statistics().await?;

        // Get recent activity
        let activities = ActivityLog::get_recent(100).await?;

        // Update metrics store
        let mut store = self.metrics_store.write().await;

        // Record database metrics
        self.record_metric(
            &mut store,
            "conversations.total",
            stats.conversation_count as f64,
            "count",
        )?;
        self.record_metric(
            &mut store,
            "messages.total",
            stats.message_count as f64,
            "count",
        )?;
        self.record_metric(&mut store, "users.total", stats.user_count as f64, "count")?;
        self.record_metric(
            &mut store,
            "models.available",
            stats.model_count as f64,
            "count",
        )?;

        // Calculate activity metrics
        let mut event_counts: HashMap<String, u64> = HashMap::new();
        let mut total_cost = 0.0;
        let mut total_duration = 0;

        for activity in &activities {
            *event_counts.entry(activity.event_type.clone()).or_insert(0) += 1;
            if let Some(cost) = activity.cost {
                total_cost += cost;
            }
            if let Some(duration) = activity.duration_ms {
                total_duration += duration;
            }
        }

        // Record activity metrics
        for (event_type, count) in event_counts {
            self.record_metric(
                &mut store,
                &format!("events.{}", event_type),
                count as f64,
                "count",
            )?;
        }

        self.record_metric(&mut store, "cost.total", total_cost, "usd")?;
        if !activities.is_empty() {
            self.record_metric(
                &mut store,
                "latency.average",
                (total_duration as f64) / (activities.len() as f64),
                "ms",
            )?;
        }

        info!("Metrics collection completed");
        Ok(())
    }

    /// Generate analytics report
    pub async fn generate_report(
        &self,
        report_type: ReportType,
        period: AggregationPeriod,
    ) -> Result<String> {
        info!("Generating {:?} report for {:?}", report_type, period);

        let store = self.metrics_store.read().await;
        let mut report = String::new();

        // Add header
        report.push_str(&format!(
            "# {} Report\n\n",
            match report_type {
                ReportType::Executive => "Executive Analytics",
                ReportType::Operational => "Operational Metrics",
                ReportType::Financial => "Financial Analysis",
                ReportType::Performance => "Performance Report",
                ReportType::Custom => "Custom Analytics",
            }
        ));

        report.push_str(&format!("**Period**: {:?}\n", period));
        report.push_str(&format!(
            "**Generated**: {}\n\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Add summary section
        report.push_str("## Summary\n\n");
        if let Some(aggregations) = store.aggregations.get(&period) {
            for (metric, value) in aggregations {
                report.push_str(&format!("- **{}**: {:.2}\n", metric, value));
            }
        }
        report.push_str("\n");

        // Add trends section
        report.push_str("## Trends\n\n");
        let trends = self.analyze_trends(&store, period).await?;
        for trend in &trends {
            report.push_str(&format!(
                "- **{}**: {:?} ({:+.1}%)\n",
                trend.metric, trend.direction, trend.change_percent
            ));
        }
        report.push_str("\n");

        // Add insights section
        report.push_str("## Insights\n\n");
        let insights = self.generate_insights(&store).await?;
        for insight in &insights {
            report.push_str(&format!("### {}\n", insight.title));
            report.push_str(&format!("{}\n\n", insight.description));
            if !insight.recommendations.is_empty() {
                report.push_str("**Recommendations**:\n");
                for rec in &insight.recommendations {
                    report.push_str(&format!("- {}\n", rec));
                }
                report.push_str("\n");
            }
        }

        info!("Report generation completed");
        Ok(report)
    }

    /// Generate real-time dashboard data
    pub async fn generate_dashboard(&self) -> Result<DashboardData> {
        debug!("Generating dashboard data");

        let store = self.metrics_store.read().await;
        let mut widgets = Vec::new();

        // Query volume widget
        if let Some(series) = store.time_series.get("events.query_start") {
            let chart_data = self.create_line_chart_data(series, 24)?;
            widgets.push(DashboardWidget {
                title: "Query Volume (24h)".to_string(),
                widget_type: "line_chart".to_string(),
                data: serde_json::to_value(chart_data)?,
            });
        }

        // Cost breakdown widget
        let cost_data = self.create_cost_breakdown(&store)?;
        widgets.push(DashboardWidget {
            title: "Cost Breakdown".to_string(),
            widget_type: "pie_chart".to_string(),
            data: serde_json::to_value(cost_data)?,
        });

        // Performance metrics widget
        if let Some(latency) = store.real_time.get("latency.average") {
            widgets.push(DashboardWidget {
                title: "Average Latency".to_string(),
                widget_type: "gauge".to_string(),
                data: serde_json::json!({
                    "value": latency.value,
                    "unit": latency.unit,
                    "min": 0,
                    "max": 1000,
                    "thresholds": {
                        "good": 200,
                        "warning": 500,
                        "critical": 1000
                    }
                }),
            });
        }

        // System health widget
        let health_data = self.create_health_status(&store)?;
        widgets.push(DashboardWidget {
            title: "System Health".to_string(),
            widget_type: "status_grid".to_string(),
            data: serde_json::to_value(health_data)?,
        });

        Ok(DashboardData {
            title: "Hive AI Analytics Dashboard".to_string(),
            updated_at: Utc::now(),
            widgets,
        })
    }

    /// Analyze costs and provide optimization recommendations
    pub async fn analyze_costs(&self) -> Result<CostMetrics> {
        info!("Analyzing costs");

        let db = get_database().await?;
        let activities = ActivityLog::get_recent(1000).await?;

        let mut metrics = CostMetrics::default();
        let mut query_count = 0;

        // Calculate costs by model and user
        for activity in &activities {
            if let Some(cost) = activity.cost {
                metrics.total_cost += cost;

                if let Some(model) = &activity.model_used {
                    *metrics.cost_by_model.entry(model.clone()).or_insert(0.0) += cost;
                }

                if let Some(user_id) = &activity.user_id {
                    *metrics.cost_by_user.entry(user_id.clone()).or_insert(0.0) += cost;
                }

                // Group by day
                let day = activity.created_at[..10].to_string();
                *metrics.cost_by_period.entry(day).or_insert(0.0) += cost;
            }

            if activity.event_type == "query_complete" {
                query_count += 1;
            }
        }

        if query_count > 0 {
            metrics.average_cost_per_query = metrics.total_cost / query_count as f64;
        }

        info!("Cost analysis completed: total=${:.2}", metrics.total_cost);
        Ok(metrics)
    }

    /// Get cost optimization recommendations
    pub async fn get_cost_optimizations(&self) -> Result<Vec<String>> {
        let metrics = self.analyze_costs().await?;
        let mut recommendations = Vec::new();

        // Check for expensive models
        for (model, cost) in &metrics.cost_by_model {
            if cost / metrics.total_cost > 0.5 {
                recommendations.push(format!(
                    "Model '{}' accounts for {:.0}% of costs. Consider using a more cost-effective model for non-critical queries.",
                    model,
                    (cost / metrics.total_cost) * 100.0
                ));
            }
        }

        // Check for high-volume users
        for (user, cost) in &metrics.cost_by_user {
            if cost / metrics.total_cost > 0.3 {
                recommendations.push(format!(
                    "User '{}' accounts for {:.0}% of costs. Consider implementing rate limiting or usage quotas.",
                    user,
                    (cost / metrics.total_cost) * 100.0
                ));
            }
        }

        // Check average cost per query
        if metrics.average_cost_per_query > 0.10 {
            recommendations.push(format!(
                "Average cost per query (${:.3}) is high. Consider caching frequent queries or using smaller models for simple tasks.",
                metrics.average_cost_per_query
            ));
        }

        Ok(recommendations)
    }

    // Private helper methods

    async fn start_metrics_collection(&self) -> Result<()> {
        let engine = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                engine.config.update_interval_secs,
            ));

            loop {
                interval.tick().await;
                if let Err(e) = engine.collect_metrics().await {
                    warn!("Failed to collect metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    fn record_metric(
        &self,
        store: &mut MetricsStore,
        name: &str,
        value: f64,
        unit: &str,
    ) -> Result<()> {
        // Update real-time metric
        store.real_time.insert(
            name.to_string(),
            MetricValue {
                value,
                timestamp: Utc::now(),
                unit: unit.to_string(),
            },
        );

        // Update time series
        let series = store
            .time_series
            .entry(name.to_string())
            .or_insert_with(|| TimeSeries {
                name: name.to_string(),
                data_points: VecDeque::new(),
                metric_type: MetricType::Gauge,
                unit: unit.to_string(),
            });

        series.data_points.push_back(DataPoint {
            timestamp: Utc::now(),
            value,
            tags: HashMap::new(),
        });

        // Limit time series size
        while series.data_points.len() > 1000 {
            series.data_points.pop_front();
        }

        Ok(())
    }

    async fn analyze_trends(
        &self,
        store: &MetricsStore,
        period: AggregationPeriod,
    ) -> Result<Vec<TrendInfo>> {
        let mut trends = Vec::new();

        for (metric_name, series) in &store.time_series {
            if series.data_points.len() < 2 {
                continue;
            }

            // Calculate trend
            let recent = series.data_points.back().map(|p| p.value).unwrap_or(0.0);
            let previous = series
                .data_points
                .iter()
                .rev()
                .nth(10)
                .map(|p| p.value)
                .unwrap_or(recent);

            let change_percent = if previous != 0.0 {
                ((recent - previous) / previous) * 100.0
            } else {
                0.0
            };

            let direction = if change_percent > 5.0 {
                TrendDirection::Up
            } else if change_percent < -5.0 {
                TrendDirection::Down
            } else {
                TrendDirection::Stable
            };

            trends.push(TrendInfo {
                metric: metric_name.clone(),
                direction,
                change_percent,
                prediction: None, // TODO: Implement prediction
            });
        }

        Ok(trends)
    }

    async fn generate_insights(&self, store: &MetricsStore) -> Result<Vec<AnalyticsInsight>> {
        let mut insights = Vec::new();

        // Check for high costs
        if let Some(cost) = store.real_time.get("cost.total") {
            if cost.value > 100.0 {
                insights.push(AnalyticsInsight {
                    category: InsightCategory::Cost,
                    title: "High Usage Costs Detected".to_string(),
                    description: format!("Total costs have reached ${:.2} this period", cost.value),
                    impact: ImpactLevel::High,
                    recommendations: vec![
                        "Review model selection for cost efficiency".to_string(),
                        "Implement caching for frequent queries".to_string(),
                        "Consider usage limits or quotas".to_string(),
                    ],
                });
            }
        }

        // Check for performance issues
        if let Some(latency) = store.real_time.get("latency.average") {
            if latency.value > 500.0 {
                insights.push(AnalyticsInsight {
                    category: InsightCategory::Performance,
                    title: "High Latency Detected".to_string(),
                    description: format!("Average latency is {:.0}ms", latency.value),
                    impact: ImpactLevel::Medium,
                    recommendations: vec![
                        "Check model server health".to_string(),
                        "Consider using faster models for time-sensitive queries".to_string(),
                        "Review network connectivity".to_string(),
                    ],
                });
            }
        }

        Ok(insights)
    }

    fn create_line_chart_data(
        &self,
        series: &TimeSeries,
        hours: usize,
    ) -> Result<serde_json::Value> {
        let now = Utc::now();
        let cutoff = now - Duration::hours(hours as i64);

        let points: Vec<_> = series
            .data_points
            .iter()
            .filter(|p| p.timestamp > cutoff)
            .map(|p| {
                serde_json::json!({
                    "x": p.timestamp.timestamp(),
                    "y": p.value
                })
            })
            .collect();

        Ok(serde_json::json!({
            "series": [{
                "name": series.name,
                "data": points
            }],
            "unit": series.unit
        }))
    }

    fn create_cost_breakdown(&self, store: &MetricsStore) -> Result<serde_json::Value> {
        let mut breakdown = Vec::new();
        let mut total = 0.0;

        // Group costs by model
        for (metric, value) in &store.real_time {
            if metric.starts_with("cost.model.") {
                let model = metric.replace("cost.model.", "");
                breakdown.push(serde_json::json!({
                    "name": model,
                    "value": value.value
                }));
                total += value.value;
            }
        }

        Ok(serde_json::json!({
            "data": breakdown,
            "total": total
        }))
    }

    fn create_health_status(&self, store: &MetricsStore) -> Result<serde_json::Value> {
        let mut components = Vec::new();

        // Database health
        components.push(serde_json::json!({
            "name": "Database",
            "status": "healthy",
            "metric": store.real_time.get("database.connections")
                .map(|m| m.value)
                .unwrap_or(0.0)
        }));

        // API health
        if let Some(latency) = store.real_time.get("latency.average") {
            let status = if latency.value < 200.0 {
                "healthy"
            } else if latency.value < 500.0 {
                "warning"
            } else {
                "critical"
            };

            components.push(serde_json::json!({
                "name": "API Performance",
                "status": status,
                "metric": latency.value
            }));
        }

        Ok(serde_json::json!({ "components": components }))
    }
}

// Implement Clone for AnalyticsEngine (needed for Arc)
impl Clone for AnalyticsEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics_store: Arc::clone(&self.metrics_store),
            cost_analyzer: Arc::clone(&self.cost_analyzer),
            trend_analyzer: Arc::clone(&self.trend_analyzer),
            dashboard_builder: Arc::clone(&self.dashboard_builder),
            report_generator: Arc::clone(&self.report_generator),
        }
    }
}

impl CostAnalyzer {
    fn new() -> Self {
        let mut model_costs = HashMap::new();

        // Add common model pricing (example values)
        model_costs.insert(
            "gpt-4".to_string(),
            ModelPricing {
                provider: "openai".to_string(),
                input_cost_per_1k: 0.03,
                output_cost_per_1k: 0.06,
                minimum_cost: 0.0,
            },
        );

        model_costs.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                provider: "anthropic".to_string(),
                input_cost_per_1k: 0.015,
                output_cost_per_1k: 0.075,
                minimum_cost: 0.0,
            },
        );

        model_costs.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                provider: "anthropic".to_string(),
                input_cost_per_1k: 0.003,
                output_cost_per_1k: 0.015,
                minimum_cost: 0.0,
            },
        );

        Self {
            model_costs,
            optimization_rules: Vec::new(),
        }
    }
}

impl TrendAnalyzer {
    fn new() -> Self {
        Self {
            ma_calculator: MovingAverageCalculator {
                window_sizes: vec![5, 10, 20],
            },
            anomaly_detector: AnomalyDetector {
                threshold_multiplier: 2.5,
                min_data_points: 20,
            },
            predictor: TrendPredictor { horizon_days: 7 },
        }
    }
}

impl DashboardBuilder {
    fn new() -> Self {
        let mut templates = HashMap::new();

        // Create default dashboard template
        templates.insert(
            "default".to_string(),
            DashboardTemplate {
                name: "Default Dashboard".to_string(),
                layout: DashboardLayout::Grid { rows: 3, cols: 2 },
                widgets: vec![
                    Widget {
                        widget_type: WidgetType::LineChart,
                        title: "Query Volume".to_string(),
                        position: (0, 0),
                        size: (1, 2),
                    },
                    Widget {
                        widget_type: WidgetType::Gauge,
                        title: "Average Latency".to_string(),
                        position: (1, 0),
                        size: (1, 1),
                    },
                    Widget {
                        widget_type: WidgetType::BarChart,
                        title: "Cost by Model".to_string(),
                        position: (1, 1),
                        size: (1, 1),
                    },
                    Widget {
                        widget_type: WidgetType::Table,
                        title: "Recent Activity".to_string(),
                        position: (2, 0),
                        size: (1, 2),
                    },
                ],
            },
        );

        Self { templates }
    }
}

impl ReportGenerator {
    fn new() -> Self {
        let mut templates = HashMap::new();

        // Executive report template
        templates.insert(
            ReportType::Executive,
            ReportTemplate {
                name: "Executive Report".to_string(),
                sections: vec![
                    ReportSection {
                        title: "Executive Summary".to_string(),
                        content_type: ReportContentType::Summary,
                        data_source: "aggregated_metrics".to_string(),
                    },
                    ReportSection {
                        title: "Key Performance Indicators".to_string(),
                        content_type: ReportContentType::Chart,
                        data_source: "kpi_metrics".to_string(),
                    },
                    ReportSection {
                        title: "Cost Analysis".to_string(),
                        content_type: ReportContentType::Analysis,
                        data_source: "cost_metrics".to_string(),
                    },
                    ReportSection {
                        title: "Strategic Recommendations".to_string(),
                        content_type: ReportContentType::Recommendations,
                        data_source: "insights".to_string(),
                    },
                ],
                format: ReportFormat::Html,
            },
        );

        Self { templates }
    }
}

/// Global analytics engine instance
static ANALYTICS_ENGINE: tokio::sync::OnceCell<Arc<AnalyticsEngine>> =
    tokio::sync::OnceCell::const_new();

/// Initialize the analytics engine
pub async fn initialize_analytics(config: Option<AnalyticsConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let engine = Arc::new(AnalyticsEngine::new(config).await?);

    ANALYTICS_ENGINE
        .set(engine)
        .map_err(|_| anyhow::anyhow!("Analytics engine already initialized"))?;

    Ok(())
}

/// Get the global analytics engine instance
pub async fn get_analytics_engine() -> Result<Arc<AnalyticsEngine>> {
    ANALYTICS_ENGINE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Analytics engine not initialized"))
        .map(Arc::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analytics_initialization() -> Result<()> {
        let config = AnalyticsConfig::default();
        let engine = AnalyticsEngine::new(config).await?;

        // Test basic functionality
        engine.collect_metrics().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_cost_analysis() -> Result<()> {
        let config = AnalyticsConfig::default();
        let engine = AnalyticsEngine::new(config).await?;

        let costs = engine.analyze_costs().await?;
        assert!(costs.total_cost >= 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_dashboard_generation() -> Result<()> {
        let config = AnalyticsConfig::default();
        let engine = AnalyticsEngine::new(config).await?;

        let dashboard = engine.generate_dashboard().await?;
        assert!(!dashboard.widgets.is_empty());

        Ok(())
    }
}
