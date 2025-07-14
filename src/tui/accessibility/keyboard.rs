//! Keyboard navigation support
//!
//! Provides comprehensive keyboard-only navigation

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Keyboard navigation manager
pub struct KeyboardNavigation {
    keyboard_only: bool,
    focus_order: Vec<String>,
    current_focus_index: usize,
    custom_shortcuts: std::collections::HashMap<String, KeyBinding>,
}

/// Key binding definition
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key_code: KeyCode,
    pub modifiers: KeyModifiers,
    pub action: String,
    pub description: String,
}

/// Navigation direction
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationDirection {
    Next,
    Previous,
    First,
    Last,
    Up,
    Down,
    Left,
    Right,
}

/// Focus management result
#[derive(Debug, Clone, PartialEq)]
pub enum FocusResult {
    /// Focus moved successfully
    Moved(String),
    /// Focus stayed on same element
    Stayed,
    /// Focus reached boundary
    Boundary,
    /// Action was performed
    Action(String),
}

impl KeyboardNavigation {
    /// Create new keyboard navigation manager
    pub fn new() -> Self {
        Self {
            keyboard_only: false,
            focus_order: Vec::new(),
            current_focus_index: 0,
            custom_shortcuts: std::collections::HashMap::new(),
        }
    }

    /// Enable or disable keyboard-only mode
    pub fn set_keyboard_only(&mut self, keyboard_only: bool) {
        self.keyboard_only = keyboard_only;
    }

    /// Check if keyboard-only mode is active
    pub fn is_keyboard_only(&self) -> bool {
        self.keyboard_only
    }

    /// Set focus order for navigation
    pub fn set_focus_order(&mut self, order: Vec<String>) {
        self.focus_order = order;
        self.current_focus_index = 0;
    }

    /// Add element to focus order
    pub fn add_focusable_element(&mut self, element_id: String) {
        if !self.focus_order.contains(&element_id) {
            self.focus_order.push(element_id);
        }
    }

    /// Remove element from focus order
    pub fn remove_focusable_element(&mut self, element_id: &str) {
        if let Some(pos) = self.focus_order.iter().position(|x| x == element_id) {
            self.focus_order.remove(pos);

            // Adjust current focus index if necessary
            if self.current_focus_index >= self.focus_order.len() && !self.focus_order.is_empty() {
                self.current_focus_index = self.focus_order.len() - 1;
            }
        }
    }

    /// Navigate focus in specified direction
    pub fn navigate(&mut self, direction: NavigationDirection) -> FocusResult {
        if self.focus_order.is_empty() {
            return FocusResult::Boundary;
        }

        let old_index = self.current_focus_index;

        match direction {
            NavigationDirection::Next => {
                if self.current_focus_index < self.focus_order.len() - 1 {
                    self.current_focus_index += 1;
                } else {
                    return FocusResult::Boundary;
                }
            }
            NavigationDirection::Previous => {
                if self.current_focus_index > 0 {
                    self.current_focus_index -= 1;
                } else {
                    return FocusResult::Boundary;
                }
            }
            NavigationDirection::First => {
                self.current_focus_index = 0;
            }
            NavigationDirection::Last => {
                self.current_focus_index = self.focus_order.len() - 1;
            }
            _ => {
                // For Up/Down/Left/Right, delegate to specific panel navigation
                return FocusResult::Action(format!("Navigate {:?}", direction));
            }
        }

        if old_index != self.current_focus_index {
            FocusResult::Moved(self.focus_order[self.current_focus_index].clone())
        } else {
            FocusResult::Stayed
        }
    }

    /// Get currently focused element
    pub fn current_focus(&self) -> Option<&String> {
        self.focus_order.get(self.current_focus_index)
    }

    /// Set focus to specific element
    pub fn set_focus(&mut self, element_id: &str) -> FocusResult {
        if let Some(index) = self.focus_order.iter().position(|x| x == element_id) {
            self.current_focus_index = index;
            FocusResult::Moved(element_id.to_string())
        } else {
            FocusResult::Stayed
        }
    }

    /// Handle key event for navigation
    pub fn handle_key_event(&mut self, key: KeyEvent) -> FocusResult {
        // Check for custom shortcuts first
        let shortcut_key = format!("{:?}+{:?}", key.modifiers, key.code);
        if let Some(binding) = self.custom_shortcuts.get(&shortcut_key) {
            return FocusResult::Action(binding.action.clone());
        }

        // Handle standard navigation keys
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Tab) => self.navigate(NavigationDirection::Next),
            (KeyModifiers::SHIFT, KeyCode::Tab) => self.navigate(NavigationDirection::Previous),
            (KeyModifiers::CONTROL, KeyCode::Home) => self.navigate(NavigationDirection::First),
            (KeyModifiers::CONTROL, KeyCode::End) => self.navigate(NavigationDirection::Last),
            (KeyModifiers::NONE, KeyCode::Up) => self.navigate(NavigationDirection::Up),
            (KeyModifiers::NONE, KeyCode::Down) => self.navigate(NavigationDirection::Down),
            (KeyModifiers::NONE, KeyCode::Left) => self.navigate(NavigationDirection::Left),
            (KeyModifiers::NONE, KeyCode::Right) => self.navigate(NavigationDirection::Right),
            (KeyModifiers::NONE, KeyCode::Enter) => FocusResult::Action("activate".to_string()),
            (KeyModifiers::NONE, KeyCode::Char(' ')) => FocusResult::Action("select".to_string()),
            (KeyModifiers::NONE, KeyCode::Esc) => FocusResult::Action("escape".to_string()),
            _ => FocusResult::Stayed,
        }
    }

    /// Add custom keyboard shortcut
    pub fn add_shortcut(
        &mut self,
        key_code: KeyCode,
        modifiers: KeyModifiers,
        action: String,
        description: String,
    ) {
        let key = format!("{:?}+{:?}", modifiers, key_code);
        let binding = KeyBinding {
            key_code,
            modifiers,
            action,
            description,
        };
        self.custom_shortcuts.insert(key, binding);
    }

    /// Remove custom keyboard shortcut
    pub fn remove_shortcut(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        let key = format!("{:?}+{:?}", modifiers, key_code);
        self.custom_shortcuts.remove(&key);
    }

    /// Get all keyboard shortcuts
    pub fn get_all_shortcuts(&self) -> Vec<(String, String)> {
        let mut shortcuts = vec![
            ("Tab".to_string(), "Next element".to_string()),
            ("Shift+Tab".to_string(), "Previous element".to_string()),
            ("Ctrl+Home".to_string(), "First element".to_string()),
            ("Ctrl+End".to_string(), "Last element".to_string()),
            (
                "Arrow Keys".to_string(),
                "Navigate within element".to_string(),
            ),
            ("Enter".to_string(), "Activate element".to_string()),
            ("Space".to_string(), "Select/toggle element".to_string()),
            ("Escape".to_string(), "Cancel/close".to_string()),
        ];

        // Add custom shortcuts
        for binding in self.custom_shortcuts.values() {
            let key_desc = if binding.modifiers.is_empty() {
                format!("{:?}", binding.key_code)
            } else {
                format!("{:?}+{:?}", binding.modifiers, binding.key_code)
            };
            shortcuts.push((key_desc, binding.description.clone()));
        }

        shortcuts
    }

    /// Get shortcuts for current context
    pub fn get_context_shortcuts(&self, context: &str) -> Vec<(String, String)> {
        let mut shortcuts = self.get_all_shortcuts();

        // Add context-specific shortcuts
        match context {
            "explorer" => {
                shortcuts.extend(vec![
                    ("Enter".to_string(), "Open file/directory".to_string()),
                    ("Space".to_string(), "Expand/collapse directory".to_string()),
                    ("Backspace".to_string(), "Go up one level".to_string()),
                    ("h".to_string(), "Toggle hidden files".to_string()),
                    ("r".to_string(), "Refresh".to_string()),
                ]);
            }
            "editor" => {
                shortcuts.extend(vec![
                    ("Ctrl+O".to_string(), "Open file".to_string()),
                    ("Ctrl+S".to_string(), "Save file".to_string()),
                    ("Ctrl+W".to_string(), "Close tab".to_string()),
                    ("Ctrl+F".to_string(), "Find".to_string()),
                    ("Ctrl+H".to_string(), "Replace".to_string()),
                ]);
            }
            "terminal" => {
                shortcuts.extend(vec![
                    ("Enter".to_string(), "Execute command".to_string()),
                    ("Up/Down".to_string(), "Command history".to_string()),
                    ("Ctrl+C".to_string(), "Cancel command".to_string()),
                    ("Ctrl+L".to_string(), "Clear terminal".to_string()),
                ]);
            }
            _ => {}
        }

        shortcuts
    }

    /// Check if key combination is reserved
    pub fn is_reserved_key(&self, key_code: KeyCode, modifiers: KeyModifiers) -> bool {
        matches!(
            (modifiers, key_code),
            (KeyModifiers::NONE, KeyCode::Tab)
                | (KeyModifiers::SHIFT, KeyCode::Tab)
                | (KeyModifiers::CONTROL, KeyCode::Home)
                | (KeyModifiers::CONTROL, KeyCode::End)
                | (KeyModifiers::NONE, KeyCode::Enter)
                | (KeyModifiers::NONE, KeyCode::Esc)
        )
    }

    /// Get help text for keyboard navigation
    pub fn get_navigation_help(&self) -> String {
        vec![
            "Keyboard Navigation Help:",
            "",
            "Tab / Shift+Tab: Move between elements",
            "Arrow keys: Navigate within elements",
            "Enter: Activate buttons and links",
            "Space: Select checkboxes and toggle buttons",
            "Escape: Close dialogs and cancel operations",
            "Ctrl+Home/End: Jump to first/last element",
            "",
            "Panel Shortcuts:",
            "F1: Explorer panel",
            "F2: Editor panel",
            "F3: Terminal panel",
            "F4: Consensus panel",
            "",
            "Global Shortcuts:",
            "Ctrl+P: Quick file search",
            "Ctrl+Shift+P: Command palette",
            "Ctrl+`: Toggle terminal",
            "Ctrl+Q: Quit application",
        ]
        .join("\n")
    }

    /// Reset focus to first element
    pub fn reset_focus(&mut self) {
        self.current_focus_index = 0;
    }

    /// Get focus order for debugging
    pub fn focus_order(&self) -> &[String] {
        &self.focus_order
    }

    /// Get current focus index
    pub fn current_focus_index(&self) -> usize {
        self.current_focus_index
    }
}

impl Default for KeyboardNavigation {
    fn default() -> Self {
        Self::new()
    }
}
