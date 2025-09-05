//! Approval Workflow System
//!
//! Provides enterprise-grade approval workflows for consensus pipeline operations.
//! Supports multi-level approvals, timeout handling, and notification systems.

use super::{registry::HookId, AuditEvent, AuditEventType, HookAuditLogger};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Approval workflow manager
pub struct ApprovalWorkflow {
    pending_requests: Arc<RwLock<HashMap<String, ApprovalRequest>>>,
    completed_requests: Arc<RwLock<HashMap<String, CompletedApproval>>>,
    approval_rules: Arc<RwLock<Vec<ApprovalRule>>>,
    notification_handlers: Arc<RwLock<Vec<Box<dyn NotificationHandler + Send + Sync>>>>,
    audit_logger: Option<Arc<HookAuditLogger>>,
    config: ApprovalWorkflowConfig,
}

/// Configuration for approval workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalWorkflowConfig {
    /// Default timeout for approvals in seconds
    pub default_timeout_seconds: u64,

    /// Maximum number of concurrent pending approvals
    pub max_concurrent_approvals: usize,

    /// Whether to require explicit approvals for all operations
    pub require_explicit_approval: bool,

    /// Auto-approval settings
    pub auto_approval: AutoApprovalConfig,

    /// Notification settings
    pub notifications: NotificationConfig,

    /// Escalation settings
    pub escalation: EscalationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApprovalConfig {
    /// Enable auto-approval for certain request types
    pub enabled: bool,

    /// Cost threshold below which auto-approval is allowed
    pub cost_threshold: f64,

    /// Quality score threshold above which auto-approval is allowed
    pub quality_threshold: f64,

    /// List of request types that can be auto-approved
    pub allowed_request_types: Vec<String>,

    /// Maximum number of auto-approvals per hour
    pub rate_limit_per_hour: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable notifications for approval requests
    pub enabled: bool,

    /// Notification channels to use
    pub channels: Vec<NotificationChannel>,

    /// Delay before sending notifications (seconds)
    pub initial_delay_seconds: u64,

    /// Interval for reminder notifications (seconds)
    pub reminder_interval_seconds: u64,

    /// Maximum number of reminders to send
    pub max_reminders: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Enable escalation for overdue approvals
    pub enabled: bool,

    /// Time after which to escalate (seconds)
    pub escalation_timeout_seconds: u64,

    /// Escalation levels and their timeouts
    pub escalation_levels: Vec<EscalationLevel>,

    /// Whether to auto-approve after all escalations
    pub auto_approve_after_escalation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub timeout_seconds: u64,
    pub approvers: Vec<String>,
    pub notification_channels: Vec<NotificationChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email {
        recipients: Vec<String>,
    },
    Slack {
        webhook_url: String,
        channel: String,
    },
    Teams {
        webhook_url: String,
    },
    Discord {
        webhook_url: String,
        channel_id: String,
    },
    Console,
    Log,
}

/// Approval request that requires review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub hook_id: HookId,
    pub request_type: String,
    pub description: String,
    pub requested_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub priority: ApprovalPriority,
    pub required_approvers: Vec<String>,
    pub received_approvals: Vec<ApprovalDecision>,
    pub current_escalation_level: u32,
    pub notification_count: u32,
    pub last_notification_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Approval decision from a reviewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub approver: String,
    pub decision: ApprovalStatus,
    pub reason: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Status of an approval request
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Escalated,
    AutoApproved,
}

/// Priority level for approval requests
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ApprovalPriority {
    Low,
    Normal,
    Medium,
    High,
    Critical,
    Emergency,
}

/// Completed approval record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedApproval {
    pub request: ApprovalRequest,
    pub final_status: ApprovalStatus,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub total_duration_seconds: u64,
    pub escalation_count: u32,
    pub notification_count: u32,
}

/// Rule for determining approval requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub conditions: Vec<ApprovalCondition>,
    pub required_approvers: Vec<String>,
    pub auto_approval_allowed: bool,
    pub priority: ApprovalPriority,
    pub timeout_seconds: Option<u64>,
    pub escalation_enabled: bool,
}

/// Condition for approval rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    Matches, // Regex
}

/// Trait for notification handlers
pub trait NotificationHandler {
    fn send_notification(&self, request: &ApprovalRequest, message: &str) -> Result<()>;
    fn get_channel_type(&self) -> &'static str;
}

/// Console notification handler
pub struct ConsoleNotificationHandler;

impl NotificationHandler for ConsoleNotificationHandler {
    fn send_notification(&self, request: &ApprovalRequest, message: &str) -> Result<()> {
        println!("🔔 APPROVAL REQUIRED");
        println!("ID: {}", request.id);
        println!("Type: {}", request.request_type);
        println!("Description: {}", request.description);
        println!("Priority: {:?}", request.priority);
        println!("Message: {}", message);
        if let Some(expires_at) = request.expires_at {
            println!("Expires: {}", expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        println!("---");
        Ok(())
    }

    fn get_channel_type(&self) -> &'static str {
        "console"
    }
}

/// Log notification handler
pub struct LogNotificationHandler;

impl NotificationHandler for LogNotificationHandler {
    fn send_notification(&self, request: &ApprovalRequest, message: &str) -> Result<()> {
        tracing::info!(
            approval_id = %request.id,
            request_type = %request.request_type,
            priority = ?request.priority,
            message = %message,
            "Approval notification"
        );
        Ok(())
    }

    fn get_channel_type(&self) -> &'static str {
        "log"
    }
}

impl Default for ApprovalWorkflowConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 300, // 5 minutes
            max_concurrent_approvals: 10,
            require_explicit_approval: false,
            auto_approval: AutoApprovalConfig::default(),
            notifications: NotificationConfig::default(),
            escalation: EscalationConfig::default(),
        }
    }
}

impl Default for AutoApprovalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cost_threshold: 0.10, // $0.10
            quality_threshold: 0.8,
            allowed_request_types: vec!["cost_estimate".to_string(), "quality_warning".to_string()],
            rate_limit_per_hour: 20,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![NotificationChannel::Console, NotificationChannel::Log],
            initial_delay_seconds: 30,
            reminder_interval_seconds: 60,
            max_reminders: 3,
        }
    }
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            escalation_timeout_seconds: 180, // 3 minutes
            escalation_levels: vec![
                EscalationLevel {
                    level: 1,
                    timeout_seconds: 120,
                    approvers: vec!["supervisor".to_string()],
                    notification_channels: vec![NotificationChannel::Console],
                },
                EscalationLevel {
                    level: 2,
                    timeout_seconds: 240,
                    approvers: vec!["manager".to_string()],
                    notification_channels: vec![NotificationChannel::Console],
                },
            ],
            auto_approve_after_escalation: false,
        }
    }
}

impl ApprovalWorkflow {
    /// Create a new approval workflow
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            completed_requests: Arc::new(RwLock::new(HashMap::new())),
            approval_rules: Arc::new(RwLock::new(Vec::new())),
            notification_handlers: Arc::new(RwLock::new(vec![
                Box::new(ConsoleNotificationHandler),
                Box::new(LogNotificationHandler),
            ])),
            audit_logger: None,
            config: ApprovalWorkflowConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ApprovalWorkflowConfig) -> Self {
        let mut workflow = Self::new();
        workflow.config = config;
        workflow
    }

    /// Set audit logger
    pub fn with_audit_logger(mut self, audit_logger: Arc<HookAuditLogger>) -> Self {
        self.audit_logger = Some(audit_logger);
        self
    }

    /// Submit a new approval request
    pub async fn submit_approval_request(&self, mut request: ApprovalRequest) -> Result<String> {
        // Check if we're at the concurrent approval limit
        {
            let pending = self.pending_requests.read().await;
            if pending.len() >= self.config.max_concurrent_approvals {
                return Err(anyhow::anyhow!(
                    "Maximum concurrent approvals ({}) reached",
                    self.config.max_concurrent_approvals
                ));
            }
        }

        // Apply approval rules to determine requirements
        self.apply_approval_rules(&mut request).await?;

        // Check for auto-approval eligibility
        if self.can_auto_approve(&request).await? {
            return self.auto_approve_request(request).await;
        }

        // Set expiration if not provided
        if request.expires_at.is_none() {
            request.expires_at = Some(
                chrono::Utc::now()
                    + chrono::Duration::seconds(self.config.default_timeout_seconds as i64),
            );
        }

        let request_id = request.id.clone();

        // Store the pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id.clone(), request.clone());
        }

        // Log the submission
        if let Some(audit_logger) = &self.audit_logger {
            let event = AuditEvent::new(AuditEventType::ApprovalRequested {
                hook_id: HookId(request_id.clone()),
                execution_id: request_id.clone(),
                approvers: request.required_approvers.clone(),
            })
            .with_context(
                "details",
                format!("Approval request submitted: {}", request.description),
            )
            .with_context("result", "pending");

            audit_logger.log_event(event).await?;
        }

        // Send initial notification
        if self.config.notifications.enabled {
            self.send_notification(&request, "New approval request submitted")
                .await?;
        }

        // Start background monitoring for this request
        self.start_request_monitoring(request_id.clone()).await;

        Ok(request_id)
    }

    /// Process an approval decision
    pub async fn process_approval_decision(
        &self,
        request_id: &str,
        approver: &str,
        decision: ApprovalStatus,
        reason: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ApprovalProcessResult> {
        let mut pending = self.pending_requests.write().await;
        let mut request = pending
            .get(request_id)
            .ok_or_else(|| anyhow::anyhow!("Approval request not found: {}", request_id))?
            .clone();

        // Check if request has expired
        if let Some(expires_at) = request.expires_at {
            if chrono::Utc::now() > expires_at {
                pending.remove(request_id);
                drop(pending);

                return self
                    .complete_request(request, ApprovalStatus::Expired)
                    .await;
            }
        }

        // Add the approval decision
        let approval_decision = ApprovalDecision {
            approver: approver.to_string(),
            decision: decision.clone(),
            reason,
            timestamp: chrono::Utc::now(),
            metadata: metadata.unwrap_or_default(),
        };

        request.received_approvals.push(approval_decision);

        // Update the request in pending list
        pending.insert(request_id.to_string(), request.clone());
        drop(pending);

        // Log the decision
        if let Some(audit_logger) = &self.audit_logger {
            let event = match decision {
                ApprovalStatus::Approved => AuditEventType::ApprovalGranted {
                    hook_id: request.hook_id.clone(),
                    execution_id: request_id.to_string(),
                    approver: approver.to_string(),
                },
                ApprovalStatus::Rejected => AuditEventType::ApprovalDenied {
                    hook_id: request.hook_id.clone(),
                    execution_id: request_id.to_string(),
                    approver: approver.to_string(),
                },
                _ => AuditEventType::ApprovalRequested {
                    hook_id: request.hook_id.clone(),
                    execution_id: request_id.to_string(),
                    approvers: vec![approver.to_string()],
                },
            };

            let audit_event =
                AuditEvent::new(event).with_context("approval_decision", format!("{:?}", decision));
            audit_logger.log_event(audit_event).await?;
        }

        // Check if we have a final decision
        match decision {
            ApprovalStatus::Rejected => {
                // Remove from pending and complete
                let mut pending = self.pending_requests.write().await;
                pending.remove(request_id);
                drop(pending);

                self.complete_request(request, ApprovalStatus::Rejected)
                    .await
            }
            ApprovalStatus::Approved => {
                // Check if we have all required approvals
                if self.has_sufficient_approvals(&request).await? {
                    let mut pending = self.pending_requests.write().await;
                    pending.remove(request_id);
                    drop(pending);

                    self.complete_request(request, ApprovalStatus::Approved)
                        .await
                } else {
                    // Still need more approvals
                    Ok(ApprovalProcessResult {
                        final_status: ApprovalStatus::Pending,
                        approved: false,
                        message: "Additional approvals required".to_string(),
                        remaining_approvers: self.get_remaining_approvers(&request).await?,
                    })
                }
            }
            _ => {
                // Update status but keep pending
                Ok(ApprovalProcessResult {
                    final_status: ApprovalStatus::Pending,
                    approved: false,
                    message: format!("Approval decision recorded: {:?}", decision),
                    remaining_approvers: self.get_remaining_approvers(&request).await?,
                })
            }
        }
    }

    /// Get all pending approval requests
    pub async fn get_pending_approvals(&self) -> Result<Vec<ApprovalRequest>> {
        let pending = self.pending_requests.read().await;
        Ok(pending.values().cloned().collect())
    }

    /// Get a specific approval request by ID
    pub async fn get_approval_request(&self, request_id: &str) -> Result<Option<ApprovalRequest>> {
        let pending = self.pending_requests.read().await;
        Ok(pending.get(request_id).cloned())
    }

    /// Cancel a pending approval request
    pub async fn cancel_approval_request(&self, request_id: &str, reason: &str) -> Result<()> {
        let mut pending = self.pending_requests.write().await;
        let request = pending
            .remove(request_id)
            .ok_or_else(|| anyhow::anyhow!("Approval request not found: {}", request_id))?;
        drop(pending);

        // Log the cancellation
        if let Some(audit_logger) = &self.audit_logger {
            let event = AuditEventType::ExecutionDenied {
                hook_id: request.hook_id.clone(),
                execution_id: request_id.to_string(),
                reason: format!("Approval request cancelled: {}", reason),
            };

            let audit_event = AuditEvent::new(event)
                .with_context("cancelled_by", &request.requested_by)
                .with_context("reason", reason);
            audit_logger.log_event(audit_event).await?;
        }

        Ok(())
    }

    /// Add an approval rule
    pub async fn add_approval_rule(&self, rule: ApprovalRule) -> Result<()> {
        let mut rules = self.approval_rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// Remove an approval rule
    pub async fn remove_approval_rule(&self, rule_id: &str) -> Result<bool> {
        let mut rules = self.approval_rules.write().await;
        let initial_len = rules.len();
        rules.retain(|rule| rule.id != rule_id);
        Ok(rules.len() < initial_len)
    }

    /// List all approval rules
    pub async fn list_approval_rules(&self) -> Result<Vec<ApprovalRule>> {
        let rules = self.approval_rules.read().await;
        Ok(rules.clone())
    }

    /// Get approval statistics
    pub async fn get_approval_statistics(&self) -> Result<ApprovalStatistics> {
        let pending = self.pending_requests.read().await;
        let completed = self.completed_requests.read().await;

        let total_pending = pending.len();
        let total_completed = completed.len();

        let mut stats_by_type = HashMap::new();
        let mut stats_by_status = HashMap::new();

        // Count pending by type
        for request in pending.values() {
            *stats_by_type
                .entry(request.request_type.clone())
                .or_insert(0) += 1;
        }

        // Count completed by status
        for completed_approval in completed.values() {
            *stats_by_status
                .entry(completed_approval.final_status.clone())
                .or_insert(0) += 1;
        }

        let avg_completion_time = if !completed.is_empty() {
            completed
                .values()
                .map(|c| c.total_duration_seconds)
                .sum::<u64>() as f64
                / completed.len() as f64
        } else {
            0.0
        };

        Ok(ApprovalStatistics {
            total_pending,
            total_completed,
            stats_by_type,
            stats_by_status,
            average_completion_time_seconds: avg_completion_time,
        })
    }

    /// Clean up expired requests
    pub async fn cleanup_expired_requests(&self) -> Result<Vec<String>> {
        let now = chrono::Utc::now();
        let mut pending = self.pending_requests.write().await;
        let mut expired_ids = Vec::new();

        // Find expired requests
        let mut to_remove = Vec::new();
        for (id, request) in pending.iter() {
            if let Some(expires_at) = request.expires_at {
                if now > expires_at {
                    to_remove.push((id.clone(), request.clone()));
                }
            }
        }

        // Remove expired requests and complete them
        for (id, request) in to_remove {
            pending.remove(&id);
            expired_ids.push(id.clone());

            // Complete the request as expired
            let mut completed = self.completed_requests.write().await;
            let duration = (now - request.created_at).num_seconds() as u64;

            completed.insert(
                id,
                CompletedApproval {
                    request,
                    final_status: ApprovalStatus::Expired,
                    completed_at: now,
                    total_duration_seconds: duration,
                    escalation_count: 0,
                    notification_count: 0,
                },
            );
        }

        Ok(expired_ids)
    }

    // Private helper methods

    /// Apply approval rules to determine requirements
    async fn apply_approval_rules(&self, request: &mut ApprovalRequest) -> Result<()> {
        let rules = self.approval_rules.read().await;

        for rule in rules.iter() {
            if self.rule_matches_request(rule, request)? {
                // Apply rule requirements
                request
                    .required_approvers
                    .extend(rule.required_approvers.clone());

                if rule.priority > request.priority {
                    request.priority = rule.priority.clone();
                }

                if let Some(timeout) = rule.timeout_seconds {
                    if request.expires_at.is_none()
                        || request.expires_at.unwrap()
                            > chrono::Utc::now() + chrono::Duration::seconds(timeout as i64)
                    {
                        request.expires_at =
                            Some(chrono::Utc::now() + chrono::Duration::seconds(timeout as i64));
                    }
                }
            }
        }

        // Remove duplicates from required approvers
        request.required_approvers.sort();
        request.required_approvers.dedup();

        Ok(())
    }

    /// Check if a rule matches a request
    fn rule_matches_request(&self, rule: &ApprovalRule, request: &ApprovalRequest) -> Result<bool> {
        for condition in &rule.conditions {
            if !self.evaluate_condition(condition, request)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Evaluate a single condition
    fn evaluate_condition(
        &self,
        condition: &ApprovalCondition,
        request: &ApprovalRequest,
    ) -> Result<bool> {
        let actual_value = self.get_field_value(&condition.field, request)?;

        match condition.operator {
            ConditionOperator::Equals => Ok(actual_value == condition.value),
            ConditionOperator::NotEquals => Ok(actual_value != condition.value),
            ConditionOperator::GreaterThan => {
                if let (Some(actual), Some(expected)) =
                    (actual_value.as_f64(), condition.value.as_f64())
                {
                    Ok(actual > expected)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::LessThan => {
                if let (Some(actual), Some(expected)) =
                    (actual_value.as_f64(), condition.value.as_f64())
                {
                    Ok(actual < expected)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::GreaterThanOrEqual => {
                if let (Some(actual), Some(expected)) =
                    (actual_value.as_f64(), condition.value.as_f64())
                {
                    Ok(actual >= expected)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::LessThanOrEqual => {
                if let (Some(actual), Some(expected)) =
                    (actual_value.as_f64(), condition.value.as_f64())
                {
                    Ok(actual <= expected)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::Contains => {
                if let (Some(actual), Some(expected)) =
                    (actual_value.as_str(), condition.value.as_str())
                {
                    Ok(actual.contains(expected))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::StartsWith => {
                if let (Some(actual), Some(expected)) =
                    (actual_value.as_str(), condition.value.as_str())
                {
                    Ok(actual.starts_with(expected))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::EndsWith => {
                if let (Some(actual), Some(expected)) =
                    (actual_value.as_str(), condition.value.as_str())
                {
                    Ok(actual.ends_with(expected))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::Matches => {
                if let (Some(actual), Some(pattern)) =
                    (actual_value.as_str(), condition.value.as_str())
                {
                    match regex::Regex::new(pattern) {
                        Ok(re) => Ok(re.is_match(actual)),
                        Err(_) => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Get field value from request
    fn get_field_value(&self, field: &str, request: &ApprovalRequest) -> Result<serde_json::Value> {
        match field {
            "request_type" => Ok(serde_json::Value::String(request.request_type.clone())),
            "requested_by" => Ok(serde_json::Value::String(request.requested_by.clone())),
            "priority" => Ok(serde_json::Value::String(format!("{:?}", request.priority))),
            "description" => Ok(serde_json::Value::String(request.description.clone())),
            _ => {
                // Check metadata
                if let Some(value) = request.metadata.get(field) {
                    Ok(value.clone())
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
        }
    }

    /// Check if request can be auto-approved
    async fn can_auto_approve(&self, request: &ApprovalRequest) -> Result<bool> {
        if !self.config.auto_approval.enabled {
            return Ok(false);
        }

        // Check if request type is allowed for auto-approval
        if !self
            .config
            .auto_approval
            .allowed_request_types
            .contains(&request.request_type)
        {
            return Ok(false);
        }

        // Check cost threshold
        if let Some(cost) = request.metadata.get("cost").and_then(|v| v.as_f64()) {
            if cost > self.config.auto_approval.cost_threshold {
                return Ok(false);
            }
        }

        // Check quality threshold
        if let Some(quality) = request
            .metadata
            .get("quality_score")
            .and_then(|v| v.as_f64())
        {
            if quality < self.config.auto_approval.quality_threshold {
                return Ok(false);
            }
        }

        // TODO: Check rate limiting

        Ok(true)
    }

    /// Auto-approve a request
    async fn auto_approve_request(&self, mut request: ApprovalRequest) -> Result<String> {
        let decision = ApprovalDecision {
            approver: "system".to_string(),
            decision: ApprovalStatus::AutoApproved,
            reason: Some("Auto-approved based on configured rules".to_string()),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        request.received_approvals.push(decision);
        let request_id = request.id.clone();

        // Complete the request immediately
        let now = chrono::Utc::now();
        let duration = (now - request.created_at).num_seconds() as u64;

        let hook_id = request.hook_id.clone();

        let mut completed = self.completed_requests.write().await;
        completed.insert(
            request_id.clone(),
            CompletedApproval {
                request,
                final_status: ApprovalStatus::AutoApproved,
                completed_at: now,
                total_duration_seconds: duration,
                escalation_count: 0,
                notification_count: 0,
            },
        );

        // Log the auto-approval
        if let Some(audit_logger) = &self.audit_logger {
            let event = AuditEventType::ApprovalGranted {
                hook_id,
                execution_id: request_id.clone(),
                approver: "system".to_string(),
            };

            let audit_event = AuditEvent::new(event)
                .with_context("auto_approved", true)
                .with_context("duration_ms", duration);
            audit_logger.log_event(audit_event).await?;
        }

        Ok(request_id)
    }

    /// Check if request has sufficient approvals
    async fn has_sufficient_approvals(&self, request: &ApprovalRequest) -> Result<bool> {
        if request.required_approvers.is_empty() {
            // If no specific approvers required, any approval is sufficient
            return Ok(request
                .received_approvals
                .iter()
                .any(|a| a.decision == ApprovalStatus::Approved));
        }

        // Check if all required approvers have approved
        let approved_by: Vec<&String> = request
            .received_approvals
            .iter()
            .filter(|a| a.decision == ApprovalStatus::Approved)
            .map(|a| &a.approver)
            .collect();

        Ok(request
            .required_approvers
            .iter()
            .all(|required| approved_by.contains(&required)))
    }

    /// Get remaining approvers needed
    async fn get_remaining_approvers(&self, request: &ApprovalRequest) -> Result<Vec<String>> {
        if request.required_approvers.is_empty() {
            return Ok(Vec::new());
        }

        let approved_by: Vec<&String> = request
            .received_approvals
            .iter()
            .filter(|a| a.decision == ApprovalStatus::Approved)
            .map(|a| &a.approver)
            .collect();

        let remaining: Vec<String> = request
            .required_approvers
            .iter()
            .filter(|required| !approved_by.contains(required))
            .cloned()
            .collect();

        Ok(remaining)
    }

    /// Complete a request with final status
    async fn complete_request(
        &self,
        request: ApprovalRequest,
        final_status: ApprovalStatus,
    ) -> Result<ApprovalProcessResult> {
        let now = chrono::Utc::now();
        let duration = (now - request.created_at).num_seconds() as u64;
        let approved = final_status == ApprovalStatus::Approved
            || final_status == ApprovalStatus::AutoApproved;

        // Store completed request
        let mut completed = self.completed_requests.write().await;
        completed.insert(
            request.id.clone(),
            CompletedApproval {
                request: request.clone(),
                final_status: final_status.clone(),
                completed_at: now,
                total_duration_seconds: duration,
                escalation_count: request.current_escalation_level,
                notification_count: request.notification_count,
            },
        );

        // Log completion
        if let Some(audit_logger) = &self.audit_logger {
            let event = match final_status {
                ApprovalStatus::Approved => AuditEventType::ApprovalGranted {
                    hook_id: request.hook_id.clone(),
                    execution_id: request.id.clone(),
                    approver: request.requested_by.clone(),
                },
                ApprovalStatus::Rejected => AuditEventType::ApprovalDenied {
                    hook_id: request.hook_id.clone(),
                    execution_id: request.id.clone(),
                    approver: request.requested_by.clone(),
                },
                _ => AuditEventType::ExecutionCompleted {
                    hook_id: request.hook_id.clone(),
                    execution_id: request.id.clone(),
                    success: final_status == ApprovalStatus::Approved,
                    duration_ms: duration * 1000,
                },
            };

            let audit_event = AuditEvent::new(event)
                .with_context("approval_status", format!("{:?}", final_status))
                .with_context("duration_seconds", duration);
            audit_logger.log_event(audit_event).await?;
        }

        Ok(ApprovalProcessResult {
            final_status: final_status.clone(),
            approved,
            message: format!("Request completed: {:?}", final_status),
            remaining_approvers: Vec::new(),
        })
    }

    /// Send notification for a request
    async fn send_notification(&self, request: &ApprovalRequest, message: &str) -> Result<()> {
        let handlers = self.notification_handlers.read().await;

        for handler in handlers.iter() {
            if let Err(e) = handler.send_notification(request, message) {
                tracing::warn!(
                    "Failed to send notification via {}: {}",
                    handler.get_channel_type(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Start monitoring a request for timeouts and escalations
    async fn start_request_monitoring(&self, request_id: String) {
        let workflow = self.clone();
        tokio::spawn(async move {
            // Initial delay before first notification
            tokio::time::sleep(tokio::time::Duration::from_secs(
                workflow.config.notifications.initial_delay_seconds,
            ))
            .await;

            let mut reminder_count = 0;
            loop {
                // Check if request still exists and needs monitoring
                let needs_monitoring = {
                    let pending = workflow.pending_requests.read().await;
                    pending.contains_key(&request_id)
                };

                if !needs_monitoring {
                    break;
                }

                // Send reminder if configured
                if workflow.config.notifications.enabled
                    && reminder_count < workflow.config.notifications.max_reminders
                {
                    if let Ok(Some(request)) = workflow.get_approval_request(&request_id).await {
                        let _ = workflow
                            .send_notification(
                                &request,
                                &format!(
                                    "Reminder {}: Approval still required",
                                    reminder_count + 1
                                ),
                            )
                            .await;
                        reminder_count += 1;
                    }
                }

                // Wait for next reminder interval
                tokio::time::sleep(tokio::time::Duration::from_secs(
                    workflow.config.notifications.reminder_interval_seconds,
                ))
                .await;
            }
        });
    }
}

// Clone implementation for Arc compatibility
impl Clone for ApprovalWorkflow {
    fn clone(&self) -> Self {
        Self {
            pending_requests: self.pending_requests.clone(),
            completed_requests: self.completed_requests.clone(),
            approval_rules: self.approval_rules.clone(),
            notification_handlers: self.notification_handlers.clone(),
            audit_logger: self.audit_logger.clone(),
            config: self.config.clone(),
        }
    }
}

/// Result of approval processing
#[derive(Debug, Clone)]
pub struct ApprovalProcessResult {
    pub final_status: ApprovalStatus,
    pub approved: bool,
    pub message: String,
    pub remaining_approvers: Vec<String>,
}

/// Approval statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStatistics {
    pub total_pending: usize,
    pub total_completed: usize,
    pub stats_by_type: HashMap<String, usize>,
    pub stats_by_status: HashMap<ApprovalStatus, usize>,
    pub average_completion_time_seconds: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_approval_workflow_creation() {
        let workflow = ApprovalWorkflow::new();
        let stats = workflow.get_approval_statistics().await.unwrap();
        assert_eq!(stats.total_pending, 0);
        assert_eq!(stats.total_completed, 0);
    }

    #[tokio::test]
    async fn test_approval_request_submission() {
        let workflow = ApprovalWorkflow::new();

        let request = ApprovalRequest {
            id: Uuid::new_v4().to_string(),
            hook_id: HookId::new(),
            request_type: "test".to_string(),
            description: "Test approval request".to_string(),
            requested_by: "user1".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
            priority: ApprovalPriority::Normal,
            required_approvers: vec!["manager".to_string()],
            received_approvals: Vec::new(),
            current_escalation_level: 0,
            notification_count: 0,
            last_notification_at: None,
        };

        let request_id = workflow.submit_approval_request(request).await.unwrap();
        assert!(!request_id.is_empty());

        let stats = workflow.get_approval_statistics().await.unwrap();
        assert_eq!(stats.total_pending, 1);
    }

    #[tokio::test]
    async fn test_auto_approval() {
        let mut config = ApprovalWorkflowConfig::default();
        config.auto_approval.enabled = true;
        config.auto_approval.cost_threshold = 1.0;
        config.auto_approval.allowed_request_types = vec!["test".to_string()];

        let workflow = ApprovalWorkflow::with_config(config);

        let mut metadata = HashMap::new();
        metadata.insert(
            "cost".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()),
        );
        metadata.insert(
            "quality_score".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.9).unwrap()),
        );

        let request = ApprovalRequest {
            id: Uuid::new_v4().to_string(),
            hook_id: HookId::new(),
            request_type: "test".to_string(),
            description: "Test auto-approval".to_string(),
            requested_by: "user1".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            metadata,
            priority: ApprovalPriority::Normal,
            required_approvers: Vec::new(),
            received_approvals: Vec::new(),
            current_escalation_level: 0,
            notification_count: 0,
            last_notification_at: None,
        };

        let request_id = workflow.submit_approval_request(request).await.unwrap();

        // Should be auto-approved, so no pending requests
        let stats = workflow.get_approval_statistics().await.unwrap();
        assert_eq!(stats.total_pending, 0);
        assert_eq!(stats.total_completed, 1);
    }
}
