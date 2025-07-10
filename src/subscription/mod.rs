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
pub mod usage_tracker;
pub mod reminder;

pub use conversation_gateway::{ConversationGateway, ConversationAuthorization};
pub use usage_tracker::{UsageTracker, UsageNotification};
pub use reminder::SubscriptionReminder;

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

/// User subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub user_id: String,
    pub email: String,
    pub tier: SubscriptionTier,
    pub daily_limit: u32,
    pub monthly_limit: u32,
    pub expires_at: DateTime<Utc>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub credits_remaining: u32,
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

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub daily_used: u32,
    pub monthly_used: u32,
    pub credits_remaining: u32,
    pub last_conversation: Option<DateTime<Utc>>,
}

/// Credit pack options
#[derive(Debug, Clone, Copy)]
pub enum CreditPack {
    Starter,  // 25 credits for $3
    Value,    // 75 credits for $7
    Power,    // 200 credits for $15
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
    Upgraded { from: SubscriptionTier, to: SubscriptionTier },
    Downgraded { from: SubscriptionTier, to: SubscriptionTier },
    Cancelled,
}

/// Subscription display information for UI
pub struct SubscriptionDisplay {
    pub username: String,
    pub tier: SubscriptionTier,
    pub daily_remaining: u32,
    pub daily_limit: u32,
    pub credits_remaining: u32,
    pub is_trial: bool,
}

impl SubscriptionDisplay {
    pub fn format(&self) -> String {
        let tier_display = if self.is_trial {
            format!("{} (Trial)", self.tier)
        } else {
            self.tier.to_string()
        };

        if self.tier == SubscriptionTier::Unlimited {
            format!("{} | {} | Unlimited", self.username, tier_display)
        } else if self.daily_remaining == 0 && self.credits_remaining > 0 {
            format!("{} | {} | Using credits ({} left)", 
                self.username, tier_display, self.credits_remaining)
        } else {
            format!("{} | {} | {}/{} daily", 
                self.username, tier_display, self.daily_remaining, self.daily_limit)
        }
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
            credits_remaining: 0,
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
            credits_remaining: 0,
            is_trial: false,
        };

        assert_eq!(display.format(), "user@example.com | PREMIUM | 180/200 daily");
    }
}