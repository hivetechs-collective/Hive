//! Git stash operations and management
//! 
//! Provides VS Code-style stash functionality including:
//! - List stashes with descriptions and timestamps
//! - Save stash with optional message
//! - Apply/pop/drop stash operations
//! - Show stash diff and content
//! - Auto-stash before risky operations

use anyhow::{Result, Context};
use git2::{Repository, Oid, Object, Tree, DiffOptions, DiffFormat};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, instrument};
use serde::{Serialize, Deserialize};

use super::{GitOperations, get_optimized_git_manager, log_performance_stats};

/// Information about a git stash entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StashInfo {
    pub index: usize,
    pub oid: String,
    pub message: String,
    pub author: String,
    pub timestamp: u64,
    pub formatted_time: String,
    pub has_untracked: bool,
    pub files_changed: Vec<String>,
    pub stats: StashStats,
}

/// Statistics about a stash entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StashStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl Default for StashStats {
    fn default() -> Self {
        Self {
            files_changed: 0,
            insertions: 0,
            deletions: 0,
        }
    }
}

/// Options for creating a stash
#[derive(Debug, Clone, Default)]
pub struct StashSaveOptions {
    pub message: Option<String>,
    pub include_untracked: bool,
    pub include_ignored: bool,
    pub keep_index: bool,
}

/// Options for applying a stash
#[derive(Debug, Clone, Default)]
pub struct StashApplyOptions {
    pub reinstanate_index: bool,
    pub check_index: bool,
}

/// Git stash operations service
pub struct GitStash {
    repo: Repository,
    repo_path: PathBuf,
}

impl GitStash {
    /// Create a new GitStash instance
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
    
    /// List all stashes in the repository
    #[instrument(skip(self))]
    pub fn list_stashes(&mut self) -> Result<Vec<StashInfo>> {
        let mut stashes = Vec::new();
        
        self.repo.stash_foreach(|index, message, oid| {
            match self.get_stash_info(index, message, *oid) {
                Ok(stash_info) => {
                    stashes.push(stash_info);
                    true // Continue iteration
                }
                Err(e) => {
                    warn!("Failed to get stash info for index {}: {}", index, e);
                    true // Continue iteration despite error
                }
            }
        })?;
        
        info!("Found {} stashes", stashes.len());
        Ok(stashes)
    }
    
    /// Get detailed information about a specific stash
    fn get_stash_info(&self, index: usize, message: &str, oid: Oid) -> Result<StashInfo> {
        let commit = self.repo.find_commit(oid)?;
        let author = commit.author();
        let timestamp = author.when().seconds() as u64;
        
        // Format timestamp
        let formatted_time = if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let now = duration.as_secs();
            let diff = now.saturating_sub(timestamp);
            
            if diff < 60 {
                format!("{} seconds ago", diff)
            } else if diff < 3600 {
                format!("{} minutes ago", diff / 60)
            } else if diff < 86400 {
                format!("{} hours ago", diff / 3600)
            } else {
                format!("{} days ago", diff / 86400)
            }
        } else {
            "unknown time".to_string()
        };
        
        // Check if stash has untracked files (3rd parent)
        let has_untracked = commit.parent_count() > 2;
        
        // Get files changed and stats
        let (files_changed, stats) = self.get_stash_changes(oid)?;
        
        Ok(StashInfo {
            index,
            oid: oid.to_string(),
            message: message.to_string(),
            author: format!("{} <{}>", author.name().unwrap_or("Unknown"), author.email().unwrap_or("unknown@example.com")),
            timestamp,
            formatted_time,
            has_untracked,
            files_changed,
            stats,
        })
    }
    
    /// Get the changes in a stash
    fn get_stash_changes(&self, stash_oid: Oid) -> Result<(Vec<String>, StashStats)> {
        let stash_commit = self.repo.find_commit(stash_oid)?;
        
        // Get the stash tree and parent tree
        let stash_tree = stash_commit.tree()?;
        let parent_tree = if stash_commit.parent_count() > 0 {
            Some(stash_commit.parent(0)?.tree()?)
        } else {
            None
        };
        
        let mut files_changed = Vec::new();
        let mut stats = StashStats::default();
        
        if let Some(parent_tree) = parent_tree {
            let diff = self.repo.diff_tree_to_tree(
                Some(&parent_tree),
                Some(&stash_tree),
                None,
            )?;
            
            diff.foreach(
                &mut |delta, _progress| {
                    if let (Some(old_file), Some(new_file)) = (delta.old_file().path(), delta.new_file().path()) {
                        let file_path = new_file.to_string_lossy().to_string();
                        if !files_changed.contains(&file_path) {
                            files_changed.push(file_path);
                        }
                    }
                    stats.files_changed += 1;
                    true
                },
                None,
                None,
                Some(&mut |_delta, _hunk, line| {
                    match line.origin() {
                        '+' => stats.insertions += 1,
                        '-' => stats.deletions += 1,
                        _ => {}
                    }
                    true
                }),
            )?;
        }
        
        Ok((files_changed, stats))
    }
    
    /// Save current changes as a stash
    #[instrument(skip(self), fields(message = ?opts.message))]
    pub fn save_stash(&mut self, opts: StashSaveOptions) -> Result<Oid> {
        let signature = self.get_signature()?;
        let message = opts.message.as_deref().unwrap_or("WIP on branch");
        
        // Determine stash flags
        let mut flags = git2::StashFlags::DEFAULT;
        if opts.include_untracked {
            flags |= git2::StashFlags::INCLUDE_UNTRACKED;
        }
        if opts.include_ignored {
            flags |= git2::StashFlags::INCLUDE_IGNORED;
        }
        if opts.keep_index {
            flags |= git2::StashFlags::KEEP_INDEX;
        }
        
        let stash_oid = self.repo.stash_save2(&signature, Some(message), Some(flags))?;
        
        info!("Created stash: {} - {}", stash_oid, message);
        Ok(stash_oid)
    }
    
    /// Quick stash with default options
    pub fn quick_stash(&mut self) -> Result<Oid> {
        let current_branch = self.get_current_branch_name()?;
        let message = format!("WIP on {}", current_branch);
        
        self.save_stash(StashSaveOptions {
            message: Some(message),
            include_untracked: true,
            ..Default::default()
        })
    }
    
    /// Apply a stash by index
    #[instrument(skip(self), fields(index = %index))]
    pub fn apply_stash(&mut self, index: usize, opts: StashApplyOptions) -> Result<()> {
        let mut apply_opts = git2::StashApplyOptions::new();
        
        // Set progress callback
        apply_opts.progress_cb(|progress| {
            info!("Applying stash: {:?}", progress);
            true
        });
        
        // Set apply flags
        if opts.reinstanate_index {
            apply_opts.reinstantiate_index();
        }
        
        self.repo.stash_apply(index, Some(&mut apply_opts))?;
        
        info!("Applied stash at index {}", index);
        Ok(())
    }
    
    /// Pop a stash (apply and remove)
    #[instrument(skip(self), fields(index = %index))]
    pub fn pop_stash(&mut self, index: usize, opts: StashApplyOptions) -> Result<()> {
        let mut apply_opts = git2::StashApplyOptions::new();
        
        // Set progress callback
        apply_opts.progress_cb(|progress| {
            info!("Popping stash: {:?}", progress);
            true
        });
        
        // Set apply flags
        if opts.reinstanate_index {
            apply_opts.reinstantiate_index();
        }
        
        self.repo.stash_pop(index, Some(&mut apply_opts))?;
        
        info!("Popped stash at index {}", index);
        Ok(())
    }
    
    /// Drop a stash
    #[instrument(skip(self), fields(index = %index))]
    pub fn drop_stash(&mut self, index: usize) -> Result<()> {
        self.repo.stash_drop(index)?;
        info!("Dropped stash at index {}", index);
        Ok(())
    }
    
    /// Show stash diff as text
    pub fn show_stash_diff(&mut self, index: usize) -> Result<String> {
        let stashes = self.list_stashes()?;
        if index >= stashes.len() {
            return Err(anyhow::anyhow!("Stash index {} not found", index));
        }
        
        let stash_info = &stashes[index];
        let stash_oid = Oid::from_str(&stash_info.oid)?;
        let stash_commit = self.repo.find_commit(stash_oid)?;
        
        // Get the stash tree and parent tree
        let stash_tree = stash_commit.tree()?;
        let parent_tree = if stash_commit.parent_count() > 0 {
            Some(stash_commit.parent(0)?.tree()?)
        } else {
            None
        };
        
        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&stash_tree),
            None,
        )?;
        
        let mut diff_text = String::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("<binary>");
            match line.origin() {
                '+' | '-' | ' ' => diff_text.push(line.origin()),
                _ => {}
            }
            diff_text.push_str(content);
            true
        })?;
        
        Ok(diff_text)
    }
    
    /// Get files changed in a stash
    pub fn get_stash_files(&mut self, index: usize) -> Result<Vec<String>> {
        let stashes = self.list_stashes()?;
        if index >= stashes.len() {
            return Err(anyhow::anyhow!("Stash index {} not found", index));
        }
        
        Ok(stashes[index].files_changed.clone())
    }
    
    /// Check if there are any stashes
    pub fn has_stashes(&mut self) -> bool {
        let mut has_any = false;
        let _ = self.repo.stash_foreach(|_index, _message, _oid| {
            has_any = true;
            false // Stop after first stash found
        });
        has_any
    }
    
    /// Get the number of stashes
    pub fn stash_count(&mut self) -> usize {
        let mut count = 0;
        let _ = self.repo.stash_foreach(|_index, _message, _oid| {
            count += 1;
            true // Continue counting
        });
        count
    }
    
    /// Auto-stash before potentially destructive operations
    pub fn auto_stash_if_needed(&mut self) -> Result<Option<Oid>> {
        // Check if working directory is clean
        if self.is_working_directory_clean()? {
            return Ok(None);
        }
        
        // Create auto-stash
        let current_branch = self.get_current_branch_name()?;
        let message = format!("Auto-stash before operation on {}", current_branch);
        
        let stash_oid = self.save_stash(StashSaveOptions {
            message: Some(message),
            include_untracked: true,
            ..Default::default()
        })?;
        
        info!("Created auto-stash: {}", stash_oid);
        Ok(Some(stash_oid))
    }
    
    /// Check if working directory is clean
    fn is_working_directory_clean(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(statuses.is_empty())
    }
    
    /// Get current branch name
    fn get_current_branch_name(&self) -> Result<String> {
        let head = self.repo.head()?;
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }
    
    /// Get git signature for commits
    fn get_signature(&self) -> Result<git2::Signature> {
        // Try to get signature from config
        if let Ok(config) = self.repo.config() {
            if let (Ok(name), Ok(email)) = (config.get_string("user.name"), config.get_string("user.email")) {
                return Ok(git2::Signature::now(&name, &email)?);
            }
        }
        
        // Fallback to default
        Ok(git2::Signature::now("Hive User", "user@hive.local")?)
    }
}

/// Async wrapper functions for stash operations
pub mod async_ops {
    use super::*;
    use std::time::Instant;
    
    /// Async wrapper for listing stashes
    pub async fn list_stashes(repo_path: &Path) -> Result<Vec<StashInfo>> {
        let repo_path = repo_path.to_path_buf();
        let start_time = Instant::now();
        
        let result = tokio::task::spawn_blocking(move || -> Result<Vec<StashInfo>> {
            let stash = GitStash::new(&repo_path)?;
            stash.list_stashes()
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Listed stashes in {:?}", elapsed);
        result
    }
    
    /// Async wrapper for saving stash
    pub async fn save_stash(repo_path: &Path, opts: StashSaveOptions) -> Result<Oid> {
        let repo_path = repo_path.to_path_buf();
        let start_time = Instant::now();
        
        let result = tokio::task::spawn_blocking(move || -> Result<Oid> {
            let stash = GitStash::new(&repo_path)?;
            stash.save_stash(opts)
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Saved stash in {:?}", elapsed);
        result
    }
    
    /// Async wrapper for applying stash
    pub async fn apply_stash(repo_path: &Path, index: usize, opts: StashApplyOptions) -> Result<()> {
        let repo_path = repo_path.to_path_buf();
        let start_time = Instant::now();
        
        let result = tokio::task::spawn_blocking(move || -> Result<()> {
            let stash = GitStash::new(&repo_path)?;
            stash.apply_stash(index, opts)
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Applied stash in {:?}", elapsed);
        result
    }
    
    /// Async wrapper for popping stash
    pub async fn pop_stash(repo_path: &Path, index: usize, opts: StashApplyOptions) -> Result<()> {
        let repo_path = repo_path.to_path_buf();
        let start_time = Instant::now();
        
        let result = tokio::task::spawn_blocking(move || -> Result<()> {
            let stash = GitStash::new(&repo_path)?;
            stash.pop_stash(index, opts)
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Popped stash in {:?}", elapsed);
        result
    }
    
    /// Async wrapper for dropping stash
    pub async fn drop_stash(repo_path: &Path, index: usize) -> Result<()> {
        let repo_path = repo_path.to_path_buf();
        let start_time = Instant::now();
        
        let result = tokio::task::spawn_blocking(move || -> Result<()> {
            let stash = GitStash::new(&repo_path)?;
            stash.drop_stash(index)
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Dropped stash in {:?}", elapsed);
        result
    }
    
    /// Async wrapper for showing stash diff
    pub async fn show_stash_diff(repo_path: &Path, index: usize) -> Result<String> {
        let repo_path = repo_path.to_path_buf();
        let start_time = Instant::now();
        
        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            let stash = GitStash::new(&repo_path)?;
            stash.show_stash_diff(index)
        }).await?;
        
        let elapsed = start_time.elapsed();
        info!("Generated stash diff in {:?}", elapsed);
        result
    }
}