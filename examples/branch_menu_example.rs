//! Example usage of the enhanced branch menu
//!
//! This example demonstrates how to integrate the branch menu
//! into a Dioxus application with full branch management features

use crate::desktop::git::{
    use_git_state, BranchInfo, BranchMenu, BranchMenuProps, BranchOperation, BranchOperationResult,
    GitRepository,
};
use dioxus::prelude::*;
use std::path::PathBuf;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    // Git state management
    let git_state = use_git_state();
    let mut show_branch_menu = use_signal(|| false);
    let mut operation_messages = use_signal(|| Vec::<String>::new());

    // Example repository path (would come from workspace in real app)
    let repo_path = Some(PathBuf::from("."));

    rsx! {
        div {
            style: "padding: 20px; background: #1e1e1e; color: #cccccc; \
                    min-height: 100vh; font-family: 'Segoe UI', sans-serif;",

            h1 { "Enhanced Branch Menu Example" }

            // Current branch display
            div {
                style: "margin: 20px 0; padding: 16px; background: #252526; \
                        border-radius: 6px;",

                h3 { "Current Branch" }

                if let Some(branch) = git_state.branch_info.read().as_ref() {
                    div {
                        style: "display: flex; align-items: center; gap: 12px;",

                        span {
                            style: "font-size: 16px;",
                            "🔧 {branch.name}"
                        }

                        if branch.ahead > 0 || branch.behind > 0 {
                            span {
                                style: "color: #808080;",
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
                } else {
                    span { style: "color: #808080;", "No branch information" }
                }
            }

            // Button to show branch menu
            button {
                style: "padding: 10px 20px; background: #0e639c; color: white; \
                        border: none; border-radius: 4px; cursor: pointer; \
                        font-size: 14px;",
                onclick: move |_| {
                    show_branch_menu.set(true);
                },
                "Open Branch Menu"
            }

            // Operation messages
            if !operation_messages.read().is_empty() {
                div {
                    style: "margin-top: 20px; padding: 16px; background: #252526; \
                            border-radius: 6px;",

                    h3 { "Recent Operations" }

                    for message in operation_messages.read().iter() {
                        div {
                            style: "padding: 8px; margin: 4px 0; background: #1e1e1e; \
                                    border-radius: 4px; font-size: 13px;",
                            "{message}"
                        }
                    }
                }
            }

            // Branch menu
            BranchMenu {
                repo_path: repo_path.clone(),
                branch_info: git_state.branch_info.read().clone(),
                visible: show_branch_menu,
                position: (100, 300), // Position relative to bottom of screen
                on_branch_selected: move |branch_name: String| {
                    // Handle branch checkout
                    operation_messages.write().push(
                        format!("Switching to branch: {}", branch_name)
                    );

                    // In a real app, this would trigger the actual checkout
                    if let Some(path) = repo_path.as_ref() {
                        match checkout_branch(path, &branch_name) {
                            Ok(_) => {
                                operation_messages.write().push(
                                    format!("✅ Successfully checked out '{}'", branch_name)
                                );
                            }
                            Err(e) => {
                                operation_messages.write().push(
                                    format!("❌ Failed to checkout: {}", e)
                                );
                            }
                        }
                    }
                },
                on_operation_complete: move |result: BranchOperationResult| {
                    // Handle operation results
                    let icon = if result.success { "✅" } else { "❌" };
                    let operation_type = match &result.operation {
                        BranchOperation::Checkout(name) => format!("Checkout '{}'", name),
                        BranchOperation::Create(name) => format!("Create '{}'", name),
                        BranchOperation::Delete(name) => format!("Delete '{}'", name),
                        BranchOperation::Fetch => "Fetch".to_string(),
                        BranchOperation::Pull => "Pull".to_string(),
                        BranchOperation::Push => "Push".to_string(),
                    };

                    operation_messages.write().push(
                        format!("{} {}: {}", icon, operation_type, result.message)
                    );
                }
            }
        }
    }
}

// Helper function for checkout (simplified)
fn checkout_branch(
    repo_path: &PathBuf,
    branch_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("checkout")
        .arg(branch_name)
        .current_dir(repo_path)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Git checkout failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}
