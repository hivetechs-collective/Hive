/// Model Performance Tracking
/// 
/// Comprehensive performance monitoring system for OpenRouter models with
/// latency tracking, success rates, quality metrics, and intelligent fallback.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance metrics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub model_id: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f32,
    pub average_latency_ms: f32,
    pub p50_latency_ms: f32,
    pub p95_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub average_tokens_per_second: f32,
    pub error_rate: f32,
    pub timeout_rate: f32,
    pub quality_score: f32, // 0.0 - 1.0, based on user feedback
    pub last_updated: DateTime<Utc>,
}

/// Model performance entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEntry {
    pub id: String,
    pub model_id: String,
    pub timestamp: DateTime<Utc>,
    pub latency_ms: u64,
    pub tokens_generated: u32,
    pub tokens_per_second: f32,
    pub success: bool,
    pub error_type: Option<ErrorType>,
    pub quality_rating: Option<f32>, // User feedback if available
    pub request_type: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Error types for categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    Timeout,
    RateLimit,
    InvalidRequest,
    ModelUnavailable,
    InsufficientCredits,
    NetworkError,
    Other(String),
}

/// Performance window for rolling metrics
#[derive(Debug, Clone)]
pub struct PerformanceWindow {
    pub duration_minutes: u32,
    pub entries: VecDeque<PerformanceEntry>,
}

/// Model health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    pub model_id: String,
    pub status: HealthStatus,
    pub issues: Vec<String>,
    pub recommendation: Option<String>,
}

/// Health status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unavailable,
}

/// Performance comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub model_a: String,
    pub model_b: String,
    pub latency_difference_ms: f32,
    pub success_rate_difference: f32,
    pub cost_difference: f32,
    pub quality_difference: f32,
    pub recommendation: String,
}

/// Fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub primary_model: String,
    pub fallback_chain: Vec<String>,
    pub trigger_conditions: FallbackTriggers,
    pub max_retries_per_model: u32,
}

/// Conditions that trigger fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackTriggers {
    pub error_rate_threshold: f32,
    pub latency_threshold_ms: u64,
    pub consecutive_failures: u32,
    pub timeout_threshold_ms: u64,
}

/// Scoring weights for model ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub quality: f32,
    pub reliability: f32,
    pub speed: f32,
    pub cost: f32,
    pub throughput: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            quality: 0.3,
            reliability: 0.25,
            speed: 0.2,
            cost: 0.15,
            throughput: 0.1,
        }
    }
}

/// Model ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRanking {
    pub model_id: String,
    pub score: f32,
    pub rank: u32,
    pub task_type: String,
    pub metrics_snapshot: PerformanceMetrics,
    pub last_updated: DateTime<Utc>,
}

/// Task-specific model recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecommendation {
    pub task_type: String,
    pub recommended_models: Vec<ModelRanking>,
    pub reasoning: String,
    pub confidence: f32,
}

/// A/B Test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    pub test_id: String,
    pub name: String,
    pub description: String,
    pub model_a: String,
    pub model_b: String,
    pub sample_size: usize,
    pub test_queries: Vec<String>,
    pub metrics_to_compare: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub duration_hours: u32,
    pub status: ABTestStatus,
}

/// A/B Test status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ABTestStatus {
    Planned,
    Running,
    Completed,
    Paused,
    Cancelled,
}

/// A/B Test result for a single query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    pub test_id: String,
    pub query_id: String,
    pub model_id: String,
    pub latency_ms: u64,
    pub tokens_generated: u32,
    pub success: bool,
    pub error_type: Option<ErrorType>,
    pub quality_rating: Option<f32>,
    pub timestamp: DateTime<Utc>,
}

/// A/B Test statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestAnalysis {
    pub test_id: String,
    pub model_a: String,
    pub model_b: String,
    pub sample_size_a: usize,
    pub sample_size_b: usize,
    pub metrics_comparison: ABTestMetricsComparison,
    pub statistical_significance: StatisticalSignificance,
    pub recommendation: String,
    pub confidence_level: f32,
    pub completed_at: DateTime<Utc>,
}

/// Comparison of metrics between models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestMetricsComparison {
    pub latency_comparison: MetricComparison,
    pub success_rate_comparison: MetricComparison,
    pub quality_comparison: MetricComparison,
    pub throughput_comparison: MetricComparison,
}

/// Statistical comparison for a specific metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric_name: String,
    pub model_a_value: f32,
    pub model_b_value: f32,
    pub difference: f32,
    pub percentage_change: f32,
    pub better_model: Option<String>,
}

/// Statistical significance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSignificance {
    pub is_significant: bool,
    pub p_value: f32,
    pub confidence_interval: (f32, f32),
    pub effect_size: f32,
    pub power: f32,
}

impl Default for FallbackTriggers {
    fn default() -> Self {
        Self {
            error_rate_threshold: 0.2,     // 20% error rate
            latency_threshold_ms: 10000,    // 10 seconds
            consecutive_failures: 3,
            timeout_threshold_ms: 30000,    // 30 seconds
        }
    }
}

/// Circuit breaker state for model fallback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub model_id: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub last_failure: Option<DateTime<Utc>>,
    pub next_attempt: Option<DateTime<Utc>>,
    pub success_count_after_recovery: u32,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Circuit broken, using fallback
    HalfOpen,  // Testing if circuit can be closed
}

/// Fallback execution result
#[derive(Debug, Clone)]
pub struct FallbackResult {
    pub attempted_models: Vec<String>,
    pub successful_model: Option<String>,
    pub total_attempts: u32,
    pub total_latency_ms: u64,
    pub errors: Vec<(String, ErrorType)>,
}

/// Model performance tracker
pub struct PerformanceTracker {
    metrics: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    windows: Arc<RwLock<HashMap<String, PerformanceWindow>>>,
    fallback_configs: Arc<RwLock<HashMap<String, FallbackConfig>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
    ab_tests: Arc<RwLock<HashMap<String, ABTestConfig>>>,
    ab_results: Arc<RwLock<HashMap<String, Vec<ABTestResult>>>>,
    window_duration_minutes: u32,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new(window_duration_minutes: u32) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            windows: Arc::new(RwLock::new(HashMap::new())),
            fallback_configs: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            ab_tests: Arc::new(RwLock::new(HashMap::new())),
            ab_results: Arc::new(RwLock::new(HashMap::new())),
            window_duration_minutes,
        }
    }

    /// Track a performance entry
    pub async fn track_performance(
        &self,
        model_id: &str,
        latency_ms: u64,
        tokens_generated: u32,
        success: bool,
        error_type: Option<ErrorType>,
        quality_rating: Option<f32>,
        request_type: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let tokens_per_second = if latency_ms > 0 {
            (tokens_generated as f32 * 1000.0) / latency_ms as f32
        } else {
            0.0
        };

        let entry = PerformanceEntry {
            id: uuid::Uuid::new_v4().to_string(),
            model_id: model_id.to_string(),
            timestamp: Utc::now(),
            latency_ms,
            tokens_generated,
            tokens_per_second,
            success,
            error_type,
            quality_rating,
            request_type: request_type.to_string(),
            metadata,
        };

        // Add to window
        {
            let mut windows = self.windows.write().await;
            let window = windows
                .entry(model_id.to_string())
                .or_insert_with(|| PerformanceWindow {
                    duration_minutes: self.window_duration_minutes,
                    entries: VecDeque::new(),
                });

            window.entries.push_back(entry.clone());

            // Remove old entries
            let cutoff = Utc::now() - chrono::Duration::minutes(self.window_duration_minutes as i64);
            while let Some(front) = window.entries.front() {
                if front.timestamp < cutoff {
                    window.entries.pop_front();
                } else {
                    break;
                }
            }
        }

        // Update metrics
        self.update_metrics(model_id).await?;

        // Check for fallback triggers
        if !success {
            self.check_fallback_triggers(model_id).await?;
        }

        Ok(())
    }

    /// Update metrics for a model
    async fn update_metrics(&self, model_id: &str) -> Result<()> {
        let windows = self.windows.read().await;
        
        if let Some(window) = windows.get(model_id) {
            let entries: Vec<&PerformanceEntry> = window.entries.iter().collect();
            
            if entries.is_empty() {
                return Ok(());
            }

            let total_requests = entries.len() as u64;
            let successful_requests = entries.iter().filter(|e| e.success).count() as u64;
            let failed_requests = total_requests - successful_requests;
            let success_rate = successful_requests as f32 / total_requests as f32;

            // Calculate latency metrics
            let mut latencies: Vec<u64> = entries
                .iter()
                .filter(|e| e.success)
                .map(|e| e.latency_ms)
                .collect();
            
            if latencies.is_empty() {
                return Ok(());
            }

            latencies.sort_unstable();
            
            let average_latency_ms = latencies.iter().sum::<u64>() as f32 / latencies.len() as f32;
            let p50_latency_ms = percentile(&latencies, 0.5) as f32;
            let p95_latency_ms = percentile(&latencies, 0.95) as f32;
            let p99_latency_ms = percentile(&latencies, 0.99) as f32;

            // Calculate tokens per second
            let total_tokens_per_second: f32 = entries
                .iter()
                .filter(|e| e.success)
                .map(|e| e.tokens_per_second)
                .sum();
            let average_tokens_per_second = total_tokens_per_second / successful_requests as f32;

            // Calculate error rates
            let timeout_count = entries
                .iter()
                .filter(|e| matches!(e.error_type, Some(ErrorType::Timeout)))
                .count() as f32;
            let timeout_rate = timeout_count / total_requests as f32;
            let error_rate = failed_requests as f32 / total_requests as f32;

            // Calculate quality score
            let quality_ratings: Vec<f32> = entries
                .iter()
                .filter_map(|e| e.quality_rating)
                .collect();
            
            let quality_score = if quality_ratings.is_empty() {
                // Estimate based on success rate and latency
                let latency_score = 1.0 - (average_latency_ms / 10000.0).min(1.0);
                success_rate * 0.7 + latency_score * 0.3
            } else {
                quality_ratings.iter().sum::<f32>() / quality_ratings.len() as f32
            };

            let metrics = PerformanceMetrics {
                model_id: model_id.to_string(),
                total_requests,
                successful_requests,
                failed_requests,
                success_rate,
                average_latency_ms,
                p50_latency_ms,
                p95_latency_ms,
                p99_latency_ms,
                average_tokens_per_second,
                error_rate,
                timeout_rate,
                quality_score,
                last_updated: Utc::now(),
            };

            let mut all_metrics = self.metrics.write().await;
            all_metrics.insert(model_id.to_string(), metrics);
        }

        Ok(())
    }

    /// Get performance metrics for a model
    pub async fn get_metrics(&self, model_id: &str) -> Option<PerformanceMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(model_id).cloned()
    }

    /// Get all performance metrics
    pub async fn get_all_metrics(&self) -> Vec<PerformanceMetrics> {
        let metrics = self.metrics.read().await;
        metrics.values().cloned().collect()
    }

    /// Get model health status
    pub async fn get_model_health(&self, model_id: &str) -> Result<ModelHealth> {
        let metrics = self.get_metrics(model_id).await;
        
        let (status, mut issues, recommendation) = if let Some(m) = metrics {
            let mut issues = Vec::new();
            let mut status = HealthStatus::Healthy;

            // Check success rate
            if m.success_rate < 0.5 {
                status = HealthStatus::Unavailable;
                issues.push(format!("Critical: Success rate {:.1}%", m.success_rate * 100.0));
            } else if m.success_rate < 0.8 {
                status = HealthStatus::Unhealthy;
                issues.push(format!("Low success rate: {:.1}%", m.success_rate * 100.0));
            } else if m.success_rate < 0.95 {
                status = HealthStatus::Degraded;
                issues.push(format!("Degraded success rate: {:.1}%", m.success_rate * 100.0));
            }

            // Check latency
            if m.p95_latency_ms > 10000.0 {
                if status == HealthStatus::Healthy {
                    status = HealthStatus::Degraded;
                }
                issues.push(format!("High latency: P95 = {:.0}ms", m.p95_latency_ms));
            }

            // Check timeout rate
            if m.timeout_rate > 0.1 {
                if status == HealthStatus::Healthy {
                    status = HealthStatus::Degraded;
                }
                issues.push(format!("High timeout rate: {:.1}%", m.timeout_rate * 100.0));
            }

            let recommendation = match status {
                HealthStatus::Unavailable => Some("Consider switching to fallback model immediately".to_string()),
                HealthStatus::Unhealthy => Some("Monitor closely and prepare fallback".to_string()),
                HealthStatus::Degraded => Some("Performance degraded, consider load balancing".to_string()),
                HealthStatus::Healthy => None,
            };

            (status, issues, recommendation)
        } else {
            (
                HealthStatus::Unavailable,
                vec!["No performance data available".to_string()],
                Some("Model has not been used yet".to_string()),
            )
        };

        Ok(ModelHealth {
            model_id: model_id.to_string(),
            status,
            issues,
            recommendation,
        })
    }

    /// Compare two models
    pub async fn compare_models(
        &self,
        model_a: &str,
        model_b: &str,
    ) -> Result<PerformanceComparison> {
        let metrics_a = self.get_metrics(model_a).await
            .ok_or_else(|| anyhow::anyhow!("No metrics for model {}", model_a))?;
        let metrics_b = self.get_metrics(model_b).await
            .ok_or_else(|| anyhow::anyhow!("No metrics for model {}", model_b))?;

        let latency_difference_ms = metrics_a.average_latency_ms - metrics_b.average_latency_ms;
        let success_rate_difference = metrics_a.success_rate - metrics_b.success_rate;
        let quality_difference = metrics_a.quality_score - metrics_b.quality_score;

        // Cost difference would come from the cost tracker
        let cost_difference = 0.0; // Placeholder

        let recommendation = if quality_difference > 0.1 && latency_difference_ms < 1000.0 {
            format!("{} offers better quality with similar latency", model_a)
        } else if quality_difference < -0.1 && latency_difference_ms > -1000.0 {
            format!("{} offers better quality with similar latency", model_b)
        } else if latency_difference_ms < -2000.0 && success_rate_difference > -0.05 {
            format!("{} is significantly faster with similar reliability", model_a)
        } else if latency_difference_ms > 2000.0 && success_rate_difference < 0.05 {
            format!("{} is significantly faster with similar reliability", model_b)
        } else {
            "Both models offer similar performance".to_string()
        };

        Ok(PerformanceComparison {
            model_a: model_a.to_string(),
            model_b: model_b.to_string(),
            latency_difference_ms,
            success_rate_difference,
            cost_difference,
            quality_difference,
            recommendation,
        })
    }

    /// Configure fallback for a model
    pub async fn configure_fallback(
        &self,
        primary_model: &str,
        fallback_chain: Vec<String>,
        triggers: Option<FallbackTriggers>,
    ) -> Result<()> {
        let config = FallbackConfig {
            primary_model: primary_model.to_string(),
            fallback_chain,
            trigger_conditions: triggers.unwrap_or_default(),
            max_retries_per_model: 2,
        };

        let mut configs = self.fallback_configs.write().await;
        configs.insert(primary_model.to_string(), config);

        Ok(())
    }

    /// Get fallback recommendation
    pub async fn get_fallback_recommendation(&self, model_id: &str) -> Result<Option<String>> {
        let health = self.get_model_health(model_id).await?;
        
        if health.status == HealthStatus::Healthy {
            return Ok(None);
        }

        let configs = self.fallback_configs.read().await;
        if let Some(config) = configs.get(model_id) {
            if !config.fallback_chain.is_empty() {
                // Find the first healthy fallback
                for fallback in &config.fallback_chain {
                    let fallback_health = self.get_model_health(fallback).await?;
                    if fallback_health.status == HealthStatus::Healthy {
                        return Ok(Some(fallback.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Check if fallback should be triggered
    async fn check_fallback_triggers(&self, model_id: &str) -> Result<()> {
        let configs = self.fallback_configs.read().await;
        
        if let Some(config) = configs.get(model_id) {
            if let Some(metrics) = self.get_metrics(model_id).await {
                let mut should_trigger = false;
                let mut reasons = Vec::new();

                if metrics.error_rate > config.trigger_conditions.error_rate_threshold {
                    should_trigger = true;
                    reasons.push(format!(
                        "Error rate {:.1}% exceeds threshold {:.1}%",
                        metrics.error_rate * 100.0,
                        config.trigger_conditions.error_rate_threshold * 100.0
                    ));
                }

                if metrics.average_latency_ms > config.trigger_conditions.latency_threshold_ms as f32 {
                    should_trigger = true;
                    reasons.push(format!(
                        "Average latency {:.0}ms exceeds threshold {}ms",
                        metrics.average_latency_ms,
                        config.trigger_conditions.latency_threshold_ms
                    ));
                }

                if should_trigger {
                    log::warn!(
                        "Fallback triggered for model {}: {}",
                        model_id,
                        reasons.join(", ")
                    );
                }
            }
        }

        Ok(())
    }

    /// Get performance ranking with customizable scoring
    pub async fn get_performance_ranking(&self) -> Vec<(String, f32)> {
        self.get_task_specific_ranking("general", &ScoringWeights::default()).await
    }

    /// Get task-specific performance ranking
    pub async fn get_task_specific_ranking(
        &self, 
        task_type: &str, 
        weights: &ScoringWeights
    ) -> Vec<(String, f32)> {
        let metrics = self.metrics.read().await;
        
        let mut rankings: Vec<(String, f32)> = metrics
            .iter()
            .map(|(model_id, m)| {
                let score = self.calculate_model_score(m, task_type, weights);
                (model_id.clone(), score)
            })
            .collect();

        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }

    /// Calculate comprehensive model score
    fn calculate_model_score(&self, metrics: &PerformanceMetrics, task_type: &str, weights: &ScoringWeights) -> f32 {
        // Normalize latency score (lower is better)
        let latency_score = 1.0 - (metrics.average_latency_ms / 15000.0).min(1.0);
        
        // Success rate score
        let reliability_score = metrics.success_rate;
        
        // Quality score
        let quality_score = metrics.quality_score;
        
        // Cost efficiency score (placeholder - would integrate with cost tracker)
        let cost_score = 0.8; // Default good value
        
        // Throughput score (tokens per second)
        let throughput_score = (metrics.average_tokens_per_second / 100.0).min(1.0);
        
        // Task-specific adjustments
        let task_multiplier = match task_type {
            "code-analysis" => {
                // Favor quality and reliability for code analysis
                quality_score * 0.5 + reliability_score * 0.3 + latency_score * 0.2
            },
            "creative-writing" => {
                // Favor quality and throughput for creative tasks
                quality_score * 0.6 + throughput_score * 0.3 + reliability_score * 0.1
            },
            "quick-response" => {
                // Favor speed and reliability for quick responses
                latency_score * 0.5 + reliability_score * 0.3 + quality_score * 0.2
            },
            "cost-efficient" => {
                // Favor cost efficiency
                cost_score * 0.4 + reliability_score * 0.3 + quality_score * 0.3
            },
            _ => {
                // General balanced scoring
                quality_score * weights.quality 
                    + reliability_score * weights.reliability
                    + latency_score * weights.speed
                    + cost_score * weights.cost
                    + throughput_score * weights.throughput
            }
        };
        
        task_multiplier.max(0.0).min(1.0)
    }

    /// Get model recommendations for a specific task
    pub async fn get_task_recommendations(&self, task_type: &str, limit: usize) -> Result<TaskRecommendation> {
        let weights = match task_type {
            "code-analysis" => ScoringWeights {
                quality: 0.4,
                reliability: 0.3,
                speed: 0.15,
                cost: 0.1,
                throughput: 0.05,
            },
            "creative-writing" => ScoringWeights {
                quality: 0.45,
                reliability: 0.1,
                speed: 0.15,
                cost: 0.1,
                throughput: 0.2,
            },
            "quick-response" => ScoringWeights {
                quality: 0.2,
                reliability: 0.3,
                speed: 0.4,
                cost: 0.05,
                throughput: 0.05,
            },
            "cost-efficient" => ScoringWeights {
                quality: 0.2,
                reliability: 0.25,
                speed: 0.15,
                cost: 0.35,
                throughput: 0.05,
            },
            _ => ScoringWeights::default(),
        };

        let rankings = self.get_task_specific_ranking(task_type, &weights).await;
        let metrics = self.metrics.read().await;
        
        let recommended_models: Vec<ModelRanking> = rankings
            .iter()
            .take(limit)
            .enumerate()
            .filter_map(|(index, (model_id, score))| {
                metrics.get(model_id).map(|m| ModelRanking {
                    model_id: model_id.clone(),
                    score: *score,
                    rank: index as u32 + 1,
                    task_type: task_type.to_string(),
                    metrics_snapshot: m.clone(),
                    last_updated: Utc::now(),
                })
            })
            .collect();

        let reasoning = format!(
            "Recommendations based on {} performance data points, optimized for {} with emphasis on {}",
            recommended_models.len(),
            task_type,
            match task_type {
                "code-analysis" => "quality and reliability",
                "creative-writing" => "quality and throughput",
                "quick-response" => "speed and reliability",
                "cost-efficient" => "cost efficiency",
                _ => "balanced performance"
            }
        );

        let confidence = if recommended_models.is_empty() {
            0.0
        } else if recommended_models[0].metrics_snapshot.total_requests < 5 {
            0.3  // Low confidence with limited data
        } else if recommended_models[0].metrics_snapshot.total_requests < 20 {
            0.7  // Medium confidence
        } else {
            0.9  // High confidence with substantial data
        };

        Ok(TaskRecommendation {
            task_type: task_type.to_string(),
            recommended_models,
            reasoning,
            confidence,
        })
    }

    /// Get detailed model rankings for all task types
    pub async fn get_comprehensive_rankings(&self) -> HashMap<String, Vec<ModelRanking>> {
        let mut rankings = HashMap::new();
        let task_types = vec![
            "general",
            "code-analysis", 
            "creative-writing",
            "quick-response",
            "cost-efficient"
        ];

        let metrics = self.metrics.read().await;
        
        for task_type in task_types {
            let weights = match task_type {
                "code-analysis" => ScoringWeights {
                    quality: 0.4, reliability: 0.3, speed: 0.15, cost: 0.1, throughput: 0.05,
                },
                "creative-writing" => ScoringWeights {
                    quality: 0.45, reliability: 0.1, speed: 0.15, cost: 0.1, throughput: 0.2,
                },
                "quick-response" => ScoringWeights {
                    quality: 0.2, reliability: 0.3, speed: 0.4, cost: 0.05, throughput: 0.05,
                },
                "cost-efficient" => ScoringWeights {
                    quality: 0.2, reliability: 0.25, speed: 0.15, cost: 0.35, throughput: 0.05,
                },
                _ => ScoringWeights::default(),
            };

            let task_rankings = self.get_task_specific_ranking(task_type, &weights).await;
            
            let model_rankings: Vec<ModelRanking> = task_rankings
                .iter()
                .enumerate()
                .filter_map(|(index, (model_id, score))| {
                    metrics.get(model_id).map(|m| ModelRanking {
                        model_id: model_id.clone(),
                        score: *score,
                        rank: index as u32 + 1,
                        task_type: task_type.to_string(),
                        metrics_snapshot: m.clone(),
                        last_updated: Utc::now(),
                    })
                })
                .collect();

            rankings.insert(task_type.to_string(), model_rankings);
        }

        rankings
    }

    /// Get top performing models across all metrics
    pub async fn get_top_performers(&self, limit: usize) -> Vec<ModelRanking> {
        let rankings = self.get_performance_ranking().await;
        let metrics = self.metrics.read().await;
        
        rankings
            .iter()
            .take(limit)
            .enumerate()
            .filter_map(|(index, (model_id, score))| {
                metrics.get(model_id).map(|m| ModelRanking {
                    model_id: model_id.clone(),
                    score: *score,
                    rank: index as u32 + 1,
                    task_type: "general".to_string(),
                    metrics_snapshot: m.clone(),
                    last_updated: Utc::now(),
                })
            })
            .collect()
    }

    /// Execute request with automatic fallback
    pub async fn execute_with_fallback<F, T, E>(
        &self,
        primary_model: &str,
        request_fn: F,
    ) -> Result<(T, FallbackResult)>
    where
        F: Fn(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send>>,
        E: std::fmt::Debug,
    {
        let mut fallback_result = FallbackResult {
            attempted_models: Vec::new(),
            successful_model: None,
            total_attempts: 0,
            total_latency_ms: 0,
            errors: Vec::new(),
        };

        // Check if primary model is available (circuit breaker state)
        let models_to_try = self.get_fallback_chain(primary_model).await?;
        
        for model_id in models_to_try {
            fallback_result.attempted_models.push(model_id.clone());
            fallback_result.total_attempts += 1;

            // Check circuit breaker state
            if !self.is_model_available(&model_id).await {
                fallback_result.errors.push((
                    model_id.clone(),
                    ErrorType::Other("Circuit breaker open".to_string())
                ));
                continue;
            }

            let start_time = std::time::Instant::now();
            
            match request_fn(&model_id).await {
                Ok(result) => {
                    let latency_ms = start_time.elapsed().as_millis() as u64;
                    fallback_result.total_latency_ms += latency_ms;
                    fallback_result.successful_model = Some(model_id.clone());
                    
                    // Record success
                    self.record_circuit_breaker_success(&model_id).await?;
                    
                    return Ok((result, fallback_result));
                }
                Err(error) => {
                    let latency_ms = start_time.elapsed().as_millis() as u64;
                    fallback_result.total_latency_ms += latency_ms;
                    
                    let error_type = self.classify_error(&format!("{:?}", error));
                    fallback_result.errors.push((model_id.clone(), error_type.clone()));
                    
                    // Record failure and update circuit breaker
                    self.record_circuit_breaker_failure(&model_id, error_type).await?;
                }
            }
        }

        anyhow::bail!(
            "All fallback models failed. Attempts: {} Errors: {:?}",
            fallback_result.total_attempts,
            fallback_result.errors
        )
    }

    /// Get the fallback chain for a model
    async fn get_fallback_chain(&self, primary_model: &str) -> Result<Vec<String>> {
        let configs = self.fallback_configs.read().await;
        
        if let Some(config) = configs.get(primary_model) {
            let mut chain = vec![primary_model.to_string()];
            chain.extend(config.fallback_chain.clone());
            Ok(chain)
        } else {
            // Default fallback chain based on performance ranking
            let rankings = self.get_performance_ranking().await;
            let mut chain = vec![primary_model.to_string()];
            
            // Add top 3 performing models as fallbacks
            for (model_id, _) in rankings.iter().take(3) {
                if model_id != primary_model {
                    chain.push(model_id.clone());
                }
            }
            
            Ok(chain)
        }
    }

    /// Check if model is available (circuit breaker state)
    async fn is_model_available(&self, model_id: &str) -> bool {
        let breakers = self.circuit_breakers.read().await;
        
        if let Some(breaker) = breakers.get(model_id) {
            match breaker.state {
                CircuitState::Closed => true,
                CircuitState::Open => {
                    // Check if we can transition to half-open
                    if let Some(next_attempt) = breaker.next_attempt {
                        Utc::now() >= next_attempt
                    } else {
                        false
                    }
                }
                CircuitState::HalfOpen => true, // Allow one test request
            }
        } else {
            true // No circuit breaker means available
        }
    }

    /// Record circuit breaker success
    async fn record_circuit_breaker_success(&self, model_id: &str) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().await;
        
        if let Some(breaker) = breakers.get_mut(model_id) {
            match breaker.state {
                CircuitState::HalfOpen => {
                    breaker.success_count_after_recovery += 1;
                    
                    // After 3 successful attempts, close the circuit
                    if breaker.success_count_after_recovery >= 3 {
                        breaker.state = CircuitState::Closed;
                        breaker.failure_count = 0;
                        breaker.success_count_after_recovery = 0;
                        breaker.last_failure = None;
                        breaker.next_attempt = None;
                        
                        log::info!("Circuit breaker closed for model: {}", model_id);
                    }
                }
                CircuitState::Closed => {
                    // Reset failure count on successful request
                    breaker.failure_count = 0;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Record circuit breaker failure
    async fn record_circuit_breaker_failure(&self, model_id: &str, error_type: ErrorType) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().await;
        
        let breaker = breakers.entry(model_id.to_string()).or_insert_with(|| {
            CircuitBreakerState {
                model_id: model_id.to_string(),
                state: CircuitState::Closed,
                failure_count: 0,
                last_failure: None,
                next_attempt: None,
                success_count_after_recovery: 0,
            }
        });

        breaker.failure_count += 1;
        breaker.last_failure = Some(Utc::now());

        // Open circuit after 5 consecutive failures
        if breaker.failure_count >= 5 && breaker.state == CircuitState::Closed {
            breaker.state = CircuitState::Open;
            breaker.next_attempt = Some(Utc::now() + chrono::Duration::minutes(5));
            
            log::warn!(
                "Circuit breaker opened for model: {} after {} failures. Error: {:?}",
                model_id,
                breaker.failure_count,
                error_type
            );
        } else if breaker.state == CircuitState::HalfOpen {
            // Failed during test, go back to open
            breaker.state = CircuitState::Open;
            breaker.next_attempt = Some(Utc::now() + chrono::Duration::minutes(10));
            breaker.success_count_after_recovery = 0;
            
            log::warn!(
                "Circuit breaker test failed for model: {}, staying open",
                model_id
            );
        }

        Ok(())
    }

    /// Classify error for circuit breaker logic
    fn classify_error(&self, error_msg: &str) -> ErrorType {
        let error_lower = error_msg.to_lowercase();
        
        if error_lower.contains("timeout") {
            ErrorType::Timeout
        } else if error_lower.contains("rate limit") || error_lower.contains("too many requests") {
            ErrorType::RateLimit
        } else if error_lower.contains("invalid") || error_lower.contains("bad request") {
            ErrorType::InvalidRequest
        } else if error_lower.contains("unavailable") || error_lower.contains("not found") {
            ErrorType::ModelUnavailable
        } else if error_lower.contains("credit") || error_lower.contains("quota") {
            ErrorType::InsufficientCredits
        } else if error_lower.contains("network") || error_lower.contains("connection") {
            ErrorType::NetworkError
        } else {
            ErrorType::Other(error_msg.to_string())
        }
    }

    /// Get circuit breaker status for all models
    pub async fn get_circuit_breaker_status(&self) -> Vec<CircuitBreakerState> {
        let breakers = self.circuit_breakers.read().await;
        breakers.values().cloned().collect()
    }

    /// Reset circuit breaker for a model
    pub async fn reset_circuit_breaker(&self, model_id: &str) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().await;
        
        if let Some(breaker) = breakers.get_mut(model_id) {
            breaker.state = CircuitState::Closed;
            breaker.failure_count = 0;
            breaker.last_failure = None;
            breaker.next_attempt = None;
            breaker.success_count_after_recovery = 0;
            
            log::info!("Circuit breaker manually reset for model: {}", model_id);
        }

        Ok(())
    }

    /// Transition circuit breaker to half-open for testing
    async fn transition_to_half_open(&self, model_id: &str) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().await;
        
        if let Some(breaker) = breakers.get_mut(model_id) {
            if breaker.state == CircuitState::Open {
                breaker.state = CircuitState::HalfOpen;
                breaker.success_count_after_recovery = 0;
                
                log::info!("Circuit breaker transitioned to half-open for model: {}", model_id);
            }
        }

        Ok(())
    }

    /// Create a new A/B test
    pub async fn create_ab_test(
        &self,
        name: String,
        description: String,
        model_a: String,
        model_b: String,
        test_queries: Vec<String>,
        sample_size: usize,
        duration_hours: u32,
    ) -> Result<String> {
        let test_id = uuid::Uuid::new_v4().to_string();
        
        let config = ABTestConfig {
            test_id: test_id.clone(),
            name: name.clone(),
            description,
            model_a,
            model_b,
            sample_size,
            test_queries,
            metrics_to_compare: vec![
                "latency".to_string(),
                "success_rate".to_string(),
                "quality".to_string(),
                "throughput".to_string(),
            ],
            started_at: Utc::now(),
            duration_hours,
            status: ABTestStatus::Planned,
        };

        let mut tests = self.ab_tests.write().await;
        tests.insert(test_id.clone(), config);

        let mut results = self.ab_results.write().await;
        results.insert(test_id.clone(), Vec::new());

        log::info!("Created A/B test: {} ({})", name, test_id);
        Ok(test_id)
    }

    /// Start an A/B test
    pub async fn start_ab_test(&self, test_id: &str) -> Result<()> {
        let mut tests = self.ab_tests.write().await;
        
        if let Some(test) = tests.get_mut(test_id) {
            test.status = ABTestStatus::Running;
            test.started_at = Utc::now();
            
            log::info!("Started A/B test: {}", test_id);
            Ok(())
        } else {
            anyhow::bail!("A/B test not found: {}", test_id)
        }
    }

    /// Record A/B test result
    pub async fn record_ab_test_result(
        &self,
        test_id: &str,
        query_id: &str,
        model_id: &str,
        latency_ms: u64,
        tokens_generated: u32,
        success: bool,
        error_type: Option<ErrorType>,
        quality_rating: Option<f32>,
    ) -> Result<()> {
        let result = ABTestResult {
            test_id: test_id.to_string(),
            query_id: query_id.to_string(),
            model_id: model_id.to_string(),
            latency_ms,
            tokens_generated,
            success,
            error_type,
            quality_rating,
            timestamp: Utc::now(),
        };

        let mut results = self.ab_results.write().await;
        if let Some(test_results) = results.get_mut(test_id) {
            test_results.push(result);
        } else {
            anyhow::bail!("A/B test not found: {}", test_id);
        }

        // Check if test should be completed
        self.check_ab_test_completion(test_id).await?;

        Ok(())
    }

    /// Check if A/B test should be completed
    async fn check_ab_test_completion(&self, test_id: &str) -> Result<()> {
        let tests = self.ab_tests.read().await;
        let results = self.ab_results.read().await;

        if let (Some(test), Some(test_results)) = (tests.get(test_id), results.get(test_id)) {
            if test.status == ABTestStatus::Running {
                let end_time = test.started_at + chrono::Duration::hours(test.duration_hours as i64);
                let has_enough_samples = test_results.len() >= test.sample_size;
                let time_expired = Utc::now() >= end_time;

                if has_enough_samples || time_expired {
                    drop(tests);
                    drop(results);
                    self.complete_ab_test(test_id).await?;
                }
            }
        }

        Ok(())
    }

    /// Complete an A/B test
    pub async fn complete_ab_test(&self, test_id: &str) -> Result<()> {
        let mut tests = self.ab_tests.write().await;
        
        if let Some(test) = tests.get_mut(test_id) {
            test.status = ABTestStatus::Completed;
            log::info!("Completed A/B test: {}", test_id);
        }

        Ok(())
    }

    /// Analyze A/B test results
    pub async fn analyze_ab_test(&self, test_id: &str) -> Result<ABTestAnalysis> {
        let tests = self.ab_tests.read().await;
        let results = self.ab_results.read().await;

        let test = tests.get(test_id)
            .ok_or_else(|| anyhow::anyhow!("A/B test not found: {}", test_id))?;

        let test_results = results.get(test_id)
            .ok_or_else(|| anyhow::anyhow!("A/B test results not found: {}", test_id))?;

        // Separate results by model
        let results_a: Vec<&ABTestResult> = test_results
            .iter()
            .filter(|r| r.model_id == test.model_a)
            .collect();
        
        let results_b: Vec<&ABTestResult> = test_results
            .iter()
            .filter(|r| r.model_id == test.model_b)
            .collect();

        if results_a.is_empty() || results_b.is_empty() {
            anyhow::bail!("Insufficient data for analysis");
        }

        // Calculate metrics for each model
        let metrics_a = self.calculate_ab_test_metrics(&results_a);
        let metrics_b = self.calculate_ab_test_metrics(&results_b);

        // Compare metrics
        let latency_comparison = MetricComparison {
            metric_name: "Average Latency (ms)".to_string(),
            model_a_value: metrics_a.avg_latency,
            model_b_value: metrics_b.avg_latency,
            difference: metrics_a.avg_latency - metrics_b.avg_latency,
            percentage_change: ((metrics_a.avg_latency - metrics_b.avg_latency) / metrics_b.avg_latency) * 100.0,
            better_model: if metrics_a.avg_latency < metrics_b.avg_latency {
                Some(test.model_a.clone())
            } else {
                Some(test.model_b.clone())
            },
        };

        let success_rate_comparison = MetricComparison {
            metric_name: "Success Rate (%)".to_string(),
            model_a_value: metrics_a.success_rate * 100.0,
            model_b_value: metrics_b.success_rate * 100.0,
            difference: (metrics_a.success_rate - metrics_b.success_rate) * 100.0,
            percentage_change: ((metrics_a.success_rate - metrics_b.success_rate) / metrics_b.success_rate) * 100.0,
            better_model: if metrics_a.success_rate > metrics_b.success_rate {
                Some(test.model_a.clone())
            } else {
                Some(test.model_b.clone())
            },
        };

        let quality_comparison = MetricComparison {
            metric_name: "Quality Score".to_string(),
            model_a_value: metrics_a.avg_quality,
            model_b_value: metrics_b.avg_quality,
            difference: metrics_a.avg_quality - metrics_b.avg_quality,
            percentage_change: ((metrics_a.avg_quality - metrics_b.avg_quality) / metrics_b.avg_quality) * 100.0,
            better_model: if metrics_a.avg_quality > metrics_b.avg_quality {
                Some(test.model_a.clone())
            } else {
                Some(test.model_b.clone())
            },
        };

        let throughput_comparison = MetricComparison {
            metric_name: "Throughput (tokens/sec)".to_string(),
            model_a_value: metrics_a.avg_throughput,
            model_b_value: metrics_b.avg_throughput,
            difference: metrics_a.avg_throughput - metrics_b.avg_throughput,
            percentage_change: ((metrics_a.avg_throughput - metrics_b.avg_throughput) / metrics_b.avg_throughput) * 100.0,
            better_model: if metrics_a.avg_throughput > metrics_b.avg_throughput {
                Some(test.model_a.clone())
            } else {
                Some(test.model_b.clone())
            },
        };

        // Statistical significance analysis (simplified)
        let statistical_significance = self.calculate_statistical_significance(&results_a, &results_b);

        // Generate recommendation
        let mut winning_metrics = 0;
        if latency_comparison.better_model.as_ref() == Some(&test.model_a) { winning_metrics += 1; }
        if success_rate_comparison.better_model.as_ref() == Some(&test.model_a) { winning_metrics += 1; }
        if quality_comparison.better_model.as_ref() == Some(&test.model_a) { winning_metrics += 1; }
        if throughput_comparison.better_model.as_ref() == Some(&test.model_a) { winning_metrics += 1; }

        let recommendation = if statistical_significance.is_significant {
            if winning_metrics >= 3 {
                format!("🎯 Recommend {} - significantly better across {} out of 4 metrics", test.model_a, winning_metrics)
            } else if winning_metrics <= 1 {
                format!("🎯 Recommend {} - significantly better across {} out of 4 metrics", test.model_b, 4 - winning_metrics)
            } else {
                "⚖️  Mixed results - consider task-specific selection or further testing".to_string()
            }
        } else {
            "📊 No statistically significant difference detected - either model acceptable".to_string()
        };

        let confidence_level = if statistical_significance.is_significant {
            0.95
        } else {
            statistical_significance.power
        };

        Ok(ABTestAnalysis {
            test_id: test_id.to_string(),
            model_a: test.model_a.clone(),
            model_b: test.model_b.clone(),
            sample_size_a: results_a.len(),
            sample_size_b: results_b.len(),
            metrics_comparison: ABTestMetricsComparison {
                latency_comparison,
                success_rate_comparison,
                quality_comparison,
                throughput_comparison,
            },
            statistical_significance,
            recommendation,
            confidence_level,
            completed_at: Utc::now(),
        })
    }

    /// Calculate metrics for A/B test results
    fn calculate_ab_test_metrics(&self, results: &[&ABTestResult]) -> ABTestMetrics {
        let successful_results: Vec<&ABTestResult> = results
            .iter()
            .filter(|r| r.success)
            .copied()
            .collect();

        let avg_latency = if successful_results.is_empty() {
            0.0
        } else {
            successful_results.iter().map(|r| r.latency_ms as f32).sum::<f32>() / successful_results.len() as f32
        };

        let success_rate = successful_results.len() as f32 / results.len() as f32;

        let avg_quality = successful_results
            .iter()
            .filter_map(|r| r.quality_rating)
            .sum::<f32>() / successful_results.len().max(1) as f32;

        let avg_throughput = if successful_results.is_empty() {
            0.0
        } else {
            successful_results
                .iter()
                .map(|r| if r.latency_ms > 0 { (r.tokens_generated as f32 * 1000.0) / r.latency_ms as f32 } else { 0.0 })
                .sum::<f32>() / successful_results.len() as f32
        };

        ABTestMetrics {
            avg_latency,
            success_rate,
            avg_quality,
            avg_throughput,
        }
    }

    /// Calculate statistical significance (simplified t-test)
    fn calculate_statistical_significance(&self, results_a: &[&ABTestResult], results_b: &[&ABTestResult]) -> StatisticalSignificance {
        // Simplified statistical analysis - in production would use proper statistical libraries
        let n_a = results_a.len() as f32;
        let n_b = results_b.len() as f32;

        if n_a < 5.0 || n_b < 5.0 {
            return StatisticalSignificance {
                is_significant: false,
                p_value: 1.0,
                confidence_interval: (0.0, 0.0),
                effect_size: 0.0,
                power: 0.1,
            };
        }

        // Use latency as primary metric for significance testing
        let latencies_a: Vec<f32> = results_a.iter().filter(|r| r.success).map(|r| r.latency_ms as f32).collect();
        let latencies_b: Vec<f32> = results_b.iter().filter(|r| r.success).map(|r| r.latency_ms as f32).collect();

        if latencies_a.is_empty() || latencies_b.is_empty() {
            return StatisticalSignificance {
                is_significant: false,
                p_value: 1.0,
                confidence_interval: (0.0, 0.0),
                effect_size: 0.0,
                power: 0.1,
            };
        }

        let mean_a = latencies_a.iter().sum::<f32>() / latencies_a.len() as f32;
        let mean_b = latencies_b.iter().sum::<f32>() / latencies_b.len() as f32;

        let var_a = latencies_a.iter().map(|x| (x - mean_a).powi(2)).sum::<f32>() / (latencies_a.len() as f32 - 1.0);
        let var_b = latencies_b.iter().map(|x| (x - mean_b).powi(2)).sum::<f32>() / (latencies_b.len() as f32 - 1.0);

        let pooled_std = ((var_a / latencies_a.len() as f32) + (var_b / latencies_b.len() as f32)).sqrt();
        let t_stat = (mean_a - mean_b) / pooled_std;

        // Simplified p-value estimation
        let p_value = if t_stat.abs() > 2.0 {
            0.05
        } else if t_stat.abs() > 1.5 {
            0.15
        } else {
            0.30
        };

        let is_significant = p_value < 0.05;
        let effect_size = (mean_a - mean_b) / pooled_std;
        let power = if is_significant { 0.8 } else { 0.2 };

        StatisticalSignificance {
            is_significant,
            p_value,
            confidence_interval: (mean_a - mean_b - 1.96 * pooled_std, mean_a - mean_b + 1.96 * pooled_std),
            effect_size,
            power,
        }
    }

    /// Get all A/B tests
    pub async fn get_ab_tests(&self) -> Vec<ABTestConfig> {
        let tests = self.ab_tests.read().await;
        tests.values().cloned().collect()
    }

    /// Get A/B test by ID
    pub async fn get_ab_test(&self, test_id: &str) -> Option<ABTestConfig> {
        let tests = self.ab_tests.read().await;
        tests.get(test_id).cloned()
    }
}

/// A/B test metrics for analysis
#[derive(Debug, Clone)]
struct ABTestMetrics {
    avg_latency: f32,
    success_rate: f32,
    avg_quality: f32,
    avg_throughput: f32,
}

/// Calculate percentile from sorted data
fn percentile(sorted_data: &[u64], p: f32) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }
    
    let idx = ((sorted_data.len() - 1) as f32 * p) as usize;
    sorted_data[idx]
}

/// Model performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub metrics: PerformanceMetrics,
    pub health: ModelHealth,
    pub ranking: u32,
    pub fallback_recommendation: Option<String>,
}

impl ModelPerformance {
    /// Create a performance summary
    pub async fn from_tracker(
        tracker: &PerformanceTracker,
        model_id: &str,
    ) -> Result<Self> {
        let metrics = tracker
            .get_metrics(model_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("No metrics available for model"))?;

        let health = tracker.get_model_health(model_id).await?;
        let fallback_recommendation = tracker.get_fallback_recommendation(model_id).await?;

        let rankings = tracker.get_performance_ranking().await;
        let ranking = rankings
            .iter()
            .position(|(id, _)| id == model_id)
            .map(|pos| pos as u32 + 1)
            .unwrap_or(0);

        Ok(Self {
            metrics,
            health,
            ranking,
            fallback_recommendation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_tracking() {
        let tracker = PerformanceTracker::new(5);

        // Track some performance entries
        tracker
            .track_performance(
                "test-model",
                1000,
                150,
                true,
                None,
                Some(0.9),
                "test",
                None,
            )
            .await
            .unwrap();

        // Get metrics
        let metrics = tracker.get_metrics("test-model").await.unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_model_health() {
        let tracker = PerformanceTracker::new(5);

        // Track failures
        for _ in 0..10 {
            tracker
                .track_performance(
                    "unhealthy-model",
                    5000,
                    100,
                    false,
                    Some(ErrorType::Timeout),
                    None,
                    "test",
                    None,
                )
                .await
                .unwrap();
        }

        let health = tracker.get_model_health("unhealthy-model").await.unwrap();
        assert_eq!(health.status, HealthStatus::Unavailable);
        assert!(!health.issues.is_empty());
    }
}