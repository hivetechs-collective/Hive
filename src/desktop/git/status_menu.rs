//! Git status menu component for branch switching
//!
//! Provides a dropdown menu for git branch operations

use super::{BranchInfo, BranchType, GitRepository};
use dioxus::prelude::*;
use std::path::PathBuf;

/// Props for the git status menu component
#[derive(Props, Clone, PartialEq)]
pub struct GitStatusMenuProps {
    /// Current repository path
    pub repo_path: Option<PathBuf>,
    /// Current branch info
    pub branch_info: Option<BranchInfo>,
    /// Visibility state
    pub visible: Signal<bool>,
    /// Position for the menu (x, y)
    pub position: (i32, i32),
    /// Callback when a branch is selected
    pub on_branch_selected: EventHandler<String>,
    /// Callback when create branch is selected
    pub on_create_branch: EventHandler<()>,
}

/// Git status menu component
#[component]
pub fn GitStatusMenu(props: GitStatusMenuProps) -> Element {
    let mut branches = use_signal(|| Vec::<BranchInfo>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    // Load branches when menu becomes visible
    use_effect(move || {
        if *props.visible.read() {
            loading.set(true);
            error.set(None);

            if let Some(repo_path) = props.repo_path.clone() {
                spawn(async move {
                    match GitRepository::open(&repo_path) {
                        Ok(repo) => match repo.list_branches() {
                            Ok(branch_list) => {
                                branches.set(branch_list);
                                loading.set(false);
                            }
                            Err(e) => {
                                error.set(Some(format!("Failed to list branches: {}", e)));
                                loading.set(false);
                            }
                        },
                        Err(e) => {
                            error.set(Some(format!("Failed to open repository: {}", e)));
                            loading.set(false);
                        }
                    }
                });
            } else {
                error.set(Some("No repository path provided".to_string()));
                loading.set(false);
            }
        }
    });

    if !*props.visible.read() {
        return rsx! { div {} };
    }

    let current_branch = props.branch_info.as_ref().map(|b| &b.name);
    let (x, y) = props.position;

    rsx! {
        // Backdrop to capture clicks outside
        div {
            class: "git-status-menu-backdrop",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
            onclick: {
                let mut visible = props.visible.clone();
                move |_| {
                    visible.set(false);
                }
            }
        }

        // Menu container
        div {
            class: "git-status-menu",
            style: format!(
                "position: fixed; left: {}px; bottom: {}px; background: #252526; border: 1px solid #3e3e42; \
                 border-radius: 6px; box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4); \
                 min-width: 250px; max-width: 400px; max-height: 400px; overflow: hidden; \
                 z-index: 1000; font-size: 13px;",
                x, y + 30
            ),
            onclick: move |evt| {
                // Prevent closing when clicking inside menu
                evt.stop_propagation();
            },

            // Menu header
            div {
                class: "menu-header",
                style: "padding: 10px 12px; border-bottom: 1px solid #3e3e42; font-weight: 600; color: #cccccc;",
                "Git Branches"
            }

            // Menu content
            div {
                class: "menu-content",
                style: "max-height: 300px; overflow-y: auto;",

                if *loading.read() {
                    div {
                        style: "padding: 20px; text-align: center; color: #808080;",
                        "Loading branches..."
                    }
                } else if let Some(err) = error.read().as_ref() {
                    div {
                        style: "padding: 20px; color: #f48771;",
                        "{err}"
                    }
                } else {
                    // Local branches section
                    div {
                        class: "branch-section",
                        div {
                            class: "section-header",
                            style: "padding: 8px 12px; font-size: 11px; color: #808080; text-transform: uppercase;",
                            "Local Branches"
                        }

                        for branch in branches.read().iter().filter(|b| b.branch_type == BranchType::Local) {
                            BranchMenuItem {
                                branch: branch.clone(),
                                is_current: Some(&branch.name) == current_branch,
                                on_click: {
                                    let branch_name = branch.name.clone();
                                    let on_branch_selected = props.on_branch_selected.clone();
                                    let mut visible = props.visible.clone();
                                    move |_| {
                                        on_branch_selected.call(branch_name.clone());
                                        visible.set(false);
                                    }
                                }
                            }
                        }
                    }

                    // Remote branches section
                    if branches.read().iter().any(|b| b.branch_type == BranchType::Remote) {
                        div {
                            class: "branch-section",
                            div {
                                class: "section-header",
                                style: "padding: 8px 12px; font-size: 11px; color: #808080; text-transform: uppercase; border-top: 1px solid #3e3e42;",
                                "Remote Branches"
                            }

                            for branch in branches.read().iter().filter(|b| b.branch_type == BranchType::Remote) {
                                BranchMenuItem {
                                    branch: branch.clone(),
                                    is_current: false,
                                    on_click: {
                                        let branch_name = branch.name.clone();
                                        let on_branch_selected = props.on_branch_selected.clone();
                                        let mut visible = props.visible.clone();
                                        move |_| {
                                            on_branch_selected.call(branch_name.clone());
                                            visible.set(false);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Menu footer with actions
            div {
                class: "menu-footer",
                style: "border-top: 1px solid #3e3e42; padding: 8px;",

                button {
                    class: "menu-action-button",
                    style: "width: 100%; padding: 6px 12px; background: #0e639c; color: white; \
                            border: none; border-radius: 3px; cursor: pointer; font-size: 13px; \
                            transition: background 0.2s;",
                    // Hover effects would go here
                    onclick: {
                        let on_create_branch = props.on_create_branch.clone();
                        let mut visible = props.visible.clone();
                        move |_| {
                            on_create_branch.call(());
                            visible.set(false);
                        }
                    },
                    "✚ Create New Branch"
                }
            }
        }
    }
}

/// Individual branch menu item component
#[component]
fn BranchMenuItem(branch: BranchInfo, is_current: bool, on_click: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "branch-menu-item",
            style: format!(
                "padding: 8px 12px; cursor: pointer; display: flex; align-items: center; gap: 8px; \
                 transition: background 0.2s; {}",
                if is_current { "background: #094771;" } else { "" }
            ),
            // Hover effects would go here
            onclick: move |_| {
                if !is_current {
                    on_click.call(());
                }
            },

            // Branch icon
            span {
                style: "font-size: 14px;",
                match branch.branch_type {
                    BranchType::Local => "🔧",
                    BranchType::Remote => "☁️",
                    BranchType::Tag => "🏷️",
                }
            }

            // Branch name
            span {
                style: if is_current { "color: white; font-weight: 600;" } else { "color: #cccccc;" },
                "{branch.name}"
            }

            // Current indicator
            if is_current {
                span {
                    style: "margin-left: auto; color: #4ec9b0; font-size: 11px;",
                    "✓ current"
                }
            }

            // Ahead/behind indicators
            if branch.ahead > 0 || branch.behind > 0 {
                span {
                    style: "margin-left: auto; font-size: 11px; color: #808080;",
                    {
                        if branch.ahead > 0 && branch.behind > 0 {
                            format!("↑{} ↓{}", branch.ahead, branch.behind)
                        } else if branch.ahead > 0 {
                            format!("↑{}", branch.ahead)
                        } else {
                            format!("↓{}", branch.behind)
                        }
                    }
                }
            }
        }
    }
}
