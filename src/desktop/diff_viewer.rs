//! Diff viewer component
//! 
//! Displays file changes in side-by-side or inline view with syntax highlighting

use dioxus::prelude::*;
use crate::desktop::git::{DiffResult, DiffViewMode, DiffLineType};

/// Props for the diff viewer component
#[derive(Props, Clone, PartialEq)]
pub struct DiffViewerProps {
    /// The diff result to display
    pub diff: DiffResult,
    /// View mode (side-by-side or inline)
    pub view_mode: DiffViewMode,
    /// File path for syntax highlighting detection
    pub file_path: String,
    /// Callback when stage/unstage is clicked
    #[props(default)]
    pub on_stage: Option<EventHandler<(usize, bool)>>, // (line_number, is_staged)
}

/// Diff viewer component that shows file changes
#[component]
pub fn DiffViewer(props: DiffViewerProps) -> Element {
    let diff = props.diff;
    let view_mode = props.view_mode;
    
    rsx! {
        div {
            class: "diff-viewer",
            style: "width: 100%; height: 100%; overflow: hidden; display: flex; flex-direction: column; background: #1e1e1e;",
            
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
                        "Side by Side"
                    }
                    
                    button {
                        style: if view_mode == DiffViewMode::Inline { 
                            "padding: 4px 12px; background: #094771; color: white; border: none; border-radius: 3px; cursor: pointer;" 
                        } else { 
                            "padding: 4px 12px; background: transparent; color: #cccccc; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer;" 
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
                    SideBySideDiff { diff: diff.clone() }
                } else {
                    InlineDiff { diff: diff.clone() }
                }
            }
        }
    }
}

/// Side-by-side diff view
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