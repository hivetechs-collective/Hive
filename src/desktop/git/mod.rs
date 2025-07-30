//! Git integration module for Hive Consensus
//! 
//! Provides VS Code-style git functionality including:
//! - Repository management
//! - Branch operations
//! - File status tracking
//! - Remote operations
//! - Real-time updates

pub mod repository;
pub mod branch;
pub mod status;
pub mod watcher;
pub mod diff;
pub mod operations;
pub mod toolbar;
pub mod context_manager;
pub mod status_menu;
pub mod repository_selector;
pub mod repository_dropdown;
pub mod multi_repo_manager;
pub mod statusbar_integration_fixed;
pub mod branch_menu;
pub mod commit_box;

pub use repository::{GitRepository, RepositoryInfo};
pub use branch::{BranchInfo, BranchType, BranchFilter, BranchSort, CommitInfo, sort_branches, validate_branch_name};
pub use status::{FileStatus, StatusType, SyncStatus};
pub use watcher::{GitWatcher, GitEvent};
pub use diff::{DiffResult, DiffHunk, DiffLine, DiffLineType, DiffViewMode, compute_diff, get_file_diff};
pub use operations::{GitOperations, GitOperation, GitOperationResult, GitOperationProgress, ProgressCallback, CancellationToken};
pub use toolbar::{GitToolbar, GitToolbarProps, OperationState};
pub use context_manager::{GitContextManager, GitContextEvent, use_git_context, provide_git_context};
pub use status_menu::{GitStatusMenu, GitStatusMenuProps};
pub use repository_selector::{
    RepositorySelector, RepositoryInfo as RepoSelectorInfo, RepositoryStatus as RepoStatus,
    RepositorySelectorState, UpstreamStatus, REPOSITORY_SELECTOR_STYLES
};
pub use repository_dropdown::{RepositoryDropdown, RepositoryDropdownProps, REPOSITORY_DROPDOWN_STYLES};
pub use multi_repo_manager::{MultiRepoManager, RepositoriesSummary};
pub use statusbar_integration_fixed::{
    initialize_git_statusbar_integration,
    setup_git_statusbar_integration,
    setup_git_watcher_integration,
    setup_statusbar_event_subscription,
};
pub use branch_menu::{BranchMenu, BranchOperation, BranchOperationResult};
pub use commit_box::{CommitBox, CommitBoxProps, CommitTemplate, ConventionalType};

use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Global git state for the application
#[derive(Clone, PartialEq)]
pub struct GitState {
    pub repositories: Signal<Vec<RepositoryInfo>>,
    pub active_repo: Signal<Option<RepositoryInfo>>,
    pub branch_info: Signal<Option<BranchInfo>>,
    pub file_statuses: Signal<HashMap<PathBuf, FileStatus>>,
    pub sync_status: Signal<SyncStatus>,
}

impl GitState {
    /// Create a new GitState instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Refresh the git status for the given repository path
    pub async fn refresh_status(&self, _repo_path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation would refresh git status for the repository
        // For now, this is a stub that doesn't fail compilation
        Ok(())
    }
}

impl Default for GitState {
    fn default() -> Self {
        Self {
            repositories: Signal::new(vec![]),
            active_repo: Signal::new(None),
            branch_info: Signal::new(None),
            file_statuses: Signal::new(HashMap::new()),
            sync_status: Signal::new(SyncStatus::default()),
        }
    }
}

/// Initialize git state for the application
pub fn use_git_state() -> GitState {
    use_context_provider(|| GitState::default())
}