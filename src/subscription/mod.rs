//! Subscription management system with D1 integration
//!
//! This module provides:
//! - User subscription management
//! - Daily conversation tracking
//! - Usage monitoring and warnings
//! - Credit pack consumption
//! - Trial period handling

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod conversation_gateway;
pub mod reminder;
pub mod usage_tracker;

pub use conversation_gateway::{ConversationAuthorization, ConversationGateway};
pub use reminder::SubscriptionReminder;
pub use usage_tracker::{UsageNotification, UsageTracker};

/// Subscription tiers matching TypeScript implementation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Basic,
    Standard,
    Premium,
    Unlimited,
    Team,
}

impl SubscriptionTier {
    pub fn daily_limit(&self) -> u32 {
        match self {
            Self::Free => 10,
            Self::Basic => 50,
            Self::Standard => 100,
            Self::Premium => 200,
            Self::Unlimited => u32::MAX,
            Self::Team => 600,
        }
    }

    pub fn monthly_limit(&self) -> u32 {
        match self {
            Self::Free => 300,
            Self::Basic => 1000,
            Self::Standard => 2000,
            Self::Premium => 4000,
            Self::Unlimited => u32::MAX,
            Self::Team => 12000,
        }
    }

    pub fn price(&self) -> &'static str {
        match self {
            Self::Free => "$0/month",
            Self::Basic => "$5/month",
            Self::Standard => "$10/month",
            Self::Premium => "$20/month",
            Self::Unlimited => "$30/month",
            Self::Team => "$115/month",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "basic" => Self::Basic,
            "standard" => Self::Standard,
            "premium" => Self::Premium,
            "unlimited" => Self::Unlimited,
            "team" => Self::Team,
            _ => Self::Free,
        }
    }
}

impl std::fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "FREE"),
            Self::Basic => write!(f, "BASIC"),
            Self::Standard => write!(f, "STANDARD"),
            Self::Premium => write!(f, "PREMIUM"),
            Self::Unlimited => write!(f, "UNLIMITED"),
            Self::Team => write!(f, "TEAM"),
        }
    }
}

/// User subscription information (cached from D1)
/// SECURITY: This is READ-ONLY cache data. Never modify locally.
/// Credits and limits are always verified with D1 before use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub user_id: String,
    pub email: String,
    pub tier: SubscriptionTier,
    pub daily_limit: u32,
    pub monthly_limit: u32,
    pub expires_at: DateTime<Utc>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    // SECURITY: Removed credits_remaining - must fetch from D1
    pub features: Vec<String>,
    pub cached_at: DateTime<Utc>,
}

impl SubscriptionInfo {
    /// Check if user is in trial period
    pub fn is_in_trial(&self) -> bool {
        if let Some(trial_ends) = self.trial_ends_at {
            trial_ends > Utc::now()
        } else {
            false
        }
    }

    /// Get effective daily limit (unlimited during trial)
    pub fn effective_daily_limit(&self) -> u32 {
        if self.is_in_trial() {
            u32::MAX
        } else {
            self.daily_limit
        }
    }

    /// Check if cache is still valid (24 hours)
    pub fn is_cache_valid(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age.num_hours() < 24
    }
}

/// Conversation verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationVerification {
    pub verified: bool,
    pub remaining_conversations: u32,
    pub usage_updated: bool,
}

/// Usage statistics (local tracking only)
/// SECURITY: Credits are NOT tracked here - only D1 knows credit balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub daily_used: u32,
    pub monthly_used: u32,
    // SECURITY: Removed credits_remaining - must fetch from D1
    pub last_conversation: Option<DateTime<Utc>>,
}

/// Credit pack options
#[derive(Debug, Clone, Copy)]
pub enum CreditPack {
    Starter, // 25 credits for $3
    Value,   // 75 credits for $7
    Power,   // 200 credits for $15
}

impl CreditPack {
    pub fn credits(&self) -> u32 {
        match self {
            Self::Starter => 25,
            Self::Value => 75,
            Self::Power => 200,
        }
    }

    pub fn price(&self) -> f32 {
        match self {
            Self::Starter => 3.0,
            Self::Value => 7.0,
            Self::Power => 15.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Starter => "Starter Pack",
            Self::Value => "Value Pack",
            Self::Power => "Power Pack",
        }
    }
}

/// License change detection
#[derive(Debug, Clone, PartialEq)]
pub enum LicenseChange {
    NoChange,
    Renewed,
    Upgraded {
        from: SubscriptionTier,
        to: SubscriptionTier,
    },
    Downgraded {
        from: SubscriptionTier,
        to: SubscriptionTier,
    },
    Cancelled,
}

/// Subscription display information for UI
/// SECURITY: This is display-only data. Actual remaining count comes from D1.
pub struct SubscriptionDisplay {
    pub username: String,
    pub tier: SubscriptionTier,
    pub daily_remaining: u32,
    pub daily_limit: u32,
    pub daily_used: u32,
    pub total_remaining: Option<u32>, // From D1 (includes daily + credits)
    pub is_trial: bool,
    pub average_daily_usage: Option<f32>, // Average since account creation
    pub days_active: u32,                 // Days since account creation
}

impl SubscriptionDisplay {
    pub fn format(&self) -> String {
        let tier_display = if self.is_trial {
            "FREE (Trial)"
        } else {
            match self.tier {
                SubscriptionTier::Free => "FREE",
                SubscriptionTier::Basic => "BASIC",
                SubscriptionTier::Standard => "STANDARD",
                SubscriptionTier::Premium => "PREMIUM",
                SubscriptionTier::Unlimited => "UNLIMITED",
                SubscriptionTier::Team => "TEAM",
            }
        };

        // Format average usage
        let avg_display = if let Some(avg) = self.average_daily_usage {
            format!(" | Avg: {:.1}/day", avg)
        } else {
            String::new()
        };

        // Always prefer D1's authoritative count when available
        let base_display = if let Some(total) = self.total_remaining {
            if self.tier == SubscriptionTier::Unlimited {
                format!("{} | {} | Unlimited", self.username, tier_display)
            } else if total == 0 {
                // Out of conversations - show clear limit reached message
                format!(
                    "{} | {} | Daily limit reached ({}/{})",
                    self.username, tier_display, self.daily_used, self.daily_limit
                )
            } else if self.daily_remaining == 0 && total > 0 {
                // Using credit packs
                format!(
                    "{} | {} | {} credits remaining",
                    self.username, tier_display, total
                )
            } else if self.is_trial && total > 0 {
                // During trial, show remaining from D1
                format!(
                    "{} | {} | {} remaining (trial)",
                    self.username, tier_display, total
                )
            } else {
                // Normal display with remaining
                format!(
                    "{} | {} | {} remaining today",
                    self.username, tier_display, total
                )
            }
        } else {
            // Fallback when D1 info not available (local display only)
            if self.daily_remaining == 0 {
                format!(
                    "{} | {} | Daily limit reached ({}/{})",
                    self.username, tier_display, self.daily_used, self.daily_limit
                )
            } else if self.is_trial {
                format!(
                    "{} | {} | {} remaining (trial)",
                    self.username, tier_display, self.daily_remaining
                )
            } else {
                format!(
                    "{} | {} | {}/{} daily",
                    self.username, tier_display, self.daily_remaining, self.daily_limit
                )
            }
        };

        // Append average usage
        format!("{}{}", base_display, avg_display)
    }

    /// Load subscription display info from database
    pub async fn load_from_database() -> Result<Self> {
        use crate::core::database::get_database;
        use chrono::{Datelike, Timelike, Utc};

        let db = get_database().await?;
        let conn = db.get_connection()?;

        // Query all subscription info from unified database
        let result = tokio::task::spawn_blocking(move || -> Result<(String, String, String, u32, bool, Option<f32>, u32)> {
            use rusqlite::OptionalExtension;

            // First, get the currently configured license key
            let current_license_key = conn.query_row(
                "SELECT value FROM configurations WHERE key = 'hive_license_key'",
                [],
                |row| row.get::<_, String>(0)
            ).optional()?;

            if current_license_key.is_none() {
                return Ok(("No license".to_string(), "free".to_string(), String::new(), 0, false, None, 0));
            }

            let license_key = current_license_key.unwrap();

            // Get user info that matches the current license key
            let user_result = conn.query_row(
                "SELECT id, email, tier, license_key FROM users WHERE license_key = ?1",
                rusqlite::params![&license_key],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                }
            ).optional()?;

            let (user_id, email, tier_str, license_key) = match user_result {
                Some(data) => data,
                None => return Ok(("No user".to_string(), "free".to_string(), String::new(), 0, false, None, 0)),
            };

            // Get conversation count for today from conversation_usage table
            // Using UTC midnight as the reset point for global consistency
            let now = Utc::now();
            let start_of_day = now
                .with_hour(0).unwrap()
                .with_minute(0).unwrap()
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap();

            let daily_used: u32 = conn.query_row(
                "SELECT COUNT(*) FROM conversation_usage
                 WHERE user_id = ?1
                 AND datetime(timestamp) >= datetime(?2)",
                rusqlite::params![&user_id, start_of_day.to_rfc3339()],
                |row| row.get(0)
            ).unwrap_or(0);

            tracing::debug!("Daily usage query: user_id={}, start_of_day={}, count={}",
                         user_id, start_of_day.to_rfc3339(), daily_used);

            // SECURITY: DO NOT query credits_remaining - only D1 knows the balance

            // Check if in trial period (first 7 days after user creation)
            let is_trial = conn.query_row(
                "SELECT
                    CAST((julianday('now') - julianday(created_at)) AS INTEGER) < 7 as is_trial
                FROM users
                WHERE id = ?1",
                rusqlite::params![&user_id],
                |row| row.get::<_, bool>(0)
            ).unwrap_or(false);

            // Calculate average daily usage since account creation
            let (total_conversations, days_active): (u32, u32) = conn.query_row(
                "SELECT
                    COUNT(*) as total_conversations,
                    MAX(1, CAST((julianday('now') - julianday(MIN(u.created_at))) AS INTEGER) + 1) as days_active
                FROM conversation_usage cu
                JOIN users u ON cu.user_id = u.id
                WHERE cu.user_id = ?1",
                rusqlite::params![&user_id],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).unwrap_or((0, 1));

            let average_daily_usage = if days_active > 0 {
                Some(total_conversations as f32 / days_active as f32)
            } else {
                None
            };

            tracing::debug!("Average usage calculation: total={}, days={}, avg={:?}",
                         total_conversations, days_active, average_daily_usage);

            Ok((email, tier_str, license_key, daily_used, is_trial, average_daily_usage, days_active))
        }).await??;

        let (
            username,
            subscription_tier,
            _license_key,
            daily_used,
            is_trial,
            average_daily_usage,
            days_active,
        ) = result;

        // Parse tier
        let tier = SubscriptionTier::from_string(&subscription_tier);

        // Calculate remaining daily conversations
        let daily_limit = if is_trial {
            u32::MAX // Unlimited during trial
        } else {
            tier.daily_limit()
        };

        let daily_remaining = if daily_limit == u32::MAX {
            u32::MAX
        } else if daily_limit > daily_used {
            daily_limit - daily_used
        } else {
            0
        };

        Ok(SubscriptionDisplay {
            username,
            tier,
            daily_remaining,
            daily_limit,
            daily_used,
            total_remaining: None, // Will be updated from D1 responses
            is_trial,
            average_daily_usage,
            days_active,
        })
    }

    /// Update display with fresh data from D1
    pub fn update_from_d1(&mut self, total_remaining: u32) {
        self.total_remaining = Some(total_remaining);

        // If D1 says we have 0 remaining and we're on a free plan,
        // we're definitely not in trial (trial gives unlimited)
        if total_remaining == 0 && self.tier == SubscriptionTier::Free {
            self.is_trial = false;
            self.daily_remaining = 0;
        }
    }

    /// Get recommended tier based on average daily usage
    pub fn recommended_tier(&self) -> Option<(SubscriptionTier, &'static str)> {
        let avg = self.average_daily_usage?;

        Some(match avg {
            _ if avg <= 8.0 => (SubscriptionTier::Free, "Your FREE plan is perfect!"),
            _ if avg <= 40.0 => (
                SubscriptionTier::Basic,
                "Upgrade to BASIC (50/day) - Save 70% vs credits",
            ),
            _ if avg <= 80.0 => (
                SubscriptionTier::Standard,
                "Upgrade to STANDARD (100/day) - Best value",
            ),
            _ if avg <= 150.0 => (SubscriptionTier::Premium, "Upgrade to PREMIUM (200/day)"),
            _ if avg <= 500.0 => (
                SubscriptionTier::Unlimited,
                "Go UNLIMITED - No daily limits",
            ),
            _ => (SubscriptionTier::Team, "Contact us for TEAM plan - 600/day"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_tier_limits() {
        assert_eq!(SubscriptionTier::Free.daily_limit(), 10);
        assert_eq!(SubscriptionTier::Basic.daily_limit(), 50);
        assert_eq!(SubscriptionTier::Premium.monthly_limit(), 4000);
        assert_eq!(SubscriptionTier::Unlimited.daily_limit(), u32::MAX);
    }

    #[test]
    fn test_trial_period() {
        let mut sub = SubscriptionInfo {
            user_id: "test".to_string(),
            email: "test@example.com".to_string(),
            tier: SubscriptionTier::Basic,
            daily_limit: 50,
            monthly_limit: 1000,
            expires_at: Utc::now() + Duration::days(30),
            trial_ends_at: Some(Utc::now() + Duration::days(5)),
            // Credits removed for security
            features: vec![],
            cached_at: Utc::now(),
        };

        assert!(sub.is_in_trial());
        assert_eq!(sub.effective_daily_limit(), u32::MAX);

        sub.trial_ends_at = Some(Utc::now() - Duration::days(1));
        assert!(!sub.is_in_trial());
        assert_eq!(sub.effective_daily_limit(), 50);
    }

    #[test]
    fn test_subscription_display() {
        let display = SubscriptionDisplay {
            username: "user@example.com".to_string(),
            tier: SubscriptionTier::Premium,
            daily_remaining: 180,
            daily_limit: 200,
            daily_used: 20,
            total_remaining: None,
            is_trial: false,
            average_daily_usage: Some(15.5),
            days_active: 10,
        };

        assert_eq!(
            display.format(),
            "user@example.com | PREMIUM | 180/200 daily | Avg: 15.5/day"
        );
    }

    #[test]
    fn test_trial_display() {
        let display = SubscriptionDisplay {
            username: "trial@example.com".to_string(),
            tier: SubscriptionTier::Free,
            daily_remaining: u32::MAX,
            daily_limit: u32::MAX,
            daily_used: 5,
            total_remaining: None,
            is_trial: true,
            average_daily_usage: Some(3.2),
            days_active: 3,
        };

        assert_eq!(
            display.format(),
            "trial@example.com | FREE (Trial) | Unlimited (5 used today) | Avg: 3.2/day"
        );
    }

    #[test]
    fn test_trial_display_with_d1() {
        let display = SubscriptionDisplay {
            username: "trial@example.com".to_string(),
            tier: SubscriptionTier::Free,
            daily_remaining: u32::MAX,
            daily_limit: u32::MAX,
            daily_used: 5,
            total_remaining: Some(995), // From D1
            is_trial: true,
            average_daily_usage: Some(4.0),
            days_active: 2,
        };

        assert_eq!(
            display.format(),
            "trial@example.com | FREE (Trial) | 995 remaining today | Avg: 4.0/day"
        );
    }

    #[test]
    fn test_trial_limit_reached() {
        let display = SubscriptionDisplay {
            username: "trial@example.com".to_string(),
            tier: SubscriptionTier::Free,
            daily_remaining: 0,
            daily_limit: 10,
            daily_used: 1000,
            total_remaining: Some(0), // From D1
            is_trial: true,
            average_daily_usage: Some(250.0),
            days_active: 4,
        };

        assert_eq!(
            display.format(),
            "trial@example.com | FREE (Trial) | Trial limit reached for today | Avg: 250.0/day"
        );
    }

    #[test]
    fn test_daily_limit_reached() {
        let display = SubscriptionDisplay {
            username: "user@example.com".to_string(),
            tier: SubscriptionTier::Basic,
            daily_remaining: 0,
            daily_limit: 50,
            daily_used: 50,
            total_remaining: Some(0), // From D1
            is_trial: false,
            average_daily_usage: Some(45.5),
            days_active: 30,
        };

        assert_eq!(
            display.format(),
            "user@example.com | BASIC | Daily limit reached (0/50 daily) | Avg: 45.5/day"
        );
    }

    #[test]
    fn test_credit_pack_usage() {
        let display = SubscriptionDisplay {
            username: "user@example.com".to_string(),
            tier: SubscriptionTier::Basic,
            daily_remaining: 0,
            daily_limit: 50,
            daily_used: 50,
            total_remaining: Some(25), // Using credit pack
            is_trial: false,
            average_daily_usage: Some(48.0),
            days_active: 15,
        };

        assert_eq!(
            display.format(),
            "user@example.com | BASIC | 25 credits remaining | Avg: 48.0/day"
        );
    }
}
