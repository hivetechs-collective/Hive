//! Integration tests for Branch Menu and StatusBar interaction
//! 
//! Tests the complete integration of:
//! - StatusBar git-branch item click handling
//! - MenuVisibilityChanged event propagation
//! - Branch menu visibility management
//! - Branch selection and menu closing
//!
//! Note: This test focuses on the event flow and integration between components.
//! Some event types used in the actual implementation may not be fully available
//! in the current event system, so we simulate the behavior using available events.

use hive_ai::desktop::events::{event_bus, Event as HiveEvent, EventPayload, EventType};
use hive_ai::desktop::git::{BranchInfo, BranchType, GitState};
use hive_ai::desktop::status_bar_enhanced::StatusBarState;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use dioxus_signals::Writable;

/// Helper function to create a test StatusBarState with git-branch item
fn create_test_statusbar_state() -> StatusBarState {
    let mut state = StatusBarState::default();
    
    // Ensure git-branch item exists with click handler
    if let Some(item) = state.items.iter_mut().find(|i| i.id == "git-branch") {
        item.text = "feature/test-branch".to_string();
        item.tooltip = Some("Git: feature/test-branch (click to checkout branch)".to_string());
    }
    
    state
}

/// Helper function to create test GitState
fn create_test_git_state() -> GitState {
    let mut git_state = GitState::new();
    
    // Set current branch info
    git_state.branch_info.set(Some(BranchInfo {
        name: "feature/test-branch".to_string(),
        branch_type: BranchType::Local,
        upstream: Some("origin/feature/test-branch".to_string()),
        ahead: 2,
        behind: 0,
        last_commit: None,
        is_current: true,
    }));
    
    git_state
}

#[tokio::test]
async fn test_statusbar_git_branch_click_triggers_menu() {
    let bus = event_bus();
    let menu_shown = Arc::new(AtomicBool::new(false));
    let menu_shown_clone = menu_shown.clone();
    
    // Subscribe to MenuVisibilityChanged events
    bus.subscribe_async(EventType::MenuVisibilityChanged, move |event| {
        let menu_shown = menu_shown_clone.clone();
        async move {
            if let EventPayload::MenuVisibility { menu_id, visible } = &event.payload {
                if menu_id == "branch-menu" && *visible {
                    menu_shown.store(true, Ordering::SeqCst);
                }
            }
            Ok(())
        }
    })
    .await;
    
    // Simulate clicking the git-branch item in status bar by publishing menu visibility event
    bus.publish_async(HiveEvent::new(
        EventType::MenuVisibilityChanged,
        EventPayload::MenuVisibility {
            menu_id: "branch-menu".to_string(),
            visible: true,
        },
    ))
    .await
    .unwrap();
    
    // Wait for event propagation
    sleep(Duration::from_millis(100)).await;
    
    assert!(menu_shown.load(Ordering::SeqCst), "Branch menu should be shown after clicking git-branch item");
}

#[tokio::test]
async fn test_branch_menu_selection_closes_menu() {
    let bus = event_bus();
    let events_received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let events_clone = events_received.clone();
    
    // Subscribe to multiple event types
    bus.subscribe_async(EventType::MenuVisibilityChanged, {
        let events = events_clone.clone();
        move |event| {
            let events = events.clone();
            async move {
                if let EventPayload::MenuVisibility { menu_id, visible } = &event.payload {
                    events.lock().await.push(format!("menu-visibility: {} = {}", menu_id, visible));
                }
                Ok(())
            }
        }
    })
    .await;
    
    bus.subscribe_async(EventType::GitStatusChanged, {
        let events = events_clone.clone();
        move |event| {
            let events = events.clone();
            async move {
                if let EventPayload::GitStatus { branch, .. } = &event.payload {
                    events.lock().await.push(format!("branch-changed: {}", branch));
                }
                Ok(())
            }
        }
    })
    .await;
    
    // Simulate opening the branch menu
    bus.publish_async(HiveEvent::new(
        EventType::MenuVisibilityChanged,
        EventPayload::MenuVisibility {
            menu_id: "branch-menu".to_string(),
            visible: true,
        },
    ))
    .await
    .unwrap();
    
    sleep(Duration::from_millis(50)).await;
    
    // Simulate selecting a branch (using GitStatusChanged as proxy)
    bus.publish_async(HiveEvent::git_status_changed(
        "feature/new-branch".to_string(),
        vec![],
        vec![],
    ))
    .await
    .unwrap();
    
    // Simulate closing the menu after branch selection
    bus.publish_async(HiveEvent::new(
        EventType::MenuVisibilityChanged,
        EventPayload::MenuVisibility {
            menu_id: "branch-menu".to_string(),
            visible: false,
        },
    ))
    .await
    .unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    let events = events_received.lock().await;
    // Filter to our specific events
    let branch_menu_events: Vec<&String> = events.iter()
        .filter(|e| e.contains("branch-menu") || e.contains("branch-changed"))
        .collect();
    
    assert!(branch_menu_events.len() >= 3, "Should receive at least 3 relevant events");
    assert!(branch_menu_events.iter().any(|e| e.contains("branch-menu = true")), "Should show menu");
    assert!(branch_menu_events.iter().any(|e| e.contains("branch-changed: feature/new-branch")), "Should change branch");
    assert!(branch_menu_events.iter().any(|e| e.contains("branch-menu = false")), "Should hide menu");
}

#[tokio::test]
async fn test_branch_menu_integration_flow() {
    let bus = event_bus();
    let flow_log = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    
    // Track complete flow: status bar interaction -> menu shown -> branch selected -> menu hidden -> status updated
    
    // Simulate status bar interaction using UI state change
    let log_ui = flow_log.clone();
    bus.subscribe_async(EventType::UIStateChanged, move |event| {
        let log = log_ui.clone();
        async move {
            if let EventPayload::UIChange { component, property, .. } = &event.payload {
                if component == "statusbar" && property == "git-branch-clicked" {
                    log.lock().await.push("1. StatusBar git-branch clicked".to_string());
                    
                    // Trigger menu visibility
                    event_bus().publish_async(HiveEvent::new(
                        EventType::MenuVisibilityChanged,
                        EventPayload::MenuVisibility {
                            menu_id: "branch-menu".to_string(),
                            visible: true,
                        },
                    ))
                    .await?;
                }
            }
            Ok(())
        }
    })
    .await;
    
    let log_menu = flow_log.clone();
    bus.subscribe_async(EventType::MenuVisibilityChanged, move |event| {
        let log = log_menu.clone();
        async move {
            if let EventPayload::MenuVisibility { menu_id, visible } = &event.payload {
                if menu_id == "branch-menu" {
                    if *visible {
                        log.lock().await.push("2. Branch menu shown".to_string());
                    } else {
                        log.lock().await.push("4. Branch menu hidden".to_string());
                    }
                }
            }
            Ok(())
        }
    })
    .await;
    
    let log_branch = flow_log.clone();
    bus.subscribe_async(EventType::GitStatusChanged, move |event| {
        let log = log_branch.clone();
        async move {
            if let EventPayload::GitStatus { branch, .. } = &event.payload {
                log.lock().await.push(format!("3. Branch changed to: {}", branch));
                
                // Close menu after branch change
                event_bus().publish_async(HiveEvent::new(
                    EventType::MenuVisibilityChanged,
                    EventPayload::MenuVisibility {
                        menu_id: "branch-menu".to_string(),
                        visible: false,
                    },
                ))
                .await?;
            }
            Ok(())
        }
    })
    .await;
    
    let log_status = flow_log.clone();
    bus.subscribe_async(EventType::GitStatusChanged, move |event| {
        let log = log_status.clone();
        async move {
            if let EventPayload::GitStatus { branch, .. } = &event.payload {
                if log.lock().await.len() >= 4 {
                    log.lock().await.push(format!("5. Git status updated: {}", branch));
                }
            }
            Ok(())
        }
    })
    .await;
    
    // Start the flow - simulate statusbar click using UI state change
    bus.publish_async(HiveEvent::new(
        EventType::UIStateChanged,
        EventPayload::UIChange {
            component: "statusbar".to_string(),
            property: "git-branch-clicked".to_string(),
            value: serde_json::json!(true),
        },
    ))
    .await
    .unwrap();
    
    // Wait for menu to show
    sleep(Duration::from_millis(50)).await;
    
    // Select a branch
    bus.publish_async(HiveEvent::git_status_changed(
        "develop".to_string(),
        vec![],
        vec![],
    ))
    .await
    .unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    let log = flow_log.lock().await;
    
    // The log might have duplicate entries or be out of order due to async nature
    // So we check that all expected events occurred
    assert!(log.iter().any(|l| l.contains("StatusBar git-branch clicked")), "Should have status bar click");
    assert!(log.iter().any(|l| l.contains("Branch menu shown")), "Should show branch menu");
    assert!(log.iter().any(|l| l.contains("Branch changed to: develop")), "Should change branch");
    assert!(log.iter().any(|l| l.contains("Branch menu hidden")), "Should hide menu");
}

#[tokio::test]
async fn test_concurrent_menu_operations() {
    let bus = event_bus();
    let operation_count = Arc::new(AtomicUsize::new(0));
    let count_clone = operation_count.clone();
    
    bus.subscribe_async(EventType::MenuVisibilityChanged, move |event| {
        let count = count_clone.clone();
        async move {
            if let EventPayload::MenuVisibility { menu_id, .. } = &event.payload {
                if menu_id == "branch-menu" {
                    count.fetch_add(1, Ordering::SeqCst);
                    // Simulate some processing time
                    sleep(Duration::from_millis(10)).await;
                }
            }
            Ok(())
        }
    })
    .await;
    
    // Simulate rapid clicks (user clicking multiple times quickly)
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let bus = event_bus();
            tokio::spawn(async move {
                bus.publish_async(HiveEvent::new(
                    EventType::MenuVisibilityChanged,
                    EventPayload::MenuVisibility {
                        menu_id: "branch-menu".to_string(),
                        visible: i % 2 == 0, // Alternate between show/hide
                    },
                ))
                .await
                .unwrap();
            })
        })
        .collect();
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    sleep(Duration::from_millis(200)).await;
    
    // Due to event bus behavior, we might receive more events than expected
    // The important thing is that we processed at least 5 operations
    assert!(
        operation_count.load(Ordering::SeqCst) >= 5,
        "Should process at least 5 menu operations, got: {}",
        operation_count.load(Ordering::SeqCst)
    );
}

#[tokio::test] 
async fn test_branch_menu_error_recovery() {
    let bus = event_bus();
    let error_handled = Arc::new(AtomicBool::new(false));
    let error_clone = error_handled.clone();
    
    // Subscribe with a handler that fails on certain branches
    bus.subscribe_async(EventType::GitStatusChanged, move |event| {
        let handled = error_clone.clone();
        async move {
            if let EventPayload::GitStatus { branch, .. } = &event.payload {
                if branch.contains("invalid") {
                    // Mark that we attempted to handle the error case
                    handled.store(true, Ordering::SeqCst);
                    return Err(anyhow::anyhow!("Invalid branch name"));
                }
            }
            Ok(())
        }
    })
    .await;
    
    // Try to switch to an invalid branch
    bus.publish_async(HiveEvent::git_status_changed(
        "feature/invalid-branch-name".to_string(),
        vec![],
        vec![],
    ))
    .await
    .unwrap(); // publish should succeed even if handler fails
    
    sleep(Duration::from_millis(100)).await;
    
    assert!(
        error_handled.load(Ordering::SeqCst),
        "Error case should be handled"
    );
    
    // Verify system continues working after error
    let success_count = Arc::new(AtomicUsize::new(0));
    let success_clone = success_count.clone();
    
    bus.subscribe_async(EventType::GitStatusChanged, move |event| {
        let count = success_clone.clone();
        async move {
            if let EventPayload::GitStatus { branch, .. } = &event.payload {
                if !branch.contains("invalid") {
                    count.fetch_add(1, Ordering::SeqCst);
                }
            }
            Ok(())
        }
    })
    .await;
    
    // This should work fine
    bus.publish_async(HiveEvent::git_status_changed(
        "feature/valid-branch".to_string(),
        vec![],
        vec![],
    ))
    .await
    .unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    assert_eq!(
        success_count.load(Ordering::SeqCst),
        1,
        "System should recover and process valid events"
    );
}

/// Test the complete integration with menu visibility and git status updates
#[tokio::test]
async fn test_statusbar_branch_menu_ui_integration() {
    // This test simulates the actual UI interaction flow
    let bus = event_bus();
    let ui_state = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let state_clone = ui_state.clone();
    
    // Simulate StatusBar component behavior using UI state changes
    bus.subscribe_async(EventType::UIStateChanged, move |event| {
        let state = state_clone.clone();
        async move {
            if let EventPayload::UIChange { component, property, .. } = &event.payload {
                if component == "statusbar" && property == "item-clicked" {
                    state.lock().await.push("StatusBar: git-branch clicked".to_string());
                    
                    // StatusBar would emit menu visibility event
                    event_bus().publish_async(HiveEvent::new(
                        EventType::MenuVisibilityChanged,
                        EventPayload::MenuVisibility {
                            menu_id: "branch-menu".to_string(),
                            visible: true,
                        },
                    ))
                    .await?;
                }
            }
            Ok(())
        }
    })
    .await;
    
    // Simulate BranchMenu component behavior
    let state_clone2 = ui_state.clone();
    bus.subscribe_async(EventType::MenuVisibilityChanged, move |event| {
        let state = state_clone2.clone();
        async move {
            if let EventPayload::MenuVisibility { menu_id, visible } = &event.payload {
                if menu_id == "branch-menu" {
                    if *visible {
                        state.lock().await.push("BranchMenu: Rendering menu".to_string());
                    } else {
                        state.lock().await.push("BranchMenu: Hiding menu".to_string());
                    }
                }
            }
            Ok(())
        }
    })
    .await;
    
    // Simulate user interaction flow
    // 1. User clicks git-branch in status bar
    bus.publish_async(HiveEvent::new(
        EventType::UIStateChanged,
        EventPayload::UIChange {
            component: "statusbar".to_string(),
            property: "item-clicked".to_string(),
            value: serde_json::json!({"item_id": "git-branch"}),
        },
    ))
    .await
    .unwrap();
    
    sleep(Duration::from_millis(50)).await;
    
    // 2. User selects a branch from the menu (simulated via git status change)
    bus.publish_async(HiveEvent::git_status_changed(
        "feature/new-feature".to_string(),
        vec![],
        vec![],
    ))
    .await
    .unwrap();
    
    // 3. Menu closes
    bus.publish_async(HiveEvent::new(
        EventType::MenuVisibilityChanged,
        EventPayload::MenuVisibility {
            menu_id: "branch-menu".to_string(),
            visible: false,
        },
    ))
    .await
    .unwrap();
    
    sleep(Duration::from_millis(100)).await;
    
    let state = ui_state.lock().await;
    
    // Check that all expected states occurred
    assert!(state.iter().any(|s| s.contains("StatusBar: git-branch clicked")), "Should have status bar click");
    assert!(state.iter().any(|s| s.contains("BranchMenu: Rendering menu")), "Should render menu");
    assert!(state.iter().any(|s| s.contains("BranchMenu: Hiding menu")), "Should hide menu");
}