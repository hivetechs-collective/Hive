//! Git Decoration Configuration UI
//!
//! User interface for configuring git decoration appearance and behavior

use dioxus::prelude::*;
use crate::desktop::git::decorations::{GitDecorationConfig, DecorationStyles, GitDecorationManager};

/// Props for the decoration config UI
#[derive(Props, Clone, PartialEq)]
pub struct DecorationConfigProps {
    /// The decoration manager to configure
    pub decoration_manager: GitDecorationManager,
    /// Whether the config panel is visible
    pub visible: bool,
    /// Callback when config panel should be closed
    pub on_close: EventHandler<()>,
}

/// Git decoration configuration UI component
#[component]
pub fn GitDecorationConfigUI(props: DecorationConfigProps) -> Element {
    let mut local_config = use_signal(|| props.decoration_manager.config.read().clone());
    
    if !props.visible {
        return rsx! { div {} };
    }

    let config = local_config.read();

    rsx! {
        div {
            class: "decoration-config-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |evt| {
                if evt.target() == evt.current_target() {
                    props.on_close.call(());
                }
            },

            div {
                class: "decoration-config-panel",
                style: "background: var(--vscode-panel-background); border: 1px solid var(--vscode-panel-border); border-radius: 8px; padding: 24px; min-width: 500px; max-width: 600px; max-height: 80vh; overflow-y: auto;",

                // Header
                div {
                    class: "config-header",
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",

                    h2 {
                        style: "margin: 0; color: var(--vscode-foreground); font-size: 18px; font-weight: 600;",
                        "Git Decorations Settings"
                    }

                    button {
                        class: "btn-icon",
                        style: "width: 32px; height: 32px; padding: 0; background: transparent; border: none; color: var(--vscode-foreground); cursor: pointer;",
                        onclick: move |_| props.on_close.call(()),
                        title: "Close",
                        i { class: "fa-solid fa-xmark", style: "font-size: 16px;" }
                    }
                }

                // General Settings
                div {
                    class: "config-section",
                    style: "margin-bottom: 24px;",

                    h3 {
                        style: "margin: 0 0 12px 0; color: var(--vscode-foreground); font-size: 14px; font-weight: 600;",
                        "General"
                    }

                    div {
                        class: "config-group",
                        style: "display: flex; flex-direction: column; gap: 12px;",

                        // Enable decorations
                        label {
                            class: "checkbox-label",
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: config.enabled,
                                onchange: move |evt| {
                                    let mut new_config = local_config.read().clone();
                                    new_config.enabled = evt.checked();
                                    local_config.set(new_config);
                                }
                            }

                            span {
                                style: "color: var(--vscode-foreground); font-size: 13px;",
                                "Enable git decorations"
                            }
                        }

                        // Show status letters
                        label {
                            class: "checkbox-label",
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: config.show_status_letters,
                                disabled: !config.enabled,
                                onchange: move |evt| {
                                    let mut new_config = local_config.read().clone();
                                    new_config.show_status_letters = evt.checked();
                                    local_config.set(new_config);
                                }
                            }

                            span {
                                style: "color: var(--vscode-foreground); font-size: 13px;",
                                "Show status letters (M, A, D, etc.)"
                            }
                        }

                        // Show colors
                        label {
                            class: "checkbox-label",
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: config.show_colors,
                                disabled: !config.enabled,
                                onchange: move |evt| {
                                    let mut new_config = local_config.read().clone();
                                    new_config.show_colors = evt.checked();
                                    local_config.set(new_config);
                                }
                            }

                            span {
                                style: "color: var(--vscode-foreground); font-size: 13px;",
                                "Show status colors"
                            }
                        }

                        // Show folder decorations
                        label {
                            class: "checkbox-label",
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: config.show_folder_decorations,
                                disabled: !config.enabled,
                                onchange: move |evt| {
                                    let mut new_config = local_config.read().clone();
                                    new_config.show_folder_decorations = evt.checked();
                                    local_config.set(new_config);
                                }
                            }

                            span {
                                style: "color: var(--vscode-foreground); font-size: 13px;",
                                "Show folder decorations"
                            }
                        }

                        // Gray out ignored files
                        label {
                            class: "checkbox-label",
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: config.gray_out_ignored,
                                disabled: !config.enabled,
                                onchange: move |evt| {
                                    let mut new_config = local_config.read().clone();
                                    new_config.gray_out_ignored = evt.checked();
                                    local_config.set(new_config);
                                }
                            }

                            span {
                                style: "color: var(--vscode-foreground); font-size: 13px;",
                                "Gray out ignored files"
                            }
                        }

                        // Show conflict markers
                        label {
                            class: "checkbox-label",
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",

                            input {
                                r#type: "checkbox",
                                checked: config.show_conflict_markers,
                                disabled: !config.enabled,
                                onchange: move |evt| {
                                    let mut new_config = local_config.read().clone();
                                    new_config.show_conflict_markers = evt.checked();
                                    local_config.set(new_config);
                                }
                            }

                            span {
                                style: "color: var(--vscode-foreground); font-size: 13px;",
                                "Show conflict markers"
                            }
                        }
                    }
                }

                // Color Settings
                div {
                    class: "config-section",
                    style: "margin-bottom: 24px;",

                    h3 {
                        style: "margin: 0 0 12px 0; color: var(--vscode-foreground); font-size: 14px; font-weight: 600;",
                        "Colors"
                    }

                    div {
                        class: "color-grid",
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",

                        ColorInput {
                            label: "Modified",
                            color: config.styles.modified_color.clone(),
                            disabled: !config.enabled || !config.show_colors,
                            on_change: move |color: String| {
                                let mut new_config = local_config.read().clone();
                                new_config.styles.modified_color = color;
                                local_config.set(new_config);
                            }
                        }

                        ColorInput {
                            label: "Added",
                            color: config.styles.added_color.clone(),
                            disabled: !config.enabled || !config.show_colors,
                            on_change: move |color: String| {
                                let mut new_config = local_config.read().clone();
                                new_config.styles.added_color = color;
                                local_config.set(new_config);
                            }
                        }

                        ColorInput {
                            label: "Deleted",
                            color: config.styles.deleted_color.clone(),
                            disabled: !config.enabled || !config.show_colors,
                            on_change: move |color: String| {
                                let mut new_config = local_config.read().clone();
                                new_config.styles.deleted_color = color;
                                local_config.set(new_config);
                            }
                        }

                        ColorInput {
                            label: "Untracked",
                            color: config.styles.untracked_color.clone(),
                            disabled: !config.enabled || !config.show_colors,
                            on_change: move |color: String| {
                                let mut new_config = local_config.read().clone();
                                new_config.styles.untracked_color = color;
                                local_config.set(new_config);
                            }
                        }

                        ColorInput {
                            label: "Conflict",
                            color: config.styles.conflict_color.clone(),
                            disabled: !config.enabled || !config.show_colors,
                            on_change: move |color: String| {
                                let mut new_config = local_config.read().clone();
                                new_config.styles.conflict_color = color;
                                local_config.set(new_config);
                            }
                        }

                        ColorInput {
                            label: "Ignored",
                            color: config.styles.ignored_color.clone(),
                            disabled: !config.enabled || !config.show_colors,
                            on_change: move |color: String| {
                                let mut new_config = local_config.read().clone();
                                new_config.styles.ignored_color = color;
                                local_config.set(new_config);
                            }
                        }
                    }
                }

                // Advanced Settings
                div {
                    class: "config-section",
                    style: "margin-bottom: 24px;",

                    h3 {
                        style: "margin: 0 0 12px 0; color: var(--vscode-foreground); font-size: 14px; font-weight: 600;",
                        "Advanced"
                    }

                    div {
                        class: "config-group",
                        style: "display: flex; flex-direction: column; gap: 12px;",

                        // Opacity
                        div {
                            class: "slider-group",

                            label {
                                style: "color: var(--vscode-foreground); font-size: 13px; margin-bottom: 4px; display: block;",
                                "Opacity: {(config.styles.opacity * 100.0) as u32}%"
                            }

                            input {
                                r#type: "range",
                                min: "0.1",
                                max: "1.0",
                                step: "0.1",
                                value: "{config.styles.opacity}",
                                disabled: !config.enabled,
                                style: "width: 100%;",
                                oninput: move |evt| {
                                    if let Ok(opacity) = evt.value().parse::<f32>() {
                                        let mut new_config = local_config.read().clone();
                                        new_config.styles.opacity = opacity;
                                        local_config.set(new_config);
                                    }
                                }
                            }
                        }

                        // Font weight
                        div {
                            class: "select-group",

                            label {
                                style: "color: var(--vscode-foreground); font-size: 13px; margin-bottom: 4px; display: block;",
                                "Status Letter Font Weight"
                            }

                            select {
                                style: "width: 100%; padding: 6px; background: var(--vscode-input-background); border: 1px solid var(--vscode-input-border); color: var(--vscode-input-foreground); border-radius: 4px;",
                                disabled: !config.enabled || !config.show_status_letters,
                                value: "{config.styles.status_font_weight}",
                                onchange: move |evt| {
                                    let mut new_config = local_config.read().clone();
                                    new_config.styles.status_font_weight = evt.value();
                                    local_config.set(new_config);
                                },

                                option { value: "400", "Normal" }
                                option { value: "500", "Medium" }
                                option { value: "600", "Semi-bold" }
                                option { value: "700", "Bold" }
                            }
                        }
                    }
                }

                // Action buttons
                div {
                    class: "config-actions",
                    style: "display: flex; justify-content: space-between; gap: 12px; padding-top: 20px; border-top: 1px solid var(--vscode-panel-border);",

                    div {
                        class: "action-group",
                        style: "display: flex; gap: 8px;",

                        button {
                            class: "btn btn-secondary",
                            style: "padding: 8px 16px; font-size: 13px;",
                            onclick: move |_| {
                                local_config.set(GitDecorationConfig::default());
                            },
                            "Reset to Defaults"
                        }
                    }

                    div {
                        class: "action-group",
                        style: "display: flex; gap: 8px;",

                        button {
                            class: "btn btn-secondary",
                            style: "padding: 8px 16px; font-size: 13px;",
                            onclick: move |_| {
                                props.on_close.call(());
                            },
                            "Cancel"
                        }

                        button {
                            class: "btn btn-primary",
                            style: "padding: 8px 16px; font-size: 13px;",
                            onclick: move |_| {
                                props.decoration_manager.update_config(local_config.read().clone());
                                props.on_close.call(());
                            },
                            "Apply"
                        }
                    }
                }
            }
        }
    }
}

/// Color input component
#[component]
fn ColorInput(
    label: String,
    color: String,
    disabled: bool,
    on_change: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "color-input-group",

            label {
                style: "color: var(--vscode-foreground); font-size: 13px; margin-bottom: 4px; display: block;",
                "{label}"
            }

            div {
                class: "color-input-wrapper",
                style: "display: flex; align-items: center; gap: 8px;",

                input {
                    r#type: "color",
                    value: "{color}",
                    disabled: disabled,
                    style: "width: 40px; height: 32px; border: 1px solid var(--vscode-input-border); border-radius: 4px; background: transparent; cursor: pointer;",
                    onchange: move |evt| {
                        on_change.call(evt.value());
                    }
                }

                input {
                    r#type: "text",
                    value: "{color}",
                    disabled: disabled,
                    style: "flex: 1; padding: 6px; background: var(--vscode-input-background); border: 1px solid var(--vscode-input-border); color: var(--vscode-input-foreground); border-radius: 4px; font-family: monospace; font-size: 12px;",
                    onchange: move |evt| {
                        let value = evt.value();
                        if value.starts_with('#') && (value.len() == 7 || value.len() == 4) {
                            on_change.call(value);
                        }
                    }
                }
            }
        }
    }
}

/// Preview component showing decoration examples
#[component]
pub fn GitDecorationPreview(config: GitDecorationConfig) -> Element {
    if !config.enabled {
        return rsx! {
            div {
                style: "padding: 20px; text-align: center; color: var(--vscode-tab-inactiveForeground);",
                "Git decorations are disabled"
            }
        };
    }

    rsx! {
        div {
            class: "decoration-preview",
            style: "background: var(--vscode-sideBar-background); border: 1px solid var(--vscode-sideBar-border); border-radius: 4px; padding: 12px;",

            h4 {
                style: "margin: 0 0 12px 0; color: var(--vscode-foreground); font-size: 13px; font-weight: 600;",
                "Preview"
            }

            div {
                class: "preview-items",
                style: "display: flex; flex-direction: column; gap: 6px; font-family: var(--vscode-font-family); font-size: 13px;",

                // Modified file
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    i { class: "fa-solid fa-file", style: "width: 16px; color: #519aba;" }
                    span { "example.rs" }
                    if config.show_status_letters {
                        span {
                            style: "color: {config.styles.modified_color}; opacity: {config.styles.opacity}; font-weight: {config.styles.status_font_weight}; font-size: 11px;",
                            "M"
                        }
                    }
                }

                // Added file
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    i { class: "fa-solid fa-file", style: "width: 16px; color: #519aba;" }
                    span { "new_file.rs" }
                    if config.show_status_letters {
                        span {
                            style: "color: {config.styles.added_color}; opacity: {config.styles.opacity}; font-weight: {config.styles.status_font_weight}; font-size: 11px;",
                            "A"
                        }
                    }
                }

                // Folder with changes
                if config.show_folder_decorations {
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        i { class: "fa-solid fa-folder", style: "width: 16px; color: #dcb67a;" }
                        span { "src" }
                        span {
                            style: "color: {config.styles.modified_color}; opacity: {config.styles.opacity}; font-size: 10px;",
                            "●"
                        }
                    }
                }

                // Conflict file
                if config.show_conflict_markers {
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        i { class: "fa-solid fa-file", style: "width: 16px; color: #519aba;" }
                        span { "conflict.rs" }
                        span {
                            style: "color: {config.styles.conflict_color}; opacity: {config.styles.opacity}; font-weight: {config.styles.status_font_weight}; font-size: 11px;",
                            "!"
                        }
                    }
                }

                // Ignored file
                if config.gray_out_ignored {
                    div {
                        style: "display: flex; align-items: center; gap: 8px; opacity: 0.5;",
                        i { class: "fa-solid fa-file", style: "width: 16px; color: {config.styles.ignored_color};" }
                        span { style: "color: {config.styles.ignored_color};", "ignored.log" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GitDecorationConfig::default();
        assert!(config.enabled);
        assert!(config.show_status_letters);
        assert!(config.show_colors);
    }

    #[test]
    fn test_default_styles() {
        let styles = DecorationStyles::default();
        assert_eq!(styles.modified_color, "#e2c08d");
        assert_eq!(styles.added_color, "#73c991");
        assert_eq!(styles.opacity, 0.8);
    }
}