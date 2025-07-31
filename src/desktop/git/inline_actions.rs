//! Inline diff actions component
//! 
//! Provides VS Code-style inline action buttons for staging, reverting, and managing hunks and lines

use dioxus::prelude::*;
use crate::desktop::git::{DiffHunk, DiffLine, DiffAction, DiffActionResult, DiffGitOperations, HunkStageStatus, DiffLineType};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

/// Props for inline action components
#[derive(Props, Clone, PartialEq)]
pub struct InlineActionsProps {
    /// The hunk for which to show actions
    pub hunk: DiffHunk,
    /// Current file path
    pub file_path: String,
    /// Repository path
    pub repo_path: String,
    /// Whether actions are currently processing
    #[props(default = false)]
    pub is_processing: bool,
    /// Callback when action is triggered
    #[props(default)]
    pub on_action: Option<EventHandler<DiffAction>>,
    /// Whether to show on hover only
    #[props(default = true)]
    pub show_on_hover: bool,
    /// Whether keyboard shortcuts are enabled
    #[props(default = true)]
    pub keyboard_shortcuts: bool,
}

/// Props for line-level actions
#[derive(Props, Clone, PartialEq)]
pub struct LineActionsProps {
    /// The line for which to show actions
    pub line: DiffLine,
    /// Current file path
    pub file_path: String,
    /// Repository path
    pub repo_path: String,
    /// Whether actions are currently processing
    #[props(default = false)]
    pub is_processing: bool,
    /// Callback when action is triggered
    #[props(default)]
    pub on_action: Option<EventHandler<DiffAction>>,
    /// Whether to show on hover only
    #[props(default = true)]
    pub show_on_hover: bool,
}

/// Inline hunk actions component - shows stage/unstage/revert buttons for hunks
#[component]
pub fn HunkInlineActions(props: InlineActionsProps) -> Element {
    let is_hovered = use_signal(|| false);
    let is_visible = !props.show_on_hover || is_hovered();
    
    // Determine available actions based on hunk status
    let can_stage = !props.hunk.is_staged && props.hunk.is_stageable;
    let can_unstage = props.hunk.is_staged;
    let can_revert = props.hunk.stage_status != HunkStageStatus::Conflicted;
    
    rsx! {
        div {
            class: "hunk-inline-actions",
            style: format!(
                "position: absolute; right: 8px; top: 4px; display: {}; gap: 4px; z-index: 10;",
                if is_visible || props.is_processing { "flex" } else { "none" }
            ),
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),
            
            // Processing indicator
            if props.is_processing {
                div {
                    class: "processing-indicator",
                    style: "padding: 2px 6px; background: rgba(0, 122, 204, 0.8); color: white; font-size: 10px; border-radius: 3px; animation: pulse 1.5s infinite;",
                    "Processing..."
                }
            }
            
            // Stage hunk button
            if can_stage && !props.is_processing {
                button {
                    class: "action-button stage-hunk",
                    style: "padding: 2px 6px; background: #238636; color: white; border: none; border-radius: 3px; font-size: 10px; cursor: pointer; display: flex; align-items: center; gap: 2px;",
                    title: "Stage hunk (Alt+S)",
                    onclick: move |_| {
                        if let Some(handler) = &props.on_action {
                            handler.call(DiffAction::StageHunk(props.hunk.hunk_id.clone()));
                        }
                    },
                    onmouseenter: move |_| {
                        // Add subtle highlight effect
                    },
                    
                    // Stage icon (plus symbol)
                    span {
                        style: "font-weight: bold; font-size: 8px;",
                        "+"
                    }
                    "Stage"
                }
            }
            
            // Unstage hunk button
            if can_unstage && !props.is_processing {
                button {
                    class: "action-button unstage-hunk",
                    style: "padding: 2px 6px; background: #f85149; color: white; border: none; border-radius: 3px; font-size: 10px; cursor: pointer; display: flex; align-items: center; gap: 2px;",
                    title: "Unstage hunk (Alt+U)",
                    onclick: move |_| {
                        if let Some(handler) = &props.on_action {
                            handler.call(DiffAction::UnstageHunk(props.hunk.hunk_id.clone()));
                        }
                    },
                    
                    // Unstage icon (minus symbol)
                    span {
                        style: "font-weight: bold; font-size: 8px;",
                        "−"
                    }
                    "Unstage"
                }
            }
            
            // Revert hunk button
            if can_revert && !props.is_processing {
                button {
                    class: "action-button revert-hunk",
                    style: "padding: 2px 6px; background: #fd7e14; color: white; border: none; border-radius: 3px; font-size: 10px; cursor: pointer; display: flex; align-items: center; gap: 2px;",
                    title: "Revert hunk changes (Alt+R)",
                    onclick: move |_| {
                        if let Some(handler) = &props.on_action {
                            handler.call(DiffAction::RevertHunk(props.hunk.hunk_id.clone()));
                        }
                    },
                    
                    // Revert icon (undo symbol)
                    span {
                        style: "font-weight: bold; font-size: 8px;",
                        "↶"
                    }
                    "Revert"
                }
            }
            
            // Hunk status indicator
            div {
                class: "hunk-status",
                style: format!(
                    "padding: 2px 6px; font-size: 10px; border-radius: 3px; {}",
                    match props.hunk.stage_status {
                        HunkStageStatus::Staged => "background: rgba(35, 134, 54, 0.2); color: #238636;",
                        HunkStageStatus::PartiallyStaged => "background: rgba(253, 126, 20, 0.2); color: #fd7e14;",
                        HunkStageStatus::Conflicted => "background: rgba(248, 81, 73, 0.2); color: #f85149;",
                        HunkStageStatus::Unstaged => "background: rgba(139, 148, 158, 0.2); color: #8b949e;",
                    }
                ),
                match props.hunk.stage_status {
                    HunkStageStatus::Staged => "Staged",
                    HunkStageStatus::PartiallyStaged => "Partial",
                    HunkStageStatus::Conflicted => "Conflict",
                    HunkStageStatus::Unstaged => "Unstaged",
                }
            }
        }
    }
}

/// Inline line actions component - shows stage/unstage buttons for individual lines
#[component]  
pub fn LineInlineActions(props: LineActionsProps) -> Element {
    let is_hovered = use_signal(|| false);
    let is_visible = !props.show_on_hover || is_hovered();
    
    // Only show for stageable lines (added/deleted, not unchanged)
    let is_stageable = props.line.is_stageable && 
        matches!(props.line.line_type, crate::desktop::git::DiffLineType::Added | crate::desktop::git::DiffLineType::Deleted);
    
    if !is_stageable {
        return rsx! { div {} };
    }
    
    let can_stage = !props.line.is_staged;
    let can_unstage = props.line.is_staged;
    
    rsx! {
        div {
            class: "line-inline-actions",
            style: format!(
                "position: absolute; right: 8px; display: {}; gap: 2px; z-index: 10;",
                if is_visible || props.is_processing { "flex" } else { "none" }
            ),
            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),
            
            // Processing indicator
            if props.is_processing {
                div {
                    class: "processing-indicator",
                    style: "width: 8px; height: 8px; background: #007acc; border-radius: 50%; animation: pulse 1.5s infinite;",
                }
            }
            
            // Stage line button
            if can_stage && !props.is_processing {
                button {
                    class: "action-button stage-line",
                    style: "width: 16px; height: 16px; background: #238636; color: white; border: none; border-radius: 2px; font-size: 8px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                    title: "Stage line",
                    onclick: move |_| {
                        if let Some(handler) = &props.on_action {
                            handler.call(DiffAction::StageLine(props.line.line_id.clone()));
                        }
                    },
                    
                    "+"
                }
            }
            
            // Unstage line button
            if can_unstage && !props.is_processing {
                button {
                    class: "action-button unstage-line",
                    style: "width: 16px; height: 16px; background: #f85149; color: white; border: none; border-radius: 2px; font-size: 8px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                    title: "Unstage line",
                    onclick: move |_| {
                        if let Some(handler) = &props.on_action {
                            handler.call(DiffAction::UnstageLine(props.line.line_id.clone()));
                        }
                    },
                    
                    "−"
                }
            }
        }
    }
}

/// Enhanced hunk component with inline actions support
#[component]
pub fn EnhancedDiffHunk(props: EnhancedDiffHunkProps) -> Element {
    let processing_lines = use_signal(|| std::collections::HashSet::<String>::new());
    let processing_hunk = use_signal(|| false);
    
    rsx! {
        div {
            class: "enhanced-diff-hunk",
            style: "position: relative; margin-bottom: 16px;",
            
            // Hunk header with inline actions
            div {
                class: "hunk-header",
                style: "position: relative; color: #3794ff; background: #2d2d30; padding: 4px 8px; margin-bottom: 8px; font-size: 11px;",
                onmouseenter: move |_| {
                    // Show hunk actions on hover
                },
                
                "@@ -{props.hunk.original_start},{props.hunk.original_count} +{props.hunk.modified_start},{props.hunk.modified_count} @@"
                
                // Hunk inline actions
                HunkInlineActions {
                    hunk: props.hunk.clone(),
                    file_path: props.file_path.clone(),
                    repo_path: props.repo_path.clone(),
                    is_processing: processing_hunk(),
                    on_action: move |action| {
                        processing_hunk.set(true);
                        if let Some(handler) = &props.on_hunk_action {
                            handler.call(action);
                        }
                        // Reset processing state after a delay
                        spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                            processing_hunk.set(false);
                        });
                    },
                }
            }
            
            // Hunk lines with line-level actions
            for line in &props.hunk.lines {
                div {
                    key: "{line.line_id}",
                    class: "enhanced-diff-line",
                    style: format!(
                        "position: relative; display: flex; min-height: 20px; {}",
                        match line.line_type {
                            crate::desktop::git::DiffLineType::Added => "background-color: rgba(155, 185, 85, 0.2);",
                            crate::desktop::git::DiffLineType::Deleted => "background-color: rgba(255, 97, 136, 0.2);",
                            crate::desktop::git::DiffLineType::Modified => "background-color: rgba(97, 175, 239, 0.2);",
                            crate::desktop::git::DiffLineType::Unchanged => "",
                        }
                    ),
                    onmouseenter: move |_| {
                        // Show line actions on hover
                    },
                    
                    // Line type indicator
                    span {
                        style: format!(
                            "width: 20px; text-align: center; user-select: none; color: {};",
                            match line.line_type {
                                crate::desktop::git::DiffLineType::Added => "#9bb955",
                                crate::desktop::git::DiffLineType::Deleted => "#ff6188",
                                _ => "transparent",
                            }
                        ),
                        match line.line_type {
                            crate::desktop::git::DiffLineType::Added => "+",
                            crate::desktop::git::DiffLineType::Deleted => "-",
                            _ => " ",
                        }
                    }
                    
                    // Line numbers
                    span {
                        style: "color: #858585; width: 40px; text-align: right; padding-right: 8px; user-select: none;",
                        if let Some(num) = line.original_line_number {
                            "{num}"
                        } else {
                            ""
                        }
                    }
                    
                    span {
                        style: "color: #858585; width: 40px; text-align: right; padding-right: 12px; user-select: none;",
                        if let Some(num) = line.modified_line_number {
                            "{num}"
                        } else {
                            ""
                        }
                    }
                    
                    // Line content
                    span {
                        style: "flex: 1; white-space: pre; padding-right: 60px;", // Extra padding for action buttons
                        "{line.content}"
                    }
                    
                    // Line inline actions
                    LineInlineActions {
                        line: line.clone(),
                        file_path: props.file_path.clone(),
                        repo_path: props.repo_path.clone(),
                        is_processing: processing_lines().contains(&line.line_id),
                        on_action: move |action| {
                            let line_id = line.line_id.clone();
                            processing_lines.with_mut(|lines| {
                                lines.insert(line_id.clone());
                            });
                            
                            if let Some(handler) = &props.on_line_action {
                                handler.call(action);
                            }
                            
                            // Reset processing state after a delay
                            spawn(async move {
                                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                processing_lines.with_mut(|lines| {
                                    lines.remove(&line_id);
                                });
                            });
                        },
                    }
                }
            }
        }
    }
}

/// Props for enhanced diff hunk component
#[derive(Props, Clone, PartialEq)]
pub struct EnhancedDiffHunkProps {
    /// The hunk to display
    pub hunk: DiffHunk,
    /// Current file path
    pub file_path: String,
    /// Repository path
    pub repo_path: String,
    /// Callback when hunk action is triggered
    #[props(default)]
    pub on_hunk_action: Option<EventHandler<DiffAction>>,
    /// Callback when line action is triggered
    #[props(default)]
    pub on_line_action: Option<EventHandler<DiffAction>>,
}

/// Keyboard shortcut handler for diff actions (simplified for desktop)
#[component]
pub fn DiffKeyboardHandler(props: DiffKeyboardHandlerProps) -> Element {
    // For desktop apps, we might handle this differently
    // This is a placeholder implementation
    rsx! {
        // This component handles keyboard shortcuts globally
        div { 
            style: "display: none;",
            // Desktop keyboard handling would be implemented here
        }
    }
}

/// Props for keyboard shortcut handler
#[derive(Props, Clone, PartialEq)]
pub struct DiffKeyboardHandlerProps {
    /// Whether keyboard shortcuts are enabled
    #[props(default = true)]
    pub enabled: bool,
    /// Callback for Alt+S (stage)
    #[props(default)]
    pub on_stage_shortcut: Option<EventHandler<()>>,
    /// Callback for Alt+U (unstage)
    #[props(default)]
    pub on_unstage_shortcut: Option<EventHandler<()>>,
    /// Callback for Alt+R (revert)
    #[props(default)]
    pub on_revert_shortcut: Option<EventHandler<()>>,
}

/// VS Code-style action tooltip component
#[component]
pub fn ActionTooltip(props: ActionTooltipProps) -> Element {
    rsx! {
        div {
            class: "action-tooltip",
            style: format!(
                "position: absolute; bottom: 100%; left: 50%; transform: translateX(-50%); \
                 background: #1e1e1e; color: #cccccc; padding: 4px 8px; border-radius: 3px; \
                 font-size: 11px; white-space: nowrap; z-index: 1000; \
                 border: 1px solid #3e3e42; box-shadow: 0 2px 8px rgba(0,0,0,0.3); \
                 display: {};",
                if props.visible { "block" } else { "none" }
            ),
            
            "{props.text}"
            
            // Tooltip arrow
            div {
                style: "position: absolute; top: 100%; left: 50%; transform: translateX(-50%); \
                       width: 0; height: 0; border-left: 4px solid transparent; \
                       border-right: 4px solid transparent; border-top: 4px solid #3e3e42;",
            }
        }
    }
}

/// Props for action tooltip
#[derive(Props, Clone, PartialEq)]
pub struct ActionTooltipProps {
    /// Tooltip text
    pub text: String,
    /// Whether tooltip is visible
    pub visible: bool,
}