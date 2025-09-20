//! Git stash UI components with VS Code-style interface
//!
//! Provides:
//! - Stash list panel with icons and timestamps
//! - Stash creation dialog with message input
//! - Stash preview panel showing changes
//! - Context menu for stash operations

use dioxus::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

use super::stash::{async_ops, GitStash, StashApplyOptions, StashInfo, StashSaveOptions};
// use super::icons::GitIcons;

/// VS Code-style stash list component
#[component]
pub fn StashList(
    repo_path: Signal<Option<PathBuf>>,
    on_stash_select: Option<EventHandler<StashInfo>>,
    on_stash_action: Option<EventHandler<StashAction>>,
) -> Element {
    let mut stashes = use_signal(|| Vec::<StashInfo>::new());
    let mut selected_stash = use_signal(|| None::<usize>);
    let mut loading = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    // Load stashes when repo changes
    use_effect(move || {
        if let Some(path) = repo_path.read().as_ref() {
            let path = path.clone();
            let mut loading = loading.clone();
            let mut error_message = error_message.clone();
            let mut stashes = stashes.clone();
            spawn(async move {
                loading.set(true);
                error_message.set(None);

                match async_ops::list_stashes(&path).await {
                    Ok(stash_list) => {
                        stashes.set(stash_list);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to load stashes: {}", e)));
                    }
                }

                loading.set(false);
            });
        }
    });

    rsx! {
        div {
            class: "stash-list-container",
            style: STASH_LIST_STYLES,

            // Header with stash count and actions
            div {
                class: "stash-header",
                style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border-bottom: 1px solid #333; background: #252526;",

                div {
                    class: "stash-title",
                    style: "font-weight: 600; color: #cccccc; font-size: 13px;",
                    "STASHES ({stashes.read().len()})"
                }

                div {
                    class: "stash-actions",
                    style: "display: flex; gap: 4px;",

                    button {
                        class: "stash-action-btn",
                        style: STASH_ACTION_BUTTON_STYLES,
                        title: "Save Stash",
                        onclick: move |_| {
                            if let Some(handler) = &on_stash_action {
                                handler.call(StashAction::Save);
                            }
                        },
                        "💾"
                    }

                    button {
                        class: "stash-action-btn",
                        style: STASH_ACTION_BUTTON_STYLES,
                        title: "Refresh Stashes",
                        onclick: move |_| {
                            if let Some(path) = repo_path.read().as_ref() {
                                let path = path.clone();
                                spawn(async move {
                                    loading.set(true);
                                    match async_ops::list_stashes(&path).await {
                                        Ok(stash_list) => stashes.set(stash_list),
                                        Err(e) => error_message.set(Some(format!("Failed to refresh: {}", e))),
                                    }
                                    loading.set(false);
                                });
                            }
                        },
                        "🔄"
                    }
                }
            }

            // Loading indicator
            if loading.read().clone() {
                div {
                    class: "stash-loading",
                    style: "padding: 16px; text-align: center; color: #888; font-size: 12px;",
                    "Loading stashes..."
                }
            }

            // Error message
            if let Some(error) = error_message.read().as_ref() {
                div {
                    class: "stash-error",
                    style: "padding: 12px; background: #f48771; color: #1e1e1e; font-size: 12px; margin: 8px;",
                    "{error}"
                }
            }

            // Stash list
            if !loading.read().clone() && error_message.read().is_none() {
                div {
                    class: "stash-items",
                    style: "flex: 1; overflow-y: auto;",

                    if stashes.read().is_empty() {
                        div {
                            class: "no-stashes",
                            style: "padding: 24px; text-align: center; color: #888; font-size: 12px;",
                            div { "No stashes found" }
                            div {
                                style: "margin-top: 8px; font-size: 11px;",
                                "Use 'Save Stash' to stash your changes"
                            }
                        }
                    } else {
                        for (index, stash) in stashes.read().iter().enumerate() {
                            StashListItem {
                                key: "{index}",
                                stash: stash.clone(),
                                index,
                                is_selected: selected_stash.read().map_or(false, |sel| sel == index),
                                on_select: {
                                    let mut selected_stash_clone = selected_stash;
                                    let on_stash_select = on_stash_select.clone();
                                    let stash_clone = stash.clone();
                                    move |idx| {
                                        selected_stash_clone.set(Some(idx));
                                        if let Some(handler) = &on_stash_select {
                                            handler.call(stash_clone.clone());
                                        }
                                    }
                                },
                                on_action: {
                                    let on_stash_action = on_stash_action.clone();
                                    move |action| {
                                        if let Some(handler) = &on_stash_action {
                                            handler.call(action);
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
}

/// Individual stash list item component
#[component]
pub fn StashListItem(
    stash: StashInfo,
    index: usize,
    is_selected: bool,
    on_select: EventHandler<usize>,
    on_action: EventHandler<StashAction>,
) -> Element {
    let mut show_context_menu = use_signal(|| false);
    let mut context_menu_pos = use_signal(|| (0, 0));

    rsx! {
        div {
            class: "stash-item",
            style: format!("{}{}",
                STASH_ITEM_STYLES,
                if is_selected { "background: #094771 !important;" } else { "" }
            ),
            onclick: move |_| on_select.call(index),
            oncontextmenu: move |evt| {
                evt.prevent_default();
                // Use coordinates relative to viewport for context menu positioning
                context_menu_pos.set((100, 100)); // Fallback position
                show_context_menu.set(true);
            },

            div {
                class: "stash-main",
                style: "display: flex; align-items: center; gap: 8px; flex: 1;",

                // Stash icon
                div {
                    class: "stash-icon",
                    style: "font-size: 16px; color: #f0ad4e; flex-shrink: 0;",
                    "📦"
                }

                // Stash info
                div {
                    class: "stash-info",
                    style: "flex: 1; min-width: 0;",

                    div {
                        class: "stash-message",
                        style: "font-size: 13px; color: #cccccc; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        title: "{stash.message}",
                        "{stash.message}"
                    }

                    div {
                        class: "stash-details",
                        style: "font-size: 11px; color: #888; margin-top: 2px; display: flex; gap: 12px;",

                        span {
                            title: "Files changed",
                            "📁 {stash.stats.files_changed}"
                        }

                        span {
                            title: "Insertions",
                            style: "color: #89d185;",
                            "+{stash.stats.insertions}"
                        }

                        span {
                            title: "Deletions",
                            style: "color: #f48771;",
                            "-{stash.stats.deletions}"
                        }

                        span {
                            title: "Created {stash.formatted_time}",
                            "{stash.formatted_time}"
                        }

                        if stash.has_untracked {
                            span {
                                title: "Includes untracked files",
                                style: "color: #f0ad4e;",
                                "📄"
                            }
                        }
                    }
                }
            }

            // Context menu
            if show_context_menu.read().clone() {
                StashContextMenu {
                    stash: stash.clone(),
                    index: index,
                    position: context_menu_pos.read().clone(),
                    on_action: move |action| {
                        show_context_menu.set(false);
                        on_action.call(action);
                    },
                    on_close: move || show_context_menu.set(false)
                }
            }
        }
    }
}

/// Context menu for stash operations
#[component]
fn StashContextMenu(
    stash: StashInfo,
    index: usize,
    position: (i32, i32),
    on_action: EventHandler<StashAction>,
    on_close: EventHandler<()>,
) -> Element {
    // Close menu when clicking outside (simplified for desktop)
    use_effect(move || {
        // In a desktop app, we'd handle this differently
        // For now, we'll rely on explicit close buttons
    });

    rsx! {
        div {
            class: "stash-context-menu",
            style: format!("{}left: {}px; top: {}px;", STASH_CONTEXT_MENU_STYLES, position.0, position.1),
            onclick: move |evt| evt.stop_propagation(),

            div {
                class: "context-menu-item",
                style: CONTEXT_MENU_ITEM_STYLES,
                onclick: move |_| on_action.call(StashAction::Apply(index)),
                div {
                    style: "margin-right: 8px;",
                    "📋"
                }
                "Apply Stash"
            }

            div {
                class: "context-menu-item",
                style: CONTEXT_MENU_ITEM_STYLES,
                onclick: move |_| on_action.call(StashAction::Pop(index)),
                div {
                    style: "margin-right: 8px;",
                    "📤"
                }
                "Pop Stash"
            }

            div {
                class: "context-menu-separator",
                style: "height: 1px; background: #333; margin: 4px 0;"
            }

            div {
                class: "context-menu-item",
                style: CONTEXT_MENU_ITEM_STYLES,
                onclick: move |_| on_action.call(StashAction::Show(index)),
                div {
                    style: "margin-right: 8px;",
                    "👁"
                }
                "Show Changes"
            }

            div {
                class: "context-menu-separator",
                style: "height: 1px; background: #333; margin: 4px 0;"
            }

            div {
                class: "context-menu-item danger",
                style: format!("{} color: #f48771;", CONTEXT_MENU_ITEM_STYLES),
                onclick: move |_| on_action.call(StashAction::Drop(index)),
                div {
                    style: "margin-right: 8px;",
                    "🗑"
                }
                "Delete Stash"
            }
        }
    }
}

/// Stash creation dialog
#[component]
pub fn StashSaveDialog(
    repo_path: Signal<Option<PathBuf>>,
    is_open: Signal<bool>,
    on_save: EventHandler<StashSaveOptions>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut message = use_signal(|| String::new());
    let mut include_untracked = use_signal(|| true);
    let mut include_ignored = use_signal(|| false);
    let mut keep_index = use_signal(|| false);
    let mut is_saving = use_signal(|| false);

    if !is_open.read().clone() {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "stash-dialog-overlay",
            style: DIALOG_OVERLAY_STYLES,

            div {
                class: "stash-dialog",
                style: STASH_DIALOG_STYLES,
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    class: "dialog-header",
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 16px; border-bottom: 1px solid #333;",

                    h3 {
                        style: "margin: 0; color: #cccccc; font-size: 14px; font-weight: 600;",
                        "Save Stash"
                    }

                    button {
                        class: "dialog-close",
                        style: "background: none; border: none; color: #888; font-size: 16px; cursor: pointer; padding: 4px;",
                        onclick: move |_| on_cancel.call(()),
                        "✕"
                    }
                }

                // Content
                div {
                    class: "dialog-content",
                    style: "padding: 16px;",

                    // Message input
                    div {
                        class: "form-group",
                        style: "margin-bottom: 16px;",

                        label {
                            style: "display: block; margin-bottom: 4px; color: #cccccc; font-size: 12px; font-weight: 500;",
                            "Stash Message"
                        }

                        input {
                            r#type: "text",
                            placeholder: "Optional stash message...",
                            value: "{message.read()}",
                            style: STASH_INPUT_STYLES,
                            oninput: move |evt| message.set(evt.value()),
                            autofocus: true
                        }
                    }

                    // Options
                    div {
                        class: "stash-options",
                        style: "margin-bottom: 16px;",

                        div {
                            class: "checkbox-group",
                            style: "display: flex; align-items: center; margin-bottom: 8px;",

                            input {
                                id: "include-untracked",
                                r#type: "checkbox",
                                checked: include_untracked.read().clone(),
                                style: "margin-right: 8px;",
                                onchange: move |evt| include_untracked.set(evt.checked())
                            }

                            label {
                                r#for: "include-untracked",
                                style: "color: #cccccc; font-size: 12px; cursor: pointer;",
                                "Include untracked files"
                            }
                        }

                        div {
                            class: "checkbox-group",
                            style: "display: flex; align-items: center; margin-bottom: 8px;",

                            input {
                                id: "include-ignored",
                                r#type: "checkbox",
                                checked: include_ignored.read().clone(),
                                style: "margin-right: 8px;",
                                onchange: move |evt| include_ignored.set(evt.checked())
                            }

                            label {
                                r#for: "include-ignored",
                                style: "color: #cccccc; font-size: 12px; cursor: pointer;",
                                "Include ignored files"
                            }
                        }

                        div {
                            class: "checkbox-group",
                            style: "display: flex; align-items: center;",

                            input {
                                id: "keep-index",
                                r#type: "checkbox",
                                checked: keep_index.read().clone(),
                                style: "margin-right: 8px;",
                                onchange: move |evt| keep_index.set(evt.checked())
                            }

                            label {
                                r#for: "keep-index",
                                style: "color: #cccccc; font-size: 12px; cursor: pointer;",
                                "Keep staged changes in index"
                            }
                        }
                    }
                }

                // Footer
                div {
                    class: "dialog-footer",
                    style: "display: flex; justify-content: flex-end; gap: 8px; padding: 16px; border-top: 1px solid #333;",

                    button {
                        class: "btn-secondary",
                        style: SECONDARY_BUTTON_STYLES,
                        onclick: move |_| on_cancel.call(()),
                        disabled: is_saving.read().clone(),
                        "Cancel"
                    }

                    button {
                        class: "btn-primary",
                        style: PRIMARY_BUTTON_STYLES,
                        onclick: move |_| {
                            let opts = StashSaveOptions {
                                message: if message.read().is_empty() { None } else { Some(message.read().clone()) },
                                include_untracked: include_untracked.read().clone(),
                                include_ignored: include_ignored.read().clone(),
                                keep_index: keep_index.read().clone(),
                            };
                            on_save.call(opts);
                        },
                        disabled: is_saving.read().clone(),
                        if is_saving.read().clone() { "Saving..." } else { "Save Stash" }
                    }
                }
            }
        }
    }
}

/// Stash preview panel showing changes
#[component]
pub fn StashPreview(
    repo_path: Signal<Option<PathBuf>>,
    selected_stash: Signal<Option<StashInfo>>,
) -> Element {
    let mut diff_content = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    // Load diff when stash selection changes
    use_effect(move || {
        if let (Some(path), Some(stash)) =
            (repo_path.read().as_ref(), selected_stash.read().as_ref())
        {
            let path = path.clone();
            let index = stash.index;

            spawn(async move {
                loading.set(true);
                error_message.set(None);

                match async_ops::show_stash_diff(&path, index).await {
                    Ok(diff) => {
                        diff_content.set(diff);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to load diff: {}", e)));
                    }
                }

                loading.set(false);
            });
        } else {
            diff_content.set(String::new());
        }
    });

    rsx! {
        div {
            class: "stash-preview",
            style: STASH_PREVIEW_STYLES,

            if let Some(stash) = selected_stash.read().as_ref() {
                // Preview header
                div {
                    class: "preview-header",
                    style: "padding: 12px; border-bottom: 1px solid #333; background: #252526;",

                    div {
                        class: "stash-title",
                        style: "font-weight: 600; color: #cccccc; font-size: 13px; margin-bottom: 4px;",
                        "{stash.message}"
                    }

                    div {
                        class: "stash-meta",
                        style: "font-size: 11px; color: #888; display: flex; gap: 16px;",

                        span { "Stash #{stash.index}" }
                        span { "{stash.formatted_time}" }
                        span { "by {stash.author}" }
                        span { "{stash.stats.files_changed} files" }
                        span {
                            style: "color: #89d185;",
                            "+{stash.stats.insertions}"
                        }
                        span {
                            style: "color: #f48771;",
                            "-{stash.stats.deletions}"
                        }
                    }
                }

                // Preview content
                div {
                    class: "preview-content",
                    style: "flex: 1; overflow: auto;",

                    if loading.read().clone() {
                        div {
                            class: "preview-loading",
                            style: "padding: 24px; text-align: center; color: #888; font-size: 12px;",
                            "Loading diff..."
                        }
                    } else if let Some(error) = error_message.read().as_ref() {
                        div {
                            class: "preview-error",
                            style: "padding: 16px; background: #f48771; color: #1e1e1e; font-size: 12px; margin: 12px;",
                            "{error}"
                        }
                    } else if diff_content.read().is_empty() {
                        div {
                            class: "preview-empty",
                            style: "padding: 24px; text-align: center; color: #888; font-size: 12px;",
                            "No changes to display"
                        }
                    } else {
                        pre {
                            class: "diff-content",
                            style: "margin: 0; padding: 12px; font-family: 'Monaco', 'Consolas', monospace; font-size: 11px; line-height: 1.4; color: #cccccc; white-space: pre-wrap; overflow-wrap: break-word;",
                            "{diff_content.read()}"
                        }
                    }
                }
            } else {
                div {
                    class: "no-selection",
                    style: "display: flex; align-items: center; justify-content: center; height: 100%; color: #888; font-size: 12px;",
                    "Select a stash to view changes"
                }
            }
        }
    }
}

/// Stash action types
#[derive(Debug, Clone, PartialEq)]
pub enum StashAction {
    Save,
    Apply(usize),
    Pop(usize),
    Drop(usize),
    Show(usize),
}

// Styles constants
const STASH_LIST_STYLES: &str = "
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    border: 1px solid #333;
    font-family: 'Segoe UI', system-ui, sans-serif;
";

const STASH_ACTION_BUTTON_STYLES: &str = "
    background: none;
    border: 1px solid #333;
    color: #cccccc;
    padding: 4px 8px;
    cursor: pointer;
    font-size: 12px;
    border-radius: 3px;
    transition: all 0.1s ease;

    &:hover {
        background: #333;
        border-color: #555;
    }
";

const STASH_ITEM_STYLES: &str = "
    padding: 8px 12px;
    border-bottom: 1px solid #333;
    cursor: pointer;
    transition: background 0.1s ease;

    &:hover {
        background: #2a2d2e;
    }

    &:last-child {
        border-bottom: none;
    }
";

const STASH_CONTEXT_MENU_STYLES: &str = "
    position: fixed;
    background: #252526;
    border: 1px solid #333;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 1000;
    min-width: 160px;
    padding: 4px 0;
";

const CONTEXT_MENU_ITEM_STYLES: &str = "
    display: flex;
    align-items: center;
    padding: 8px 12px;
    font-size: 12px;
    color: #cccccc;
    cursor: pointer;
    transition: background 0.1s ease;

    &:hover {
        background: #094771;
    }
";

const DIALOG_OVERLAY_STYLES: &str = "
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
";

const STASH_DIALOG_STYLES: &str = "
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
    width: 480px;
    max-width: 90vw;
    max-height: 80vh;
    overflow: hidden;
";

const STASH_INPUT_STYLES: &str = "
    width: 100%;
    padding: 8px 12px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 3px;
    color: #cccccc;
    font-size: 12px;
    font-family: inherit;

    &:focus {
        outline: none;
        border-color: #007acc;
        box-shadow: 0 0 0 1px #007acc;
    }
";

const PRIMARY_BUTTON_STYLES: &str = "
    background: #007acc;
    border: 1px solid #007acc;
    color: white;
    padding: 8px 16px;
    font-size: 12px;
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.1s ease;

    &:hover:not(:disabled) {
        background: #0e639c;
        border-color: #0e639c;
    }

    &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
";

const SECONDARY_BUTTON_STYLES: &str = "
    background: none;
    border: 1px solid #333;
    color: #cccccc;
    padding: 8px 16px;
    font-size: 12px;
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.1s ease;

    &:hover:not(:disabled) {
        background: #333;
        border-color: #555;
    }

    &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
";

const STASH_PREVIEW_STYLES: &str = "
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    border: 1px solid #333;
    font-family: 'Segoe UI', system-ui, sans-serif;
";
