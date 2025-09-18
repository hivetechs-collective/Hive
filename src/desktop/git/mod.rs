//! Git integration module for Hive Consensus
//!
//! Provides VS Code-style git functionality including:
//! - Repository management
//! - Branch operations
//! - File status tracking
//! - Remote operations
//! - Real-time updates

pub mod action_processor;
pub mod branch;
pub mod branch_menu;
pub mod commit_box;
pub mod context_manager;
pub mod decoration_config_ui;
pub mod decoration_styles;
pub mod decoration_watcher;
pub mod decorations;
pub mod diff;
pub mod enhanced_file_explorer;
pub mod inline_actions;
pub mod keyboard_shortcuts;
pub mod lazygit_controller;
pub mod lazygit_data_provider;
pub mod lazygit_updater;
pub mod lazygit_updater_db;
pub mod multi_repo_manager;
pub mod operations;
pub mod performance;
pub mod performance_config_ui;
pub mod performance_monitor;
pub mod repository;
pub mod repository_dropdown;
pub mod repository_selector;
pub mod stash;
pub mod stash_manager;
pub mod stash_shortcuts;
pub mod stash_ui;
pub mod status;
pub mod status_menu;
pub mod statusbar_integration_fixed;
pub mod toolbar;
pub mod watcher;

pub use action_processor::{DiffActionProcessor, ExecutionResult, ProcessorConfig};
pub use branch::{
    sort_branches, validate_branch_name, BranchFilter, BranchInfo, BranchSort, BranchType,
    CommitInfo,
};
pub use branch_menu::{BranchMenu, BranchOperation, BranchOperationResult};
pub use commit_box::{CommitBox, CommitBoxProps, CommitTemplate, ConventionalType};
pub use context_manager::{
    provide_git_context, use_git_context, GitContextEvent, GitContextManager,
};
pub use decoration_config_ui::{GitDecorationConfigUI, GitDecorationPreview};
pub use decoration_styles::{
    get_conflict_class, get_file_git_classes, get_git_decoration_css_variables,
    get_git_decoration_styles, get_git_status_class, get_inline_git_status_style,
    get_vscode_git_theme_styles, GitDecorationDynamicStyles, GitDecorationStyles,
    VSCodeGitThemeStyles,
};
pub use decoration_watcher::{
    use_git_decoration_watcher, DecorationEvent, DecorationWatcherConfig, GitDecorationWatcher,
};
pub use decorations::{
    use_git_decoration_manager, DecorationStyles, EditorGutterDecorations, EnhancedGitStatus,
    FileGitDecoration, FolderDecoration, GitDecorationConfig, GitDecorationManager,
};
pub use diff::{
    compute_diff, compute_diff_with_context, get_file_diff, get_file_diff_with_context, DiffAction,
    DiffActionResult, DiffActionState, DiffGitOperations, DiffHunk, DiffLine, DiffLineType,
    DiffResult, DiffUndoEntry, DiffViewMode, HunkStageStatus,
};
pub use enhanced_file_explorer::{
    use_git_decorations, EnhancedFileExplorer, GitDecorationIntegration,
};
pub use inline_actions::{ActionTooltip, EnhancedDiffHunk, HunkInlineActions, LineInlineActions};
pub use keyboard_shortcuts::{
    FocusManager, GlobalShortcutHandler, KeyCombination, ShortcutAction, ShortcutConfig,
    ShortcutHelpDialog, ShortcutManager,
};
pub use lazygit_controller::{is_lazygit_available, LazyGitController, LazyGitPanel};
pub use lazygit_data_provider::{
    get_lazygit_file_diff, get_lazygit_file_statuses, LazyGitDataProvider,
};
pub use lazygit_updater::{force_update_lazygit, initialize_lazygit_updater, LazyGitUpdater};
pub use lazygit_updater_db::{
    force_update_lazygit_db, get_lazygit_update_history, initialize_lazygit_updater_db,
    LazyGitUpdaterDB,
};
pub use multi_repo_manager::{MultiRepoManager, RepositoriesSummary};
pub use operations::{
    CancellationToken, GitOperation, GitOperationProgress, GitOperationResult, GitOperations,
    ProgressCallback,
};
pub use performance::{
    BatchProcessor, OptimizedGitManager, PaginatedResult, PerformanceConfig, PerformanceStats,
};
pub use performance_config_ui::{PerformanceConfigProps, PerformanceConfigUI, PerformancePresets};
pub use performance_monitor::{
    AutoRefreshPerformanceMonitor, PerformanceGraph, PerformanceMonitor, PerformanceMonitorProps,
};
pub use repository::{
    clear_performance_caches, get_git_performance_stats, get_optimized_git_manager,
    log_performance_stats, GitRepository, RepositoryInfo,
};
pub use repository_dropdown::{
    RepositoryDropdown, RepositoryDropdownProps, REPOSITORY_DROPDOWN_STYLES,
};
pub use repository_selector::{
    RepositoryInfo as RepoSelectorInfo, RepositorySelector, RepositorySelectorState,
    RepositoryStatus as RepoStatus, UpstreamStatus, REPOSITORY_SELECTOR_STYLES,
};
pub use stash::{GitStash, StashApplyOptions, StashInfo, StashSaveOptions, StashStats};
pub use stash_manager::{StashManager, StashPanel};
pub use stash_shortcuts::{
    handle_stash_shortcut, StashKeyboardHandler, StashShortcutAction, StashShortcutsInfo,
};
pub use stash_ui::{StashAction, StashList, StashPreview, StashSaveDialog};
pub use status::{FileStatus, StatusType, SyncStatus};
pub use status_menu::{GitStatusMenu, GitStatusMenuProps};
pub use statusbar_integration_fixed::{
    initialize_git_statusbar_integration, setup_git_statusbar_integration,
    setup_git_watcher_integration, setup_statusbar_event_subscription,
};
pub use toolbar::{GitToolbar, GitToolbarProps, OperationState};
pub use watcher::{GitEvent, GitWatcher};

use anyhow::Result;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Global git state for the application with performance optimizations
#[derive(Clone)]
pub struct GitState {
    pub repositories: Signal<Vec<RepositoryInfo>>,
    pub active_repo: Signal<Option<RepositoryInfo>>,
    pub branch_info: Signal<Option<BranchInfo>>,
    pub file_statuses: Signal<HashMap<PathBuf, FileStatus>>,
    pub sync_status: Signal<SyncStatus>,
    /// Performance manager for optimized git operations
    pub performance_manager: Arc<OptimizedGitManager>,
}

impl GitState {
    /// Create a new GitState instance with performance optimizations
    pub fn new() -> Self {
        Self {
            repositories: Signal::new(vec![]),
            active_repo: Signal::new(None),
            branch_info: Signal::new(None),
            file_statuses: Signal::new(HashMap::new()),
            sync_status: Signal::new(SyncStatus::default()),
            performance_manager: get_optimized_git_manager(),
        }
    }

    /// Refresh the git status for the given repository path using optimized operations
    pub async fn refresh_status(&self, repo_path: &PathBuf) -> Result<()> {
        // Use the optimized manager to get file statuses
        let file_statuses = self
            .performance_manager
            .get_file_statuses_batched(repo_path)
            .await?;

        // Convert to our internal format
        let mut status_map = HashMap::new();
        for (path, git_status) in file_statuses {
            let file_status = FileStatus {
                path: path.clone(),
                status_type: StatusType::from_git2_status(git_status),
                is_staged: git_status.contains(git2::Status::INDEX_MODIFIED)
                    || git_status.contains(git2::Status::INDEX_NEW)
                    || git_status.contains(git2::Status::INDEX_DELETED),
                has_staged_changes: git_status.contains(git2::Status::INDEX_MODIFIED)
                    || git_status.contains(git2::Status::INDEX_NEW)
                    || git_status.contains(git2::Status::INDEX_DELETED),
                has_unstaged_changes: git_status.contains(git2::Status::WT_MODIFIED)
                    || git_status.contains(git2::Status::WT_NEW)
                    || git_status.contains(git2::Status::WT_DELETED),
            };
            status_map.insert(path, file_status);
        }

        // Update the signal
        let mut file_statuses_signal = self.file_statuses.clone();
        file_statuses_signal.set(status_map);

        Ok(())
    }

    /// Discover repositories optimized for large workspaces
    pub async fn discover_repositories_optimized(
        &self,
        workspace_path: &PathBuf,
    ) -> Result<Vec<RepositoryInfo>> {
        self.performance_manager
            .discover_repositories_optimized(workspace_path)
            .await
    }

    /// Get branches with pagination for large repositories
    pub async fn get_branches_paginated(
        &self,
        repo_path: &PathBuf,
        page: usize,
    ) -> Result<PaginatedResult<BranchInfo>> {
        self.performance_manager
            .get_branches_paginated(repo_path, page)
            .await
    }

    /// Get performance statistics for monitoring
    pub fn get_performance_stats(&self) -> PerformanceStats {
        self.performance_manager.get_stats()
    }

    /// Clear all performance caches
    pub fn clear_performance_caches(&self) {
        self.performance_manager.clear_caches();
    }

    /// Preload repositories for better performance
    pub fn preload_repositories(&self, repo_paths: Vec<PathBuf>) {
        self.performance_manager.preload_repositories(repo_paths);
    }
}

impl PartialEq for GitState {
    fn eq(&self, other: &Self) -> bool {
        // Compare only the signals, not the performance manager
        self.repositories == other.repositories
            && self.active_repo == other.active_repo
            && self.branch_info == other.branch_info
            && self.file_statuses == other.file_statuses
            && self.sync_status == other.sync_status
    }
}

impl Default for GitState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize git state for the application
pub fn use_git_state() -> GitState {
    use_context_provider(|| GitState::default())
}
