//! Git Module Integration with Event Bus
//!
//! Example of how to integrate the event bus with the Git module

use super::{event_bus, Event, EventType};
use crate::desktop::git::{GitRepository, FileStatus, StatusType};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Setup Git module event publishers
pub async fn setup_git_event_publishers(repo: &GitRepository) -> Result<()> {
    let bus = event_bus();
    
    // Watch for repository changes
    let repo_path = repo.path().to_path_buf();
    let repo_name = repo.name().to_string();
    let remote_url = repo.remote_url("origin").ok();
    
    // Publish repository changed event
    let event = Event::repository_changed(repo_path.clone(), repo_name, remote_url);
    bus.publish_async(event).await?;
    
    Ok(())
}

/// Publish git status changes
pub async fn publish_git_status_update(
    branch: String,
    statuses: Vec<(PathBuf, FileStatus)>,
) -> Result<()> {
    let bus = event_bus();
    
    // Separate modified and staged files
    let mut modified_files = Vec::new();
    let mut staged_files = Vec::new();
    
    for (path, status) in statuses {
        match status.status_type {
            StatusType::Modified | StatusType::Deleted => {
                modified_files.push(path.to_string_lossy().to_string());
            }
            StatusType::Added | StatusType::Renamed => {
                staged_files.push(path.to_string_lossy().to_string());
            }
            _ => {}
        }
    }
    
    let event = Event::git_status_changed(branch, modified_files, staged_files);
    bus.publish_async(event).await?;
    
    Ok(())
}

/// Subscribe to file changes and update git status
pub async fn setup_git_file_watcher() -> Result<()> {
    let bus = event_bus();
    
    bus.subscribe_async(EventType::FileChanged, |event| async move {
        if let super::EventPayload::FileChange { path, .. } = &event.payload {
            info!("Git: File changed, refreshing status for {:?}", path);
            
            // In a real implementation, this would:
            // 1. Check if the file is in a git repository
            // 2. Update the git status for that file
            // 3. Publish a GitStatusChanged event if the status changed
            
            // Example:
            // if let Some(repo) = find_repository_for_path(path) {
            //     let status = repo.get_file_status(path)?;
            //     update_git_cache(path, status);
            //     publish_git_status_update(...).await?;
            // }
        }
        Ok(())
    })
    .await;
    
    Ok(())
}

/// Example: Git watcher integration
pub mod git_watcher_integration {
    use super::*;
    use crate::desktop::git::GitEvent;
    
    /// Convert GitEvent to Event Bus events
    pub async fn handle_git_event(git_event: GitEvent) -> Result<()> {
        let bus = event_bus();
        
        match git_event {
            GitEvent::StatusChanged { repository, branch } => {
                // Get current status and publish
                // let statuses = repository.get_all_statuses()?;
                // publish_git_status_update(branch, statuses).await?;
                
                info!("Git event: Status changed in {} on branch {}", repository, branch);
            }
            GitEvent::BranchChanged { repository, old_branch, new_branch } => {
                info!("Git event: Branch changed from {} to {} in {}", old_branch, new_branch, repository);
                
                // Also publish a git status update for the new branch
                // let statuses = get_branch_status(&new_branch)?;
                // publish_git_status_update(new_branch, statuses).await?;
            }
            GitEvent::RemoteChanged { repository } => {
                info!("Git event: Remote changed in {}", repository);
                
                // Could trigger a repository changed event
                // let event = Event::repository_changed(...);
                // bus.publish_async(event).await?;
            }
            GitEvent::CommitDetected { repository, commit_hash } => {
                info!("Git event: New commit {} in {}", commit_hash, repository);
                
                // Could trigger status update
            }
        }
        
        Ok(())
    }
}