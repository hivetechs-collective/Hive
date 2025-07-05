//! Consensus Integration - Deep integration of hooks with consensus pipeline
//! 
//! Provides fine-grained control over the 4-stage consensus process with enterprise features:
//! - Hook points at each consensus stage
//! - Cost threshold management with approval workflows  
//! - Quality validation workflows with automated gates
//! - Performance monitoring with real-time analytics
//! - Enterprise compliance and audit logging

use super::{
    HooksSystem, HookEvent, EventType, EventSource,
    HookAuditLogger, AuditEvent, AuditEventType,
    approval_workflow::{ApprovalWorkflow, ApprovalRequest, ApprovalStatus, ApprovalPriority},
    registry::HookId,
};
use crate::consensus::types::{ConsensusConfig, Stage, StageResult, StageAnalytics};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Consensus hook integration manager
pub struct ConsensusIntegration {
    hooks_system: Arc<HooksSystem>,
    approval_workflow: Arc<ApprovalWorkflow>,
    audit_logger: Arc<HookAuditLogger>,
    cost_controller: Arc<RwLock<CostController>>,
    quality_gate_manager: Arc<RwLock<QualityGateManager>>,
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
    config: ConsensusIntegrationConfig,
}

/// Configuration for consensus integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusIntegrationConfig {
    /// Whether to enable hook integration
    pub enabled: bool,
    
    /// Maximum cost per stage before requiring approval
    pub stage_cost_threshold: f64,
    
    /// Total cost threshold for entire consensus
    pub total_cost_threshold: f64,
    
    /// Quality gates configuration
    pub quality_gates: QualityGatesConfig,
    
    /// Performance monitoring configuration
    pub performance_monitoring: PerformanceMonitoringConfig,
    
    /// Hook execution timeout in seconds
    pub hook_timeout_seconds: u64,
    
    /// Whether to continue on hook failures
    pub continue_on_hook_failure: bool,
    
    /// Enterprise features configuration
    pub enterprise: EnterpriseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGatesConfig {
    /// Minimum quality score threshold (0.0 - 1.0)
    pub min_quality_score: f64,
    
    /// Maximum error rate threshold (0.0 - 1.0)
    pub max_error_rate: f64,
    
    /// Maximum retry count per stage
    pub max_retry_count: u32,
    
    /// Whether to block low-quality responses
    pub block_low_quality: bool,
    
    /// Quality gate checks to perform
    pub checks: Vec<QualityCheckType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoringConfig {
    /// Maximum duration per stage in seconds
    pub max_stage_duration: f64,
    
    /// Maximum memory usage per stage in MB
    pub max_memory_usage: Option<u64>,
    
    /// Whether to enable detailed performance tracking
    pub detailed_tracking: bool,
    
    /// Performance alert thresholds
    pub alert_thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Whether approval workflows are required
    pub require_approvals: bool,
    
    /// Approval timeout in seconds
    pub approval_timeout_seconds: u64,
    
    /// Whether to enable comprehensive audit logging
    pub comprehensive_audit: bool,
    
    /// RBAC integration settings
    pub rbac_integration: bool,
    
    /// Compliance settings
    pub compliance: ComplianceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Whether to log all consensus operations
    pub log_all_operations: bool,
    
    /// Data retention period in days
    pub data_retention_days: u32,
    
    /// Required approval levels for different operations
    pub approval_levels: HashMap<String, Vec<String>>,
    
    /// Compliance tags to apply
    pub compliance_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityCheckType {
    /// Check response coherence and consistency
    Coherence,
    
    /// Check for potential factual errors
    FactualAccuracy,
    
    /// Check response completeness
    Completeness,
    
    /// Check for inappropriate content
    ContentSafety,
    
    /// Check response relevance to query
    Relevance,
    
    /// Custom quality check
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Slow stage duration threshold in seconds
    pub slow_stage_threshold: f64,
    
    /// High memory usage threshold in MB
    pub high_memory_threshold: u64,
    
    /// High error rate threshold (0.0 - 1.0)
    pub high_error_rate_threshold: f64,
    
    /// Alert cooldown period in seconds
    pub alert_cooldown_seconds: u64,
}

/// Cost control manager
pub struct CostController {
    stage_costs: HashMap<String, f64>,
    total_cost: f64,
    thresholds: CostThresholds,
    approval_pending: HashMap<String, ApprovalRequest>,
    last_alert_time: HashMap<String, Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostThresholds {
    pub stage_warning: f64,
    pub stage_critical: f64,
    pub total_warning: f64,
    pub total_critical: f64,
    pub budget_limit: f64,
}

/// Quality gate manager
pub struct QualityGateManager {
    quality_scores: HashMap<String, f64>,
    error_counts: HashMap<String, u32>,
    retry_counts: HashMap<String, u32>,
    gate_config: QualityGatesConfig,
    failed_gates: Vec<QualityGateFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateFailure {
    pub stage: String,
    pub check_type: QualityCheckType,
    pub actual_value: f64,
    pub threshold: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action_taken: QualityGateAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityGateAction {
    /// Response was blocked
    Blocked,
    
    /// Response was allowed with warning
    AllowedWithWarning,
    
    /// Stage was retried
    Retried,
    
    /// Escalated for manual review
    Escalated,
}

/// Performance monitoring manager
pub struct PerformanceMonitor {
    stage_durations: HashMap<String, Vec<f64>>,
    memory_usage: HashMap<String, Vec<u64>>,
    error_rates: HashMap<String, f64>,
    alerts: Vec<PerformanceAlert>,
    last_alert_times: HashMap<String, Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: PerformanceAlertType,
    pub stage: String,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlertType {
    SlowStage,
    HighMemoryUsage,
    HighErrorRate,
    ResourceExhaustion,
}

impl Default for ConsensusIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stage_cost_threshold: 0.50, // $0.50 per stage
            total_cost_threshold: 2.00, // $2.00 total
            quality_gates: QualityGatesConfig::default(),
            performance_monitoring: PerformanceMonitoringConfig::default(),
            hook_timeout_seconds: 30,
            continue_on_hook_failure: true,
            enterprise: EnterpriseConfig::default(),
        }
    }
}

impl Default for QualityGatesConfig {
    fn default() -> Self {
        Self {
            min_quality_score: 0.7,
            max_error_rate: 0.1,
            max_retry_count: 3,
            block_low_quality: false,
            checks: vec![
                QualityCheckType::Coherence,
                QualityCheckType::Completeness,
                QualityCheckType::Relevance,
            ],
        }
    }
}

impl Default for PerformanceMonitoringConfig {
    fn default() -> Self {
        Self {
            max_stage_duration: 30.0, // 30 seconds
            max_memory_usage: Some(512), // 512 MB
            detailed_tracking: true,
            alert_thresholds: PerformanceThresholds::default(),
        }
    }
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            require_approvals: false,
            approval_timeout_seconds: 300, // 5 minutes  
            comprehensive_audit: true,
            rbac_integration: true,
            compliance: ComplianceConfig::default(),
        }
    }
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            log_all_operations: true,
            data_retention_days: 90,
            approval_levels: HashMap::new(),
            compliance_tags: vec!["consensus".to_string(), "ai-generated".to_string()],
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            slow_stage_threshold: 10.0, // 10 seconds
            high_memory_threshold: 256, // 256 MB
            high_error_rate_threshold: 0.05, // 5%
            alert_cooldown_seconds: 300, // 5 minutes
        }
    }
}

impl ConsensusIntegration {
    /// Create a new consensus integration manager
    pub async fn new(
        hooks_system: Arc<HooksSystem>,
        config: ConsensusIntegrationConfig,
    ) -> Result<Self> {
        let approval_workflow = Arc::new(ApprovalWorkflow::new());
        let audit_logger = Arc::new(
            HookAuditLogger::new(std::path::PathBuf::from("consensus_hooks_audit.log")).await?
        );
        
        let cost_thresholds = CostThresholds {
            stage_warning: config.stage_cost_threshold * 0.8,
            stage_critical: config.stage_cost_threshold,
            total_warning: config.total_cost_threshold * 0.8,
            total_critical: config.total_cost_threshold,
            budget_limit: config.total_cost_threshold * 1.5,
        };
        
        let cost_controller = Arc::new(RwLock::new(CostController {
            stage_costs: HashMap::new(),
            total_cost: 0.0,
            thresholds: cost_thresholds,
            approval_pending: HashMap::new(),
            last_alert_time: HashMap::new(),
        }));
        
        let quality_gate_manager = Arc::new(RwLock::new(QualityGateManager {
            quality_scores: HashMap::new(),
            error_counts: HashMap::new(),
            retry_counts: HashMap::new(),
            gate_config: config.quality_gates.clone(),
            failed_gates: Vec::new(),
        }));
        
        let performance_monitor = Arc::new(RwLock::new(PerformanceMonitor {
            stage_durations: HashMap::new(),
            memory_usage: HashMap::new(),
            error_rates: HashMap::new(),
            alerts: Vec::new(),
            last_alert_times: HashMap::new(),
        }));
        
        Ok(Self {
            hooks_system,
            approval_workflow,
            audit_logger,
            cost_controller,
            quality_gate_manager,
            performance_monitor,
            config,
        })
    }
    
    /// Execute pre-stage hooks with cost and quality checks
    pub async fn execute_pre_stage_hooks(
        &self,
        stage: Stage,
        conversation_id: &str,
        question: &str,
        model: &str,
        estimated_cost: f64,
    ) -> Result<StageHookResult> {
        let stage_name = stage.as_str();
        let start_time = Instant::now();
        
        // Check cost thresholds first
        if let Some(approval_required) = self.check_cost_thresholds(stage_name, estimated_cost).await? {
            return Ok(StageHookResult {
                proceed: false,
                approval_required: Some(approval_required),
                warnings: vec![format!("Cost threshold exceeded for {} stage", stage_name)],
                modifications: HashMap::new(),
            });
        }
        
        // Create pre-stage event
        let event_type = match stage {
            Stage::Generator => EventType::BeforeGeneratorStage,
            Stage::Refiner => EventType::BeforeRefinerStage,
            Stage::Validator => EventType::BeforeValidatorStage,
            Stage::Curator => EventType::BeforeCuratorStage,
        };
        
        let event = HookEvent::new(
            event_type,
            EventSource::Consensus { stage: stage_name.to_string() }
        )
        .with_context("conversation_id", conversation_id)
        .with_context("question", question)
        .with_context("model", model)
        .with_context("estimated_cost", estimated_cost)
        .with_context("timestamp", chrono::Utc::now().to_rfc3339());
        
        // Dispatch event to hooks system
        if let Err(e) = self.hooks_system.dispatch_event(event).await {
            tracing::warn!("Failed to dispatch pre-{} hooks: {}", stage_name, e);
            if !self.config.continue_on_hook_failure {
                return Err(e.context(format!("Pre-{} hook execution failed", stage_name)));
            }
        }
        
        // Log the pre-stage execution
        let event = AuditEvent::new(AuditEventType::HookRegistered {
            hook_id: HookId(format!("pre_{}_stage", stage_name)),
            hook_name: format!("Pre-{} stage hooks", stage_name),
        })
        .with_context("details", format!("Pre-{} stage hooks executed", stage_name))
        .with_context("result", "success")
        .with_context("duration", start_time.elapsed().as_secs_f64());
        
        self.audit_logger.log_event(event).await?;
        
        Ok(StageHookResult {
            proceed: true,
            approval_required: None,
            warnings: Vec::new(),
            modifications: HashMap::new(),
        })
    }
    
    /// Execute post-stage hooks with quality validation
    pub async fn execute_post_stage_hooks(
        &self,
        stage: Stage,
        stage_result: &StageResult,
    ) -> Result<StageHookResult> {
        let stage_name = stage.as_str();
        let start_time = Instant::now();
        
        // Perform quality gate checks
        let quality_result = self.check_quality_gates(stage, stage_result).await?;
        if !quality_result.passed {
            return Ok(StageHookResult {
                proceed: quality_result.can_continue,
                approval_required: quality_result.approval_required,
                warnings: quality_result.warnings,
                modifications: HashMap::new(),
            });
        }
        
        // Update performance monitoring
        self.update_performance_metrics(stage, stage_result).await?;
        
        // Create post-stage event
        let event_type = match stage {
            Stage::Generator => EventType::AfterGeneratorStage,
            Stage::Refiner => EventType::AfterRefinerStage,
            Stage::Validator => EventType::AfterValidatorStage,
            Stage::Curator => EventType::AfterCuratorStage,
        };
        
        let event = HookEvent::new(
            event_type,
            EventSource::Consensus { stage: stage_name.to_string() }
        )
        .with_context("conversation_id", &stage_result.conversation_id)
        .with_context("stage_id", &stage_result.stage_id)
        .with_context("answer", &stage_result.answer)
        .with_context("model", &stage_result.model)
        .with_context("duration", stage_result.analytics.as_ref().map(|a| a.duration).unwrap_or(0.0))
        .with_context("cost", stage_result.analytics.as_ref().map(|a| a.cost).unwrap_or(0.0))
        .with_context("quality_score", stage_result.analytics.as_ref().map(|a| a.quality_score).unwrap_or(0.0));
        
        // Dispatch event to hooks system
        if let Err(e) = self.hooks_system.dispatch_event(event).await {
            tracing::warn!("Failed to dispatch post-{} hooks: {}", stage_name, e);
            if !self.config.continue_on_hook_failure {
                return Err(e.context(format!("Post-{} hook execution failed", stage_name)));
            }
        }
        
        // Log the post-stage execution
        let event = AuditEvent::new(AuditEventType::HookRegistered {
            hook_id: HookId(format!("post_{}_stage", stage_name)),
            hook_name: format!("Post-{} stage hooks", stage_name),
        })
        .with_context("details", format!("Post-{} stage hooks executed", stage_name))
        .with_context("result", "success")
        .with_context("duration", start_time.elapsed().as_secs_f64());
        
        self.audit_logger.log_event(event).await?;
        
        Ok(StageHookResult {
            proceed: true,
            approval_required: None,
            warnings: Vec::new(),
            modifications: HashMap::new(),
        })
    }
    
    /// Check cost thresholds and determine if approval is required
    async fn check_cost_thresholds(
        &self,
        stage: &str,
        estimated_cost: f64,
    ) -> Result<Option<ApprovalRequest>> {
        let mut controller = self.cost_controller.write().await;
        
        // Update stage cost
        controller.stage_costs.insert(stage.to_string(), estimated_cost);
        controller.total_cost += estimated_cost;
        
        // Check stage cost threshold
        if estimated_cost > controller.thresholds.stage_critical {
            if self.config.enterprise.require_approvals {
                let approval_request = ApprovalRequest {
                    id: Uuid::new_v4().to_string(),
                    hook_id: HookId::from_name("consensus.cost_threshold"),
                    request_type: "cost_threshold_exceeded".to_string(),
                    description: format!(
                        "Stage {} cost ${:.4} exceeds threshold ${:.4}",
                        stage, estimated_cost, controller.thresholds.stage_critical
                    ),
                    requested_by: "system".to_string(),
                    created_at: chrono::Utc::now(),
                    expires_at: Some(chrono::Utc::now() + chrono::Duration::seconds(self.config.enterprise.approval_timeout_seconds as i64)),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("stage".to_string(), serde_json::Value::String(stage.to_string()));
                        meta.insert("cost".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(estimated_cost).unwrap()));
                        meta.insert("threshold".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(controller.thresholds.stage_critical).unwrap()));
                        meta
                    },
                    priority: ApprovalPriority::High,
                    required_approvers: vec!["admin".to_string()],
                    received_approvals: Vec::new(),
                    current_escalation_level: 0,
                    notification_count: 0,
                    last_notification_at: None,
                };
                
                controller.approval_pending.insert(stage.to_string(), approval_request.clone());
                return Ok(Some(approval_request));
            }
        }
        
        // Check total cost threshold
        if controller.total_cost > controller.thresholds.total_critical {
            if self.config.enterprise.require_approvals {
                let approval_request = ApprovalRequest {
                    id: Uuid::new_v4().to_string(),
                    hook_id: HookId::from_name("consensus.total_cost"),
                    request_type: "total_cost_threshold_exceeded".to_string(),
                    description: format!(
                        "Total consensus cost ${:.4} exceeds threshold ${:.4}",
                        controller.total_cost, controller.thresholds.total_critical
                    ),
                    requested_by: "system".to_string(),
                    created_at: chrono::Utc::now(),
                    expires_at: Some(chrono::Utc::now() + chrono::Duration::seconds(self.config.enterprise.approval_timeout_seconds as i64)),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("total_cost".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(controller.total_cost).unwrap()));
                        meta.insert("threshold".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(controller.thresholds.total_critical).unwrap()));
                        meta
                    },
                    priority: ApprovalPriority::High,
                    required_approvers: vec!["admin".to_string()],
                    received_approvals: Vec::new(),
                    current_escalation_level: 0,
                    notification_count: 0,
                    last_notification_at: None,
                };
                
                controller.approval_pending.insert("total_cost".to_string(), approval_request.clone());
                return Ok(Some(approval_request));
            }
        }
        
        Ok(None)
    }
    
    /// Check quality gates for a stage result
    async fn check_quality_gates(
        &self,
        stage: Stage,
        stage_result: &StageResult,
    ) -> Result<QualityGateResult> {
        let mut manager = self.quality_gate_manager.write().await;
        let stage_name = stage.as_str();
        
        let mut warnings = Vec::new();
        let mut failed_checks = Vec::new();
        
        // Extract quality metrics from stage result
        let quality_score = stage_result.analytics
            .as_ref()
            .map(|a| a.quality_score)
            .unwrap_or(0.8); // Default reasonable score
        
        let error_count = stage_result.analytics
            .as_ref()
            .map(|a| a.error_count)
            .unwrap_or(0);
        
        // Update tracking
        manager.quality_scores.insert(stage_name.to_string(), quality_score);
        manager.error_counts.insert(stage_name.to_string(), error_count);
        
        // Check minimum quality score
        if quality_score < manager.gate_config.min_quality_score {
            let failure = QualityGateFailure {
                stage: stage_name.to_string(),
                check_type: QualityCheckType::Coherence,
                actual_value: quality_score,
                threshold: manager.gate_config.min_quality_score,
                timestamp: chrono::Utc::now(),
                action_taken: if manager.gate_config.block_low_quality {
                    QualityGateAction::Blocked
                } else {
                    QualityGateAction::AllowedWithWarning
                },
            };
            
            failed_checks.push(failure.clone());
            manager.failed_gates.push(failure);
            
            if manager.gate_config.block_low_quality {
                return Ok(QualityGateResult {
                    passed: false,
                    can_continue: false,
                    warnings: vec![format!("Quality score {} below threshold {}", quality_score, manager.gate_config.min_quality_score)],
                    failed_checks,
                    approval_required: if self.config.enterprise.require_approvals {
                        Some(self.create_quality_approval_request(stage_name, "low_quality_score", quality_score).await?)
                    } else {
                        None
                    },
                });
            } else {
                warnings.push(format!("Quality score {} below recommended threshold {}", quality_score, manager.gate_config.min_quality_score));
            }
        }
        
        // Check error rate
        let current_error_rate = error_count as f64 / 100.0; // Simple calculation
        if current_error_rate > manager.gate_config.max_error_rate {
            let failure = QualityGateFailure {
                stage: stage_name.to_string(),
                check_type: QualityCheckType::FactualAccuracy,
                actual_value: current_error_rate,
                threshold: manager.gate_config.max_error_rate,
                timestamp: chrono::Utc::now(),
                action_taken: QualityGateAction::AllowedWithWarning,
            };
            
            failed_checks.push(failure.clone());
            manager.failed_gates.push(failure);
            warnings.push(format!("Error rate {} exceeds threshold {}", current_error_rate, manager.gate_config.max_error_rate));
        }
        
        // Perform additional quality checks based on configuration
        for check_type in &manager.gate_config.checks {
            match check_type {
                QualityCheckType::Completeness => {
                    // Check if response is complete (simple heuristic)
                    if stage_result.answer.len() < 50 {
                        warnings.push("Response may be incomplete (too short)".to_string());
                    }
                }
                QualityCheckType::Relevance => {
                    // Check relevance (simplified check)
                    if !stage_result.answer.to_lowercase().contains(&extract_key_terms(&stage_result.question)) {
                        warnings.push("Response may not be relevant to the question".to_string());
                    }
                }
                QualityCheckType::ContentSafety => {
                    // Basic content safety check
                    if contains_inappropriate_content(&stage_result.answer) {
                        let failure = QualityGateFailure {
                            stage: stage_name.to_string(),
                            check_type: QualityCheckType::ContentSafety,
                            actual_value: 1.0,
                            threshold: 0.0,
                            timestamp: chrono::Utc::now(),
                            action_taken: QualityGateAction::Blocked,
                        };
                        
                        failed_checks.push(failure.clone());
                        manager.failed_gates.push(failure);
                        
                        return Ok(QualityGateResult {
                            passed: false,
                            can_continue: false,
                            warnings: vec!["Response contains inappropriate content".to_string()],
                            failed_checks,
                            approval_required: None,
                        });
                    }
                }
                _ => {
                    // Other checks can be implemented as needed
                }
            }
        }
        
        Ok(QualityGateResult {
            passed: failed_checks.is_empty() || !manager.gate_config.block_low_quality,
            can_continue: true,
            warnings,
            failed_checks,
            approval_required: None,
        })
    }
    
    /// Update performance metrics for a stage
    async fn update_performance_metrics(
        &self,
        stage: Stage,
        stage_result: &StageResult,
    ) -> Result<()> {
        let mut monitor = self.performance_monitor.write().await;
        let stage_name = stage.as_str();
        
        if let Some(analytics) = &stage_result.analytics {
            // Track duration
            monitor.stage_durations
                .entry(stage_name.to_string())
                .or_default()
                .push(analytics.duration);
            
            // Track memory usage if available
            if let Some(memory) = analytics.memory_usage {
                monitor.memory_usage
                    .entry(stage_name.to_string())
                    .or_default()
                    .push(memory);
            }
            
            // Calculate error rate
            let error_rate = analytics.error_count as f64 / 100.0;
            monitor.error_rates.insert(stage_name.to_string(), error_rate);
            
            // Check for performance alerts
            self.check_performance_alerts(&mut monitor, stage_name, analytics).await?;
        }
        
        Ok(())
    }
    
    /// Check for performance alerts and create them if needed
    async fn check_performance_alerts(
        &self,
        monitor: &mut PerformanceMonitor,
        stage_name: &str,
        analytics: &StageAnalytics,
    ) -> Result<()> {
        let now = Instant::now();
        let cooldown = std::time::Duration::from_secs(self.config.performance_monitoring.alert_thresholds.alert_cooldown_seconds);
        
        // Check if we're in cooldown period
        if let Some(last_alert) = monitor.last_alert_times.get(stage_name) {
            if now.duration_since(*last_alert) < cooldown {
                return Ok(());
            }
        }
        
        let thresholds = &self.config.performance_monitoring.alert_thresholds;
        
        // Check slow stage threshold
        if analytics.duration > thresholds.slow_stage_threshold {
            let alert = PerformanceAlert {
                alert_type: PerformanceAlertType::SlowStage,
                stage: stage_name.to_string(),
                value: analytics.duration,
                threshold: thresholds.slow_stage_threshold,
                timestamp: chrono::Utc::now(),
                resolved: false,
            };
            
            monitor.alerts.push(alert);
            monitor.last_alert_times.insert(stage_name.to_string(), now);
            
            tracing::warn!(
                "Performance alert: Stage {} took {:.2}s (threshold: {:.2}s)",
                stage_name, analytics.duration, thresholds.slow_stage_threshold
            );
        }
        
        // Check high memory usage
        if let Some(memory_usage) = analytics.memory_usage {
            if memory_usage > thresholds.high_memory_threshold {
                let alert = PerformanceAlert {
                    alert_type: PerformanceAlertType::HighMemoryUsage,
                    stage: stage_name.to_string(),
                    value: memory_usage as f64,
                    threshold: thresholds.high_memory_threshold as f64,
                    timestamp: chrono::Utc::now(),
                    resolved: false,
                };
                
                monitor.alerts.push(alert);
                monitor.last_alert_times.insert(stage_name.to_string(), now);
                
                tracing::warn!(
                    "Performance alert: Stage {} used {}MB memory (threshold: {}MB)",
                    stage_name, memory_usage, thresholds.high_memory_threshold
                );
            }
        }
        
        // Check error rate
        let error_rate = analytics.error_count as f64 / 100.0;
        if error_rate > thresholds.high_error_rate_threshold {
            let alert = PerformanceAlert {
                alert_type: PerformanceAlertType::HighErrorRate,
                stage: stage_name.to_string(),
                value: error_rate,
                threshold: thresholds.high_error_rate_threshold,
                timestamp: chrono::Utc::now(),
                resolved: false,
            };
            
            monitor.alerts.push(alert);
            monitor.last_alert_times.insert(stage_name.to_string(), now);
            
            tracing::warn!(
                "Performance alert: Stage {} has {:.2}% error rate (threshold: {:.2}%)",
                stage_name, error_rate * 100.0, thresholds.high_error_rate_threshold * 100.0
            );
        }
        
        Ok(())
    }
    
    /// Create an approval request for quality gate failure
    async fn create_quality_approval_request(
        &self,
        stage: &str,
        reason: &str,
        value: f64,
    ) -> Result<ApprovalRequest> {
        Ok(ApprovalRequest {
            id: Uuid::new_v4().to_string(),
            hook_id: HookId::from_name("consensus.quality_gate"),
            request_type: "quality_gate_failure".to_string(),
            description: format!(
                "Quality gate failure in {} stage: {} (value: {:.3})",
                stage, reason, value
            ),
            requested_by: "system".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::seconds(self.config.enterprise.approval_timeout_seconds as i64)),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("stage".to_string(), serde_json::Value::String(stage.to_string()));
                meta.insert("reason".to_string(), serde_json::Value::String(reason.to_string()));
                meta.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()));
                meta
            },
            priority: ApprovalPriority::Medium,
            required_approvers: vec!["admin".to_string()],
            received_approvals: Vec::new(),
            current_escalation_level: 0,
            notification_count: 0,
            last_notification_at: None,
        })
    }
    
    /// Get current cost summary
    pub async fn get_cost_summary(&self) -> Result<CostSummary> {
        let controller = self.cost_controller.read().await;
        Ok(CostSummary {
            stage_costs: controller.stage_costs.clone(),
            total_cost: controller.total_cost,
            thresholds: controller.thresholds.clone(),
            pending_approvals: controller.approval_pending.len(),
        })
    }
    
    /// Get quality gate status
    pub async fn get_quality_gate_status(&self) -> Result<QualityGateStatus> {
        let manager = self.quality_gate_manager.read().await;
        Ok(QualityGateStatus {
            quality_scores: manager.quality_scores.clone(),
            error_counts: manager.error_counts.clone(),
            retry_counts: manager.retry_counts.clone(),
            failed_gates: manager.failed_gates.clone(),
            config: manager.gate_config.clone(),
        })
    }
    
    /// Get performance monitoring status
    pub async fn get_performance_status(&self) -> Result<PerformanceStatus> {
        let monitor = self.performance_monitor.read().await;
        Ok(PerformanceStatus {
            stage_durations: monitor.stage_durations.clone(),
            memory_usage: monitor.memory_usage.clone(),
            error_rates: monitor.error_rates.clone(),
            active_alerts: monitor.alerts.iter().filter(|a| !a.resolved).cloned().collect(),
            total_alerts: monitor.alerts.len(),
        })
    }
    
    /// Reset all monitoring data
    pub async fn reset_monitoring_data(&self) -> Result<()> {
        {
            let mut controller = self.cost_controller.write().await;
            controller.stage_costs.clear();
            controller.total_cost = 0.0;
            controller.approval_pending.clear();
            controller.last_alert_time.clear();
        }
        
        {
            let mut manager = self.quality_gate_manager.write().await;
            manager.quality_scores.clear();
            manager.error_counts.clear();
            manager.retry_counts.clear();
            manager.failed_gates.clear();
        }
        
        {
            let mut monitor = self.performance_monitor.write().await;
            monitor.stage_durations.clear();
            monitor.memory_usage.clear();
            monitor.error_rates.clear();
            monitor.alerts.clear();
            monitor.last_alert_times.clear();
        }
        
        Ok(())
    }
}

/// Result of stage hook execution
#[derive(Debug, Clone)]
pub struct StageHookResult {
    pub proceed: bool,
    pub approval_required: Option<ApprovalRequest>,
    pub warnings: Vec<String>,
    pub modifications: HashMap<String, serde_json::Value>,
}

/// Result of quality gate check
#[derive(Debug, Clone)]
pub struct QualityGateResult {
    pub passed: bool,
    pub can_continue: bool,
    pub warnings: Vec<String>,
    pub failed_checks: Vec<QualityGateFailure>,
    pub approval_required: Option<ApprovalRequest>,
}

/// Cost summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub stage_costs: HashMap<String, f64>,
    pub total_cost: f64,
    pub thresholds: CostThresholds,
    pub pending_approvals: usize,
}

/// Quality gate status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateStatus {
    pub quality_scores: HashMap<String, f64>,
    pub error_counts: HashMap<String, u32>,
    pub retry_counts: HashMap<String, u32>,
    pub failed_gates: Vec<QualityGateFailure>,
    pub config: QualityGatesConfig,
}

/// Performance monitoring status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatus {
    pub stage_durations: HashMap<String, Vec<f64>>,
    pub memory_usage: HashMap<String, Vec<u64>>,
    pub error_rates: HashMap<String, f64>,
    pub active_alerts: Vec<PerformanceAlert>,
    pub total_alerts: usize,
}

// Helper functions

/// Extract key terms from a question for relevance checking
fn extract_key_terms(question: &str) -> String {
    // Simple implementation - in practice, this would use NLP
    question
        .split_whitespace()
        .filter(|word| word.len() > 3)
        .take(5)
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if content contains inappropriate material (basic implementation)
fn contains_inappropriate_content(content: &str) -> bool {
    let inappropriate_terms = [
        "explicit", "harmful", "inappropriate", // Add more as needed
    ];
    
    let content_lower = content.to_lowercase();
    inappropriate_terms.iter().any(|term| content_lower.contains(term))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_integration_config_default() {
        let config = ConsensusIntegrationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.stage_cost_threshold, 0.50);
        assert_eq!(config.total_cost_threshold, 2.00);
    }
    
    #[test]
    fn test_quality_gates_config_default() {
        let config = QualityGatesConfig::default();
        assert_eq!(config.min_quality_score, 0.7);
        assert_eq!(config.max_error_rate, 0.1);
        assert_eq!(config.max_retry_count, 3);
        assert!(!config.block_low_quality);
    }
    
    #[test]
    fn test_extract_key_terms() {
        let question = "What is the best way to implement authentication in Rust?";
        let terms = extract_key_terms(question);
        assert!(terms.contains("best"));
        assert!(terms.contains("implement"));
        assert!(terms.contains("authentication"));
        assert!(terms.contains("rust"));
    }
    
    #[test]
    fn test_inappropriate_content_detection() {
        assert!(contains_inappropriate_content("This is explicit content"));
        assert!(!contains_inappropriate_content("This is safe content"));
    }
}