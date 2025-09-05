//! Hook Audit Logging - Comprehensive audit trail for hook executions

use super::{execution::ExecutionContext, execution::ExecutionResult, HookId};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEventType {
    HookRegistered {
        hook_id: HookId,
        hook_name: String,
    },
    HookRemoved {
        hook_id: HookId,
    },
    HookModified {
        hook_id: HookId,
        changes: Vec<String>,
    },
    ExecutionStarted {
        hook_id: HookId,
        execution_id: String,
        event_type: String,
    },
    ExecutionCompleted {
        hook_id: HookId,
        execution_id: String,
        success: bool,
        duration_ms: u64,
    },
    ExecutionFailed {
        hook_id: HookId,
        execution_id: String,
        error: String,
    },
    ExecutionSkipped {
        hook_id: HookId,
        execution_id: String,
        reason: String,
    },
    ExecutionDenied {
        hook_id: HookId,
        execution_id: String,
        reason: String,
    },
    SecurityViolation {
        hook_id: HookId,
        violation: String,
    },
    ApprovalRequested {
        hook_id: HookId,
        execution_id: String,
        approvers: Vec<String>,
    },
    ApprovalGranted {
        hook_id: HookId,
        execution_id: String,
        approver: String,
    },
    ApprovalDenied {
        hook_id: HookId,
        execution_id: String,
        approver: String,
    },
    ApprovalDecisionMade {
        hook_id: HookId,
        execution_id: String,
        decision: String,
    },
    ApprovalCancelled {
        hook_id: HookId,
        execution_id: String,
        reason: String,
    },
    ApprovalAutoApproved {
        hook_id: HookId,
        execution_id: String,
        reason: String,
    },
    ApprovalCompleted {
        hook_id: HookId,
        execution_id: String,
        result: String,
    },
    QualityGateAdded {
        hook_id: HookId,
        gate_name: String,
    },
    QualityGateRemoved {
        hook_id: HookId,
        gate_name: String,
    },
    BudgetCreated {
        hook_id: HookId,
        budget_name: String,
        amount: String,
    },
    AllHooksCleared {
        count: usize,
    },
}

/// Audit event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event: AuditEventType,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub context: serde_json::Map<String, serde_json::Value>,
}

impl AuditEvent {
    pub fn new(event: AuditEventType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event,
            user_id: std::env::var("USER").ok(),
            session_id: None,
            context: serde_json::Map::new(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.context.insert(key.into(), v);
        }
        self
    }
}

/// Audit logger for hooks
pub struct HookAuditLogger {
    log_file: Arc<Mutex<File>>,
    log_path: PathBuf,
    max_size_bytes: u64,
    retention_days: u32,
}

impl HookAuditLogger {
    pub async fn new(log_path: PathBuf) -> Result<Self> {
        // Create log directory if needed
        if let Some(parent) = log_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Open or create log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await?;

        Ok(Self {
            log_file: Arc::new(Mutex::new(file)),
            log_path,
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            retention_days: 90,
        })
    }

    /// Log an audit event
    pub async fn log_event(&self, event: AuditEvent) -> Result<()> {
        let json = serde_json::to_string(&event)?;
        let line = format!("{}\n", json);

        let mut file = self.log_file.lock().await;
        file.write_all(line.as_bytes()).await?;
        file.flush().await?;

        // Check if rotation is needed
        drop(file);
        self.rotate_if_needed().await?;

        Ok(())
    }

    /// Log hook registration
    pub async fn log_hook_registered(&self, hook_id: &HookId) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::HookRegistered {
            hook_id: hook_id.clone(),
            hook_name: String::new(), // TODO: Get from hook
        });
        self.log_event(event).await
    }

    /// Log hook removal
    pub async fn log_hook_removed(&self, hook_id: &HookId) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::HookRemoved {
            hook_id: hook_id.clone(),
        });
        self.log_event(event).await
    }

    /// Log hook enabled
    pub async fn log_hook_enabled(&self, hook_id: &HookId) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::HookModified {
            hook_id: hook_id.clone(),
            changes: vec!["enabled".to_string()],
        });
        self.log_event(event).await
    }

    /// Log hook disabled
    pub async fn log_hook_disabled(&self, hook_id: &HookId) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::HookModified {
            hook_id: hook_id.clone(),
            changes: vec!["disabled".to_string()],
        });
        self.log_event(event).await
    }

    /// Log execution start
    pub async fn log_execution_start(
        &self,
        hook_id: &HookId,
        context: &ExecutionContext,
    ) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::ExecutionStarted {
            hook_id: hook_id.clone(),
            execution_id: context.execution_id.clone(),
            event_type: format!("{:?}", context.event.event_type),
        })
        .with_context("dry_run", context.dry_run);

        self.log_event(event).await
    }

    /// Log execution completion
    pub async fn log_execution_complete(
        &self,
        hook_id: &HookId,
        result: &ExecutionResult,
    ) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::ExecutionCompleted {
            hook_id: hook_id.clone(),
            execution_id: result.execution_id.clone(),
            success: result.success,
            duration_ms: result.duration_ms,
        })
        .with_context("actions_count", result.actions_executed.len());

        self.log_event(event).await
    }

    /// Log execution skipped
    pub async fn log_execution_skipped(
        &self,
        hook_id: &HookId,
        context: &ExecutionContext,
        reason: &str,
    ) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::ExecutionSkipped {
            hook_id: hook_id.clone(),
            execution_id: context.execution_id.clone(),
            reason: reason.to_string(),
        });

        self.log_event(event).await
    }

    /// Log execution denied
    pub async fn log_execution_denied(
        &self,
        hook_id: &HookId,
        context: &ExecutionContext,
    ) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::ExecutionDenied {
            hook_id: hook_id.clone(),
            execution_id: context.execution_id.clone(),
            reason: "Approval denied".to_string(),
        });

        self.log_event(event).await
    }

    /// Log security violation
    pub async fn log_security_violation(&self, hook_id: &HookId, violation: &str) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::SecurityViolation {
            hook_id: hook_id.clone(),
            violation: violation.to_string(),
        });

        self.log_event(event).await
    }

    /// Log all hooks cleared
    pub async fn log_all_hooks_cleared(&self) -> Result<()> {
        let event = AuditEvent::new(AuditEventType::AllHooksCleared {
            count: 0, // TODO: Get actual count
        });

        self.log_event(event).await
    }

    /// Get recent audit logs
    pub async fn get_recent_logs(&self, limit: usize) -> Result<Vec<AuditEvent>> {
        let file = File::open(&self.log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut events = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                events.push(event);
            }
        }

        // Return most recent events
        events.reverse();
        events.truncate(limit);
        Ok(events)
    }

    /// Search audit logs
    pub async fn search_logs(&self, criteria: SearchCriteria) -> Result<Vec<AuditEvent>> {
        let file = File::open(&self.log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut events = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                if criteria.matches(&event) {
                    events.push(event);
                }
            }
        }

        Ok(events)
    }

    /// Rotate log file if needed
    async fn rotate_if_needed(&self) -> Result<()> {
        let metadata = tokio::fs::metadata(&self.log_path).await?;

        if metadata.len() > self.max_size_bytes {
            // Rotate the log
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let rotated_path = self
                .log_path
                .with_file_name(format!("hooks_audit_{}.log", timestamp));

            // Close current file
            drop(self.log_file.lock().await);

            // Rename current log
            tokio::fs::rename(&self.log_path, &rotated_path).await?;

            // Open new file
            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_path)
                .await?;

            *self.log_file.lock().await = new_file;

            // Clean up old logs
            self.cleanup_old_logs().await?;
        }

        Ok(())
    }

    /// Clean up old log files
    async fn cleanup_old_logs(&self) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days as i64);

        if let Some(parent) = self.log_path.parent() {
            let mut entries = tokio::fs::read_dir(parent).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("hooks_audit_") && n.ends_with(".log"))
                    .unwrap_or(false)
                {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            let modified_time: DateTime<Utc> = modified.into();
                            if modified_time < cutoff {
                                let _ = tokio::fs::remove_file(path).await;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Search criteria for audit logs
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub hook_id: Option<HookId>,
    pub event_types: Option<Vec<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub success_only: bool,
}

impl SearchCriteria {
    pub fn new() -> Self {
        Self {
            hook_id: None,
            event_types: None,
            start_time: None,
            end_time: None,
            user_id: None,
            success_only: false,
        }
    }

    pub fn with_hook_id(mut self, hook_id: HookId) -> Self {
        self.hook_id = Some(hook_id);
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn matches(&self, event: &AuditEvent) -> bool {
        // Check hook ID
        if let Some(ref hook_id) = self.hook_id {
            let event_hook_id = match &event.event {
                AuditEventType::HookRegistered { hook_id, .. }
                | AuditEventType::HookRemoved { hook_id }
                | AuditEventType::HookModified { hook_id, .. }
                | AuditEventType::ExecutionStarted { hook_id, .. }
                | AuditEventType::ExecutionCompleted { hook_id, .. }
                | AuditEventType::ExecutionFailed { hook_id, .. }
                | AuditEventType::ExecutionSkipped { hook_id, .. }
                | AuditEventType::ExecutionDenied { hook_id, .. }
                | AuditEventType::SecurityViolation { hook_id, .. }
                | AuditEventType::ApprovalRequested { hook_id, .. }
                | AuditEventType::ApprovalGranted { hook_id, .. }
                | AuditEventType::ApprovalDenied { hook_id, .. }
                | AuditEventType::ApprovalDecisionMade { hook_id, .. }
                | AuditEventType::ApprovalCancelled { hook_id, .. }
                | AuditEventType::ApprovalAutoApproved { hook_id, .. }
                | AuditEventType::ApprovalCompleted { hook_id, .. }
                | AuditEventType::QualityGateAdded { hook_id, .. }
                | AuditEventType::QualityGateRemoved { hook_id, .. }
                | AuditEventType::BudgetCreated { hook_id, .. } => Some(hook_id),
                _ => None,
            };

            if event_hook_id != Some(hook_id) {
                return false;
            }
        }

        // Check time range
        if let Some(start) = self.start_time {
            if event.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if event.timestamp > end {
                return false;
            }
        }

        // Check user ID
        if let Some(ref user_id) = self.user_id {
            if event.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        // Check success only
        if self.success_only {
            match &event.event {
                AuditEventType::ExecutionCompleted { success, .. } => {
                    if !success {
                        return false;
                    }
                }
                AuditEventType::ExecutionFailed { .. }
                | AuditEventType::ExecutionDenied { .. }
                | AuditEventType::SecurityViolation { .. } => {
                    return false;
                }
                _ => {}
            }
        }

        true
    }
}

/// Generate audit report
pub struct AuditReport {
    pub total_events: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub security_violations: usize,
    pub most_active_hooks: Vec<(HookId, usize)>,
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
}

impl AuditReport {
    pub async fn generate(logger: &HookAuditLogger, days: u32) -> Result<Self> {
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::days(days as i64);

        let criteria = SearchCriteria::new().with_time_range(start_time, end_time);

        let events = logger.search_logs(criteria).await?;

        let mut report = Self {
            total_events: events.len(),
            successful_executions: 0,
            failed_executions: 0,
            security_violations: 0,
            most_active_hooks: Vec::new(),
            time_range: (start_time, end_time),
        };

        let mut hook_counts = std::collections::HashMap::new();

        for event in &events {
            match &event.event {
                AuditEventType::ExecutionCompleted {
                    hook_id, success, ..
                } => {
                    *hook_counts.entry(hook_id.clone()).or_insert(0) += 1;
                    if *success {
                        report.successful_executions += 1;
                    } else {
                        report.failed_executions += 1;
                    }
                }
                AuditEventType::ExecutionFailed { hook_id, .. } => {
                    *hook_counts.entry(hook_id.clone()).or_insert(0) += 1;
                    report.failed_executions += 1;
                }
                AuditEventType::SecurityViolation { .. } => {
                    report.security_violations += 1;
                }
                _ => {}
            }
        }

        // Sort hooks by activity
        let mut hook_list: Vec<_> = hook_counts.into_iter().collect();
        hook_list.sort_by(|a, b| b.1.cmp(&a.1));
        report.most_active_hooks = hook_list.into_iter().take(10).collect();

        Ok(report)
    }
}
