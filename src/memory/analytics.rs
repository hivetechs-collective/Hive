//! Memory analytics and insights dashboard
//!
//! This module provides:
//! - Memory usage analytics
//! - Insight generation from patterns
//! - Trend analysis and forecasting
//! - Performance metrics and optimization recommendations

use anyhow::{Context as _, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use tracing::{debug, info};

use crate::core::memory::{InsightType, MemoryInsight, SemanticSearchResult};
use crate::memory::knowledge_graph::KnowledgeGraph;
use crate::memory::pattern_learning::{Pattern, PatternLearner};

/// Memory analytics engine
#[derive(Debug)]
pub struct MemoryAnalyzer {
    /// Analytics data
    data: AnalyticsData,
    /// Configuration
    config: AnalyticsConfig,
    /// Time series data
    time_series: TimeSeriesData,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Retention period for analytics data (days)
    pub retention_days: u32,
    /// Minimum data points for trend analysis
    pub min_trend_points: usize,
    /// Anomaly detection threshold
    pub anomaly_threshold: f32,
    /// Enable predictive analytics
    pub enable_predictions: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            retention_days: 90,
            min_trend_points: 7,
            anomaly_threshold: 2.5, // Standard deviations
            enable_predictions: true,
        }
    }
}

/// Analytics data
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    /// Total memories analyzed
    pub total_memories: usize,
    /// Memory access patterns
    pub access_patterns: HashMap<String, AccessPattern>,
    /// Query statistics
    pub query_stats: QueryStatistics,
    /// Topic distribution
    pub topic_distribution: HashMap<String, TopicStats>,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Access pattern for a memory
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Total access count
    pub access_count: usize,
    /// Last access time
    pub last_accessed: Option<DateTime<Utc>>,
    /// Average relevance score when accessed
    pub avg_relevance_score: f32,
    /// Access frequency (accesses per day)
    pub access_frequency: f32,
    /// Trend (positive = increasing, negative = decreasing)
    pub trend: f32,
}

/// Query statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QueryStatistics {
    /// Total queries
    pub total_queries: usize,
    /// Successful queries (with results)
    pub successful_queries: usize,
    /// Failed queries (no results)
    pub failed_queries: usize,
    /// Average results per query
    pub avg_results_per_query: f32,
    /// Average query processing time (ms)
    pub avg_processing_time: f32,
    /// Most common query patterns
    pub common_patterns: Vec<(String, usize)>,
}

/// Topic statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TopicStats {
    /// Number of memories for this topic
    pub memory_count: usize,
    /// Average quality score
    pub avg_quality: f32,
    /// Growth rate (memories per day)
    pub growth_rate: f32,
    /// Last updated
    pub last_updated: Option<DateTime<Utc>>,
    /// Related topics
    pub related_topics: Vec<String>,
}

/// Quality metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Average embedding quality
    pub avg_embedding_quality: f32,
    /// Average retrieval precision
    pub avg_retrieval_precision: f32,
    /// Pattern recognition accuracy
    pub pattern_accuracy: f32,
    /// Knowledge graph connectivity
    pub graph_connectivity: f32,
    /// Overall system quality score
    pub overall_quality: f32,
}

/// Performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average memory indexing time (ms)
    pub avg_indexing_time: f32,
    /// Average search latency (ms)
    pub avg_search_latency: f32,
    /// Cache hit rate
    pub cache_hit_rate: f32,
    /// Memory usage (MB)
    pub memory_usage_mb: f32,
    /// CPU usage percentage
    pub cpu_usage_percent: f32,
}

/// Time series data for trend analysis
#[derive(Debug, Default)]
struct TimeSeriesData {
    /// Daily memory counts
    daily_memories: BTreeMap<String, usize>,
    /// Daily query counts
    daily_queries: BTreeMap<String, usize>,
    /// Daily quality scores
    daily_quality: BTreeMap<String, f32>,
    /// Daily performance metrics
    daily_performance: BTreeMap<String, PerformanceMetrics>,
}

/// Memory metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Total number of memories
    pub total_memories: usize,
    /// Active memories (accessed in last 30 days)
    pub active_memories: usize,
    /// Memory growth rate (per day)
    pub growth_rate: f32,
    /// Most accessed memories
    pub top_memories: Vec<(String, usize)>,
    /// Memory health score (0-100)
    pub health_score: f32,
}

/// Insight generator for creating actionable insights
pub struct InsightGenerator {
    /// Insight generation configuration
    config: InsightConfig,
}

/// Configuration for insight generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightConfig {
    /// Minimum confidence for insights
    pub min_confidence: f32,
    /// Maximum insights to generate
    pub max_insights: usize,
    /// Enable advanced analytics
    pub enable_advanced: bool,
}

impl Default for InsightConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_insights: 10,
            enable_advanced: true,
        }
    }
}

impl MemoryAnalyzer {
    /// Create a new memory analyzer
    pub fn new() -> Self {
        Self::with_config(AnalyticsConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: AnalyticsConfig) -> Self {
        Self {
            data: AnalyticsData::default(),
            config,
            time_series: TimeSeriesData::default(),
        }
    }

    /// Record a search operation
    pub async fn record_search(
        &mut self,
        query: &str,
        results: &[SemanticSearchResult],
    ) -> Result<()> {
        debug!("Recording search analytics for query: {}", query);

        // Update query statistics
        self.data.query_stats.total_queries += 1;
        if !results.is_empty() {
            self.data.query_stats.successful_queries += 1;
            let n = self.data.query_stats.successful_queries as f32;
            let prev_avg = self.data.query_stats.avg_results_per_query;
            self.data.query_stats.avg_results_per_query =
                (prev_avg * (n - 1.0) + results.len() as f32) / n;
        } else {
            self.data.query_stats.failed_queries += 1;
        }

        // Update access patterns
        for result in results {
            let pattern = self
                .data
                .access_patterns
                .entry(result.id.clone())
                .or_default();

            pattern.access_count += 1;
            pattern.last_accessed = Some(Utc::now());

            // Update average relevance score
            let n = pattern.access_count as f32;
            pattern.avg_relevance_score =
                (pattern.avg_relevance_score * (n - 1.0) + result.similarity_score) / n;
        }

        // Extract and record query patterns
        self.extract_query_patterns(query);

        // Update time series
        let today = Utc::now().format("%Y-%m-%d").to_string();
        *self.time_series.daily_queries.entry(today).or_insert(0) += 1;

        Ok(())
    }

    /// Record memory addition
    pub async fn record_memory_added(&mut self, topic: Option<&str>) -> Result<()> {
        self.data.total_memories += 1;

        // Update topic distribution
        if let Some(topic) = topic {
            let topic_stats = self
                .data
                .topic_distribution
                .entry(topic.to_string())
                .or_default();

            topic_stats.memory_count += 1;
            topic_stats.last_updated = Some(Utc::now());
        }

        // Update time series
        let today = Utc::now().format("%Y-%m-%d").to_string();
        *self.time_series.daily_memories.entry(today).or_insert(0) += 1;

        Ok(())
    }

    /// Update quality metrics
    pub async fn update_quality_metrics(&mut self, metrics: QualityMetrics) -> Result<()> {
        let overall_quality = metrics.overall_quality; // Extract value before move
        self.data.quality_metrics = metrics;

        // Update time series
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.time_series
            .daily_quality
            .insert(today, overall_quality);

        Ok(())
    }

    /// Update performance metrics
    pub async fn update_performance_metrics(&mut self, metrics: PerformanceMetrics) -> Result<()> {
        let metrics_clone = metrics.clone(); // Clone for time series
        self.data.performance_metrics = metrics;

        // Update time series
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.time_series
            .daily_performance
            .insert(today, metrics_clone);

        Ok(())
    }

    /// Get memory metrics
    pub fn get_metrics(&self) -> MemoryMetrics {
        let thirty_days_ago = Utc::now() - Duration::days(30);

        // Count active memories
        let active_memories = self
            .data
            .access_patterns
            .values()
            .filter(|p| p.last_accessed.map_or(false, |t| t > thirty_days_ago))
            .count();

        // Calculate growth rate
        let growth_rate = self.calculate_growth_rate();

        // Get top memories
        let mut top_memories: Vec<_> = self
            .data
            .access_patterns
            .iter()
            .map(|(id, pattern)| (id.clone(), pattern.access_count))
            .collect();
        top_memories.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        top_memories.truncate(10);

        // Calculate health score
        let health_score = self.calculate_health_score();

        MemoryMetrics {
            total_memories: self.data.total_memories,
            active_memories,
            growth_rate,
            top_memories,
            health_score,
        }
    }

    /// Analyze trends
    pub fn analyze_trends(&self) -> TrendAnalysis {
        debug!("Analyzing memory trends");

        let memory_trend = self.calculate_trend(&self.time_series.daily_memories);
        let query_trend = self.calculate_trend(&self.time_series.daily_queries);
        let quality_trend = self.calculate_quality_trend();

        TrendAnalysis {
            memory_growth_trend: memory_trend,
            query_volume_trend: query_trend,
            quality_trend,
            predictions: if self.config.enable_predictions {
                Some(self.generate_predictions())
            } else {
                None
            },
        }
    }

    /// Detect anomalies
    pub fn detect_anomalies(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Check for sudden drops in quality
        if let Some(anomaly) = self.detect_quality_anomaly() {
            anomalies.push(anomaly);
        }

        // Check for unusual access patterns
        anomalies.extend(self.detect_access_anomalies());

        // Check for performance issues
        if let Some(anomaly) = self.detect_performance_anomaly() {
            anomalies.push(anomaly);
        }

        anomalies
    }

    // Private helper methods

    fn extract_query_patterns(&mut self, query: &str) {
        let patterns = vec![
            ("how to", "How-to questions"),
            ("what is", "Definition questions"),
            ("why", "Explanation questions"),
            ("error", "Error-related queries"),
            ("best practice", "Best practice queries"),
        ];

        let query_lower = query.to_lowercase();
        for (pattern, _) in patterns {
            if query_lower.contains(pattern) {
                // Update common patterns
                let found = self
                    .data
                    .query_stats
                    .common_patterns
                    .iter_mut()
                    .find(|(p, _)| p == pattern);

                if let Some((_, count)) = found {
                    *count += 1;
                } else {
                    self.data
                        .query_stats
                        .common_patterns
                        .push((pattern.to_string(), 1));
                }

                // Keep only top 10
                self.data
                    .query_stats
                    .common_patterns
                    .sort_by_key(|(_, c)| std::cmp::Reverse(*c));
                self.data.query_stats.common_patterns.truncate(10);

                break;
            }
        }
    }

    fn calculate_growth_rate(&self) -> f32 {
        if self.time_series.daily_memories.len() < 2 {
            return 0.0;
        }

        let values: Vec<f32> = self
            .time_series
            .daily_memories
            .values()
            .rev()
            .take(7)
            .map(|&v| v as f32)
            .collect();

        if values.len() < 2 {
            return 0.0;
        }

        // Simple linear regression
        let n = values.len() as f32;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f32>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f32;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }

    fn calculate_health_score(&self) -> f32 {
        let mut score = 50.0; // Base score

        // Quality component (0-25 points)
        score += self.data.quality_metrics.overall_quality * 25.0;

        // Performance component (0-15 points)
        if self.data.performance_metrics.avg_search_latency < 100.0 {
            score += 15.0;
        } else if self.data.performance_metrics.avg_search_latency < 500.0 {
            score += 10.0;
        } else if self.data.performance_metrics.avg_search_latency < 1000.0 {
            score += 5.0;
        }

        // Activity component (0-10 points)
        let success_rate = if self.data.query_stats.total_queries > 0 {
            self.data.query_stats.successful_queries as f32
                / self.data.query_stats.total_queries as f32
        } else {
            0.0
        };
        score += success_rate * 10.0;

        score.min(100.0)
    }

    fn calculate_trend(&self, data: &BTreeMap<String, usize>) -> Trend {
        if data.len() < self.config.min_trend_points {
            return Trend::Stable;
        }

        let growth_rate = self.calculate_growth_rate();

        if growth_rate > 0.1 {
            Trend::Increasing
        } else if growth_rate < -0.1 {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }

    fn calculate_quality_trend(&self) -> Trend {
        let values: Vec<f32> = self
            .time_series
            .daily_quality
            .values()
            .rev()
            .take(7)
            .copied()
            .collect();

        if values.len() < 2 {
            return Trend::Stable;
        }

        let first_avg = values[..values.len() / 2].iter().sum::<f32>() / (values.len() / 2) as f32;
        let second_avg = values[values.len() / 2..].iter().sum::<f32>()
            / (values.len() - values.len() / 2) as f32;

        if second_avg > first_avg * 1.05 {
            Trend::Increasing
        } else if second_avg < first_avg * 0.95 {
            Trend::Decreasing
        } else {
            Trend::Stable
        }
    }

    fn generate_predictions(&self) -> Predictions {
        // Simple predictions based on trends
        let memory_growth = self.calculate_growth_rate();
        let next_week_memories = (self.data.total_memories as f32 + memory_growth * 7.0) as usize;

        let query_trend = self.calculate_trend(&self.time_series.daily_queries);
        let avg_daily_queries = if !self.time_series.daily_queries.is_empty() {
            self.time_series.daily_queries.values().sum::<usize>()
                / self.time_series.daily_queries.len()
        } else {
            0
        };

        let next_week_queries = match query_trend {
            Trend::Increasing => (avg_daily_queries as f32 * 1.1 * 7.0) as usize,
            Trend::Decreasing => (avg_daily_queries as f32 * 0.9 * 7.0) as usize,
            Trend::Stable => avg_daily_queries * 7,
        };

        Predictions {
            next_week_memories,
            next_week_queries,
            recommended_actions: self.generate_recommendations(),
        }
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check quality
        if self.data.quality_metrics.overall_quality < 0.7 {
            recommendations
                .push("Consider improving memory quality through better embeddings".to_string());
        }

        // Check performance
        if self.data.performance_metrics.avg_search_latency > 500.0 {
            recommendations
                .push("Search latency is high - consider optimizing indexes".to_string());
        }

        // Check cache
        if self.data.performance_metrics.cache_hit_rate < 0.5 {
            recommendations.push("Low cache hit rate - consider increasing cache size".to_string());
        }

        recommendations
    }

    fn detect_quality_anomaly(&self) -> Option<Anomaly> {
        let recent_quality: Vec<f32> = self
            .time_series
            .daily_quality
            .values()
            .rev()
            .take(7)
            .copied()
            .collect();

        if recent_quality.len() < 3 {
            return None;
        }

        let mean = recent_quality.iter().sum::<f32>() / recent_quality.len() as f32;
        let variance = recent_quality
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>()
            / recent_quality.len() as f32;
        let std_dev = variance.sqrt();

        if let Some(&latest) = recent_quality.first() {
            if (latest - mean).abs() > self.config.anomaly_threshold * std_dev {
                return Some(Anomaly {
                    anomaly_type: AnomalyType::QualityDrop,
                    description: format!(
                        "Quality score dropped to {:.2} (normal range: {:.2} ± {:.2})",
                        latest, mean, std_dev
                    ),
                    severity: Severity::High,
                    timestamp: Utc::now(),
                });
            }
        }

        None
    }

    fn detect_access_anomalies(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Look for memories with sudden access spikes
        for (id, pattern) in &self.data.access_patterns {
            if pattern.trend > 2.0 {
                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::AccessSpike,
                    description: format!("Memory {} has unusually high access rate", id),
                    severity: Severity::Low,
                    timestamp: Utc::now(),
                });
            }
        }

        anomalies
    }

    fn detect_performance_anomaly(&self) -> Option<Anomaly> {
        if self.data.performance_metrics.avg_search_latency > 1000.0 {
            Some(Anomaly {
                anomaly_type: AnomalyType::PerformanceIssue,
                description: format!(
                    "Search latency is {:.0}ms (threshold: 1000ms)",
                    self.data.performance_metrics.avg_search_latency
                ),
                severity: Severity::Medium,
                timestamp: Utc::now(),
            })
        } else {
            None
        }
    }
}

/// Trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Memory growth trend
    pub memory_growth_trend: Trend,
    /// Query volume trend
    pub query_volume_trend: Trend,
    /// Quality trend
    pub quality_trend: Trend,
    /// Predictions if enabled
    pub predictions: Option<Predictions>,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Increasing,
    Stable,
    Decreasing,
}

impl fmt::Display for Trend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trend::Increasing => write!(f, "↑"),
            Trend::Stable => write!(f, "→"),
            Trend::Decreasing => write!(f, "↓"),
        }
    }
}

/// Predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predictions {
    /// Predicted memories for next week
    pub next_week_memories: usize,
    /// Predicted queries for next week
    pub next_week_queries: usize,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Type of anomaly
    pub anomaly_type: AnomalyType,
    /// Description
    pub description: String,
    /// Severity
    pub severity: Severity,
    /// When detected
    pub timestamp: DateTime<Utc>,
}

/// Types of anomalies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    QualityDrop,
    AccessSpike,
    PerformanceIssue,
    DataCorruption,
}

/// Anomaly severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl InsightGenerator {
    /// Create a new insight generator
    pub fn new() -> Self {
        Self::with_config(InsightConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: InsightConfig) -> Self {
        Self { config }
    }

    /// Generate insights from patterns
    pub async fn from_patterns(&self, patterns: Vec<Pattern>) -> Result<Vec<MemoryInsight>> {
        let mut insights = Vec::new();

        // Find high-frequency patterns
        let high_freq_patterns: Vec<_> = patterns
            .iter()
            .filter(|p| p.frequency > 10 && p.confidence >= self.config.min_confidence)
            .collect();

        if !high_freq_patterns.is_empty() {
            insights.push(MemoryInsight {
                insight_type: InsightType::PatternDiscovered,
                description: format!(
                    "Discovered {} high-frequency patterns that could be automated",
                    high_freq_patterns.len()
                ),
                confidence: 0.9,
                supporting_evidence: high_freq_patterns
                    .iter()
                    .map(|p| p.template.clone())
                    .take(3)
                    .collect(),
                recommendations: vec![
                    "Consider creating templates for common patterns".to_string(),
                    "Automate responses for repetitive queries".to_string(),
                ],
            });
        }

        // Find patterns with low confidence
        let low_conf_patterns: Vec<_> = patterns.iter().filter(|p| p.confidence < 0.6).collect();

        if low_conf_patterns.len() > 5 {
            insights.push(MemoryInsight {
                insight_type: InsightType::OptimizationOpportunity,
                description: "Several patterns have low confidence scores".to_string(),
                confidence: 0.8,
                supporting_evidence: vec![format!(
                    "{} patterns below 60% confidence",
                    low_conf_patterns.len()
                )],
                recommendations: vec![
                    "Review and refine low-confidence patterns".to_string(),
                    "Collect more training examples".to_string(),
                ],
            });
        }

        insights.truncate(self.config.max_insights);
        Ok(insights)
    }

    /// Generate insights from knowledge graph
    pub async fn from_graph(&self, graph: &KnowledgeGraph) -> Result<Vec<MemoryInsight>> {
        let mut insights = Vec::new();

        // Analyze graph connectivity
        let stats = graph.stats().clone();

        if stats.avg_connections < 1.5 {
            insights.push(MemoryInsight {
                insight_type: InsightType::KnowledgeGap,
                description: "Knowledge graph has low connectivity".to_string(),
                confidence: 0.85,
                supporting_evidence: vec![format!(
                    "Average connections per entity: {:.1}",
                    stats.avg_connections
                )],
                recommendations: vec![
                    "Extract more relationships between entities".to_string(),
                    "Improve entity linking in new conversations".to_string(),
                ],
            });
        }

        // Find knowledge hubs
        if !stats.hubs.is_empty() {
            let hub_names: Vec<String> = stats
                .hubs
                .iter()
                .take(3)
                .map(|(name, count)| format!("{} ({} connections)", name, count))
                .collect();

            insights.push(MemoryInsight {
                insight_type: InsightType::TrendIdentified,
                description: "Identified key knowledge hubs in the system".to_string(),
                confidence: 0.9,
                supporting_evidence: hub_names,
                recommendations: vec![
                    "Focus on enriching content around these hubs".to_string(),
                    "Create specialized retrieval for hub topics".to_string(),
                ],
            });
        }

        Ok(insights)
    }

    /// Generate insights from analytics
    pub async fn from_analytics(&self, analyzer: &MemoryAnalyzer) -> Result<Vec<MemoryInsight>> {
        let mut insights = Vec::new();

        // Check for anomalies
        let anomalies = analyzer.detect_anomalies();
        for anomaly in anomalies
            .iter()
            .filter(|a| a.severity as u8 >= Severity::Medium as u8)
        {
            insights.push(MemoryInsight {
                insight_type: InsightType::AnomalyDetected,
                description: anomaly.description.clone(),
                confidence: 0.95,
                supporting_evidence: vec![format!("{:?} anomaly detected", anomaly.anomaly_type)],
                recommendations: vec![
                    "Investigate the cause of the anomaly".to_string(),
                    "Check system logs for related issues".to_string(),
                ],
            });
        }

        // Analyze trends
        let trends = analyzer.analyze_trends();
        if trends.memory_growth_trend == Trend::Decreasing {
            insights.push(MemoryInsight {
                insight_type: InsightType::TrendIdentified,
                description: "Memory growth is slowing down".to_string(),
                confidence: 0.8,
                supporting_evidence: vec!["Negative growth trend detected".to_string()],
                recommendations: vec![
                    "Encourage more knowledge capture".to_string(),
                    "Review conversation quality".to_string(),
                ],
            });
        }

        Ok(insights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_analyzer() -> Result<()> {
        let mut analyzer = MemoryAnalyzer::new();

        // Record some test data
        analyzer.record_memory_added(Some("rust")).await?;
        analyzer.record_memory_added(Some("python")).await?;

        let metrics = analyzer.get_metrics();
        assert_eq!(metrics.total_memories, 2);

        Ok(())
    }

    #[test]
    fn test_trend_calculation() {
        let analyzer = MemoryAnalyzer::new();
        let trend = analyzer.calculate_trend(&BTreeMap::new());
        assert_eq!(trend, Trend::Stable);
    }
}
