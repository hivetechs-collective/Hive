//! Multi-Repository Manager
//! 
//! Manages multiple git repositories in a workspace,
//! coordinates git operations across repositories

use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use super::repository_selector::{RepositoryInfo, RepositorySelectorState};
use super::{GitRepository, GitState, BranchInfo, SyncStatus};

/// Multi-repository manager for coordinating git operations
pub struct MultiRepoManager {
    /// Current workspace path
    workspace_path: PathBuf,
    /// Repository selector state
    selector_state: RepositorySelectorState,
    /// Git states for each repository
    git_states: HashMap<PathBuf, GitState>,
    /// Currently active repository
    active_repo: Option<PathBuf>,
}

impl MultiRepoManager {
    /// Create a new multi-repository manager
    pub fn new(workspace_path: PathBuf) -> Self {
        Self {
            workspace_path,
            selector_state: RepositorySelectorState::default(),
            git_states: HashMap::new(),
            active_repo: None,
        }
    }

    /// Initialize the manager by scanning for repositories
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing multi-repository manager for workspace: {:?}", self.workspace_path);
        
        // Scan workspace for repositories
        self.selector_state.scan_workspace(&self.workspace_path).await
            .context("Failed to scan workspace for repositories")?;

        // Initialize git states for found repositories
        for repo_info in &self.selector_state.repositories {
            let git_state = GitState::new();
            self.git_states.insert(repo_info.path.clone(), git_state);
        }

        // Set active repository to the first one found
        if let Some(first_repo) = self.selector_state.repositories.first() {
            self.set_active_repository(first_repo.path.clone()).await?;
        }

        info!("Initialized {} repositories", self.selector_state.repositories.len());
        Ok(())
    }

    /// Get the selector state
    pub fn selector_state(&self) -> &RepositorySelectorState {
        &self.selector_state
    }

    /// Get mutable selector state
    pub fn selector_state_mut(&mut self) -> &mut RepositorySelectorState {
        &mut self.selector_state
    }

    /// Get current active repository path
    pub fn active_repository(&self) -> Option<&PathBuf> {
        self.active_repo.as_ref()
    }

    /// Set active repository
    pub async fn set_active_repository(&mut self, repo_path: PathBuf) -> Result<()> {
        if !self.selector_state.repositories.iter().any(|r| r.path == repo_path) {
            warn!("Attempted to set non-existent repository as active: {:?}", repo_path);
            return Ok(());
        }

        self.active_repo = Some(repo_path.clone());
        self.selector_state.set_current_repository(&repo_path);

        // Refresh git state for the active repository
        if let Some(git_state) = self.git_states.get(&repo_path) {
            git_state.refresh_status(&repo_path).await;
        }

        info!("Set active repository to: {:?}", repo_path);
        Ok(())
    }

    /// Get git state for a specific repository
    pub fn get_git_state(&self, repo_path: &PathBuf) -> Option<&GitState> {
        self.git_states.get(repo_path)
    }

    /// Get git state for the active repository
    pub fn active_git_state(&self) -> Option<&GitState> {
        self.active_repo.as_ref()
            .and_then(|path| self.git_states.get(path))
    }

    /// Add a new repository to the manager
    pub async fn add_repository(&mut self, repo_path: PathBuf) -> Result<()> {
        if self.git_states.contains_key(&repo_path) {
            warn!("Repository already exists: {:?}", repo_path);
            return Ok(());
        }

        // Create repository info
        let repo_info = RepositoryInfo::from_path(repo_path.clone()).await;
        
        // Add to selector state
        self.selector_state.add_repository(repo_info);

        // Initialize git state
        let git_state = GitState::new();
        git_state.refresh_status(&repo_path).await;
        self.git_states.insert(repo_path.clone(), git_state);

        // If this is the first repository, make it active
        if self.active_repo.is_none() {
            self.set_active_repository(repo_path.clone()).await?;
        }

        info!("Added repository: {:?}", repo_path);
        Ok(())
    }

    /// Remove a repository from the manager
    pub async fn remove_repository(&mut self, repo_path: &PathBuf) -> Result<()> {
        self.selector_state.remove_repository(repo_path);
        self.git_states.remove(repo_path);

        // If this was the active repository, switch to another one
        if self.active_repo.as_ref() == Some(repo_path) {
            self.active_repo = None;
            if let Some(first_repo) = self.selector_state.repositories.first() {
                self.set_active_repository(first_repo.path.clone()).await?;
            }
        }

        info!("Removed repository: {:?}", repo_path);
        Ok(())
    }

    /// Refresh all repository information
    pub async fn refresh_all_repositories(&mut self) -> Result<()> {
        info!("Refreshing all repositories");
        
        let repo_paths: Vec<PathBuf> = self.selector_state.repositories
            .iter()
            .map(|r| r.path.clone())
            .collect();

        for repo_path in repo_paths {
            // Refresh repository info
            self.selector_state.refresh_repository(&repo_path).await;
            
            // Refresh git state
            if let Some(git_state) = self.git_states.get(&repo_path) {
                git_state.refresh_status(&repo_path).await;
            }
        }

        Ok(())
    }

    /// Get summary of all repositories' status
    pub fn get_repositories_summary(&self) -> RepositoriesSummary {
        let mut summary = RepositoriesSummary::default();
        
        for repo_info in &self.selector_state.repositories {
            summary.total_repos += 1;
            
            match &repo_info.status {
                super::repository_selector::RepositoryStatus::Clean => {
                    summary.clean_repos += 1;
                }
                super::repository_selector::RepositoryStatus::HasChanges => {
                    summary.repos_with_changes += 1;
                }
                super::repository_selector::RepositoryStatus::HasConflicts => {
                    summary.repos_with_conflicts += 1;
                }
                super::repository_selector::RepositoryStatus::NotRepository => {
                    summary.invalid_repos += 1;
                }
                super::repository_selector::RepositoryStatus::Error(_) => {
                    summary.error_repos += 1;
                }
            }

            // Count repositories that need sync
            if let Some(upstream) = &repo_info.upstream_status {
                if upstream.has_upstream && (upstream.ahead > 0 || upstream.behind > 0) {
                    summary.repos_needing_sync += 1;
                }
            }
        }

        summary
    }

    /// Perform sync operation on active repository
    pub async fn sync_active_repository(&mut self) -> Result<()> {
        if let Some(repo_path) = self.active_repo.clone() {
            self.sync_repository(&repo_path).await
        } else {
            warn!("No active repository to sync");
            Ok(())
        }
    }

    /// Perform sync operation on specific repository
    pub async fn sync_repository(&mut self, repo_path: &PathBuf) -> Result<()> {
        info!("Syncing repository: {:?}", repo_path);
        
        // Use the async operations from the git operations module
        super::operations::fetch(repo_path).await
            .context("Failed to fetch")?;
            
        super::operations::pull(repo_path).await
            .context("Failed to pull")?;
            
        super::operations::push(repo_path).await
            .context("Failed to push")?;

        // Refresh repository state after sync
        self.selector_state.refresh_repository(repo_path).await;
        if let Some(git_state) = self.git_states.get(repo_path) {
            git_state.refresh_status(repo_path).await;
        }

        info!("Successfully synced repository: {:?}", repo_path);
        Ok(())
    }

    /// Perform operation on all repositories
    pub async fn sync_all_repositories(&mut self) -> Result<Vec<(PathBuf, Result<()>)>> {
        info!("Syncing all repositories");
        
        let repo_paths: Vec<PathBuf> = self.selector_state.repositories
            .iter()
            .map(|r| r.path.clone())
            .collect();

        let mut results = Vec::new();

        for repo_path in repo_paths {
            let result = self.sync_repository(&repo_path).await;
            results.push((repo_path, result));
        }

        Ok(results)
    }
}

/// Summary of all repositories' status
#[derive(Debug, Default, Clone)]
pub struct RepositoriesSummary {
    pub total_repos: usize,
    pub clean_repos: usize,
    pub repos_with_changes: usize,
    pub repos_with_conflicts: usize,
    pub repos_needing_sync: usize,
    pub invalid_repos: usize,
    pub error_repos: usize,
}

impl RepositoriesSummary {
    /// Get status message for display
    pub fn status_message(&self) -> String {
        if self.total_repos == 0 {
            "No repositories".to_string()
        } else if self.repos_with_conflicts > 0 {
            format!("{} conflicts in {} repos", self.repos_with_conflicts, self.total_repos)
        } else if self.repos_with_changes > 0 {
            format!("{} repos with changes", self.repos_with_changes)
        } else if self.repos_needing_sync > 0 {
            format!("{} repos need sync", self.repos_needing_sync)
        } else {
            format!("{} repos clean", self.clean_repos)
        }
    }

    /// Get overall status color
    pub fn status_color(&self) -> &'static str {
        if self.repos_with_conflicts > 0 || self.error_repos > 0 {
            "#dc3545" // Red
        } else if self.repos_with_changes > 0 {
            "#ffc107" // Yellow
        } else if self.repos_needing_sync > 0 {
            "#17a2b8" // Blue
        } else {
            "#28a745" // Green
        }
    }
}