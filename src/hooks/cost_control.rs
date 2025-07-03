//! Cost Control System
//! 
//! Provides enterprise-grade cost management and budget enforcement for consensus operations.
//! Supports cost thresholds, budget alerts, approval workflows, and cost optimization.

use super::{
    HookAuditLogger, AuditEvent, AuditEventType,
    approval_workflow::{ApprovalWorkflow, ApprovalRequest, ApprovalStatus},
    registry::HookId,
};
use crate::consensus::types::{Stage, StageResult};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cost control manager
pub struct CostController {
    budgets: Arc<RwLock<HashMap<String, Budget>>>,
    cost_tracking: Arc<RwLock<CostTracking>>,
    thresholds: Arc<RwLock<Vec<CostThreshold>>>,
    approval_workflow: Option<Arc<ApprovalWorkflow>>,
    audit_logger: Option<Arc<HookAuditLogger>>,
    config: CostControlConfig,
    alerts: Arc<RwLock<Vec<CostAlert>>>,
}

/// Configuration for cost control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostControlConfig {
    /// Whether cost control is enabled
    pub enabled: bool,
    
    /// Default budget period in days
    pub default_budget_period_days: u32,
    
    /// Cost tracking granularity
    pub tracking_granularity: CostTrackingGranularity,
    
    /// Alert settings
    pub alerts: CostAlertConfig,
    
    /// Approval settings
    pub approvals: CostApprovalConfig,
    
    /// Optimization settings
    pub optimization: CostOptimizationConfig,
    
    /// Budget enforcement level
    pub enforcement_level: BudgetEnforcementLevel,
    
    /// Cost estimation settings
    pub estimation: CostEstimationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostTrackingGranularity {
    /// Track costs per individual operation
    PerOperation,
    
    /// Track costs per conversation
    PerConversation,
    
    /// Track costs per stage
    PerStage,
    
    /// Track costs per hour
    Hourly,
    
    /// Track costs per day
    Daily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAlertConfig {
    /// Enable cost alerts
    pub enabled: bool,
    
    /// Alert channels
    pub channels: Vec<AlertChannel>,
    
    /// Minimum cost increase to trigger alert
    pub min_cost_threshold: f64,
    
    /// Alert frequency limits
    pub rate_limiting: AlertRateLimiting,
    
    /// Escalation settings
    pub escalation: AlertEscalationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostApprovalConfig {
    /// Enable approval requirements for high costs
    pub enabled: bool,
    
    /// Cost threshold requiring approval
    pub approval_threshold: f64,
    
    /// Emergency approval threshold (requires immediate approval)
    pub emergency_threshold: f64,
    
    /// Auto-approval rules
    pub auto_approval_rules: Vec<AutoApprovalRule>,
    
    /// Approval timeout in seconds
    pub approval_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimizationConfig {
    /// Enable automatic cost optimization
    pub enabled: bool,
    
    /// Optimization strategies
    pub strategies: Vec<OptimizationStrategy>,
    
    /// Cost efficiency targets
    pub efficiency_targets: EfficiencyTargets,
    
    /// Model selection preferences
    pub model_selection: ModelSelectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetEnforcementLevel {
    /// Log cost overruns but allow operations
    LogOnly,
    
    /// Warn about cost overruns but allow operations
    Warn,
    
    /// Require approval for cost overruns
    RequireApproval,
    
    /// Block operations that would exceed budget
    Strict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimationConfig {
    /// Enable pre-operation cost estimation
    pub enabled: bool,
    
    /// Estimation accuracy target (0.0 - 1.0)
    pub accuracy_target: f64,
    
    /// Historical data window for estimates (days)
    pub historical_window_days: u32,
    
    /// Model-specific cost factors
    pub model_cost_factors: HashMap<String, ModelCostFactor>,
    
    /// Buffer factor for estimates (1.1 = 10% buffer)
    pub estimation_buffer: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Console,
    Log,
    Email { recipients: Vec<String> },
    Webhook { url: String, headers: Option<HashMap<String, String>> },
    Slack { webhook_url: String, channel: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRateLimiting {
    /// Maximum alerts per hour
    pub max_alerts_per_hour: u32,
    
    /// Cooldown period between similar alerts (seconds)
    pub cooldown_seconds: u64,
    
    /// Alert deduplication window (seconds)
    pub deduplication_window_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEscalationConfig {
    /// Enable alert escalation
    pub enabled: bool,
    
    /// Escalation levels
    pub levels: Vec<EscalationLevel>,
    
    /// Time between escalation levels (seconds)
    pub escalation_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub channels: Vec<AlertChannel>,
    pub required_acknowledgment: bool,
    pub auto_escalate_after_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApprovalRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub conditions: Vec<ApprovalCondition>,
    pub max_auto_approve_amount: f64,
    pub daily_limit: Option<f64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Contains,
    In,
    NotIn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub optimization_type: OptimizationType,
    pub potential_savings: f64, // Estimated savings as a percentage
    pub implementation_effort: OptimizationEffort,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Use more cost-effective models for certain stages
    ModelOptimization,
    
    /// Optimize prompt length and structure
    PromptOptimization,
    
    /// Use caching to reduce redundant calls
    CachingOptimization,
    
    /// Batch operations for efficiency
    BatchingOptimization,
    
    /// Route to cheaper providers when possible
    ProviderOptimization,
    
    /// Custom optimization strategy
    Custom { strategy_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyTargets {
    /// Target cost per token
    pub cost_per_token: f64,
    
    /// Target cost per operation
    pub cost_per_operation: f64,
    
    /// Target cost per quality point
    pub cost_per_quality_point: f64,
    
    /// Maximum acceptable cost variance
    pub max_cost_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionConfig {
    /// Prefer cost-effective models
    pub prefer_cost_effective: bool,
    
    /// Model cost ranking (cheaper first)
    pub cost_ranking: Vec<String>,
    
    /// Quality vs cost trade-off preference (0.0 = cost only, 1.0 = quality only)
    pub quality_cost_balance: f64,
    
    /// Minimum quality threshold for model selection
    pub min_quality_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostFactor {
    /// Base cost multiplier for this model
    pub cost_multiplier: f64,
    
    /// Expected tokens per operation
    pub expected_tokens: u32,
    
    /// Quality score factor
    pub quality_factor: f64,
    
    /// Performance factor (speed)
    pub performance_factor: f64,
}

/// Budget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: String,
    pub name: String,
    pub description: String,
    pub amount: f64,
    pub period: BudgetPeriod,
    pub scope: BudgetScope,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub current_usage: f64,
    pub alerts_sent: u32,
    pub status: BudgetStatus,
    pub created_by: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom { duration_days: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetScope {
    /// Budget applies to all operations
    Global,
    
    /// Budget applies to specific user or team
    User { user_id: String },
    
    /// Budget applies to specific project
    Project { project_id: String },
    
    /// Budget applies to specific model
    Model { model_name: String },
    
    /// Budget applies to specific stage
    Stage { stage: Stage },
    
    /// Custom scope with conditions
    Custom { conditions: Vec<BudgetCondition> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetStatus {
    Active,
    Exceeded,
    Suspended,
    Expired,
}

/// Cost threshold definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostThreshold {
    pub id: String,
    pub name: String,
    pub description: String,
    pub threshold_type: ThresholdType,
    pub amount: f64,
    pub period: Option<ThresholdPeriod>,
    pub action: ThresholdAction,
    pub enabled: bool,
    pub scope: ThresholdScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdType {
    /// Absolute cost amount
    Absolute,
    
    /// Percentage of budget
    BudgetPercentage,
    
    /// Rate of cost increase
    RateIncrease,
    
    /// Cost per unit (e.g., per token, per operation)
    PerUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdPeriod {
    PerOperation,
    PerHour,
    PerDay,
    PerWeek,
    PerMonth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdAction {
    /// Log the threshold breach
    Log,
    
    /// Send alert notification
    Alert,
    
    /// Request approval before proceeding
    RequestApproval,
    
    /// Block the operation
    Block,
    
    /// Switch to cheaper model/provider
    OptimizeModel,
    
    /// Custom action
    Custom { action_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdScope {
    Global,
    PerUser,
    PerProject,
    PerModel,
    PerStage,
}

/// Cost tracking data
#[derive(Debug, Clone, Default)]
pub struct CostTracking {
    /// Total costs by time period
    pub costs_by_period: HashMap<String, f64>,
    
    /// Costs by operation type
    pub costs_by_operation: HashMap<String, f64>,
    
    /// Costs by model
    pub costs_by_model: HashMap<String, f64>,
    
    /// Costs by stage
    pub costs_by_stage: HashMap<String, f64>,
    
    /// Costs by user/session
    pub costs_by_user: HashMap<String, f64>,
    
    /// Cost trend data
    pub cost_trends: Vec<CostDataPoint>,
    
    /// Efficiency metrics
    pub efficiency_metrics: EfficiencyMetrics,
}

#[derive(Debug, Clone)]
pub struct CostDataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cost: f64,
    pub operation_type: String,
    pub model: String,
    pub stage: String,
    pub tokens_used: u32,
    pub quality_score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct EfficiencyMetrics {
    pub average_cost_per_token: f64,
    pub average_cost_per_operation: f64,
    pub average_cost_per_quality_point: f64,
    pub cost_efficiency_trend: EfficiencyTrend,
    pub model_efficiency_rankings: Vec<ModelEfficiency>,
}

#[derive(Debug, Clone)]
pub struct ModelEfficiency {
    pub model_name: String,
    pub cost_efficiency_score: f64,
    pub quality_efficiency_score: f64,
    pub overall_efficiency_score: f64,
    pub usage_count: u32,
    pub total_cost: f64,
    pub average_quality: f64,
}

#[derive(Debug, Clone, Default)]
pub enum EfficiencyTrend {
    #[default]
    Stable,
    Improving,
    Declining,
}

/// Cost alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAlert {
    pub id: String,
    pub alert_type: CostAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub cost_amount: f64,
    pub threshold_amount: Option<f64>,
    pub budget_id: Option<String>,
    pub scope: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
    pub escalation_level: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostAlertType {
    /// Budget threshold exceeded
    BudgetThreshold,
    
    /// Cost threshold exceeded
    CostThreshold,
    
    /// Unusual cost spike detected
    CostSpike,
    
    /// Budget exhausted
    BudgetExhausted,
    
    /// Cost efficiency degradation
    EfficiencyDegradation,
    
    /// Model cost anomaly
    ModelCostAnomaly,
    
    /// Approval required
    ApprovalRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for CostControlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_budget_period_days: 30,
            tracking_granularity: CostTrackingGranularity::PerOperation,
            alerts: CostAlertConfig::default(),
            approvals: CostApprovalConfig::default(),
            optimization: CostOptimizationConfig::default(),
            enforcement_level: BudgetEnforcementLevel::Warn,
            estimation: CostEstimationConfig::default(),
        }
    }
}

impl Default for CostAlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![AlertChannel::Console, AlertChannel::Log],
            min_cost_threshold: 0.01,
            rate_limiting: AlertRateLimiting {
                max_alerts_per_hour: 10,
                cooldown_seconds: 300,
                deduplication_window_seconds: 60,
            },
            escalation: AlertEscalationConfig {
                enabled: false,
                levels: vec![],
                escalation_interval_seconds: 600,
            },
        }
    }
}

impl Default for CostApprovalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            approval_threshold: 1.0, // $1.00
            emergency_threshold: 10.0, // $10.00
            auto_approval_rules: vec![],
            approval_timeout_seconds: 300, // 5 minutes
        }
    }
}

impl Default for CostOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategies: vec![
                OptimizationStrategy {
                    id: "model_optimization".to_string(),
                    name: "Model Optimization".to_string(),
                    description: "Use more cost-effective models when quality allows".to_string(),
                    optimization_type: OptimizationType::ModelOptimization,
                    potential_savings: 0.3, // 30% potential savings
                    implementation_effort: OptimizationEffort::Medium,
                    enabled: true,
                },
                OptimizationStrategy {
                    id: "prompt_optimization".to_string(),
                    name: "Prompt Optimization".to_string(),
                    description: "Optimize prompt length and structure for efficiency".to_string(),
                    optimization_type: OptimizationType::PromptOptimization,
                    potential_savings: 0.15, // 15% potential savings
                    implementation_effort: OptimizationEffort::Low,
                    enabled: true,
                },
            ],
            efficiency_targets: EfficiencyTargets {
                cost_per_token: 0.00002, // $0.00002 per token
                cost_per_operation: 0.10, // $0.10 per operation
                cost_per_quality_point: 0.15, // $0.15 per quality point
                max_cost_variance: 0.20, // 20% variance allowed
            },
            model_selection: ModelSelectionConfig {
                prefer_cost_effective: true,
                cost_ranking: vec![
                    "gpt-3.5-turbo".to_string(),
                    "claude-3-haiku".to_string(),
                    "gpt-4".to_string(),
                    "claude-3-sonnet".to_string(),
                    "gpt-4-turbo".to_string(),
                    "claude-3-opus".to_string(),
                ],
                quality_cost_balance: 0.7, // Prefer quality but consider cost
                min_quality_threshold: 0.6,
            },
        }
    }
}

impl Default for CostEstimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            accuracy_target: 0.85, // 85% accuracy target
            historical_window_days: 30,
            model_cost_factors: HashMap::new(),
            estimation_buffer: 1.1, // 10% buffer
        }
    }
}

impl CostController {
    /// Create a new cost controller
    pub async fn new(config: CostControlConfig) -> Result<Self> {
        Ok(Self {
            budgets: Arc::new(RwLock::new(HashMap::new())),
            cost_tracking: Arc::new(RwLock::new(CostTracking::default())),
            thresholds: Arc::new(RwLock::new(Vec::new())),
            approval_workflow: None,
            audit_logger: None,
            config,
            alerts: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Set approval workflow
    pub fn with_approval_workflow(mut self, workflow: Arc<ApprovalWorkflow>) -> Self {
        self.approval_workflow = Some(workflow);
        self
    }
    
    /// Set audit logger
    pub fn with_audit_logger(mut self, logger: Arc<HookAuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }
    
    /// Create a new budget
    pub async fn create_budget(&self, mut budget: Budget) -> Result<String> {
        budget.id = Uuid::new_v4().to_string();
        budget.status = BudgetStatus::Active;
        budget.current_usage = 0.0;
        budget.alerts_sent = 0;
        
        let budget_id = budget.id.clone();
        
        {
            let mut budgets = self.budgets.write().await;
            budgets.insert(budget_id.clone(), budget.clone());
        }
        
        // Log budget creation
        if let Some(audit_logger) = &self.audit_logger {
            let event = AuditEventType::HookRegistered {
                hook_id: HookId(budget_id.clone()),
                hook_name: budget.name.clone(),
            };
            
            let audit_event = AuditEvent::new(event)
                .with_context("type", "budget")
                .with_context("amount", budget.amount)
                .with_context("created_by", &budget.created_by);
            audit_logger.log_event(audit_event).await?;
        }
        
        Ok(budget_id)
    }
    
    /// Update budget usage
    pub async fn update_budget_usage(&self, budget_id: &str, cost: f64) -> Result<BudgetUpdateResult> {
        let mut budgets = self.budgets.write().await;
        let budget = budgets.get_mut(budget_id)
            .ok_or_else(|| anyhow::anyhow!("Budget not found: {}", budget_id))?;
        
        budget.current_usage += cost;
        
        let usage_percentage = budget.current_usage / budget.amount;
        let mut alerts_triggered = Vec::new();
        
        // Check for threshold breaches
        if usage_percentage >= 1.0 && budget.status != BudgetStatus::Exceeded {
            budget.status = BudgetStatus::Exceeded;
            alerts_triggered.push(self.create_budget_alert(
                budget,
                CostAlertType::BudgetExhausted,
                AlertSeverity::Critical,
                format!("Budget '{}' has been exhausted (${:.2})", budget.name, budget.current_usage),
            ).await?);
        } else if usage_percentage >= 0.9 && budget.alerts_sent < 3 {
            budget.alerts_sent += 1;
            alerts_triggered.push(self.create_budget_alert(
                budget,
                CostAlertType::BudgetThreshold,
                AlertSeverity::Warning,
                format!("Budget '{}' is 90% used (${:.2} of ${:.2})", budget.name, budget.current_usage, budget.amount),
            ).await?);
        } else if usage_percentage >= 0.75 && budget.alerts_sent < 2 {
            budget.alerts_sent += 1;
            alerts_triggered.push(self.create_budget_alert(
                budget,
                CostAlertType::BudgetThreshold,
                AlertSeverity::Warning,
                format!("Budget '{}' is 75% used (${:.2} of ${:.2})", budget.name, budget.current_usage, budget.amount),
            ).await?);
        } else if usage_percentage >= 0.5 && budget.alerts_sent < 1 {
            budget.alerts_sent += 1;
            alerts_triggered.push(self.create_budget_alert(
                budget,
                CostAlertType::BudgetThreshold,
                AlertSeverity::Info,
                format!("Budget '{}' is 50% used (${:.2} of ${:.2})", budget.name, budget.current_usage, budget.amount),
            ).await?);
        }
        
        // Send alerts if any were triggered
        for alert in &alerts_triggered {
            self.send_cost_alert(alert).await?;
        }
        
        Ok(BudgetUpdateResult {
            budget_id: budget_id.to_string(),
            previous_usage: budget.current_usage - cost,
            new_usage: budget.current_usage,
            usage_percentage,
            status: budget.status.clone(),
            alerts_triggered,
        })
    }
    
    /// Estimate cost for an operation
    pub async fn estimate_operation_cost(
        &self,
        stage: Stage,
        model: &str,
        estimated_tokens: u32,
        context: &CostEstimationContext,
    ) -> Result<CostEstimate> {
        if !self.config.estimation.enabled {
            return Ok(CostEstimate {
                estimated_cost: 0.0,
                confidence: 0.0,
                factors: HashMap::new(),
                recommendations: Vec::new(),
            });
        }
        
        // Get model cost factor
        let model_factor = self.config.estimation.model_cost_factors
            .get(model)
            .cloned()
            .unwrap_or(ModelCostFactor {
                cost_multiplier: 1.0,
                expected_tokens: estimated_tokens,
                quality_factor: 1.0,
                performance_factor: 1.0,
            });
        
        // Base cost calculation (simplified)
        let base_cost_per_token = match model {
            m if m.contains("gpt-4") => 0.00003,
            m if m.contains("gpt-3.5") => 0.000002,
            m if m.contains("claude-3-opus") => 0.000075,
            m if m.contains("claude-3-sonnet") => 0.000015,
            m if m.contains("claude-3-haiku") => 0.000001,
            _ => 0.00001, // Default
        };
        
        let estimated_cost = (estimated_tokens as f64 * base_cost_per_token * model_factor.cost_multiplier) 
            * self.config.estimation.estimation_buffer;
        
        // Calculate confidence based on historical data
        let confidence = self.calculate_estimation_confidence(model, stage, context).await?;
        
        // Generate cost factors breakdown
        let mut factors = HashMap::new();
        factors.insert("base_cost".to_string(), estimated_tokens as f64 * base_cost_per_token);
        factors.insert("model_multiplier".to_string(), model_factor.cost_multiplier);
        factors.insert("buffer_factor".to_string(), self.config.estimation.estimation_buffer);
        factors.insert("estimated_tokens".to_string(), estimated_tokens as f64);
        
        // Generate recommendations
        let recommendations = self.generate_cost_recommendations(estimated_cost, model, stage, context).await?;
        
        Ok(CostEstimate {
            estimated_cost,
            confidence,
            factors,
            recommendations,
        })
    }
    
    /// Check if operation should be approved based on cost
    pub async fn check_cost_approval_required(
        &self,
        estimated_cost: f64,
        context: &CostEstimationContext,
    ) -> Result<Option<CostApprovalRequirement>> {
        if !self.config.approvals.enabled {
            return Ok(None);
        }
        
        // Check emergency threshold
        if estimated_cost > self.config.approvals.emergency_threshold {
            return Ok(Some(CostApprovalRequirement {
                approval_type: CostApprovalType::Emergency,
                estimated_cost,
                threshold: self.config.approvals.emergency_threshold,
                reason: format!("Cost ${:.4} exceeds emergency threshold ${:.4}", estimated_cost, self.config.approvals.emergency_threshold),
                required_approvers: vec!["finance_manager".to_string(), "senior_manager".to_string()],
                timeout_seconds: 180, // 3 minutes for emergency
            }));
        }
        
        // Check standard approval threshold
        if estimated_cost > self.config.approvals.approval_threshold {
            // Check auto-approval rules first
            if let Some(_rule) = self.find_matching_auto_approval_rule(estimated_cost, context).await? {
                return Ok(None); // Auto-approved
            }
            
            return Ok(Some(CostApprovalRequirement {
                approval_type: CostApprovalType::Standard,
                estimated_cost,
                threshold: self.config.approvals.approval_threshold,
                reason: format!("Cost ${:.4} exceeds approval threshold ${:.4}", estimated_cost, self.config.approvals.approval_threshold),
                required_approvers: vec!["manager".to_string()],
                timeout_seconds: self.config.approvals.approval_timeout_seconds,
            }));
        }
        
        Ok(None)
    }
    
    /// Record actual cost for an operation
    pub async fn record_operation_cost(
        &self,
        stage: Stage,
        stage_result: &StageResult,
        actual_cost: f64,
    ) -> Result<()> {
        let mut tracking = self.cost_tracking.write().await;
        
        // Create cost data point
        let data_point = CostDataPoint {
            timestamp: chrono::Utc::now(),
            cost: actual_cost,
            operation_type: "consensus_stage".to_string(),
            model: stage_result.model.clone(),
            stage: stage.as_str().to_string(),
            tokens_used: stage_result.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
            quality_score: stage_result.analytics.as_ref().map(|a| a.quality_score).unwrap_or(0.0),
        };
        
        // Update tracking data
        let period_key = chrono::Utc::now().format("%Y-%m-%d").to_string();
        *tracking.costs_by_period.entry(period_key).or_insert(0.0) += actual_cost;
        *tracking.costs_by_operation.entry("consensus_stage".to_string()).or_insert(0.0) += actual_cost;
        *tracking.costs_by_model.entry(stage_result.model.clone()).or_insert(0.0) += actual_cost;
        *tracking.costs_by_stage.entry(stage.as_str().to_string()).or_insert(0.0) += actual_cost;
        
        // Add to cost trends
        tracking.cost_trends.push(data_point);
        
        // Limit trend data (keep last 1000 points)
        if tracking.cost_trends.len() > 1000 {
            let drain_count = tracking.cost_trends.len() - 1000;
            tracking.cost_trends.drain(0..drain_count);
        }
        
        // Update efficiency metrics
        let cost_trends = tracking.cost_trends.clone();
        self.update_efficiency_metrics(&mut tracking.efficiency_metrics, &cost_trends).await?;
        
        // Check cost thresholds
        self.check_cost_thresholds(actual_cost, stage, &stage_result.model).await?;
        
        Ok(())
    }
    
    /// Get cost summary for a period
    pub async fn get_cost_summary(&self, period: Option<String>) -> Result<CostSummary> {
        let tracking = self.cost_tracking.read().await;
        let budgets = self.budgets.read().await;
        
        let period_key = period.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        let period_cost = tracking.costs_by_period.get(&period_key).cloned().unwrap_or(0.0);
        
        let total_budget: f64 = budgets.values()
            .filter(|b| b.status == BudgetStatus::Active)
            .map(|b| b.amount)
            .sum();
        
        let total_usage: f64 = budgets.values()
            .filter(|b| b.status == BudgetStatus::Active)
            .map(|b| b.current_usage)
            .sum();
        
        Ok(CostSummary {
            period_cost,
            total_budget,
            total_usage,
            budget_utilization: if total_budget > 0.0 { total_usage / total_budget } else { 0.0 },
            costs_by_model: tracking.costs_by_model.clone(),
            costs_by_stage: tracking.costs_by_stage.clone(),
            efficiency_metrics: tracking.efficiency_metrics.clone(),
            active_budgets: budgets.len(),
            exceeded_budgets: budgets.values().filter(|b| b.status == BudgetStatus::Exceeded).count(),
        })
    }
    
    /// Get cost optimization recommendations
    pub async fn get_optimization_recommendations(&self) -> Result<Vec<CostOptimizationRecommendation>> {
        if !self.config.optimization.enabled {
            return Ok(Vec::new());
        }
        
        let tracking = self.cost_tracking.read().await;
        let mut recommendations = Vec::new();
        
        // Analyze model efficiency
        for model_efficiency in &tracking.efficiency_metrics.model_efficiency_rankings {
            if model_efficiency.cost_efficiency_score < 0.7 {
                recommendations.push(CostOptimizationRecommendation {
                    id: Uuid::new_v4().to_string(),
                    recommendation_type: OptimizationType::ModelOptimization,
                    title: format!("Consider alternative to {}", model_efficiency.model_name),
                    description: format!(
                        "Model {} has low cost efficiency (score: {:.2}). Consider using a more cost-effective alternative.",
                        model_efficiency.model_name, model_efficiency.cost_efficiency_score
                    ),
                    potential_savings: 0.25, // 25% potential savings
                    implementation_effort: OptimizationEffort::Low,
                    priority: if model_efficiency.cost_efficiency_score < 0.5 {
                        RecommendationPriority::High
                    } else {
                        RecommendationPriority::Medium
                    },
                    estimated_impact: EstimatedImpact {
                        cost_reduction: model_efficiency.total_cost * 0.25,
                        quality_impact: -0.05, // Small quality reduction
                        implementation_time_days: 1,
                    },
                });
            }
        }
        
        // Check for cost trends
        if let EfficiencyTrend::Declining = tracking.efficiency_metrics.cost_efficiency_trend {
            recommendations.push(CostOptimizationRecommendation {
                id: Uuid::new_v4().to_string(),
                recommendation_type: OptimizationType::PromptOptimization,
                title: "Optimize prompts for efficiency".to_string(),
                description: "Cost efficiency is declining. Consider optimizing prompts to reduce token usage.".to_string(),
                potential_savings: 0.15,
                implementation_effort: OptimizationEffort::Medium,
                priority: RecommendationPriority::Medium,
                estimated_impact: EstimatedImpact {
                    cost_reduction: tracking.costs_by_operation.values().sum::<f64>() * 0.15,
                    quality_impact: 0.02, // Small quality improvement
                    implementation_time_days: 3,
                },
            });
        }
        
        // Sort by priority and potential savings
        recommendations.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(b.potential_savings.partial_cmp(&a.potential_savings).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        Ok(recommendations)
    }
    
    // Private helper methods
    
    /// Calculate estimation confidence
    async fn calculate_estimation_confidence(
        &self,
        model: &str,
        stage: Stage,
        _context: &CostEstimationContext,
    ) -> Result<f64> {
        let tracking = self.cost_tracking.read().await;
        
        // Count historical data points for this model and stage
        let relevant_points = tracking.cost_trends.iter()
            .filter(|point| point.model == model && point.stage == stage.as_str())
            .count();
        
        // Confidence based on amount of historical data
        let confidence = match relevant_points {
            0 => 0.5, // Low confidence with no historical data
            1..=5 => 0.6,
            6..=20 => 0.8,
            _ => 0.95, // High confidence with lots of data
        };
        
        Ok(confidence)
    }
    
    /// Generate cost recommendations
    async fn generate_cost_recommendations(
        &self,
        estimated_cost: f64,
        model: &str,
        stage: Stage,
        _context: &CostEstimationContext,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        // Check if cost is high
        if estimated_cost > 0.5 {
            recommendations.push("Consider using a more cost-effective model for this operation".to_string());
        }
        
        // Check model efficiency
        if self.config.optimization.model_selection.prefer_cost_effective {
            let cost_ranking = &self.config.optimization.model_selection.cost_ranking;
            if let Some(current_index) = cost_ranking.iter().position(|m| m == model) {
                if current_index > 2 { // Not in top 3 most cost-effective
                    recommendations.push(format!(
                        "Consider using {} instead for better cost efficiency",
                        cost_ranking.get(current_index.saturating_sub(1)).unwrap_or(&cost_ranking[0])
                    ));
                }
            }
        }
        
        // Stage-specific recommendations
        match stage {
            Stage::Generator => {
                if estimated_cost > 0.3 {
                    recommendations.push("Consider simplifying the initial prompt for the generator stage".to_string());
                }
            }
            Stage::Refiner => {
                if estimated_cost > 0.2 {
                    recommendations.push("The refiner stage cost is high - consider skipping if quality is already sufficient".to_string());
                }
            }
            Stage::Validator => {
                if estimated_cost > 0.15 {
                    recommendations.push("Consider using a simpler validation approach to reduce costs".to_string());
                }
            }
            Stage::Curator => {
                if estimated_cost > 0.1 {
                    recommendations.push("Final curation may not be necessary for all use cases".to_string());
                }
            }
        }
        
        Ok(recommendations)
    }
    
    /// Find matching auto-approval rule
    async fn find_matching_auto_approval_rule(
        &self,
        cost: f64,
        _context: &CostEstimationContext,
    ) -> Result<Option<&AutoApprovalRule>> {
        for rule in &self.config.approvals.auto_approval_rules {
            if rule.enabled && cost <= rule.max_auto_approve_amount {
                // In a real implementation, we would evaluate all conditions
                return Ok(Some(rule));
            }
        }
        Ok(None)
    }
    
    /// Create budget alert
    async fn create_budget_alert(
        &self,
        budget: &Budget,
        alert_type: CostAlertType,
        severity: AlertSeverity,
        message: String,
    ) -> Result<CostAlert> {
        let alert = CostAlert {
            id: Uuid::new_v4().to_string(),
            alert_type,
            severity,
            message,
            cost_amount: budget.current_usage,
            threshold_amount: Some(budget.amount),
            budget_id: Some(budget.id.clone()),
            scope: format!("{:?}", budget.scope),
            timestamp: chrono::Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            escalation_level: 0,
            metadata: HashMap::new(),
        };
        
        // Store alert
        {
            let mut alerts = self.alerts.write().await;
            alerts.push(alert.clone());
            
            // Limit stored alerts (keep last 1000)
            if alerts.len() > 1000 {
                let drain_count = alerts.len() - 1000;
                alerts.drain(0..drain_count);
            }
        }
        
        Ok(alert)
    }
    
    /// Send cost alert
    async fn send_cost_alert(&self, alert: &CostAlert) -> Result<()> {
        if !self.config.alerts.enabled {
            return Ok(());
        }
        
        for channel in &self.config.alerts.channels {
            self.send_alert_to_channel(channel, alert).await?;
        }
        
        Ok(())
    }
    
    /// Send alert to specific channel
    async fn send_alert_to_channel(&self, channel: &AlertChannel, alert: &CostAlert) -> Result<()> {
        match channel {
            AlertChannel::Console => {
                println!("💰 COST ALERT [{:?}]: {}", alert.severity, alert.message);
                println!("   Cost: ${:.4}", alert.cost_amount);
                if let Some(threshold) = alert.threshold_amount {
                    println!("   Threshold: ${:.4}", threshold);
                }
                println!("   Time: {}", alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("---");
            }
            AlertChannel::Log => {
                tracing::warn!(
                    alert_id = %alert.id,
                    alert_type = ?alert.alert_type,
                    severity = ?alert.severity,
                    cost = %alert.cost_amount,
                    threshold = ?alert.threshold_amount,
                    "Cost alert triggered: {}",
                    alert.message
                );
            }
            AlertChannel::Email { recipients: _ } => {
                // Email implementation would go here
                tracing::info!("Email cost alert sent: {}", alert.message);
            }
            AlertChannel::Webhook { url: _, headers: _ } => {
                // Webhook implementation would go here
                tracing::info!("Webhook cost alert sent: {}", alert.message);
            }
            AlertChannel::Slack { webhook_url: _, channel: _ } => {
                // Slack implementation would go here
                tracing::info!("Slack cost alert sent: {}", alert.message);
            }
        }
        
        Ok(())
    }
    
    /// Check cost thresholds
    async fn check_cost_thresholds(&self, cost: f64, stage: Stage, model: &str) -> Result<()> {
        let thresholds = self.thresholds.read().await;
        
        for threshold in thresholds.iter() {
            if !threshold.enabled {
                continue;
            }
            
            let threshold_exceeded = match threshold.threshold_type {
                ThresholdType::Absolute => cost > threshold.amount,
                ThresholdType::BudgetPercentage => {
                    // Would need to check against relevant budget
                    false // Simplified
                }
                ThresholdType::RateIncrease => {
                    // Would need to check rate of increase
                    false // Simplified
                }
                ThresholdType::PerUnit => {
                    // Would need token count for per-token calculations
                    false // Simplified
                }
            };
            
            if threshold_exceeded {
                let alert = CostAlert {
                    id: Uuid::new_v4().to_string(),
                    alert_type: CostAlertType::CostThreshold,
                    severity: AlertSeverity::Warning,
                    message: format!(
                        "Cost threshold '{}' exceeded: ${:.4} > ${:.4} (Model: {}, Stage: {:?})",
                        threshold.name, cost, threshold.amount, model, stage
                    ),
                    cost_amount: cost,
                    threshold_amount: Some(threshold.amount),
                    budget_id: None,
                    scope: format!("{:?}", threshold.scope),
                    timestamp: chrono::Utc::now(),
                    acknowledged: false,
                    acknowledged_by: None,
                    acknowledged_at: None,
                    escalation_level: 0,
                    metadata: HashMap::new(),
                };
                
                self.send_cost_alert(&alert).await?;
                
                // Store alert
                {
                    let mut alerts = self.alerts.write().await;
                    alerts.push(alert);
                }
            }
        }
        
        Ok(())
    }
    
    /// Update efficiency metrics
    async fn update_efficiency_metrics(
        &self,
        metrics: &mut EfficiencyMetrics,
        cost_trends: &[CostDataPoint],
    ) -> Result<()> {
        if cost_trends.is_empty() {
            return Ok(());
        }
        
        // Calculate average cost per token
        let total_cost: f64 = cost_trends.iter().map(|p| p.cost).sum();
        let total_tokens: u32 = cost_trends.iter().map(|p| p.tokens_used).sum();
        metrics.average_cost_per_token = if total_tokens > 0 {
            total_cost / total_tokens as f64
        } else {
            0.0
        };
        
        // Calculate average cost per operation
        metrics.average_cost_per_operation = total_cost / cost_trends.len() as f64;
        
        // Calculate average cost per quality point
        let total_quality: f64 = cost_trends.iter().map(|p| p.quality_score).sum();
        metrics.average_cost_per_quality_point = if total_quality > 0.0 {
            total_cost / total_quality
        } else {
            0.0
        };
        
        // Update model efficiency rankings
        let mut model_stats: HashMap<String, ModelEfficiencyStats> = HashMap::new();
        
        for point in cost_trends {
            let stats = model_stats.entry(point.model.clone()).or_default();
            stats.total_cost += point.cost;
            stats.total_quality += point.quality_score;
            stats.total_tokens += point.tokens_used;
            stats.usage_count += 1;
        }
        
        metrics.model_efficiency_rankings = model_stats
            .into_iter()
            .map(|(model_name, stats)| {
                let avg_cost_per_token = if stats.total_tokens > 0 {
                    stats.total_cost / stats.total_tokens as f64
                } else {
                    0.0
                };
                let avg_quality = stats.total_quality / stats.usage_count as f64;
                
                ModelEfficiency {
                    model_name,
                    cost_efficiency_score: (1.0 / (avg_cost_per_token + 0.00001)).min(1.0),
                    quality_efficiency_score: avg_quality,
                    overall_efficiency_score: (avg_quality / (avg_cost_per_token + 0.00001)).min(1.0),
                    usage_count: stats.usage_count,
                    total_cost: stats.total_cost,
                    average_quality: avg_quality,
                }
            })
            .collect();
        
        // Sort by overall efficiency
        metrics.model_efficiency_rankings.sort_by(|a, b| {
            b.overall_efficiency_score.partial_cmp(&a.overall_efficiency_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(())
    }
}

#[derive(Debug, Default)]
struct ModelEfficiencyStats {
    total_cost: f64,
    total_quality: f64,
    total_tokens: u32,
    usage_count: u32,
}

/// Context for cost estimation
#[derive(Debug, Clone)]
pub struct CostEstimationContext {
    pub user_id: Option<String>,
    pub project_id: Option<String>,
    pub conversation_id: String,
    pub question_complexity: f64, // 0.0 - 1.0
    pub expected_quality: f64, // 0.0 - 1.0
    pub priority: String,
}

/// Cost estimate result
#[derive(Debug, Clone)]
pub struct CostEstimate {
    pub estimated_cost: f64,
    pub confidence: f64, // 0.0 - 1.0
    pub factors: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

/// Cost approval requirement
#[derive(Debug, Clone)]
pub struct CostApprovalRequirement {
    pub approval_type: CostApprovalType,
    pub estimated_cost: f64,
    pub threshold: f64,
    pub reason: String,
    pub required_approvers: Vec<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub enum CostApprovalType {
    Standard,
    Emergency,
}

/// Budget update result
#[derive(Debug, Clone)]
pub struct BudgetUpdateResult {
    pub budget_id: String,
    pub previous_usage: f64,
    pub new_usage: f64,
    pub usage_percentage: f64,
    pub status: BudgetStatus,
    pub alerts_triggered: Vec<CostAlert>,
}

/// Cost summary
#[derive(Debug, Clone)]
pub struct CostSummary {
    pub period_cost: f64,
    pub total_budget: f64,
    pub total_usage: f64,
    pub budget_utilization: f64,
    pub costs_by_model: HashMap<String, f64>,
    pub costs_by_stage: HashMap<String, f64>,
    pub efficiency_metrics: EfficiencyMetrics,
    pub active_budgets: usize,
    pub exceeded_budgets: usize,
}

/// Cost optimization recommendation
#[derive(Debug, Clone)]
pub struct CostOptimizationRecommendation {
    pub id: String,
    pub recommendation_type: OptimizationType,
    pub title: String,
    pub description: String,
    pub potential_savings: f64, // Percentage
    pub implementation_effort: OptimizationEffort,
    pub priority: RecommendationPriority,
    pub estimated_impact: EstimatedImpact,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct EstimatedImpact {
    pub cost_reduction: f64, // Dollar amount
    pub quality_impact: f64, // -1.0 to 1.0 (negative = reduction)
    pub implementation_time_days: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cost_controller_creation() {
        let config = CostControlConfig::default();
        let controller = CostController::new(config).await.unwrap();
        
        let summary = controller.get_cost_summary(None).await.unwrap();
        assert_eq!(summary.active_budgets, 0);
        assert_eq!(summary.exceeded_budgets, 0);
    }
    
    #[tokio::test]
    async fn test_budget_creation_and_usage() {
        let config = CostControlConfig::default();
        let controller = CostController::new(config).await.unwrap();
        
        let budget = Budget {
            id: String::new(), // Will be set by create_budget
            name: "Test Budget".to_string(),
            description: "Test budget for unit tests".to_string(),
            amount: 100.0,
            period: BudgetPeriod::Monthly,
            scope: BudgetScope::Global,
            start_date: chrono::Utc::now(),
            end_date: chrono::Utc::now() + chrono::Duration::days(30),
            current_usage: 0.0,
            alerts_sent: 0,
            status: BudgetStatus::Active,
            created_by: "test_user".to_string(),
            tags: vec!["test".to_string()],
        };
        
        let budget_id = controller.create_budget(budget).await.unwrap();
        assert!(!budget_id.is_empty());
        
        // Test budget usage update
        let result = controller.update_budget_usage(&budget_id, 25.0).await.unwrap();
        assert_eq!(result.new_usage, 25.0);
        assert_eq!(result.usage_percentage, 0.25);
        assert_eq!(result.status, BudgetStatus::Active);
    }
    
    #[tokio::test]
    async fn test_cost_estimation() {
        let config = CostControlConfig::default();
        let controller = CostController::new(config).await.unwrap();
        
        let context = CostEstimationContext {
            user_id: Some("test_user".to_string()),
            project_id: Some("test_project".to_string()),
            conversation_id: "test_conversation".to_string(),
            question_complexity: 0.7,
            expected_quality: 0.8,
            priority: "normal".to_string(),
        };
        
        let estimate = controller.estimate_operation_cost(
            Stage::Generator,
            "gpt-4",
            1000,
            &context,
        ).await.unwrap();
        
        assert!(estimate.estimated_cost > 0.0);
        assert!(estimate.confidence >= 0.0 && estimate.confidence <= 1.0);
        assert!(!estimate.factors.is_empty());
    }
}