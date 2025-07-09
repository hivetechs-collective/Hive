// Usage Tracking Module - Tracks daily conversation usage and trial periods
// Implements the 7-day unlimited trial and tier-based daily limits

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use crate::core::database_simple::Database;
use crate::core::license::{LicenseTier, LicenseInfo};
use rusqlite::params;

/// User usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUsageInfo {
    pub user_id: String,
    pub license_tier: LicenseTier,
    pub daily_limit: u32,
    pub daily_usage: u32,
    pub usage_reset_date: Option<DateTime<Utc>>,
    pub trial_active: bool,
    pub trial_start_date: Option<DateTime<Utc>>,
    pub trial_end_date: Option<DateTime<Utc>>,
    pub trial_days_remaining: i32,
    pub is_unlimited: bool,
}

/// Usage check result
#[derive(Debug, Clone)]
pub struct UsageCheckResult {
    pub allowed: bool,
    pub reason: String,
    pub remaining_conversations: i32,
    pub usage_info: UserUsageInfo,
}

/// Usage display information for UI
#[derive(Debug, Clone, Serialize)]
pub struct UsageDisplay {
    pub daily_used: u32,
    pub daily_limit: u32,
    pub daily_percentage: f32,
    pub remaining_today: i32,
    pub reset_time: Option<String>,
    pub status_color: String,
    pub status_emoji: String,
    pub status_message: String,
    pub is_trial: bool,
    pub trial_days_left: Option<i32>,
}

/// Usage tracking service
pub struct UsageTracker {
    database: Arc<Database>,
}

impl UsageTracker {
    /// Create new usage tracker
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
    
    /// Check if user can run a conversation
    pub async fn check_usage_before_conversation(&self, user_id: &str) -> Result<UsageCheckResult> {
        let usage_info = self.get_or_create_user_usage(user_id).await?;
        
        // Check if in trial period
        if usage_info.trial_active && usage_info.trial_days_remaining > 0 {
            if let Some(trial_end) = usage_info.trial_end_date {
                if Utc::now() < trial_end {
                    return Ok(UsageCheckResult {
                        allowed: true,
                        reason: format!("Trial period active - {} days remaining", usage_info.trial_days_remaining),
                        remaining_conversations: 999999, // Unlimited during trial
                        usage_info,
                    });
                }
            }
        }
        
        // Check if unlimited tier
        if usage_info.is_unlimited {
            return Ok(UsageCheckResult {
                allowed: true,
                reason: "Unlimited tier - no conversation limits".to_string(),
                remaining_conversations: 999999,
                usage_info,
            });
        }
        
        // Check daily usage
        let remaining = usage_info.daily_limit as i32 - usage_info.daily_usage as i32;
        if remaining > 0 {
            Ok(UsageCheckResult {
                allowed: true,
                reason: format!("{} conversations remaining today", remaining),
                remaining_conversations: remaining,
                usage_info,
            })
        } else {
            Ok(UsageCheckResult {
                allowed: false,
                reason: "Daily conversation limit reached".to_string(),
                remaining_conversations: 0,
                usage_info,
            })
        }
    }
    
    /// Record successful conversation usage
    pub async fn record_conversation_usage(&self, user_id: &str, conversation_id: &str) -> Result<()> {
        let mut conn = self.database.get_connection().await?;
        
        // Get current usage info
        let usage_info = self.get_or_create_user_usage(user_id).await?;
        
        // Skip recording during trial
        if usage_info.trial_active && usage_info.trial_days_remaining > 0 {
            if let Some(trial_end) = usage_info.trial_end_date {
                if Utc::now() < trial_end {
                    info!("Skipping usage recording for user {} - in trial period", user_id);
                    return Ok(());
                }
            }
        }
        
        // Skip for unlimited tier
        if usage_info.is_unlimited {
            info!("Skipping usage recording for user {} - unlimited tier", user_id);
            return Ok(());
        }
        
        // Check if we need to reset daily usage
        let now = Utc::now();
        let should_reset = if let Some(reset_date) = usage_info.usage_reset_date {
            now >= reset_date
        } else {
            true
        };
        
        let (new_usage, new_reset_date) = if should_reset {
            // Reset usage and set new reset date (24 hours from now)
            (1, Some(now + Duration::hours(24)))
        } else {
            // Increment usage
            (usage_info.daily_usage + 1, usage_info.usage_reset_date)
        };
        
        // Update database
        let reset_date_str = new_reset_date.map(|d| d.to_rfc3339());
        conn.execute(
            "UPDATE user_profiles 
             SET daily_usage = ?1, 
                 usage_reset_date = ?2,
                 updated_at = datetime('now')
             WHERE id = ?3",
            params![new_usage, reset_date_str, user_id],
        )?;
        
        info!("Recorded usage for user {} - daily usage now: {}/{}", 
              user_id, new_usage, usage_info.daily_limit);
        
        Ok(())
    }
    
    /// Get or create user usage information
    async fn get_or_create_user_usage(&self, user_id: &str) -> Result<UserUsageInfo> {
        let mut conn = self.database.get_connection().await?;
        
        // Try to get existing user
        let user_result: Option<(String, u32, u32, Option<String>, Option<String>, Option<String>)> = 
            conn.query_row(
                "SELECT license_tier, daily_limit, daily_usage, usage_reset_date, 
                        created_at, updated_at
                 FROM user_profiles WHERE id = ?1",
                [user_id],
                |row| Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                )),
            ).optional()?;
        
        let (tier_str, daily_limit, daily_usage, usage_reset_str, created_at_str, updated_at_str) = 
            if let Some(user_data) = user_result {
                user_data
            } else {
                // Create new user with trial
                let now = Utc::now();
                let trial_end = now + Duration::days(7);
                
                conn.execute(
                    "INSERT INTO user_profiles (id, license_tier, daily_limit, daily_usage, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![user_id, "free", 10, 0, now.to_rfc3339()],
                )?;
                
                info!("Created new user {} with 7-day trial until {}", user_id, trial_end);
                
                ("free".to_string(), 10, 0, None, Some(now.to_rfc3339()), None)
            };
        
        // Parse dates
        let created_at = created_at_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&Utc));
        
        let usage_reset_date = usage_reset_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&Utc));
        
        // Calculate trial status
        let (trial_active, trial_start_date, trial_end_date, trial_days_remaining) = 
            self.calculate_trial_status(created_at, updated_at_str.as_deref());
        
        // Parse tier
        let license_tier = self.parse_tier(&tier_str);
        let is_unlimited = matches!(license_tier, LicenseTier::Unlimited | LicenseTier::Enterprise);
        
        Ok(UserUsageInfo {
            user_id: user_id.to_string(),
            license_tier,
            daily_limit,
            daily_usage,
            usage_reset_date,
            trial_active,
            trial_start_date,
            trial_end_date,
            trial_days_remaining,
            is_unlimited,
        })
    }
    
    /// Calculate trial status based on user creation date
    fn calculate_trial_status(
        &self, 
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<&str>,
    ) -> (bool, Option<DateTime<Utc>>, Option<DateTime<Utc>>, i32) {
        if let Some(created) = created_at {
            let trial_end = created + Duration::days(7);
            let now = Utc::now();
            
            // Check if user has been updated (likely means they've configured a license)
            let has_configured_license = updated_at.is_some() && updated_at != Some(&created.to_rfc3339());
            
            if now < trial_end && !has_configured_license {
                let days_remaining = (trial_end - now).num_days() as i32;
                (true, Some(created), Some(trial_end), days_remaining)
            } else {
                (false, Some(created), Some(trial_end), 0)
            }
        } else {
            (false, None, None, 0)
        }
    }
    
    /// Get usage display information for UI
    pub async fn get_usage_display(&self, user_id: &str) -> Result<UsageDisplay> {
        let usage_info = self.get_or_create_user_usage(user_id).await?;
        
        // Calculate percentage
        let daily_percentage = if usage_info.daily_limit > 0 {
            (usage_info.daily_usage as f32 / usage_info.daily_limit as f32) * 100.0
        } else {
            0.0
        };
        
        // Determine status color and emoji
        let (status_color, status_emoji) = if usage_info.trial_active {
            ("#10b981".to_string(), "🎉") // Green for trial
        } else if usage_info.is_unlimited {
            ("#3b82f6".to_string(), "♾️") // Blue for unlimited
        } else if daily_percentage >= 90.0 {
            ("#ef4444".to_string(), "🔴") // Red
        } else if daily_percentage >= 75.0 {
            ("#f97316".to_string(), "🟠") // Orange
        } else if daily_percentage >= 50.0 {
            ("#eab308".to_string(), "🟡") // Yellow
        } else {
            ("#10b981".to_string(), "🟢") // Green
        };
        
        // Create status message
        let status_message = if usage_info.trial_active {
            format!("Trial: {} days left", usage_info.trial_days_remaining)
        } else if usage_info.is_unlimited {
            "Unlimited conversations".to_string()
        } else {
            format!("{}/{} today", usage_info.daily_usage, usage_info.daily_limit)
        };
        
        // Calculate reset time
        let reset_time = usage_info.usage_reset_date
            .map(|d| {
                let duration = d - Utc::now();
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                format!("{}h {}m", hours, minutes)
            });
        
        let remaining_today = if usage_info.trial_active || usage_info.is_unlimited {
            999999
        } else {
            (usage_info.daily_limit as i32 - usage_info.daily_usage as i32).max(0)
        };
        
        Ok(UsageDisplay {
            daily_used: usage_info.daily_usage,
            daily_limit: usage_info.daily_limit,
            daily_percentage,
            remaining_today,
            reset_time,
            status_color,
            status_emoji: status_emoji.to_string(),
            status_message,
            is_trial: usage_info.trial_active,
            trial_days_left: if usage_info.trial_active { 
                Some(usage_info.trial_days_remaining) 
            } else { 
                None 
            },
        })
    }
    
    /// Update user's license tier
    pub async fn update_user_tier(&self, user_id: &str, tier: LicenseTier) -> Result<()> {
        let daily_limit = match tier {
            LicenseTier::Free => 10,
            LicenseTier::Basic => 50,
            LicenseTier::Standard => 100,
            LicenseTier::Premium => 200,
            LicenseTier::Unlimited | LicenseTier::Enterprise => 999999,
        };
        
        let mut conn = self.database.get_connection().await?;
        conn.execute(
            "UPDATE user_profiles 
             SET license_tier = ?1, 
                 daily_limit = ?2,
                 updated_at = datetime('now')
             WHERE id = ?3",
            params![tier.to_string(), daily_limit, user_id],
        )?;
        
        info!("Updated user {} to {} tier with {} daily limit", 
              user_id, tier, daily_limit);
        
        Ok(())
    }
    
    /// Parse tier string
    fn parse_tier(&self, tier: &str) -> LicenseTier {
        match tier.to_lowercase().as_str() {
            "basic" => LicenseTier::Basic,
            "standard" => LicenseTier::Standard,
            "premium" => LicenseTier::Premium,
            "unlimited" => LicenseTier::Unlimited,
            "enterprise" => LicenseTier::Enterprise,
            _ => LicenseTier::Free,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_trial_period_calculation() {
        let tracker = UsageTracker::new(Arc::new(Database::new(":memory:".into())));
        
        // Test new user gets 7-day trial
        let created = Utc::now();
        let (active, start, end, days) = tracker.calculate_trial_status(Some(created), None);
        
        assert!(active);
        assert_eq!(start, Some(created));
        assert!(end.is_some());
        assert_eq!(days, 7);
        
        // Test expired trial
        let old_created = Utc::now() - Duration::days(10);
        let (active, _, _, days) = tracker.calculate_trial_status(Some(old_created), None);
        
        assert!(!active);
        assert_eq!(days, 0);
    }
    
    #[test]
    fn test_tier_parsing() {
        let tracker = UsageTracker::new(Arc::new(Database::new(":memory:".into())));
        
        assert_eq!(tracker.parse_tier("free"), LicenseTier::Free);
        assert_eq!(tracker.parse_tier("Basic"), LicenseTier::Basic);
        assert_eq!(tracker.parse_tier("UNLIMITED"), LicenseTier::Unlimited);
        assert_eq!(tracker.parse_tier("unknown"), LicenseTier::Free);
    }
}