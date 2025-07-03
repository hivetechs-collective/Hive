//! Panel management for advanced TUI
//!
//! Manages the layout and state of different panels

use anyhow::Result;
use ratatui::layout::Rect;
use crate::tui::themes::Theme;

/// Panel manager for coordinating multiple panels
pub struct PanelManager {
    /// Currently active panel
    active_panel_id: String,
    /// Panel visibility state
    visible_panels: std::collections::HashMap<String, bool>,
}

impl PanelManager {
    /// Create new panel manager
    pub fn new() -> Self {
        let mut visible_panels = std::collections::HashMap::new();
        visible_panels.insert("explorer".to_string(), true);
        visible_panels.insert("editor".to_string(), true);
        visible_panels.insert("terminal".to_string(), true);
        visible_panels.insert("consensus".to_string(), true);
        
        Self {
            active_panel_id: "explorer".to_string(),
            visible_panels,
        }
    }
    
    /// Set active panel
    pub fn set_active_panel(&mut self, panel_id: String) {
        self.active_panel_id = panel_id;
    }
    
    /// Get active panel ID
    pub fn active_panel(&self) -> &str {
        &self.active_panel_id
    }
    
    /// Toggle panel visibility
    pub fn toggle_panel(&mut self, panel_id: &str) {
        if let Some(visible) = self.visible_panels.get_mut(panel_id) {
            *visible = !*visible;
        }
    }
    
    /// Check if panel is visible
    pub fn is_panel_visible(&self, panel_id: &str) -> bool {
        self.visible_panels.get(panel_id).copied().unwrap_or(false)
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}