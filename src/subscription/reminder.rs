//! Subscription reminder system
//! Sends email notifications at 3, 2, and 1 days before subscription expiry

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

use super::SubscriptionInfo;

/// Subscription reminder tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionReminder {
    pub user_id: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub reminder_sent: HashSet<u8>, // Track 3, 2, 1 day reminders
    pub last_checked: DateTime<Utc>,
}

/// Email notification service (placeholder for actual implementation)
pub struct EmailService {
    api_key: Option<String>,
    from_email: String,
}

impl EmailService {
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("HIVE_EMAIL_API_KEY").ok(),
            from_email: "noreply@hivetechs.io".to_string(),
        }
    }

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        // In production, this would integrate with an email service like SendGrid, SES, etc.
        tracing::info!("Would send email to {} with subject: {}", to, subject);
        tracing::debug!("Email body: {}", body);
        
        // For now, just log the email
        if self.api_key.is_some() {
            // Would make actual API call here
            Ok(())
        } else {
            tracing::warn!("Email API key not configured, skipping email send");
            Ok(())
        }
    }
}

/// Reminder manager
pub struct ReminderManager {
    config_dir: PathBuf,
    email_service: EmailService,
}

impl ReminderManager {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            email_service: EmailService::new(),
        }
    }

    /// Load reminder tracking data
    async fn load_reminders(&self) -> Result<Vec<SubscriptionReminder>> {
        let reminder_file = self.config_dir.join("subscription_reminders.json");
        
        if reminder_file.exists() {
            let content = tokio::fs::read_to_string(&reminder_file).await?;
            let reminders: Vec<SubscriptionReminder> = serde_json::from_str(&content)?;
            Ok(reminders)
        } else {
            Ok(Vec::new())
        }
    }

    /// Save reminder tracking data
    async fn save_reminders(&self, reminders: &[SubscriptionReminder]) -> Result<()> {
        let reminder_file = self.config_dir.join("subscription_reminders.json");
        let content = serde_json::to_string_pretty(reminders)?;
        tokio::fs::write(&reminder_file, content).await?;
        Ok(())
    }

    /// Check and send reminders for a specific subscription
    pub async fn check_and_send_reminders(&self, subscription: &SubscriptionInfo) -> Result<()> {
        let mut reminders = self.load_reminders().await?;
        
        // Find or create reminder record
        let reminder_idx = reminders.iter().position(|r| r.user_id == subscription.user_id);
        let mut reminder = if let Some(idx) = reminder_idx {
            reminders[idx].clone()
        } else {
            SubscriptionReminder {
                user_id: subscription.user_id.clone(),
                email: subscription.email.clone(),
                expires_at: subscription.expires_at,
                reminder_sent: HashSet::new(),
                last_checked: Utc::now(),
            }
        };

        // Update expiry date if changed
        if reminder.expires_at != subscription.expires_at {
            reminder.expires_at = subscription.expires_at;
            reminder.reminder_sent.clear(); // Reset reminders if subscription renewed
        }

        // Check if we should send reminders
        let now = Utc::now();
        let days_until_expiry = subscription.expires_at
            .signed_duration_since(now)
            .num_days();

        // Only check once per day
        if reminder.last_checked.date_naive() == now.date_naive() {
            return Ok(());
        }

        reminder.last_checked = now;

        // Send appropriate reminders
        for days in &[3, 2, 1] {
            if days_until_expiry == *days as i64 && !reminder.reminder_sent.contains(days) {
                self.send_reminder_email(&reminder, *days, subscription).await?;
                reminder.reminder_sent.insert(*days);
            }
        }

        // Update reminder record
        if let Some(idx) = reminder_idx {
            reminders[idx] = reminder;
        } else {
            reminders.push(reminder);
        }

        self.save_reminders(&reminders).await?;
        Ok(())
    }

    /// Send a reminder email
    async fn send_reminder_email(
        &self,
        reminder: &SubscriptionReminder,
        days_remaining: u8,
        subscription: &SubscriptionInfo,
    ) -> Result<()> {
        let subject = format!(
            "Your Hive AI {} subscription expires in {} day{}",
            subscription.tier,
            days_remaining,
            if days_remaining == 1 { "" } else { "s" }
        );

        let body = self.generate_email_body(days_remaining, subscription);

        self.email_service
            .send_email(&reminder.email, &subject, &body)
            .await
            .context("Failed to send reminder email")?;

        tracing::info!(
            "Sent {}-day reminder email to {} for {} subscription",
            days_remaining,
            reminder.email,
            subscription.tier
        );

        Ok(())
    }

    /// Generate email body based on days remaining
    fn generate_email_body(&self, days_remaining: u8, subscription: &SubscriptionInfo) -> String {
        let mut body = format!(
            "Hi there,\n\n\
            Your Hive AI {} subscription is expiring in {} day{}.\n\n",
            subscription.tier,
            days_remaining,
            if days_remaining == 1 { "" } else { "s" }
        );

        match days_remaining {
            3 => {
                body.push_str(
                    "We wanted to give you a heads up so you have time to renew if you'd like to continue \
                    enjoying all the benefits of your subscription.\n\n\
                    Your subscription includes:\n"
                );
                body.push_str(&format!("• {} daily conversations\n", subscription.daily_limit));
                body.push_str(&format!("• {} monthly conversations\n", subscription.monthly_limit));
                body.push_str("• Multi-model consensus pipeline\n");
                body.push_str("• Advanced analytics and reporting\n");
                body.push_str("• Priority support\n\n");
                body.push_str("Renew now to ensure uninterrupted service.\n");
            }
            2 => {
                body.push_str(
                    "Your subscription is expiring soon! Don't lose access to your enhanced features.\n\n\
                    After expiration, you'll be moved to the Free tier with:\n\
                    • 10 daily conversations only\n\
                    • Basic features\n\n\
                    Renew today to keep your current benefits.\n"
                );
            }
            1 => {
                body.push_str(
                    "⚠️ This is your final reminder!\n\n\
                    Your subscription expires tomorrow. After that, you'll automatically be moved to the \
                    Free tier with limited features.\n\n\
                    Renew now to avoid any interruption in service.\n"
                );
            }
            _ => {}
        }

        body.push_str(&format!(
            "\n🔗 Renew your subscription: https://hivetechs.io/account/subscription\n\n\
            If you have any questions or need assistance, please don't hesitate to reach out to \
            our support team at support@hivetechs.io.\n\n\
            Best regards,\n\
            The Hive AI Team\n\n\
            P.S. If you've already renewed, please disregard this email. Your subscription status \
            will update within a few minutes."
        ));

        body
    }

    /// Check all subscriptions for reminders (batch process)
    pub async fn check_all_reminders(&self) -> Result<()> {
        // This would be called by a scheduled job to check all users
        // For now, it's a placeholder for the batch processing logic
        
        tracing::info!("Checking all subscription reminders...");
        
        // In production, this would:
        // 1. Query all active subscriptions from D1
        // 2. Check each one for reminder eligibility
        // 3. Send reminders as needed
        // 4. Update tracking data
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_reminder_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ReminderManager::new(temp_dir.path().to_path_buf());

        let subscription = SubscriptionInfo {
            user_id: "test_user".to_string(),
            email: "test@example.com".to_string(),
            tier: super::super::SubscriptionTier::Premium,
            daily_limit: 200,
            monthly_limit: 4000,
            expires_at: Utc::now() + Duration::days(3),
            trial_ends_at: None,
            credits_remaining: 0,
            features: vec![],
            cached_at: Utc::now(),
        };

        // Should attempt to send 3-day reminder
        manager.check_and_send_reminders(&subscription).await.unwrap();

        // Verify reminder was tracked
        let reminders = manager.load_reminders().await.unwrap();
        assert_eq!(reminders.len(), 1);
        assert!(reminders[0].reminder_sent.contains(&3));
    }

    #[test]
    fn test_email_body_generation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ReminderManager::new(temp_dir.path().to_path_buf());

        let subscription = SubscriptionInfo {
            user_id: "test".to_string(),
            email: "test@example.com".to_string(),
            tier: super::super::SubscriptionTier::Standard,
            daily_limit: 100,
            monthly_limit: 2000,
            expires_at: Utc::now() + Duration::days(1),
            trial_ends_at: None,
            credits_remaining: 0,
            features: vec![],
            cached_at: Utc::now(),
        };

        let body = manager.generate_email_body(1, &subscription);
        assert!(body.contains("final reminder"));
        assert!(body.contains("expires tomorrow"));
    }
}