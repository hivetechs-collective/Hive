//! Git operations toolbar component
//! 
//! Provides VS Code-style git operation buttons (commit, push, pull, etc.)

use dioxus::prelude::*;
use std::path::PathBuf;

use super::{GitOperations, GitOperation, GitOperationResult};

/// Props for the git toolbar component
#[derive(Props, Clone, PartialEq)]
pub struct GitToolbarProps {
    /// Current repository path
    pub repo_path: Option<PathBuf>,
    /// Number of staged changes
    pub staged_count: usize,
    /// Number of unstaged changes
    pub unstaged_count: usize,
    /// Current branch name
    pub current_branch: Option<String>,
    /// Callback when a git operation is performed
    pub on_operation: EventHandler<GitOperation>,
}

/// Git operations toolbar component
#[component]
pub fn GitToolbar(props: GitToolbarProps) -> Element {
    let mut commit_message = use_signal(|| String::new());
    let mut show_commit_input = use_signal(|| false);
    let mut operation_status = use_signal(|| None::<String>);
    
    let repo_path = props.repo_path.clone();
    let has_repo = repo_path.is_some();
    let has_staged = props.staged_count > 0;
    let has_unstaged = props.unstaged_count > 0;
    let current_branch = props.current_branch.clone().unwrap_or_else(|| "main".to_string());
    let staged_count = props.staged_count;
    let unstaged_count = props.unstaged_count;
    
    rsx! {
        div {
            class: "git-toolbar",
            style: "padding: 8px; background: #252526; border-bottom: 1px solid #3e3e42;",
            
            // Status message
            if let Some(status) = operation_status.read().as_ref() {
                div {
                    style: "margin-bottom: 8px; padding: 4px 8px; background: #1a472a; color: #4dff4d; border-radius: 3px; font-size: 12px;",
                    "{status}"
                }
            }
            
            // Commit section
            div {
                style: "margin-bottom: 12px;",
                
                // Commit input toggle
                div {
                    style: "display: flex; align-items: center; gap: 8px; margin-bottom: 8px;",
                    
                    button {
                        style: if has_staged {
                            "padding: 6px 12px; background: #0e639c; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 13px;"
                        } else {
                            "padding: 6px 12px; background: #3c3c3c; color: #888; border: none; border-radius: 3px; cursor: not-allowed; font-size: 13px;"
                        },
                        disabled: !has_staged || !has_repo,
                        onclick: move |_| {
                            if has_staged {
                                let current_value = *show_commit_input.read();
                                *show_commit_input.write() = !current_value;
                            }
                        },
                        "✓ Commit ({staged_count})"
                    }
                    
                    if has_unstaged {
                        button {
                            style: "padding: 4px 8px; background: transparent; color: #0e639c; border: 1px solid #0e639c; border-radius: 3px; cursor: pointer; font-size: 11px;",
                            disabled: !has_repo,
                            onclick: {
                                let on_operation = props.on_operation.clone();
                                move |_| {
                                    on_operation.call(GitOperation::StageAll);
                                    *operation_status.write() = Some("Staging all changes...".to_string());
                                }
                            },
                            "+ Stage All"
                        }
                    }
                }
                
                // Commit message input
                if *show_commit_input.read() {
                    div {
                        style: "display: flex; gap: 8px;",
                        
                        input {
                            style: "flex: 1; padding: 6px 8px; background: #3c3c3c; color: white; border: 1px solid #3e3e42; border-radius: 3px; font-size: 13px;",
                            placeholder: "Commit message",
                            value: "{commit_message.read()}",
                            oninput: move |evt| {
                                *commit_message.write() = evt.value();
                            },
                            onkeypress: {
                                let on_operation = props.on_operation.clone();
                                move |evt: dioxus::events::KeyboardEvent| {
                                    if evt.key() == dioxus::events::Key::Enter && !commit_message.read().trim().is_empty() {
                                        let message = commit_message.read().clone();
                                        on_operation.call(GitOperation::Commit(message));
                                        *commit_message.write() = String::new();
                                        *show_commit_input.write() = false;
                                        *operation_status.write() = Some("Creating commit...".to_string());
                                    }
                                }
                            }
                        }
                        
                        button {
                            style: if !commit_message.read().trim().is_empty() {
                                "padding: 6px 12px; background: #0e639c; color: white; border: none; border-radius: 3px; cursor: pointer;"
                            } else {
                                "padding: 6px 12px; background: #3c3c3c; color: #888; border: none; border-radius: 3px; cursor: not-allowed;"
                            },
                            disabled: commit_message.read().trim().is_empty(),
                            onclick: {
                                let on_operation = props.on_operation.clone();
                                move |_| {
                                    let message = commit_message.read().clone();
                                    if !message.trim().is_empty() {
                                        on_operation.call(GitOperation::Commit(message));
                                        *commit_message.write() = String::new();
                                        *show_commit_input.write() = false;
                                        *operation_status.write() = Some("Creating commit...".to_string());
                                    }
                                }
                            },
                            "Commit"
                        }
                        
                        button {
                            style: "padding: 6px 8px; background: transparent; color: #888; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer;",
                            onclick: move |_| {
                                *show_commit_input.write() = false;
                                *commit_message.write() = String::new();
                            },
                            "Cancel"
                        }
                    }
                }
            }
            
            // Remote operations section
            div {
                style: "display: flex; gap: 8px; flex-wrap: wrap;",
                
                // Sync button (VS Code style - pull + push)
                button {
                    style: if has_repo {
                        "padding: 6px 12px; background: #0e639c; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 12px; display: flex; align-items: center; gap: 4px; font-weight: 500;"
                    } else {
                        "padding: 6px 12px; background: #3c3c3c; color: #888; border: 1px solid #3e3e42; border-radius: 3px; cursor: not-allowed; font-size: 12px; display: flex; align-items: center; gap: 4px;"
                    },
                    disabled: !has_repo,
                    title: "Sync Changes (Pull then Push)",
                    onclick: {
                        let on_operation = props.on_operation.clone();
                        let branch = current_branch.clone();
                        move |_| {
                            on_operation.call(GitOperation::Sync);
                            *operation_status.write() = Some(format!("Syncing with origin/{}...", branch));
                        }
                    },
                    "🔄 Sync"
                }
                
                // Push button
                button {
                    style: if has_repo {
                        "padding: 6px 12px; background: transparent; color: #cccccc; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer; font-size: 12px; display: flex; align-items: center; gap: 4px; transition: all 0.2s;"
                    } else {
                        "padding: 6px 12px; background: #3c3c3c; color: #888; border: 1px solid #3e3e42; border-radius: 3px; cursor: not-allowed; font-size: 12px; display: flex; align-items: center; gap: 4px;"
                    },
                    disabled: !has_repo,
                    title: "Push to Remote",
                    onmouseover: move |_| {},
                    onclick: {
                        let on_operation = props.on_operation.clone();
                        let branch = current_branch.clone();
                        move |_| {
                            on_operation.call(GitOperation::Push);
                            *operation_status.write() = Some(format!("Pushing to origin/{}...", branch));
                        }
                    },
                    "⬆ Push"
                }
                
                // Pull button
                button {
                    style: if has_repo {
                        "padding: 6px 12px; background: transparent; color: #cccccc; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer; font-size: 12px; display: flex; align-items: center; gap: 4px; transition: all 0.2s;"
                    } else {
                        "padding: 6px 12px; background: #3c3c3c; color: #888; border: 1px solid #3e3e42; border-radius: 3px; cursor: not-allowed; font-size: 12px; display: flex; align-items: center; gap: 4px;"
                    },
                    disabled: !has_repo,
                    title: "Pull from Remote",
                    onclick: {
                        let on_operation = props.on_operation.clone();
                        let branch = current_branch.clone();
                        move |_| {
                            on_operation.call(GitOperation::Pull);
                            *operation_status.write() = Some(format!("Pulling from origin/{}...", branch));
                        }
                    },
                    "⬇ Pull"
                }
                
                // Fetch button
                button {
                    style: if has_repo {
                        "padding: 6px 12px; background: transparent; color: #cccccc; border: 1px solid #3e3e42; border-radius: 3px; cursor: pointer; font-size: 12px; display: flex; align-items: center; gap: 4px; transition: all 0.2s;"
                    } else {
                        "padding: 6px 12px; background: #3c3c3c; color: #888; border: 1px solid #3e3e42; border-radius: 3px; cursor: not-allowed; font-size: 12px; display: flex; align-items: center; gap: 4px;"
                    },
                    disabled: !has_repo,
                    title: "Fetch from Remote",
                    onclick: {
                        let on_operation = props.on_operation.clone();
                        move |_| {
                            on_operation.call(GitOperation::Fetch);
                            *operation_status.write() = Some("Fetching from origin...".to_string());
                        }
                    },
                    "↻ Fetch"
                }
                
                // Discard all changes button (with confirmation)
                if has_unstaged {
                    button {
                        style: "padding: 6px 12px; background: transparent; color: #f48771; border: 1px solid #f48771; border-radius: 3px; cursor: pointer; font-size: 12px; display: flex; align-items: center; gap: 4px;",
                        onclick: move |_| {
                            // Add confirmation dialog later
                            *operation_status.write() = Some("Discarding all changes...".to_string());
                        },
                        "⟲ Discard All"
                    }
                }
            }
            
            // Branch info
            div {
                style: "margin-top: 12px; padding: 6px 8px; background: #1e1e1e; border-radius: 3px; font-size: 11px; color: #888;",
                "Branch: {current_branch} • Staged: {staged_count} • Unstaged: {unstaged_count}"
            }
        }
    }
}