//! Repository Selector for Multi-Repository Workspace Support
//!
//! Provides functionality to switch between multiple repositories
//! in the workspace and manage repository status.

use anyhow::Result;
use dioxus::prelude::*;
use std::path::PathBuf;
use tracing::{error, info, warn};

use super::{get_optimized_git_manager, BranchInfo, GitRepository};

/// Repository information for the selector
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryInfo {
    pub path: PathBuf,
    pub name: String,
    pub current_branch: Option<String>,
    pub status: RepositoryStatus,
    pub upstream_status: Option<UpstreamStatus>,
}

/// Repository status
#[derive(Debug, Clone, PartialEq)]
pub enum RepositoryStatus {
    Clean,
    HasChanges,
    HasConflicts,
    NotRepository,
    Error(String),
}

/// Upstream synchronization status
#[derive(Debug, Clone, PartialEq)]
pub struct UpstreamStatus {
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
}

impl RepositoryInfo {
    /// Create repository info from a path
    pub async fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        match GitRepository::open(&path) {
            Ok(repo) => {
                let current_branch = repo.current_branch().ok();
                let status = Self::detect_status(&repo).await;
                let upstream_status = Self::detect_upstream_status(&repo).await;

                Self {
                    path,
                    name,
                    current_branch,
                    status,
                    upstream_status,
                }
            }
            Err(_) => Self {
                path,
                name,
                current_branch: None,
                status: RepositoryStatus::NotRepository,
                upstream_status: None,
            },
        }
    }

    /// Detect repository status with optimizations
    async fn detect_status(repo: &GitRepository) -> RepositoryStatus {
        let repo_path = repo.path();

        // Try to use optimized manager first
        let manager = get_optimized_git_manager();
        match manager.get_file_statuses_batched(repo_path).await {
            Ok(statuses) => {
                // Check for conflicts first
                if statuses
                    .iter()
                    .any(|(_path, status)| status.is_conflicted())
                {
                    RepositoryStatus::HasConflicts
                } else if !statuses.is_empty() {
                    RepositoryStatus::HasChanges
                } else {
                    RepositoryStatus::Clean
                }
            }
            Err(_) => {
                // Fallback to synchronous version
                match repo.file_statuses() {
                    Ok(statuses) => {
                        // Check for conflicts first
                        if statuses
                            .iter()
                            .any(|(_path, status)| status.is_conflicted())
                        {
                            RepositoryStatus::HasConflicts
                        } else if !statuses.is_empty() {
                            RepositoryStatus::HasChanges
                        } else {
                            RepositoryStatus::Clean
                        }
                    }
                    Err(e) => RepositoryStatus::Error(e.to_string()),
                }
            }
        }
    }

    /// Detect upstream status
    async fn detect_upstream_status(repo: &GitRepository) -> Option<UpstreamStatus> {
        match repo.ahead_behind() {
            Ok((ahead, behind)) => {
                let has_upstream = repo.upstream_branch().unwrap_or(None).is_some();
                Some(UpstreamStatus {
                    ahead,
                    behind,
                    has_upstream,
                })
            }
            Err(_) => None,
        }
    }

    /// Get status icon for display
    pub fn status_icon(&self) -> &'static str {
        match self.status {
            RepositoryStatus::Clean => "✓",
            RepositoryStatus::HasChanges => "●",
            RepositoryStatus::HasConflicts => "⚠",
            RepositoryStatus::NotRepository => "⭘",
            RepositoryStatus::Error(_) => "⚠",
        }
    }

    /// Get status color
    pub fn status_color(&self) -> &'static str {
        match self.status {
            RepositoryStatus::Clean => "#28a745",
            RepositoryStatus::HasChanges => "#ffc107",
            RepositoryStatus::HasConflicts => "#dc3545",
            RepositoryStatus::NotRepository => "#6c757d",
            RepositoryStatus::Error(_) => "#dc3545",
        }
    }
}

/// Repository selector state
#[derive(Debug, Clone)]
pub struct RepositorySelectorState {
    pub repositories: Vec<RepositoryInfo>,
    pub current_repo_index: Option<usize>,
    pub is_expanded: bool,
}

impl Default for RepositorySelectorState {
    fn default() -> Self {
        Self {
            repositories: Vec::new(),
            current_repo_index: None,
            is_expanded: false,
        }
    }
}

impl RepositorySelectorState {
    /// Add a repository to the selector
    pub fn add_repository(&mut self, repo_info: RepositoryInfo) {
        if !self.repositories.iter().any(|r| r.path == repo_info.path) {
            self.repositories.push(repo_info);
            if self.current_repo_index.is_none() {
                self.current_repo_index = Some(0);
            }
        }
    }

    /// Remove a repository from the selector
    pub fn remove_repository(&mut self, path: &PathBuf) {
        if let Some(pos) = self.repositories.iter().position(|r| &r.path == path) {
            self.repositories.remove(pos);

            // Adjust current index
            if let Some(current) = self.current_repo_index {
                if current == pos {
                    self.current_repo_index = if self.repositories.is_empty() {
                        None
                    } else if pos >= self.repositories.len() {
                        Some(self.repositories.len() - 1)
                    } else {
                        Some(pos)
                    };
                } else if current > pos {
                    self.current_repo_index = Some(current - 1);
                }
            }
        }
    }

    /// Get current repository
    pub fn current_repository(&self) -> Option<&RepositoryInfo> {
        self.current_repo_index
            .and_then(|idx| self.repositories.get(idx))
    }

    /// Set current repository by path
    pub fn set_current_repository(&mut self, path: &PathBuf) {
        if let Some(pos) = self.repositories.iter().position(|r| &r.path == path) {
            self.current_repo_index = Some(pos);
        }
    }

    /// Update repository information
    pub async fn refresh_repository(&mut self, path: &PathBuf) {
        if let Some(pos) = self.repositories.iter().position(|r| &r.path == path) {
            let updated_info = RepositoryInfo::from_path(path.clone()).await;
            self.repositories[pos] = updated_info;
        }
    }

    /// Scan workspace for repositories
    pub async fn scan_workspace(&mut self, workspace_path: &PathBuf) -> Result<()> {
        use tokio::fs;

        let mut entries = fs::read_dir(workspace_path).await?;
        let mut found_repos = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                // Check if it's a git repository
                let git_dir = path.join(".git");
                if git_dir.exists() {
                    let repo_info = RepositoryInfo::from_path(path).await;
                    found_repos.push(repo_info);
                }
            }
        }

        // Update repositories list
        self.repositories = found_repos;
        self.current_repo_index = if self.repositories.is_empty() {
            None
        } else {
            Some(0)
        };

        info!(
            "Found {} repositories in workspace",
            self.repositories.len()
        );
        Ok(())
    }
}

/// Repository Selector Component
#[component]
pub fn RepositorySelector(
    state: Signal<RepositorySelectorState>,
    on_repository_change: EventHandler<PathBuf>,
) -> Element {
    let selector_state = state.read();

    if selector_state.repositories.is_empty() {
        return rsx! {
            div {
                class: "repository-selector empty",
                title: "No repositories found",
                span {
                    class: "repo-icon",
                    "📁"
                }
                span { "No Repo" }
            }
        };
    }

    let current_repo = selector_state.current_repository();

    rsx! {
        div {
            class: "repository-selector",

            // Current repository display
            div {
                class: "current-repo",
                onclick: move |_| {
                    let is_expanded = state.read().is_expanded;
                    state.write().is_expanded = !is_expanded;
                },

                if let Some(repo) = current_repo {
                    span {
                        class: "repo-status",
                        style: "color: {repo.status_color()}",
                        "{repo.status_icon()}"
                    }
                    span {
                        class: "repo-name",
                        title: "{repo.path.display()}",
                        "{repo.name}"
                    }
                    if let Some(branch) = &repo.current_branch {
                        span {
                            class: "repo-branch",
                            "({branch})"
                        }
                    }
                    if let Some(upstream) = &repo.upstream_status {
                        if upstream.has_upstream {
                            span {
                                class: "repo-sync",
                                "↓{upstream.behind} ↑{upstream.ahead}"
                            }
                        }
                    }
                }

                span {
                    class: "dropdown-arrow",
                    if selector_state.is_expanded { "▼" } else { "▶" }
                }
            }

            // Repository dropdown
            if selector_state.is_expanded {
                div {
                    class: "repo-dropdown",

                    for (idx, repo) in selector_state.repositories.iter().enumerate() {
                        div {
                            class: if Some(idx) == selector_state.current_repo_index {
                                "repo-item active"
                            } else {
                                "repo-item"
                            },
                            onclick: {
                                let repo_path = repo.path.clone();
                                move |_| {
                                    state.write().set_current_repository(&repo_path);
                                    state.write().is_expanded = false;
                                    on_repository_change.call(repo_path.clone());
                                }
                            },

                            span {
                                class: "repo-status",
                                style: "color: {repo.status_color()}",
                                "{repo.status_icon()}"
                            }
                            span {
                                class: "repo-name",
                                "{repo.name}"
                            }
                            if let Some(branch) = &repo.current_branch {
                                span {
                                    class: "repo-branch",
                                    "({branch})"
                                }
                            }
                            if let Some(upstream) = &repo.upstream_status {
                                if upstream.has_upstream && (upstream.ahead > 0 || upstream.behind > 0) {
                                    span {
                                        class: "repo-sync",
                                        "↓{upstream.behind} ↑{upstream.ahead}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// CSS styles for the repository selector
pub const REPOSITORY_SELECTOR_STYLES: &str = r#"
.repository-selector {
    position: relative;
    display: inline-block;
    min-width: 150px;
    font-size: 12px;
}

.repository-selector.empty {
    color: #6c757d;
    font-style: italic;
}

.current-repo {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    cursor: pointer;
    border-radius: 3px;
    transition: background-color 0.1s ease;
    user-select: none;
}

.current-repo:hover {
    background-color: rgba(255, 255, 255, 0.1);
}

.repo-status {
    font-weight: bold;
    font-size: 14px;
}

.repo-name {
    font-weight: 500;
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.repo-branch {
    color: #ccc;
    font-size: 11px;
}

.repo-sync {
    color: #ffc107;
    font-size: 11px;
    font-weight: 500;
}

.dropdown-arrow {
    font-size: 10px;
    margin-left: auto;
}

.repo-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--vscode-dropdown-background, #3c3c3c);
    border: 1px solid var(--vscode-dropdown-border, #454545);
    border-radius: 3px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    z-index: 1000;
    max-height: 300px;
    overflow-y: auto;
}

.repo-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    cursor: pointer;
    transition: background-color 0.1s ease;
}

.repo-item:hover {
    background-color: var(--vscode-list-hoverBackground, #2a2d2e);
}

.repo-item.active {
    background-color: var(--vscode-list-activeSelectionBackground, #0e639c);
}

.repo-item.active .repo-name {
    color: var(--vscode-list-activeSelectionForeground, #fff);
}

.repo-icon {
    font-size: 14px;
    margin-right: 4px;
}
"#;
