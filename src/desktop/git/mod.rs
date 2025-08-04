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
pub mod lazygit_controller;
pub mod lazygit_updater;
pub mod status_menu;
pub mod repository_selector;
pub mod repository_dropdown;
pub mod multi_repo_manager;
pub mod performance;
pub mod statusbar_integration_fixed;
pub mod branch_menu;
pub mod commit_box;
pub mod stash;
pub mod stash_ui;
pub mod stash_manager;
pub mod stash_shortcuts;
pub mod performance_config_ui;
pub mod performance_monitor;
pub mod inline_actions;
pub mod keyboard_shortcuts;
pub mod action_processor;
pub mod decorations;
pub mod decoration_watcher;
pub mod decoration_config_ui;
pub mod enhanced_file_explorer;
pub mod decoration_styles;

pub use repository::{GitRepository, RepositoryInfo, get_optimized_git_manager, get_git_performance_stats, log_performance_stats, clear_performance_caches};
pub use branch::{BranchInfo, BranchType, BranchFilter, BranchSort, CommitInfo, sort_branches, validate_branch_name};
pub use status::{FileStatus, StatusType, SyncStatus};
pub use watcher::{GitWatcher, GitEvent};
pub use diff::{
    DiffResult, DiffHunk, DiffLine, DiffLineType, DiffViewMode, HunkStageStatus,
    DiffAction, DiffActionResult, DiffActionState, DiffUndoEntry, DiffGitOperations,
    compute_diff, compute_diff_with_context, get_file_diff, get_file_diff_with_context
};
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
pub use performance::{OptimizedGitManager, PerformanceConfig, PerformanceStats, PaginatedResult, BatchProcessor};
pub use statusbar_integration_fixed::{
    initialize_git_statusbar_integration,
    setup_git_statusbar_integration,
    setup_git_watcher_integration,
    setup_statusbar_event_subscription,
};
pub use branch_menu::{BranchMenu, BranchOperation, BranchOperationResult};
pub use commit_box::{CommitBox, CommitBoxProps, CommitTemplate, ConventionalType};
pub use stash::{GitStash, StashInfo, StashSaveOptions, StashApplyOptions, StashStats};
pub use stash_ui::{StashList, StashSaveDialog, StashPreview, StashAction};
pub use stash_manager::{StashManager, StashPanel};
pub use stash_shortcuts::{StashKeyboardHandler, StashShortcutsInfo, StashShortcutAction, handle_stash_shortcut};
pub use performance_config_ui::{PerformanceConfigUI, PerformanceConfigProps, PerformancePresets};
pub use performance_monitor::{PerformanceMonitor, PerformanceMonitorProps, AutoRefreshPerformanceMonitor, PerformanceGraph};
pub use inline_actions::{HunkInlineActions, LineInlineActions, EnhancedDiffHunk, ActionTooltip};
pub use keyboard_shortcuts::{ShortcutConfig, ShortcutManager, FocusManager, GlobalShortcutHandler, ShortcutHelpDialog, KeyCombination, ShortcutAction};
pub use action_processor::{DiffActionProcessor, ExecutionResult, ProcessorConfig};
pub use decorations::{
    GitDecorationConfig, DecorationStyles, EnhancedGitStatus, FolderDecoration, 
    GitDecorationManager, FileGitDecoration, EditorGutterDecorations, use_git_decoration_manager
};
pub use decoration_watcher::{
    GitDecorationWatcher, DecorationEvent, DecorationWatcherConfig, use_git_decoration_watcher
};
pub use decoration_config_ui::{GitDecorationConfigUI, GitDecorationPreview};
pub use enhanced_file_explorer::{
    EnhancedFileExplorer, GitDecorationIntegration, use_git_decorations
};
pub use decoration_styles::{
    GitDecorationStyles, GitDecorationDynamicStyles, VSCodeGitThemeStyles,
    get_git_decoration_styles, get_git_decoration_css_variables,
    get_inline_git_status_style, get_git_status_class, get_conflict_class,
    get_file_git_classes, get_vscode_git_theme_styles
};
pub use lazygit_controller::{LazyGitController, LazyGitPanel, is_lazygit_available};
pub use lazygit_updater::{LazyGitUpdater, initialize_lazygit_updater, force_update_lazygit};

use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;

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
        let file_statuses = self.performance_manager
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
    pub async fn discover_repositories_optimized(&self, workspace_path: &PathBuf) -> Result<Vec<RepositoryInfo>> {
        self.performance_manager
            .discover_repositories_optimized(workspace_path)
            .await
    }
    
    /// Get branches with pagination for large repositories
    pub async fn get_branches_paginated(&self, repo_path: &PathBuf, page: usize) -> Result<PaginatedResult<BranchInfo>> {
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
        self.repositories == other.repositories &&
        self.active_repo == other.active_repo &&
        self.branch_info == other.branch_info &&
        self.file_statuses == other.file_statuses &&
        self.sync_status == other.sync_status
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