//! Temporal context for time-aware features
//!
//! Provides current time awareness and temporal intelligence

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use serde::{Deserialize, Serialize};

/// Temporal context manager
#[derive(Debug, Clone)]
pub struct TemporalContext {
    /// Current local time
    current_time: DateTime<Local>,
    /// Session start time
    session_start: DateTime<Local>,
    /// Time zone information
    timezone: String,
    /// Business hours configuration
    business_hours: BusinessHours,
}

/// Business hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHours {
    /// Start hour (24-hour format)
    pub start_hour: u32,
    /// End hour (24-hour format)
    pub end_hour: u32,
    /// Working days (0 = Sunday, 6 = Saturday)
    pub working_days: Vec<u32>,
    /// Time zone
    pub timezone: String,
}

/// Time-based context information
#[derive(Debug, Clone)]
pub struct TimeContext {
    pub is_business_hours: bool,
    pub time_of_day: TimeOfDay,
    pub day_of_week: String,
    pub formatted_time: String,
    pub formatted_date: String,
    pub relative_time: String,
}

/// Time of day classification
#[derive(Debug, Clone, PartialEq)]
pub enum TimeOfDay {
    EarlyMorning, // 5-8 AM
    Morning,      // 8-12 PM
    Afternoon,    // 12-17 PM
    Evening,      // 17-21 PM
    Night,        // 21-5 AM
}

impl TemporalContext {
    /// Create new temporal context
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            current_time: now,
            session_start: now,
            timezone: now.format("%z").to_string(),
            business_hours: BusinessHours::default(),
        }
    }

    /// Update current time
    pub fn update(&mut self) {
        self.current_time = Local::now();
    }

    /// Get current time formatted for display
    pub fn current_time_formatted(&self) -> String {
        self.current_time.format("%H:%M:%S").to_string()
    }

    /// Get current date formatted for display
    pub fn current_date_formatted(&self) -> String {
        self.current_time.format("%Y-%m-%d").to_string()
    }

    /// Get current datetime formatted for display
    pub fn current_datetime_formatted(&self) -> String {
        self.current_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get relative time since session start
    pub fn session_duration(&self) -> String {
        let duration = self.current_time.signed_duration_since(self.session_start);

        if duration.num_hours() > 0 {
            format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60)
        } else if duration.num_minutes() > 0 {
            format!(
                "{}m {}s",
                duration.num_minutes(),
                duration.num_seconds() % 60
            )
        } else {
            format!("{}s", duration.num_seconds())
        }
    }

    /// Get comprehensive time context
    pub fn get_context(&self) -> TimeContext {
        let hour = self.current_time.time().hour();
        let time_of_day = match hour {
            5..=7 => TimeOfDay::EarlyMorning,
            8..=11 => TimeOfDay::Morning,
            12..=16 => TimeOfDay::Afternoon,
            17..=20 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        };

        let day_of_week = self.current_time.format("%A").to_string();
        let weekday = self.current_time.weekday().num_days_from_sunday();

        let is_business_hours = self.business_hours.working_days.contains(&weekday)
            && hour >= self.business_hours.start_hour
            && hour < self.business_hours.end_hour;

        TimeContext {
            is_business_hours,
            time_of_day,
            day_of_week,
            formatted_time: self.current_time_formatted(),
            formatted_date: self.current_date_formatted(),
            relative_time: self.session_duration(),
        }
    }

    /// Get greeting based on time of day
    pub fn get_greeting(&self) -> &'static str {
        match self.get_context().time_of_day {
            TimeOfDay::EarlyMorning => "Good early morning",
            TimeOfDay::Morning => "Good morning",
            TimeOfDay::Afternoon => "Good afternoon",
            TimeOfDay::Evening => "Good evening",
            TimeOfDay::Night => "Good evening",
        }
    }

    /// Check if it's currently business hours
    pub fn is_business_hours(&self) -> bool {
        self.get_context().is_business_hours
    }

    /// Get time-appropriate suggestions
    pub fn get_time_suggestions(&self) -> Vec<String> {
        let context = self.get_context();
        let mut suggestions = Vec::new();

        match context.time_of_day {
            TimeOfDay::EarlyMorning => {
                suggestions.push("Start your day with a quick project analysis".to_string());
                suggestions.push("Review yesterday's progress".to_string());
            }
            TimeOfDay::Morning => {
                suggestions.push("Perfect time for deep work and coding".to_string());
                suggestions.push("Plan today's development tasks".to_string());
            }
            TimeOfDay::Afternoon => {
                suggestions.push("Good time for code reviews and collaboration".to_string());
                suggestions.push("Consider taking a break soon".to_string());
            }
            TimeOfDay::Evening => {
                suggestions.push("Wrap up current tasks".to_string());
                suggestions.push("Document today's progress".to_string());
            }
            TimeOfDay::Night => {
                suggestions.push("Consider getting some rest".to_string());
                suggestions.push("Light tasks only recommended".to_string());
            }
        }

        if !context.is_business_hours {
            suggestions.push("Outside business hours - personal projects?".to_string());
        }

        suggestions
    }

    /// Format duration for display
    pub fn format_duration(&self, seconds: u64) -> String {
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else {
            format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
        }
    }

    /// Get timestamp for logging
    pub fn timestamp(&self) -> String {
        self.current_time
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string()
    }

    /// Get ISO 8601 formatted timestamp
    pub fn iso_timestamp(&self) -> String {
        self.current_time.to_rfc3339()
    }

    /// Check if time is within a range
    pub fn is_time_in_range(&self, start_hour: u32, end_hour: u32) -> bool {
        let current_hour = self.current_time.time().hour();
        if start_hour <= end_hour {
            current_hour >= start_hour && current_hour < end_hour
        } else {
            // Range crosses midnight
            current_hour >= start_hour || current_hour < end_hour
        }
    }

    /// Get time until next hour
    pub fn time_until_next_hour(&self) -> chrono::Duration {
        let current = self.current_time;
        let next_hour = current
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
            + chrono::Duration::hours(1);

        next_hour.signed_duration_since(current)
    }

    /// Get "What's new" content based on time
    pub fn get_whats_new(&self) -> Vec<String> {
        let mut items = Vec::new();
        let context = self.get_context();

        // Add time-based items
        items.push(format!("Current time: {}", context.formatted_time));
        items.push(format!("Session duration: {}", context.relative_time));

        if context.is_business_hours {
            items.push("Business hours are active".to_string());
        } else {
            items.push("Outside business hours".to_string());
        }

        // Add day-specific items
        match context.day_of_week.as_str() {
            "Monday" => items.push("Start of the work week".to_string()),
            "Friday" => items.push("End of the work week approaching".to_string()),
            "Saturday" | "Sunday" => items.push("Weekend time".to_string()),
            _ => {}
        }

        items
    }

    /// Configure business hours
    pub fn set_business_hours(&mut self, business_hours: BusinessHours) {
        self.business_hours = business_hours;
    }

    /// Get current business hours
    pub fn business_hours(&self) -> &BusinessHours {
        &self.business_hours
    }

    /// Get current timezone
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    /// Get session start time
    pub fn session_start(&self) -> DateTime<Local> {
        self.session_start
    }

    /// Get current time
    pub fn current_time(&self) -> DateTime<Local> {
        self.current_time
    }
}

impl BusinessHours {
    /// Create standard business hours (9 AM - 5 PM, Mon-Fri)
    pub fn standard() -> Self {
        Self {
            start_hour: 9,
            end_hour: 17,
            working_days: vec![1, 2, 3, 4, 5], // Mon-Fri
            timezone: "Local".to_string(),
        }
    }

    /// Create extended business hours (8 AM - 6 PM, Mon-Fri)
    pub fn extended() -> Self {
        Self {
            start_hour: 8,
            end_hour: 18,
            working_days: vec![1, 2, 3, 4, 5], // Mon-Fri
            timezone: "Local".to_string(),
        }
    }

    /// Create 24/7 hours
    pub fn always() -> Self {
        Self {
            start_hour: 0,
            end_hour: 24,
            working_days: vec![0, 1, 2, 3, 4, 5, 6], // All days
            timezone: "Local".to_string(),
        }
    }
}

impl Default for BusinessHours {
    fn default() -> Self {
        Self::standard()
    }
}

impl Default for TemporalContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to get current time context
pub fn current_time_context() -> TimeContext {
    TemporalContext::new().get_context()
}

/// Helper function to get current timestamp
pub fn current_timestamp() -> String {
    TemporalContext::new().timestamp()
}

/// Helper function to format duration
pub fn format_duration(seconds: u64) -> String {
    TemporalContext::new().format_duration(seconds)
}
