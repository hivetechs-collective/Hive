//! Performance Monitoring for AI Helpers
//!
//! This module provides comprehensive monitoring and observability for the AI helper
//! ecosystem, tracking performance, errors, and usage patterns.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

/// Performance metrics for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub operation_type: OperationType,
    pub helper_name: String,
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}

/// Types of operations being monitored
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    IndexKnowledge,
    RetrieveContext,
    RecognizePattern,
    AnalyzeQuality,
    SynthesizeInsight,
    VectorSearch,
    ModelInference,
    ParallelProcessing,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable detailed performance tracking
    pub detailed_tracking: bool,

    /// Maximum metrics to keep in memory
    pub max_metrics_in_memory: usize,

    /// Metrics aggregation window
    pub aggregation_window: Duration,

    /// Enable automatic alerts
    pub enable_alerts: bool,

    /// Performance thresholds for alerts
    pub thresholds: PerformanceThresholds,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_operation_duration: Duration,
    pub min_success_rate: f64,
    pub max_error_rate: f64,
    pub max_memory_usage_mb: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            detailed_tracking: true,
            max_metrics_in_memory: 10000,
            aggregation_window: Duration::from_secs(300), // 5 minutes
            enable_alerts: true,
            thresholds: PerformanceThresholds {
                max_operation_duration: Duration::from_secs(30),
                min_success_rate: 0.95,
                max_error_rate: 0.05,
                max_memory_usage_mb: 2048.0,
            },
        }
    }
}

/// Performance monitor for AI helpers
pub struct PerformanceMonitor {
    config: MonitoringConfig,

    /// Recent operation metrics
    recent_metrics: Arc<Mutex<Vec<OperationMetrics>>>,

    /// Aggregated statistics
    aggregated_stats: Arc<RwLock<AggregatedStats>>,

    /// Active operations being tracked
    active_operations: Arc<Mutex<std::collections::HashMap<String, ActiveOperation>>>,

    /// Alert handler
    alert_handler: Arc<dyn AlertHandler + Send + Sync>,
}

/// Aggregated statistics
#[derive(Default)]
struct AggregatedStats {
    /// Total operations by type
    operations_count: std::collections::HashMap<OperationType, usize>,

    /// Success rate by operation type
    success_rates: std::collections::HashMap<OperationType, f64>,

    /// Average duration by operation type
    average_durations: std::collections::HashMap<OperationType, Duration>,

    /// Error counts by operation type
    error_counts: std::collections::HashMap<OperationType, usize>,

    /// Memory usage over time
    memory_usage_history: Vec<(DateTime<Utc>, f64)>,

    /// Throughput metrics
    throughput_per_minute: Vec<(DateTime<Utc>, usize)>,
}

/// Active operation being tracked
struct ActiveOperation {
    operation_type: OperationType,
    helper_name: String,
    start_time: Instant,
    start_utc: DateTime<Utc>,
}

/// Alert handler trait
pub trait AlertHandler: Send + Sync {
    /// Handle a performance alert
    fn handle_alert(&self, alert: PerformanceAlert);
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Types of alerts
#[derive(Debug, Clone)]
pub enum AlertType {
    SlowOperation,
    HighErrorRate,
    LowSuccessRate,
    HighMemoryUsage,
    SystemError,
}

/// Alert severity levels
#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Default alert handler that logs alerts
pub struct LoggingAlertHandler;

impl AlertHandler for LoggingAlertHandler {
    fn handle_alert(&self, alert: PerformanceAlert) {
        match alert.severity {
            AlertSeverity::Info => tracing::info!("Performance alert: {}", alert.message),
            AlertSeverity::Warning => tracing::warn!("Performance alert: {}", alert.message),
            AlertSeverity::Error => tracing::error!("Performance alert: {}", alert.message),
            AlertSeverity::Critical => {
                tracing::error!("CRITICAL performance alert: {}", alert.message)
            }
        }
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: MonitoringConfig) -> Self {
        Self::with_alert_handler(config, Arc::new(LoggingAlertHandler))
    }

    /// Create with custom alert handler
    pub fn with_alert_handler(
        config: MonitoringConfig,
        alert_handler: Arc<dyn AlertHandler + Send + Sync>,
    ) -> Self {
        Self {
            config,
            recent_metrics: Arc::new(Mutex::new(Vec::new())),
            aggregated_stats: Arc::new(RwLock::new(AggregatedStats::default())),
            active_operations: Arc::new(Mutex::new(std::collections::HashMap::new())),
            alert_handler,
        }
    }

    /// Start tracking an operation
    pub async fn start_operation(
        &self,
        operation_type: OperationType,
        helper_name: &str,
    ) -> String {
        let operation_id = uuid::Uuid::new_v4().to_string();

        let active_op = ActiveOperation {
            operation_type,
            helper_name: helper_name.to_string(),
            start_time: Instant::now(),
            start_utc: Utc::now(),
        };

        self.active_operations
            .lock()
            .await
            .insert(operation_id.clone(), active_op);

        operation_id
    }

    /// Complete an operation
    pub async fn complete_operation(
        &self,
        operation_id: &str,
        success: bool,
        error_message: Option<String>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let active_op = self.active_operations.lock().await.remove(operation_id);

        if let Some(op) = active_op {
            let duration = op.start_time.elapsed();

            let metrics = OperationMetrics {
                operation_type: op.operation_type.clone(),
                helper_name: op.helper_name,
                start_time: op.start_utc,
                duration,
                success,
                error_message: error_message.clone(),
                metadata,
            };

            // Record metrics
            self.record_metrics(metrics).await?;

            // Check for alerts
            if self.config.enable_alerts {
                self.check_alerts(&op.operation_type, duration, success, error_message)
                    .await;
            }
        }

        Ok(())
    }

    /// Record metrics
    async fn record_metrics(&self, metrics: OperationMetrics) -> Result<()> {
        // Add to recent metrics
        let mut recent = self.recent_metrics.lock().await;
        recent.push(metrics.clone());

        // Trim if over limit
        if recent.len() > self.config.max_metrics_in_memory {
            recent.drain(0..100); // Remove oldest 100
        }
        drop(recent);

        // Update aggregated stats
        self.update_aggregated_stats(&metrics).await;

        Ok(())
    }

    /// Update aggregated statistics
    async fn update_aggregated_stats(&self, metrics: &OperationMetrics) {
        let mut stats = self.aggregated_stats.write().await;

        // Update operation count
        *stats
            .operations_count
            .entry(metrics.operation_type.clone())
            .or_insert(0) += 1;

        // Update error count
        if !metrics.success {
            *stats
                .error_counts
                .entry(metrics.operation_type.clone())
                .or_insert(0) += 1;
        }

        // Update average duration
        let count = stats.operations_count[&metrics.operation_type] as f64;
        let current_avg = stats
            .average_durations
            .get(&metrics.operation_type)
            .cloned()
            .unwrap_or(Duration::ZERO);

        let new_avg = Duration::from_secs_f64(
            (current_avg.as_secs_f64() * (count - 1.0) + metrics.duration.as_secs_f64()) / count,
        );
        stats
            .average_durations
            .insert(metrics.operation_type.clone(), new_avg);

        // Update success rate
        let error_count = stats
            .error_counts
            .get(&metrics.operation_type)
            .cloned()
            .unwrap_or(0) as f64;
        let success_rate = (count - error_count) / count;
        stats
            .success_rates
            .insert(metrics.operation_type.clone(), success_rate);
    }

    /// Check for alerts
    async fn check_alerts(
        &self,
        operation_type: &OperationType,
        duration: Duration,
        success: bool,
        error_message: Option<String>,
    ) {
        // Check for slow operation
        if duration > self.config.thresholds.max_operation_duration {
            let alert = PerformanceAlert {
                alert_type: AlertType::SlowOperation,
                severity: AlertSeverity::Warning,
                message: format!(
                    "{:?} operation took {:?}, exceeding threshold of {:?}",
                    operation_type, duration, self.config.thresholds.max_operation_duration
                ),
                timestamp: Utc::now(),
                metadata: serde_json::json!({
                    "operation_type": operation_type,
                    "duration_ms": duration.as_millis(),
                }),
            };
            self.alert_handler.handle_alert(alert);
        }

        // Check for operation failure
        if !success {
            let alert = PerformanceAlert {
                alert_type: AlertType::SystemError,
                severity: AlertSeverity::Error,
                message: format!(
                    "{:?} operation failed: {}",
                    operation_type,
                    error_message.unwrap_or_else(|| "Unknown error".to_string())
                ),
                timestamp: Utc::now(),
                metadata: serde_json::json!({
                    "operation_type": operation_type,
                }),
            };
            self.alert_handler.handle_alert(alert);
        }
    }

    /// Get current performance statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let stats = self.aggregated_stats.read().await;
        let recent_metrics = self.recent_metrics.lock().await;

        // Calculate overall metrics
        let total_operations: usize = stats.operations_count.values().sum();
        let total_errors: usize = stats.error_counts.values().sum();
        let overall_success_rate = if total_operations > 0 {
            (total_operations - total_errors) as f64 / total_operations as f64
        } else {
            1.0
        };

        // Get per-operation stats
        let mut operation_stats = Vec::new();
        for (op_type, count) in &stats.operations_count {
            operation_stats.push(OperationStats {
                operation_type: op_type.clone(),
                count: *count,
                success_rate: stats.success_rates.get(op_type).cloned().unwrap_or(1.0),
                average_duration: stats
                    .average_durations
                    .get(op_type)
                    .cloned()
                    .unwrap_or(Duration::ZERO),
                error_count: stats.error_counts.get(op_type).cloned().unwrap_or(0),
            });
        }

        PerformanceStats {
            total_operations,
            overall_success_rate,
            operation_stats,
            recent_operations: recent_metrics.len(),
            memory_usage_mb: Self::get_current_memory_usage(),
        }
    }

    /// Get current memory usage
    fn get_current_memory_usage() -> f64 {
        // TODO: Implement actual memory measurement
        // For now, return a placeholder
        256.0
    }

    /// Export metrics for external analysis
    pub async fn export_metrics(&self) -> Vec<OperationMetrics> {
        self.recent_metrics.lock().await.clone()
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_operations: usize,
    pub overall_success_rate: f64,
    pub operation_stats: Vec<OperationStats>,
    pub recent_operations: usize,
    pub memory_usage_mb: f64,
}

/// Statistics for a specific operation type
#[derive(Debug, Clone)]
pub struct OperationStats {
    pub operation_type: OperationType,
    pub count: usize,
    pub success_rate: f64,
    pub average_duration: Duration,
    pub error_count: usize,
}

/// Convenience trait for monitored operations
#[async_trait::async_trait]
pub trait MonitoredOperation {
    /// Execute with monitoring
    async fn execute_monitored<T, F, Fut>(
        &self,
        monitor: &PerformanceMonitor,
        operation_type: OperationType,
        helper_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send;
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitoring() {
        let monitor = PerformanceMonitor::new(MonitoringConfig::default());

        // Track a successful operation
        let op_id = monitor
            .start_operation(OperationType::IndexKnowledge, "test_helper")
            .await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        monitor
            .complete_operation(&op_id, true, None, serde_json::json!({}))
            .await
            .unwrap();

        // Check stats
        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.overall_success_rate, 1.0);
    }
}
