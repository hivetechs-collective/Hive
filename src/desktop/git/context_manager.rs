//! Git context manager for shared repository state
//! 
//! Manages git repositories across terminal and GUI operations

use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context};
use tracing::{info, warn, debug};

use super::{GitRepository, GitOperations, RepositoryInfo, BranchInfo, BranchType, FileStatus, StatusType, get_optimized_git_manager};

/// Git context manager that shares state between terminal and GUI
#[derive(Clone)]
pub struct GitContextManager {
    /// All discovered repositories (path -> repository)
    repositories: Arc<Mutex<HashMap<PathBuf, Arc<GitRepository>>>>,
    /// Currently active path (terminal CWD or GUI folder)
    active_path: Signal<Option<PathBuf>>,
    /// Currently active repository
    active_repository: Signal<Option<Arc<GitRepository>>>,
    /// Active repository info
    active_repo_info: Signal<Option<RepositoryInfo>>,
    /// Active branch info
    active_branch_info: Signal<Option<BranchInfo>>,
    /// File statuses for active repository
    file_statuses: Signal<HashMap<PathBuf, FileStatus>>,
}

impl GitContextManager {
    /// Create a new git context manager
    pub fn new() -> Self {
        Self {
            repositories: Arc::new(Mutex::new(HashMap::new())),
            active_path: Signal::new(None),
            active_repository: Signal::new(None),
            active_repo_info: Signal::new(None),
            active_branch_info: Signal::new(None),
            file_statuses: Signal::new(HashMap::new()),
        }
    }
    
    /// Set the active path (from terminal CWD or GUI folder)
    pub async fn set_active_path(&mut self, path: PathBuf) {
        info!("Setting active path to: {:?}", path);
        self.active_path.set(Some(path.clone()));
        
        // Discover and activate repository for this path
        if let Some(repo) = self.discover_repository(&path).await {
            self.activate_repository(repo).await;
        } else {
            // No repository at this path, clear active repository
            self.clear_active_repository();
        }
    }
    
    /// Discover a git repository at the given path
    pub async fn discover_repository(&self, path: &Path) -> Option<Arc<GitRepository>> {
        // Check if we already have this repository
        let repositories = self.repositories.lock().await;
        
        // Try exact path first
        if let Some(repo) = repositories.get(path) {
            debug!("Found cached repository at {:?}", path);
            return Some(repo.clone());
        }
        
        // Try parent directories
        let mut current_path = path;
        loop {
            if let Some(repo) = repositories.get(current_path) {
                debug!("Found cached repository at parent {:?}", current_path);
                return Some(repo.clone());
            }
            
            // Check if this directory has a .git folder
            let git_dir = current_path.join(".git");
            if git_dir.exists() && git_dir.is_dir() {
                drop(repositories); // Release lock before creating new repo
                
                // Create new repository
                match GitRepository::open(current_path) {
                    Ok(repo) => {
                        let repo_arc = Arc::new(repo);
                        let mut repositories = self.repositories.lock().await;
                        repositories.insert(current_path.to_path_buf(), repo_arc.clone());
                        info!("Discovered new repository at {:?}", current_path);
                        return Some(repo_arc);
                    }
                    Err(e) => {
                        warn!("Failed to open repository at {:?}: {}", current_path, e);
                        return None;
                    }
                }
            }
            
            // Move to parent directory
            match current_path.parent() {
                Some(parent) => current_path = parent,
                None => break,
            }
        }
        
        debug!("No repository found for path {:?}", path);
        None
    }
    
    /// Activate a repository and update all related state with optimizations
    async fn activate_repository(&mut self, repo: Arc<GitRepository>) {
        info!("Activating repository at {:?}", repo.path());
        
        // Update active repository
        self.active_repository.set(Some(repo.clone()));
        
        // Get repository info
        let info = RepositoryInfo {
            path: repo.path().to_path_buf(),
            name: repo.path().file_name().unwrap_or_default().to_string_lossy().to_string(),
            is_bare: false, // TODO: check if repo is bare
            has_changes: false, // Will be updated after checking status
        };
        self.active_repo_info.set(Some(info.clone()));
        
        // Get branch info
        if let Ok(branch_name) = repo.current_branch() {
            let branch_info = BranchInfo {
                name: branch_name,
                branch_type: BranchType::Local,
                is_current: true,
                upstream: None,
                ahead: 0,
                behind: 0,
                last_commit: None,
            };
            self.active_branch_info.set(Some(branch_info));
        }
        
        // Get file statuses with optimization
        let repo_path = repo.path();
        let manager = get_optimized_git_manager();
        
        match manager.get_file_statuses_batched(repo_path).await {
            Ok(statuses) => {
                self.process_file_statuses(statuses);
            }
            Err(_) => {
                // Fallback to synchronous version
                match repo.file_statuses() {
                    Ok(statuses) => {
                        self.process_file_statuses(statuses);
                    }
                    Err(e) => {
                        warn!("Failed to get repository status: {}", e);
                        self.file_statuses.set(HashMap::new());
                    }
                }
            }
        }
    }
    
    /// Process file statuses and convert to our internal format
    fn process_file_statuses(&mut self, statuses: Vec<(PathBuf, git2::Status)>) {
        let mut status_map = HashMap::new();
        for (path, git_status) in statuses {
            // Convert git2::Status to our StatusType
            let status_type = if git_status.contains(git2::Status::WT_MODIFIED) || git_status.contains(git2::Status::INDEX_MODIFIED) {
                StatusType::Modified
            } else if git_status.contains(git2::Status::WT_NEW) || git_status.contains(git2::Status::INDEX_NEW) {
                StatusType::Added
            } else if git_status.contains(git2::Status::WT_DELETED) || git_status.contains(git2::Status::INDEX_DELETED) {
                StatusType::Deleted
            } else if git_status.contains(git2::Status::WT_RENAMED) || git_status.contains(git2::Status::INDEX_RENAMED) {
                StatusType::Renamed
            } else if git_status.is_wt_new() {
                StatusType::Untracked
            } else {
                continue; // Skip other statuses
            };
            
            let file_status = FileStatus {
                status_type,
                has_staged_changes: git_status.contains(git2::Status::INDEX_NEW) ||
                                  git_status.contains(git2::Status::INDEX_MODIFIED) ||
                                  git_status.contains(git2::Status::INDEX_DELETED) ||
                                  git_status.contains(git2::Status::INDEX_RENAMED),
                has_unstaged_changes: git_status.contains(git2::Status::WT_NEW) ||
                                    git_status.contains(git2::Status::WT_MODIFIED) ||
                                    git_status.contains(git2::Status::WT_DELETED),
            };
            status_map.insert(path, file_status);
        }
        self.file_statuses.set(status_map);
    }
    
    /// Clear the active repository state
    fn clear_active_repository(&mut self) {
        info!("Clearing active repository");
        self.active_repository.set(None);
        self.active_repo_info.set(None);
        self.active_branch_info.set(None);
        self.file_statuses.set(HashMap::new());
    }
    
    /// Get the currently active repository
    pub fn active_repository(&self) -> Option<Arc<GitRepository>> {
        self.active_repository.read().clone()
    }
    
    /// Get the currently active repository info
    pub fn active_repo_info(&self) -> Option<RepositoryInfo> {
        self.active_repo_info.read().clone()
    }
    
    /// Get the currently active branch info
    pub fn active_branch_info(&self) -> Option<BranchInfo> {
        self.active_branch_info.read().clone()
    }
    
    /// Get file statuses for the active repository
    pub fn file_statuses(&self) -> HashMap<PathBuf, FileStatus> {
        self.file_statuses.read().clone()
    }
    
    /// Create git operations for the active repository
    pub fn create_operations(&self) -> Option<GitOperations> {
        let repo_info = self.active_repo_info.read();
        if let Some(info) = repo_info.as_ref() {
            GitOperations::new(&info.path).ok()
        } else {
            None
        }
    }
    
    /// Refresh the active repository state
    pub async fn refresh(&mut self) {
        let repo = self.active_repository.read().clone();
        if let Some(repo) = repo {
            self.activate_repository(repo).await;
        }
    }
    
    /// Check if a path is within a git repository
    pub async fn is_git_repository(&self, path: &Path) -> bool {
        self.discover_repository(path).await.is_some()
    }
    
    /// Get repository for a specific path
    pub async fn get_repository(&self, path: &Path) -> Option<Arc<GitRepository>> {
        self.discover_repository(path).await
    }
    
    /// Clear all cached repositories
    pub async fn clear_cache(&self) {
        let mut repositories = self.repositories.lock().await;
        repositories.clear();
        info!("Cleared repository cache");
    }
}

/// Global git context manager hook
pub fn use_git_context() -> GitContextManager {
    use_context()
}

/// Provide git context manager
pub fn provide_git_context() -> GitContextManager {
    let manager = GitContextManager::new();
    use_context_provider(|| manager.clone());
    manager
}

/// Event for git context changes
#[derive(Debug, Clone)]
pub enum GitContextEvent {
    /// Active repository changed
    RepositoryChanged(Option<PathBuf>),
    /// Branch changed in active repository
    BranchChanged(Option<String>),
    /// File statuses updated
    StatusesUpdated,
}