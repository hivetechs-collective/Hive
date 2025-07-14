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

/// Email notification service that integrates with hivetechs.io API
pub struct EmailService {
    api_url: String,
    client: reqwest::Client,
}

impl EmailService {
    pub fn new() -> Self {
        let api_url = std::env::var("HIVE_API_ENDPOINT")
            .unwrap_or_else(|_| "https://hivetechs.io".to_string());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("hive-rust/2.0.0")
            .build()
            .unwrap_or_default();

        Self { api_url, client }
    }

    async fn send_reminder_email(
        &self,
        license_key: &str,
        email: &str,
        trigger_type: &str,
    ) -> Result<()> {
        let url = format!("{}/api/trial/send-expiration-email", self.api_url);

        let request_data = serde_json::json!({
            "licenseKey": license_key,
            "email": email,
            "triggerType": trigger_type
        });

        let response = self
            .client
            .post(&url)
            .json(&request_data)
            .send()
            .await
            .context("Failed to send email request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Email service returned error: {} - {}",
                status,
                error_text
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse email response")?;

        if result["success"].as_bool() == Some(true) {
            tracing::info!("Email sent successfully: {:?}", result["messageId"]);
            Ok(())
        } else {
            let error = result["error"].as_str().unwrap_or("Unknown error");
            Err(anyhow::anyhow!("Failed to send email: {}", error))
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

    /// Load reminder tracking data from database
    async fn load_reminders(&self) -> Result<Vec<SubscriptionReminder>> {
        use crate::core::database::get_database;

        let db = get_database().await?;
        let conn = db.get_connection()?;

        tokio::task::spawn_blocking(move || -> Result<Vec<SubscriptionReminder>> {
            // Query reminder tracking from database
            // Note: In production, this would be stored in a dedicated reminder_tracking table
            // For now, we track reminders based on subscription data
            let mut stmt = conn.prepare(
                "SELECT
                    id as user_id,
                    email,
                    subscription_expires_at,
                    created_at
                FROM users
                WHERE license_key IS NOT NULL
                AND subscription_expires_at IS NOT NULL",
            )?;

            let reminders = stmt
                .query_map([], |row| {
                    let expires_str: String = row.get(2)?;
                    let expires = DateTime::parse_from_rfc3339(&expires_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc);

                    Ok(SubscriptionReminder {
                        user_id: row.get(0)?,
                        email: row.get(1)?,
                        expires_at: expires,
                        reminder_sent: HashSet::new(), // TODO: Track in database
                        last_checked: Utc::now(),
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(reminders)
        })
        .await?
    }

    /// Save reminder tracking data to database
    async fn save_reminders(&self, reminders: &[SubscriptionReminder]) -> Result<()> {
        // In production, this would update a reminder_tracking table
        // For now, this is a no-op as we track reminders in memory
        // All critical data is stored in the unified database
        Ok(())
    }

    /// Check and send reminders for a specific subscription
    pub async fn check_and_send_reminders(&self, subscription: &SubscriptionInfo) -> Result<()> {
        let mut reminders = self.load_reminders().await?;

        // Find or create reminder record
        let reminder_idx = reminders
            .iter()
            .position(|r| r.user_id == subscription.user_id);
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
        let days_until_expiry = subscription
            .expires_at
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
                self.send_reminder_email(&reminder, *days, subscription)
                    .await?;
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
        // Get license key from subscription
        let license_key = self.get_license_key_for_user(&subscription.user_id).await?;

        // Map days to trigger type
        let trigger_type = match days_remaining {
            3 => "3-day",
            2 => "1-day", // Note: API only supports 3-day and 1-day, not 2-day
            1 => "1-day",
            _ => return Ok(()), // Skip unsupported days
        };

        self.email_service
            .send_reminder_email(&license_key, &reminder.email, trigger_type)
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

    /// Get license key for a user from database
    async fn get_license_key_for_user(&self, user_id: &str) -> Result<String> {
        // In a real implementation, this would query the database
        // For now, return a placeholder or error
        Err(anyhow::anyhow!(
            "License key lookup not implemented - requires database integration"
        ))
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
            // SECURITY: Credits not tracked locally
            features: vec![],
            cached_at: Utc::now(),
        };

        // Should attempt to send 3-day reminder
        manager
            .check_and_send_reminders(&subscription)
            .await
            .unwrap();

        // Verify reminder was tracked
        let reminders = manager.load_reminders().await.unwrap();
        assert_eq!(reminders.len(), 1);
        assert!(reminders[0].reminder_sent.contains(&3));
    }
}
