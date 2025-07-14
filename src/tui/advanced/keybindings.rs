//! VS Code-like keybindings for advanced TUI
//!
//! Provides comprehensive keyboard shortcuts similar to VS Code

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Keybinding manager for VS Code-like shortcuts
pub struct KeybindingManager {
    /// Custom keybindings
    bindings: HashMap<KeySequence, Action>,
    /// Context-specific bindings
    context_bindings: HashMap<String, HashMap<KeySequence, Action>>,
}

/// Key sequence for binding
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    pub modifiers: KeyModifiers,
    pub key: KeyCode,
}

/// Action to perform
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Navigate to different panel
    SwitchPanel(String),
    /// Execute command
    Command(String),
    /// Show dialog
    ShowDialog(String),
    /// Custom action
    Custom(String),
}

impl KeybindingManager {
    /// Create new keybinding manager with defaults
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            context_bindings: HashMap::new(),
        };

        manager.setup_default_bindings();
        manager
    }

    /// Setup default VS Code-like keybindings
    fn setup_default_bindings(&mut self) {
        // Global shortcuts
        self.add_binding(
            KeySequence {
                modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                key: KeyCode::Char('P'),
            },
            Action::ShowDialog("command_palette".to_string()),
        );

        self.add_binding(
            KeySequence {
                modifiers: KeyModifiers::CONTROL,
                key: KeyCode::Char('p'),
            },
            Action::ShowDialog("quick_open".to_string()),
        );

        self.add_binding(
            KeySequence {
                modifiers: KeyModifiers::CONTROL,
                key: KeyCode::Char('`'),
            },
            Action::SwitchPanel("terminal".to_string()),
        );

        // Panel shortcuts
        self.add_binding(
            KeySequence {
                modifiers: KeyModifiers::NONE,
                key: KeyCode::F(1),
            },
            Action::SwitchPanel("explorer".to_string()),
        );

        self.add_binding(
            KeySequence {
                modifiers: KeyModifiers::NONE,
                key: KeyCode::F(2),
            },
            Action::SwitchPanel("editor".to_string()),
        );

        self.add_binding(
            KeySequence {
                modifiers: KeyModifiers::NONE,
                key: KeyCode::F(3),
            },
            Action::SwitchPanel("terminal".to_string()),
        );

        self.add_binding(
            KeySequence {
                modifiers: KeyModifiers::NONE,
                key: KeyCode::F(4),
            },
            Action::SwitchPanel("consensus".to_string()),
        );
    }

    /// Add custom keybinding
    pub fn add_binding(&mut self, sequence: KeySequence, action: Action) {
        self.bindings.insert(sequence, action);
    }

    /// Remove keybinding
    pub fn remove_binding(&mut self, sequence: &KeySequence) {
        self.bindings.remove(sequence);
    }

    /// Get action for key event
    pub fn get_action(&self, key: KeyEvent, context: Option<&str>) -> Option<&Action> {
        let sequence = KeySequence {
            modifiers: key.modifiers,
            key: key.code,
        };

        // Check context-specific bindings first
        if let Some(context) = context {
            if let Some(context_bindings) = self.context_bindings.get(context) {
                if let Some(action) = context_bindings.get(&sequence) {
                    return Some(action);
                }
            }
        }

        // Check global bindings
        self.bindings.get(&sequence)
    }

    /// Add context-specific binding
    pub fn add_context_binding(&mut self, context: String, sequence: KeySequence, action: Action) {
        self.context_bindings
            .entry(context)
            .or_insert_with(HashMap::new)
            .insert(sequence, action);
    }

    /// Get all bindings for display
    pub fn get_all_bindings(&self) -> Vec<(KeySequence, Action)> {
        self.bindings
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for KeybindingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeySequence {
    /// Create from key event
    pub fn from_key_event(key: KeyEvent) -> Self {
        Self {
            modifiers: key.modifiers,
            key: key.code,
        }
    }

    /// Display string for keybinding
    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt".to_string());
        }

        let key_str = match self.key {
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            _ => "?".to_string(),
        };
        parts.push(key_str);

        parts.join("+")
    }
}
