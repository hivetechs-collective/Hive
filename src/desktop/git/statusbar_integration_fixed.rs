//! GitState to StatusBar Integration (Fixed Version)
//! 
//! Provides reactive updates from GitState to StatusBar via Event Bus

use dioxus::prelude::*;
use crate::desktop::events::{event_bus, Event, EventType, EventPayload};
use crate::desktop::git::{GitState, GitWatcher, GitEvent as GitWatcherEvent, SyncStatus};
use crate::desktop::status_bar_enhanced::{StatusBarState, StatusBarItem};
use crate::desktop::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, debug, error};

/// Setup GitState to StatusBar reactive updates using Dioxus effects
/// 
/// This function creates effects that:
/// 1. Watch for GitState changes and update StatusBar
/// 2. Publish GitStatusChanged events to the Event Bus
pub fn setup_git_statusbar_integration(
    git_state: GitState,
    mut status_bar_state: Signal<StatusBarState>,
) {
    // Effect to watch GitState changes and update StatusBar
    use_effect(move || {
        // Read current git state
        let branch_info = git_state.branch_info.read().clone();
        let sync_status = git_state.sync_status.read().clone();
        let file_statuses = git_state.file_statuses.read().clone();
        
        // Update status bar items
        if let Some(branch_info) = &branch_info {
            status_bar_state.write().update_item("git-branch", branch_info.name.clone());
            
            // Update tooltip
            let mut state = status_bar_state.write();
            if let Some(item) = state.items.iter_mut().find(|i| i.id == "git-branch") {
                item.tooltip = Some(format!(
                    "Git: {} (click to checkout branch)", 
                    branch_info.name
                ));
            }
        }
        
        // Update sync status
        let sync_text = format_sync_status(&sync_status);
        status_bar_state.write().update_item("git-sync", sync_text.0);
        
        // Update sync tooltip
        let mut state = status_bar_state.write();
        if let Some(item) = state.items.iter_mut().find(|i| i.id == "git-sync") {
            item.tooltip = Some(sync_text.1);
        }
        
        // Publish event to Event Bus
        spawn(async move {
            let branch_name = branch_info
                .as_ref()
                .map(|b| b.name.clone())
                .unwrap_or_else(|| "main".to_string());
            
            let modified_files: Vec<String> = file_statuses
                .iter()
                .filter_map(|(path, status)| {
                    use crate::desktop::git::StatusType;
                    match status.status_type {
                        StatusType::Modified | StatusType::Deleted => {
                            Some(path.to_string_lossy().to_string())
                        }
                        _ => None,
                    }
                })
                .collect();
            
            let staged_files: Vec<String> = file_statuses
                .iter()
                .filter_map(|(path, status)| {
                    use crate::desktop::git::StatusType;
                    match status.status_type {
                        StatusType::Added | StatusType::Renamed => {
                            Some(path.to_string_lossy().to_string())
                        }
                        _ => None,
                    }
                })
                .collect();
            
            let bus = event_bus();
            let event = Event::git_status_changed(branch_name, modified_files, staged_files);
            if let Err(e) = bus.publish_async(event).await {
                error!("Failed to publish git status changed event: {}", e);
            }
        });
    });
}

/// Setup GitWatcher integration to refresh GitState
pub fn setup_git_watcher_integration(
    git_state: GitState,
    mut active_git_watcher: Signal<Option<GitWatcher>>,
    repo_path: PathBuf,
    app_state: Signal<AppState>,
) {
    spawn(async move {
        match GitWatcher::new(&repo_path) {
            Ok((watcher, mut event_rx)) => {
                info!("Git watcher started for: {:?}", repo_path);
                active_git_watcher.set(Some(watcher));
                
                // Process git watcher events
                while let Some(event) = event_rx.recv().await {
                    debug!("Git watcher event: {:?}", event);
                    
                    match event {
                        GitWatcherEvent::StatusChanged | GitWatcherEvent::BranchChanged => {
                            // Trigger refresh
                            let repo_path = repo_path.clone();
                            if let Err(e) = git_state.refresh_status(&repo_path).await {
                                error!("Failed to refresh git status: {}", e);
                            }
                        }
                        GitWatcherEvent::FileStatusChanged(changed_files) => {
                            // Refresh git status
                            let repo_path = repo_path.clone();
                            if let Err(e) = git_state.refresh_status(&repo_path).await {
                                error!("Failed to refresh git status: {}", e);
                            }
                            
                            // Trigger file explorer refresh to update git status indicators
                            let mut app_state_clone = app_state.clone();
                            spawn(async move {
                                if let Err(e) = app_state_clone.write().file_explorer.refresh().await {
                                    error!("Failed to refresh file explorer: {}", e);
                                } else {
                                    debug!("File explorer refreshed for {:?} changed files", changed_files.len());
                                }
                            });
                        }
                        GitWatcherEvent::RemoteChanged => {
                            debug!("Remote changed, consider fetching");
                        }
                        GitWatcherEvent::ConfigChanged => {
                            debug!("Git config changed");
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create git watcher: {}", e);
            }
        }
    });
}

/// Setup StatusBar subscription to GitStatusChanged events using a channel
pub fn setup_statusbar_event_subscription(
    mut status_bar_state: Signal<StatusBarState>,
) {
    spawn(async move {
        let bus = event_bus();
        
        // Create a channel for event updates
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Subscribe with a simple closure that sends to channel
        bus.subscribe_async(EventType::GitStatusChanged, move |event| {
            let tx = tx.clone();
            async move {
                let _ = tx.send(event);
                Ok(())
            }
        }).await;
        
        // Process events from channel
        while let Some(event) = rx.recv().await {
            if let EventPayload::GitStatus { branch, modified_files, staged_files } = &event.payload {
                // Update status bar
                status_bar_state.write().update_item("git-branch", branch.clone());
                
                // Update tooltip if there are changes
                let total_changes = modified_files.len() + staged_files.len();
                if total_changes > 0 {
                    let mut state = status_bar_state.write();
                    if let Some(item) = state.items.iter_mut().find(|i| i.id == "git-branch") {
                        item.tooltip = Some(format!(
                            "Git: {} ({} changes)", 
                            branch, 
                            total_changes
                        ));
                    }
                }
                
                info!("StatusBar updated from GitStatusChanged event");
            }
        }
    });
}

/// Format sync status for display
fn format_sync_status(sync_status: &SyncStatus) -> (String, String) {
    if !sync_status.has_upstream {
        ("Publish Branch".to_string(), "No upstream branch set (click to publish)".to_string())
    } else if sync_status.is_syncing {
        ("Syncing...".to_string(), "Synchronizing with remote".to_string())
    } else if sync_status.behind == 0 && sync_status.ahead == 0 {
        ("✓".to_string(), "Up to date with remote".to_string())
    } else {
        let text = format!("↓{} ↑{}", sync_status.behind, sync_status.ahead);
        let tooltip = format!(
            "Behind: {}, Ahead: {} (click to sync)", 
            sync_status.behind, 
            sync_status.ahead
        );
        (text, tooltip)
    }
}

/// Initialize complete GitState to StatusBar integration
/// 
/// Call this from your main App component after initializing both GitState and StatusBarState
pub fn initialize_git_statusbar_integration(
    git_state: GitState,
    status_bar_state: Signal<StatusBarState>,
    active_git_watcher: Signal<Option<GitWatcher>>,
    repo_path: Option<PathBuf>,
    app_state: Signal<AppState>,
) {
    // Setup reactive updates
    setup_git_statusbar_integration(git_state.clone(), status_bar_state);
    
    // Setup git watcher if we have a repo path
    if let Some(path) = repo_path {
        setup_git_watcher_integration(git_state, active_git_watcher, path, app_state);
    }
    
    // Setup event bus subscription
    setup_statusbar_event_subscription(status_bar_state);
}