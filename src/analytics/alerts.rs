//! Real-time Alerting System
//! 
//! Monitors metrics and triggers alerts based on configurable thresholds:
//! - Cost overruns and budget alerts
//! - Performance degradation warnings
//! - Anomaly detection notifications
//! - Custom metric monitoring
//! - Multi-channel notifications (email, webhook, etc.)

use anyhow::{Result, Context};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Interval};

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    CostThreshold,
    BudgetExceeded,
    PerformanceDegradation,
    AnomalyDetected,
    ModelError,
    SystemHealth,
    Custom,
}

/// Alert condition operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
    Contains,
    NotContains,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub metric: String,
    pub operator: ConditionOperator,
    pub threshold: f64,
    pub duration: Option<Duration>,
    pub aggregation: Option<AggregationType>,
}

/// Aggregation types for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    Average,
    Sum,
    Min,
    Max,
    Count,
    Rate,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub conditions: Vec<AlertCondition>,
    pub condition_logic: ConditionLogic,
    pub cooldown: Duration,
    pub enabled: bool,
    pub channels: Vec<NotificationChannel>,
    pub metadata: HashMap<String, String>,
}

/// Condition logic (AND/OR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionLogic {
    All, // AND
    Any, // OR
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NotificationChannel {
    Email {
        recipients: Vec<String>,
        template: Option<String>,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
        method: String,
    },
    Slack {
        webhook_url: String,
        channel: Option<String>,
        mention: Option<String>,
    },
    Console {
        format: String,
    },
    Custom {
        handler: String,
        config: HashMap<String, String>,
    },
}

/// Alert instance (triggered alert)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub status: AlertStatus,
    pub metrics: HashMap<String, f64>,
    pub context: HashMap<String, String>,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
}

/// Alert manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub check_interval: Duration,
    pub retention_period: Duration,
    pub max_alerts_per_rule: usize,
    pub global_cooldown: Duration,
    pub notification_timeout: Duration,
    pub batch_notifications: bool,
    pub batch_window: Duration,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::seconds(30),
            retention_period: Duration::days(30),
            max_alerts_per_rule: 100,
            global_cooldown: Duration::minutes(5),
            notification_timeout: Duration::seconds(30),
            batch_notifications: true,
            batch_window: Duration::minutes(1),
        }
    }
}

/// Real-time alert manager
pub struct AlertManager {
    config: Arc<RwLock<AlertConfig>>,
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    alerts: Arc<RwLock<HashMap<String, Alert>>>,
    cooldowns: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    metric_store: Arc<RwLock<MetricStore>>,
    notification_sender: mpsc::Sender<NotificationRequest>,
    shutdown_sender: mpsc::Sender<()>,
}

/// Metric store for real-time data
struct MetricStore {
    metrics: HashMap<String, MetricTimeSeries>,
    max_age: Duration,
}

/// Time series data for a metric
struct MetricTimeSeries {
    values: Vec<(DateTime<Utc>, f64)>,
    last_updated: DateTime<Utc>,
}

/// Notification request
struct NotificationRequest {
    alert: Alert,
    channels: Vec<NotificationChannel>,
}

impl AlertManager {
    /// Create new alert manager
    pub async fn new(config: AlertConfig) -> Result<Self> {
        let (notification_sender, notification_receiver) = mpsc::channel(1000);
        let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);

        let manager = Self {
            config: Arc::new(RwLock::new(config)),
            rules: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            cooldowns: Arc::new(RwLock::new(HashMap::new())),
            metric_store: Arc::new(RwLock::new(MetricStore {
                metrics: HashMap::new(),
                max_age: Duration::hours(1),
            })),
            notification_sender,
            shutdown_sender,
        };

        // Start background tasks
        manager.start_monitoring_task();
        manager.start_notification_task(notification_receiver);
        manager.start_cleanup_task();

        Ok(manager)
    }

    /// Add or update alert rule
    pub async fn add_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove alert rule
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.remove(rule_id)
            .ok_or_else(|| anyhow::anyhow!("Rule not found"))?;
        Ok(())
    }

    /// Get all rules
    pub async fn get_rules(&self) -> Vec<AlertRule> {
        let rules = self.rules.read().await;
        rules.values().cloned().collect()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values()
            .filter(|a| a.status == AlertStatus::Active)
            .cloned()
            .collect()
    }

    /// Get all alerts
    pub async fn get_all_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        let alert = alerts.get_mut(alert_id)
            .ok_or_else(|| anyhow::anyhow!("Alert not found"))?;
        
        alert.status = AlertStatus::Acknowledged;
        Ok(())
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        let alert = alerts.get_mut(alert_id)
            .ok_or_else(|| anyhow::anyhow!("Alert not found"))?;
        
        alert.status = AlertStatus::Resolved;
        alert.resolved_at = Some(Utc::now());
        Ok(())
    }

    /// Update metric value
    pub async fn update_metric(&self, name: &str, value: f64) -> Result<()> {
        let mut store = self.metric_store.write().await;
        let now = Utc::now();

        let max_age = store.max_age; // Get max_age before the mutable borrow
        
        let series = store.metrics.entry(name.to_string()).or_insert_with(|| {
            MetricTimeSeries {
                values: Vec::new(),
                last_updated: now,
            }
        });

        series.values.push((now, value));
        series.last_updated = now;

        // Clean old values
        let cutoff = now - max_age;
        series.values.retain(|(timestamp, _)| *timestamp > cutoff);

        Ok(())
    }

    /// Batch update metrics
    pub async fn update_metrics(&self, metrics: HashMap<String, f64>) -> Result<()> {
        for (name, value) in metrics {
            self.update_metric(&name, value).await?;
        }
        Ok(())
    }

    /// Start monitoring task
    fn start_monitoring_task(&self) {
        let rules = Arc::clone(&self.rules);
        let alerts = Arc::clone(&self.alerts);
        let cooldowns = Arc::clone(&self.cooldowns);
        let metric_store = Arc::clone(&self.metric_store);
        let notification_sender = self.notification_sender.clone();
        let config = Arc::clone(&self.config);

        tokio::spawn(async move {
            let mut interval = {
                let config = config.read().await;
                interval(config.check_interval.to_std().unwrap())
            };

            loop {
                interval.tick().await;

                // Check each rule
                let rules_snapshot = rules.read().await.clone();
                
                for (rule_id, rule) in rules_snapshot {
                    if !rule.enabled {
                        continue;
                    }

                    // Check cooldown
                    let cooldowns_read = cooldowns.read().await;
                    if let Some(last_triggered) = cooldowns_read.get(&rule_id) {
                        if Utc::now() - *last_triggered < rule.cooldown {
                            continue;
                        }
                    }
                    drop(cooldowns_read);

                    // Evaluate conditions
                    let store = metric_store.read().await;
                    let triggered = Self::evaluate_rule(&rule, &store).await;
                    drop(store);

                    if triggered {
                        // Create alert
                        let alert = Alert {
                            id: uuid::Uuid::new_v4().to_string(),
                            rule_id: rule_id.clone(),
                            alert_type: rule.alert_type,
                            severity: rule.severity,
                            title: format!("{} Alert", rule.name),
                            message: Self::generate_alert_message(&rule).await,
                            triggered_at: Utc::now(),
                            resolved_at: None,
                            status: AlertStatus::Active,
                            metrics: Self::collect_alert_metrics(&rule, &*metric_store.read().await).await,
                            context: rule.metadata.clone(),
                        };

                        // Store alert
                        let mut alerts_write = alerts.write().await;
                        alerts_write.insert(alert.id.clone(), alert.clone());
                        drop(alerts_write);

                        // Update cooldown
                        let mut cooldowns_write = cooldowns.write().await;
                        cooldowns_write.insert(rule_id.clone(), Utc::now());
                        drop(cooldowns_write);

                        // Send notification
                        let _ = notification_sender.send(NotificationRequest {
                            alert,
                            channels: rule.channels.clone(),
                        }).await;
                    }
                }
            }
        });
    }

    /// Start notification task
    fn start_notification_task(&self, mut receiver: mpsc::Receiver<NotificationRequest>) {
        let config = Arc::clone(&self.config);

        tokio::spawn(async move {
            let mut batch: Vec<NotificationRequest> = Vec::new();
            let mut batch_timer = interval(Duration::minutes(1).to_std().unwrap());

            loop {
                tokio::select! {
                    Some(request) = receiver.recv() => {
                        let config = config.read().await;
                        if config.batch_notifications {
                            batch.push(request);
                        } else {
                            Self::send_notifications(vec![request]).await;
                        }
                    }
                    _ = batch_timer.tick() => {
                        if !batch.is_empty() {
                            Self::send_notifications(std::mem::take(&mut batch)).await;
                        }
                    }
                }
            }
        });
    }

    /// Start cleanup task
    fn start_cleanup_task(&self) {
        let alerts = Arc::clone(&self.alerts);
        let config = Arc::clone(&self.config);

        tokio::spawn(async move {
            let mut interval = interval(Duration::hours(1).to_std().unwrap());

            loop {
                interval.tick().await;

                let config = config.read().await;
                let cutoff = Utc::now() - config.retention_period;
                drop(config);

                let mut alerts_write = alerts.write().await;
                alerts_write.retain(|_, alert| {
                    alert.triggered_at > cutoff || alert.status == AlertStatus::Active
                });
            }
        });
    }

    /// Evaluate alert rule
    async fn evaluate_rule(rule: &AlertRule, store: &MetricStore) -> bool {
        let results: Vec<bool> = rule.conditions.iter()
            .map(|condition| Self::evaluate_condition(condition, store))
            .collect();

        match rule.condition_logic {
            ConditionLogic::All => results.iter().all(|&r| r),
            ConditionLogic::Any => results.iter().any(|&r| r),
        }
    }

    /// Evaluate single condition
    fn evaluate_condition(condition: &AlertCondition, store: &MetricStore) -> bool {
        if let Some(series) = store.metrics.get(&condition.metric) {
            let value = if let Some(aggregation) = &condition.aggregation {
                Self::aggregate_values(&series.values, aggregation, condition.duration.as_ref())
            } else {
                series.values.last().map(|(_, v)| *v).unwrap_or(0.0)
            };

            match condition.operator {
                ConditionOperator::GreaterThan => value > condition.threshold,
                ConditionOperator::LessThan => value < condition.threshold,
                ConditionOperator::GreaterThanOrEqual => value >= condition.threshold,
                ConditionOperator::LessThanOrEqual => value <= condition.threshold,
                ConditionOperator::Equal => (value - condition.threshold).abs() < f64::EPSILON,
                ConditionOperator::NotEqual => (value - condition.threshold).abs() >= f64::EPSILON,
                _ => false, // Contains/NotContains not applicable to numeric values
            }
        } else {
            false
        }
    }

    /// Aggregate metric values
    fn aggregate_values(values: &[(DateTime<Utc>, f64)], aggregation: &AggregationType, duration: Option<&Duration>) -> f64 {
        let cutoff = duration.map(|d| Utc::now() - *d);
        
        let filtered_values: Vec<f64> = values.iter()
            .filter(|(timestamp, _)| {
                cutoff.map_or(true, |c| *timestamp > c)
            })
            .map(|(_, value)| *value)
            .collect();

        if filtered_values.is_empty() {
            return 0.0;
        }

        match aggregation {
            AggregationType::Average => filtered_values.iter().sum::<f64>() / filtered_values.len() as f64,
            AggregationType::Sum => filtered_values.iter().sum(),
            AggregationType::Min => filtered_values.iter().cloned().fold(f64::INFINITY, f64::min),
            AggregationType::Max => filtered_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            AggregationType::Count => filtered_values.len() as f64,
            AggregationType::Rate => {
                if values.len() < 2 {
                    0.0
                } else {
                    let first = values.first().unwrap();
                    let last = values.last().unwrap();
                    let time_diff = (last.0 - first.0).num_seconds() as f64;
                    if time_diff > 0.0 {
                        (last.1 - first.1) / time_diff
                    } else {
                        0.0
                    }
                }
            }
        }
    }

    /// Generate alert message
    async fn generate_alert_message(rule: &AlertRule) -> String {
        format!(
            "{}: {} - {}",
            rule.severity.to_string().to_uppercase(),
            rule.name,
            rule.description
        )
    }

    /// Collect metrics for alert
    async fn collect_alert_metrics(rule: &AlertRule, store: &MetricStore) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        for condition in &rule.conditions {
            if let Some(series) = store.metrics.get(&condition.metric) {
                if let Some((_, value)) = series.values.last() {
                    metrics.insert(condition.metric.clone(), *value);
                }
            }
        }
        
        metrics
    }

    /// Send notifications
    async fn send_notifications(requests: Vec<NotificationRequest>) {
        for request in requests {
            for channel in &request.channels {
                if let Err(e) = Self::send_to_channel(&request.alert, channel).await {
                    eprintln!("Failed to send notification: {}", e);
                }
            }
        }
    }

    /// Send to specific channel
    async fn send_to_channel(alert: &Alert, channel: &NotificationChannel) -> Result<()> {
        match channel {
            NotificationChannel::Console { format } => {
                let message = match format.as_str() {
                    "json" => serde_json::to_string_pretty(alert)?,
                    _ => format!("[{}] {} - {}", alert.severity.to_string().to_uppercase(), alert.title, alert.message),
                };
                println!("ALERT: {}", message);
                Ok(())
            }
            NotificationChannel::Webhook { url, headers, method } => {
                let client = reqwest::Client::new();
                let mut request = match method.as_str() {
                    "GET" => client.get(url),
                    "PUT" => client.put(url),
                    _ => client.post(url),
                };

                for (key, value) in headers {
                    request = request.header(key, value);
                }

                request = request.json(alert);
                request.send().await?;
                Ok(())
            }
            NotificationChannel::Slack { webhook_url, channel, mention } => {
                let client = reqwest::Client::new();
                
                let mut text = format!("*{}*: {}\n{}", 
                    alert.severity.to_string().to_uppercase(),
                    alert.title,
                    alert.message
                );

                if let Some(mention) = mention {
                    text = format!("{} {}", mention, text);
                }

                let payload = serde_json::json!({
                    "text": text,
                    "channel": channel,
                    "icon_emoji": match alert.severity {
                        AlertSeverity::Info => ":information_source:",
                        AlertSeverity::Warning => ":warning:",
                        AlertSeverity::Critical => ":rotating_light:",
                        AlertSeverity::Emergency => ":fire:",
                    },
                });

                client.post(webhook_url)
                    .json(&payload)
                    .send()
                    .await?;
                
                Ok(())
            }
            _ => Ok(()), // Other channels not implemented in this example
        }
    }

    /// Create predefined alert rules
    pub async fn create_default_rules(&self) -> Result<()> {
        // Cost threshold alert
        self.add_rule(AlertRule {
            id: "cost_threshold".to_string(),
            name: "High Cost Alert".to_string(),
            description: "Triggered when hourly cost exceeds threshold".to_string(),
            alert_type: AlertType::CostThreshold,
            severity: AlertSeverity::Warning,
            conditions: vec![
                AlertCondition {
                    metric: "cost.hourly".to_string(),
                    operator: ConditionOperator::GreaterThan,
                    threshold: 100.0,
                    duration: None,
                    aggregation: None,
                },
            ],
            condition_logic: ConditionLogic::Any,
            cooldown: Duration::hours(1),
            enabled: true,
            channels: vec![
                NotificationChannel::Console {
                    format: "text".to_string(),
                },
            ],
            metadata: HashMap::new(),
        }).await?;

        // Budget exceeded alert
        self.add_rule(AlertRule {
            id: "budget_exceeded".to_string(),
            name: "Budget Exceeded".to_string(),
            description: "Monthly budget has been exceeded".to_string(),
            alert_type: AlertType::BudgetExceeded,
            severity: AlertSeverity::Critical,
            conditions: vec![
                AlertCondition {
                    metric: "cost.monthly.percentage".to_string(),
                    operator: ConditionOperator::GreaterThan,
                    threshold: 100.0,
                    duration: None,
                    aggregation: None,
                },
            ],
            condition_logic: ConditionLogic::Any,
            cooldown: Duration::days(1),
            enabled: true,
            channels: vec![
                NotificationChannel::Console {
                    format: "text".to_string(),
                },
            ],
            metadata: HashMap::new(),
        }).await?;

        // Performance degradation alert
        self.add_rule(AlertRule {
            id: "performance_degradation".to_string(),
            name: "Performance Degradation".to_string(),
            description: "API latency exceeds acceptable threshold".to_string(),
            alert_type: AlertType::PerformanceDegradation,
            severity: AlertSeverity::Warning,
            conditions: vec![
                AlertCondition {
                    metric: "performance.api.latency".to_string(),
                    operator: ConditionOperator::GreaterThan,
                    threshold: 1000.0, // 1 second
                    duration: Some(Duration::minutes(5)),
                    aggregation: Some(AggregationType::Average),
                },
            ],
            condition_logic: ConditionLogic::Any,
            cooldown: Duration::minutes(15),
            enabled: true,
            channels: vec![
                NotificationChannel::Console {
                    format: "text".to_string(),
                },
            ],
            metadata: HashMap::new(),
        }).await?;

        Ok(())
    }

    /// Get alert statistics
    pub async fn get_statistics(&self) -> AlertStatistics {
        let alerts = self.alerts.read().await;
        let rules = self.rules.read().await;

        let mut stats = AlertStatistics {
            total_alerts: alerts.len(),
            active_alerts: 0,
            acknowledged_alerts: 0,
            resolved_alerts: 0,
            alerts_by_severity: HashMap::new(),
            alerts_by_type: HashMap::new(),
            enabled_rules: 0,
            total_rules: rules.len(),
        };

        for alert in alerts.values() {
            match alert.status {
                AlertStatus::Active => stats.active_alerts += 1,
                AlertStatus::Acknowledged => stats.acknowledged_alerts += 1,
                AlertStatus::Resolved => stats.resolved_alerts += 1,
                _ => {}
            }

            *stats.alerts_by_severity.entry(alert.severity).or_insert(0) += 1;
            *stats.alerts_by_type.entry(alert.alert_type).or_insert(0) += 1;
        }

        stats.enabled_rules = rules.values().filter(|r| r.enabled).count();

        stats
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub active_alerts: usize,
    pub acknowledged_alerts: usize,
    pub resolved_alerts: usize,
    pub alerts_by_severity: HashMap<AlertSeverity, usize>,
    pub alerts_by_type: HashMap<AlertType, usize>,
    pub enabled_rules: usize,
    pub total_rules: usize,
}

// Display implementations for enum types
impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "info"),
            AlertSeverity::Warning => write!(f, "warning"),
            AlertSeverity::Critical => write!(f, "critical"),
            AlertSeverity::Emergency => write!(f, "emergency"),
        }
    }
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::CostThreshold => write!(f, "cost_threshold"),
            AlertType::BudgetExceeded => write!(f, "budget_exceeded"),
            AlertType::PerformanceDegradation => write!(f, "performance_degradation"),
            AlertType::AnomalyDetected => write!(f, "anomaly_detected"),
            AlertType::ModelError => write!(f, "model_error"),
            AlertType::SystemHealth => write!(f, "system_health"),
            AlertType::Custom => write!(f, "custom"),
        }
    }
}

// Hash implementations for HashMap keys
impl std::hash::Hash for AlertSeverity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

impl std::hash::Hash for AlertType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_manager_creation() -> Result<()> {
        let config = AlertConfig::default();
        let manager = AlertManager::new(config).await?;
        
        // Create default rules
        manager.create_default_rules().await?;
        
        let rules = manager.get_rules().await;
        assert!(!rules.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_metric_updates() -> Result<()> {
        let config = AlertConfig::default();
        let manager = AlertManager::new(config).await?;
        
        // Update metrics
        manager.update_metric("test.metric", 100.0).await?;
        manager.update_metric("test.metric", 150.0).await?;
        
        // Verify metrics are stored
        let store = manager.metric_store.read().await;
        assert!(store.metrics.contains_key("test.metric"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_alert_rule_evaluation() -> Result<()> {
        let config = AlertConfig {
            check_interval: Duration::milliseconds(100),
            ..Default::default()
        };
        let manager = AlertManager::new(config).await?;
        
        // Add test rule
        manager.add_rule(AlertRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test alert rule".to_string(),
            alert_type: AlertType::Custom,
            severity: AlertSeverity::Info,
            conditions: vec![
                AlertCondition {
                    metric: "test.value".to_string(),
                    operator: ConditionOperator::GreaterThan,
                    threshold: 50.0,
                    duration: None,
                    aggregation: None,
                },
            ],
            condition_logic: ConditionLogic::Any,
            cooldown: Duration::seconds(1),
            enabled: true,
            channels: vec![
                NotificationChannel::Console {
                    format: "text".to_string(),
                },
            ],
            metadata: HashMap::new(),
        }).await?;
        
        // Update metric to trigger alert
        manager.update_metric("test.value", 75.0).await?;
        
        // Wait for check interval
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        
        // Check if alert was created
        let alerts = manager.get_active_alerts().await;
        assert!(!alerts.is_empty());
        
        Ok(())
    }
}