// Auto-Accept Keyboard Shortcuts Handler
use dioxus::prelude::*;
use crate::consensus::operation_intelligence::AutoAcceptMode;
use std::sync::Arc;

/// Keyboard shortcut handler for auto-accept modes
#[derive(Clone)]
pub struct AutoAcceptShortcuts {
    /// Current mode
    current_mode: Signal<AutoAcceptMode>,
}

impl std::fmt::Debug for AutoAcceptShortcuts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoAcceptShortcuts")
            .field("current_mode", &self.current_mode)
            .finish()
    }
}

impl AutoAcceptShortcuts {
    pub fn new(
        current_mode: Signal<AutoAcceptMode>,
    ) -> Self {
        Self {
            current_mode,
        }
    }

    /// Handle keyboard shortcut
    pub fn handle_key_event(&self, key_event: &KeyboardEvent) {
        // Shift+Tab cycles through auto-accept modes
        if key_event.shift_key() && key_event.key() == "Tab" {
            self.cycle_mode();
        }
        
        // Ctrl+Shift+A toggles auto-accept on/off
        if key_event.ctrl_key() && key_event.shift_key() && key_event.key() == "A" {
            self.toggle_auto_accept();
        }
        
        // Escape key sets to manual mode
        if key_event.key() == "Escape" {
            self.set_manual_mode();
        }
    }

    fn cycle_mode(&self) {
        let mut mode = self.current_mode;
        let current = *mode.read();
        let next_mode = match current {
            AutoAcceptMode::Manual => AutoAcceptMode::Conservative,
            AutoAcceptMode::Conservative => AutoAcceptMode::Balanced,
            AutoAcceptMode::Balanced => AutoAcceptMode::Aggressive,
            AutoAcceptMode::Aggressive => AutoAcceptMode::Plan,
            AutoAcceptMode::Plan => AutoAcceptMode::Manual,
        };
        
        mode.set(next_mode);
        tracing::info!("Auto-accept mode: {:?}", next_mode);
    }

    fn toggle_auto_accept(&self) {
        let mut mode = self.current_mode;
        let current = *mode.read();
        let new_mode = match current {
            AutoAcceptMode::Manual => AutoAcceptMode::Conservative,
            _ => AutoAcceptMode::Manual,
        };
        
        mode.set(new_mode);
        tracing::info!("Auto-accept: {}", 
            if matches!(new_mode, AutoAcceptMode::Manual) { "OFF" } else { "ON" });
    }

    fn set_manual_mode(&self) {
        let mut mode = self.current_mode;
        mode.set(AutoAcceptMode::Manual);
        tracing::info!("Auto-accept disabled");
    }
}

/// Keyboard shortcut display component
#[component]
pub fn KeyboardShortcutDisplay(
    show_shortcuts: Signal<bool>,
    current_mode: Signal<AutoAcceptMode>,
    on_mode_change: EventHandler<AutoAcceptMode>,
) -> Element {
    let shortcuts = use_signal(|| {
        AutoAcceptShortcuts::new(current_mode)
    });
    
    // Watch for mode changes from shortcuts
    use_effect(move || {
        let mode = *current_mode.read();
        on_mode_change.call(mode);
    });

    // Set up keyboard event listener
    use_effect(move || {
        // In a real implementation, this would set up global keyboard listeners
        // For now, we'll handle it through the component event system
    });

    rsx! {
        if show_shortcuts() {
            div {
                class: "keyboard-shortcuts-overlay",
                onclick: move |_| show_shortcuts.set(false),
                
                div {
                    class: "shortcuts-panel",
                    onclick: move |e| e.stop_propagation(),
                    
                    div {
                        class: "panel-header",
                        h3 { "Keyboard Shortcuts" }
                        button {
                            class: "close-button",
                            onclick: move |_| show_shortcuts.set(false),
                            "×"
                        }
                    }
                    
                    div {
                        class: "shortcuts-content",
                        
                        ShortcutGroup {
                            title: "Auto-Accept Modes",
                            shortcuts: vec![
                                ("Shift + Tab", "Cycle through modes"),
                                ("Ctrl + Shift + A", "Toggle auto-accept on/off"), 
                                ("Esc", "Set to manual mode"),
                            ]
                        }
                        
                        ShortcutGroup {
                            title: "Operation Control",
                            shortcuts: vec![
                                ("Enter", "Confirm operation"),
                                ("Ctrl + Z", "Undo last operation"),
                                ("Ctrl + Shift + R", "Trigger rollback"),
                            ]
                        }
                        
                        ShortcutGroup {
                            title: "Navigation",
                            shortcuts: vec![
                                ("Ctrl + P", "Open command palette"),
                                ("Ctrl + ,", "Open settings"),
                                ("F1", "Show help"),
                            ]
                        }
                    }
                    
                    div {
                        class: "current-mode-display",
                        h4 { "Current Mode" }
                        div {
                            class: "mode-indicator {get_mode_class(&current_mode())}",
                            "{current_mode():?}"
                        }
                        div {
                            class: "mode-description",
                            "{get_mode_description(&current_mode())}"
                        }
                    }
                }
            }
        }
    }
}

/// Keyboard shortcut notification
#[component]
pub fn ShortcutNotification(
    message: Signal<Option<String>>,
    visible: Signal<bool>,
    duration_ms: Option<u32>,
) -> Element {
    let timeout_handle = use_signal(|| None::<i32>);
    
    // Show notification when message changes
    use_effect(move || {
        if let Some(_msg) = message.read().as_ref() {
            visible.set(true);
            
            // Auto-hide after duration
            if let Some(duration) = duration_ms {
                let mut visible_clone = visible.clone();
                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(duration as u64)).await;
                    visible_clone.set(false);
                });
            }
        }
    });

    rsx! {
        if visible() {
            if let Some(ref msg) = message() {
                div {
                    class: "shortcut-notification",
                    onclick: move |_| visible.set(false),
                    
                    div {
                        class: "notification-content",
                        
                        div {
                            class: "notification-icon",
                            "⌨️"
                        }
                        
                        div {
                            class: "notification-message",
                            "{msg}"
                        }
                        
                        button {
                            class: "notification-close",
                            onclick: move |_| visible.set(false),
                            "×"
                        }
                    }
                }
            }
        }
    }
}

/// Individual shortcut group display
#[component]
pub fn ShortcutGroup(
    title: String,
    shortcuts: Vec<(&'static str, &'static str)>,
) -> Element {
    rsx! {
        div {
            class: "shortcut-group",
            
            h4 {
                class: "group-title",
                "{title}"
            }
            
            div {
                class: "shortcuts-list",
                for (key, description) in shortcuts {
                    div {
                        class: "shortcut-item",
                        
                        div {
                            class: "shortcut-key",
                            "{key}"
                        }
                        
                        div {
                            class: "shortcut-description",
                            "{description}"
                        }
                    }
                }
            }
        }
    }
}

/// Mode status indicator component
#[component]
pub fn ModeStatusIndicator(
    current_mode: Signal<AutoAcceptMode>,
    compact: Option<bool>,
) -> Element {
    let is_compact = compact.unwrap_or(false);
    
    rsx! {
        div {
            class: if is_compact { "mode-status-indicator compact" } else { "mode-status-indicator" },
            title: "{get_mode_description(&current_mode())}",
            
            div {
                class: format!("mode-icon {}", get_mode_class(&current_mode())),
                "{get_mode_icon(&current_mode())}"
            }
            
            if !is_compact {
                div {
                    class: "mode-text",
                    "{current_mode():?}"
                }
            }
        }
    }
}

/// Global keyboard event listener component
#[component]
pub fn GlobalKeyboardListener(
    current_mode: Signal<AutoAcceptMode>,
    on_mode_change: EventHandler<AutoAcceptMode>,
    on_shortcut_triggered: EventHandler<String>,
) -> Element {
    let shortcuts = use_signal(|| {
        AutoAcceptShortcuts::new(current_mode)
    });

    // This component handles global keyboard events
    // In a desktop app, this would integrate with the window's event system
    
    rsx! {
        div {
            // Hidden component that handles keyboard events
            style: "display: none;",
            tabindex: "0", // Make it focusable to receive keyboard events
            onkeydown: move |event| {
                let key_event = KeyboardEvent {
                    key: event.key().to_string(),
                    shift_key: event.modifiers().shift(),
                    ctrl_key: event.modifiers().ctrl(),
                    alt_key: event.modifiers().alt(),
                };
                shortcuts().handle_key_event(&key_event);
            },
        }
    }
}

// Helper functions and types

#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key: String,
    pub shift_key: bool,
    pub ctrl_key: bool,
    pub alt_key: bool,
}

impl KeyboardEvent {
    pub fn key(&self) -> &str {
        &self.key
    }
    
    pub fn shift_key(&self) -> bool {
        self.shift_key
    }
    
    pub fn ctrl_key(&self) -> bool {
        self.ctrl_key
    }
    
    pub fn alt_key(&self) -> bool {
        self.alt_key
    }
}

fn get_mode_class(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Manual => "manual",
        AutoAcceptMode::Conservative => "conservative",
        AutoAcceptMode::Balanced => "balanced",
        AutoAcceptMode::Aggressive => "aggressive",
        AutoAcceptMode::Plan => "plan",
    }
}

fn get_mode_icon(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Manual => "✋",
        AutoAcceptMode::Conservative => "🛡️",
        AutoAcceptMode::Balanced => "⚖️",
        AutoAcceptMode::Aggressive => "⚡",
        AutoAcceptMode::Plan => "📋",
    }
}

fn get_mode_description(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Manual => "All operations require manual approval",
        AutoAcceptMode::Conservative => "Auto-accept only very safe operations (>90% confidence, <15% risk)",
        AutoAcceptMode::Balanced => "Auto-accept moderately safe operations (>80% confidence, <25% risk)",
        AutoAcceptMode::Aggressive => "Auto-accept most operations (>70% confidence, <40% risk)",
        AutoAcceptMode::Plan => "Generate execution plans without auto-execution",
    }
}