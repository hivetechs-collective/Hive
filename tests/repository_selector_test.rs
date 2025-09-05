//! Tests for repository selector in status bar

use hive::desktop::status_bar_enhanced::{StatusBarState, StatusBarAlignment};
use hive::desktop::events::{EventBus, EventType, Event};
use hive::desktop::workspace::WorkspaceState;
use std::path::PathBuf;

#[test]
fn test_repository_selector_in_status_bar() {
    let mut status_bar = StatusBarState::default();
    
    // Check that repository selector exists
    let repo_selector = status_bar.items.iter()
        .find(|item| item.id == "repository-selector")
        .expect("Repository selector should exist");
    
    assert_eq!(repo_selector.text, "hive");
    assert_eq!(repo_selector.tooltip, Some("Select Repository".to_string()));
    assert_eq!(repo_selector.icon, Some("folder".to_string()));
    assert_eq!(repo_selector.alignment, StatusBarAlignment::Left);
    assert_eq!(repo_selector.priority, 110); // Highest priority
}

#[test]
fn test_update_repository_selector() {
    let mut status_bar = StatusBarState::default();
    
    // Update repository selector
    status_bar.update_repository_selector("my-project", "/home/user/projects/my-project");
    
    // Verify update
    let repo_selector = status_bar.items.iter()
        .find(|item| item.id == "repository-selector")
        .expect("Repository selector should exist");
    
    assert_eq!(repo_selector.text, "my-project");
    assert_eq!(
        repo_selector.tooltip,
        Some("Repository: my-project (/home/user/projects/my-project)".to_string())
    );
}

#[test]
fn test_repository_selector_priority() {
    let status_bar = StatusBarState::default();
    
    // Get all left-aligned items
    let mut left_items: Vec<_> = status_bar.items.iter()
        .filter(|item| item.alignment == StatusBarAlignment::Left)
        .collect();
    
    // Sort by priority (highest first)
    left_items.sort_by(|a, b| b.priority.cmp(&a.priority));
    
    // Repository selector should be first
    assert_eq!(left_items[0].id, "repository-selector");
}

#[test]
fn test_repository_selector_click_event() {
    let event_bus = EventBus::new();
    let mut receiver = event_bus.subscribe();
    
    // Simulate repository selector click
    event_bus.publish(Event::empty(EventType::RepositorySelectorRequested));
    
    // Verify event was published
    let event = receiver.try_recv().expect("Should receive event");
    assert_eq!(event.event_type, EventType::RepositorySelectorRequested);
}

#[test]
fn test_workspace_integration() {
    let mut workspace = WorkspaceState::new(PathBuf::from("/home/user/projects"));
    let mut status_bar = StatusBarState::default();
    
    // Add repositories to workspace
    workspace.repositories = vec![
        hive::desktop::git::repository::RepositoryInfo {
            path: PathBuf::from("/home/user/projects/project1"),
            name: "project1".to_string(),
            remote_url: None,
            current_branch: "main".to_string(),
            ahead: 0,
            behind: 0,
            modified_files: 0,
            staged_files: 0,
            has_conflicts: false,
        },
        hive::desktop::git::repository::RepositoryInfo {
            path: PathBuf::from("/home/user/projects/project2"),
            name: "project2".to_string(),
            remote_url: None,
            current_branch: "develop".to_string(),
            ahead: 0,
            behind: 0,
            modified_files: 0,
            staged_files: 0,
            has_conflicts: false,
        },
    ];
    
    // Set active repository
    workspace.active_repository = Some(PathBuf::from("/home/user/projects/project2"));
    
    // Update status bar based on active repository
    if let Some(active_repo) = workspace.get_active_repository() {
        status_bar.update_repository_selector(
            &active_repo.name,
            &active_repo.path.to_string_lossy()
        );
    }
    
    // Verify status bar shows correct repository
    let repo_selector = status_bar.items.iter()
        .find(|item| item.id == "repository-selector")
        .expect("Repository selector should exist");
    
    assert_eq!(repo_selector.text, "project2");
    assert!(repo_selector.tooltip.as_ref().unwrap().contains("project2"));
}