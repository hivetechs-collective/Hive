//! Enhanced git branch menu with full branch management features
//! 
//! Provides a comprehensive branch management UI with search, filtering,
//! creation, checkout, and deletion capabilities

use dioxus::prelude::*;
use std::path::PathBuf;
use super::{GitRepository, BranchInfo, BranchType, BranchFilter, BranchSort, sort_branches, validate_branch_name};
use tracing::{info, error};
use anyhow::Result;

/// Props for the enhanced branch menu component
#[derive(Props, Clone, PartialEq)]
pub struct BranchMenuProps {
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
    /// Callback when branch operation completes
    pub on_operation_complete: EventHandler<BranchOperationResult>,
}

/// Result of a branch operation
#[derive(Debug, Clone)]
pub struct BranchOperationResult {
    pub operation: BranchOperation,
    pub success: bool,
    pub message: String,
}

/// Types of branch operations
#[derive(Debug, Clone)]
pub enum BranchOperation {
    Checkout(String),
    Create(String),
    Delete(String),
    Fetch,
    Pull,
    Push,
}

/// Enhanced git branch menu component
#[component]
pub fn BranchMenu(props: BranchMenuProps) -> Element {
    // Clone values to avoid ownership issues
    let repo_path_for_effect = props.repo_path.clone();
    let repo_path_for_dialog = props.repo_path.clone();
    let repo_path_for_switch = props.repo_path.clone();
    let repo_path_for_create = props.repo_path.clone();
    let repo_path_for_delete = props.repo_path.clone();
    
    // State management
    let mut branches = use_signal(|| Vec::<BranchInfo>::new());
    let mut filtered_branches = use_signal(|| Vec::<BranchInfo>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut filter = use_signal(|| BranchFilter::default());
    let mut show_create_dialog = use_signal(|| false);
    let mut show_delete_confirmation = use_signal(|| None::<String>);
    let mut selected_branch = use_signal(|| None::<String>);
    
    // Load branches when menu becomes visible
    use_effect(move || {
        if *props.visible.read() {
            loading.set(true);
            error.set(None);
            
            if let Some(repo_path) = props.repo_path.clone() {
                spawn(async move {
                    match load_branches_with_details(&repo_path).await {
                        Ok(branch_list) => {
                            branches.set(branch_list.clone());
                            apply_filter(&branch_list, &filter.read(), &mut filtered_branches);
                            loading.set(false);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load branches: {}", e)));
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
    
    // Apply filter when it changes
    use_effect(move || {
        let current_filter = filter.read();
        let current_branches = branches.read();
        apply_filter(&current_branches, &current_filter, &mut filtered_branches);
    });
    
    if !*props.visible.read() {
        return rsx! { div {} };
    }
    
    let current_branch = props.branch_info.as_ref().map(|b| &b.name);
    let (x, y) = props.position;
    
    rsx! {
        // Backdrop to capture clicks outside
        div {
            class: "branch-menu-backdrop",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
            onclick: {
                let mut visible = props.visible.clone();
                move |_| {
                    visible.set(false);
                }
            }
        }
        
        // Main menu container
        div {
            class: "branch-menu",
            style: format!(
                "position: fixed; left: {}px; bottom: {}px; background: #252526; border: 1px solid #3e3e42; \
                 border-radius: 6px; box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4); \
                 width: 500px; height: 600px; display: flex; flex-direction: column; \
                 z-index: 1000; font-size: 13px;",
                x, y + 30
            ),
            onclick: move |evt| {
                evt.stop_propagation();
            },
            
            // Menu header with search
            div {
                class: "menu-header",
                style: "padding: 12px; border-bottom: 1px solid #3e3e42;",
                
                // Title
                div {
                    style: "font-weight: 600; color: #cccccc; margin-bottom: 10px;",
                    "Git Branches"
                }
                
                // Search box
                div {
                    class: "search-container",
                    style: "position: relative;",
                    
                    input {
                        r#type: "text",
                        placeholder: "Search branches...",
                        value: "{filter.read().search_term}",
                        style: "width: 100%; padding: 6px 30px 6px 10px; background: #3c3c3c; \
                                border: 1px solid #464647; border-radius: 3px; color: #cccccc; \
                                font-size: 13px;",
                        oninput: move |evt| {
                            filter.write().search_term = evt.value();
                        }
                    }
                    
                    // Search icon
                    span {
                        style: "position: absolute; right: 10px; top: 50%; transform: translateY(-50%); \
                                color: #808080; font-size: 14px;",
                        "🔍"
                    }
                }
                
                // Filter options
                div {
                    class: "filter-options",
                    style: "display: flex; gap: 12px; margin-top: 8px; font-size: 12px;",
                    
                    FilterCheckbox {
                        label: "Local",
                        checked: filter.read().show_local,
                        on_change: move |checked| {
                            filter.write().show_local = checked;
                        }
                    }
                    
                    FilterCheckbox {
                        label: "Remote",
                        checked: filter.read().show_remote,
                        on_change: move |checked| {
                            filter.write().show_remote = checked;
                        }
                    }
                    
                    FilterCheckbox {
                        label: "Tags",
                        checked: filter.read().show_tags,
                        on_change: move |checked| {
                            filter.write().show_tags = checked;
                        }
                    }
                    
                    // Sort dropdown
                    div {
                        style: "margin-left: auto;",
                        select {
                            style: "background: #3c3c3c; border: 1px solid #464647; \
                                    border-radius: 3px; color: #cccccc; padding: 2px 8px; \
                                    font-size: 12px;",
                            onchange: move |evt| {
                                filter.write().sort_by = match evt.value().as_str() {
                                    "name" => BranchSort::Name,
                                    "commit" => BranchSort::LastCommit,
                                    "ahead" => BranchSort::Ahead,
                                    "behind" => BranchSort::Behind,
                                    _ => BranchSort::Name,
                                };
                            },
                            option { value: "name", "Name" }
                            option { value: "commit", "Last Commit" }
                            option { value: "ahead", "Ahead" }
                            option { value: "behind", "Behind" }
                        }
                    }
                }
            }
            
            // Branch list
            div {
                class: "branch-list",
                style: "flex: 1; overflow-y: auto; padding: 8px 0;",
                
                if *loading.read() {
                    div {
                        style: "padding: 40px; text-align: center; color: #808080;",
                        "Loading branches..."
                    }
                } else if let Some(err) = error.read().as_ref() {
                    div {
                        style: "padding: 40px; text-align: center; color: #f48771;",
                        "{err}"
                    }
                } else if filtered_branches.read().is_empty() {
                    div {
                        style: "padding: 40px; text-align: center; color: #808080;",
                        if branches.read().is_empty() {
                            "No branches found"
                        } else {
                            "No branches match the filter"
                        }
                    }
                } else {
                    for branch in filtered_branches.read().iter() {
                        BranchListItem {
                            branch: branch.clone(),
                            is_current: Some(&branch.name) == current_branch,
                            is_selected: selected_branch.read().as_ref() == Some(&branch.name),
                            on_click: {
                                let branch_name = branch.name.clone();
                                let mut selected = selected_branch.clone();
                                move |_| {
                                    selected.set(Some(branch_name.clone()));
                                }
                            },
                            on_double_click: {
                                let branch_name = branch.name.clone();
                                let on_branch_selected = props.on_branch_selected.clone();
                                let mut visible = props.visible.clone();
                                move |_| {
                                    on_branch_selected.call(branch_name.clone());
                                    visible.set(false);
                                }
                            },
                            on_delete: {
                                let branch_name = branch.name.clone();
                                let mut show_delete = show_delete_confirmation.clone();
                                move |_| {
                                    show_delete.set(Some(branch_name.clone()));
                                }
                            }
                        }
                    }
                }
            }
            
            // Actions toolbar
            div {
                class: "actions-toolbar",
                style: "border-top: 1px solid #3e3e42; padding: 12px; display: flex; gap: 8px;",
                
                // Create branch button
                button {
                    class: "action-button primary",
                    style: "padding: 6px 12px; background: #0e639c; color: white; \
                            border: none; border-radius: 3px; cursor: pointer; \
                            font-size: 13px; transition: background 0.2s;",
                    onclick: {
                        let mut show_create = show_create_dialog.clone();
                        move |_| {
                            show_create.set(true);
                        }
                    },
                    "✚ Create Branch"
                }
                
                // Checkout button
                if let Some(selected) = selected_branch.read().as_ref() {
                    if Some(selected) != current_branch {
                        button {
                            class: "action-button",
                            style: "padding: 6px 12px; background: #3c3c3c; color: white; \
                                    border: 1px solid #464647; border-radius: 3px; \
                                    cursor: pointer; font-size: 13px; transition: background 0.2s;",
                            onclick: {
                                let branch_name = selected.clone();
                                let on_branch_selected = props.on_branch_selected.clone();
                                let mut visible = props.visible.clone();
                                move |_| {
                                    on_branch_selected.call(branch_name.clone());
                                    visible.set(false);
                                }
                            },
                            "↻ Checkout"
                        }
                    }
                }
                
                // Fetch button
                button {
                    class: "action-button",
                    style: "margin-left: auto; padding: 6px 12px; background: #3c3c3c; \
                            color: white; border: 1px solid #464647; border-radius: 3px; \
                            cursor: pointer; font-size: 13px; transition: background 0.2s;",
                    onclick: {
                        let repo_path = repo_path_for_switch.clone();
                        let on_operation = props.on_operation_complete.clone();
                        move |_| {
                            if let Some(path) = repo_path.as_ref() {
                                perform_fetch(path.clone(), on_operation.clone());
                            }
                        }
                    },
                    "⬇ Fetch All"
                }
            }
        }
        
        // Create branch dialog
        if *show_create_dialog.read() {
            CreateBranchDialog {
                repo_path: repo_path_for_dialog.clone(),
                visible: show_create_dialog.clone(),
                on_create: {
                    let on_operation = props.on_operation_complete.clone();
                    let mut branches_signal = branches.clone();
                    let repo_path = repo_path_for_create.clone();
                    move |branch_name: String| {
                        on_operation.call(BranchOperationResult {
                            operation: BranchOperation::Create(branch_name.clone()),
                            success: true,
                            message: format!("Created branch '{}'", branch_name),
                        });
                        
                        // Reload branches
                        if let Some(path) = repo_path.as_ref() {
                            let path = path.clone();
                            spawn(async move {
                                if let Ok(branch_list) = load_branches_with_details(&path).await {
                                    branches_signal.set(branch_list);
                                }
                            });
                        }
                    }
                }
            }
        }
        
        // Delete confirmation dialog
        if let Some(branch_to_delete) = show_delete_confirmation.read().as_ref() {
            DeleteConfirmationDialog {
                branch_name: branch_to_delete.clone(),
                visible: show_delete_confirmation.clone(),
                on_confirm: {
                    let branch_name = branch_to_delete.clone();
                    let repo_path = repo_path_for_delete.clone();
                    let on_operation = props.on_operation_complete.clone();
                    let mut branches_signal = branches.clone();
                    move |_| {
                        if let Some(path) = repo_path.as_ref() {
                            perform_delete_branch(
                                path.clone(),
                                branch_name.clone(),
                                on_operation.clone(),
                                branches_signal.clone()
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Filter checkbox component
#[component]
fn FilterCheckbox(label: &'static str, checked: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        label {
            style: "display: flex; align-items: center; gap: 4px; cursor: pointer; \
                    color: #cccccc;",
            input {
                r#type: "checkbox",
                checked: checked,
                onchange: move |evt| {
                    on_change.call(evt.checked());
                }
            }
            "{label}"
        }
    }
}

/// Individual branch list item
#[component]
fn BranchListItem(
    branch: BranchInfo,
    is_current: bool,
    is_selected: bool,
    on_click: EventHandler<()>,
    on_double_click: EventHandler<()>,
    on_delete: EventHandler<()>
) -> Element {
    rsx! {
        div {
            class: "branch-list-item",
            style: format!(
                "padding: 8px 16px; cursor: pointer; display: flex; align-items: center; \
                 gap: 8px; transition: background 0.2s; {}",
                if is_current {
                    "background: #094771;"
                } else if is_selected {
                    "background: #2a2d2e;"
                } else {
                    ""
                }
            ),
            onclick: move |_| {
                on_click.call(());
            },
            ondblclick: move |_| {
                on_double_click.call(());
            },
            
            // Branch icon
            span {
                style: "font-size: 14px; width: 20px; text-align: center;",
                match branch.branch_type {
                    BranchType::Local => "🔧",
                    BranchType::Remote => "☁️",
                    BranchType::Tag => "🏷️",
                }
            }
            
            // Branch info
            div {
                style: "flex: 1; display: flex; flex-direction: column; gap: 2px;",
                
                // Branch name and indicators
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    
                    span {
                        style: if is_current { 
                            "color: white; font-weight: 600;" 
                        } else { 
                            "color: #cccccc;" 
                        },
                        "{branch.name}"
                    }
                    
                    if is_current {
                        span {
                            style: "color: #4ec9b0; font-size: 11px;",
                            "✓ current"
                        }
                    }
                    
                    if branch.ahead > 0 || branch.behind > 0 {
                        span {
                            style: "font-size: 11px; color: #808080;",
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
                
                // Last commit info
                if let Some(commit) = &branch.last_commit {
                    div {
                        style: "font-size: 11px; color: #808080; display: flex; gap: 8px;",
                        
                        span {
                            style: "font-family: monospace;",
                            "{commit.hash_short}"
                        }
                        
                        span {
                            style: "flex: 1; overflow: hidden; text-overflow: ellipsis; \
                                    white-space: nowrap;",
                            "{commit.message}"
                        }
                        
                        span {
                            title: "{commit.date}",
                            {format_relative_time(&commit.date)}
                        }
                    }
                }
            }
            
            // Delete button (for non-current local branches)
            if branch.branch_type == BranchType::Local && !is_current {
                button {
                    class: "delete-button",
                    style: "padding: 4px 8px; background: transparent; border: none; \
                            color: #808080; cursor: pointer; font-size: 12px; \
                            transition: color 0.2s;",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_delete.call(());
                    },
                    "🗑️"
                }
            }
        }
    }
}

/// Create branch dialog
#[component]
fn CreateBranchDialog(
    repo_path: Option<PathBuf>,
    visible: Signal<bool>,
    on_create: EventHandler<String>
) -> Element {
    let mut branch_name = use_signal(|| String::new());
    let mut from_branch = use_signal(|| String::from("HEAD"));
    let mut error = use_signal(|| None::<String>);
    let mut creating = use_signal(|| false);
    
    rsx! {
        // Dialog backdrop
        div {
            class: "dialog-backdrop",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; \
                    background: rgba(0, 0, 0, 0.5); z-index: 1100; \
                    display: flex; align-items: center; justify-content: center;",
            onclick: move |_| {
                visible.set(false);
            },
            
            // Dialog content
            div {
                class: "dialog-content",
                style: "background: #252526; border: 1px solid #3e3e42; \
                        border-radius: 6px; padding: 20px; width: 400px; \
                        box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4);",
                onclick: move |evt| {
                    evt.stop_propagation();
                },
                
                h3 {
                    style: "margin: 0 0 16px 0; color: #cccccc; font-size: 16px;",
                    "Create New Branch"
                }
                
                // Branch name input
                div {
                    style: "margin-bottom: 12px;",
                    label {
                        style: "display: block; margin-bottom: 4px; color: #cccccc; \
                                font-size: 13px;",
                        "Branch Name"
                    }
                    input {
                        r#type: "text",
                        placeholder: "feature/my-new-feature",
                        value: "{branch_name.read()}",
                        style: "width: 100%; padding: 8px; background: #3c3c3c; \
                                border: 1px solid #464647; border-radius: 3px; \
                                color: #cccccc; font-size: 13px;",
                        oninput: move |evt| {
                            branch_name.set(evt.value());
                            // Validate on input
                            match validate_branch_name(&evt.value()) {
                                Ok(_) => error.set(None),
                                Err(e) => error.set(Some(e.to_string())),
                            }
                        }
                    }
                }
                
                // From branch selector
                div {
                    style: "margin-bottom: 16px;",
                    label {
                        style: "display: block; margin-bottom: 4px; color: #cccccc; \
                                font-size: 13px;",
                        "Create From"
                    }
                    select {
                        style: "width: 100%; padding: 8px; background: #3c3c3c; \
                                border: 1px solid #464647; border-radius: 3px; \
                                color: #cccccc; font-size: 13px;",
                        value: "{from_branch.read()}",
                        onchange: move |evt| {
                            from_branch.set(evt.value());
                        },
                        option { value: "HEAD", "Current HEAD" }
                        // TODO: Load and display available branches
                    }
                }
                
                // Error message
                if let Some(err) = error.read().as_ref() {
                    div {
                        style: "margin-bottom: 16px; padding: 8px; background: #5a1d1d; \
                                border: 1px solid #f48771; border-radius: 3px; \
                                color: #f48771; font-size: 12px;",
                        "{err}"
                    }
                }
                
                // Action buttons
                div {
                    style: "display: flex; gap: 8px; justify-content: flex-end;",
                    
                    button {
                        style: "padding: 8px 16px; background: #3c3c3c; color: #cccccc; \
                                border: 1px solid #464647; border-radius: 3px; \
                                cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            visible.set(false);
                        },
                        "Cancel"
                    }
                    
                    button {
                        style: "padding: 8px 16px; background: #0e639c; color: white; \
                                border: none; border-radius: 3px; cursor: pointer; \
                                font-size: 13px;",
                        disabled: branch_name.read().is_empty() || error.read().is_some() || *creating.read(),
                        onclick: move |_| {
                            let name = branch_name.read().clone();
                            let from = from_branch.read().clone();
                            
                            if let Some(path) = repo_path.as_ref() {
                                creating.set(true);
                                let path = path.clone();
                                let on_create_handler = on_create.clone();
                                let mut visible_signal = visible.clone();
                                let mut error_signal = error.clone();
                                let mut creating_signal = creating.clone();
                                
                                spawn(async move {
                                    match create_branch(&path, &name, &from).await {
                                        Ok(_) => {
                                            on_create_handler.call(name);
                                            visible_signal.set(false);
                                        }
                                        Err(e) => {
                                            error_signal.set(Some(format!("Failed to create branch: {}", e)));
                                        }
                                    }
                                    creating_signal.set(false);
                                });
                            }
                        },
                        if *creating.read() { "Creating..." } else { "Create" }
                    }
                }
            }
        }
    }
}

/// Delete confirmation dialog
#[component]
fn DeleteConfirmationDialog(
    branch_name: String,
    visible: Signal<Option<String>>,
    on_confirm: EventHandler<()>
) -> Element {
    rsx! {
        // Dialog backdrop
        div {
            class: "dialog-backdrop",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; \
                    background: rgba(0, 0, 0, 0.5); z-index: 1100; \
                    display: flex; align-items: center; justify-content: center;",
            onclick: move |_| {
                visible.set(None);
            },
            
            // Dialog content
            div {
                class: "dialog-content",
                style: "background: #252526; border: 1px solid #3e3e42; \
                        border-radius: 6px; padding: 20px; width: 400px; \
                        box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4);",
                onclick: move |evt| {
                    evt.stop_propagation();
                },
                
                h3 {
                    style: "margin: 0 0 16px 0; color: #cccccc; font-size: 16px;",
                    "Delete Branch"
                }
                
                p {
                    style: "margin-bottom: 20px; color: #cccccc; font-size: 13px;",
                    "Are you sure you want to delete the branch '",
                    strong { "{branch_name}" },
                    "'? This action cannot be undone."
                }
                
                // Action buttons
                div {
                    style: "display: flex; gap: 8px; justify-content: flex-end;",
                    
                    button {
                        style: "padding: 8px 16px; background: #3c3c3c; color: #cccccc; \
                                border: 1px solid #464647; border-radius: 3px; \
                                cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            visible.set(None);
                        },
                        "Cancel"
                    }
                    
                    button {
                        style: "padding: 8px 16px; background: #f14c4c; color: white; \
                                border: none; border-radius: 3px; cursor: pointer; \
                                font-size: 13px;",
                        onclick: move |_| {
                            on_confirm.call(());
                            visible.set(None);
                        },
                        "Delete"
                    }
                }
            }
        }
    }
}

// Helper functions

/// Load branches with commit details
async fn load_branches_with_details(repo_path: &PathBuf) -> Result<Vec<BranchInfo>> {
    let path = repo_path.clone();
    tokio::task::spawn_blocking(move || {
        let repo = GitRepository::open(&path)?;
        let mut branches = repo.list_branches()?;
        
        // Load last commit info for each branch
        for branch in &mut branches {
            if let Ok(commit_info) = repo.get_branch_commit(&branch.name) {
                branch.last_commit = Some(commit_info);
            }
        }
        
        Ok(branches)
    }).await?
}

/// Apply filter and sort branches
fn apply_filter(branches: &[BranchInfo], filter: &BranchFilter, output: &mut Signal<Vec<BranchInfo>>) {
    let mut filtered: Vec<BranchInfo> = branches
        .iter()
        .filter(|b| b.matches_filter(filter))
        .cloned()
        .collect();
    
    sort_branches(&mut filtered, &filter.sort_by);
    output.set(filtered);
}

/// Format relative time for display
fn format_relative_time(date: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*date);
    
    if duration.num_days() > 365 {
        format!("{} years ago", duration.num_days() / 365)
    } else if duration.num_days() > 30 {
        format!("{} months ago", duration.num_days() / 30)
    } else if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

/// Create a new branch
async fn create_branch(repo_path: &PathBuf, branch_name: &str, from: &str) -> Result<()> {
    let path = repo_path.clone();
    let name = branch_name.to_string();
    let from_ref = from.to_string();
    
    tokio::task::spawn_blocking(move || {
        let repo = GitRepository::open(&path)?;
        repo.create_branch(&name, &from_ref)?;
        Ok(())
    }).await?
}

/// Perform git fetch
fn perform_fetch(repo_path: PathBuf, on_complete: EventHandler<BranchOperationResult>) {
    spawn(async move {
        match fetch_all(&repo_path).await {
            Ok(_) => {
                on_complete.call(BranchOperationResult {
                    operation: BranchOperation::Fetch,
                    success: true,
                    message: "Successfully fetched all remotes".to_string(),
                });
            }
            Err(e) => {
                on_complete.call(BranchOperationResult {
                    operation: BranchOperation::Fetch,
                    success: false,
                    message: format!("Failed to fetch: {}", e),
                });
            }
        }
    });
}

/// Fetch all remotes
async fn fetch_all(repo_path: &PathBuf) -> Result<()> {
    let path = repo_path.clone();
    tokio::task::spawn_blocking(move || {
        let repo = GitRepository::open(&path)?;
        repo.fetch_all()?;
        Ok(())
    }).await?
}

/// Delete a branch
fn perform_delete_branch(
    repo_path: PathBuf,
    branch_name: String,
    on_complete: EventHandler<BranchOperationResult>,
    mut branches: Signal<Vec<BranchInfo>>
) {
    spawn(async move {
        match delete_branch(&repo_path, &branch_name).await {
            Ok(_) => {
                on_complete.call(BranchOperationResult {
                    operation: BranchOperation::Delete(branch_name.clone()),
                    success: true,
                    message: format!("Deleted branch '{}'", branch_name),
                });
                
                // Reload branches
                if let Ok(branch_list) = load_branches_with_details(&repo_path).await {
                    branches.set(branch_list);
                }
            }
            Err(e) => {
                on_complete.call(BranchOperationResult {
                    operation: BranchOperation::Delete(branch_name),
                    success: false,
                    message: format!("Failed to delete branch: {}", e),
                });
            }
        }
    });
}

/// Delete a branch
async fn delete_branch(repo_path: &PathBuf, branch_name: &str) -> Result<()> {
    let path = repo_path.clone();
    let name = branch_name.to_string();
    
    tokio::task::spawn_blocking(move || {
        let repo = GitRepository::open(&path)?;
        repo.delete_branch(&name)?;
        Ok(())
    }).await?
}