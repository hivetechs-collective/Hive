// Temporal Context Provider - Claude Code-Inspired Web Search Enhancement
// Provides intelligent date/time context for AI models to improve current information retrieval

use anyhow::Result;
use chrono::{DateTime, Datelike, Local, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Temporal context data injected into prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    /// Direct instruction for AI models about current date
    pub search_instruction: String,
    /// Temporal awareness context
    pub temporal_awareness: String,
    /// Current date in YYYY-MM-DD format
    pub current_date: String,
    /// Full datetime string
    pub current_datetime: String,
    /// Business context if applicable
    pub business_context: Option<BusinessContext>,
}

impl Default for TemporalContext {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            search_instruction: format!("Today's date is {}", now.format("%Y-%m-%d")),
            temporal_awareness: format!("Current time: {}", now.to_rfc3339()),
            current_date: now.format("%Y-%m-%d").to_string(),
            current_datetime: now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            business_context: None,
        }
    }
}

/// Business context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessContext {
    pub is_business_day: bool,
    pub is_market_hours: bool,
    pub quarter: String,
    pub fiscal_period: String,
}

/// Calendar for business days
pub struct BusinessCalendar;

impl BusinessCalendar {
    pub fn is_business_day(&self, date: &DateTime<Tz>) -> bool {
        let weekday = date.weekday();
        !matches!(weekday, chrono::Weekday::Sat | chrono::Weekday::Sun)
    }

    pub fn current_quarter(&self, date: &DateTime<Tz>) -> String {
        let month = date.month();
        let quarter = match month {
            1..=3 => "Q1",
            4..=6 => "Q2",
            7..=9 => "Q3",
            10..=12 => "Q4",
            _ => unreachable!(),
        };
        format!("{} {}", quarter, date.year())
    }
}

/// Calendar for market hours
pub struct MarketCalendar;

impl MarketCalendar {
    pub fn is_market_hours(&self, date: &DateTime<Tz>) -> bool {
        // NYSE hours: 9:30 AM - 4:00 PM ET
        if !BusinessCalendar.is_business_day(date) {
            return false;
        }

        let hour = date.time().hour();
        let minute = date.minute();
        let time_minutes = hour * 60 + minute;

        // 9:30 AM = 570 minutes, 4:00 PM = 960 minutes
        time_minutes >= 570 && time_minutes <= 960
    }
}

/// Temporal context provider
pub struct TemporalContextProvider {
    timezone: Tz,
    business_calendar: Arc<BusinessCalendar>,
    market_calendar: Arc<MarketCalendar>,
}

impl Default for TemporalContextProvider {
    fn default() -> Self {
        Self::new(chrono_tz::US::Eastern)
    }
}

impl TemporalContextProvider {
    pub fn new(timezone: Tz) -> Self {
        Self {
            timezone,
            business_calendar: Arc::new(BusinessCalendar),
            market_calendar: Arc::new(MarketCalendar),
        }
    }

    /// Build current temporal context
    pub async fn build_current_context(&self) -> Result<TemporalContext> {
        let now = Utc::now().with_timezone(&self.timezone);

        // Build search instruction for AI models
        let search_instruction = format!(
            "IMPORTANT: Today's date is {} ({}). When performing web searches or looking up current information, prioritize results from {} onwards and clearly indicate if information might be outdated.",
            now.format("%A, %B %d, %Y"),
            now.format("%Y-%m-%d"),
            now.year()
        );

        // Build temporal awareness context
        let temporal_awareness = format!(
            "Current context: {} at {} ({}). Use this when interpreting 'recent', 'latest', 'current', 'today', or any time-sensitive queries.",
            now.format("%A, %B %d, %Y"),
            now.format("%H:%M %Z"),
            now.to_rfc3339()
        );

        // Build business context
        let business_context = self.build_business_context(&now).await?;

        Ok(TemporalContext {
            search_instruction,
            temporal_awareness,
            current_date: now.format("%Y-%m-%d").to_string(),
            current_datetime: now.format("%A, %B %d, %Y at %H:%M:%S %Z").to_string(),
            business_context,
        })
    }

    /// Build business context
    async fn build_business_context(&self, now: &DateTime<Tz>) -> Result<Option<BusinessContext>> {
        Ok(Some(BusinessContext {
            is_business_day: self.business_calendar.is_business_day(now),
            is_market_hours: self.market_calendar.is_market_hours(now),
            quarter: self.business_calendar.current_quarter(now),
            fiscal_period: format!("FY{}", now.year()),
        }))
    }

    /// Check if query requires temporal context
    pub fn requires_temporal_context(&self, query: &str) -> bool {
        let temporal_indicators = [
            // Direct temporal requests
            "latest",
            "recent",
            "current",
            "today",
            "now",
            "this week",
            "this month",
            "this year",
            "2024",
            "2025",
            "what's new",
            "recent updates",
            "current version",
            // Web search indicators
            "search",
            "lookup",
            "find online",
            "google",
            "web search",
            "internet",
            "news",
            "trends",
            "happening",
            "breaking",
            "announcement",
            // Version/release indicators
            "latest release",
            "new features",
            "just released",
            "recently published",
            "cutting edge",
            "state of the art",
            "bleeding edge",
            // Market/business indicators
            "stock price",
            "market",
            "earnings",
            "financial",
            "trading",
            "cryptocurrency",
            "bitcoin",
            "forex",
        ];

        let query_lower = query.to_lowercase();
        temporal_indicators
            .iter()
            .any(|&indicator| query_lower.contains(indicator))
    }

    /// Inject temporal context into prompt
    pub fn inject_context(&self, prompt: &str, context: &TemporalContext) -> String {
        format!(
            "{}\n\n{}\n\nUser Query: {}",
            context.search_instruction, context.temporal_awareness, prompt
        )
    }

    /// Get contextual time description
    pub fn get_time_context(&self) -> String {
        let now = Local::now();
        let hour = now.time().hour();

        let time_of_day = match hour {
            5..=11 => "morning",
            12..=16 => "afternoon",
            17..=20 => "evening",
            _ => "night",
        };

        format!("It is currently {} {}", time_of_day, now.format("%Z"))
    }

    /// Format for different time contexts
    pub fn format_for_context(&self, context_type: &str) -> String {
        let now = Local::now();

        match context_type {
            "business" => now.format("%A, %B %d, %Y at %I:%M %p %Z").to_string(),
            "technical" => now.to_rfc3339(),
            "casual" => now.format("%B %d, %Y").to_string(),
            "precise" => now.format("%Y-%m-%d %H:%M:%S%.3f %Z").to_string(),
            _ => now.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_detection() {
        let provider = TemporalContextProvider::default();

        // Should detect temporal context
        assert!(provider.requires_temporal_context("What are the latest Rust features?"));
        assert!(provider.requires_temporal_context("Search for recent AI developments"));
        assert!(provider.requires_temporal_context("What's the current stock price of AAPL?"));
        assert!(provider.requires_temporal_context("Find news about OpenAI today"));

        // Should not detect temporal context
        assert!(!provider.requires_temporal_context("How do I write a for loop in Rust?"));
        assert!(!provider.requires_temporal_context("Explain the concept of ownership"));
    }

    #[tokio::test]
    async fn test_context_building() {
        let provider = TemporalContextProvider::default();
        let context = provider.build_current_context().await.unwrap();

        assert!(!context.search_instruction.is_empty());
        assert!(!context.temporal_awareness.is_empty());
        assert!(!context.current_date.is_empty());
        assert!(!context.current_datetime.is_empty());
        assert!(context.business_context.is_some());
    }
}
