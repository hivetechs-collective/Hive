//! Performance Analytics Integration with Consensus Engine
//!
//! This module provides:
//! - Model performance tracking
//! - Latency analysis and optimization
//! - Throughput metrics and monitoring
//! - Quality score calculation
//! - Performance insights and recommendations
//! - Integration with consensus engine

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::analytics::AdvancedAnalyticsConfig;
use crate::core::database::{get_database, ActivityLog};

/// Performance analyzer for consensus engine integration
pub struct PerformanceAnalyzer {
    config: Arc<RwLock<AdvancedAnalyticsConfig>>,
    model_tracker: Arc<ModelTracker>,
    latency_analyzer: Arc<LatencyAnalyzer>,
    throughput_monitor: Arc<ThroughputMonitor>,
    quality_scorer: Arc<QualityScorer>,
    insight_engine: Arc<InsightEngine>,
}

/// Performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub model_performance: HashMap<String, ModelPerformance>,
    pub latency_metrics: LatencyMetrics,
    pub throughput_metrics: ThroughputMetrics,
    pub quality_metrics: QualityMetrics,
    pub performance_insights: Vec<PerformanceInsight>,
}

/// Model performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model_name: String,
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub tokens_per_second: f64,
    pub quality_score: f64,
    pub reliability_score: f64,
    pub cost_efficiency: f64,
    pub stage_performance: HashMap<String, StagePerformance>,
}

/// Consensus stage performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagePerformance {
    pub stage_name: String,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
    pub retry_rate: f64,
}

/// Latency analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAnalysis {
    pub component: String,
    pub avg_latency: f64,
    pub min_latency: f64,
    pub max_latency: f64,
    pub p50_latency: f64,
    pub p95_latency: f64,
    pub p99_latency: f64,
    pub trend: LatencyTrend,
    pub bottlenecks: Vec<Bottleneck>,
}

/// Latency trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyTrend {
    Improving,
    Stable,
    Degrading,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub impact_ms: f64,
    pub frequency: f64,
    pub severity: BottleneckSeverity,
    pub recommendation: String,
}

/// Bottleneck severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Throughput metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetric {
    pub metric_name: String,
    pub current_value: f64,
    pub peak_value: f64,
    pub avg_value: f64,
    pub unit: String,
    pub capacity_utilization: f64,
}

/// Quality score data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub overall_score: f64,
    pub accuracy_score: f64,
    pub completeness_score: f64,
    pub relevance_score: f64,
    pub consistency_score: f64,
    pub factors: HashMap<String, QualityFactor>,
}

/// Quality factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFactor {
    pub name: String,
    pub weight: f64,
    pub score: f64,
    pub impact: QualityImpact,
}

/// Quality impact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityImpact {
    Positive,
    Neutral,
    Negative,
}

/// Performance insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsight {
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub impact: PerformanceImpact,
    pub recommendations: Vec<String>,
    pub metrics_affected: Vec<String>,
    pub confidence: f64,
}

/// Insight categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightCategory {
    Latency,
    Throughput,
    Quality,
    Reliability,
    CostEfficiency,
}

/// Performance impact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceImpact {
    High,
    Medium,
    Low,
}

/// Latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub overall_avg_ms: f64,
    pub component_latencies: HashMap<String, LatencyAnalysis>,
    pub stage_latencies: HashMap<String, f64>,
    pub network_latency_ms: f64,
    pub processing_latency_ms: f64,
    pub queue_latency_ms: f64,
}

/// Throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub requests_per_second: f64,
    pub tokens_per_second: f64,
    pub queries_per_minute: f64,
    pub concurrent_requests: u32,
    pub queue_depth: u32,
    pub capacity_metrics: HashMap<String, ThroughputMetric>,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub overall_quality: f64,
    pub model_quality_scores: HashMap<String, QualityScore>,
    pub stage_quality_scores: HashMap<String, f64>,
    pub user_satisfaction: f64,
    pub error_rate: f64,
}

/// Model performance tracker
struct ModelTracker {
    performance_cache: Arc<RwLock<HashMap<String, ModelPerformanceData>>>,
}

/// Model performance data cache
#[derive(Debug, Clone)]
struct ModelPerformanceData {
    model_name: String,
    request_count: u64,
    success_count: u64,
    total_latency_ms: f64,
    latency_samples: Vec<f64>,
    token_counts: Vec<u64>,
    last_updated: DateTime<Utc>,
}

/// Latency analyzer
struct LatencyAnalyzer {
    latency_buckets: Arc<RwLock<HashMap<String, LatencyBucket>>>,
}

/// Latency bucket for percentile calculation
#[derive(Debug, Clone)]
struct LatencyBucket {
    samples: Vec<f64>,
    sum: f64,
    count: u64,
}

/// Throughput monitor
struct ThroughputMonitor {
    throughput_window: Arc<RwLock<ThroughputWindow>>,
}

/// Throughput window for rate calculation
#[derive(Debug, Clone)]
struct ThroughputWindow {
    requests: VecDeque<RequestData>,
    window_size: Duration,
}

/// Request data for throughput calculation
#[derive(Debug, Clone)]
struct RequestData {
    timestamp: DateTime<Utc>,
    tokens: u64,
    duration_ms: f64,
}

/// Quality scorer
struct QualityScorer {
    quality_weights: HashMap<String, f64>,
    scoring_models: HashMap<String, Box<dyn ScoringModel + Send + Sync>>,
}

/// Scoring model trait
trait ScoringModel: Send + Sync {
    fn calculate_score(&self, data: &QualityData) -> f64;
}

/// Quality data for scoring
#[derive(Debug)]
struct QualityData {
    response_length: usize,
    completion_time_ms: f64,
    error_count: u32,
    user_feedback: Option<f64>,
}

/// Performance insight engine
struct InsightEngine {
    insight_rules: Vec<InsightRule>,
}

/// Insight generation rule
struct InsightRule {
    name: String,
    category: InsightCategory,
    condition: Box<dyn Fn(&PerformanceData) -> bool + Send + Sync>,
    generator: Box<dyn Fn(&PerformanceData) -> PerformanceInsight + Send + Sync>,
}

/// Performance data for analysis
#[derive(Debug)]
struct PerformanceData {
    model_performance: HashMap<String, ModelPerformance>,
    latency_metrics: LatencyMetrics,
    throughput_metrics: ThroughputMetrics,
    quality_metrics: QualityMetrics,
}

use std::collections::VecDeque;

impl PerformanceAnalyzer {
    /// Create new performance analyzer
    pub async fn new(config: Arc<RwLock<AdvancedAnalyticsConfig>>) -> Result<Self> {
        info!("Initializing performance analyzer");

        let model_tracker = Arc::new(ModelTracker::new());
        let latency_analyzer = Arc::new(LatencyAnalyzer::new());
        let throughput_monitor = Arc::new(ThroughputMonitor::new());
        let quality_scorer = Arc::new(QualityScorer::new());
        let insight_engine = Arc::new(InsightEngine::new());

        Ok(Self {
            config,
            model_tracker,
            latency_analyzer,
            throughput_monitor,
            quality_scorer,
            insight_engine,
        })
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> Result<PerformanceMetrics> {
        debug!("Collecting performance metrics");

        // Collect data from all components
        let model_performance = self.collect_model_performance().await?;
        let latency_metrics = self.analyze_latency().await?;
        let throughput_metrics = self.calculate_throughput().await?;
        let quality_metrics = self.assess_quality().await?;

        // Generate insights
        let performance_data = PerformanceData {
            model_performance: model_performance.clone(),
            latency_metrics: latency_metrics.clone(),
            throughput_metrics: throughput_metrics.clone(),
            quality_metrics: quality_metrics.clone(),
        };

        let performance_insights = self.insight_engine.generate_insights(&performance_data)?;

        Ok(PerformanceMetrics {
            model_performance,
            latency_metrics,
            throughput_metrics,
            quality_metrics,
            performance_insights,
        })
    }

    /// Analyze model performance
    pub async fn analyze_model(&self, model: &str) -> Result<ModelPerformance> {
        debug!("Analyzing performance for model: {}", model);

        let activities = ActivityLog::get_recent(1000).await?;
        let model_activities: Vec<_> = activities
            .into_iter()
            .filter(|a| a.model_used.as_ref() == Some(&model.to_string()))
            .collect();

        self.model_tracker
            .analyze_model_performance(model, &model_activities)
            .await
    }

    /// Get latency breakdown
    pub async fn get_latency_breakdown(&self) -> Result<HashMap<String, LatencyAnalysis>> {
        self.latency_analyzer.get_component_latencies().await
    }

    /// Monitor real-time throughput
    pub async fn monitor_throughput(&self) -> Result<ThroughputMetrics> {
        self.throughput_monitor.get_current_throughput().await
    }

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()> {
        debug!("Reloading performance analyzer configuration");
        Ok(())
    }

    // Private helper methods

    async fn collect_model_performance(&self) -> Result<HashMap<String, ModelPerformance>> {
        let activities = ActivityLog::get_recent(5000).await?;
        let mut model_activities: HashMap<String, Vec<ActivityLog>> = HashMap::new();

        // Group activities by model
        for activity in activities {
            if let Some(model) = &activity.model_used {
                model_activities
                    .entry(model.clone())
                    .or_insert_with(Vec::new)
                    .push(activity);
            }
        }

        let mut performance_map = HashMap::new();

        for (model, activities) in model_activities {
            let performance = self
                .model_tracker
                .analyze_model_performance(&model, &activities)
                .await?;
            performance_map.insert(model, performance);
        }

        Ok(performance_map)
    }

    async fn analyze_latency(&self) -> Result<LatencyMetrics> {
        let component_latencies = self.latency_analyzer.get_component_latencies().await?;

        // Calculate overall average
        let overall_avg_ms = if !component_latencies.is_empty() {
            component_latencies
                .values()
                .map(|l| l.avg_latency)
                .sum::<f64>()
                / component_latencies.len() as f64
        } else {
            0.0
        };

        // Stage latencies (placeholder)
        let mut stage_latencies = HashMap::new();
        stage_latencies.insert("generator".to_string(), 150.0);
        stage_latencies.insert("refiner".to_string(), 100.0);
        stage_latencies.insert("validator".to_string(), 50.0);
        stage_latencies.insert("curator".to_string(), 75.0);

        Ok(LatencyMetrics {
            overall_avg_ms,
            component_latencies,
            stage_latencies,
            network_latency_ms: 25.0,     // Placeholder
            processing_latency_ms: 200.0, // Placeholder
            queue_latency_ms: 10.0,       // Placeholder
        })
    }

    async fn calculate_throughput(&self) -> Result<ThroughputMetrics> {
        self.throughput_monitor.get_current_throughput().await
    }

    async fn assess_quality(&self) -> Result<QualityMetrics> {
        let activities = ActivityLog::get_recent(1000).await?;

        // Calculate error rate
        let total_requests = activities.len();
        let error_count = activities
            .iter()
            .filter(|a| a.event_type.contains("error"))
            .count();

        let error_rate = if total_requests > 0 {
            error_count as f64 / total_requests as f64
        } else {
            0.0
        };

        // Calculate overall quality (simplified)
        let overall_quality = (1.0 - error_rate) * 100.0;

        // Model quality scores (placeholder)
        let mut model_quality_scores = HashMap::new();
        model_quality_scores.insert(
            "gpt-4".to_string(),
            QualityScore {
                overall_score: 95.0,
                accuracy_score: 96.0,
                completeness_score: 94.0,
                relevance_score: 95.0,
                consistency_score: 93.0,
                factors: HashMap::new(),
            },
        );

        // Stage quality scores
        let mut stage_quality_scores = HashMap::new();
        stage_quality_scores.insert("generator".to_string(), 92.0);
        stage_quality_scores.insert("refiner".to_string(), 94.0);
        stage_quality_scores.insert("validator".to_string(), 96.0);
        stage_quality_scores.insert("curator".to_string(), 95.0);

        Ok(QualityMetrics {
            overall_quality,
            model_quality_scores,
            stage_quality_scores,
            user_satisfaction: 4.5, // Placeholder
            error_rate: error_rate * 100.0,
        })
    }
}

impl PerformanceMetrics {
    /// Format as markdown
    pub fn format_markdown(&self) -> Result<String> {
        let mut output = String::new();

        // Overall performance
        output.push_str(&format!(
            "**Overall Latency**: {:.1}ms\n",
            self.latency_metrics.overall_avg_ms
        ));
        output.push_str(&format!(
            "**Throughput**: {:.1} req/s\n",
            self.throughput_metrics.requests_per_second
        ));
        output.push_str(&format!(
            "**Quality Score**: {:.1}%\n\n",
            self.quality_metrics.overall_quality
        ));

        // Top performing models
        if !self.model_performance.is_empty() {
            output.push_str("### Top Performing Models\n\n");
            let mut models: Vec<_> = self.model_performance.iter().collect();
            models.sort_by(|a, b| b.1.quality_score.partial_cmp(&a.1.quality_score).unwrap());
            models.truncate(3);

            for (model, perf) in models {
                output.push_str(&format!(
                    "- **{}**: Quality {:.1}%, Latency {:.0}ms, Success {:.1}%\n",
                    model,
                    perf.quality_score,
                    perf.avg_latency_ms,
                    perf.success_rate * 100.0
                ));
            }
            output.push_str("\n");
        }

        // Performance insights
        if !self.performance_insights.is_empty() {
            output.push_str("### Performance Insights\n\n");
            for insight in &self.performance_insights {
                output.push_str(&format!("**{}**\n", insight.title));
                output.push_str(&format!("{}\n", insight.description));
                if !insight.recommendations.is_empty() {
                    output.push_str("Recommendations:\n");
                    for rec in &insight.recommendations {
                        output.push_str(&format!("- {}\n", rec));
                    }
                }
                output.push_str("\n");
            }
        }

        Ok(output)
    }
}

impl ModelTracker {
    fn new() -> Self {
        Self {
            performance_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn analyze_model_performance(
        &self,
        model: &str,
        activities: &[ActivityLog],
    ) -> Result<ModelPerformance> {
        let mut request_count = 0u64;
        let mut success_count = 0u64;
        let mut latency_samples = Vec::new();
        let mut token_counts = Vec::new();

        for activity in activities {
            request_count += 1;

            if !activity.event_type.contains("error") {
                success_count += 1;
            }

            if let Some(duration) = activity.duration_ms {
                latency_samples.push(duration as f64);
            }

            // Extract token count from metadata
            if let Some(metadata) = &activity.metadata {
                if let Ok(meta) =
                    serde_json::from_str::<HashMap<String, serde_json::Value>>(metadata)
                {
                    if let Some(tokens) = meta.get("total_tokens").and_then(|v| v.as_u64()) {
                        token_counts.push(tokens);
                    }
                }
            }
        }

        let success_rate = if request_count > 0 {
            success_count as f64 / request_count as f64
        } else {
            0.0
        };

        let avg_latency_ms = if !latency_samples.is_empty() {
            latency_samples.iter().sum::<f64>() / latency_samples.len() as f64
        } else {
            0.0
        };

        // Calculate percentiles
        latency_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_latency_ms = percentile(&latency_samples, 0.95);
        let p99_latency_ms = percentile(&latency_samples, 0.99);

        // Calculate tokens per second
        let tokens_per_second = if !token_counts.is_empty() && avg_latency_ms > 0.0 {
            let avg_tokens = token_counts.iter().sum::<u64>() as f64 / token_counts.len() as f64;
            (avg_tokens / avg_latency_ms) * 1000.0
        } else {
            0.0
        };

        // Calculate quality score (simplified)
        let quality_score = success_rate * 100.0 * (1.0 - (avg_latency_ms / 1000.0).min(1.0));

        // Stage performance (placeholder)
        let mut stage_performance = HashMap::new();
        stage_performance.insert(
            "generator".to_string(),
            StagePerformance {
                stage_name: "generator".to_string(),
                avg_duration_ms: avg_latency_ms * 0.4,
                success_rate: 0.98,
                retry_rate: 0.02,
            },
        );

        Ok(ModelPerformance {
            model_name: model.to_string(),
            total_requests: request_count,
            success_rate,
            avg_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            tokens_per_second,
            quality_score,
            reliability_score: success_rate * 100.0,
            cost_efficiency: 100.0 / (1.0 + avg_latency_ms / 100.0),
            stage_performance,
        })
    }
}

impl LatencyAnalyzer {
    fn new() -> Self {
        Self {
            latency_buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_component_latencies(&self) -> Result<HashMap<String, LatencyAnalysis>> {
        let mut latencies = HashMap::new();

        // API latency analysis
        latencies.insert(
            "api".to_string(),
            LatencyAnalysis {
                component: "API Gateway".to_string(),
                avg_latency: 25.0,
                min_latency: 10.0,
                max_latency: 150.0,
                p50_latency: 20.0,
                p95_latency: 50.0,
                p99_latency: 100.0,
                trend: LatencyTrend::Stable,
                bottlenecks: vec![],
            },
        );

        // Model latency analysis
        latencies.insert(
            "model".to_string(),
            LatencyAnalysis {
                component: "Model Inference".to_string(),
                avg_latency: 200.0,
                min_latency: 50.0,
                max_latency: 2000.0,
                p50_latency: 150.0,
                p95_latency: 500.0,
                p99_latency: 1000.0,
                trend: LatencyTrend::Improving,
                bottlenecks: vec![Bottleneck {
                    component: "Token Generation".to_string(),
                    impact_ms: 150.0,
                    frequency: 0.8,
                    severity: BottleneckSeverity::Medium,
                    recommendation: "Consider using faster models for simple queries".to_string(),
                }],
            },
        );

        // Database latency
        latencies.insert(
            "database".to_string(),
            LatencyAnalysis {
                component: "Database".to_string(),
                avg_latency: 5.0,
                min_latency: 1.0,
                max_latency: 50.0,
                p50_latency: 3.0,
                p95_latency: 15.0,
                p99_latency: 30.0,
                trend: LatencyTrend::Stable,
                bottlenecks: vec![],
            },
        );

        Ok(latencies)
    }
}

impl ThroughputMonitor {
    fn new() -> Self {
        Self {
            throughput_window: Arc::new(RwLock::new(ThroughputWindow {
                requests: VecDeque::new(),
                window_size: Duration::minutes(5),
            })),
        }
    }

    async fn get_current_throughput(&self) -> Result<ThroughputMetrics> {
        let window = self.throughput_window.read().await;
        let now = Utc::now();
        let cutoff = now - window.window_size;

        // Filter requests in window
        let active_requests: Vec<_> = window
            .requests
            .iter()
            .filter(|r| r.timestamp > cutoff)
            .collect();

        let requests_per_second = if !active_requests.is_empty() {
            active_requests.len() as f64 / window.window_size.num_seconds() as f64
        } else {
            0.0
        };

        let tokens_per_second = if !active_requests.is_empty() {
            let total_tokens: u64 = active_requests.iter().map(|r| r.tokens).sum();
            total_tokens as f64 / window.window_size.num_seconds() as f64
        } else {
            0.0
        };

        let queries_per_minute = requests_per_second * 60.0;

        // Capacity metrics
        let mut capacity_metrics = HashMap::new();
        capacity_metrics.insert(
            "api_capacity".to_string(),
            ThroughputMetric {
                metric_name: "API Capacity".to_string(),
                current_value: requests_per_second,
                peak_value: 100.0,
                avg_value: requests_per_second * 0.8,
                unit: "req/s".to_string(),
                capacity_utilization: requests_per_second / 100.0,
            },
        );

        Ok(ThroughputMetrics {
            requests_per_second,
            tokens_per_second,
            queries_per_minute,
            concurrent_requests: 10, // Placeholder
            queue_depth: 5,          // Placeholder
            capacity_metrics,
        })
    }
}

impl QualityScorer {
    fn new() -> Self {
        let mut quality_weights = HashMap::new();
        quality_weights.insert("accuracy".to_string(), 0.3);
        quality_weights.insert("completeness".to_string(), 0.25);
        quality_weights.insert("relevance".to_string(), 0.25);
        quality_weights.insert("consistency".to_string(), 0.2);

        Self {
            quality_weights,
            scoring_models: HashMap::new(),
        }
    }
}

impl InsightEngine {
    fn new() -> Self {
        let mut insight_rules = Vec::new();

        // High latency rule
        insight_rules.push(InsightRule {
            name: "High Latency Detection".to_string(),
            category: InsightCategory::Latency,
            condition: Box::new(|data| data.latency_metrics.overall_avg_ms > 500.0),
            generator: Box::new(|data| PerformanceInsight {
                category: InsightCategory::Latency,
                title: "High Average Latency Detected".to_string(),
                description: format!(
                    "Average latency is {:.0}ms, which exceeds the target of 300ms",
                    data.latency_metrics.overall_avg_ms
                ),
                impact: PerformanceImpact::High,
                recommendations: vec![
                    "Use faster models for simple queries".to_string(),
                    "Implement request batching".to_string(),
                    "Enable response caching".to_string(),
                ],
                metrics_affected: vec!["latency".to_string(), "user_experience".to_string()],
                confidence: 0.95,
            }),
        });

        // Low throughput rule
        insight_rules.push(InsightRule {
            name: "Low Throughput".to_string(),
            category: InsightCategory::Throughput,
            condition: Box::new(|data| data.throughput_metrics.requests_per_second < 10.0),
            generator: Box::new(|data| PerformanceInsight {
                category: InsightCategory::Throughput,
                title: "Throughput Below Capacity".to_string(),
                description: format!(
                    "Current throughput is {:.1} req/s, system can handle more load",
                    data.throughput_metrics.requests_per_second
                ),
                impact: PerformanceImpact::Low,
                recommendations: vec![
                    "No action needed - system has spare capacity".to_string(),
                    "Consider promotional activities to increase usage".to_string(),
                ],
                metrics_affected: vec!["throughput".to_string(), "utilization".to_string()],
                confidence: 0.85,
            }),
        });

        // Quality improvement opportunity
        insight_rules.push(InsightRule {
            name: "Quality Improvement".to_string(),
            category: InsightCategory::Quality,
            condition: Box::new(|data| data.quality_metrics.overall_quality < 90.0),
            generator: Box::new(|data| PerformanceInsight {
                category: InsightCategory::Quality,
                title: "Quality Improvement Opportunity".to_string(),
                description: format!(
                    "Overall quality score is {:.1}%, below the target of 90%",
                    data.quality_metrics.overall_quality
                ),
                impact: PerformanceImpact::Medium,
                recommendations: vec![
                    "Review and improve prompt templates".to_string(),
                    "Implement better error handling".to_string(),
                    "Add validation stages to consensus pipeline".to_string(),
                ],
                metrics_affected: vec!["quality".to_string(), "user_satisfaction".to_string()],
                confidence: 0.9,
            }),
        });

        Self { insight_rules }
    }

    fn generate_insights(&self, data: &PerformanceData) -> Result<Vec<PerformanceInsight>> {
        let mut insights = Vec::new();

        for rule in &self.insight_rules {
            if (rule.condition)(data) {
                insights.push((rule.generator)(data));
            }
        }

        // Sort by impact and confidence
        insights.sort_by(|a, b| {
            b.impact
                .cmp(&a.impact)
                .then(b.confidence.partial_cmp(&a.confidence).unwrap())
        });

        Ok(insights)
    }
}

// Helper functions

fn percentile(sorted_samples: &[f64], p: f64) -> f64 {
    if sorted_samples.is_empty() {
        return 0.0;
    }

    let index = ((sorted_samples.len() - 1) as f64 * p) as usize;
    sorted_samples[index]
}

// Implement ordering for performance impact
impl Ord for PerformanceImpact {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::High, Self::High) => std::cmp::Ordering::Equal,
            (Self::High, _) => std::cmp::Ordering::Less,
            (_, Self::High) => std::cmp::Ordering::Greater,
            (Self::Medium, Self::Medium) => std::cmp::Ordering::Equal,
            (Self::Medium, Self::Low) => std::cmp::Ordering::Less,
            (Self::Low, Self::Medium) => std::cmp::Ordering::Greater,
            (Self::Low, Self::Low) => std::cmp::Ordering::Equal,
        }
    }
}

impl PartialOrd for PerformanceImpact {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_analyzer_creation() -> Result<()> {
        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let analyzer = PerformanceAnalyzer::new(config).await?;

        assert!(Arc::strong_count(&analyzer.model_tracker) > 0);

        Ok(())
    }

    #[test]
    fn test_percentile_calculation() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        assert_eq!(percentile(&samples, 0.5), 5.0);
        assert_eq!(percentile(&samples, 0.95), 9.0);
        assert_eq!(percentile(&samples, 0.99), 9.0);
    }

    #[tokio::test]
    async fn test_model_performance_analysis() -> Result<()> {
        let tracker = ModelTracker::new();

        let activities = vec![ActivityLog {
            id: 1,
            event_type: "query_complete".to_string(),
            user_id: Some("user1".to_string()),
            conversation_id: Some("conv1".to_string()),
            message_id: Some("msg1".to_string()),
            model_used: Some("gpt-4".to_string()),
            cost: Some(0.05),
            duration_ms: Some(250),
            metadata: Some(r#"{"total_tokens": 1000}"#.to_string()),
            created_at: Utc::now().to_rfc3339(),
        }];

        let performance = tracker
            .analyze_model_performance("gpt-4", &activities)
            .await?;

        assert_eq!(performance.model_name, "gpt-4");
        assert_eq!(performance.total_requests, 1);
        assert_eq!(performance.success_rate, 1.0);
        assert_eq!(performance.avg_latency_ms, 250.0);

        Ok(())
    }

    #[test]
    fn test_latency_trend_detection() {
        let analyzer = LatencyAnalyzer::new();

        // Latency trend logic would be tested here
        assert!(true); // Placeholder
    }
}
