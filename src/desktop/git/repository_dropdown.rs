//! Enhanced Repository Dropdown Menu Component
//! 
//! A VS Code-style dropdown menu for repository selection with advanced features
//! including search, filtering, status indicators, and rich metadata display.

use dioxus::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{info, debug};

use super::repository_selector::{RepositoryInfo, RepositorySelectorState, RepositoryStatus};
use crate::desktop::workspace::repository_discovery::{RepositoryMetadata, ProjectType};

/// Props for the enhanced repository dropdown component
#[derive(Props, Clone, PartialEq)]
pub struct RepositoryDropdownProps {
    /// Repository selector state
    pub selector_state: Signal<RepositorySelectorState>,
    /// Extended repository metadata (optional)
    pub repository_metadata: Option<HashMap<PathBuf, RepositoryMetadata>>,
    /// Whether the dropdown is visible
    pub visible: Signal<bool>,
    /// Position for the dropdown (x, y)
    pub position: (i32, i32),
    /// Callback when a repository is selected
    pub on_repository_selected: EventHandler<PathBuf>,
    /// Callback to refresh repositories
    pub on_refresh_requested: Option<EventHandler<()>>,
    /// Callback to open repository management
    pub on_manage_repositories: Option<EventHandler<()>>,
}

/// State for repository filtering
#[derive(Debug, Clone, Default)]
struct FilterState {
    search_term: String,
    show_clean: bool,
    show_modified: bool,
    show_conflicts: bool,
    filter_by_type: Option<ProjectType>,
}

impl FilterState {
    fn new() -> Self {
        Self {
            search_term: String::new(),
            show_clean: true,
            show_modified: true,
            show_conflicts: true,
            filter_by_type: None,
        }
    }
}

/// Enhanced repository dropdown menu component
#[component]
pub fn RepositoryDropdown(mut props: RepositoryDropdownProps) -> Element {
    // State management
    let mut filter_state = use_signal(FilterState::new);
    let mut loading = use_signal(|| false);
    let mut selected_index = use_signal(|| None::<usize>);
    
    // Get current repository info
    let selector_state = props.selector_state.read();
    let current_repo = selector_state.current_repository();
    
    // Filter repositories based on filter state
    let filtered_repos = use_memo(move || {
        let filter = filter_state.read();
        let selector = props.selector_state.read();
        
        selector.repositories
            .iter()
            .enumerate()
            .filter(|(_, repo)| {
                // Search filter
                if !filter.search_term.is_empty() {
                    let search_lower = filter.search_term.to_lowercase();
                    if !repo.name.to_lowercase().contains(&search_lower) &&
                       !repo.path.to_string_lossy().to_lowercase().contains(&search_lower) {
                        return false;
                    }
                }
                
                // Status filter
                match &repo.status {
                    RepositoryStatus::Clean => filter.show_clean,
                    RepositoryStatus::HasChanges => filter.show_modified,
                    RepositoryStatus::HasConflicts => filter.show_conflicts,
                    _ => true,
                }
            })
            .map(|(idx, repo)| (idx, repo.clone()))
            .collect::<Vec<_>>()
    });
    
    // Keyboard navigation
    use_effect(move || {
        if *props.visible.read() {
            // Focus on search input when dropdown opens
            // This would require a proper focus management system
        }
    });
    
    if !*props.visible.read() {
        return rsx! { div {} };
    }
    
    let (x, y) = props.position;
    
    rsx! {
        // Backdrop to capture clicks outside
        div {
            class: "repository-dropdown-backdrop",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
            onclick: move |_| {
                props.visible.set(false);
            }
        }
        
        // Main dropdown container
        div {
            class: "repository-dropdown",
            style: format!(
                "position: fixed; left: {}px; top: {}px; background: var(--vscode-dropdown-background, #252526); \
                 border: 1px solid var(--vscode-dropdown-border, #454545); \
                 border-radius: 6px; box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4); \
                 width: 400px; max-height: 600px; display: flex; flex-direction: column; \
                 z-index: 1000; font-size: 13px; color: var(--vscode-dropdown-foreground, #cccccc);",
                x, y + 30
            ),
            onclick: move |evt| {
                evt.stop_propagation();
            },
            
            // Header with search
            div {
                class: "dropdown-header",
                style: "padding: 12px; border-bottom: 1px solid var(--vscode-widget-border, #454545);",
                
                // Title row
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px;",
                    
                    div {
                        style: "font-weight: 600; display: flex; align-items: center; gap: 8px;",
                        span {
                            class: "codicon codicon-source-control",
                            style: "font-size: 16px;",
                        }
                        "Repository"
                    }
                    
                    // Action buttons
                    div {
                        style: "display: flex; gap: 8px;",
                        
                        // Refresh button
                        if let Some(on_refresh) = &props.on_refresh_requested {
                            button {
                                class: "icon-button",
                                style: "background: transparent; border: none; color: inherit; \
                                        cursor: pointer; padding: 4px; border-radius: 3px; \
                                        display: flex; align-items: center; justify-content: center;",
                                title: "Refresh Repositories",
                                onclick: {
                                    let handler = on_refresh.clone();
                                    move |_| {
                                        loading.set(true);
                                        handler.call(());
                                    }
                                },
                                span {
                                    class: "codicon codicon-refresh",
                                    style: "font-size: 16px;",
                                }
                            }
                        }
                        
                        // Settings button
                        if let Some(on_manage) = &props.on_manage_repositories {
                            button {
                                class: "icon-button",
                                style: "background: transparent; border: none; color: inherit; \
                                        cursor: pointer; padding: 4px; border-radius: 3px; \
                                        display: flex; align-items: center; justify-content: center;",
                                title: "Manage Repositories",
                                onclick: {
                                    let handler = on_manage.clone();
                                    move |_| {
                                        handler.call(());
                                        props.visible.set(false);
                                    }
                                },
                                span {
                                    class: "codicon codicon-settings-gear",
                                    style: "font-size: 16px;",
                                }
                            }
                        }
                    }
                }
                
                // Search box
                div {
                    class: "search-container",
                    style: "position: relative;",
                    
                    span {
                        class: "codicon codicon-search",
                        style: "position: absolute; left: 10px; top: 50%; transform: translateY(-50%); \
                                color: var(--vscode-input-placeholderForeground, #808080); font-size: 14px;",
                    }
                    
                    input {
                        r#type: "text",
                        placeholder: "Search repositories...",
                        value: "{filter_state.read().search_term}",
                        style: "width: 100%; padding: 6px 10px 6px 32px; \
                                background: var(--vscode-input-background, #3c3c3c); \
                                border: 1px solid var(--vscode-input-border, #464647); \
                                border-radius: 3px; color: var(--vscode-input-foreground, #cccccc); \
                                font-size: 13px; outline: none;",
                        oninput: move |evt| {
                            filter_state.write().search_term = evt.value();
                        }
                    }
                }
                
                // Filter chips
                div {
                    class: "filter-chips",
                    style: "display: flex; gap: 6px; margin-top: 8px; flex-wrap: wrap;",
                    
                    FilterChip {
                        label: "Clean",
                        icon: "check",
                        active: filter_state.read().show_clean,
                        color: "#28a745",
                        on_toggle: move |active| {
                            filter_state.write().show_clean = active;
                        }
                    }
                    
                    FilterChip {
                        label: "Modified",
                        icon: "circle-filled",
                        active: filter_state.read().show_modified,
                        color: "#ffc107",
                        on_toggle: move |active| {
                            filter_state.write().show_modified = active;
                        }
                    }
                    
                    FilterChip {
                        label: "Conflicts",
                        icon: "warning",
                        active: filter_state.read().show_conflicts,
                        color: "#dc3545",
                        on_toggle: move |active| {
                            filter_state.write().show_conflicts = active;
                        }
                    }
                }
            }
            
            // Repository list
            div {
                class: "repository-list",
                style: "flex: 1; overflow-y: auto; max-height: 400px;",
                
                if *loading.read() {
                    div {
                        style: "padding: 40px; text-align: center; color: var(--vscode-descriptionForeground, #808080);",
                        span {
                            class: "codicon codicon-loading codicon-modifier-spin",
                            style: "font-size: 24px; margin-bottom: 8px; display: block;",
                        }
                        "Loading repositories..."
                    }
                } else if filtered_repos.read().is_empty() {
                    div {
                        style: "padding: 40px; text-align: center; color: var(--vscode-descriptionForeground, #808080);",
                        if selector_state.repositories.is_empty() {
                            "No repositories found"
                        } else {
                            "No repositories match the filter"
                        }
                    }
                } else {
                    for (idx, (original_idx, repo)) in filtered_repos.read().iter().enumerate() {
                        RepositoryListItem {
                            repository: repo.clone(),
                            metadata: props.repository_metadata.as_ref()
                                .and_then(|m| m.get(&repo.path).cloned()),
                            is_current: Some(&repo.path) == current_repo.map(|r| &r.path),
                            is_selected: selected_index.read().as_ref() == Some(&idx),
                            on_click: {
                                let idx = idx;
                                move |_| {
                                    selected_index.set(Some(idx));
                                }
                            },
                            on_double_click: {
                                let path = repo.path.clone();
                                let handler = props.on_repository_selected.clone();
                                let mut visible = props.visible.clone();
                                move |_| {
                                    handler.call(path.clone());
                                    visible.set(false);
                                }
                            }
                        }
                    }
                }
            }
            
            // Footer with stats
            div {
                class: "dropdown-footer",
                style: "padding: 8px 12px; border-top: 1px solid var(--vscode-widget-border, #454545); \
                        font-size: 11px; color: var(--vscode-descriptionForeground, #808080);",
                
                if let Some(current) = current_repo {
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        
                        span {
                            "Current: {current.name}"
                        }
                        
                        span {
                            "{filtered_repos.read().len()} of {selector_state.repositories.len()} repositories"
                        }
                    }
                } else {
                    "{selector_state.repositories.len()} repositories"
                }
            }
        }
    }
}

/// Filter chip component
#[component]
fn FilterChip(
    label: &'static str,
    icon: &'static str,
    active: bool,
    color: &'static str,
    on_toggle: EventHandler<bool>
) -> Element {
    rsx! {
        button {
            class: if active { "filter-chip active" } else { "filter-chip" },
            style: format!(
                "padding: 4px 8px; background: {}; border: 1px solid {}; \
                 border-radius: 12px; cursor: pointer; font-size: 11px; \
                 display: flex; align-items: center; gap: 4px; transition: all 0.2s;",
                if active { color } else { "transparent" },
                if active { color } else { "#464647" }
            ),
            onclick: move |_| {
                on_toggle.call(!active);
            },
            
            span {
                class: format!("codicon codicon-{}", icon),
                style: format!(
                    "font-size: 12px; color: {};",
                    if active { "white" } else { color }
                ),
            }
            
            span {
                style: format!(
                    "color: {};",
                    if active { "white" } else { "#cccccc" }
                ),
                "{label}"
            }
        }
    }
}

/// Repository list item component
#[component]
fn RepositoryListItem(
    repository: RepositoryInfo,
    metadata: Option<RepositoryMetadata>,
    is_current: bool,
    is_selected: bool,
    on_click: EventHandler<()>,
    on_double_click: EventHandler<()>
) -> Element {
    // Get project type icon
    let project_icon = if let Some(ref meta) = metadata {
        if let Some(ref project_info) = meta.project_info {
            match project_info.project_type {
                ProjectType::Rust => "🦀",
                ProjectType::JavaScript => "📜",
                ProjectType::TypeScript => "💙",
                ProjectType::Python => "🐍",
                ProjectType::Go => "🐹",
                ProjectType::Java => "☕",
                ProjectType::CSharp => "🔷",
                ProjectType::Cpp => "⚙️",
                ProjectType::Ruby => "💎",
                ProjectType::PHP => "🐘",
                ProjectType::Swift => "🦉",
                _ => "📁",
            }
        } else {
            "📁"
        }
    } else {
        "📁"
    };
    
    rsx! {
        div {
            class: "repository-list-item",
            style: format!(
                "padding: 8px 12px; cursor: pointer; display: flex; align-items: center; \
                 gap: 8px; transition: background 0.1s; {}",
                if is_current {
                    "background: var(--vscode-list-activeSelectionBackground, #094771);"
                } else if is_selected {
                    "background: var(--vscode-list-hoverBackground, #2a2d2e);"
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
            
            // Status indicator
            span {
                class: "repo-status-icon",
                style: format!(
                    "font-size: 16px; color: {}; flex-shrink: 0;",
                    repository.status_color()
                ),
                "{repository.status_icon()}"
            }
            
            // Project type icon
            span {
                style: "font-size: 16px; flex-shrink: 0;",
                "{project_icon}"
            }
            
            // Repository info
            div {
                style: "flex: 1; display: flex; flex-direction: column; gap: 2px; min-width: 0;",
                
                // Name and branch
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    
                    span {
                        style: if is_current { 
                            "font-weight: 600; color: var(--vscode-list-activeSelectionForeground, white);" 
                        } else { 
                            "font-weight: 500;" 
                        },
                        class: "repo-name",
                        title: "{repository.path.display()}",
                        "{repository.name}"
                    }
                    
                    if let Some(branch) = &repository.current_branch {
                        span {
                            style: "color: var(--vscode-descriptionForeground, #808080); font-size: 11px;",
                            class: "codicon codicon-git-branch",
                            " {branch}"
                        }
                    }
                    
                    if is_current {
                        span {
                            style: "color: var(--vscode-charts-green, #4ec9b0); font-size: 11px; \
                                    margin-left: auto;",
                            "● current"
                        }
                    }
                }
                
                // Additional metadata
                if let Some(ref meta) = metadata {
                    div {
                        style: "font-size: 11px; color: var(--vscode-descriptionForeground, #808080); \
                                display: flex; gap: 12px;",
                        
                        // Project info
                        if let Some(ref project_info) = meta.project_info {
                            if let Some(ref version) = project_info.version {
                                span { "v{version}" }
                            }
                        }
                        
                        // Git status summary
                        if let Some(ref upstream) = meta.git_status.upstream {
                            if upstream.ahead > 0 || upstream.behind > 0 {
                                span {
                                    "↑{upstream.ahead} ↓{upstream.behind}"
                                }
                            }
                        }
                        
                        // Working directory status
                        if meta.git_status.working_dir_status.modified_files > 0 {
                            span {
                                "+{meta.git_status.working_dir_status.modified_files}"
                            }
                        }
                    }
                }
            }
            
            // Upstream status
            if let Some(ref upstream) = repository.upstream_status {
                if upstream.has_upstream && (upstream.ahead > 0 || upstream.behind > 0) {
                    div {
                        style: "display: flex; align-items: center; gap: 4px; \
                                color: var(--vscode-gitDecoration-modifiedResourceForeground, #e2c08d); \
                                font-size: 11px;",
                        
                        if upstream.behind > 0 {
                            span {
                                class: "codicon codicon-arrow-down",
                                title: "{upstream.behind} commits behind",
                                "{upstream.behind}"
                            }
                        }
                        
                        if upstream.ahead > 0 {
                            span {
                                class: "codicon codicon-arrow-up",
                                title: "{upstream.ahead} commits ahead",
                                "{upstream.ahead}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// CSS styles for the repository dropdown
pub const REPOSITORY_DROPDOWN_STYLES: &str = r#"
/* Repository dropdown container */
.repository-dropdown {
    font-family: var(--vscode-font-family, -apple-system, BlinkMacSystemFont, sans-serif);
}

/* Icon buttons */
.icon-button:hover {
    background-color: var(--vscode-toolbar-hoverBackground, rgba(90, 93, 94, 0.31)) !important;
}

/* Search input focus */
.search-container input:focus {
    border-color: var(--vscode-focusBorder, #007acc) !important;
}

/* Filter chips */
.filter-chip {
    outline: none;
    font-family: inherit;
}

.filter-chip:hover {
    background-color: var(--vscode-button-secondaryHoverBackground, #464647) !important;
}

/* Repository list */
.repository-list {
    scrollbar-width: thin;
    scrollbar-color: var(--vscode-scrollbarSlider-background, rgba(121, 121, 121, 0.4)) transparent;
}

.repository-list::-webkit-scrollbar {
    width: 10px;
}

.repository-list::-webkit-scrollbar-track {
    background: transparent;
}

.repository-list::-webkit-scrollbar-thumb {
    background-color: var(--vscode-scrollbarSlider-background, rgba(121, 121, 121, 0.4));
    border-radius: 5px;
}

.repository-list::-webkit-scrollbar-thumb:hover {
    background-color: var(--vscode-scrollbarSlider-hoverBackground, rgba(100, 100, 100, 0.7));
}

/* Repository list item */
.repository-list-item {
    border-left: 2px solid transparent;
}

.repository-list-item:hover {
    border-left-color: var(--vscode-list-hoverBackground, #2a2d2e);
}

.repository-list-item .repo-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* Loading spinner animation */
@keyframes codicon-spin {
    100% { transform: rotate(360deg); }
}

.codicon-modifier-spin {
    animation: codicon-spin 1.5s linear infinite;
}

/* Dropdown shadow for better visibility */
.repository-dropdown {
    box-shadow: 0 0 0 1px var(--vscode-widget-shadow, rgba(0, 0, 0, 0.36)),
                0 8px 16px var(--vscode-widget-shadow, rgba(0, 0, 0, 0.36));
}
"#;