//! Git operations implementation
//! 
//! Provides functionality for staging, committing, pushing, pulling, and other git operations

use anyhow::{Result, Context};
use git2::{Repository, Signature, ObjectType, Oid};
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};

use super::GitRepository;

/// Git operations service for performing git actions
pub struct GitOperations {
    repo: Repository,
    repo_path: PathBuf,
}

impl GitOperations {
    /// Create a new GitOperations instance
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::discover(repo_path)
            .context("Failed to discover git repository")?;
        
        let repo_path = repo.workdir()
            .unwrap_or_else(|| repo.path())
            .to_path_buf();
        
        Ok(Self {
            repo,
            repo_path,
        })
    }
    
    /// Stage a file for commit
    pub fn stage_file(&self, file_path: &Path) -> Result<()> {
        let relative_path = file_path.strip_prefix(&self.repo_path)
            .context("File is not in repository")?;
        
        let mut index = self.repo.index()?;
        index.add_path(relative_path)?;
        index.write()?;
        
        info!("Staged file: {:?}", relative_path);
        Ok(())
    }
    
    /// Unstage a file
    pub fn unstage_file(&self, file_path: &Path) -> Result<()> {
        let relative_path = file_path.strip_prefix(&self.repo_path)
            .context("File is not in repository")?;
        
        // Get HEAD commit
        let head = self.repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        
        // Reset the file to HEAD
        let mut index = self.repo.index()?;
        let entry = head_tree.get_path(relative_path)?;
        index.add(&git2::IndexEntry {
            ctime: git2::IndexTime::new(0, 0),
            mtime: git2::IndexTime::new(0, 0),
            dev: 0,
            ino: 0,
            mode: entry.filemode() as u32,
            uid: 0,
            gid: 0,
            file_size: 0,
            id: entry.id(),
            flags: 0,
            flags_extended: 0,
            path: relative_path.to_string_lossy().as_bytes().to_vec(),
        })?;
        index.write()?;
        
        info!("Unstaged file: {:?}", relative_path);
        Ok(())
    }
    
    /// Stage all changes
    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(&["*"], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        
        info!("Staged all changes");
        Ok(())
    }
    
    /// Unstage all changes
    pub fn unstage_all(&self) -> Result<()> {
        let head = self.repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        
        let mut index = self.repo.index()?;
        index.read_tree(&head_tree)?;
        index.write()?;
        
        info!("Unstaged all changes");
        Ok(())
    }
    
    /// Commit staged changes
    pub fn commit(&self, message: &str) -> Result<Oid> {
        let signature = self.get_signature()?;
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let parent_commit = if let Ok(head) = self.repo.head() {
            Some(head.peel_to_commit()?)
        } else {
            None
        };
        
        let parents: Vec<&git2::Commit> = if let Some(ref commit) = parent_commit {
            vec![commit]
        } else {
            vec![]
        };
        
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        
        info!("Created commit: {} - {}", commit_id, message);
        Ok(commit_id)
    }
    
    /// Push to remote
    pub fn push(&self, remote_name: &str, branch_name: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name)?;
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(callbacks);
        
        remote.push(&[&refspec], Some(&mut push_opts))?;
        
        info!("Pushed {} to {}", branch_name, remote_name);
        Ok(())
    }
    
    /// Pull from remote
    pub fn pull(&self, remote_name: &str, branch_name: &str) -> Result<()> {
        // Fetch first
        let mut remote = self.repo.find_remote(remote_name)?;
        let refspec = format!("refs/heads/{}:refs/remotes/{}/{}", branch_name, remote_name, branch_name);
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        
        remote.fetch(&[&refspec], Some(&mut fetch_opts), None)?;
        
        // Now merge
        let fetch_head = self.repo.fetchhead_foreach(|ref_name, remote_url, oid, is_merge| {
            if is_merge {
                info!("Merging {} from {:?}", oid, std::str::from_utf8(remote_url).unwrap_or("<invalid UTF-8>"));
                true
            } else {
                false
            }
        })?;
        
        info!("Pulled {} from {}", branch_name, remote_name);
        Ok(())
    }
    
    /// Fetch from remote
    pub fn fetch(&self, remote_name: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name)?;
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        
        remote.fetch(&[] as &[&str], Some(&mut fetch_opts), None)?;
        
        info!("Fetched from {}", remote_name);
        Ok(())
    }
    
    /// Discard changes in a file
    pub fn discard_file_changes(&self, file_path: &Path) -> Result<()> {
        let relative_path = file_path.strip_prefix(&self.repo_path)
            .context("File is not in repository")?;
        
        // Get the file content from HEAD
        let head = self.repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        
        if let Ok(entry) = head_tree.get_path(relative_path) {
            let blob = self.repo.find_blob(entry.id())?;
            std::fs::write(file_path, blob.content())?;
            info!("Discarded changes in: {:?}", relative_path);
        } else {
            // File is new, just delete it
            if file_path.exists() {
                std::fs::remove_file(file_path)?;
                info!("Deleted new file: {:?}", relative_path);
            }
        }
        
        Ok(())
    }
    
    /// Get git signature for commits
    fn get_signature(&self) -> Result<Signature> {
        // Try to get signature from config
        if let Ok(config) = self.repo.config() {
            if let (Ok(name), Ok(email)) = (config.get_string("user.name"), config.get_string("user.email")) {
                return Ok(Signature::now(&name, &email)?);
            }
        }
        
        // Fallback to default
        Ok(Signature::now("Hive User", "user@hive.local")?)
    }
    
    /// Get the default remote name (usually "origin")
    pub fn get_default_remote(&self) -> Result<String> {
        let remotes = self.repo.remotes()?;
        
        // Look for "origin" first
        for i in 0..remotes.len() {
            if let Some(name) = remotes.get(i) {
                if name == "origin" {
                    return Ok(name.to_string());
                }
            }
        }
        
        // Fall back to first remote
        if let Some(name) = remotes.get(0) {
            Ok(name.to_string())
        } else {
            Err(anyhow::anyhow!("No remotes configured"))
        }
    }
    
    /// Get current branch name
    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }
}

/// Git operation types for UI
#[derive(Debug, Clone, PartialEq)]
pub enum GitOperation {
    Stage(PathBuf),
    Unstage(PathBuf),
    StageAll,
    UnstageAll,
    Commit(String),
    Push,
    Pull,
    Fetch,
    Sync, // Pull then Push (VS Code style)
    DiscardChanges(PathBuf),
}

/// Result of a git operation
#[derive(Debug, Clone)]
pub struct GitOperationResult {
    pub operation: GitOperation,
    pub success: bool,
    pub message: String,
}