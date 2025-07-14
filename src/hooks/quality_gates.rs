//! Quality Gates System
//!
//! Provides automated quality validation workflows for consensus pipeline stages.
//! Supports configurable quality metrics, thresholds, and automated remediation.

use super::{
    approval_workflow::{ApprovalRequest, ApprovalStatus, ApprovalWorkflow},
    registry::HookId,
    AuditEvent, AuditEventType, EventSource, EventType, HookAuditLogger, HookEvent,
};
use crate::consensus::types::{Stage, StageAnalytics, StageResult};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Quality gates manager
pub struct QualityGateManager {
    gates: Arc<RwLock<Vec<QualityGate>>>,
    metrics: Arc<RwLock<QualityMetrics>>,
    violations: Arc<RwLock<Vec<QualityViolation>>>,
    approval_workflow: Option<Arc<ApprovalWorkflow>>,
    audit_logger: Option<Arc<HookAuditLogger>>,
    config: QualityGateConfig,
}

/// Configuration for quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    /// Whether quality gates are enabled
    pub enabled: bool,

    /// Default action when quality gate fails
    pub default_failure_action: QualityGateAction,

    /// Whether to block operations on quality failures
    pub block_on_failure: bool,

    /// Quality metrics configuration
    pub metrics_config: QualityMetricsConfig,

    /// Remediation settings
    pub remediation: RemediationConfig,

    /// Notification settings
    pub notifications: QualityNotificationConfig,

    /// Grace period for quality violations (seconds)
    pub grace_period_seconds: u64,

    /// Maximum violations per hour before escalation
    pub max_violations_per_hour: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsConfig {
    /// Response quality scoring weights
    pub quality_weights: QualityWeights,

    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,

    /// Content quality settings
    pub content_quality: ContentQualityConfig,

    /// Model-specific adjustments
    pub model_adjustments: HashMap<String, ModelQualityAdjustment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWeights {
    /// Weight for coherence score (0.0 - 1.0)
    pub coherence: f64,

    /// Weight for completeness score (0.0 - 1.0)
    pub completeness: f64,

    /// Weight for relevance score (0.0 - 1.0)
    pub relevance: f64,

    /// Weight for factual accuracy score (0.0 - 1.0)
    pub accuracy: f64,

    /// Weight for safety score (0.0 - 1.0)
    pub safety: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum acceptable response time (seconds)
    pub max_response_time: f64,

    /// Maximum acceptable token usage per stage
    pub max_tokens: u32,

    /// Maximum acceptable cost per stage
    pub max_cost: f64,

    /// Maximum acceptable retry count
    pub max_retries: u32,

    /// Maximum acceptable error rate (0.0 - 1.0)
    pub max_error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentQualityConfig {
    /// Minimum response length (characters)
    pub min_length: usize,

    /// Maximum response length (characters)
    pub max_length: usize,

    /// Required keywords that should appear in response
    pub required_keywords: Vec<String>,

    /// Forbidden keywords that should not appear
    pub forbidden_keywords: Vec<String>,

    /// Language detection settings
    pub language_detection: LanguageDetectionConfig,

    /// Content safety settings
    pub safety_settings: ContentSafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDetectionConfig {
    /// Expected language code (e.g., "en", "es")
    pub expected_language: Option<String>,

    /// Minimum confidence for language detection
    pub min_confidence: f64,

    /// Whether to enforce language consistency
    pub enforce_consistency: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSafetyConfig {
    /// Enable content safety checks
    pub enabled: bool,

    /// Safety categories to check
    pub categories: Vec<SafetyCategory>,

    /// Action to take on safety violations
    pub violation_action: SafetyViolationAction,

    /// Custom safety rules
    pub custom_rules: Vec<SafetyRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyCategory {
    /// Harmful or dangerous content
    Harmful,

    /// Hate speech or discrimination
    HateSpeech,

    /// Adult or sexual content
    Adult,

    /// Violence or graphic content
    Violence,

    /// Personal information exposure
    PrivacyViolation,

    /// Misinformation or false claims
    Misinformation,

    /// Spam or promotional content
    Spam,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyViolationAction {
    /// Block the response entirely
    Block,

    /// Sanitize the content
    Sanitize,

    /// Flag for manual review
    Flag,

    /// Log but allow
    LogOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRule {
    pub id: String,
    pub name: String,
    pub pattern: String, // Regex pattern
    pub category: SafetyCategory,
    pub action: SafetyViolationAction,
    pub severity: SafetySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQualityAdjustment {
    /// Model-specific quality threshold adjustment
    pub quality_threshold_adjustment: f64,

    /// Model-specific performance expectations
    pub performance_expectations: PerformanceExpectation,

    /// Known model limitations
    pub known_limitations: Vec<String>,

    /// Model-specific quality weights
    pub custom_weights: Option<QualityWeights>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceExpectation {
    /// Expected response time range
    pub response_time_range: (f64, f64),

    /// Expected token usage range
    pub token_usage_range: (u32, u32),

    /// Expected cost range
    pub cost_range: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationConfig {
    /// Enable automatic remediation
    pub enabled: bool,

    /// Remediation strategies
    pub strategies: Vec<RemediationStrategy>,

    /// Maximum remediation attempts per stage
    pub max_attempts: u32,

    /// Backoff strategy for retries
    pub backoff_strategy: BackoffStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationStrategy {
    pub id: String,
    pub name: String,
    pub applicable_failures: Vec<QualityFailureType>,
    pub action: RemediationAction,
    pub priority: u32,
    pub success_rate: f64, // Historical success rate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemediationAction {
    /// Retry with the same parameters
    Retry,

    /// Retry with adjusted parameters
    RetryWithAdjustment {
        parameter_adjustments: HashMap<String, serde_json::Value>,
    },

    /// Switch to a different model
    SwitchModel { target_model: String },

    /// Adjust prompt or context
    AdjustPrompt {
        adjustment_type: PromptAdjustmentType,
    },

    /// Request human intervention
    RequestHumanIntervention,

    /// Use fallback response
    UseFallback { fallback_response: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptAdjustmentType {
    /// Make prompt more specific
    MoreSpecific,

    /// Make prompt more general
    MoreGeneral,

    /// Add context
    AddContext,

    /// Simplify language
    SimplifyLanguage,

    /// Add examples
    AddExamples,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Linear backoff (constant delay)
    Linear { delay_seconds: u64 },

    /// Exponential backoff
    Exponential {
        base_delay_seconds: u64,
        multiplier: f64,
    },

    /// Fixed delays
    Fixed { delays_seconds: Vec<u64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityNotificationConfig {
    /// Enable quality gate notifications
    pub enabled: bool,

    /// Notification channels
    pub channels: Vec<NotificationChannel>,

    /// Minimum severity to trigger notifications
    pub min_severity: QualityViolationSeverity,

    /// Notification throttling (max per hour)
    pub max_notifications_per_hour: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Console,
    Log,
    Email { recipients: Vec<String> },
    Webhook { url: String },
}

/// A quality gate that validates specific criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub stage: Option<Stage>, // None means applies to all stages
    pub criteria: Vec<QualityCriterion>,
    pub failure_action: QualityGateAction,
    pub enabled: bool,
    pub priority: u32, // Higher numbers = higher priority
    pub tags: Vec<String>,
}

/// A specific quality criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCriterion {
    pub id: String,
    pub name: String,
    pub metric: QualityMetric,
    pub threshold: QualityThreshold,
    pub weight: f64,    // Importance weight
    pub required: bool, // If true, failure blocks execution
}

/// Quality metrics that can be evaluated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityMetric {
    /// Overall quality score
    OverallQuality,

    /// Response coherence
    Coherence,

    /// Response completeness
    Completeness,

    /// Response relevance to query
    Relevance,

    /// Factual accuracy
    Accuracy,

    /// Content safety
    Safety,

    /// Response time performance
    ResponseTime,

    /// Token usage efficiency
    TokenEfficiency,

    /// Cost efficiency
    CostEfficiency,

    /// Error rate
    ErrorRate,

    /// Response length
    ResponseLength,

    /// Language consistency
    LanguageConsistency,

    /// Keyword presence
    KeywordPresence { keywords: Vec<String> },

    /// Custom metric
    Custom {
        name: String,
        evaluator: String, // Function name or script
    },
}

/// Quality threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThreshold {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub target_value: Option<f64>,
    pub tolerance: Option<f64>, // Allowed deviation from target
}

/// Action to take when quality gate fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityGateAction {
    /// Block execution
    Block,

    /// Allow with warning
    Warn,

    /// Request approval
    RequestApproval,

    /// Trigger remediation
    Remediate,

    /// Log only
    LogOnly,

    /// Custom action
    Custom { action_id: String },
}

/// Quality violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub id: String,
    pub gate_id: String,
    pub criterion_id: String,
    pub stage: Stage,
    pub violation_type: QualityFailureType,
    pub severity: QualityViolationSeverity,
    pub actual_value: f64,
    pub expected_value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub conversation_id: String,
    pub stage_id: String,
    pub remediation_attempted: bool,
    pub remediation_successful: Option<bool>,
    pub action_taken: QualityGateAction,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityFailureType {
    /// Quality score below threshold
    LowQuality,

    /// Poor performance metrics
    PerformanceDegradation,

    /// Content safety violation
    SafetyViolation,

    /// Language or format issues
    FormatViolation,

    /// Missing required content
    IncompleteResponse,

    /// Factual accuracy issues
    AccuracyViolation,

    /// Custom failure type
    Custom { failure_type: String },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum QualityViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Quality metrics tracking
#[derive(Debug, Clone, Default)]
pub struct QualityMetrics {
    /// Quality scores by stage
    pub stage_scores: HashMap<String, Vec<f64>>,

    /// Performance metrics by stage
    pub performance_metrics: HashMap<String, Vec<PerformanceMetric>>,

    /// Safety violations by stage
    pub safety_violations: HashMap<String, u32>,

    /// Remediation success rates
    pub remediation_success_rates: HashMap<String, f64>,

    /// Overall quality trend
    pub quality_trend: QualityTrend,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub response_time: f64,
    pub token_usage: u32,
    pub cost: f64,
    pub error_count: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityTrend {
    pub current_average: f64,
    pub trend_direction: TrendDirection,
    pub trend_strength: f64, // 0.0 - 1.0
    pub data_points: Vec<(chrono::DateTime<chrono::Utc>, f64)>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TrendDirection {
    #[default]
    Stable,
    Improving,
    Declining,
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_failure_action: QualityGateAction::Warn,
            block_on_failure: false,
            metrics_config: QualityMetricsConfig::default(),
            remediation: RemediationConfig::default(),
            notifications: QualityNotificationConfig::default(),
            grace_period_seconds: 60,
            max_violations_per_hour: 10,
        }
    }
}

impl Default for QualityMetricsConfig {
    fn default() -> Self {
        Self {
            quality_weights: QualityWeights::default(),
            performance_thresholds: PerformanceThresholds::default(),
            content_quality: ContentQualityConfig::default(),
            model_adjustments: HashMap::new(),
        }
    }
}

impl Default for QualityWeights {
    fn default() -> Self {
        Self {
            coherence: 0.25,
            completeness: 0.20,
            relevance: 0.25,
            accuracy: 0.20,
            safety: 0.10,
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_response_time: 30.0, // 30 seconds
            max_tokens: 4000,
            max_cost: 0.50, // $0.50
            max_retries: 3,
            max_error_rate: 0.05, // 5%
        }
    }
}

impl Default for ContentQualityConfig {
    fn default() -> Self {
        Self {
            min_length: 10,
            max_length: 10000,
            required_keywords: Vec::new(),
            forbidden_keywords: vec!["inappropriate".to_string(), "harmful".to_string()],
            language_detection: LanguageDetectionConfig::default(),
            safety_settings: ContentSafetyConfig::default(),
        }
    }
}

impl Default for LanguageDetectionConfig {
    fn default() -> Self {
        Self {
            expected_language: Some("en".to_string()),
            min_confidence: 0.8,
            enforce_consistency: true,
        }
    }
}

impl Default for ContentSafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            categories: vec![
                SafetyCategory::Harmful,
                SafetyCategory::HateSpeech,
                SafetyCategory::PrivacyViolation,
            ],
            violation_action: SafetyViolationAction::Flag,
            custom_rules: Vec::new(),
        }
    }
}

impl Default for RemediationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategies: vec![
                RemediationStrategy {
                    id: "retry_simple".to_string(),
                    name: "Simple Retry".to_string(),
                    applicable_failures: vec![
                        QualityFailureType::PerformanceDegradation,
                        QualityFailureType::LowQuality,
                    ],
                    action: RemediationAction::Retry,
                    priority: 1,
                    success_rate: 0.6,
                },
                RemediationStrategy {
                    id: "adjust_prompt".to_string(),
                    name: "Adjust Prompt".to_string(),
                    applicable_failures: vec![
                        QualityFailureType::IncompleteResponse,
                        QualityFailureType::LowQuality,
                    ],
                    action: RemediationAction::AdjustPrompt {
                        adjustment_type: PromptAdjustmentType::MoreSpecific,
                    },
                    priority: 2,
                    success_rate: 0.75,
                },
            ],
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::Linear { delay_seconds: 5 },
        }
    }
}

impl Default for QualityNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![NotificationChannel::Console, NotificationChannel::Log],
            min_severity: QualityViolationSeverity::Warning,
            max_notifications_per_hour: 10,
        }
    }
}

impl QualityGateManager {
    /// Create a new quality gate manager
    pub async fn new(config: QualityGateConfig) -> Result<Self> {
        Ok(Self {
            gates: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(QualityMetrics::default())),
            violations: Arc::new(RwLock::new(Vec::new())),
            approval_workflow: None,
            audit_logger: None,
            config,
        })
    }

    /// Set approval workflow for handling approval requests
    pub fn with_approval_workflow(mut self, workflow: Arc<ApprovalWorkflow>) -> Self {
        self.approval_workflow = Some(workflow);
        self
    }

    /// Set audit logger
    pub fn with_audit_logger(mut self, logger: Arc<HookAuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    /// Add a quality gate
    pub async fn add_gate(&self, gate: QualityGate) -> Result<()> {
        let mut gates = self.gates.write().await;

        // Check for duplicate IDs
        if gates.iter().any(|g| g.id == gate.id) {
            return Err(anyhow::anyhow!(
                "Quality gate with ID '{}' already exists",
                gate.id
            ));
        }

        gates.push(gate.clone());

        // Sort by priority (higher first)
        gates.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Log the addition
        if let Some(audit_logger) = &self.audit_logger {
            let event = AuditEventType::HookRegistered {
                hook_id: HookId(gate.id.clone()),
                hook_name: gate.name.clone(),
            };

            let audit_event = AuditEvent::new(event)
                .with_context("gate_type", "quality_gate")
                .with_context("priority", gate.priority);
            audit_logger.log_event(audit_event).await?;
        }

        Ok(())
    }

    /// Remove a quality gate
    pub async fn remove_gate(&self, gate_id: &str) -> Result<bool> {
        let mut gates = self.gates.write().await;
        let initial_len = gates.len();
        gates.retain(|g| g.id != gate_id);

        let removed = gates.len() < initial_len;

        if removed {
            // Log the removal
            if let Some(audit_logger) = &self.audit_logger {
                let event = AuditEventType::HookRemoved {
                    hook_id: HookId(gate_id.to_string()),
                };

                let audit_event = AuditEvent::new(event)
                    .with_context("gate_type", "quality_gate")
                    .with_context("gate_id", gate_id.to_string());
                audit_logger.log_event(audit_event).await?;
            }
        }

        Ok(removed)
    }

    /// Evaluate quality gates for a stage result
    pub async fn evaluate_stage_result(
        &self,
        stage: Stage,
        stage_result: &StageResult,
    ) -> Result<QualityEvaluationResult> {
        if !self.config.enabled {
            return Ok(QualityEvaluationResult {
                passed: true,
                violations: Vec::new(),
                overall_score: 1.0,
                stage_scores: HashMap::new(),
                actions_required: Vec::new(),
                remediation_suggestions: Vec::new(),
            });
        }

        let gates = self.gates.read().await;
        let applicable_gates: Vec<&QualityGate> = gates
            .iter()
            .filter(|gate| gate.enabled && (gate.stage.is_none() || gate.stage == Some(stage)))
            .collect();

        if applicable_gates.is_empty() {
            return Ok(QualityEvaluationResult {
                passed: true,
                violations: Vec::new(),
                overall_score: 1.0,
                stage_scores: HashMap::new(),
                actions_required: Vec::new(),
                remediation_suggestions: Vec::new(),
            });
        }

        let mut violations = Vec::new();
        let mut stage_scores = HashMap::new();
        let mut actions_required = Vec::new();
        let mut remediation_suggestions = Vec::new();

        // Evaluate each applicable gate
        for gate in applicable_gates {
            let gate_result = self.evaluate_gate(gate, stage, stage_result).await?;

            stage_scores.insert(gate.id.clone(), gate_result.score);

            if !gate_result.passed {
                let gate_violations = gate_result.violations.clone();
                violations.extend(gate_violations.clone()); // Use the cloned version

                // Determine required actions
                match &gate.failure_action {
                    QualityGateAction::Block => {
                        actions_required.push(QualityActionRequired::Block {
                            gate_id: gate.id.clone(),
                            reason: format!("Quality gate '{}' failed", gate.name),
                        });
                    }
                    QualityGateAction::RequestApproval => {
                        if let Some(approval_workflow) = &self.approval_workflow {
                            let approval_request = self
                                .create_quality_approval_request(
                                    &gate.id,
                                    &gate.name,
                                    stage,
                                    stage_result,
                                    &gate_violations,
                                )
                                .await?;

                            actions_required
                                .push(QualityActionRequired::RequestApproval { approval_request });
                        } else {
                            // Fallback to warning if no approval workflow
                            actions_required.push(QualityActionRequired::Warn {
                                message: format!(
                                    "Quality gate '{}' failed (no approval workflow configured)",
                                    gate.name
                                ),
                            });
                        }
                    }
                    QualityGateAction::Remediate => {
                        let suggestions = self
                            .generate_remediation_suggestions(&gate_violations)
                            .await?;
                        remediation_suggestions.extend(suggestions);

                        actions_required.push(QualityActionRequired::Remediate {
                            strategies: remediation_suggestions.clone(),
                        });
                    }
                    QualityGateAction::Warn => {
                        actions_required.push(QualityActionRequired::Warn {
                            message: format!(
                                "Quality gate '{}' failed: {}",
                                gate.name,
                                gate_result.failure_reason.unwrap_or_default()
                            ),
                        });
                    }
                    QualityGateAction::LogOnly => {
                        // Just log, no action required
                        tracing::warn!(
                            "Quality gate '{}' failed: {}",
                            gate.name,
                            gate_result.failure_reason.unwrap_or_default()
                        );
                    }
                    QualityGateAction::Custom { action_id } => {
                        actions_required.push(QualityActionRequired::Custom {
                            action_id: action_id.clone(),
                            gate_id: gate.id.clone(),
                        });
                    }
                }
            }
        }

        // Store violations
        {
            let mut stored_violations = self.violations.write().await;
            stored_violations.extend(violations.clone());

            // Limit stored violations (keep last 1000)
            if stored_violations.len() > 1000 {
                let drain_count = stored_violations.len() - 1000;
                stored_violations.drain(0..drain_count);
            }
        }

        // Update metrics
        self.update_quality_metrics(stage, stage_result, &stage_scores)
            .await?;

        // Calculate overall score
        let overall_score = if stage_scores.is_empty() {
            1.0
        } else {
            stage_scores.values().sum::<f64>() / stage_scores.len() as f64
        };

        let passed = violations.is_empty()
            || !actions_required
                .iter()
                .any(|action| matches!(action, QualityActionRequired::Block { .. }));

        // Send notifications if needed
        if !violations.is_empty() && self.config.notifications.enabled {
            self.send_quality_notifications(&violations).await?;
        }

        Ok(QualityEvaluationResult {
            passed,
            violations,
            overall_score,
            stage_scores,
            actions_required,
            remediation_suggestions,
        })
    }

    /// Evaluate a single quality gate
    async fn evaluate_gate(
        &self,
        gate: &QualityGate,
        stage: Stage,
        stage_result: &StageResult,
    ) -> Result<GateEvaluationResult> {
        let mut violations = Vec::new();
        let mut criterion_scores = Vec::new();
        let mut failure_reasons = Vec::new();

        for criterion in &gate.criteria {
            let criterion_result = self
                .evaluate_criterion(criterion, stage, stage_result)
                .await?;
            criterion_scores.push(criterion_result.score * criterion.weight);

            if !criterion_result.passed {
                let violation = QualityViolation {
                    id: Uuid::new_v4().to_string(),
                    gate_id: gate.id.clone(),
                    criterion_id: criterion.id.clone(),
                    stage,
                    violation_type: criterion_result.failure_type.clone(),
                    severity: self.determine_violation_severity(
                        &criterion_result.failure_type,
                        criterion_result.score,
                    ),
                    actual_value: criterion_result.actual_value,
                    expected_value: criterion_result.expected_value,
                    timestamp: chrono::Utc::now(),
                    conversation_id: stage_result.conversation_id.clone(),
                    stage_id: stage_result.stage_id.clone(),
                    remediation_attempted: false,
                    remediation_successful: None,
                    action_taken: gate.failure_action.clone(),
                    metadata: HashMap::new(),
                };

                violations.push(violation);
                failure_reasons.push(criterion_result.failure_reason);

                // If criterion is required and failed, gate fails
                if criterion.required {
                    return Ok(GateEvaluationResult {
                        passed: false,
                        score: 0.0,
                        violations,
                        failure_reason: Some(failure_reasons.join("; ")),
                    });
                }
            }
        }

        // Calculate gate score as weighted average
        let total_weight: f64 = gate.criteria.iter().map(|c| c.weight).sum();
        let gate_score = if total_weight > 0.0 {
            criterion_scores.iter().sum::<f64>() / total_weight
        } else {
            1.0
        };

        let passed = violations.is_empty();

        Ok(GateEvaluationResult {
            passed,
            score: gate_score,
            violations,
            failure_reason: if failure_reasons.is_empty() {
                None
            } else {
                Some(failure_reasons.join("; "))
            },
        })
    }

    /// Evaluate a single quality criterion
    async fn evaluate_criterion(
        &self,
        criterion: &QualityCriterion,
        stage: Stage,
        stage_result: &StageResult,
    ) -> Result<CriterionEvaluationResult> {
        let metric_value = self
            .calculate_metric_value(&criterion.metric, stage, stage_result)
            .await?;

        let (passed, failure_type, failure_reason) =
            self.check_threshold(&criterion.threshold, metric_value, &criterion.metric)?;

        let expected_value = criterion
            .threshold
            .target_value
            .or(criterion.threshold.min_value)
            .or(criterion.threshold.max_value)
            .unwrap_or(0.0);

        Ok(CriterionEvaluationResult {
            passed,
            score: if passed {
                1.0
            } else {
                metric_value.max(0.0).min(1.0)
            },
            actual_value: metric_value,
            expected_value,
            failure_type: failure_type.unwrap_or(QualityFailureType::LowQuality),
            failure_reason: failure_reason.unwrap_or_default(),
        })
    }

    /// Calculate the value for a specific quality metric
    async fn calculate_metric_value(
        &self,
        metric: &QualityMetric,
        stage: Stage,
        stage_result: &StageResult,
    ) -> Result<f64> {
        match metric {
            QualityMetric::OverallQuality => {
                // Calculate composite quality score
                Ok(stage_result
                    .analytics
                    .as_ref()
                    .map(|a| a.quality_score)
                    .unwrap_or(0.8))
            }

            QualityMetric::Coherence => {
                // Analyze response coherence (simplified)
                Ok(self.analyze_coherence(&stage_result.answer)?)
            }

            QualityMetric::Completeness => {
                // Check response completeness
                Ok(self.analyze_completeness(&stage_result.question, &stage_result.answer)?)
            }

            QualityMetric::Relevance => {
                // Check relevance to question
                Ok(self.analyze_relevance(&stage_result.question, &stage_result.answer)?)
            }

            QualityMetric::Accuracy => {
                // Check factual accuracy (simplified)
                Ok(self.analyze_accuracy(&stage_result.answer)?)
            }

            QualityMetric::Safety => {
                // Check content safety
                Ok(self.analyze_safety(&stage_result.answer)?)
            }

            QualityMetric::ResponseTime => {
                // Check response time
                Ok(stage_result
                    .analytics
                    .as_ref()
                    .map(|a| {
                        let max_time = self
                            .config
                            .metrics_config
                            .performance_thresholds
                            .max_response_time;
                        (max_time - a.duration).max(0.0) / max_time
                    })
                    .unwrap_or(1.0))
            }

            QualityMetric::TokenEfficiency => {
                // Check token usage efficiency
                Ok(stage_result
                    .usage
                    .as_ref()
                    .map(|u| {
                        let max_tokens =
                            self.config.metrics_config.performance_thresholds.max_tokens as f64;
                        (max_tokens - u.total_tokens as f64).max(0.0) / max_tokens
                    })
                    .unwrap_or(1.0))
            }

            QualityMetric::CostEfficiency => {
                // Check cost efficiency
                Ok(stage_result
                    .analytics
                    .as_ref()
                    .map(|a| {
                        let max_cost = self.config.metrics_config.performance_thresholds.max_cost;
                        (max_cost - a.cost).max(0.0) / max_cost
                    })
                    .unwrap_or(1.0))
            }

            QualityMetric::ErrorRate => {
                // Check error rate
                Ok(stage_result
                    .analytics
                    .as_ref()
                    .map(|a| {
                        let max_error_rate = self
                            .config
                            .metrics_config
                            .performance_thresholds
                            .max_error_rate;
                        (max_error_rate - (a.error_count as f64 / 100.0)).max(0.0) / max_error_rate
                    })
                    .unwrap_or(1.0))
            }

            QualityMetric::ResponseLength => {
                // Check response length
                let length = stage_result.answer.len() as f64;
                let min_length = self.config.metrics_config.content_quality.min_length as f64;
                let max_length = self.config.metrics_config.content_quality.max_length as f64;

                if length < min_length {
                    Ok(length / min_length)
                } else if length > max_length {
                    Ok(max_length / length)
                } else {
                    Ok(1.0)
                }
            }

            QualityMetric::LanguageConsistency => {
                // Check language consistency (simplified)
                Ok(self.analyze_language_consistency(&stage_result.answer)?)
            }

            QualityMetric::KeywordPresence { keywords } => {
                // Check for required keywords
                let answer_lower = stage_result.answer.to_lowercase();
                let present_count = keywords
                    .iter()
                    .filter(|keyword| answer_lower.contains(&keyword.to_lowercase()))
                    .count();

                Ok(if keywords.is_empty() {
                    1.0
                } else {
                    present_count as f64 / keywords.len() as f64
                })
            }

            QualityMetric::Custom {
                name: _,
                evaluator: _,
            } => {
                // Custom metrics would need custom evaluators
                // For now, return a default score
                Ok(0.8)
            }
        }
    }

    /// Check if metric value meets threshold requirements
    fn check_threshold(
        &self,
        threshold: &QualityThreshold,
        value: f64,
        metric: &QualityMetric,
    ) -> Result<(bool, Option<QualityFailureType>, Option<String>)> {
        let mut passed = true;
        let mut failure_type = None;
        let mut failure_reason = None;

        // Check minimum value
        if let Some(min_value) = threshold.min_value {
            if value < min_value {
                passed = false;
                failure_type = Some(QualityFailureType::LowQuality);
                failure_reason = Some(format!(
                    "Value {:.3} below minimum threshold {:.3}",
                    value, min_value
                ));
            }
        }

        // Check maximum value
        if let Some(max_value) = threshold.max_value {
            if value > max_value {
                passed = false;
                failure_type = Some(self.determine_failure_type_for_metric(metric));
                failure_reason = Some(format!(
                    "Value {:.3} exceeds maximum threshold {:.3}",
                    value, max_value
                ));
            }
        }

        // Check target value with tolerance
        if let (Some(target), Some(tolerance)) = (threshold.target_value, threshold.tolerance) {
            if (value - target).abs() > tolerance {
                passed = false;
                failure_type = Some(self.determine_failure_type_for_metric(metric));
                failure_reason = Some(format!(
                    "Value {:.3} deviates from target {:.3} by more than tolerance {:.3}",
                    value, target, tolerance
                ));
            }
        }

        Ok((passed, failure_type, failure_reason))
    }

    /// Determine failure type based on metric
    fn determine_failure_type_for_metric(&self, metric: &QualityMetric) -> QualityFailureType {
        match metric {
            QualityMetric::OverallQuality
            | QualityMetric::Coherence
            | QualityMetric::Completeness
            | QualityMetric::Relevance
            | QualityMetric::Accuracy => QualityFailureType::LowQuality,
            QualityMetric::Safety => QualityFailureType::SafetyViolation,
            QualityMetric::ResponseTime
            | QualityMetric::TokenEfficiency
            | QualityMetric::CostEfficiency
            | QualityMetric::ErrorRate => QualityFailureType::PerformanceDegradation,
            QualityMetric::ResponseLength | QualityMetric::LanguageConsistency => {
                QualityFailureType::FormatViolation
            }
            QualityMetric::KeywordPresence { .. } => QualityFailureType::IncompleteResponse,
            QualityMetric::Custom { .. } => QualityFailureType::Custom {
                failure_type: "custom_metric_failure".to_string(),
            },
        }
    }

    /// Determine violation severity
    fn determine_violation_severity(
        &self,
        failure_type: &QualityFailureType,
        score: f64,
    ) -> QualityViolationSeverity {
        match failure_type {
            QualityFailureType::SafetyViolation => QualityViolationSeverity::Critical,
            QualityFailureType::AccuracyViolation => {
                if score < 0.5 {
                    QualityViolationSeverity::Error
                } else {
                    QualityViolationSeverity::Warning
                }
            }
            QualityFailureType::LowQuality => {
                if score < 0.3 {
                    QualityViolationSeverity::Error
                } else if score < 0.6 {
                    QualityViolationSeverity::Warning
                } else {
                    QualityViolationSeverity::Info
                }
            }
            QualityFailureType::PerformanceDegradation => QualityViolationSeverity::Warning,
            QualityFailureType::FormatViolation => QualityViolationSeverity::Info,
            QualityFailureType::IncompleteResponse => QualityViolationSeverity::Warning,
            QualityFailureType::Custom { .. } => QualityViolationSeverity::Warning,
        }
    }

    // Quality analysis methods (simplified implementations)

    fn analyze_coherence(&self, text: &str) -> Result<f64> {
        // Simplified coherence analysis
        let sentences: Vec<&str> = text.split('.').filter(|s| !s.trim().is_empty()).collect();
        if sentences.is_empty() {
            return Ok(0.0);
        }

        // Basic coherence heuristics
        let avg_sentence_length =
            sentences.iter().map(|s| s.len()).sum::<usize>() as f64 / sentences.len() as f64;

        // Score based on sentence length variation (more varied = more coherent)
        let coherence_score = if avg_sentence_length > 20.0 && avg_sentence_length < 200.0 {
            0.8
        } else {
            0.6
        };

        Ok(coherence_score)
    }

    fn analyze_completeness(&self, question: &str, answer: &str) -> Result<f64> {
        // Simplified completeness analysis
        let question_words: Vec<&str> = question.split_whitespace().collect();
        let answer_words: Vec<&str> = answer.split_whitespace().collect();

        if question_words.is_empty() {
            return Ok(1.0);
        }

        // Check if answer addresses key terms from question
        let key_terms_addressed = question_words
            .iter()
            .filter(|word| word.len() > 3) // Focus on meaningful words
            .filter(|word| answer.to_lowercase().contains(&word.to_lowercase()))
            .count();

        let key_terms_total = question_words.iter().filter(|word| word.len() > 3).count();

        if key_terms_total == 0 {
            return Ok(1.0);
        }

        let completeness_score = key_terms_addressed as f64 / key_terms_total as f64;
        Ok(completeness_score.min(1.0))
    }

    fn analyze_relevance(&self, question: &str, answer: &str) -> Result<f64> {
        // Simplified relevance analysis
        let question_lower = question.to_lowercase();
        let answer_lower = answer.to_lowercase();

        // Extract key terms from question
        let question_terms: Vec<&str> = question_lower
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .take(5) // Focus on first 5 meaningful terms
            .collect();

        if question_terms.is_empty() {
            return Ok(1.0);
        }

        // Count how many question terms appear in answer
        let relevant_terms = question_terms
            .iter()
            .filter(|term| answer_lower.contains(*term))
            .count();

        let relevance_score = relevant_terms as f64 / question_terms.len() as f64;
        Ok(relevance_score)
    }

    fn analyze_accuracy(&self, text: &str) -> Result<f64> {
        // Simplified accuracy analysis
        // Look for confidence indicators and hedging language
        let text_lower = text.to_lowercase();

        let confidence_indicators = [
            "certainly",
            "definitely",
            "proven",
            "confirmed",
            "established",
            "fact",
        ];

        let uncertainty_indicators = [
            "might",
            "maybe",
            "possibly",
            "potentially",
            "could be",
            "seems",
            "appears",
        ];

        let confidence_count = confidence_indicators
            .iter()
            .filter(|indicator| text_lower.contains(*indicator))
            .count();

        let uncertainty_count = uncertainty_indicators
            .iter()
            .filter(|indicator| text_lower.contains(*indicator))
            .count();

        // Simple scoring based on confidence vs uncertainty
        let accuracy_score = if confidence_count > uncertainty_count {
            0.8
        } else if uncertainty_count > confidence_count {
            0.6
        } else {
            0.7
        };

        Ok(accuracy_score)
    }

    fn analyze_safety(&self, text: &str) -> Result<f64> {
        // Simplified safety analysis
        let forbidden_keywords = &self
            .config
            .metrics_config
            .content_quality
            .forbidden_keywords;

        let text_lower = text.to_lowercase();
        let violations = forbidden_keywords
            .iter()
            .filter(|keyword| text_lower.contains(&keyword.to_lowercase()))
            .count();

        if violations > 0 {
            Ok(0.0) // Any safety violation results in 0 score
        } else {
            Ok(1.0)
        }
    }

    fn analyze_language_consistency(&self, text: &str) -> Result<f64> {
        // Simplified language consistency check
        // In a real implementation, this would use language detection libraries

        if let Some(expected_lang) = &self
            .config
            .metrics_config
            .content_quality
            .language_detection
            .expected_language
        {
            // For now, just check if text contains expected language patterns
            match expected_lang.as_str() {
                "en" => {
                    // Simple English pattern check
                    let english_indicators = ["the", "and", "or", "is", "are", "was", "were"];
                    let text_lower = text.to_lowercase();
                    let english_count = english_indicators
                        .iter()
                        .filter(|indicator| text_lower.contains(*indicator))
                        .count();

                    Ok(if english_count > 0 { 1.0 } else { 0.5 })
                }
                _ => Ok(0.8), // Default score for other languages
            }
        } else {
            Ok(1.0) // No language requirements
        }
    }

    /// Update quality metrics tracking
    async fn update_quality_metrics(
        &self,
        stage: Stage,
        stage_result: &StageResult,
        stage_scores: &HashMap<String, f64>,
    ) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let stage_name = stage.as_str();

        // Update stage scores
        let overall_score = stage_scores.values().sum::<f64>() / stage_scores.len().max(1) as f64;
        metrics
            .stage_scores
            .entry(stage_name.to_string())
            .or_default()
            .push(overall_score);

        // Update performance metrics
        if let Some(analytics) = &stage_result.analytics {
            let perf_metric = PerformanceMetric {
                response_time: analytics.duration,
                token_usage: stage_result
                    .usage
                    .as_ref()
                    .map(|u| u.total_tokens)
                    .unwrap_or(0),
                cost: analytics.cost,
                error_count: analytics.error_count,
                timestamp: chrono::Utc::now(),
            };

            metrics
                .performance_metrics
                .entry(stage_name.to_string())
                .or_default()
                .push(perf_metric);
        }

        // Update quality trend
        metrics
            .quality_trend
            .data_points
            .push((chrono::Utc::now(), overall_score));

        // Keep only last 100 data points for trend analysis
        if metrics.quality_trend.data_points.len() > 100 {
            let drain_count = metrics.quality_trend.data_points.len() - 100;
            metrics.quality_trend.data_points.drain(0..drain_count);
        }

        // Recalculate trend
        self.calculate_quality_trend(&mut metrics.quality_trend);

        Ok(())
    }

    /// Calculate quality trend direction and strength
    fn calculate_quality_trend(&self, trend: &mut QualityTrend) {
        if trend.data_points.len() < 2 {
            trend.current_average = trend
                .data_points
                .first()
                .map(|(_, score)| *score)
                .unwrap_or(0.0);
            trend.trend_direction = TrendDirection::Stable;
            trend.trend_strength = 0.0;
            return;
        }

        // Calculate current average
        let total_score: f64 = trend.data_points.iter().map(|(_, score)| score).sum();
        trend.current_average = total_score / trend.data_points.len() as f64;

        // Calculate trend using linear regression slope
        let n = trend.data_points.len() as f64;
        let sum_x: f64 = (0..trend.data_points.len()).map(|i| i as f64).sum();
        let sum_y: f64 = trend.data_points.iter().map(|(_, score)| score).sum();
        let sum_xy: f64 = trend
            .data_points
            .iter()
            .enumerate()
            .map(|(i, (_, score))| i as f64 * score)
            .sum();
        let sum_x2: f64 = (0..trend.data_points.len())
            .map(|i| (i as f64).powi(2))
            .sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        // Determine trend direction and strength
        if slope.abs() < 0.001 {
            trend.trend_direction = TrendDirection::Stable;
            trend.trend_strength = 0.0;
        } else if slope > 0.0 {
            trend.trend_direction = TrendDirection::Improving;
            trend.trend_strength = slope.min(1.0);
        } else {
            trend.trend_direction = TrendDirection::Declining;
            trend.trend_strength = (-slope).min(1.0);
        }
    }

    /// Create approval request for quality gate failure
    async fn create_quality_approval_request(
        &self,
        gate_id: &str,
        gate_name: &str,
        stage: Stage,
        stage_result: &StageResult,
        violations: &[QualityViolation],
    ) -> Result<ApprovalRequest> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "gate_id".to_string(),
            serde_json::Value::String(gate_id.to_string()),
        );
        metadata.insert(
            "gate_name".to_string(),
            serde_json::Value::String(gate_name.to_string()),
        );
        metadata.insert(
            "stage".to_string(),
            serde_json::Value::String(stage.as_str().to_string()),
        );
        metadata.insert(
            "conversation_id".to_string(),
            serde_json::Value::String(stage_result.conversation_id.clone()),
        );
        metadata.insert(
            "violations_count".to_string(),
            serde_json::Value::Number(violations.len().into()),
        );

        Ok(ApprovalRequest {
            id: Uuid::new_v4().to_string(),
            hook_id: HookId(gate_id.to_string()),
            request_type: "quality_gate_failure".to_string(),
            description: format!(
                "Quality gate '{}' failed for {} stage with {} violation(s)",
                gate_name,
                stage.as_str(),
                violations.len()
            ),
            requested_by: "quality_system".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::seconds(300)), // 5 minutes
            metadata,
            priority: super::approval_workflow::ApprovalPriority::High,
            required_approvers: vec!["quality_manager".to_string()],
            received_approvals: Vec::new(),
            current_escalation_level: 0,
            notification_count: 0,
            last_notification_at: None,
        })
    }

    /// Generate remediation suggestions for violations
    async fn generate_remediation_suggestions(
        &self,
        violations: &[QualityViolation],
    ) -> Result<Vec<RemediationSuggestion>> {
        let mut suggestions = Vec::new();

        for violation in violations {
            let applicable_strategies: Vec<&RemediationStrategy> = self
                .config
                .remediation
                .strategies
                .iter()
                .filter(|strategy| {
                    strategy
                        .applicable_failures
                        .contains(&violation.violation_type)
                })
                .collect();

            for strategy in applicable_strategies {
                suggestions.push(RemediationSuggestion {
                    id: Uuid::new_v4().to_string(),
                    strategy_id: strategy.id.clone(),
                    strategy_name: strategy.name.clone(),
                    violation_id: violation.id.clone(),
                    action: strategy.action.clone(),
                    success_probability: strategy.success_rate,
                    estimated_effort: self.estimate_remediation_effort(&strategy.action),
                    priority: strategy.priority,
                });
            }
        }

        // Sort by priority and success probability
        suggestions.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then(
                b.success_probability
                    .partial_cmp(&a.success_probability)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });

        Ok(suggestions)
    }

    /// Estimate effort required for remediation action
    fn estimate_remediation_effort(&self, action: &RemediationAction) -> RemediationEffort {
        match action {
            RemediationAction::Retry => RemediationEffort::Low,
            RemediationAction::RetryWithAdjustment { .. } => RemediationEffort::Medium,
            RemediationAction::SwitchModel { .. } => RemediationEffort::Medium,
            RemediationAction::AdjustPrompt { .. } => RemediationEffort::Medium,
            RemediationAction::RequestHumanIntervention => RemediationEffort::High,
            RemediationAction::UseFallback { .. } => RemediationEffort::Low,
        }
    }

    /// Send quality notifications
    async fn send_quality_notifications(&self, violations: &[QualityViolation]) -> Result<()> {
        for violation in violations {
            if violation.severity >= self.config.notifications.min_severity {
                for channel in &self.config.notifications.channels {
                    self.send_quality_notification(channel, violation).await?;
                }
            }
        }
        Ok(())
    }

    /// Send a single quality notification
    async fn send_quality_notification(
        &self,
        channel: &NotificationChannel,
        violation: &QualityViolation,
    ) -> Result<()> {
        let message = format!(
            "Quality violation detected: {:?} in {} stage (Severity: {:?})",
            violation.violation_type,
            violation.stage.as_str(),
            violation.severity
        );

        match channel {
            NotificationChannel::Console => {
                println!("🚨 QUALITY ALERT: {}", message);
            }
            NotificationChannel::Log => {
                tracing::warn!("Quality violation: {}", message);
            }
            NotificationChannel::Email { recipients: _ } => {
                // Email implementation would go here
                tracing::info!("Email notification sent: {}", message);
            }
            NotificationChannel::Webhook { url: _ } => {
                // Webhook implementation would go here
                tracing::info!("Webhook notification sent: {}", message);
            }
        }

        Ok(())
    }

    /// Get quality statistics
    pub async fn get_quality_statistics(&self) -> Result<QualityStatistics> {
        let metrics = self.metrics.read().await;
        let violations = self.violations.read().await;

        let total_violations = violations.len();
        let violations_by_type = violations.iter().fold(HashMap::new(), |mut acc, v| {
            *acc.entry(format!("{:?}", v.violation_type)).or_insert(0) += 1;
            acc
        });

        let violations_by_severity = violations.iter().fold(HashMap::new(), |mut acc, v| {
            *acc.entry(v.severity.clone()).or_insert(0) += 1;
            acc
        });

        Ok(QualityStatistics {
            total_violations,
            violations_by_type,
            violations_by_severity,
            quality_trend: metrics.quality_trend.clone(),
            average_scores_by_stage: metrics
                .stage_scores
                .iter()
                .map(|(stage, scores)| {
                    let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                    (stage.clone(), avg)
                })
                .collect(),
        })
    }
}

/// Result of quality evaluation
#[derive(Debug, Clone)]
pub struct QualityEvaluationResult {
    pub passed: bool,
    pub violations: Vec<QualityViolation>,
    pub overall_score: f64,
    pub stage_scores: HashMap<String, f64>,
    pub actions_required: Vec<QualityActionRequired>,
    pub remediation_suggestions: Vec<RemediationSuggestion>,
}

/// Actions required based on quality evaluation
#[derive(Debug, Clone)]
pub enum QualityActionRequired {
    Block {
        gate_id: String,
        reason: String,
    },
    Warn {
        message: String,
    },
    RequestApproval {
        approval_request: ApprovalRequest,
    },
    Remediate {
        strategies: Vec<RemediationSuggestion>,
    },
    Custom {
        action_id: String,
        gate_id: String,
    },
}

/// Remediation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSuggestion {
    pub id: String,
    pub strategy_id: String,
    pub strategy_name: String,
    pub violation_id: String,
    pub action: RemediationAction,
    pub success_probability: f64,
    pub estimated_effort: RemediationEffort,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemediationEffort {
    Low,
    Medium,
    High,
}

/// Result of evaluating a single gate
#[derive(Debug, Clone)]
struct GateEvaluationResult {
    passed: bool,
    score: f64,
    violations: Vec<QualityViolation>,
    failure_reason: Option<String>,
}

/// Result of evaluating a single criterion
#[derive(Debug, Clone)]
struct CriterionEvaluationResult {
    passed: bool,
    score: f64,
    actual_value: f64,
    expected_value: f64,
    failure_type: QualityFailureType,
    failure_reason: String,
}

/// Quality statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStatistics {
    pub total_violations: usize,
    pub violations_by_type: HashMap<String, usize>,
    pub violations_by_severity: HashMap<QualityViolationSeverity, usize>,
    pub quality_trend: QualityTrend,
    pub average_scores_by_stage: HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::types::{StageAnalytics, TokenUsage};

    #[tokio::test]
    async fn test_quality_gate_manager_creation() {
        let config = QualityGateConfig::default();
        let manager = QualityGateManager::new(config).await.unwrap();

        let stats = manager.get_quality_statistics().await.unwrap();
        assert_eq!(stats.total_violations, 0);
    }

    #[tokio::test]
    async fn test_quality_gate_addition() {
        let config = QualityGateConfig::default();
        let manager = QualityGateManager::new(config).await.unwrap();

        let gate = QualityGate {
            id: "test_gate".to_string(),
            name: "Test Gate".to_string(),
            description: "Test quality gate".to_string(),
            stage: Some(Stage::Generator),
            criteria: vec![QualityCriterion {
                id: "test_criterion".to_string(),
                name: "Test Criterion".to_string(),
                metric: QualityMetric::OverallQuality,
                threshold: QualityThreshold {
                    min_value: Some(0.7),
                    max_value: None,
                    target_value: Some(0.9),
                    tolerance: Some(0.1),
                },
                weight: 1.0,
                required: true,
            }],
            failure_action: QualityGateAction::Warn,
            enabled: true,
            priority: 1,
            tags: vec!["test".to_string()],
        };

        manager.add_gate(gate).await.unwrap();

        // Verify gate was added
        let gates = manager.gates.read().await;
        assert_eq!(gates.len(), 1);
        assert_eq!(gates[0].id, "test_gate");
    }

    #[tokio::test]
    async fn test_quality_evaluation() {
        let config = QualityGateConfig::default();
        let manager = QualityGateManager::new(config).await.unwrap();

        // Add a test gate
        let gate = QualityGate {
            id: "test_gate".to_string(),
            name: "Test Gate".to_string(),
            description: "Test quality gate".to_string(),
            stage: Some(Stage::Generator),
            criteria: vec![QualityCriterion {
                id: "test_criterion".to_string(),
                name: "Test Criterion".to_string(),
                metric: QualityMetric::OverallQuality,
                threshold: QualityThreshold {
                    min_value: Some(0.5),
                    max_value: None,
                    target_value: None,
                    tolerance: None,
                },
                weight: 1.0,
                required: false,
            }],
            failure_action: QualityGateAction::Warn,
            enabled: true,
            priority: 1,
            tags: vec!["test".to_string()],
        };

        manager.add_gate(gate).await.unwrap();

        // Create a test stage result
        let stage_result = StageResult {
            stage_id: "test_stage".to_string(),
            stage_name: "generator".to_string(),
            question: "What is Rust?".to_string(),
            answer:
                "Rust is a systems programming language focused on safety, speed, and concurrency."
                    .to_string(),
            model: "test_model".to_string(),
            conversation_id: "test_conversation".to_string(),
            timestamp: chrono::Utc::now(),
            usage: Some(TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            analytics: Some(StageAnalytics {
                duration: 1.5,
                cost: 0.01,
                provider: "test".to_string(),
                model_internal_id: "test_model".to_string(),
                quality_score: 0.8,
                error_count: 0,
                fallback_used: false,
                rate_limit_hit: false,
                retry_count: 0,
                start_time: chrono::Utc::now(),
                end_time: chrono::Utc::now(),
                memory_usage: None,
                features: crate::consensus::types::AnalyticsFeatures {
                    streaming: true,
                    routing_variant: "test".to_string(),
                    optimization_applied: Some(true),
                },
            }),
        };

        // Evaluate the stage result
        let result = manager
            .evaluate_stage_result(Stage::Generator, &stage_result)
            .await
            .unwrap();

        // Should pass since quality score (0.8) is above threshold (0.5)
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
        assert!(result.overall_score > 0.5);
    }
}
