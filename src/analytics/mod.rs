//! Advanced Analytics Engine for Hive AI
//! 
//! This module provides comprehensive business intelligence and analytics capabilities:
//! - ML-powered trend analysis with predictive modeling
//! - Executive reporting with professional visualizations
//! - Cost intelligence and optimization recommendations
//! - Real-time analytics dashboards
//! - Performance analytics integration
//! - Advanced ML models for forecasting
//! - Enterprise reporting templates
//! - Real-time alerting system
//! - Custom dashboard builder
//! - Analytics REST API
//! - Comprehensive export functionality

pub mod trend_analysis;
pub mod executive;
pub mod cost_intelligence;
pub mod dashboard;
pub mod performance;
pub mod ml_models;
pub mod templates;
pub mod alerts;
pub mod builder;
pub mod api;
pub mod export;

// Re-export key types
pub use trend_analysis::{
    TrendAnalyzer, TrendPrediction, SeasonalPattern, AnomalyDetection,
    TimeSeriesModel, ForecastHorizon, ConfidenceInterval,
};

pub use executive::{
    ExecutiveReporter, ExecutiveSummary, KeyMetric, BusinessInsight,
    ReportFormat, ReportPeriod, VisualizationConfig,
};

pub use cost_intelligence::{
    CostIntelligence, CostOptimization, BudgetAlert, SpendingPattern,
    ModelEfficiency, CostAllocation, OptimizationStrategy,
};

pub use dashboard::{
    DashboardEngine, DashboardLayout, Widget, WidgetType,
    RealTimeMetric, RefreshRate, DataStream,
};

pub use performance::{
    PerformanceAnalyzer, ModelPerformance, LatencyAnalysis,
    ThroughputMetric, QualityScore, PerformanceInsight,
};

pub use ml_models::{
    MLEngine, MLPrediction, ModelType, MLConfig,
    AnomalyDetectionResult, TimeSeriesData, TimeSeriesDecomposition,
};

pub use templates::{
    TemplateManager, ReportTemplate, TemplateType, ReportBuilder,
    ReportRequest, Report, ReportSection,
};

pub use alerts::{
    AlertManager, AlertRule, Alert, AlertSeverity, AlertType,
    AlertCondition, NotificationChannel, AlertConfig,
};

pub use builder::{
    DashboardBuilder, Dashboard, DashboardConfig, WidgetConfig,
    DashboardTheme, DataSource, DataBinding,
};

pub use api::{
    AnalyticsApi, ApiConfig, ApiKey, Permission, ApiResponse,
};

pub use export::{
    DataExporter, ExportConfig, ExportFormat, ExportJob,
    ExportStatus, DeliveryMethod, FormattingOptions,
};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Advanced Analytics Engine configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdvancedAnalyticsConfig {
    /// Enable ML-powered predictions
    pub enable_ml_predictions: bool,
    /// Prediction horizon (days)
    pub prediction_horizon_days: u32,
    /// Confidence level for predictions (0.0-1.0)
    pub confidence_level: f64,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Real-time update interval (seconds)
    pub real_time_interval_secs: u64,
    /// Executive report generation settings
    pub executive_report: ExecutiveReportConfig,
    /// Cost optimization settings
    pub cost_optimization: CostOptimizationConfig,
}

/// Executive report configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutiveReportConfig {
    /// Include predictive analytics
    pub include_predictions: bool,
    /// Include competitive analysis
    pub include_competitive: bool,
    /// Visualization style
    pub visualization_style: String,
    /// Report format preferences
    pub preferred_format: String,
}

/// Cost optimization configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostOptimizationConfig {
    /// Budget threshold for alerts
    pub budget_alert_threshold: f64,
    /// Enable automatic optimization suggestions
    pub auto_suggest_optimizations: bool,
    /// Model efficiency threshold
    pub efficiency_threshold: f64,
    /// Cost allocation tracking
    pub track_allocation: bool,
}

impl Default for AdvancedAnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_ml_predictions: true,
            prediction_horizon_days: 30,
            confidence_level: 0.95,
            enable_anomaly_detection: true,
            real_time_interval_secs: 5,
            executive_report: ExecutiveReportConfig {
                include_predictions: true,
                include_competitive: false,
                visualization_style: "professional".to_string(),
                preferred_format: "markdown".to_string(),
            },
            cost_optimization: CostOptimizationConfig {
                budget_alert_threshold: 1000.0,
                auto_suggest_optimizations: true,
                efficiency_threshold: 0.8,
                track_allocation: true,
            },
        }
    }
}

/// Main Advanced Analytics Engine
pub struct AdvancedAnalyticsEngine {
    config: Arc<RwLock<AdvancedAnalyticsConfig>>,
    trend_analyzer: Arc<TrendAnalyzer>,
    executive_reporter: Arc<ExecutiveReporter>,
    cost_intelligence: Arc<CostIntelligence>,
    dashboard_engine: Arc<DashboardEngine>,
    performance_analyzer: Arc<PerformanceAnalyzer>,
}

impl AdvancedAnalyticsEngine {
    /// Create a new advanced analytics engine
    pub async fn new(config: AdvancedAnalyticsConfig) -> Result<Self> {
        let config = Arc::new(RwLock::new(config));
        
        // Initialize components
        let trend_analyzer = Arc::new(TrendAnalyzer::new(Arc::clone(&config)).await?);
        let executive_reporter = Arc::new(ExecutiveReporter::new(Arc::clone(&config)).await?);
        let cost_intelligence = Arc::new(CostIntelligence::new(Arc::clone(&config)).await?);
        let dashboard_engine = Arc::new(DashboardEngine::new(Arc::clone(&config)).await?);
        let performance_analyzer = Arc::new(PerformanceAnalyzer::new(Arc::clone(&config)).await?);

        Ok(Self {
            config,
            trend_analyzer,
            executive_reporter,
            cost_intelligence,
            dashboard_engine,
            performance_analyzer,
        })
    }

    /// Get trend analyzer
    pub fn trends(&self) -> Arc<TrendAnalyzer> {
        Arc::clone(&self.trend_analyzer)
    }

    /// Get executive reporter
    pub fn executive(&self) -> Arc<ExecutiveReporter> {
        Arc::clone(&self.executive_reporter)
    }

    /// Get cost intelligence
    pub fn costs(&self) -> Arc<CostIntelligence> {
        Arc::clone(&self.cost_intelligence)
    }

    /// Get dashboard engine
    pub fn dashboard(&self) -> Arc<DashboardEngine> {
        Arc::clone(&self.dashboard_engine)
    }

    /// Get performance analyzer
    pub fn performance(&self) -> Arc<PerformanceAnalyzer> {
        Arc::clone(&self.performance_analyzer)
    }

    /// Generate comprehensive analytics overview
    pub async fn overview(&self, period: &str, format: &str) -> Result<String> {
        // Collect data from all components
        let trends = self.trend_analyzer.get_current_trends().await?;
        let executive_summary = self.executive_reporter.generate_summary(period).await?;
        let cost_insights = self.cost_intelligence.get_insights().await?;
        let performance_metrics = self.performance_analyzer.get_metrics().await?;

        // Format based on requested format
        match format {
            "json" => self.format_as_json(trends, executive_summary, cost_insights, performance_metrics),
            "markdown" | _ => self.format_as_markdown(trends, executive_summary, cost_insights, performance_metrics),
        }
    }

    /// Update configuration
    pub async fn update_config(&self, config: AdvancedAnalyticsConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        
        // Notify components of config change
        self.trend_analyzer.reload_config().await?;
        self.executive_reporter.reload_config().await?;
        self.cost_intelligence.reload_config().await?;
        self.dashboard_engine.reload_config().await?;
        self.performance_analyzer.reload_config().await?;

        Ok(())
    }

    // Private helper methods
    
    fn format_as_markdown(
        &self,
        trends: trend_analysis::TrendSummary,
        executive: executive::ExecutiveSummary,
        costs: cost_intelligence::CostInsights,
        performance: performance::PerformanceMetrics,
    ) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("# Analytics Overview\n\n");
        output.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));
        
        // Executive Summary
        output.push_str("## Executive Summary\n\n");
        output.push_str(&executive.format_markdown()?);
        output.push_str("\n");
        
        // Trends
        output.push_str("## Trend Analysis\n\n");
        output.push_str(&trends.format_markdown()?);
        output.push_str("\n");
        
        // Cost Intelligence
        output.push_str("## Cost Intelligence\n\n");
        output.push_str(&costs.format_markdown()?);
        output.push_str("\n");
        
        // Performance
        output.push_str("## Performance Analytics\n\n");
        output.push_str(&performance.format_markdown()?);
        
        Ok(output)
    }

    fn format_as_json(
        &self,
        trends: trend_analysis::TrendSummary,
        executive: executive::ExecutiveSummary,
        costs: cost_intelligence::CostInsights,
        performance: performance::PerformanceMetrics,
    ) -> Result<String> {
        let output = serde_json::json!({
            "generated_at": chrono::Utc::now(),
            "executive_summary": executive,
            "trends": trends,
            "cost_intelligence": costs,
            "performance": performance,
        });
        
        serde_json::to_string_pretty(&output).map_err(Into::into)
    }
}

/// Global advanced analytics engine instance
static ADVANCED_ANALYTICS: tokio::sync::OnceCell<Arc<AdvancedAnalyticsEngine>> = 
    tokio::sync::OnceCell::const_new();

/// Initialize the advanced analytics engine
pub async fn initialize_advanced_analytics(config: Option<AdvancedAnalyticsConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let engine = Arc::new(AdvancedAnalyticsEngine::new(config).await?);
    
    ADVANCED_ANALYTICS
        .set(engine)
        .map_err(|_| anyhow::anyhow!("Advanced analytics engine already initialized"))?;
    
    Ok(())
}

/// Get the global advanced analytics engine
pub async fn get_advanced_analytics() -> Result<Arc<AdvancedAnalyticsEngine>> {
    ADVANCED_ANALYTICS
        .get()
        .ok_or_else(|| anyhow::anyhow!("Advanced analytics engine not initialized"))
        .map(Arc::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_advanced_analytics_initialization() -> Result<()> {
        let config = AdvancedAnalyticsConfig::default();
        let engine = AdvancedAnalyticsEngine::new(config).await?;
        
        // Test that all components are initialized
        assert!(Arc::strong_count(&engine.trend_analyzer) > 0);
        assert!(Arc::strong_count(&engine.executive_reporter) > 0);
        assert!(Arc::strong_count(&engine.cost_intelligence) > 0);
        assert!(Arc::strong_count(&engine.dashboard_engine) > 0);
        assert!(Arc::strong_count(&engine.performance_analyzer) > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_overview_generation() -> Result<()> {
        let config = AdvancedAnalyticsConfig::default();
        let engine = AdvancedAnalyticsEngine::new(config).await?;
        
        let overview = engine.overview("week", "markdown").await?;
        assert!(!overview.is_empty());
        assert!(overview.contains("Analytics Overview"));
        
        Ok(())
    }
}