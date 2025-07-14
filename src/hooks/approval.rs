//! Hook Approval Workflows - Approval system for sensitive hook operations

use super::{ExecutionContext, HookId};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Status of an approval request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
    Cancelled,
}

/// An approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub hook_id: HookId,
    pub execution_id: String,
    pub approvers: Vec<String>,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ApprovalStatus,
}

/// Response to an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub request_id: String,
    pub approver: String,
    pub approved: bool,
    pub comment: Option<String>,
    pub responded_at: DateTime<Utc>,
}

/// Notification for approval requests
#[derive(Debug, Clone)]
pub struct ApprovalNotification {
    pub request: ApprovalRequest,
    pub urgency: NotificationUrgency,
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationUrgency {
    Low,
    Normal,
    High,
    Critical,
}

/// Approval workflow manager
pub struct ApprovalWorkflow {
    pending_requests: Arc<RwLock<HashMap<String, ApprovalRequest>>>,
    responses: Arc<RwLock<HashMap<String, Vec<ApprovalResponse>>>>,
    notification_tx: mpsc::Sender<ApprovalNotification>,
    notification_rx: Arc<RwLock<mpsc::Receiver<ApprovalNotification>>>,
}

impl ApprovalWorkflow {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
            notification_tx: tx,
            notification_rx: Arc::new(RwLock::new(rx)),
        }
    }

    /// Request approval for a hook execution
    pub async fn request_approval(
        &self,
        hook_id: &HookId,
        context: &ExecutionContext,
    ) -> Result<bool> {
        let request = ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            hook_id: hook_id.clone(),
            execution_id: context.execution_id.clone(),
            approvers: vec!["admin".to_string()], // Default approvers
            message: format!("Hook '{}' requires approval", hook_id.0),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
            status: ApprovalStatus::Pending,
        };

        self.request_approval_with_details(request).await
    }

    /// Request approval with detailed configuration
    pub async fn request_approval_with_details(&self, request: ApprovalRequest) -> Result<bool> {
        let request_id = request.id.clone();

        // Store request
        {
            let mut requests = self.pending_requests.write().await;
            requests.insert(request_id.clone(), request.clone());
        }

        // Send notification
        let notification = ApprovalNotification {
            request: request.clone(),
            urgency: self.calculate_urgency(&request),
        };

        let _ = self.notification_tx.send(notification).await;

        // In a real implementation, this would:
        // 1. Send notifications to approvers (email, Slack, etc.)
        // 2. Wait for responses
        // 3. Check approval rules (unanimous, majority, etc.)

        // For now, simulate approval process
        self.simulate_approval(&request_id).await
    }

    /// Submit an approval response
    pub async fn submit_response(&self, response: ApprovalResponse) -> Result<()> {
        // Validate request exists and is pending
        let request = {
            let requests = self.pending_requests.read().await;
            requests
                .get(&response.request_id)
                .cloned()
                .ok_or_else(|| anyhow!("Approval request not found"))?
        };

        if request.status != ApprovalStatus::Pending {
            return Err(anyhow!("Request is no longer pending"));
        }

        if request.expires_at < Utc::now() {
            // Mark as expired
            let mut requests = self.pending_requests.write().await;
            if let Some(req) = requests.get_mut(&response.request_id) {
                req.status = ApprovalStatus::Expired;
            }
            return Err(anyhow!("Request has expired"));
        }

        // Store response
        let mut responses = self.responses.write().await;
        responses
            .entry(response.request_id.clone())
            .or_insert_with(Vec::new)
            .push(response.clone());

        // Check if we have enough approvals
        let request_responses = responses.get(&response.request_id).unwrap();
        let approvals = request_responses.iter().filter(|r| r.approved).count();
        let denials = request_responses.iter().filter(|r| !r.approved).count();

        // Update status based on responses
        let mut requests = self.pending_requests.write().await;
        if let Some(req) = requests.get_mut(&response.request_id) {
            if denials > 0 {
                req.status = ApprovalStatus::Denied;
            } else if approvals >= req.approvers.len() / 2 + 1 {
                req.status = ApprovalStatus::Approved;
            }
        }

        Ok(())
    }

    /// Get pending approval requests
    pub async fn get_pending_requests(&self) -> Result<Vec<ApprovalRequest>> {
        let requests = self.pending_requests.read().await;
        Ok(requests
            .values()
            .filter(|r| r.status == ApprovalStatus::Pending)
            .cloned()
            .collect())
    }

    /// Get approval request by ID
    pub async fn get_request(&self, request_id: &str) -> Result<Option<ApprovalRequest>> {
        let requests = self.pending_requests.read().await;
        Ok(requests.get(request_id).cloned())
    }

    /// Cancel an approval request
    pub async fn cancel_request(&self, request_id: &str) -> Result<()> {
        let mut requests = self.pending_requests.write().await;
        let request = requests
            .get_mut(request_id)
            .ok_or_else(|| anyhow!("Request not found"))?;

        if request.status != ApprovalStatus::Pending {
            return Err(anyhow!("Request is no longer pending"));
        }

        request.status = ApprovalStatus::Cancelled;
        Ok(())
    }

    /// Check and expire old requests
    pub async fn expire_old_requests(&self) -> Result<()> {
        let now = Utc::now();
        let mut requests = self.pending_requests.write().await;

        for request in requests.values_mut() {
            if request.status == ApprovalStatus::Pending && request.expires_at < now {
                request.status = ApprovalStatus::Expired;
            }
        }

        Ok(())
    }

    /// Get approval statistics
    pub async fn get_statistics(&self) -> ApprovalStatistics {
        let requests = self.pending_requests.read().await;

        let mut stats = ApprovalStatistics::default();

        for request in requests.values() {
            stats.total_requests += 1;

            match request.status {
                ApprovalStatus::Pending => stats.pending_requests += 1,
                ApprovalStatus::Approved => stats.approved_requests += 1,
                ApprovalStatus::Denied => stats.denied_requests += 1,
                ApprovalStatus::Expired => stats.expired_requests += 1,
                ApprovalStatus::Cancelled => stats.cancelled_requests += 1,
            }
        }

        stats
    }

    /// Calculate notification urgency
    fn calculate_urgency(&self, request: &ApprovalRequest) -> NotificationUrgency {
        let time_until_expiry = request.expires_at.signed_duration_since(Utc::now());

        if time_until_expiry.num_minutes() < 5 {
            NotificationUrgency::Critical
        } else if time_until_expiry.num_minutes() < 15 {
            NotificationUrgency::High
        } else if time_until_expiry.num_hours() < 1 {
            NotificationUrgency::Normal
        } else {
            NotificationUrgency::Low
        }
    }

    /// Simulate approval for development/testing
    async fn simulate_approval(&self, request_id: &str) -> Result<bool> {
        // In development, auto-approve after showing prompt
        use dialoguer::{theme::ColorfulTheme, Confirm};

        let request = self
            .get_request(request_id)
            .await?
            .ok_or_else(|| anyhow!("Request not found"))?;

        println!("\n🔔 Approval Required");
        println!("─────────────────────");
        println!("Hook: {}", request.hook_id.0);
        println!("Message: {}", request.message);
        println!(
            "Expires: {}",
            request.expires_at.format("%Y-%m-%d %H:%M:%S UTC")
        );

        let approved = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you approve this hook execution?")
            .default(false)
            .interact()?;

        let response = ApprovalResponse {
            request_id: request_id.to_string(),
            approver: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
            approved,
            comment: None,
            responded_at: Utc::now(),
        };

        self.submit_response(response).await?;

        Ok(approved)
    }
}

/// Approval statistics
#[derive(Debug, Default, Serialize)]
pub struct ApprovalStatistics {
    pub total_requests: usize,
    pub pending_requests: usize,
    pub approved_requests: usize,
    pub denied_requests: usize,
    pub expired_requests: usize,
    pub cancelled_requests: usize,
}

/// Approval rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRules {
    pub require_all_approvers: bool,
    pub minimum_approvals: usize,
    pub allow_self_approval: bool,
    pub auto_approve_trusted_hooks: bool,
    pub escalation_timeout_minutes: Option<u32>,
}

impl Default for ApprovalRules {
    fn default() -> Self {
        Self {
            require_all_approvers: false,
            minimum_approvals: 1,
            allow_self_approval: false,
            auto_approve_trusted_hooks: false,
            escalation_timeout_minutes: None,
        }
    }
}

/// Approval notification handler
pub struct ApprovalNotificationHandler {
    receiver: Arc<RwLock<mpsc::Receiver<ApprovalNotification>>>,
}

impl ApprovalNotificationHandler {
    pub fn new(receiver: Arc<RwLock<mpsc::Receiver<ApprovalNotification>>>) -> Self {
        Self { receiver }
    }

    /// Process pending notifications
    pub async fn process_notifications(&self) -> Result<()> {
        let mut rx = self.receiver.write().await;

        while let Ok(notification) = rx.try_recv() {
            self.handle_notification(notification).await?;
        }

        Ok(())
    }

    /// Handle a single notification
    async fn handle_notification(&self, notification: ApprovalNotification) -> Result<()> {
        match notification.urgency {
            NotificationUrgency::Critical => {
                // Send immediate notifications (email, Slack, etc.)
                tracing::error!(
                    "CRITICAL approval required: {}",
                    notification.request.message
                );
            }
            NotificationUrgency::High => {
                tracing::warn!(
                    "High priority approval required: {}",
                    notification.request.message
                );
            }
            NotificationUrgency::Normal => {
                tracing::info!("Approval required: {}", notification.request.message);
            }
            NotificationUrgency::Low => {
                tracing::debug!("Approval requested: {}", notification.request.message);
            }
        }

        // In a real implementation, this would:
        // - Send emails to approvers
        // - Post to Slack/Teams channels
        // - Create tickets in issue tracking systems
        // - Send push notifications

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_approval_workflow() {
        let workflow = ApprovalWorkflow::new();

        let request = ApprovalRequest {
            id: "test-123".to_string(),
            hook_id: HookId::new(),
            execution_id: "exec-456".to_string(),
            approvers: vec!["alice".to_string(), "bob".to_string()],
            message: "Test approval".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            status: ApprovalStatus::Pending,
        };

        // Submit request
        let mut requests = workflow.pending_requests.write().await;
        requests.insert(request.id.clone(), request.clone());
        drop(requests);

        // Submit approval
        let response = ApprovalResponse {
            request_id: "test-123".to_string(),
            approver: "alice".to_string(),
            approved: true,
            comment: Some("Looks good".to_string()),
            responded_at: Utc::now(),
        };

        workflow.submit_response(response).await.unwrap();

        // Check status
        let updated_request = workflow.get_request("test-123").await.unwrap().unwrap();
        assert_eq!(updated_request.status, ApprovalStatus::Approved);
    }
}
