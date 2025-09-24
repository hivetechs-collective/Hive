//! Event Bus Usage Examples
//!
//! Shows how to integrate the event bus with various desktop components

use super::{event_bus, ConsensusStage, Event, EventPayload, EventType, FileChangeType};
use anyhow::Result;
use std::path::PathBuf;

/// Example: Git component publishing status changes
pub async fn publish_git_status_change(
    branch: String,
    modified: Vec<String>,
    staged: Vec<String>,
) -> Result<()> {
    let bus = event_bus();
    let event = Event::git_status_changed(branch, modified, staged);
    bus.publish_async(event).await?;
    Ok(())
}

/// Example: File watcher publishing file changes
pub async fn publish_file_change(path: PathBuf, change_type: FileChangeType) -> Result<()> {
    let bus = event_bus();
    let event = Event::file_changed(path, change_type);
    bus.publish_async(event).await?;
    Ok(())
}

/// Example: Build system publishing build events
pub async fn publish_build_events() -> Result<()> {
    let bus = event_bus();

    // Build started
    let start_event = Event::build_started("release".to_string(), "optimized".to_string());
    bus.publish_async(start_event).await?;

    // Simulate build process...
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Build completed
    let complete_event =
        Event::build_completed(true, vec![], vec!["unused variable: `x`".to_string()], 2000);
    bus.publish_async(complete_event).await?;

    Ok(())
}

/// Example: Status bar subscribing to multiple event types
pub async fn setup_status_bar_subscriptions() -> Result<()> {
    let bus = event_bus();

    // Subscribe to git status changes
    bus.subscribe(EventType::GitStatusChanged, |event| {
        if let EventPayload::GitStatus {
            branch,
            modified_files,
            ..
        } = &event.payload
        {
            tracing::info!(
                "Status bar: Git branch {} with {} modified files",
                branch,
                modified_files.len()
            );
        }
        Ok(())
    })
    .await;

    // Subscribe to build events
    bus.subscribe(EventType::BuildStarted, |event| {
        if let EventPayload::BuildInfo { target, profile } = &event.payload {
            tracing::info!("Status bar: Build started - {} {}", target, profile);
        }
        Ok(())
    })
    .await;

    bus.subscribe(EventType::BuildCompleted, |event| {
        if let EventPayload::BuildResult {
            success,
            duration_ms,
            ..
        } = &event.payload
        {
            tracing::info!(
                "Status bar: Build {} in {}ms",
                if *success { "succeeded" } else { "failed" },
                duration_ms
            );
        }
        Ok(())
    })
    .await;

    Ok(())
}

/// Example: Problems panel subscribing to diagnostics
pub async fn setup_problems_panel() -> Result<()> {
    let bus = event_bus();

    bus.subscribe_async(EventType::ProblemsUpdated, |event| async move {
        if let EventPayload::ProblemsUpdate {
            added,
            removed,
            total_count,
        } = &event.payload
        {
            tracing::info!(
                "Problems panel: {} added, {} removed, {} total",
                added.len(),
                removed.len(),
                total_count
            );

            // Update UI with new problems...
            // This could trigger re-rendering of the problems list
        }
        Ok(())
    })
    .await;

    Ok(())
}

/// Example: Consensus UI subscribing to consensus state changes
pub async fn setup_consensus_ui() -> Result<()> {
    let bus = event_bus();

    bus.subscribe_async(EventType::ConsensusStateChanged, |event| async move {
        if let EventPayload::ConsensusState {
            stage,
            progress,
            message,
        } = &event.payload
        {
            match stage {
                ConsensusStage::Initializing => {
                    tracing::info!("Consensus UI: Initializing...");
                }
                ConsensusStage::Generating => {
                    tracing::info!("Consensus UI: Generating ({}%)", progress);
                }
                ConsensusStage::Refining => {
                    tracing::info!("Consensus UI: Refining ({}%)", progress);
                }
                ConsensusStage::Validating => {
                    tracing::info!("Consensus UI: Validating ({}%)", progress);
                }
                ConsensusStage::Curating => {
                    tracing::info!("Consensus UI: Curating ({}%)", progress);
                }
                ConsensusStage::Completed => {
                    tracing::info!("Consensus UI: Completed!");
                    if let Some(msg) = message {
                        tracing::info!("Result: {}", msg);
                    }
                }
                ConsensusStage::Failed => {
                    tracing::error!("Consensus UI: Failed!");
                    if let Some(msg) = message {
                        tracing::error!("Error: {}", msg);
                    }
                }
            }

            // Update progress bar, stage indicator, etc.
        }
        Ok(())
    })
    .await;

    Ok(())
}

/// Example: File explorer reacting to file system changes
pub async fn setup_file_explorer() -> Result<()> {
    let bus = event_bus();

    // Subscribe to file changes
    let handle = bus
        .subscribe_async(EventType::FileChanged, |event| async move {
            if let EventPayload::FileChange { path, change_type } = &event.payload {
                match change_type {
                    FileChangeType::Created => {
                        tracing::info!("File explorer: New file created: {:?}", path);
                        // Add to tree view...
                    }
                    FileChangeType::Modified => {
                        tracing::info!("File explorer: File modified: {:?}", path);
                        // Update icon/status...
                    }
                    FileChangeType::Deleted => {
                        tracing::info!("File explorer: File deleted: {:?}", path);
                        // Remove from tree view...
                    }
                    FileChangeType::Renamed { old_path } => {
                        tracing::info!(
                            "File explorer: File renamed from {:?} to {:?}",
                            old_path,
                            path
                        );
                        // Update tree view...
                    }
                }
            }
            Ok(())
        })
        .await;

    // Subscribe to repository changes
    bus.subscribe_async(EventType::RepositoryChanged, |event| async move {
        if let EventPayload::RepositoryInfo { path, name, .. } = &event.payload {
            tracing::info!(
                "File explorer: Repository changed to {} at {:?}",
                name,
                path
            );
            // Reload file tree for new repository...
        }
        Ok(())
    })
    .await;

    // Later, if needed, unsubscribe
    // bus.unsubscribe(&handle).await?;

    Ok(())
}

/// Example: Terminal component publishing output
pub async fn publish_terminal_output(terminal_id: String, data: String) -> Result<()> {
    let bus = event_bus();
    let event = Event::terminal_output(terminal_id, data);
    bus.publish_async(event).await?;
    Ok(())
}

/// Example: Configuration component handling config changes
pub async fn setup_config_handler() -> Result<()> {
    let bus = event_bus();

    bus.subscribe_async(EventType::ConfigurationChanged, |event| async move {
        if let EventPayload::ConfigChange {
            section,
            key,
            old_value,
            new_value,
        } = &event.payload
        {
            tracing::info!(
                "Config changed: {}.{} from {:?} to {}",
                section,
                key,
                old_value,
                new_value
            );

            // Apply configuration changes...
            // This might trigger theme changes, reload components, etc.
        }
        Ok(())
    })
    .await;

    Ok(())
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_usage() {
        // Setup subscribers
        setup_status_bar_subscriptions().await.unwrap();
        setup_file_explorer().await.unwrap();

        // Publish some events
        publish_git_status_change("main".to_string(), vec!["src/main.rs".to_string()], vec![])
            .await
            .unwrap();

        publish_file_change(PathBuf::from("/src/lib.rs"), FileChangeType::Modified)
            .await
            .unwrap();

        // Give async handlers time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
