//! Accessibility Features for HiveTechs Consensus TUI
//!
//! Provides enhanced accessibility support including screen reader compatibility,
//! high contrast modes, and reduced motion options.

use anyhow::Result;
use std::env;

/// Accessibility manager for TUI
pub struct AccessibilityManager {
    screen_reader_mode: bool,
    high_contrast_mode: bool,
    reduced_motion: bool,
    keyboard_only: bool,
    announce_changes: bool,
}

/// Accessibility announcements for screen readers
#[derive(Debug, Clone)]
pub struct AccessibilityAnnouncement {
    pub text: String,
    pub priority: AnnouncementPriority,
    pub region: AccessibilityRegion,
}

/// Priority levels for announcements
#[derive(Debug, Clone, PartialEq)]
pub enum AnnouncementPriority {
    Low,      // Informational updates
    Medium,   // Status changes
    High,     // Important notifications
    Critical, // Errors or urgent information
}

/// Regions of the interface for announcements
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityRegion {
    Input,
    Messages,
    Consensus,
    Status,
    Global,
}

impl AccessibilityManager {
    /// Create new accessibility manager with auto-detection
    pub fn new() -> Self {
        let screen_reader_mode = Self::detect_screen_reader();
        let high_contrast_mode = Self::detect_high_contrast_preference();
        let reduced_motion = Self::detect_reduced_motion_preference();
        let keyboard_only = Self::detect_keyboard_only_preference();

        Self {
            screen_reader_mode,
            high_contrast_mode,
            reduced_motion,
            keyboard_only,
            announce_changes: screen_reader_mode,
        }
    }

    /// Check if screen reader support is needed
    pub fn is_screen_reader_mode(&self) -> bool {
        self.screen_reader_mode
    }

    /// Check if high contrast mode is active
    pub fn is_high_contrast_mode(&self) -> bool {
        self.high_contrast_mode
    }

    /// Check if reduced motion is preferred
    pub fn is_reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    /// Check if keyboard-only navigation is preferred
    pub fn is_keyboard_only(&self) -> bool {
        self.keyboard_only
    }

    /// Enable screen reader mode
    pub fn enable_screen_reader_mode(&mut self) {
        self.screen_reader_mode = true;
        self.announce_changes = true;
        self.keyboard_only = true; // Screen readers typically use keyboard
    }

    /// Create accessibility announcement
    pub fn create_announcement(
        &self,
        text: &str,
        priority: AnnouncementPriority,
        region: AccessibilityRegion,
    ) -> Option<AccessibilityAnnouncement> {
        if !self.announce_changes {
            return None;
        }

        Some(AccessibilityAnnouncement {
            text: text.to_string(),
            priority,
            region,
        })
    }

    /// Get screen reader friendly text for consensus progress
    pub fn consensus_progress_description(
        &self,
        stage: &str,
        progress: u16,
        model: &str,
    ) -> String {
        if self.screen_reader_mode {
            format!(
                "Consensus stage {} at {}% progress using model {}",
                stage, progress, model
            )
        } else {
            format!("{} → {}% ({})", stage, progress, model)
        }
    }

    /// Get accessible command help
    pub fn get_accessible_help(&self) -> String {
        if self.screen_reader_mode {
            "Keyboard navigation help. Use arrow keys to navigate command history. \
             Use F1 through F4 to switch between interface panels. \
             Press question mark for detailed shortcuts. \
             Use Control plus H for quick ask command. \
             Use Control plus A for quick analyze command. \
             Use Control plus P for quick plan command. \
             Use Control plus T to cycle themes. \
             Press Escape to clear input or close dialogs."
        } else {
            "Use arrow keys for history, F1-F4 for panels, ? for help"
        }
        .to_string()
    }

    /// Get accessible status line
    pub fn get_accessible_status(&self, auto_accept: bool, context: u8, focus: &str) -> String {
        if self.screen_reader_mode {
            format!(
                "Status: Auto-accept edits is {}. Context remaining: {} percent. Currently focused on {}.",
                if auto_accept { "enabled" } else { "disabled" },
                context,
                focus
            )
        } else {
            format!(
                "Auto-accept: {} | Context: {}% | Focus: {}",
                if auto_accept { "on" } else { "off" },
                context,
                focus
            )
        }
    }

    /// Convert progress visualization to accessible text
    pub fn accessible_progress_bar(&self, progress: u16, total: u16) -> String {
        if self.screen_reader_mode {
            let percentage = (progress as f32 / total as f32 * 100.0) as u8;
            format!(
                "Progress: {} out of {} steps completed, {} percent",
                progress, total, percentage
            )
        } else {
            format!("{}/{} ({}%)", progress, total, progress * 100 / total)
        }
    }

    /// Get color description for screen readers
    pub fn describe_color(&self, color_name: &str) -> String {
        if self.screen_reader_mode {
            match color_name {
                "red" => "error indication".to_string(),
                "green" => "success indication".to_string(),
                "yellow" => "warning or in progress indication".to_string(),
                "blue" => "information indication".to_string(),
                "cyan" => "accent or link indication".to_string(),
                "gray" => "secondary or disabled indication".to_string(),
                _ => format!("{} color", color_name),
            }
        } else {
            color_name.to_string()
        }
    }

    /// Check for environment variable accessibility hints
    fn detect_screen_reader() -> bool {
        // Check for common screen reader environment variables
        env::var("NVDA_RUNNING").is_ok() ||
        env::var("JAWS_RUNNING").is_ok() ||
        env::var("SCREEN_READER").is_ok() ||
        env::var("ACCESSIBILITY_MODE").is_ok() ||
        // macOS VoiceOver
        env::var("VOICEOVER_RUNNING").is_ok() ||
        // Linux screen readers
        env::var("ORCA_RUNNING").is_ok() ||
        env::var("SPEECHD_RUNNING").is_ok()
    }

    fn detect_high_contrast_preference() -> bool {
        // Check for high contrast preference
        env::var("HIGH_CONTRAST").is_ok() || env::var("HIVE_HIGH_CONTRAST").is_ok()
    }

    fn detect_reduced_motion_preference() -> bool {
        // Check for reduced motion preference
        env::var("REDUCED_MOTION").is_ok() ||
        env::var("HIVE_REDUCED_MOTION").is_ok() ||
        // macOS prefers-reduced-motion equivalent
        env::var("PREFER_REDUCED_MOTION").is_ok()
    }

    fn detect_keyboard_only_preference() -> bool {
        // Check for keyboard-only navigation preference
        env::var("KEYBOARD_ONLY").is_ok()
            || env::var("HIVE_KEYBOARD_ONLY").is_ok()
            || Self::detect_screen_reader() // Screen readers typically use keyboard
    }
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen reader announcement helper
pub struct ScreenReaderAnnouncer {
    announcements: Vec<AccessibilityAnnouncement>,
    last_announcement: Option<String>,
}

impl ScreenReaderAnnouncer {
    pub fn new() -> Self {
        Self {
            announcements: Vec::new(),
            last_announcement: None,
        }
    }

    /// Add announcement to queue
    pub fn announce(&mut self, announcement: AccessibilityAnnouncement) {
        // Avoid duplicate announcements
        if let Some(ref last) = self.last_announcement {
            if last == &announcement.text {
                return;
            }
        }

        self.announcements.push(announcement.clone());
        self.last_announcement = Some(announcement.text);
    }

    /// Get pending announcements
    pub fn get_announcements(&mut self) -> Vec<AccessibilityAnnouncement> {
        let announcements = self.announcements.clone();
        self.announcements.clear();
        announcements
    }

    /// Clear all announcements
    pub fn clear(&mut self) {
        self.announcements.clear();
        self.last_announcement = None;
    }
}

impl Default for ScreenReaderAnnouncer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_manager_creation() {
        let manager = AccessibilityManager::new();
        // Should detect based on environment or defaults
        assert!(!manager.is_screen_reader_mode() || env::var("SCREEN_READER").is_ok());
    }

    #[test]
    fn test_announcements() {
        let manager = AccessibilityManager::new();

        let announcement = manager.create_announcement(
            "Test message",
            AnnouncementPriority::Medium,
            AccessibilityRegion::Messages,
        );

        // Should create announcement if screen reader mode is enabled
        assert_eq!(announcement.is_some(), manager.is_screen_reader_mode());
    }

    #[test]
    fn test_accessible_descriptions() {
        let mut manager = AccessibilityManager::new();
        manager.enable_screen_reader_mode();

        let progress_desc = manager.consensus_progress_description("Generator", 50, "claude-3");
        assert!(progress_desc.contains("Generator"));
        assert!(progress_desc.contains("50%"));
        assert!(progress_desc.contains("claude-3"));

        let help = manager.get_accessible_help();
        assert!(help.contains("arrow keys"));
        assert!(help.contains("F1"));
    }

    #[test]
    fn test_color_descriptions() {
        let mut manager = AccessibilityManager::new();
        manager.enable_screen_reader_mode();

        assert_eq!(manager.describe_color("red"), "error indication");
        assert_eq!(manager.describe_color("green"), "success indication");
        assert_eq!(
            manager.describe_color("yellow"),
            "warning or in progress indication"
        );
    }
}
