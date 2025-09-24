//! Code Completion and IntelliSense UI
//!
//! Provides the completion popup and parameter hints

use super::language::{AIEnhancedCompletion, LanguageService};
use dioxus::prelude::*;
use lsp_types::CompletionItem;

#[derive(Debug, Clone)]
pub struct CompletionState {
    pub visible: bool,
    pub items: Vec<CompletionItem>,
    pub selected_index: usize,
    pub filter: String,
    pub position: CompletionPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletionPosition {
    pub line: usize,
    pub column: usize,
    pub x: f32,
    pub y: f32,
}

impl Default for CompletionState {
    fn default() -> Self {
        Self {
            visible: false,
            items: Vec::new(),
            selected_index: 0,
            filter: String::new(),
            position: CompletionPosition {
                line: 0,
                column: 0,
                x: 0.0,
                y: 0.0,
            },
        }
    }
}

#[component]
pub fn CompletionPopup(
    state: Signal<CompletionState>,
    on_select: EventHandler<CompletionItem>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !state.read().visible || state.read().items.is_empty() {
        return rsx! { div {} };
    }

    let position = state.read().position.clone();
    let popup_style = format!(
        r#"
        position: absolute;
        left: {}px;
        top: {}px;
        background: #252526;
        border: 1px solid #454545;
        border-radius: 4px;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
        max-height: 300px;
        min-width: 300px;
        overflow-y: auto;
        z-index: 1000;
        font-family: 'JetBrains Mono', monospace;
        font-size: 13px;
        "#,
        position.x,
        position.y + 20.0 // Position below cursor
    );

    rsx! {
        div {
            class: "completion-popup",
            style: "{popup_style}",
            onkeydown: move |evt: KeyboardEvent| {
                match evt.key() {
                    Key::ArrowUp => {
                        let mut state_mut = state.write();
                        if state_mut.selected_index > 0 {
                            state_mut.selected_index -= 1;
                        }
                    }
                    Key::ArrowDown => {
                        let mut state_mut = state.write();
                        if state_mut.selected_index < state_mut.items.len() - 1 {
                            state_mut.selected_index += 1;
                        }
                    }
                    Key::Enter => {
                        let selected_index = state.read().selected_index;
                        if let Some(item) = state.read().items.get(selected_index) {
                            on_select.call(item.clone());
                        }
                    }
                    Key::Escape => {
                        on_cancel.call(());
                    }
                    _ => {}
                }
            },

            // Completion items
            {
                let items = state.read().items.clone();
                let selected_index = state.read().selected_index;
                rsx! {
                    for (index, item) in items.iter().enumerate() {
                        {
                            let item_clone = item.clone();
                            let is_selected = index == selected_index;
                            let on_select_clone = on_select.clone();
                            rsx! {
                                CompletionItemComponent {
                                    item: item_clone.clone(),
                                    is_selected: is_selected,
                                    on_click: move |_| {
                                        on_select_clone.call(item_clone.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CompletionItemComponent(
    item: CompletionItem,
    is_selected: bool,
    on_click: EventHandler<()>,
) -> Element {
    let item_style = if is_selected {
        r#"
        padding: 4px 8px;
        background: #094771;
        color: #ffffff;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        "#
    } else {
        r#"
        padding: 4px 8px;
        background: transparent;
        color: #cccccc;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        &:hover {
            background: #2a2d2e;
        }
        "#
    };

    let icon = match item.kind {
        Some(lsp_types::CompletionItemKind::KEYWORD) => "🔤",
        Some(lsp_types::CompletionItemKind::FUNCTION) => "ƒ",
        Some(lsp_types::CompletionItemKind::METHOD) => "𝓜",
        Some(lsp_types::CompletionItemKind::VARIABLE) => "𝓥",
        Some(lsp_types::CompletionItemKind::CLASS) => "𝒞",
        Some(lsp_types::CompletionItemKind::STRUCT) => "𝒮",
        Some(lsp_types::CompletionItemKind::INTERFACE) => "𝐼",
        Some(lsp_types::CompletionItemKind::MODULE) => "📦",
        Some(lsp_types::CompletionItemKind::PROPERTY) => "●",
        _ => "◯",
    };

    let icon_color = match item.kind {
        Some(lsp_types::CompletionItemKind::KEYWORD) => "#569CD6",
        Some(lsp_types::CompletionItemKind::FUNCTION) => "#DCDCAA",
        Some(lsp_types::CompletionItemKind::METHOD) => "#DCDCAA",
        Some(lsp_types::CompletionItemKind::VARIABLE) => "#9CDCFE",
        Some(lsp_types::CompletionItemKind::CLASS) => "#4EC9B0",
        Some(lsp_types::CompletionItemKind::STRUCT) => "#4EC9B0",
        _ => "#cccccc",
    };

    rsx! {
        div {
            class: "completion-item",
            style: "{item_style}",
            onclick: move |_| on_click.call(()),

            // Icon
            span {
                style: "color: {icon_color}; font-size: 14px; width: 20px; text-align: center;",
                "{icon}"
            }

            // Label
            span {
                style: "flex: 1;",
                "{item.label}"
            }

            // Detail
            if let Some(detail) = &item.detail {
                span {
                    style: "color: #858585; font-size: 11px;",
                    "{detail}"
                }
            }
        }
    }
}

/// Parameter hints component
#[component]
pub fn ParameterHints(
    function_name: String,
    parameters: Vec<String>,
    current_parameter: usize,
    position: CompletionPosition,
) -> Element {
    let hint_style = format!(
        r#"
        position: absolute;
        left: {}px;
        top: {}px;
        background: #1e1e1e;
        border: 1px solid #454545;
        border-radius: 4px;
        padding: 4px 8px;
        font-family: 'JetBrains Mono', monospace;
        font-size: 13px;
        color: #cccccc;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
        z-index: 999;
        "#,
        position.x,
        position.y - 30.0 // Position above cursor
    );

    rsx! {
        div {
            class: "parameter-hints",
            style: "{hint_style}",

            span {
                style: "color: #DCDCAA;",
                "{function_name}"
            }
            "("

            for (index, param) in parameters.iter().enumerate() {
                span {
                    style: if index == current_parameter {
                        "font-weight: bold; color: #FFC107;"
                    } else {
                        "color: #9CDCFE;"
                    },
                    "{param}"
                }

                if index < parameters.len() - 1 {
                    ", "
                }
            }

            ")"
        }
    }
}

/// AI-enhanced completion item with consensus indicator
#[component]
pub fn AICompletionItem(
    completion: AIEnhancedCompletion,
    is_selected: bool,
    on_click: EventHandler<()>,
) -> Element {
    let confidence_color = if completion.ai_confidence > 0.8 {
        "#4CAF50" // Green for high confidence
    } else if completion.ai_confidence > 0.5 {
        "#FFC107" // Yellow for medium confidence
    } else {
        "#F44336" // Red for low confidence
    };

    rsx! {
        div {
            style: "position: relative;",

            CompletionItemComponent {
                item: completion.base_completion.clone(),
                is_selected,
                on_click,
            }

            // AI confidence indicator
            if completion.is_from_consensus {
                div {
                    style: format!(
                        "position: absolute; right: 8px; top: 50%; transform: translateY(-50%);
                         width: 8px; height: 8px; border-radius: 50%; background: {};",
                        confidence_color
                    ),
                    title: format!("AI Confidence: {:.0}%", completion.ai_confidence * 100.0),
                }
            }
        }
    }
}
