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

pub use repository::{GitRepository, RepositoryInfo};
pub use branch::{BranchInfo, BranchType};
pub use status::{FileStatus, StatusType, SyncStatus};
pub use watcher::{GitWatcher, GitEvent};
pub use diff::{DiffResult, DiffHunk, DiffLine, DiffLineType, DiffViewMode, compute_diff, get_file_diff};
pub use operations::{GitOperations, GitOperation, GitOperationResult};
pub use toolbar::{GitToolbar, GitToolbarProps};
pub use context_manager::{GitContextManager, GitContextEvent, use_git_context, provide_git_context};

use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Global git state for the application
#[derive(Clone)]
pub struct GitState {
    pub repositories: Signal<Vec<RepositoryInfo>>,
    pub active_repo: Signal<Option<RepositoryInfo>>,
    pub branch_info: Signal<Option<BranchInfo>>,
    pub file_statuses: Signal<HashMap<PathBuf, FileStatus>>,
    pub sync_status: Signal<SyncStatus>,
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