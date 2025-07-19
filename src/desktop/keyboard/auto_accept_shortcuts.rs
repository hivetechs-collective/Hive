// Auto-Accept Keyboard Shortcuts Handler
use dioxus::prelude::*;
use crate::consensus::operation_intelligence::AutoAcceptMode;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Keyboard shortcut handler for auto-accept modes
#[derive(Debug, Clone)]
pub struct AutoAcceptShortcuts {
    /// Current mode
    current_mode: Signal<AutoAcceptMode>,
    
    /// Callback for mode changes
    on_mode_change: Arc<dyn Fn(AutoAcceptMode) + Send + Sync>,
    
    /// Notification callback
    on_notification: Arc<dyn Fn(String) + Send + Sync>,
}

impl AutoAcceptShortcuts {
    pub fn new(
        current_mode: Signal<AutoAcceptMode>,
        on_mode_change: Arc<dyn Fn(AutoAcceptMode) + Send + Sync>,
        on_notification: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Self {
        Self {
            current_mode,
            on_mode_change,
            on_notification,
        }
    }

    /// Handle keyboard events
    pub fn handle_key_event(&self, event: &KeyboardEvent) -> bool {
        // Check for Shift+Tab (cycle forward)
        if event.key() == "Tab" && event.shift_key() && !event.ctrl_key() && !event.alt_key() {
            self.cycle_mode_forward();
            return true;
        }
        
        // Check for Ctrl+Shift+Tab (cycle backward)
        if event.key() == "Tab" && event.shift_key() && event.ctrl_key() && !event.alt_key() {
            self.cycle_mode_backward();
            return true;
        }
        
        // Check for direct mode shortcuts (Ctrl+Alt+Number)
        if event.ctrl_key() && event.alt_key() && !event.shift_key() {
            match event.key().as_str() {
                "1" => {
                    self.set_mode(AutoAcceptMode::Conservative);
                    return true;
                }
                "2" => {
                    self.set_mode(AutoAcceptMode::Balanced);
                    return true;
                }
                "3" => {
                    self.set_mode(AutoAcceptMode::Aggressive);
                    return true;
                }
                "4" => {
                    self.set_mode(AutoAcceptMode::Plan);
                    return true;
                }
                "5" => {
                    self.set_mode(AutoAcceptMode::Manual);
                    return true;
                }
                _ => {}
            }
        }
        
        false
    }

    /// Cycle to the next mode
    fn cycle_mode_forward(&self) {
        let current = self.current_mode.read().clone();
        let next = match current {
            AutoAcceptMode::Conservative => AutoAcceptMode::Balanced,
            AutoAcceptMode::Balanced => AutoAcceptMode::Aggressive,
            AutoAcceptMode::Aggressive => AutoAcceptMode::Plan,
            AutoAcceptMode::Plan => AutoAcceptMode::Manual,
            AutoAcceptMode::Manual => AutoAcceptMode::Conservative,
        };
        
        self.set_mode(next);
    }

    /// Cycle to the previous mode
    fn cycle_mode_backward(&self) {
        let current = self.current_mode.read().clone();
        let previous = match current {
            AutoAcceptMode::Conservative => AutoAcceptMode::Manual,
            AutoAcceptMode::Balanced => AutoAcceptMode::Conservative,
            AutoAcceptMode::Aggressive => AutoAcceptMode::Balanced,
            AutoAcceptMode::Plan => AutoAcceptMode::Aggressive,
            AutoAcceptMode::Manual => AutoAcceptMode::Plan,
        };
        
        self.set_mode(previous);
    }

    /// Set the mode and notify
    fn set_mode(&self, mode: AutoAcceptMode) {
        // Update the signal
        self.current_mode.set(mode.clone());
        
        // Call the callback
        (self.on_mode_change)(mode.clone());
        
        // Show notification
        let message = format!("Auto-accept mode: {}", get_mode_label(&mode));
        (self.on_notification)(message);
    }
}

/// Global keyboard event handler component
#[component]
pub fn KeyboardHandler(
    shortcuts: AutoAcceptShortcuts,
    children: Element,
) -> Element {
    // Set up global keyboard listener
    use_effect(move || {
        let shortcuts_clone = shortcuts.clone();
        
        // Add event listener to document
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let key_event = KeyboardEvent::from(event);
            shortcuts_clone.handle_key_event(&key_event);
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        
        // Clean up on drop
        closure.forget();
    });

    rsx! {
        div {
            class: "keyboard-handler",
            onkeydown: move |event| {
                shortcuts.handle_key_event(&event.data);
            },
            {children}
        }
    }
}

/// Visual keyboard shortcut hints component
#[component]
pub fn KeyboardShortcutHints() -> Element {
    let show_hints = use_signal(|| false);
    
    rsx! {
        div {
            class: "keyboard-shortcuts",
            
            // Toggle button
            button {
                class: "shortcuts-toggle",
                onclick: move |_| show_hints.set(!show_hints()),
                title: "Show keyboard shortcuts",
                "⌨️"
            }
            
            // Hints panel
            if show_hints() {
                div {
                    class: "shortcuts-panel",
                    
                    div {
                        class: "shortcuts-header",
                        h3 { "Auto-Accept Keyboard Shortcuts" }
                        button {
                            class: "close-btn",
                            onclick: move |_| show_hints.set(false),
                            "×"
                        }
                    }
                    
                    div {
                        class: "shortcuts-content",
                        
                        div {
                            class: "shortcut-section",
                            
                            h4 { "Mode Cycling" }
                            
                            ShortcutItem {
                                keys: "Shift + Tab",
                                description: "Cycle to next auto-accept mode"
                            }
                            
                            ShortcutItem {
                                keys: "Ctrl + Shift + Tab",
                                description: "Cycle to previous auto-accept mode"
                            }
                        }
                        
                        div {
                            class: "shortcut-section",
                            
                            h4 { "Direct Mode Selection" }
                            
                            ShortcutItem {
                                keys: "Ctrl + Alt + 1",
                                description: "Conservative mode"
                            }
                            
                            ShortcutItem {
                                keys: "Ctrl + Alt + 2",
                                description: "Balanced mode"
                            }
                            
                            ShortcutItem {
                                keys: "Ctrl + Alt + 3",
                                description: "Aggressive mode"
                            }
                            
                            ShortcutItem {
                                keys: "Ctrl + Alt + 4",
                                description: "Plan only mode"
                            }
                            
                            ShortcutItem {
                                keys: "Ctrl + Alt + 5",
                                description: "Manual mode"
                            }
                        }
                        
                        div {
                            class: "shortcut-section",
                            
                            h4 { "Mode Descriptions" }
                            
                            ModeDescription {
                                mode: "Conservative",
                                description: "Only auto-accept very safe operations (>90% confidence, <15% risk)"
                            }
                            
                            ModeDescription {
                                mode: "Balanced",
                                description: "Balance safety with automation (>80% confidence, <25% risk)"
                            }
                            
                            ModeDescription {
                                mode: "Aggressive",
                                description: "Maximize automation, accept more risk (>70% confidence, <40% risk)"
                            }
                            
                            ModeDescription {
                                mode: "Plan Only",
                                description: "Generate plans but don't execute automatically"
                            }
                            
                            ModeDescription {
                                mode: "Manual",
                                description: "Require confirmation for all operations"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual shortcut item
#[component]
fn ShortcutItem(keys: &'static str, description: &'static str) -> Element {
    rsx! {
        div {
            class: "shortcut-item",
            
            div {
                class: "shortcut-keys",
                {keys.split(" + ").map(|key| rsx! {
                    span { class: "key", "{key}" }
                    if key != keys.split(" + ").last().unwrap() {
                        span { class: "plus", "+" }
                    }
                })}
            }
            
            div {
                class: "shortcut-description",
                "{description}"
            }
        }
    }
}

/// Mode description component
#[component]
fn ModeDescription(mode: &'static str, description: &'static str) -> Element {
    rsx! {
        div {
            class: "mode-description-item",
            
            div {
                class: "mode-name",
                "{mode}"
            }
            
            div {
                class: "mode-desc",
                "{description}"
            }
        }
    }
}

/// Auto-accept mode toast notification
#[component]
pub fn ModeChangeNotification(
    message: Signal<Option<String>>,
    duration_ms: Option<u64>,
) -> Element {
    let visible = use_signal(|| false);
    let timeout_handle = use_signal(|| None::<i32>);
    
    // Show/hide logic
    use_effect(move || {
        if let Some(msg) = message.read().as_ref() {
            visible.set(true);
            
            // Clear existing timeout
            if let Some(handle) = timeout_handle.read().as_ref() {
                web_sys::window().unwrap().clear_timeout_with_handle(*handle);
            }
            
            // Set new timeout
            let duration = duration_ms.unwrap_or(2000);
            let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                visible.set(false);
            }) as Box<dyn FnMut()>);
            
            let handle = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    duration as i32,
                )
                .unwrap();
            
            timeout_handle.set(Some(handle));
            callback.forget();
        }
    });

    rsx! {
        if visible() && message().is_some() {
            div {
                class: "mode-notification {if visible() { \"visible\" } else { \"\" }}",
                onclick: move |_| visible.set(false),
                
                div {
                    class: "notification-content",
                    
                    div {
                        class: "notification-icon",
                        "🔧"
                    }
                    
                    div {
                        class: "notification-message",
                        "{message().unwrap_or_default()}"
                    }
                    
                    button {
                        class: "notification-close",
                        onclick: move |e| {
                            e.stop_propagation();
                            visible.set(false);
                        },
                        "×"
                    }
                }
            }
        }
    }
}

/// Auto-accept mode indicator in status bar
#[component]
pub fn ModeIndicator(current_mode: Signal<AutoAcceptMode>) -> Element {
    let mode = current_mode();
    
    rsx! {
        div {
            class: "mode-indicator mode-{get_mode_class(&mode)}",
            title: "{get_mode_description(&mode)}",
            
            div {
                class: "mode-icon",
            }
            
            span {
                class: "mode-text",
                "{get_mode_short_label(&mode)}"
            }
        }
    }
}

// Helper functions

fn get_mode_label(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "Conservative",
        AutoAcceptMode::Balanced => "Balanced",
        AutoAcceptMode::Aggressive => "Aggressive",
        AutoAcceptMode::Plan => "Plan Only",
        AutoAcceptMode::Manual => "Manual",
    }
}

fn get_mode_short_label(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "CONS",
        AutoAcceptMode::Balanced => "BAL",
        AutoAcceptMode::Aggressive => "AGG",
        AutoAcceptMode::Plan => "PLAN",
        AutoAcceptMode::Manual => "MAN",
    }
}

fn get_mode_description(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "Only auto-accept very safe operations (>90% confidence, <15% risk)",
        AutoAcceptMode::Balanced => "Balance safety with automation (>80% confidence, <25% risk)",
        AutoAcceptMode::Aggressive => "Maximize automation, accept more risk (>70% confidence, <40% risk)",
        AutoAcceptMode::Plan => "Generate plans but don't execute automatically",
        AutoAcceptMode::Manual => "Require confirmation for all operations",
    }
}

fn get_mode_class(mode: &AutoAcceptMode) -> &'static str {
    match mode {
        AutoAcceptMode::Conservative => "conservative",
        AutoAcceptMode::Balanced => "balanced",
        AutoAcceptMode::Aggressive => "aggressive",
        AutoAcceptMode::Plan => "plan",
        AutoAcceptMode::Manual => "manual",
    }
}

/// Mock KeyboardEvent for compatibility
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    key: String,
    shift_key: bool,
    ctrl_key: bool,
    alt_key: bool,
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

impl From<web_sys::KeyboardEvent> for KeyboardEvent {
    fn from(event: web_sys::KeyboardEvent) -> Self {
        Self {
            key: event.key(),
            shift_key: event.shift_key(),
            ctrl_key: event.ctrl_key(),
            alt_key: event.alt_key(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mode_cycling() {
        // Test forward cycling
        assert_eq!(
            get_next_mode(&AutoAcceptMode::Conservative),
            AutoAcceptMode::Balanced
        );
        assert_eq!(
            get_next_mode(&AutoAcceptMode::Manual),
            AutoAcceptMode::Conservative
        );
        
        // Test backward cycling
        assert_eq!(
            get_previous_mode(&AutoAcceptMode::Conservative),
            AutoAcceptMode::Manual
        );
        assert_eq!(
            get_previous_mode(&AutoAcceptMode::Balanced),
            AutoAcceptMode::Conservative
        );
    }
    
    fn get_next_mode(current: &AutoAcceptMode) -> AutoAcceptMode {
        match current {
            AutoAcceptMode::Conservative => AutoAcceptMode::Balanced,
            AutoAcceptMode::Balanced => AutoAcceptMode::Aggressive,
            AutoAcceptMode::Aggressive => AutoAcceptMode::Plan,
            AutoAcceptMode::Plan => AutoAcceptMode::Manual,
            AutoAcceptMode::Manual => AutoAcceptMode::Conservative,
        }
    }
    
    fn get_previous_mode(current: &AutoAcceptMode) -> AutoAcceptMode {
        match current {
            AutoAcceptMode::Conservative => AutoAcceptMode::Manual,
            AutoAcceptMode::Balanced => AutoAcceptMode::Conservative,
            AutoAcceptMode::Aggressive => AutoAcceptMode::Balanced,
            AutoAcceptMode::Plan => AutoAcceptMode::Aggressive,
            AutoAcceptMode::Manual => AutoAcceptMode::Plan,
        }
    }
}