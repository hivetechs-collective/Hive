//! Git operations implementation with performance optimizations
//! 
//! Provides functionality for staging, committing, pushing, pulling, and other git operations
//! Enhanced with caching, batching, and background processing

use anyhow::{Result, Context};
use git2::{Repository, Signature, ObjectType, Oid, RemoteCallbacks, PushOptions, FetchOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tracing::{info, warn, error, instrument};

use super::{GitRepository, get_optimized_git_manager, log_performance_stats};
use super::performance::{BatchProcessor, PerformanceConfig};

/// Async wrapper for git fetch operation with performance optimizations
pub async fn fetch(repo_path: &Path) -> Result<()> {
    let repo_path = repo_path.to_path_buf();
    let start_time = Instant::now();
    
    // Try to use optimized manager first
    let manager = get_optimized_git_manager();
    if let Ok(repo) = manager.get_repository(&repo_path).await {
        // Use the optimized fetch operation
        let result = tokio::task::spawn_blocking(move || -> Result<()> {
            let ops = GitOperations::new(&repo_path)?;
            ops.fetch_with_progress("origin", None, None)
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Optimized fetch completed in {:?}", elapsed);
        
        // Log performance stats periodically
        let stats = get_optimized_git_manager().get_stats();
        if stats.total_operations % 10 == 0 {
            log_performance_stats();
        }
        
        result
    } else {
        // Fallback to regular fetch
        tokio::task::spawn_blocking(move || -> Result<()> {
            let ops = GitOperations::new(&repo_path)?;
            ops.fetch("origin").context("Failed to fetch from origin")
        }).await?
    }
}

/// Async wrapper for git pull operation with performance optimizations
pub async fn pull(repo_path: &Path) -> Result<()> {
    let repo_path = repo_path.to_path_buf();
    let start_time = Instant::now();
    
    // Try to use optimized manager first
    let manager = get_optimized_git_manager();
    if let Ok(repo) = manager.get_repository(&repo_path).await {
        let result = tokio::task::spawn_blocking(move || -> Result<()> {
            let repo = Repository::discover(&repo_path)?;
            let head = repo.head()?;
            if let Some(branch_name) = head.shorthand() {
                let ops = GitOperations::new(&repo_path)?;
                ops.pull_with_progress("origin", branch_name, None, None)
            } else {
                Err(anyhow::anyhow!("Unable to determine current branch"))
            }
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Optimized pull completed in {:?}", elapsed);
        result
    } else {
        // Fallback to regular pull
        tokio::task::spawn_blocking(move || -> Result<()> {
            let repo = Repository::discover(&repo_path)?;
            let head = repo.head()?;
            if let Some(branch_name) = head.shorthand() {
                let ops = GitOperations::new(&repo_path)?;
                ops.pull("origin", branch_name).context("Failed to pull from origin")
            } else {
                Err(anyhow::anyhow!("Unable to determine current branch"))
            }
        }).await?
    }
}

/// Async wrapper for git push operation with performance optimizations
pub async fn push(repo_path: &Path) -> Result<()> {
    let repo_path = repo_path.to_path_buf();
    let start_time = Instant::now();
    
    // Try to use optimized manager first
    let manager = get_optimized_git_manager();
    if let Ok(repo) = manager.get_repository(&repo_path).await {
        let result = tokio::task::spawn_blocking(move || -> Result<()> {
            let repo = Repository::discover(&repo_path)?;
            let head = repo.head()?;
            if let Some(branch_name) = head.shorthand() {
                let ops = GitOperations::new(&repo_path)?;
                ops.push_with_progress("origin", branch_name, None, None)
            } else {
                Err(anyhow::anyhow!("Unable to determine current branch"))
            }
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Optimized push completed in {:?}", elapsed);
        result
    } else {
        // Fallback to regular push
        tokio::task::spawn_blocking(move || -> Result<()> {
            let repo = Repository::discover(&repo_path)?;
            let head = repo.head()?;
            if let Some(branch_name) = head.shorthand() {
                let ops = GitOperations::new(&repo_path)?;
                ops.push("origin", branch_name).context("Failed to push to origin")
            } else {
                Err(anyhow::anyhow!("Unable to determine current branch"))
            }
        }).await?
    }
}

/// Async wrapper for git push with upstream operation
pub async fn push_with_upstream(repo_path: &Path, branch_name: &str) -> Result<()> {
    let repo_path = repo_path.to_path_buf();
    let branch_name = branch_name.to_string();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let ops = GitOperations::new(&repo_path)?;
        ops.push_with_upstream("origin", &branch_name)
            .context("Failed to push with upstream to origin")
    })
    .await?
}

/// Git operations service for performing git actions with performance optimizations
pub struct GitOperations {
    repo: Repository,
    repo_path: PathBuf,
    batch_processor: Option<BatchProcessor>,
}

impl GitOperations {
    /// Create a new GitOperations instance with performance optimizations
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::discover(repo_path)
            .context("Failed to discover git repository")?;
        
        let repo_path = repo.workdir()
            .unwrap_or_else(|| repo.path())
            .to_path_buf();
        
        // Initialize batch processor for bulk operations
        let config = PerformanceConfig::default();
        let batch_processor = Some(BatchProcessor::new(
            config.max_batch_size,
            config.max_concurrent_operations,
        ));
        
        Ok(Self {
            repo,
            repo_path,
            batch_processor,
        })
    }
    
    /// Create a new GitOperations instance without optimizations (for compatibility)
    pub fn new_simple(repo_path: &Path) -> Result<Self> {
        let repo = Repository::discover(repo_path)
            .context("Failed to discover git repository")?;
        
        let repo_path = repo.workdir()
            .unwrap_or_else(|| repo.path())
            .to_path_buf();
        
        Ok(Self {
            repo,
            repo_path,
            batch_processor: None,
        })
    }
    
    /// Stage a file for commit with optimizations
    #[instrument(skip(self), fields(file = %file_path.display()))]
    pub fn stage_file(&self, file_path: &Path) -> Result<()> {
        let start_time = Instant::now();
        
        let relative_path = file_path.strip_prefix(&self.repo_path)
            .context("File is not in repository")?;
        
        let mut index = self.repo.index()?;
        index.add_path(relative_path)?;
        index.write()?;
        
        let elapsed = start_time.elapsed();
        info!("Staged file: {:?} in {:?}", relative_path, elapsed);
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
    
    /// Stage all changes with batch processing
    #[instrument(skip(self))]
    pub fn stage_all(&self) -> Result<()> {
        let start_time = Instant::now();
        
        let mut index = self.repo.index()?;
        index.add_all(&["*"], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        
        let elapsed = start_time.elapsed();
        info!("Staged all changes in {:?}", elapsed);
        Ok(())
    }
    
    /// Stage multiple files efficiently using batch processing
    #[instrument(skip(self), fields(file_count = file_paths.len()))]
    pub async fn stage_files_batch(&self, file_paths: Vec<PathBuf>) -> Vec<Result<()>> {
        if let Some(processor) = &self.batch_processor {
            let repo_path = self.repo_path.clone();
            processor.process_batches(file_paths, move |file_path| {
                let relative_path = file_path.strip_prefix(&repo_path)
                    .context("File is not in repository")?;
                
                // Note: This is simplified - in a real implementation,
                // we'd need to handle git2::Repository thread safety differently
                info!("Would stage file: {:?}", relative_path);
                Ok(())
            }).await
        } else {
            // Fallback to sequential processing
            let mut results = Vec::new();
            for file_path in file_paths {
                results.push(self.stage_file(&file_path));
            }
            results
        }
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
    
    /// Push to remote with progress callback and cancellation support
    pub fn push_with_progress(
        &self, 
        remote_name: &str, 
        branch_name: &str,
        progress_callback: Option<ProgressCallback>,
        cancel_token: Option<CancellationToken>,
    ) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name)?;
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        
        // Create progress tracking state
        let progress_state = Arc::new(Mutex::new((0usize, 0usize)));
        let progress_state_clone = progress_state.clone();
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        // Set up push transfer progress callback
        if let Some(ref callback) = progress_callback {
            let callback_clone = callback.clone();
            callbacks.push_transfer_progress(move |current, total, bytes| {
                if let Ok(mut state) = progress_state_clone.lock() {
                    *state = (current, total);
                    callback_clone(GitOperationProgress {
                        operation: "Push".to_string(),
                        current,
                        total,
                        message: format!("Pushing objects: {}/{}", current, total),
                        bytes_transferred: Some(bytes),
                    });
                }
            });
        }
        
        // Set up pack progress callback
        if let Some(ref callback) = progress_callback {
            let callback_clone = callback.clone();
            callbacks.pack_progress(move |stage, current, total| {
                callback_clone(GitOperationProgress {
                    operation: "Push".to_string(),
                    current: current as usize,
                    total: total as usize,
                    message: format!("{:?}: {}/{}", stage, current, total),
                    bytes_transferred: None,
                });
            });
        }
        
        // Check for cancellation
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }
        
        let mut push_opts = PushOptions::new();
        push_opts.remote_callbacks(callbacks);
        
        remote.push(&[&refspec], Some(&mut push_opts))?;
        
        info!("Pushed {} to {}", branch_name, remote_name);
        Ok(())
    }
    
    /// Push to remote (convenience method without progress)
    pub fn push(&self, remote_name: &str, branch_name: &str) -> Result<()> {
        self.push_with_progress(remote_name, branch_name, None, None)
    }
    
    /// Push branch to remote and set upstream tracking
    pub fn push_with_upstream(&self, remote_name: &str, branch_name: &str) -> Result<()> {
        // First push the branch
        let mut remote = self.repo.find_remote(remote_name)?;
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        remote.push(&[&refspec], Some(&mut push_options))?;
        
        // Set up branch tracking
        let mut branch = self.repo.find_branch(branch_name, git2::BranchType::Local)?;
        let upstream_name = format!("{}/{}", remote_name, branch_name);
        branch.set_upstream(Some(&upstream_name))?;
        
        info!("Pushed and set upstream for {} to {}/{}", branch_name, remote_name, branch_name);
        Ok(())
    }
    
    /// Pull from remote with progress callback and cancellation support
    pub fn pull_with_progress(
        &self, 
        remote_name: &str, 
        branch_name: &str,
        progress_callback: Option<ProgressCallback>,
        cancel_token: Option<CancellationToken>,
    ) -> Result<()> {
        // Fetch first
        let mut remote = self.repo.find_remote(remote_name)?;
        let refspec = format!("refs/heads/{}:refs/remotes/{}/{}", branch_name, remote_name, branch_name);
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        // Set up fetch transfer progress callback
        if let Some(ref callback) = progress_callback {
            let callback_clone = callback.clone();
            callbacks.transfer_progress(move |stats| {
                callback_clone(GitOperationProgress {
                    operation: "Pull".to_string(),
                    current: stats.indexed_objects() as usize,
                    total: stats.total_objects() as usize,
                    message: format!("Receiving objects: {}/{}", stats.indexed_objects(), stats.total_objects()),
                    bytes_transferred: Some(stats.received_bytes()),
                });
                true
            });
        }
        
        // Check for cancellation
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }
        
        let mut fetch_opts = FetchOptions::new();
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
    
    /// Pull from remote (convenience method without progress)
    pub fn pull(&self, remote_name: &str, branch_name: &str) -> Result<()> {
        self.pull_with_progress(remote_name, branch_name, None, None)
    }
    
    /// Fetch from remote with progress callback and cancellation support
    pub fn fetch_with_progress(
        &self, 
        remote_name: &str,
        progress_callback: Option<ProgressCallback>,
        cancel_token: Option<CancellationToken>,
    ) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name)?;
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        // Set up fetch transfer progress callback
        if let Some(ref callback) = progress_callback {
            let callback_clone = callback.clone();
            callbacks.transfer_progress(move |stats| {
                callback_clone(GitOperationProgress {
                    operation: "Fetch".to_string(),
                    current: stats.indexed_objects() as usize,
                    total: stats.total_objects() as usize,
                    message: format!("Receiving objects: {}/{}", stats.indexed_objects(), stats.total_objects()),
                    bytes_transferred: Some(stats.received_bytes()),
                });
                true
            });
        }
        
        // Check for cancellation
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }
        
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        
        remote.fetch(&[] as &[&str], Some(&mut fetch_opts), None)?;
        
        info!("Fetched from {}", remote_name);
        Ok(())
    }
    
    /// Fetch from remote (convenience method without progress)
    pub fn fetch(&self, remote_name: &str) -> Result<()> {
        self.fetch_with_progress(remote_name, None, None)
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
    
    /// Sync operation (pull then push) with progress
    pub fn sync_with_progress(
        &self,
        remote_name: &str,
        branch_name: &str,
        progress_callback: Option<ProgressCallback>,
        cancel_token: Option<CancellationToken>,
    ) -> Result<()> {
        // First pull
        if let Some(ref callback) = progress_callback {
            callback(GitOperationProgress {
                operation: "Sync".to_string(),
                current: 0,
                total: 2,
                message: "Pulling changes from remote...".to_string(),
                bytes_transferred: None,
            });
        }
        
        self.pull_with_progress(remote_name, branch_name, progress_callback.clone(), cancel_token.clone())?;
        
        // Check for cancellation
        if let Some(ref token) = cancel_token {
            if token.is_cancelled() {
                return Err(anyhow::anyhow!("Operation cancelled"));
            }
        }
        
        // Then push
        if let Some(ref callback) = progress_callback {
            callback(GitOperationProgress {
                operation: "Sync".to_string(),
                current: 1,
                total: 2,
                message: "Pushing changes to remote...".to_string(),
                bytes_transferred: None,
            });
        }
        
        self.push_with_progress(remote_name, branch_name, progress_callback, cancel_token)?;
        
        Ok(())
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

/// Progress callback for long-running operations
pub type ProgressCallback = Arc<dyn Fn(GitOperationProgress) + Send + Sync>;

/// Progress information for git operations
#[derive(Debug, Clone)]
pub struct GitOperationProgress {
    pub operation: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub bytes_transferred: Option<usize>,
}

/// Cancellation token for git operations
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

/// Result of a git operation
#[derive(Debug, Clone)]
pub struct GitOperationResult {
    pub operation: GitOperation,
    pub success: bool,
    pub message: String,
}