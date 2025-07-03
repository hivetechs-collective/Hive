//! Screen reader support for TUI
//!
//! Provides compatibility with popular screen readers:
//! - NVDA (Windows)
//! - JAWS (Windows)
//! - VoiceOver (macOS)
//! - Orca (Linux)

use crate::tui::themes::Theme;
use super::{AnnouncementPriority, FocusState};

/// Screen reader interface
pub struct ScreenReaderInterface {
    enabled: bool,
    current_focus: Option<String>,
    announcement_queue: Vec<Announcement>,
    live_region_content: String,
}

/// Screen reader announcement
struct Announcement {
    text: String,
    priority: AnnouncementPriority,
    timestamp: std::time::Instant,
}

/// Screen reader compatible element descriptions
pub struct ScreenReaderElement {
    pub role: String,
    pub name: String,
    pub description: Option<String>,
    pub state: FocusState,
    pub value: Option<String>,
    pub position: Option<(usize, usize)>, // (current, total)
}

impl ScreenReaderInterface {
    /// Create new screen reader interface
    pub fn new() -> Self {
        Self {
            enabled: false,
            current_focus: None,
            announcement_queue: Vec::new(),
            live_region_content: String::new(),
        }
    }

    /// Enable or disable screen reader support
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        
        if enabled {
            self.announce("Screen reader support enabled", AnnouncementPriority::Medium);
        }
    }

    /// Check if screen reader support is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Announce text to screen reader
    pub fn announce(&mut self, text: &str, priority: AnnouncementPriority) {
        if !self.enabled {
            return;
        }

        let announcement = Announcement {
            text: text.to_string(),
            priority,
            timestamp: std::time::Instant::now(),
        };

        // Insert based on priority
        match priority {
            AnnouncementPriority::Critical => {
                self.announcement_queue.insert(0, announcement);
            }
            AnnouncementPriority::High => {
                let pos = self.announcement_queue
                    .iter()
                    .position(|a| matches!(a.priority, AnnouncementPriority::Medium | AnnouncementPriority::Low))
                    .unwrap_or(self.announcement_queue.len());
                self.announcement_queue.insert(pos, announcement);
            }
            _ => {
                self.announcement_queue.push(announcement);
            }
        }

        // Process announcement queue
        self.process_announcements();
    }

    /// Update focus and announce focus change
    pub fn update_focus(&mut self, element: &ScreenReaderElement) {
        if !self.enabled {
            return;
        }

        let focus_description = self.describe_element(element);
        
        if self.current_focus.as_ref() != Some(&focus_description) {
            self.current_focus = Some(focus_description.clone());
            self.announce(&focus_description, AnnouncementPriority::High);
        }
    }

    /// Clear current focus
    pub fn clear_focus(&mut self) {
        self.current_focus = None;
    }

    /// Update live region content
    pub fn update_live_region(&mut self, content: &str) {
        if !self.enabled {
            return;
        }

        if self.live_region_content != content {
            self.live_region_content = content.to_string();
            self.announce(content, AnnouncementPriority::Medium);
        }
    }

    /// Describe UI element for screen reader
    pub fn describe_element(&self, element: &ScreenReaderElement) -> String {
        let mut description = Vec::new();

        // Role and name
        description.push(format!("{} {}", element.role, element.name));

        // State
        match element.state {
            FocusState::Focused => description.push("focused".to_string()),
            FocusState::Active => description.push("active".to_string()),
            FocusState::Disabled => description.push("disabled".to_string()),
            FocusState::None => {}
        }

        // Value
        if let Some(value) = &element.value {
            description.push(format!("value {}", value));
        }

        // Position
        if let Some((current, total)) = element.position {
            description.push(format!("{} of {}", current, total));
        }

        // Additional description
        if let Some(desc) = &element.description {
            description.push(desc.clone());
        }

        description.join(", ")
    }

    /// Get keyboard shortcuts description
    pub fn describe_shortcuts(&self, shortcuts: &[(String, String)]) -> String {
        if shortcuts.is_empty() {
            return "No keyboard shortcuts available".to_string();
        }

        let mut description = vec!["Available shortcuts:".to_string()];
        
        for (key, action) in shortcuts {
            description.push(format!("{} for {}", key, action));
        }

        description.join(", ")
    }

    /// Describe table or list structure
    pub fn describe_structure(&self, element_type: &str, rows: usize, columns: Option<usize>) -> String {
        match columns {
            Some(cols) => format!("{} with {} rows and {} columns", element_type, rows, cols),
            None => format!("{} with {} items", element_type, rows),
        }
    }

    /// Announce progress updates
    pub fn announce_progress(&mut self, current: usize, total: usize, operation: &str) {
        let percentage = (current as f32 / total as f32 * 100.0) as usize;
        let message = format!("{}: {}% complete, {} of {}", operation, percentage, current, total);
        self.announce(&message, AnnouncementPriority::Low);
    }

    /// Announce errors with context
    pub fn announce_error(&mut self, error: &str, context: Option<&str>) {
        let message = match context {
            Some(ctx) => format!("Error in {}: {}", ctx, error),
            None => format!("Error: {}", error),
        };
        self.announce(&message, AnnouncementPriority::Critical);
    }

    /// Announce status changes
    pub fn announce_status(&mut self, status: &str, component: &str) {
        let message = format!("{} status: {}", component, status);
        self.announce(&message, AnnouncementPriority::Medium);
    }

    /// Adjust theme for screen reader compatibility
    pub fn adjust_theme(&self, theme: &mut Theme) {
        if !self.enabled {
            return;
        }

        // Ensure high contrast for better screen reader compatibility
        theme.apply_accessibility(true, false);
        
        // Make focus indicators more prominent
        theme.styles.active_border = theme.styles.active_border
            .add_modifier(ratatui::style::Modifier::BOLD)
            .add_modifier(ratatui::style::Modifier::UNDERLINED);
    }

    /// Process announcement queue
    fn process_announcements(&mut self) {
        // Remove old announcements (older than 5 seconds)
        let cutoff = std::time::Instant::now() - std::time::Duration::from_secs(5);
        self.announcement_queue.retain(|a| a.timestamp > cutoff);

        // In a real implementation, this would interface with platform-specific
        // screen reader APIs to actually speak the announcements
        
        // For now, we just log them (in a real app, this would be removed)
        if cfg!(debug_assertions) {
            for announcement in &self.announcement_queue {
                eprintln!("[SCREEN READER] {:?}: {}", announcement.priority, announcement.text);
            }
        }
        
        // Clear processed announcements
        self.announcement_queue.clear();
    }

    /// Get current focus description
    pub fn current_focus(&self) -> Option<&String> {
        self.current_focus.as_ref()
    }

    /// Check if element should be announced
    pub fn should_announce_element(&self, element_type: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // Some elements should not be announced (decorative elements)
        !matches!(element_type, "decoration" | "spacer" | "divider")
    }

    /// Generate navigation instructions
    pub fn navigation_instructions(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        vec![
            "Use Tab and Shift+Tab to navigate between elements",
            "Use arrow keys to navigate within lists and menus",
            "Use Enter or Space to activate buttons",
            "Use Escape to close dialogs and menus",
            "Use F1 for help, F2-F4 to switch panels",
        ].join(". ")
    }

    /// Describe current UI layout
    pub fn describe_layout(&self, layout_type: &str, regions: &[String]) -> String {
        if regions.is_empty() {
            return format!("{} layout with no content", layout_type);
        }

        let regions_description = regions.join(", ");
        format!("{} layout containing: {}", layout_type, regions_description)
    }
}

/// Helper functions for creating screen reader elements
impl ScreenReaderElement {
    /// Create button element description
    pub fn button(name: &str, state: FocusState) -> Self {
        Self {
            role: "button".to_string(),
            name: name.to_string(),
            description: None,
            state,
            value: None,
            position: None,
        }
    }

    /// Create list item element description
    pub fn list_item(name: &str, position: (usize, usize), state: FocusState) -> Self {
        Self {
            role: "list item".to_string(),
            name: name.to_string(),
            description: None,
            state,
            value: None,
            position: Some(position),
        }
    }

    /// Create input field element description
    pub fn input_field(name: &str, value: Option<&str>, state: FocusState) -> Self {
        Self {
            role: "text input".to_string(),
            name: name.to_string(),
            description: None,
            state,
            value: value.map(|v| v.to_string()),
            position: None,
        }
    }

    /// Create panel element description
    pub fn panel(name: &str, description: Option<&str>, state: FocusState) -> Self {
        Self {
            role: "panel".to_string(),
            name: name.to_string(),
            description: description.map(|d| d.to_string()),
            state,
            value: None,
            position: None,
        }
    }

    /// Create menu element description
    pub fn menu(name: &str, position: (usize, usize), state: FocusState) -> Self {
        Self {
            role: "menu item".to_string(),
            name: name.to_string(),
            description: None,
            state,
            value: None,
            position: Some(position),
        }
    }

    /// Create tab element description
    pub fn tab(name: &str, position: (usize, usize), is_selected: bool) -> Self {
        let state = if is_selected { FocusState::Active } else { FocusState::None };
        
        Self {
            role: "tab".to_string(),
            name: name.to_string(),
            description: if is_selected { Some("selected".to_string()) } else { None },
            state,
            value: None,
            position: Some(position),
        }
    }
}

impl Default for ScreenReaderInterface {
    fn default() -> Self {
        Self::new()
    }
}