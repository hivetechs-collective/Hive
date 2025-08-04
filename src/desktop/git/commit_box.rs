//! Enhanced commit box component for git operations
//! 
//! Provides a VS Code-style commit interface with:
//! - Multi-line textarea
//! - Character counter with visual warnings
//! - Commit templates
//! - Conventional commit type buttons
//! - Ctrl+Enter support

use dioxus::prelude::*;
use dioxus::events::{MouseData, KeyboardData};
use std::fmt::Write as _;

/// Commit template types
#[derive(Debug, Clone, PartialEq)]
pub enum CommitTemplate {
    None,
    BugFix,
    Feature,
    Refactor,
    Documentation,
}

impl CommitTemplate {
    fn template_text(&self) -> &'static str {
        match self {
            CommitTemplate::None => "",
            CommitTemplate::BugFix => "Fix: \n\n- Fixed issue where \n- Resolved problem with \n\nCloses #",
            CommitTemplate::Feature => "Feat: \n\n- Added support for \n- Implemented \n\nRelated to #",
            CommitTemplate::Refactor => "Refactor: \n\n- Simplified \n- Improved performance of \n- Cleaned up ",
            CommitTemplate::Documentation => "Docs: \n\n- Updated documentation for \n- Added examples for \n- Clarified ",
        }
    }
    
    fn display_name(&self) -> &'static str {
        match self {
            CommitTemplate::None => "No Template",
            CommitTemplate::BugFix => "Bug Fix",
            CommitTemplate::Feature => "Feature",
            CommitTemplate::Refactor => "Refactor",
            CommitTemplate::Documentation => "Documentation",
        }
    }
}

/// Conventional commit types
#[derive(Debug, Clone, PartialEq)]
pub enum ConventionalType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Test,
    Chore,
}

impl ConventionalType {
    fn as_str(&self) -> &'static str {
        match self {
            ConventionalType::Feat => "feat",
            ConventionalType::Fix => "fix",
            ConventionalType::Docs => "docs",
            ConventionalType::Style => "style",
            ConventionalType::Refactor => "refactor",
            ConventionalType::Test => "test",
            ConventionalType::Chore => "chore",
        }
    }
    
    fn tooltip(&self) -> &'static str {
        match self {
            ConventionalType::Feat => "A new feature",
            ConventionalType::Fix => "A bug fix",
            ConventionalType::Docs => "Documentation changes",
            ConventionalType::Style => "Code style changes (formatting, etc)",
            ConventionalType::Refactor => "Code refactoring without changing functionality",
            ConventionalType::Test => "Adding or updating tests",
            ConventionalType::Chore => "Maintenance tasks, dependency updates, etc",
        }
    }
}

/// Props for the commit box component
#[derive(Props, Clone, PartialEq)]
pub struct CommitBoxProps {
    /// Whether the commit box is visible
    pub visible: bool,
    /// Number of staged files
    pub staged_count: usize,
    /// Callback when commit is submitted
    pub on_commit: EventHandler<String>,
    /// Callback when commit box is closed
    pub on_close: EventHandler<()>,
}

/// Enhanced commit box component
#[component]
pub fn CommitBox(props: CommitBoxProps) -> Element {
    if !props.visible {
        return rsx! { div {} };
    }
    
    // State
    let mut commit_message = use_signal(|| String::new());
    let mut selected_template = use_signal(|| CommitTemplate::None);
    let mut show_template_dropdown = use_signal(|| false);
    
    // Parse commit message into subject and body
    let message = commit_message.read();
    let lines: Vec<&str> = message.lines().collect();
    let subject_line = lines.first().unwrap_or(&"");
    let subject_length = subject_line.len();
    let body_lines: Vec<&str> = if lines.len() > 2 { lines[2..].iter().copied().collect() } else { vec![] };
    let body_text = body_lines.join("\n");
    let body_length = body_text.len();
    
    // Character counter colors
    let subject_color = if subject_length <= 50 {
        "#4ec9b0" // Green
    } else if subject_length <= 72 {
        "#dcdcaa" // Yellow
    } else {
        "#f48771" // Red
    };
    
    // Warning indicators
    let show_50_warning = subject_length > 50;
    let show_72_warning = subject_length > 72;
    
    // Note: Removed on_template_select closure - will be inlined in the event handlers
    
    // Note: Removed apply_conventional_type closure - will be inlined in the event handlers
    
    // Note: Removed handle_commit closure - will be inlined in the event handlers
    
    rsx! {
        div {
            class: "commit-box-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); z-index: 1000; display: flex; align-items: center; justify-content: center;",
            onclick: {
                let on_close = props.on_close.clone();
                move |_e: Event<dioxus::events::MouseData>| {
                    // Close when clicking outside
                    // Note: In newer Dioxus versions, target() and current_target() are not available
                    // For overlay clicks, we'll handle this differently or skip the check
                    on_close.call(());
                }
            },
            
            div {
                class: "commit-box",
                style: "background: #252526; border: 1px solid #3e3e42; border-radius: 6px; padding: 20px; width: 600px; max-width: 90vw; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);",
                onclick: move |e: Event<dioxus::events::MouseData>| {
                    // Prevent clicks inside the commit box from closing it
                    e.stop_propagation();
                },
                
                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                    
                    h3 {
                        style: "margin: 0; color: #cccccc; font-size: 16px; font-weight: 500;",
                        "Commit Changes ({props.staged_count} files staged)"
                    }
                    
                    button {
                        style: "background: transparent; border: none; color: #888; font-size: 18px; cursor: pointer; padding: 4px 8px;",
                        onclick: {
                            let on_close = props.on_close.clone();
                            move |_| on_close.call(())
                        },
                        "×"
                    }
                }
                
                // Template selector
                div {
                    style: "margin-bottom: 12px; position: relative;",
                    
                    button {
                        style: "width: 100%; padding: 8px 12px; background: #3c3c3c; color: #cccccc; border: 1px solid #3e3e42; border-radius: 4px; text-align: left; cursor: pointer; font-size: 13px; display: flex; justify-content: space-between; align-items: center;",
                        onclick: move |_| {
                            let current = *show_template_dropdown.read();
                            *show_template_dropdown.write() = !current;
                        },
                        
                        span { "{selected_template.read().display_name()}" }
                        span { style: "font-size: 10px;", "▼" }
                    }
                    
                    // Dropdown menu
                    if *show_template_dropdown.read() {
                        div {
                            style: "position: absolute; top: 100%; left: 0; right: 0; background: #1e1e1e; border: 1px solid #3e3e42; border-radius: 4px; margin-top: 4px; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);",
                            
                            for template in [CommitTemplate::None, CommitTemplate::BugFix, CommitTemplate::Feature, CommitTemplate::Refactor, CommitTemplate::Documentation] {
                                button {
                                    style: "width: 100%; padding: 8px 12px; background: transparent; color: #cccccc; border: none; text-align: left; cursor: pointer; font-size: 13px;",
                                    onmouseover: move |_e| {
                                        // Note: Direct element manipulation removed in newer Dioxus
                                        // Would need to use state-based styling instead
                                    },
                                    onmouseout: move |_e| {
                                        // Note: Direct element manipulation removed in newer Dioxus
                                        // Would need to use state-based styling instead
                                    },
                                    onclick: {
                                        let template = template.clone();
                                        let mut commit_message = commit_message.clone();
                                        let mut selected_template = selected_template.clone();
                                        let mut show_template_dropdown = show_template_dropdown.clone();
                                        move |_| {
                                            if template != CommitTemplate::None {
                                                commit_message.set(template.template_text().to_string());
                                            }
                                            selected_template.set(template.clone());
                                            show_template_dropdown.set(false);
                                        }
                                    },
                                    "{template.display_name()}"
                                }
                            }
                        }
                    }
                }
                
                // Conventional commit type buttons
                div {
                    style: "display: flex; gap: 6px; margin-bottom: 12px; flex-wrap: wrap;",
                    
                    for commit_type in [ConventionalType::Feat, ConventionalType::Fix, ConventionalType::Docs, ConventionalType::Style, ConventionalType::Refactor, ConventionalType::Test, ConventionalType::Chore] {
                        button {
                            style: "padding: 4px 10px; background: #3c3c3c; color: #888; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer; font-size: 12px; transition: all 0.2s;",
                            title: commit_type.tooltip(),
                            onmouseover: move |_e| {
                                // Note: Direct element manipulation removed in newer Dioxus
                                // Would need to use state-based styling instead
                            },
                            onmouseout: move |_e| {
                                // Note: Direct element manipulation removed in newer Dioxus
                                // Would need to use state-based styling instead
                            },
                            onclick: {
                                let commit_type = commit_type.clone();
                                let mut commit_message = commit_message.clone();
                                move |_| {
                                    let current_message = commit_message.read().clone();
                                    let type_prefix = format!("{}: ", commit_type.as_str());
                                    
                                    // Check if message already starts with a conventional type
                                    let conventional_types = ["feat:", "fix:", "docs:", "style:", "refactor:", "test:", "chore:"];
                                    let mut new_message = current_message.clone();
                                    
                                    // Remove existing type if present
                                    for existing_type in &conventional_types {
                                        if current_message.to_lowercase().starts_with(existing_type) {
                                            new_message = current_message[existing_type.len()..].trim_start().to_string();
                                            break;
                                        }
                                    }
                                    
                                    // Add new type
                                    commit_message.set(format!("{}{}", type_prefix, new_message));
                                }
                            },
                            "{commit_type.as_str()}"
                        }
                    }
                }
                
                // Commit message textarea
                div {
                    style: "position: relative; margin-bottom: 12px;",
                    
                    textarea {
                        style: "width: 100%; min-height: 200px; padding: 12px; background: #1e1e1e; color: #cccccc; border: 1px solid #3e3e42; border-radius: 4px; font-family: 'Consolas', 'Monaco', monospace; font-size: 14px; line-height: 1.5; resize: vertical;",
                        placeholder: "Enter commit message...\n\nDetailed description (optional)",
                        value: "{commit_message.read()}",
                        oninput: move |evt| {
                            *commit_message.write() = evt.value();
                        },
                        onkeydown: {
                            let mut commit_message = commit_message.clone();
                            let on_commit = props.on_commit.clone();
                            let on_close = props.on_close.clone();
                            move |evt: Event<dioxus::events::KeyboardData>| {
                                if evt.modifiers().ctrl() && evt.key() == dioxus::events::Key::Enter {
                                    evt.stop_propagation();
                                    let message = commit_message.read().trim().to_string();
                                    if !message.is_empty() {
                                        on_commit.call(message);
                                        commit_message.set(String::new());
                                        on_close.call(());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Warning indicators
                    if show_50_warning || show_72_warning {
                        div {
                            style: "position: absolute; top: 12px; right: 12px; display: flex; gap: 8px;",
                            
                            if show_50_warning && !show_72_warning {
                                div {
                                    style: "background: #dcdcaa; color: #1e1e1e; padding: 2px 6px; border-radius: 3px; font-size: 11px; font-weight: 500;",
                                    title: "Subject line should be 50 characters or less",
                                    "50"
                                }
                            }
                            
                            if show_72_warning {
                                div {
                                    style: "background: #f48771; color: #1e1e1e; padding: 2px 6px; border-radius: 3px; font-size: 11px; font-weight: 500;",
                                    title: "Subject line must not exceed 72 characters",
                                    "72"
                                }
                            }
                        }
                    }
                }
                
                // Character counter
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; font-size: 12px;",
                    
                    div {
                        style: "display: flex; gap: 16px;",
                        
                        span {
                            style: format!("color: {};", subject_color),
                            "Subject: {subject_length}/50"
                        }
                        
                        span {
                            style: "color: #888;",
                            "Body: {body_length}/∞"
                        }
                    }
                    
                    span {
                        style: "color: #666; font-style: italic;",
                        "Press Ctrl+Enter to commit"
                    }
                }
                
                // Actions
                div {
                    style: "display: flex; justify-content: flex-end; gap: 8px;",
                    
                    button {
                        style: "padding: 8px 16px; background: transparent; color: #888; border: 1px solid #3e3e42; border-radius: 4px; cursor: pointer; font-size: 13px;",
                        onclick: {
                            let on_close = props.on_close.clone();
                            move |_| on_close.call(())
                        },
                        "Cancel"
                    }
                    
                    button {
                        style: if !commit_message.read().trim().is_empty() {
                            "padding: 8px 16px; background: #0e639c; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: 500;"
                        } else {
                            "padding: 8px 16px; background: #3c3c3c; color: #666; border: none; border-radius: 4px; cursor: not-allowed; font-size: 13px;"
                        },
                        disabled: commit_message.read().trim().is_empty(),
                        onclick: {
                            let mut commit_message = commit_message.clone();
                            let on_commit = props.on_commit.clone();
                            let on_close = props.on_close.clone();
                            move |_| {
                                let message = commit_message.read().trim().to_string();
                                if !message.is_empty() {
                                    on_commit.call(message);
                                    commit_message.set(String::new());
                                    on_close.call(());
                                }
                            }
                        },
                        "Commit"
                    }
                }
            }
        }
    }
}