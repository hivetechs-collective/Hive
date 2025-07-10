//! Usage Tracking and Limit Management
//! Monitors conversation usage and prompts users when approaching limits

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Datelike};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{SubscriptionInfo, SubscriptionTier, UsageStatistics, CreditPack};
use crate::core::license::{LicenseManager, LicenseStatus};

/// Usage notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageNotification {
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub action: Option<NotificationAction>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    Info,
    Warning,
    Critical,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub label: String,
    pub url: String,
}

/// Usage tracking and limit management
pub struct UsageTracker {
    config_dir: PathBuf,
    subscription: Option<SubscriptionInfo>,
    usage: Option<UsageStatistics>,
}

impl UsageTracker {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            subscription: None,
            usage: None,
        }
    }

    /// Load subscription and usage data
    pub async fn load_data(&mut self) -> Result<()> {
        // Load subscription from cache
        let sub_cache_file = self.config_dir.join("subscription_cache.json");
        if sub_cache_file.exists() {
            let content = tokio::fs::read_to_string(&sub_cache_file).await?;
            self.subscription = serde_json::from_str(&content).ok();
        }

        // Load usage data
        let usage_file = self.config_dir.join("usage_stats.json");
        if usage_file.exists() {
            let content = tokio::fs::read_to_string(&usage_file).await?;
            self.usage = serde_json::from_str(&content).ok();
        }

        Ok(())
    }

    /// Save usage data
    pub async fn save_usage(&self) -> Result<()> {
        if let Some(usage) = &self.usage {
            let usage_file = self.config_dir.join("usage_stats.json");
            let content = serde_json::to_string_pretty(usage)?;
            tokio::fs::write(&usage_file, content).await?;
        }
        Ok(())
    }

    /// Check usage before allowing a conversation
    /// Returns notification if user should be warned about limits
    pub async fn check_usage_before_conversation(&mut self) -> Result<(bool, Option<UsageNotification>)> {
        // Ensure data is loaded
        self.load_data().await?;

        let subscription = match &self.subscription {
            Some(sub) if sub.is_cache_valid() => sub,
            _ => {
                // Try to get subscription from license manager
                let license_manager = LicenseManager::new(self.config_dir.clone());
                let status = license_manager.check_license_status().await?;
                
                if !status.is_valid {
                    return Ok((false, Some(UsageNotification {
                        notification_type: NotificationType::Blocked,
                        title: "No Valid License".to_string(),
                        message: "Please configure your Hive license key to use consensus features.".to_string(),
                        action: Some(NotificationAction {
                            label: "Configure License".to_string(),
                            url: "https://hivetechs.io/pricing".to_string(),
                        }),
                    })));
                }

                // Create basic subscription info from license
                let sub = SubscriptionInfo {
                    user_id: status.user_id.unwrap_or_default(),
                    email: String::new(), // Will be filled from gateway
                    tier: match status.tier {
                        crate::core::license::LicenseTier::Free => SubscriptionTier::Free,
                        crate::core::license::LicenseTier::Basic => SubscriptionTier::Basic,
                        crate::core::license::LicenseTier::Standard => SubscriptionTier::Standard,
                        crate::core::license::LicenseTier::Premium => SubscriptionTier::Premium,
                        crate::core::license::LicenseTier::Unlimited => SubscriptionTier::Unlimited,
                        crate::core::license::LicenseTier::Enterprise => SubscriptionTier::Team,
                    },
                    daily_limit: status.tier.daily_limit(),
                    monthly_limit: status.tier.monthly_limit(),
                    expires_at: Utc::now() + chrono::Duration::days(30), // Default
                    trial_ends_at: None,
                    credits_remaining: 0,
                    features: vec![],
                    cached_at: Utc::now(),
                };
                
                self.subscription = Some(sub.clone());
                &self.subscription.as_ref().unwrap()
            }
        };

        // Get or create usage statistics
        if self.usage.is_none() {
            self.usage = Some(UsageStatistics {
                daily_used: 0,
                monthly_used: 0,
                credits_remaining: subscription.credits_remaining,
                last_conversation: None,
            });
        }
        
        let usage = self.usage.as_ref().unwrap();
        
        // Calculate usage percentages
        let daily_percentage = (usage.daily_used as f64 / subscription.daily_limit as f64) * 100.0;
        let monthly_percentage = (usage.monthly_used as f64 / subscription.monthly_limit as f64) * 100.0;
        
        // Clone values we'll need
        let subscription_tier = subscription.tier.clone();
        let credits_remaining = usage.credits_remaining;
        let daily_used = usage.daily_used;
        let monthly_used = usage.monthly_used;

        // Check if daily limit is exceeded
        if daily_used >= subscription.daily_limit {
            // Check if user has credit packs
            if credits_remaining > 0 {
                Ok((true, Some(UsageNotification {
                    notification_type: NotificationType::Info,
                    title: "💳 Using Credit Pack".to_string(),
                    message: format!(
                        "Daily allowance exhausted. Using 1 of your {} purchased credits for this conversation.",
                        credits_remaining
                    ),
                    action: Some(NotificationAction {
                        label: "Buy More Credits".to_string(),
                        url: "https://hivetechs.io/pricing".to_string(),
                    }),
                })))
            } else {
                let message = Self::generate_limit_reached_message(subscription, usage);
                let url = Self::get_upgrade_url(&subscription_tier);
                Ok((false, Some(UsageNotification {
                    notification_type: NotificationType::Blocked,
                    title: "🚫 Usage Limit Reached".to_string(),
                    message,
                    action: Some(NotificationAction {
                        label: "Upgrade Now".to_string(),
                        url,
                    }),
                })))
            }
        } else {
            // Check for warning thresholds
            let notification = self.generate_usage_warning(subscription, usage, daily_percentage, monthly_percentage);
            Ok((true, notification))
        }
    }

    /// Record successful conversation usage
    pub async fn record_conversation_usage(&mut self) -> Result<()> {
        self.load_data().await?;
        
        let usage = self.usage.get_or_insert(UsageStatistics {
            daily_used: 0,
            monthly_used: 0,
            credits_remaining: 0,
            last_conversation: None,
        });

        // Check if we need to use credit packs
        if let Some(subscription) = &self.subscription {
            if usage.daily_used >= subscription.daily_limit {
                // Daily allowance exhausted, use credit pack if available
                if usage.credits_remaining > 0 {
                    usage.credits_remaining -= 1;
                    tracing::info!("Using credit pack. {} credits remaining", usage.credits_remaining);
                }
                // Still increment monthly count
                usage.monthly_used += 1;
            } else {
                // Use daily allowance
                usage.daily_used += 1;
                usage.monthly_used += 1;
            }
        } else {
            usage.daily_used += 1;
            usage.monthly_used += 1;
        }

        usage.last_conversation = Some(Utc::now());
        self.save_usage().await?;

        tracing::info!("Conversation usage recorded");
        Ok(())
    }

    /// Generate appropriate usage warning based on current usage
    fn generate_usage_warning(
        &self,
        subscription: &SubscriptionInfo,
        usage: &UsageStatistics,
        daily_percentage: f64,
        monthly_percentage: f64,
    ) -> Option<UsageNotification> {
        // Critical warning (90%+ usage)
        if daily_percentage >= 90.0 || monthly_percentage >= 90.0 {
            Some(UsageNotification {
                notification_type: NotificationType::Critical,
                title: "🚨 Approaching Usage Limit".to_string(),
                message: self.generate_critical_warning_message(subscription, usage, daily_percentage, monthly_percentage),
                action: Some(NotificationAction {
                    label: "Upgrade Now".to_string(),
                    url: Self::get_upgrade_url(&subscription.tier),
                }),
            })
        }
        // High usage warning (75%+ usage)
        else if daily_percentage >= 75.0 || monthly_percentage >= 75.0 {
            Some(UsageNotification {
                notification_type: NotificationType::Warning,
                title: "⚠️ High Usage Alert".to_string(),
                message: self.generate_high_usage_message(subscription, usage, daily_percentage, monthly_percentage),
                action: Some(NotificationAction {
                    label: "View Plans".to_string(),
                    url: "https://hivetechs.io/pricing".to_string(),
                }),
            })
        }
        // Moderate usage (50%+ usage) - only for free tier
        else if subscription.tier == SubscriptionTier::Free && (daily_percentage >= 50.0 || monthly_percentage >= 50.0) {
            Some(UsageNotification {
                notification_type: NotificationType::Info,
                title: "📊 Usage Update".to_string(),
                message: self.generate_moderate_usage_message(subscription, usage, daily_percentage, monthly_percentage),
                action: Some(NotificationAction {
                    label: "Start Free Trial".to_string(),
                    url: "https://hivetechs.io/pricing".to_string(),
                }),
            })
        } else {
            None
        }
    }

    fn generate_limit_reached_message(subscription: &SubscriptionInfo, usage: &UsageStatistics) -> String {
        let mut message = format!("You've reached your daily allowance on the {} tier:\n\n", subscription.tier);
        message.push_str(&format!("**Daily Allowance**: {}/{} conversations used today\n", usage.daily_used, subscription.daily_limit));
        message.push_str(&format!("**Monthly Usage**: {}/{} conversations this month\n\n", usage.monthly_used, subscription.monthly_limit));

        if subscription.tier == SubscriptionTier::Free {
            message.push_str("**🚀 Start Your 7-Day FREE Trial**\n");
            message.push_str("Get unlimited access to all premium features:\n");
            message.push_str("• Multi-model consensus pipeline\n");
            message.push_str("• Advanced analytics and cost tracking\n");
            message.push_str("• Performance benchmarking\n");
            message.push_str("• Priority support\n\n");
            message.push_str("After your trial, choose a plan that fits your needs:\n");
            message.push_str("• **Basic ($5/month)**: 50 daily, 1,000 monthly conversations\n");
            message.push_str("• **Standard ($10/month)**: 100 daily, 2,000 monthly conversations\n");
            message.push_str("• **Premium ($20/month)**: 200 daily, 4,000 monthly conversations\n");
        } else {
            message.push_str("**Upgrade Options:**\n");
            
            if let Some(next_tier) = Self::get_next_tier(&subscription.tier) {
                message.push_str(&format!(
                    "• **{} ({})**: {} daily, {} monthly conversations\n",
                    next_tier, next_tier.price(), next_tier.daily_limit(), next_tier.monthly_limit()
                ));
            }
            
            message.push_str("\n**Or purchase additional credits:**\n");
            message.push_str("• **Starter Pack (25 credits)**: $3\n");
            message.push_str("• **Value Pack (75 credits)**: $7\n");
            message.push_str("• **Power Pack (200 credits)**: $15\n");
        }

        message.push_str("\nLimits reset tomorrow (daily) and next month (monthly).");
        message
    }

    fn generate_critical_warning_message(
        &self,
        subscription: &SubscriptionInfo,
        usage: &UsageStatistics,
        daily_percentage: f64,
        monthly_percentage: f64,
    ) -> String {
        let highest_percentage = daily_percentage.max(monthly_percentage);
        let is_daily = daily_percentage >= monthly_percentage;
        
        let mut message = format!(
            "You're at {:.0}% of your daily allowance.\n\n",
            daily_percentage
        );
        message.push_str("**Current Usage:**\n");
        message.push_str(&format!("• Today: {}/{} conversations ({:.0}%)\n", usage.daily_used, subscription.daily_limit, daily_percentage));
        message.push_str(&format!("• This Month: {}/{} conversations ({:.0}%)\n\n", usage.monthly_used, subscription.monthly_limit, monthly_percentage));
        
        if usage.credits_remaining > 0 {
            message.push_str(&format!("**Good news!** You have {} credit pack conversations available when you reach your daily limit.\n\n", usage.credits_remaining));
        }
        
        message.push_str("**Avoid interruptions** by upgrading now or purchasing credits.");
        message
    }

    fn generate_high_usage_message(
        &self,
        subscription: &SubscriptionInfo,
        usage: &UsageStatistics,
        daily_percentage: f64,
        monthly_percentage: f64,
    ) -> String {
        let mut message = format!("You're using your {} subscription heavily this period.\n\n", subscription.tier);
        message.push_str("**Current Usage:**\n");
        message.push_str(&format!("• Daily: {}/{} ({:.0}%)\n", usage.daily_used, subscription.daily_limit, daily_percentage));
        message.push_str(&format!("• Monthly: {}/{} ({:.0}%)\n\n", usage.monthly_used, subscription.monthly_limit, monthly_percentage));
        
        if subscription.tier == SubscriptionTier::Free {
            message.push_str("**🚀 Ready for more?** Start your 7-day unlimited trial!");
        } else {
            message.push_str("**Consider upgrading** if you frequently reach these usage levels.");
        }
        
        message
    }

    fn generate_moderate_usage_message(
        &self,
        subscription: &SubscriptionInfo,
        usage: &UsageStatistics,
        daily_percentage: f64,
        monthly_percentage: f64,
    ) -> String {
        let mut message = "You're getting good use out of hive-tools!\n\n".to_string();
        message.push_str(&format!("**Today's Usage**: {}/{} conversations ({:.0}%)\n", 
            usage.daily_used, subscription.daily_limit, daily_percentage));
        message.push_str(&format!("**This Month**: {}/{} conversations ({:.0}%)\n\n", 
            usage.monthly_used, subscription.monthly_limit, monthly_percentage));
        message.push_str("**💡 Tip**: Start your 7-day free trial to unlock unlimited conversations and premium features!");
        
        message
    }

    fn get_upgrade_url(current_tier: &SubscriptionTier) -> String {
        // All tiers go to the same pricing page
        "https://hivetechs.io/pricing".to_string()
    }

    fn get_next_tier(current_tier: &SubscriptionTier) -> Option<SubscriptionTier> {
        match current_tier {
            SubscriptionTier::Free => Some(SubscriptionTier::Basic),
            SubscriptionTier::Basic => Some(SubscriptionTier::Standard),
            SubscriptionTier::Standard => Some(SubscriptionTier::Premium),
            SubscriptionTier::Premium => Some(SubscriptionTier::Unlimited),
            SubscriptionTier::Unlimited => Some(SubscriptionTier::Team),
            SubscriptionTier::Team => None,
        }
    }

    /// Get formatted usage display for user interface
    pub async fn get_usage_display(&mut self) -> Result<String> {
        self.load_data().await?;

        let subscription = match &self.subscription {
            Some(sub) => sub,
            None => return Ok("📊 Usage information unavailable - please configure your license".to_string()),
        };

        let usage = self.usage.as_ref().unwrap_or(&UsageStatistics {
            daily_used: 0,
            monthly_used: 0,
            credits_remaining: 0,
            last_conversation: None,
        });

        let daily_percentage = ((usage.daily_used as f64 / subscription.daily_limit as f64) * 100.0).round() as u32;
        let monthly_percentage = ((usage.monthly_used as f64 / subscription.monthly_limit as f64) * 100.0).round() as u32;

        let mut display = format!("📊 **Usage Overview ({} TIER)**\n\n", subscription.tier);
        
        // Progress bars with visual indicators
        let daily_bar = self.generate_progress_bar(daily_percentage);
        let monthly_bar = self.generate_progress_bar(monthly_percentage);
        
        display.push_str(&format!("**Today**: {}/{} conversations\n", usage.daily_used, subscription.daily_limit));
        display.push_str(&format!("{} {}%\n\n", daily_bar, daily_percentage));
        
        display.push_str(&format!("**This Month**: {}/{} conversations\n", usage.monthly_used, subscription.monthly_limit));
        display.push_str(&format!("{} {}%\n\n", monthly_bar, monthly_percentage));
        
        if usage.credits_remaining > 0 {
            display.push_str(&format!("💳 **Credit Packs**: {} conversations available\n\n", usage.credits_remaining));
        }
        
        // Status indicator
        let max_percentage = daily_percentage.max(monthly_percentage);
        if max_percentage >= 90 {
            display.push_str("🚨 **Status**: Approaching limits - consider upgrading soon\n");
        } else if max_percentage >= 75 {
            display.push_str("⚠️ **Status**: High usage - upgrade recommended\n");
        } else if max_percentage >= 50 {
            display.push_str("📈 **Status**: Moderate usage - you're on track\n");
        } else {
            display.push_str("✅ **Status**: Plenty of conversations remaining\n");
        }
        
        // Reset information
        let now = Utc::now();
        let tomorrow = now + chrono::Duration::days(1);
        let next_month = if now.month() == 12 {
            now.with_month(1).unwrap().with_year(now.year() + 1).unwrap()
        } else {
            now.with_month(now.month() + 1).unwrap()
        };
        
        display.push_str(&format!(
            "\n**Resets**: Daily ({}) | Monthly ({})",
            tomorrow.format("%Y-%m-%d"),
            next_month.format("%Y-%m-%d")
        ));
        
        Ok(display)
    }

    /// Generate a visual progress bar for usage percentages
    fn generate_progress_bar(&self, percentage: u32) -> String {
        let width: usize = 20;
        let filled = ((percentage as f64 / 100.0) * width as f64).round() as usize;
        let empty = width.saturating_sub(filled);
        
        // Use different characters based on usage level
        let (fill_char, empty_char) = if percentage >= 90 {
            ('🔴', '⚫')
        } else if percentage >= 75 {
            ('🟠', '⚫')
        } else if percentage >= 50 {
            ('🟡', '⚫')
        } else {
            ('🟢', '⚫')
        };
        
        format!("[{}{}]", 
            fill_char.to_string().repeat(filled.min(width) / 4),
            empty_char.to_string().repeat(empty.min(width) / 4)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_usage_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut tracker = UsageTracker::new(temp_dir.path().to_path_buf());

        // Set up test subscription
        tracker.subscription = Some(SubscriptionInfo {
            user_id: "test".to_string(),
            email: "test@example.com".to_string(),
            tier: SubscriptionTier::Basic,
            daily_limit: 50,
            monthly_limit: 1000,
            expires_at: Utc::now() + chrono::Duration::days(30),
            trial_ends_at: None,
            credits_remaining: 5,
            features: vec![],
            cached_at: Utc::now(),
        });

        // Test no usage
        let (allowed, notification) = tracker.check_usage_before_conversation().await.unwrap();
        assert!(allowed);
        assert!(notification.is_none());

        // Test recording usage
        tracker.record_conversation_usage().await.unwrap();
        assert_eq!(tracker.usage.as_ref().unwrap().daily_used, 1);
    }

    #[test]
    fn test_progress_bar_generation() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = UsageTracker::new(temp_dir.path().to_path_buf());

        let bar = tracker.generate_progress_bar(0);
        assert!(bar.contains('🟢'));

        let bar = tracker.generate_progress_bar(75);
        assert!(bar.contains('🟠'));

        let bar = tracker.generate_progress_bar(95);
        assert!(bar.contains('🔴'));
    }
}