//! Git repository management
//! 
//! Core functionality for interacting with git repositories

use git2::{Repository, RepositoryOpenFlags, StatusOptions};
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tracing::{info, warn, error};

/// Information about a git repository
#[derive(Debug, Clone, PartialEq)]
pub struct RepositoryInfo {
    pub path: PathBuf,
    pub name: String,
    pub is_bare: bool,
    pub has_changes: bool,
}

/// Main git repository manager
pub struct GitRepository {
    repo: Repository,
    path: PathBuf,
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
    
    /// Discover all git repositories in a workspace
    pub fn discover_repositories(workspace_path: &Path) -> Vec<RepositoryInfo> {
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
        
        // TODO: In the future, scan for nested repositories
        
        repositories
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
    
    /// Get file statuses
    pub fn file_statuses(&self) -> Result<Vec<(PathBuf, git2::Status)>> {
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
}