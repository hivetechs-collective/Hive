//! Keyboard shortcuts for diff actions
//! 
//! Provides VS Code-style keyboard shortcuts for staging, reverting, and managing diffs

use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::desktop::git::{DiffAction, DiffHunk, DiffLine};

/// Keyboard shortcut configuration
#[derive(Debug, Clone)]
pub struct ShortcutConfig {
    /// Alt+S: Stage hunk/line
    pub stage: KeyCombination,
    /// Alt+U: Unstage hunk/line  
    pub unstage: KeyCombination,
    /// Alt+R: Revert hunk/line
    pub revert: KeyCombination,
    /// Ctrl+A: Stage all hunks
    pub stage_all: KeyCombination,
    /// Ctrl+Shift+A: Unstage all hunks
    pub unstage_all: KeyCombination,
    /// Escape: Cancel current operation
    pub cancel: KeyCombination,
    /// F1: Show help
    pub help: KeyCombination,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            stage: KeyCombination {
                key: "KeyS".to_string(),
                alt: true,
                ctrl: false,
                shift: false,
                meta: false,
            },
            unstage: KeyCombination {
                key: "KeyU".to_string(),
                alt: true,
                ctrl: false,
                shift: false,
                meta: false,
            },
            revert: KeyCombination {
                key: "KeyR".to_string(),
                alt: true,
                ctrl: false,
                shift: false,
                meta: false,
            },
            stage_all: KeyCombination {
                key: "KeyA".to_string(),
                alt: false,
                ctrl: true,
                shift: false,
                meta: false,
            },
            unstage_all: KeyCombination {
                key: "KeyA".to_string(),
                alt: false,
                ctrl: true,
                shift: true,
                meta: false,
            },
            cancel: KeyCombination {
                key: "Escape".to_string(),
                alt: false,
                ctrl: false,
                shift: false,
                meta: false,
            },
            help: KeyCombination {
                key: "F1".to_string(),
                alt: false,
                ctrl: false,
                shift: false,
                meta: false,
            },
        }
    }
}

/// Key combination for shortcuts
#[derive(Debug, Clone, PartialEq)]
pub struct KeyCombination {
    pub key: String,
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub meta: bool,
}

impl KeyCombination {
    /// Check if this combination matches a keyboard event (placeholder for desktop)
    pub fn matches(&self, _event: &str) -> bool {
        // This would be implemented differently for desktop apps
        // For now, just return false as a placeholder
        false
    }
    
    /// Get human-readable representation
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();
        
        if self.ctrl { parts.push("Ctrl"); }
        if self.alt { parts.push("Alt"); }
        if self.shift { parts.push("Shift"); }
        if self.meta { parts.push("Cmd"); }
        
        // Convert key code to readable name
        let key_name = match self.key.as_str() {
            "KeyA" => "A",
            "KeyS" => "S", 
            "KeyU" => "U",
            "KeyR" => "R",
            "Escape" => "Esc",
            "F1" => "F1",
            other => other,
        };
        
        parts.push(key_name);
        parts.join("+")
    }
}

/// Shortcut action types
#[derive(Debug, Clone)]
pub enum ShortcutAction {
    /// Stage focused hunk or line
    Stage,
    /// Unstage focused hunk or line
    Unstage,
    /// Revert focused hunk or line
    Revert,
    /// Stage all hunks
    StageAll,
    /// Unstage all hunks
    UnstageAll,
    /// Cancel current operation
    Cancel,
    /// Show help dialog
    ShowHelp,
}

/// Context for determining what to act on
#[derive(Debug, Clone)]
pub struct ActionContext {
    /// Currently focused hunk ID
    pub focused_hunk: Option<String>,
    /// Currently focused line ID
    pub focused_line: Option<String>,
    /// All available hunks
    pub hunks: Vec<DiffHunk>,
    /// Current file path
    pub file_path: String,
    /// Repository path
    pub repo_path: String,
}

/// Keyboard shortcut manager
#[derive(Clone)]
pub struct ShortcutManager {
    config: ShortcutConfig,
    context: Arc<Mutex<Option<ActionContext>>>,
    action_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(ShortcutAction) + Send + Sync>>>>,
}

impl ShortcutManager {
    /// Create new shortcut manager
    pub fn new(config: ShortcutConfig) -> Self {
        Self {
            config,
            context: Arc::new(Mutex::new(None)),
            action_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Update action context
    pub fn update_context(&self, context: ActionContext) {
        if let Ok(mut ctx) = self.context.lock() {
            *ctx = Some(context);
        }
    }
    
    /// Register action handler
    pub fn register_handler<F>(&self, id: String, handler: F)
    where
        F: Fn(ShortcutAction) + Send + Sync + 'static,
    {
        if let Ok(mut handlers) = self.action_handlers.lock() {
            handlers.insert(id, Box::new(handler));
        }
    }
    
    /// Handle keyboard event (placeholder for desktop)
    pub fn handle_key_event(&self, event: &str) -> bool {
        let action = if self.config.stage.matches(event) {
            Some(ShortcutAction::Stage)
        } else if self.config.unstage.matches(event) {
            Some(ShortcutAction::Unstage)
        } else if self.config.revert.matches(event) {
            Some(ShortcutAction::Revert)
        } else if self.config.stage_all.matches(event) {
            Some(ShortcutAction::StageAll)
        } else if self.config.unstage_all.matches(event) {
            Some(ShortcutAction::UnstageAll)
        } else if self.config.cancel.matches(event) {
            Some(ShortcutAction::Cancel)
        } else if self.config.help.matches(event) {
            Some(ShortcutAction::ShowHelp)
        } else {
            None
        };
        
        if let Some(action) = action {
            self.execute_action(action);
            true
        } else {
            false
        }
    }
    
    /// Execute shortcut action
    fn execute_action(&self, action: ShortcutAction) {
        if let Ok(handlers) = self.action_handlers.lock() {
            for handler in handlers.values() {
                handler(action.clone());
            }
        }
    }
    
    /// Get all shortcut descriptions for help
    pub fn get_shortcuts_help(&self) -> Vec<(String, String)> {
        vec![
            (self.config.stage.to_string(), "Stage hunk or line".to_string()),
            (self.config.unstage.to_string(), "Unstage hunk or line".to_string()),
            (self.config.revert.to_string(), "Revert hunk or line".to_string()),
            (self.config.stage_all.to_string(), "Stage all changes".to_string()),
            (self.config.unstage_all.to_string(), "Unstage all changes".to_string()),
            (self.config.cancel.to_string(), "Cancel operation".to_string()),
            (self.config.help.to_string(), "Show help".to_string()),
        ]
    }
}

/// Global keyboard shortcut handler component (simplified for desktop)
#[component]
pub fn GlobalShortcutHandler(props: GlobalShortcutHandlerProps) -> Element {
    // For desktop apps, keyboard handling might be different
    // This is a placeholder implementation
    rsx! {
        div { 
            style: "display: none;",
            // Desktop keyboard shortcut handling would be implemented here
        }
    }
}

/// Props for global shortcut handler
#[derive(Props, Clone, PartialEq)]
pub struct GlobalShortcutHandlerProps {
    /// Whether shortcuts are enabled
    #[props(default = true)]
    pub enabled: bool,
}

/// Shortcut help dialog component
#[component]
pub fn ShortcutHelpDialog(props: ShortcutHelpDialogProps) -> Element {
    let manager = use_context::<Arc<ShortcutManager>>();
    let shortcuts = manager.get_shortcuts_help();
    
    if !props.visible {
        return rsx! { div {} };
    }
    
    rsx! {
        div {
            class: "shortcut-help-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); z-index: 1000; display: flex; align-items: center; justify-content: center;",
            onclick: move |_| {
                if let Some(handler) = &props.on_close {
                    handler.call(());
                }
            },
            
            div {
                class: "shortcut-help-dialog",
                style: "background: #2d2d30; border: 1px solid #3e3e42; border-radius: 6px; padding: 24px; max-width: 500px; width: 90%;",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                    
                    h3 {
                        style: "color: #cccccc; margin: 0; font-size: 16px; font-weight: 600;",
                        "Keyboard Shortcuts"
                    }
                    
                    button {
                        style: "background: none; border: none; color: #cccccc; font-size: 18px; cursor: pointer; padding: 4px;",
                        onclick: move |_| {
                            if let Some(handler) = &props.on_close {
                                handler.call(());
                            }
                        },
                        "×"
                    }
                }
                
                div {
                    class: "shortcuts-list",
                    style: "display: flex; flex-direction: column; gap: 12px;",
                    
                    for (shortcut, description) in shortcuts {
                        div {
                            class: "shortcut-item",
                            style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 0;",
                            
                            span {
                                style: "color: #cccccc; font-size: 14px;",
                                "{description}"
                            }
                            
                            span {
                                class: "shortcut-key",
                                style: "background: #3e3e42; color: #cccccc; padding: 4px 8px; border-radius: 3px; font-family: monospace; font-size: 12px;",
                                "{shortcut}"
                            }
                        }
                    }
                }
                
                div {
                    class: "dialog-footer",
                    style: "margin-top: 20px; text-align: center;",
                    
                    p {
                        style: "color: #8b949e; font-size: 12px; margin: 0;",
                        "Hover over hunks and lines to see available actions"
                    }
                }
            }
        }
    }
}

/// Props for shortcut help dialog
#[derive(Props, Clone, PartialEq)]
pub struct ShortcutHelpDialogProps {
    /// Whether dialog is visible
    pub visible: bool,
    /// Callback when dialog should close
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
}

/// Focus manager for tracking current hunk/line selection
#[derive(Clone)]
pub struct FocusManager {
    focused_hunk: Arc<Mutex<Option<String>>>,
    focused_line: Arc<Mutex<Option<String>>>,
}

impl FocusManager {
    /// Create new focus manager
    pub fn new() -> Self {
        Self {
            focused_hunk: Arc::new(Mutex::new(None)),
            focused_line: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Set focused hunk
    pub fn set_focused_hunk(&self, hunk_id: Option<String>) {
        if let Ok(mut focused) = self.focused_hunk.lock() {
            *focused = hunk_id;
        }
    }
    
    /// Set focused line
    pub fn set_focused_line(&self, line_id: Option<String>) {
        if let Ok(mut focused) = self.focused_line.lock() {
            *focused = line_id;
        }
    }
    
    /// Get focused hunk
    pub fn get_focused_hunk(&self) -> Option<String> {
        if let Ok(focused) = self.focused_hunk.lock() {
            focused.clone()
        } else {
            None
        }
    }
    
    /// Get focused line
    pub fn get_focused_line(&self) -> Option<String> {
        if let Ok(focused) = self.focused_line.lock() {
            focused.clone()
        } else {
            None
        }
    }
}

/// Focusable hunk component that tracks focus
#[component]
pub fn FocusableHunk(props: FocusableHunkProps) -> Element {
    let focus_manager = use_context::<Arc<FocusManager>>();
    let is_focused = use_signal(|| false);
    
    rsx! {
        div {
            class: "focusable-hunk",
            style: format!(
                "position: relative; {}",
                if is_focused() { "outline: 2px solid #007acc; outline-offset: 2px;" } else { "" }
            ),
            tabindex: 0,
            onfocusin: move |_| {
                is_focused.set(true);
                focus_manager.set_focused_hunk(Some(props.hunk_id.clone()));
            },
            onfocusout: move |_| {
                is_focused.set(false);
                focus_manager.set_focused_hunk(None);
            },
            onmouseenter: move |_| {
                focus_manager.set_focused_hunk(Some(props.hunk_id.clone()));
            },
            onmouseleave: move |_| {
                if !is_focused() {
                    focus_manager.set_focused_hunk(None);
                }
            },
            
            {props.children}
        }
    }
}

/// Props for focusable hunk
#[derive(Props, Clone, PartialEq)]
pub struct FocusableHunkProps {
    /// Hunk ID
    pub hunk_id: String,
    /// Child elements
    pub children: Element,
}

/// Focusable line component that tracks focus
#[component]
pub fn FocusableLine(props: FocusableLineProps) -> Element {
    let focus_manager = use_context::<Arc<FocusManager>>();
    let is_focused = use_signal(|| false);
    
    rsx! {
        div {
            class: "focusable-line",
            style: format!(
                "position: relative; {}",
                if is_focused() { "outline: 1px solid #007acc;" } else { "" }
            ),
            tabindex: 0,
            onfocusin: move |_| {
                is_focused.set(true);
                focus_manager.set_focused_line(Some(props.line_id.clone()));
            },
            onfocusout: move |_| {
                is_focused.set(false);
                focus_manager.set_focused_line(None);
            },
            onmouseenter: move |_| {
                focus_manager.set_focused_line(Some(props.line_id.clone()));
            },
            onmouseleave: move |_| {
                if !is_focused() {
                    focus_manager.set_focused_line(None);
                }
            },
            
            {props.children}
        }
    }
}

/// Props for focusable line
#[derive(Props, Clone, PartialEq)]
pub struct FocusableLineProps {
    /// Line ID
    pub line_id: String,
    /// Child elements
    pub children: Element,
}