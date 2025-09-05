// Auto-Accept Settings Panel Component
// Provides UI for configuring auto-accept behavior and preferences

use dioxus::prelude::*;
use crate::consensus::smart_decision_engine::{UserPreferences, CustomRule, RuleAction};
use crate::consensus::operation_analysis::AutoAcceptMode;
use crate::desktop::styles::theme::ThemeColors;
use crate::desktop::components::common::{Button, Toggle, Select, Input};

/// Auto-accept settings panel props
#[derive(Props, Clone, PartialEq)]
pub struct AutoAcceptSettingsProps {
    /// Current user preferences
    pub preferences: UserPreferences,
    
    /// Callback when preferences change
    pub on_preferences_change: EventHandler<UserPreferences>,
    
    /// Theme colors
    pub theme: ThemeColors,
}

/// Auto-accept settings panel component
#[component]
pub fn AutoAcceptSettings(props: AutoAcceptSettingsProps) -> Element {
    let mut preferences = use_signal(|| props.preferences.clone());
    let mut show_custom_rules = use_signal(|| false);
    let mut new_rule_pattern = use_signal(|| String::new());
    let mut new_rule_reason = use_signal(|| String::new());
    
    // Update parent when preferences change
    use_effect(move || {
        props.on_preferences_change.call(preferences());
    });
    
    rsx! {
        div {
            class: "auto-accept-settings",
            style: "padding: 20px; max-width: 800px; margin: 0 auto;",
            
            // Header
            h2 {
                style: "color: {props.theme.text}; margin-bottom: 24px;",
                "🤖 Auto-Accept Settings"
            }
            
            // Mode Selection
            div {
                style: "
                    background: {props.theme.background_secondary};
                    border: 1px solid {props.theme.border};
                    border-radius: 8px;
                    padding: 20px;
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
                    margin-bottom: 16px;
                ",
                
                h3 {
                    style: "
                        margin: 0 0 16px;
                        color: {props.theme.text};
                        font-size: 18px;
                        font-weight: 600;
                    ",
                    "Auto-Accept Mode"
                }
                
                div {
                    class: "mode-selection",
                    style: "display: flex; flex-direction: column; gap: 12px;",
                        
                        ModeOption {
                            mode: AutoAcceptMode::Manual,
                            current_mode: preferences().preferred_mode,
                            label: "Manual",
                            description: "Review every operation before execution",
                            icon: "🔒",
                            theme: props.theme.clone(),
                            on_select: move |mode| {
                                let mut prefs = preferences();
                                prefs.preferred_mode = mode;
                                preferences.set(prefs);
                            }
                        }
                        
                        ModeOption {
                            mode: AutoAcceptMode::Conservative,
                            current_mode: preferences().preferred_mode,
                            label: "Conservative",
                            description: "Auto-accept only very safe operations (>90% confidence)",
                            icon: "🛡️",
                            theme: props.theme.clone(),
                            on_select: move |mode| {
                                let mut prefs = preferences();
                                prefs.preferred_mode = mode;
                                preferences.set(prefs);
                            }
                        }
                        
                        ModeOption {
                            mode: AutoAcceptMode::Balanced,
                            current_mode: preferences().preferred_mode,
                            label: "Balanced",
                            description: "Auto-accept reasonably safe operations (>80% confidence)",
                            icon: "⚖️",
                            theme: props.theme.clone(),
                            on_select: move |mode| {
                                let mut prefs = preferences();
                                prefs.preferred_mode = mode;
                                preferences.set(prefs);
                            }
                        }
                        
                        ModeOption {
                            mode: AutoAcceptMode::Aggressive,
                            current_mode: preferences().preferred_mode,
                            label: "Aggressive",
                            description: "Auto-accept most operations unless high risk (>70% confidence)",
                            icon: "🚀",
                            theme: props.theme.clone(),
                            on_select: move |mode| {
                                let mut prefs = preferences();
                                prefs.preferred_mode = mode;
                                preferences.set(prefs);
                            }
                        }
                        
                        ModeOption {
                            mode: AutoAcceptMode::Plan,
                            current_mode: preferences().preferred_mode,
                            label: "Plan Only",
                            description: "Generate execution plans but don't execute",
                            icon: "📋",
                            theme: props.theme.clone(),
                            on_select: move |mode| {
                                let mut prefs = preferences();
                                prefs.preferred_mode = mode;
                                preferences.set(prefs);
                            }
                        }
                    }
                }
            }
            
            // Risk Settings
            div {
                style: "
                    background: {props.theme.background_secondary};
                    border: 1px solid {props.theme.border};
                    border-radius: 8px;
                    padding: 20px;
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
                    margin-bottom: 16px;
                ",
                
                h3 {
                    style: "
                        margin: 0 0 16px;
                        color: {props.theme.text};
                        font-size: 18px;
                        font-weight: 600;
                    ",
                    "Risk Settings"
                }
                
                div {
                        class: "risk-settings",
                        style: "display: flex; flex-direction: column; gap: 16px;",
                        
                        // Risk Tolerance Slider
                        div {
                            label {
                                style: "color: {props.theme.text_secondary}; font-size: 14px;",
                                {format!("Risk Tolerance: {:.0}%", preferences().risk_tolerance * 100.0)}
                            }
                            input {
                                r#type: "range",
                                min: "0",
                                max: "100",
                                value: "{preferences().risk_tolerance * 100.0}",
                                style: "width: 100%;",
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse::<f32>() {
                                        let mut prefs = preferences();
                                        prefs.risk_tolerance = value / 100.0;
                                        preferences.set(prefs);
                                    }
                                }
                            }
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 12px; color: {props.theme.text_secondary};",
                                span { "Conservative" }
                                span { "Moderate" }
                                span { "Aggressive" }
                            }
                        }
                        
                        // AI Trust Level
                        div {
                            label {
                                style: "color: {props.theme.text_secondary}; font-size: 14px;",
                                {format!("AI Trust Level: {:.0}%", preferences().trust_ai_suggestions * 100.0)}
                            }
                            input {
                                r#type: "range",
                                min: "0",
                                max: "100",
                                value: "{preferences().trust_ai_suggestions * 100.0}",
                                style: "width: 100%;",
                                oninput: move |evt| {
                                    if let Ok(value) = evt.value().parse::<f32>() {
                                        let mut prefs = preferences();
                                        prefs.trust_ai_suggestions = value / 100.0;
                                        preferences.set(prefs);
                                    }
                                }
                            }
                        }
                }
            }
            
            // Safety Options
            div {
                style: "
                    background: {props.theme.background_secondary};
                    border: 1px solid {props.theme.border};
                    border-radius: 8px;
                    padding: 20px;
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
                    margin-bottom: 16px;
                ",
                
                h3 {
                    style: "
                        margin: 0 0 16px;
                        color: {props.theme.text};
                        font-size: 18px;
                        font-weight: 600;
                    ",
                    "Safety Options"
                }
                
                div {
                    class: "safety-options",
                    style: "display: flex; flex-direction: column; gap: 12px;",
                        
                        Toggle {
                            label: "Auto-create backups",
                            checked: preferences().auto_backup,
                            theme: props.theme.clone(),
                            on_change: move |checked| {
                                let mut prefs = preferences();
                                prefs.auto_backup = checked;
                                preferences.set(prefs);
                            }
                        }
                        
                        Toggle {
                            label: "Require confirmation for deletions",
                            checked: preferences().require_confirmation_for_deletions,
                            theme: props.theme.clone(),
                            on_change: move |checked| {
                                let mut prefs = preferences();
                                prefs.require_confirmation_for_deletions = checked;
                                preferences.set(prefs);
                            }
                        }
                        
                        Toggle {
                            label: "Require confirmation for mass updates",
                            checked: preferences().require_confirmation_for_mass_updates,
                            theme: props.theme.clone(),
                            on_change: move |checked| {
                                let mut prefs = preferences();
                                prefs.require_confirmation_for_mass_updates = checked;
                                preferences.set(prefs);
                            }
                        }
                }
            }
            
            // Custom Rules
            div {
                style: "
                    background: {props.theme.background_secondary};
                    border: 1px solid {props.theme.border};
                    border-radius: 8px;
                    padding: 20px;
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
                    margin-bottom: 16px;
                ",
                
                h3 {
                    style: "
                        margin: 0 0 16px;
                        color: {props.theme.text};
                        font-size: 18px;
                        font-weight: 600;
                    ",
                    "Custom Rules"
                }
                
                div {
                    class: "custom-rules",
                    
                    // Existing rules
                    if !preferences().custom_rules.is_empty() {
                        div {
                            style: "margin-bottom: 16px;",
                            for (idx, rule) in preferences().custom_rules.iter().enumerate() {
                                CustomRuleItem {
                                    rule: rule.clone(),
                                    theme: props.theme.clone(),
                                    on_delete: move |_| {
                                        let mut prefs = preferences();
                                        prefs.custom_rules.remove(idx);
                                        preferences.set(prefs);
                                    }
                                }
                            }
                        }
                    }
                    
                    // Add new rule
                    if show_custom_rules() {
                        div {
                            style: "border: 1px solid {props.theme.border}; padding: 16px; border-radius: 8px; margin-top: 12px;",
                            
                            Input {
                                placeholder: "File pattern (regex)",
                                value: new_rule_pattern(),
                                theme: props.theme.clone(),
                                on_change: move |val| new_rule_pattern.set(val),
                            }
                            
                            Select {
                                options: vec![
                                    ("always_accept", "Always Accept"),
                                    ("always_reject", "Always Reject"),
                                    ("require_review", "Require Review"),
                                ],
                                theme: props.theme.clone(),
                                on_change: move |_val| {
                                    // TODO: Handle select change
                                },
                            }
                            
                            Input {
                                placeholder: "Reason",
                                value: new_rule_reason(),
                                theme: props.theme.clone(),
                                on_change: move |val| new_rule_reason.set(val),
                            }
                            
                            div {
                                style: "display: flex; gap: 8px; margin-top: 12px;",
                                
                                Button {
                                    label: "Add Rule",
                                    variant: "primary",
                                    theme: props.theme.clone(),
                                    on_click: move |_| {
                                        if !new_rule_pattern().is_empty() && !new_rule_reason().is_empty() {
                                            let mut prefs = preferences();
                                            prefs.custom_rules.push(CustomRule {
                                                pattern: new_rule_pattern(),
                                                action: RuleAction::AlwaysAutoExecute, // Default for now
                                                reason: new_rule_reason(),
                                            });
                                            preferences.set(prefs);
                                            new_rule_pattern.set(String::new());
                                            new_rule_reason.set(String::new());
                                            show_custom_rules.set(false);
                                        }
                                    }
                                }
                                
                                Button {
                                    label: "Cancel",
                                    variant: "secondary",
                                    theme: props.theme.clone(),
                                    on_click: move |_| {
                                        show_custom_rules.set(false);
                                        new_rule_pattern.set(String::new());
                                        new_rule_reason.set(String::new());
                                    }
                                }
                            }
                        }
                    } else {
                        Button {
                            label: "+ Add Custom Rule",
                            variant: "secondary",
                            theme: props.theme.clone(),
                            on_click: move |_| show_custom_rules.set(true),
                        }
                    }
                }
            }
    }
}

/// Mode option component
#[derive(Props, Clone, PartialEq)]
struct ModeOptionProps {
    mode: AutoAcceptMode,
    current_mode: AutoAcceptMode,
    label: &'static str,
    description: &'static str,
    icon: &'static str,
    theme: ThemeColors,
    on_select: EventHandler<AutoAcceptMode>,
}

#[component]
fn ModeOption(props: ModeOptionProps) -> Element {
    let is_selected = props.mode == props.current_mode;
    let background_color = if is_selected { format!("{}10", props.theme.primary) } else { "transparent".to_string() };
    let border_color = if is_selected { props.theme.primary.clone() } else { props.theme.border.clone() };
    
    rsx! {
        div {
            class: "mode-option",
            style: "
                display: flex;
                align-items: center;
                padding: 16px;
                border: 2px solid {border_color};
                border-radius: 8px;
                cursor: pointer;
                transition: all 0.2s;
                background: {background_color};
            ",
            onclick: move |_| props.on_select.call(props.mode),
            
            span {
                style: "font-size: 24px; margin-right: 16px;",
                "{props.icon}"
            }
            
            div {
                style: "flex: 1;",
                
                h4 {
                    style: "margin: 0; color: {props.theme.text}; font-size: 16px;",
                    "{props.label}"
                }
                
                p {
                    style: "margin: 4px 0 0; color: {props.theme.text_secondary}; font-size: 14px;",
                    "{props.description}"
                }
            }
            
            if is_selected {
                span {
                    style: "color: {props.theme.primary}; font-size: 20px;",
                    "✓"
                }
            }
        }
    }
}

/// Custom rule item component
#[derive(Props, Clone, PartialEq)]
struct CustomRuleItemProps {
    rule: CustomRule,
    theme: ThemeColors,
    on_delete: EventHandler<()>,
}

#[component]
fn CustomRuleItem(props: CustomRuleItemProps) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                padding: 12px;
                background: {props.theme.background_secondary};
                border-radius: 6px;
                margin-bottom: 8px;
            ",
            
            div {
                style: "flex: 1;",
                
                code {
                    style: "color: {props.theme.primary}; font-family: monospace;",
                    "{props.rule.pattern}"
                }
                
                span {
                    style: "color: {props.theme.text_secondary}; margin: 0 8px;",
                    "→"
                }
                
                span {
                    style: "color: {props.theme.text};",
                    match props.rule.action {
                        RuleAction::AlwaysAutoExecute => "Always Accept",
                        RuleAction::AlwaysBlock => "Always Reject",
                        RuleAction::AlwaysConfirm => "Require Review",
                        RuleAction::RequireBackup => "Require Backup",
                    }
                }
                
                span {
                    style: "color: {props.theme.text_secondary}; margin-left: 12px; font-size: 14px;",
                    "({props.rule.reason})"
                }
            }
            
            button {
                style: "
                    background: none;
                    border: none;
                    color: {props.theme.error};
                    cursor: pointer;
                    padding: 4px 8px;
                    font-size: 16px;
                ",
                onclick: move |_| props.on_delete.call(()),
                "×"
            }
        }
    }
}