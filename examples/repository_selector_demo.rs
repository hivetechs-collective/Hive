//! Repository Selector Demo
//! 
//! This example demonstrates how the repository selector works in the status bar

use dioxus::prelude::*;
use hive::desktop::status_bar_enhanced::{StatusBarState, EnhancedStatusBar, STATUS_BAR_STYLES};
use hive::desktop::events::{EventBus, EventType, Event};
use hive::desktop::workspace::WorkspaceState;
use std::path::PathBuf;

fn main() {
    // Launch the desktop app
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Initialize state
    let mut status_bar_state = use_signal(|| StatusBarState::default());
    let mut event_bus = use_signal(|| EventBus::new());
    let mut workspace_state = use_signal(|| {
        let mut ws = WorkspaceState::new(PathBuf::from("/home/user/projects"));
        // Simulate having multiple repositories
        ws.repositories = vec![
            hive::desktop::git::repository::RepositoryInfo {
                path: PathBuf::from("/home/user/projects/hive"),
                name: "hive".to_string(),
                remote_url: Some("https://github.com/user/hive.git".to_string()),
                current_branch: "main".to_string(),
                ahead: 0,
                behind: 0,
                modified_files: 0,
                staged_files: 0,
                has_conflicts: false,
            },
            hive::desktop::git::repository::RepositoryInfo {
                path: PathBuf::from("/home/user/projects/another-project"),
                name: "another-project".to_string(),
                remote_url: Some("https://github.com/user/another-project.git".to_string()),
                current_branch: "develop".to_string(),
                ahead: 2,
                behind: 1,
                modified_files: 3,
                staged_files: 1,
                has_conflicts: false,
            },
        ];
        ws.active_repository = Some(PathBuf::from("/home/user/projects/hive"));
        ws
    });
    
    // Update repository selector with current active repository
    use_effect(move || {
        let ws = workspace_state.read();
        if let Some(active_repo) = ws.get_active_repository() {
            status_bar_state.with_mut(|state| {
                state.update_repository_selector(
                    &active_repo.name,
                    &active_repo.path.to_string_lossy()
                );
            });
        }
    });
    
    // Subscribe to repository change events
    use_effect(move || {
        let event_bus_clone = event_bus.clone();
        spawn(async move {
            let mut receiver = event_bus_clone.read().subscribe();
            while let Ok(event) = receiver.recv().await {
                if event.event_type == EventType::RepositoryChanged {
                    println!("Repository changed!");
                }
            }
        });
    });
    
    rsx! {
        style { {STATUS_BAR_STYLES} }
        style { {DEMO_STYLES} }
        
        div {
            class: "demo-container",
            
            h1 { "Repository Selector Demo" }
            
            div {
                class: "status-bar-container",
                EnhancedStatusBar {
                    state: status_bar_state,
                    on_item_click: move |item_id: String| {
                        println!("Status bar item clicked: {}", item_id);
                    },
                    on_repository_selector_click: {
                        let event_bus = event_bus.clone();
                        move |_| {
                            println!("Repository selector clicked!");
                            // Emit event for repository selector
                            event_bus.read().publish(Event::empty(EventType::RepositorySelectorRequested));
                            
                            // In a real app, this would open a repository selector dialog/menu
                            // For demo, we'll just switch to the next repository
                            workspace_state.with_mut(|ws| {
                                let repos = &ws.repositories;
                                if let Some(current) = &ws.active_repository {
                                    // Find current index
                                    if let Some(idx) = repos.iter().position(|r| &r.path == current) {
                                        // Switch to next repository (wrap around)
                                        let next_idx = (idx + 1) % repos.len();
                                        let next_repo = &repos[next_idx];
                                        ws.active_repository = Some(next_repo.path.clone());
                                        
                                        // Update status bar
                                        status_bar_state.with_mut(|state| {
                                            state.update_repository_selector(
                                                &next_repo.name,
                                                &next_repo.path.to_string_lossy()
                                            );
                                            state.update_git_branch(&next_repo.current_branch);
                                            state.update_git_sync_status(next_repo.ahead, next_repo.behind);
                                        });
                                        
                                        // Emit repository changed event
                                        event_bus.read().publish(Event::repository_changed(
                                            next_repo.path.clone(),
                                            next_repo.name.clone(),
                                            next_repo.remote_url.clone(),
                                        ));
                                    }
                                }
                            });
                        }
                    },
                }
            }
            
            div {
                class: "info-panel",
                h2 { "Workspace Information" }
                
                {
                    let ws = workspace_state.read();
                    rsx! {
                        p { "Root: {ws.root_path.display()}" }
                        p { "Repositories: {ws.repositories.len()}" }
                        
                        if let Some(active) = ws.get_active_repository() {
                            div {
                                class: "active-repo",
                                h3 { "Active Repository" }
                                p { "Name: {active.name}" }
                                p { "Path: {active.path.display()}" }
                                p { "Branch: {active.current_branch}" }
                                p { "Changes: {active.modified_files} modified, {active.staged_files} staged" }
                            }
                        }
                        
                        h3 { "All Repositories" }
                        ul {
                            for repo in &ws.repositories {
                                li {
                                    class: if Some(&repo.path) == ws.active_repository.as_ref() { "active" } else { "" },
                                    "{repo.name} ({repo.current_branch})"
                                }
                            }
                        }
                    }
                }
            }
            
            p {
                class: "help-text",
                "Click on the repository name in the status bar to cycle through repositories"
            }
        }
    }
}

const DEMO_STYLES: &str = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    margin: 0;
    padding: 20px;
    background-color: #1e1e1e;
    color: #d4d4d4;
}

.demo-container {
    max-width: 1200px;
    margin: 0 auto;
}

h1, h2, h3 {
    color: #fff;
}

.status-bar-container {
    margin: 20px 0;
    border: 2px solid #444;
    border-radius: 4px;
    overflow: hidden;
}

.info-panel {
    background-color: #252526;
    padding: 20px;
    border-radius: 4px;
    margin: 20px 0;
}

.active-repo {
    background-color: #2d2d30;
    padding: 15px;
    border-radius: 4px;
    margin: 10px 0;
}

ul {
    list-style: none;
    padding-left: 0;
}

li {
    padding: 5px 10px;
    margin: 2px 0;
    background-color: #2d2d30;
    border-radius: 3px;
}

li.active {
    background-color: #094771;
    font-weight: bold;
}

.help-text {
    color: #888;
    font-style: italic;
    margin-top: 30px;
}

/* Ensure codicon font is loaded for the demo */
@font-face {
    font-family: 'codicon';
    src: url('https://microsoft.github.io/vscode-codicons/dist/codicon.ttf') format('truetype');
}

.codicon-folder:before { content: '\ea83' }
.codicon-source-control:before { content: '\eaff' }
.codicon-sync:before { content: '\ea77' }
.codicon-error:before { content: '\ea87' }
.codicon-warning:before { content: '\ea6c' }
.codicon-circuit-board:before { content: '\eba4' }
.codicon-dashboard:before { content: '\eb9a' }
.codicon-feedback:before { content: '\eb8f' }
"#;