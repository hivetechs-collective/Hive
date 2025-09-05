//! Example of integrating the repository dropdown with the status bar
//! 
//! This example shows how to use the enhanced repository dropdown menu
//! in a VS Code-style interface.

use dioxus::prelude::*;
use hive::desktop::{
    git::{
        RepositorySelectorState, RepositoryDropdown, RepositoryDropdownProps,
        RepositoryInfo, RepositoryStatus, UpstreamStatus, REPOSITORY_DROPDOWN_STYLES,
    },
    status_bar_enhanced::{EnhancedStatusBar, StatusBarState, StatusBarItem, StatusBarAlignment},
    workspace::repository_discovery::{
        RepositoryMetadata, DiscoveryMetadata, GitStatusInfo, WorkingDirStatus,
        ProjectInfo, ProjectType, RepositoryStats, DiscoveryMethod,
    },
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    dioxus_desktop::launch(App);
}

#[component]
fn App() -> Element {
    // Repository selector state
    let mut selector_state = use_signal(|| {
        let mut state = RepositorySelectorState::default();
        
        // Add some example repositories
        state.add_repository(RepositoryInfo {
            path: PathBuf::from("/Users/dev/projects/hive"),
            name: "hive".to_string(),
            current_branch: Some("main".to_string()),
            status: RepositoryStatus::HasChanges,
            upstream_status: Some(UpstreamStatus {
                ahead: 2,
                behind: 0,
                has_upstream: true,
            }),
        });
        
        state.add_repository(RepositoryInfo {
            path: PathBuf::from("/Users/dev/projects/rust-analyzer"),
            name: "rust-analyzer".to_string(),
            current_branch: Some("master".to_string()),
            status: RepositoryStatus::Clean,
            upstream_status: Some(UpstreamStatus {
                ahead: 0,
                behind: 5,
                has_upstream: true,
            }),
        });
        
        state.add_repository(RepositoryInfo {
            path: PathBuf::from("/Users/dev/projects/dioxus"),
            name: "dioxus".to_string(),
            current_branch: Some("feature/new-ui".to_string()),
            status: RepositoryStatus::HasConflicts,
            upstream_status: None,
        });
        
        state
    });
    
    // Extended metadata for repositories
    let repository_metadata = use_signal(|| {
        let mut metadata = HashMap::new();
        
        // Hive metadata
        metadata.insert(
            PathBuf::from("/Users/dev/projects/hive"),
            RepositoryMetadata {
                basic_info: hive::desktop::git::repository::RepositoryInfo {
                    path: PathBuf::from("/Users/dev/projects/hive"),
                    name: "hive".to_string(),
                    is_bare: false,
                    has_changes: true,
                },
                discovery: DiscoveryMetadata {
                    discovered_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    last_refreshed: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    discovery_method: DiscoveryMethod::DirectScan,
                    confidence: 1.0,
                    is_accessible: true,
                    errors: vec![],
                },
                git_status: GitStatusInfo {
                    current_branch: Some("main".to_string()),
                    upstream: None,
                    working_dir_status: WorkingDirStatus {
                        modified_files: 5,
                        staged_files: 2,
                        untracked_files: 1,
                        conflicted_files: 0,
                        is_clean: false,
                    },
                    remotes: vec![],
                    recent_commits: vec![],
                },
                stats: RepositoryStats {
                    file_count: Some(450),
                    size_bytes: Some(12_000_000),
                    primary_language: Some("Rust".to_string()),
                    languages: HashMap::new(),
                    last_commit: None,
                    contributor_count: Some(3),
                },
                project_info: Some(ProjectInfo {
                    project_type: ProjectType::Rust,
                    name: Some("hive".to_string()),
                    version: Some("2.0.0".to_string()),
                    description: Some("AI-powered development assistant".to_string()),
                    build_system: Some("Cargo".to_string()),
                    key_files: vec!["Cargo.toml".to_string()],
                }),
            }
        );
        
        // Rust analyzer metadata
        metadata.insert(
            PathBuf::from("/Users/dev/projects/rust-analyzer"),
            RepositoryMetadata {
                basic_info: hive::desktop::git::repository::RepositoryInfo {
                    path: PathBuf::from("/Users/dev/projects/rust-analyzer"),
                    name: "rust-analyzer".to_string(),
                    is_bare: false,
                    has_changes: false,
                },
                discovery: DiscoveryMetadata {
                    discovered_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    last_refreshed: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    discovery_method: DiscoveryMethod::DirectScan,
                    confidence: 1.0,
                    is_accessible: true,
                    errors: vec![],
                },
                git_status: GitStatusInfo {
                    current_branch: Some("master".to_string()),
                    upstream: None,
                    working_dir_status: WorkingDirStatus {
                        modified_files: 0,
                        staged_files: 0,
                        untracked_files: 0,
                        conflicted_files: 0,
                        is_clean: true,
                    },
                    remotes: vec![],
                    recent_commits: vec![],
                },
                stats: RepositoryStats {
                    file_count: Some(2500),
                    size_bytes: Some(45_000_000),
                    primary_language: Some("Rust".to_string()),
                    languages: HashMap::new(),
                    last_commit: None,
                    contributor_count: Some(150),
                },
                project_info: Some(ProjectInfo {
                    project_type: ProjectType::Rust,
                    name: Some("rust-analyzer".to_string()),
                    version: Some("0.3.1234".to_string()),
                    description: Some("Rust IDE support".to_string()),
                    build_system: Some("Cargo".to_string()),
                    key_files: vec!["Cargo.toml".to_string()],
                }),
            }
        );
        
        metadata
    });
    
    // Dropdown visibility state
    let mut dropdown_visible = use_signal(|| false);
    
    // Status bar state
    let mut status_bar_state = use_signal(StatusBarState::default);
    
    // Update status bar with current repository info
    use_effect(move || {
        if let Some(current_repo) = selector_state.read().current_repository() {
            status_bar_state.write().update_git_branch(
                current_repo.current_branch.as_deref().unwrap_or("no branch")
            );
            
            if let Some(upstream) = &current_repo.upstream_status {
                status_bar_state.write().update_git_sync_status(
                    upstream.ahead as u32,
                    upstream.behind as u32
                );
            }
        }
    });
    
    rsx! {
        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh; \
                    background: var(--vscode-editor-background, #1e1e1e); \
                    color: var(--vscode-editor-foreground, #cccccc);",
            
            // Add styles
            style { {REPOSITORY_DROPDOWN_STYLES} }
            style { {hive::desktop::status_bar_enhanced::STATUS_BAR_STYLES} }
            style { {CUSTOM_STYLES} }
            
            // Main content area
            div {
                class: "main-content",
                style: "flex: 1; padding: 20px; overflow-y: auto;",
                
                h1 { "Repository Dropdown Example" }
                
                p {
                    "This example demonstrates the enhanced repository dropdown menu integrated with the status bar."
                }
                
                div {
                    style: "margin-top: 20px;",
                    
                    h2 { "Current Repository" }
                    
                    if let Some(current) = selector_state.read().current_repository() {
                        div {
                            style: "background: var(--vscode-editor-inactiveSelectionBackground, #3a3d41); \
                                    padding: 15px; border-radius: 5px; margin-top: 10px;",
                            
                            p { strong { "Name: " } "{current.name}" }
                            p { strong { "Path: " } "{current.path.display()}" }
                            p { strong { "Branch: " } "{current.current_branch.as_deref().unwrap_or(\"N/A\")}" }
                            p { 
                                strong { "Status: " } 
                                span {
                                    style: "color: {current.status_color()};",
                                    "{current.status_icon()} "
                                }
                                "{current.status:?}"
                            }
                        }
                    } else {
                        p { "No repository selected" }
                    }
                }
                
                div {
                    style: "margin-top: 30px;",
                    
                    h2 { "Available Repositories" }
                    
                    div {
                        style: "display: flex; flex-direction: column; gap: 10px; margin-top: 10px;",
                        
                        for repo in selector_state.read().repositories.iter() {
                            div {
                                style: "background: var(--vscode-editor-inactiveSelectionBackground, #3a3d41); \
                                        padding: 10px; border-radius: 5px;",
                                
                                span {
                                    style: "color: {repo.status_color()}; margin-right: 10px;",
                                    "{repo.status_icon()}"
                                }
                                strong { "{repo.name}" }
                                " - "
                                span {
                                    style: "color: var(--vscode-descriptionForeground, #808080);",
                                    "{repo.path.display()}"
                                }
                            }
                        }
                    }
                }
            }
            
            // Repository dropdown (shown when visible)
            RepositoryDropdown {
                selector_state: selector_state.clone(),
                repository_metadata: Some(repository_metadata.read().clone()),
                visible: dropdown_visible.clone(),
                position: (100, 10), // Position relative to trigger
                on_repository_selected: move |path: PathBuf| {
                    println!("Repository selected: {:?}", path);
                    selector_state.write().set_current_repository(&path);
                },
                on_refresh_requested: move |_| {
                    println!("Refresh requested");
                },
                on_manage_repositories: move |_| {
                    println!("Manage repositories requested");
                }
            }
            
            // Status bar at the bottom
            div {
                style: "position: relative;",
                
                EnhancedStatusBar {
                    state: status_bar_state.clone(),
                    on_item_click: move |item_id: String| {
                        println!("Status bar item clicked: {}", item_id);
                    },
                    on_git_branch_click: move |_| {
                        // Toggle dropdown visibility
                        dropdown_visible.set(!dropdown_visible.read());
                        println!("Git branch clicked, dropdown visible: {}", dropdown_visible.read());
                    }
                }
            }
        }
    }
}

const CUSTOM_STYLES: &str = r#"
:root {
    --vscode-editor-background: #1e1e1e;
    --vscode-editor-foreground: #cccccc;
    --vscode-dropdown-background: #252526;
    --vscode-dropdown-foreground: #cccccc;
    --vscode-dropdown-border: #454545;
    --vscode-input-background: #3c3c3c;
    --vscode-input-foreground: #cccccc;
    --vscode-input-border: #464647;
    --vscode-focusBorder: #007acc;
    --vscode-list-activeSelectionBackground: #094771;
    --vscode-list-activeSelectionForeground: #ffffff;
    --vscode-list-hoverBackground: #2a2d2e;
    --vscode-widget-border: #454545;
    --vscode-widget-shadow: rgba(0, 0, 0, 0.36);
    --vscode-descriptionForeground: #808080;
    --vscode-charts-green: #4ec9b0;
    --vscode-gitDecoration-modifiedResourceForeground: #e2c08d;
    --vscode-scrollbarSlider-background: rgba(121, 121, 121, 0.4);
    --vscode-scrollbarSlider-hoverBackground: rgba(100, 100, 100, 0.7);
    --vscode-toolbar-hoverBackground: rgba(90, 93, 94, 0.31);
    --vscode-button-secondaryHoverBackground: #464647;
    --vscode-input-placeholderForeground: #808080;
    --vscode-statusBar-background: #007acc;
    --vscode-statusBar-foreground: #ffffff;
}

body {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.codicon {
    font-family: codicon;
    font-weight: normal;
    font-style: normal;
}

/* Codicon font */
@font-face {
    font-family: codicon;
    src: url('https://microsoft.github.io/vscode-codicons/dist/codicon.ttf') format('truetype');
}
"#;