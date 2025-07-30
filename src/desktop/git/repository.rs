//! Git repository management
//! 
//! Core functionality for interacting with git repositories with performance optimizations

use git2::{Repository, RepositoryOpenFlags, StatusOptions, Oid};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tracing::{info, warn, error};
use super::{BranchInfo, BranchType, CommitInfo};
use super::performance::{OptimizedGitManager, PerformanceConfig, PerformanceStats};
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use serde::{Deserialize, Serialize};

/// Information about a git repository
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub path: PathBuf,
    pub name: String,
    pub is_bare: bool,
    pub has_changes: bool,
}

/// Main git repository manager with performance optimizations
pub struct GitRepository {
    repo: Repository,
    path: PathBuf,
}

/// Get or initialize a new optimized git manager
/// Note: Each manager instance is independent to avoid thread safety issues
pub fn get_optimized_git_manager() -> Arc<OptimizedGitManager> {
    let config = PerformanceConfig::default();
    Arc::new(OptimizedGitManager::new(config))
}

/// Get performance statistics from the global git manager
pub fn get_git_performance_stats() -> PerformanceStats {
    get_optimized_git_manager().get_stats()
}

/// Log performance statistics for monitoring
pub fn log_performance_stats() {
    let stats = get_git_performance_stats();
    info!(
        "Git Performance Stats - Cache hits: {}, Cache misses: {}, Hit rate: {:.2}%, \
        Background tasks: {}, Timeouts: {}, Avg time: {:.2}ms, Total ops: {}",
        stats.cache_hits,
        stats.cache_misses,
        stats.cache_hit_rate() * 100.0,
        stats.background_tasks_completed,
        stats.operations_timed_out,
        stats.average_operation_time_ms,
        stats.total_operations
    );
}

/// Clear all performance caches (useful for testing or memory management)
pub fn clear_performance_caches() {
    get_optimized_git_manager().clear_caches();
    info!("Cleared all git performance caches");
}

impl GitRepository {
    /// Open a git repository at the given path
    pub fn open(path: &Path) -> Result<Self> {
        // Try to discover repository from path
        let repo = Repository::discover(path)
            .context("Failed to discover git repository")?;
        
        let repo_path = repo.workdir()
            .unwrap_or_else(|| repo.path())
            .to_path_buf();
        
        info!("Opened git repository at: {:?}", repo_path);
        
        Ok(Self {
            repo,
            path: repo_path,
        })
    }
    
    /// Discover all git repositories in a workspace (optimized)
    pub fn discover_repositories(workspace_path: &Path) -> Vec<RepositoryInfo> {
        // Use the synchronous fallback for compatibility
        Self::discover_repositories_sync(workspace_path)
    }
    
    /// Discover repositories synchronously (fallback for compatibility)
    fn discover_repositories_sync(workspace_path: &Path) -> Vec<RepositoryInfo> {
        let mut repositories = Vec::new();
        
        // First check if the workspace itself is a repository
        if let Ok(repo) = Repository::discover(workspace_path) {
            if let Some(workdir) = repo.workdir() {
                let name = workdir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                repositories.push(RepositoryInfo {
                    path: workdir.to_path_buf(),
                    name,
                    is_bare: repo.is_bare(),
                    has_changes: Self::has_changes(&repo),
                });
            }
        }
        
        repositories
    }
    
    /// Discover repositories asynchronously with optimizations
    pub async fn discover_repositories_optimized(workspace_path: &Path) -> Result<Vec<RepositoryInfo>> {
        let manager = get_optimized_git_manager();
        manager.discover_repositories_optimized(workspace_path).await
    }
    
    /// Check if repository has any changes
    fn has_changes(repo: &Repository) -> bool {
        if let Ok(statuses) = repo.statuses(None) {
            !statuses.is_empty()
        } else {
            false
        }
    }
    
    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()
            .context("Failed to get HEAD reference")?;
        
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            // Detached HEAD state
            if let Some(oid) = head.target() {
                Ok(format!("(detached at {})", &oid.to_string()[..7]))
            } else {
                Ok("(unknown)".to_string())
            }
        }
    }
    
    /// Get the upstream branch for the current branch
    pub fn upstream_branch(&self) -> Result<Option<String>> {
        let head = self.repo.head()?;
        
        if let Ok(branch) = self.repo.find_branch(head.shorthand().unwrap_or(""), git2::BranchType::Local) {
            if let Ok(upstream) = branch.upstream() {
                Ok(upstream.name()?.map(|s| s.to_string()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// Count commits ahead/behind upstream
    pub fn ahead_behind(&self) -> Result<(usize, usize)> {
        let head = self.repo.head()?;
        let local_oid = head.target()
            .context("Failed to get HEAD target")?;
        
        // Try to get upstream
        if let Ok(branch) = self.repo.find_branch(head.shorthand().unwrap_or(""), git2::BranchType::Local) {
            if let Ok(upstream) = branch.upstream() {
                if let Some(upstream_oid) = upstream.get().target() {
                    let (ahead, behind) = self.repo.graph_ahead_behind(local_oid, upstream_oid)?;
                    return Ok((ahead, behind));
                }
            }
        }
        
        Ok((0, 0))
    }
    
    /// Get repository remotes
    pub fn remotes(&self) -> Result<Vec<String>> {
        let remotes = self.repo.remotes()?;
        let mut remote_names = Vec::new();
        
        for i in 0..remotes.len() {
            if let Some(name) = remotes.get(i) {
                remote_names.push(name.to_string());
            }
        }
        
        Ok(remote_names)
    }
    
    /// Get the repository path
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Check if this is a bare repository
    pub fn is_bare(&self) -> bool {
        self.repo.is_bare()
    }
    
    /// Get file statuses with optimizations
    pub fn file_statuses(&self) -> Result<Vec<(PathBuf, git2::Status)>> {
        // Use the synchronous implementation for compatibility
        self.file_statuses_sync()
    }
    
    /// Get file statuses synchronously (fallback for compatibility)
    fn file_statuses_sync(&self) -> Result<Vec<(PathBuf, git2::Status)>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        
        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut result = Vec::new();
        
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let path = self.path.join(path);
                let status = entry.status();
                result.push((path, status));
            }
        }
        
        Ok(result)
    }
    
    /// Get file statuses with caching and batching
    pub async fn file_statuses_optimized(&self) -> Result<Vec<(PathBuf, git2::Status)>> {
        let manager = get_optimized_git_manager();
        manager.get_file_statuses_batched(&self.path).await
    }
    
    /// Get the content of a file at HEAD
    pub fn get_file_at_head(&self, relative_path: &Path) -> Result<String> {
        let head = self.repo.head()?;
        let tree = head.peel_to_tree()?;
        let entry = tree.get_path(relative_path)?;
        let blob = self.repo.find_blob(entry.id())?;
        
        let content = std::str::from_utf8(blob.content())
            .context("File content is not valid UTF-8")?
            .to_string();
        
        Ok(content)
    }
    
    /// Checkout a branch
    pub fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        let branch_ref = format!("refs/heads/{}", branch_name);
        
        // Try to find the branch
        let branch = if let Ok(reference) = self.repo.find_reference(&branch_ref) {
            self.repo.reference_to_annotated_commit(&reference)?
        } else {
            // Try remote branches
            let remote_ref = format!("refs/remotes/origin/{}", branch_name);
            if let Ok(reference) = self.repo.find_reference(&remote_ref) {
                // Create local branch from remote
                let commit = self.repo.reference_to_annotated_commit(&reference)?;
                let commit_obj = self.repo.find_commit(commit.id())?;
                self.repo.branch(branch_name, &commit_obj, false)?;
                
                // Set upstream
                let mut branch = self.repo.find_branch(branch_name, git2::BranchType::Local)?;
                branch.set_upstream(Some(&format!("origin/{}", branch_name)))?;
                
                self.repo.reference_to_annotated_commit(&self.repo.find_reference(&branch_ref)?)?
            } else {
                return Err(anyhow::anyhow!("Branch '{}' not found", branch_name));
            }
        };
        
        // Checkout the branch
        let object = self.repo.find_object(branch.id(), None)?;
        self.repo.checkout_tree(&object, None)?;
        self.repo.set_head(&branch_ref)?;
        
        info!("Checked out branch: {}", branch_name);
        Ok(())
    }
    
    /// List all branches (local and remote) with optimizations
    pub fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        // Use the synchronous implementation for compatibility
        self.list_branches_sync()
    }
    
    /// List branches synchronously (fallback for compatibility)
    fn list_branches_sync(&self) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();
        
        // Get current branch name for comparison
        let current_branch = self.current_branch().unwrap_or_default();
        
        // List local branches
        for branch in self.repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                let is_current = name == current_branch;
                
                // Get ahead/behind counts if this branch has an upstream
                let (ahead, behind) = if let Ok(upstream) = branch.upstream() {
                    if let Some(upstream_oid) = upstream.get().target() {
                        if let Some(local_oid) = branch.get().target() {
                            if let Ok((ahead, behind)) = self.repo.graph_ahead_behind(local_oid, upstream_oid) {
                                (ahead, behind)
                            } else {
                                (0, 0)
                            }
                        } else {
                            (0, 0)
                        }
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                };
                
                branches.push(BranchInfo {
                    name: name.to_string(),
                    branch_type: BranchType::Local,
                    is_current,
                    upstream: branch.upstream().ok().and_then(|u| u.name().ok().flatten().map(|s| s.to_string())),
                    ahead,
                    behind,
                    last_commit: None,
                });
            }
        }
        
        // List remote branches
        for branch in self.repo.branches(Some(git2::BranchType::Remote))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                // Skip HEAD references
                if name.ends_with("/HEAD") {
                    continue;
                }
                
                branches.push(BranchInfo {
                    name: name.to_string(),
                    branch_type: BranchType::Remote,
                    is_current: false,
                    upstream: None,
                    ahead: 0,
                    behind: 0,
                    last_commit: None,
                });
            }
        }
        
        Ok(branches)
    }
    
    /// List branches with optimizations and pagination
    pub async fn list_branches_optimized(&self, page: usize) -> Result<super::performance::PaginatedResult<BranchInfo>> {
        let manager = get_optimized_git_manager();
        manager.get_branches_paginated(&self.path, page).await
    }
    
    /// Create a new branch from a reference
    pub fn create_branch(&self, branch_name: &str, from_ref: &str) -> Result<()> {
        // Validate branch name
        super::validate_branch_name(branch_name)?;
        
        // Find the commit to branch from
        let commit = if from_ref == "HEAD" {
            let head = self.repo.head()?;
            self.repo.find_commit(head.target().context("HEAD has no target")?)?
        } else if let Ok(reference) = self.repo.find_reference(&format!("refs/heads/{}", from_ref)) {
            self.repo.find_commit(reference.target().context("Reference has no target")?)?
        } else if let Ok(reference) = self.repo.find_reference(&format!("refs/remotes/{}", from_ref)) {
            self.repo.find_commit(reference.target().context("Reference has no target")?)?
        } else if let Ok(oid) = git2::Oid::from_str(from_ref) {
            self.repo.find_commit(oid)?
        } else {
            return Err(anyhow::anyhow!("Could not find reference '{}'", from_ref));
        };
        
        // Create the branch
        self.repo.branch(branch_name, &commit, false)?;
        info!("Created branch '{}' from '{}'", branch_name, from_ref);
        
        Ok(())
    }
    
    /// Delete a branch
    pub fn delete_branch(&self, branch_name: &str) -> Result<()> {
        // Find the branch
        let mut branch = self.repo.find_branch(branch_name, git2::BranchType::Local)
            .context("Branch not found")?;
        
        // Check if it's the current branch
        if branch.is_head() {
            return Err(anyhow::anyhow!("Cannot delete the current branch"));
        }
        
        // Delete the branch
        branch.delete()?;
        info!("Deleted branch '{}'", branch_name);
        
        Ok(())
    }
    
    /// Fetch all remotes
    pub fn fetch_all(&self) -> Result<()> {
        let remotes = self.repo.remotes()?;
        
        for i in 0..remotes.len() {
            if let Some(remote_name) = remotes.get(i) {
                info!("Fetching remote '{}'", remote_name);
                let mut remote = self.repo.find_remote(remote_name)?;
                
                // Fetch with default options
                let mut fetch_options = git2::FetchOptions::new();
                remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;
            }
        }
        
        info!("Fetched all remotes");
        Ok(())
    }
    
    /// Get commit info for a branch
    pub fn get_branch_commit(&self, branch_name: &str) -> Result<super::CommitInfo> {
        // Find the branch reference
        let reference = if let Ok(reference) = self.repo.find_reference(&format!("refs/heads/{}", branch_name)) {
            reference
        } else if let Ok(reference) = self.repo.find_reference(&format!("refs/remotes/{}", branch_name)) {
            reference
        } else {
            return Err(anyhow::anyhow!("Branch '{}' not found", branch_name));
        };
        
        // Get the commit
        let oid = reference.target().context("Reference has no target")?;
        let commit = self.repo.find_commit(oid)?;
        
        // Extract commit info
        let hash = commit.id().to_string();
        let hash_short = hash[..7].to_string();
        let message = commit.summary().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        
        // Convert git time to chrono DateTime
        let time = commit.time();
        let timestamp = time.seconds() + (time.offset_minutes() as i64 * 60);
        let date = chrono::DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(chrono::Utc::now);
        
        Ok(super::CommitInfo {
            hash,
            hash_short,
            message,
            author,
            date,
        })
    }
}