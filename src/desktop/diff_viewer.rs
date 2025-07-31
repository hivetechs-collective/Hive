//! Diff viewer component
//! 
//! Displays file changes in side-by-side or inline view with syntax highlighting and inline actions

use dioxus::prelude::*;
use crate::desktop::git::{DiffResult, DiffViewMode, DiffLineType, DiffAction, DiffActionState};
use crate::desktop::git::inline_actions::{EnhancedDiffHunk, HunkInlineActions, LineInlineActions};
use crate::desktop::git::keyboard_shortcuts::{GlobalShortcutHandler, ShortcutHelpDialog, FocusManager, ShortcutManager, ShortcutConfig};
use crate::desktop::git::action_processor::{DiffActionProcessor, ExecutionResult};
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, warn, error};

/// Props for the diff viewer component
#[derive(Props, Clone, PartialEq)]
pub struct DiffViewerProps {
    /// The diff result to display
    pub diff: DiffResult,
    /// View mode (side-by-side or inline)
    pub view_mode: DiffViewMode,
    /// File path for syntax highlighting detection
    pub file_path: String,
    /// Repository path
    pub repo_path: String,
    /// Callback when stage/unstage is clicked
    #[props(default)]
    pub on_stage: Option<EventHandler<(usize, bool)>>, // (line_number, is_staged)
    /// Callback when view mode changes
    #[props(default)]
    pub on_view_mode_change: Option<EventHandler<DiffViewMode>>,
    /// Callback when diff action is performed
    #[props(default)]
    pub on_diff_action: Option<EventHandler<DiffAction>>,
    /// Whether inline actions are enabled
    #[props(default = true)]
    pub inline_actions_enabled: bool,
    /// Whether keyboard shortcuts are enabled
    #[props(default = true)]
    pub keyboard_shortcuts_enabled: bool,
}

/// Diff viewer component that shows file changes
#[component]
pub fn DiffViewer(props: DiffViewerProps) -> Element {
    let diff = props.diff;
    let view_mode = props.view_mode;
    
    // Initialize action processor and managers
    let action_processor = use_signal(|| {
        DiffActionProcessor::new(&PathBuf::from(&props.repo_path), PathBuf::from(&props.file_path))
            .unwrap_or_else(|_| {
                // Create a dummy processor if git operations fail
                DiffActionProcessor::new(&PathBuf::from("."), PathBuf::from(&props.file_path))
                    .expect("Failed to create action processor")
            })
    });
    
    let focus_manager = use_signal(|| Arc::new(FocusManager::new()));
    let shortcut_manager = use_signal(|| {
        Arc::new(ShortcutManager::new(ShortcutConfig::default()))
    });
    
    // State for UI feedback
    let show_help = use_signal(|| false);
    let processing_actions = use_signal(|| std::collections::HashSet::<String>::new());
    
    // Provide contexts
    use_context_provider(|| focus_manager());
    use_context_provider(|| shortcut_manager());
    
    // Handle diff actions
    let handle_diff_action = move |action: DiffAction| {
        let action_processor = action_processor().clone();
        let file_path = PathBuf::from(&props.file_path);
        
        // Add to processing set
        let action_id = format!("{:?}", action);
        processing_actions.with_mut(|p| { p.insert(action_id.clone()); });
        
        // Execute action
        spawn(async move {
            match action_processor.execute_immediate(action.clone(), &file_path).await {
                Ok(result) => {
                    if result.success {
                        info!("Action completed successfully: {}", result.message);
                    } else {
                        warn!("Action failed: {}", result.message);
                    }
                },
                Err(e) => {
                    error!("Action execution error: {}", e);
                }
            }
            
            // Remove from processing set
            processing_actions.with_mut(|p| { p.remove(&action_id); });
            
            // Notify parent component
            if let Some(handler) = &props.on_diff_action {
                handler.call(action);
            }
        });
    };
    
    // Register shortcut handlers
    use_effect(move || {
        let manager = shortcut_manager().clone();
        let handler = handle_diff_action.clone();
        
        manager.register_handler("diff_viewer".to_string(), move |shortcut_action| {
            let focus_mgr = focus_manager().clone();
            
            match shortcut_action {
                crate::desktop::git::keyboard_shortcuts::ShortcutAction::Stage => {
                    if let Some(hunk_id) = focus_mgr.get_focused_hunk() {
                        handler(DiffAction::StageHunk(hunk_id));
                    } else if let Some(line_id) = focus_mgr.get_focused_line() {
                        handler(DiffAction::StageLine(line_id));
                    }
                },
                crate::desktop::git::keyboard_shortcuts::ShortcutAction::Unstage => {
                    if let Some(hunk_id) = focus_mgr.get_focused_hunk() {
                        handler(DiffAction::UnstageHunk(hunk_id));
                    } else if let Some(line_id) = focus_mgr.get_focused_line() {
                        handler(DiffAction::UnstageLine(line_id));
                    }
                },
                crate::desktop::git::keyboard_shortcuts::ShortcutAction::Revert => {
                    if let Some(hunk_id) = focus_mgr.get_focused_hunk() {
                        handler(DiffAction::RevertHunk(hunk_id));
                    } else if let Some(line_id) = focus_mgr.get_focused_line() {
                        handler(DiffAction::RevertLine(line_id));
                    }
                },
                crate::desktop::git::keyboard_shortcuts::ShortcutAction::ShowHelp => {
                    show_help.set(true);
                },
                _ => {}
            }
        });
    });
    
    rsx! {
        div {
            class: "diff-viewer",
            style: "width: 100%; height: 100%; overflow: hidden; display: flex; flex-direction: column; background: #1e1e1e; position: relative;",
            
            // Global keyboard shortcut handler
            if props.keyboard_shortcuts_enabled {
                GlobalShortcutHandler {
                    enabled: true,
                }
            }
            
            // Shortcut help dialog
            ShortcutHelpDialog {
                visible: show_help(),
                on_close: move |_| show_help.set(false),
            }
            
            // Diff viewer toolbar
            div {
                class: "diff-toolbar",
                style: "display: flex; align-items: center; justify-content: space-between; padding: 8px; background: #2d2d30; border-bottom: 1px solid #3e3e42;",
                
                div {
                    style: "display: flex; gap: 8px;",
                    
                    button {
                        style: if view_mode == DiffViewMode::SideBySide { 
                            "padding: 4px 12px; background: #094771; color: white; border: none; border-radius: 3px; cursor: pointer;" 
                        } else { 
                            "padding: 4px 12px; background: transparent; color: #cccccc; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer;" 
                        },
                        onclick: move |_| {
                            if let Some(handler) = &props.on_view_mode_change {
                                handler.call(DiffViewMode::SideBySide);
                            }
                        },
                        "Side by Side"
                    }
                    
                    button {
                        style: if view_mode == DiffViewMode::Inline { 
                            "padding: 4px 12px; background: #094771; color: white; border: none; border-radius: 3px; cursor: pointer;" 
                        } else { 
                            "padding: 4px 12px; background: transparent; color: #cccccc; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer;" 
                        },
                        onclick: move |_| {
                            if let Some(handler) = &props.on_view_mode_change {
                                handler.call(DiffViewMode::Inline);
                            }
                        },
                        "Inline"
                    }
                }
                
                div {
                    style: "color: #cccccc; font-size: 12px;",
                    "{diff.hunks.len()} changes"
                }
            }
            
            // Diff content area
            div {
                class: "diff-content",
                style: "flex: 1; overflow: auto; font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace; font-size: 13px;",
                
                if view_mode == DiffViewMode::SideBySide {
                    if props.inline_actions_enabled {
                        EnhancedSideBySideDiff { 
                            diff: diff.clone(),
                            file_path: props.file_path.clone(),
                            repo_path: props.repo_path.clone(),
                            on_diff_action: handle_diff_action,
                        }
                    } else {
                        SideBySideDiff { diff: diff.clone() }
                    }
                } else {
                    if props.inline_actions_enabled {
                        EnhancedInlineDiff { 
                            diff: diff.clone(),
                            file_path: props.file_path.clone(),
                            repo_path: props.repo_path.clone(),
                            on_diff_action: handle_diff_action,
                        }
                    } else {
                        InlineDiff { diff: diff.clone() }
                    }
                }
            }
        }
    }
}

/// Side-by-side diff view (basic)
#[component]
fn SideBySideDiff(diff: DiffResult) -> Element {
    rsx! {
        div {
            style: "display: flex; width: 100%; height: 100%;",
            
            // Original (left side)
            div {
                style: "flex: 1; overflow: auto; border-right: 1px solid #3e3e42;",
                
                div {
                    style: "padding: 8px; background: #252526; color: #d4d4d4; font-size: 11px; font-weight: 600; border-bottom: 1px solid #3e3e42;",
                    "Original"
                }
                
                div {
                    style: "padding: 12px;",
                    
                    for hunk in &diff.hunks {
                        div {
                            class: "diff-hunk",
                            style: "margin-bottom: 16px;",
                            
                            for line in &hunk.lines {
                                if line.line_type == DiffLineType::Deleted || line.line_type == DiffLineType::Unchanged {
                                    div {
                                        style: format!(
                                            "display: flex; min-height: 20px; {}",
                                            if line.line_type == DiffLineType::Deleted {
                                                "background-color: rgba(255, 97, 136, 0.2);"
                                            } else {
                                                ""
                                            }
                                        ),
                                        
                                        // Line number
                                        span {
                                            style: "color: #858585; width: 40px; text-align: right; padding-right: 12px; user-select: none;",
                                            if let Some(num) = line.original_line_number {
                                                "{num}"
                                            }
                                        }
                                        
                                        // Line content
                                        span {
                                            style: "flex: 1; white-space: pre;",
                                            "{line.content}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Modified (right side)
            div {
                style: "flex: 1; overflow: auto;",
                
                div {
                    style: "padding: 8px; background: #252526; color: #d4d4d4; font-size: 11px; font-weight: 600; border-bottom: 1px solid #3e3e42;",
                    "Modified"
                }
                
                div {
                    style: "padding: 12px;",
                    
                    for hunk in &diff.hunks {
                        div {
                            class: "diff-hunk",
                            style: "margin-bottom: 16px;",
                            
                            for line in &hunk.lines {
                                if line.line_type == DiffLineType::Added || line.line_type == DiffLineType::Unchanged {
                                    div {
                                        style: format!(
                                            "display: flex; min-height: 20px; {}",
                                            if line.line_type == DiffLineType::Added {
                                                "background-color: rgba(155, 185, 85, 0.2);"
                                            } else {
                                                ""
                                            }
                                        ),
                                        
                                        // Line number
                                        span {
                                            style: "color: #858585; width: 40px; text-align: right; padding-right: 12px; user-select: none;",
                                            if let Some(num) = line.modified_line_number {
                                                "{num}"
                                            }
                                        }
                                        
                                        // Line content
                                        span {
                                            style: "flex: 1; white-space: pre;",
                                            "{line.content}"
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

/// Inline diff view
#[component]
fn InlineDiff(diff: DiffResult) -> Element {
    rsx! {
        div {
            style: "padding: 12px;",
            
            for hunk in &diff.hunks {
                div {
                    class: "diff-hunk",
                    style: "margin-bottom: 16px;",
                    
                    // Hunk header
                    div {
                        style: "color: #3794ff; background: #2d2d30; padding: 4px 8px; margin-bottom: 8px; font-size: 11px;",
                        "@@ -{hunk.original_start},{hunk.original_count} +{hunk.modified_start},{hunk.modified_count} @@"
                    }
                    
                    for line in &hunk.lines {
                        div {
                            style: format!(
                                "display: flex; min-height: 20px; {}",
                                match line.line_type {
                                    DiffLineType::Added => "background-color: rgba(155, 185, 85, 0.2);",
                                    DiffLineType::Deleted => "background-color: rgba(255, 97, 136, 0.2);",
                                    DiffLineType::Modified => "background-color: rgba(97, 175, 239, 0.2);",
                                    DiffLineType::Unchanged => "",
                                }
                            ),
                            
                            // Line type indicator
                            span {
                                style: format!(
                                    "width: 20px; text-align: center; user-select: none; color: {};",
                                    match line.line_type {
                                        DiffLineType::Added => "#9bb955",
                                        DiffLineType::Deleted => "#ff6188",
                                        _ => "transparent",
                                    }
                                ),
                                match line.line_type {
                                    DiffLineType::Added => "+",
                                    DiffLineType::Deleted => "-",
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
                                style: "flex: 1; white-space: pre;",
                                "{line.content}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Legacy components for backward compatibility
#[component]
fn LegacySideBySideDiff(diff: DiffResult) -> Element {
    // Use the basic side-by-side diff for legacy compatibility
    rsx! {
        SideBySideDiff { diff }
    }
}

#[component]
fn LegacyInlineDiff(diff: DiffResult) -> Element {
    // Use the basic inline diff for legacy compatibility
    rsx! {
        InlineDiff { diff }
    }
}

/// Enhanced inline diff view with action support
#[component]
fn EnhancedInlineDiff(props: EnhancedDiffProps) -> Element {
    rsx! {
        div {
            style: "padding: 12px;",
            
            for hunk in &props.diff.hunks {
                EnhancedDiffHunk {
                    key: "{hunk.hunk_id}",
                    hunk: hunk.clone(),
                    file_path: props.file_path.clone(),
                    repo_path: props.repo_path.clone(),
                    on_hunk_action: props.on_diff_action.clone(),
                    on_line_action: props.on_diff_action.clone(),
                }
            }
        }
    }
}

/// Enhanced side-by-side diff view with action support  
#[component]
fn EnhancedSideBySideDiff(props: EnhancedDiffProps) -> Element {
    rsx! {
        div {
            style: "display: flex; width: 100%; height: 100%;",
            
            // Original (left side)
            div {
                style: "flex: 1; overflow: auto; border-right: 1px solid #3e3e42;",
                
                div {
                    style: "padding: 8px; background: #252526; color: #d4d4d4; font-size: 11px; font-weight: 600; border-bottom: 1px solid #3e3e42;",
                    "Original"
                }
                
                div {
                    style: "padding: 12px;",
                    
                    for hunk in &props.diff.hunks {
                        div {
                            key: "{hunk.hunk_id}",
                            class: "enhanced-side-by-side-hunk",
                            style: "position: relative; margin-bottom: 16px;",
                            
                            // Hunk actions (positioned on left side)
                            HunkInlineActions {
                                hunk: hunk.clone(),
                                file_path: props.file_path.clone(),
                                repo_path: props.repo_path.clone(),
                                on_action: props.on_diff_action.clone(),
                                show_on_hover: true,
                            }
                            
                            for line in &hunk.lines {
                                if line.line_type == DiffLineType::Deleted || line.line_type == DiffLineType::Unchanged {
                                    div {
                                        key: "{line.line_id}_orig",
                                        style: format!(
                                            "position: relative; display: flex; min-height: 20px; {}",
                                            if line.line_type == DiffLineType::Deleted {
                                                "background-color: rgba(255, 97, 136, 0.2);"
                                            } else {
                                                ""
                                            }
                                        ),
                                        
                                        // Line number
                                        span {
                                            style: "color: #858585; width: 40px; text-align: right; padding-right: 12px; user-select: none;",
                                            if let Some(num) = line.original_line_number {
                                                "{num}"
                                            }
                                        }
                                        
                                        // Line content
                                        span {
                                            style: "flex: 1; white-space: pre; padding-right: 60px;",
                                            "{line.content}"
                                        }
                                        
                                        // Line actions for deleted lines
                                        if line.line_type == DiffLineType::Deleted {
                                            LineInlineActions {
                                                line: line.clone(),
                                                file_path: props.file_path.clone(),
                                                repo_path: props.repo_path.clone(),
                                                on_action: props.on_diff_action.clone(),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Modified (right side)
            div {
                style: "flex: 1; overflow: auto;",
                
                div {
                    style: "padding: 8px; background: #252526; color: #d4d4d4; font-size: 11px; font-weight: 600; border-bottom: 1px solid #3e3e42;",
                    "Modified"
                }
                
                div {
                    style: "padding: 12px;",
                    
                    for hunk in &props.diff.hunks {
                        div {
                            key: "{hunk.hunk_id}_mod",
                            class: "enhanced-side-by-side-hunk-right",
                            style: "margin-bottom: 16px;",
                            
                            for line in &hunk.lines {
                                if line.line_type == DiffLineType::Added || line.line_type == DiffLineType::Unchanged {
                                    div {
                                        key: "{line.line_id}_mod",
                                        style: format!(
                                            "position: relative; display: flex; min-height: 20px; {}",
                                            if line.line_type == DiffLineType::Added {
                                                "background-color: rgba(155, 185, 85, 0.2);"
                                            } else {
                                                ""
                                            }
                                        ),
                                        
                                        // Line number
                                        span {
                                            style: "color: #858585; width: 40px; text-align: right; padding-right: 12px; user-select: none;",
                                            if let Some(num) = line.modified_line_number {
                                                "{num}"
                                            }
                                        }
                                        
                                        // Line content
                                        span {
                                            style: "flex: 1; white-space: pre; padding-right: 60px;",
                                            "{line.content}"
                                        }
                                        
                                        // Line actions for added lines
                                        if line.line_type == DiffLineType::Added {
                                            LineInlineActions {
                                                line: line.clone(),
                                                file_path: props.file_path.clone(),
                                                repo_path: props.repo_path.clone(),
                                                on_action: props.on_diff_action.clone(),
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
}

/// Props for enhanced diff components
#[derive(Props, Clone, PartialEq)]
pub struct EnhancedDiffProps {
    /// The diff result to display
    pub diff: DiffResult,
    /// Current file path
    pub file_path: String,
    /// Repository path
    pub repo_path: String,
    /// Callback for diff actions
    pub on_diff_action: EventHandler<DiffAction>,
}