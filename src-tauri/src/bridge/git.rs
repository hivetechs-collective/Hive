// Git and LazyGit integration bridge

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::spawn;
use tauri::{Window, Emitter};

// Re-use our existing terminal bridge
use crate::bridge::{create_terminal, TerminalInfo};

// Simple result type for Tauri
type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub staged: Vec<GitFileStatus>,
    pub unstaged: Vec<GitFileStatus>,
    pub untracked: Vec<GitFileStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String, // "modified", "added", "deleted", "renamed"
    pub staged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

/// Create a LazyGit terminal session
#[tauri::command]
pub async fn create_lazygit_terminal(path: String, window: Window) -> Result<TerminalInfo> {
    // First check if lazygit is installed
    let check_output = std::process::Command::new("which")
        .arg("lazygit")
        .output()
        .map_err(|e| format!("Failed to check for lazygit: {}", e))?;
    
    if !check_output.status.success() {
        return Err("LazyGit is not installed. Please install it first.".to_string());
    }
    
    // Find git root from the given path
    let git_root = find_git_root(&path)?;
    
    // Create a terminal with lazygit command
    let id = uuid::Uuid::new_v4().to_string();
    
    // Use our existing PTY implementation
    let (pty, mut rx) = hive_ai::desktop::terminal_pty::PtyProcess::spawn(
        "lazygit",
        &[],
        &git_root,
    )
    .map_err(|e| format!("Failed to create LazyGit terminal: {}", e))?;
    
    let pty = std::sync::Arc::new(pty);
    let terminal_id = id.clone();
    let window_clone = window.clone();
    
    // Spawn output handler
    let pty_clone = pty.clone();
    spawn(async move {
        while let Some(output) = rx.recv().await {
            let _ = window_clone.emit("terminal-output", serde_json::json!({
                "id": terminal_id,
                "data": output,
                "is_lazygit": true
            }));
            
            if !pty_clone.is_running().await {
                let _ = window_clone.emit("terminal-closed", terminal_id.clone());
                break;
            }
        }
    });
    
    // Store terminal instance
    let mut terminals = crate::bridge::TERMINALS.lock().await;
    terminals.insert(id.clone(), crate::bridge::TerminalInstance { pty });
    
    Ok(TerminalInfo {
        id,
        title: format!("LazyGit - {}", git_root),
        rows: 40,
        cols: 120,
    })
}

/// Get git status for a directory
#[tauri::command]
pub async fn get_git_status(path: String) -> Result<GitStatus> {
    use git2::{Repository, StatusOptions};
    
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    // Get current branch
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    
    let branch = if head.is_branch() {
        head.shorthand().unwrap_or("unknown").to_string()
    } else {
        "HEAD (detached)".to_string()
    };
    
    // Get status
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    
    let statuses = repo.statuses(Some(&mut opts))
        .map_err(|e| format!("Failed to get status: {}", e))?;
    
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();
        
        let status_str = if status.is_index_new() || status.is_wt_new() {
            "added"
        } else if status.is_index_modified() || status.is_wt_modified() {
            "modified"
        } else if status.is_index_deleted() || status.is_wt_deleted() {
            "deleted"
        } else if status.is_index_renamed() || status.is_wt_renamed() {
            "renamed"
        } else {
            "unknown"
        };
        
        let file_status = GitFileStatus {
            path: path.clone(),
            status: status_str.to_string(),
            staged: status.is_index_new() || status.is_index_modified() 
                || status.is_index_deleted() || status.is_index_renamed(),
        };
        
        if status.is_wt_new() && !status.is_index_new() {
            untracked.push(file_status.clone());
        } else if file_status.staged {
            staged.push(file_status.clone());
        }
        
        if status.is_wt_modified() || status.is_wt_deleted() {
            unstaged.push(file_status);
        }
    }
    
    // Get ahead/behind counts (simplified for now)
    let (ahead, behind) = (0, 0); // TODO: Implement proper ahead/behind calculation
    
    Ok(GitStatus {
        branch,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
    })
}

/// Get current git branch
#[tauri::command]
pub async fn get_current_branch(path: String) -> Result<String> {
    use git2::Repository;
    
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    
    Ok(if head.is_branch() {
        head.shorthand().unwrap_or("unknown").to_string()
    } else {
        "HEAD (detached)".to_string()
    })
}

/// Get list of branches
#[tauri::command]
pub async fn get_branches(path: String) -> Result<Vec<GitBranch>> {
    use git2::{Repository, BranchType};
    
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    let mut branches = Vec::new();
    
    // Get current branch name for comparison
    let current_branch = get_current_branch(path.clone()).await?;
    
    // Get local branches
    for branch in repo.branches(Some(BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {}", e))? {
        
        let (branch, _) = branch.map_err(|e| format!("Failed to get branch: {}", e))?;
        let name = branch.name()
            .map_err(|e| format!("Failed to get branch name: {}", e))?
            .unwrap_or("").to_string();
        
        branches.push(GitBranch {
            name: name.clone(),
            is_current: name == current_branch,
            is_remote: false,
            upstream: None, // TODO: Get upstream tracking branch
        });
    }
    
    Ok(branches)
}

/// Switch to a different branch
#[tauri::command]
pub async fn switch_branch(path: String, branch_name: String) -> Result<()> {
    use git2::Repository;
    
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    // Find the branch
    let branch = repo.find_branch(&branch_name, git2::BranchType::Local)
        .map_err(|e| format!("Failed to find branch: {}", e))?;
    
    // Get the reference
    let reference = branch.get();
    let tree = reference.peel_to_tree()
        .map_err(|e| format!("Failed to get tree: {}", e))?;
    
    // Checkout
    repo.checkout_tree(tree.as_object(), None)
        .map_err(|e| format!("Failed to checkout: {}", e))?;
    
    // Update HEAD
    repo.set_head(&format!("refs/heads/{}", branch_name))
        .map_err(|e| format!("Failed to set HEAD: {}", e))?;
    
    Ok(())
}

/// Create a new branch
#[tauri::command]
pub async fn create_branch(path: String, branch_name: String) -> Result<()> {
    use git2::Repository;
    
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    // Get current commit
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    
    let oid = head.target()
        .ok_or("HEAD has no target")?;
    
    let commit = repo.find_commit(oid)
        .map_err(|e| format!("Failed to find commit: {}", e))?;
    
    // Create branch
    repo.branch(&branch_name, &commit, false)
        .map_err(|e| format!("Failed to create branch: {}", e))?;
    
    Ok(())
}

/// Stage files for commit
#[tauri::command]
pub async fn stage_files(path: String, files: Vec<String>) -> Result<()> {
    use git2::{Repository, IndexAddOption};
    
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;
    
    for file in files {
        index.add_path(std::path::Path::new(&file))
            .map_err(|e| format!("Failed to stage {}: {}", file, e))?;
    }
    
    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;
    
    Ok(())
}

/// Commit staged changes
#[tauri::command]
pub async fn commit_changes(path: String, message: String) -> Result<String> {
    use git2::{Repository, Signature};
    
    let repo = Repository::open(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    
    // Get signature
    let sig = Signature::now("Hive User", "user@hivetechs.io")
        .or_else(|_| repo.signature())
        .map_err(|e| format!("Failed to get signature: {}", e))?;
    
    // Get current tree
    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;
    
    let tree_id = index.write_tree()
        .map_err(|e| format!("Failed to write tree: {}", e))?;
    
    let tree = repo.find_tree(tree_id)
        .map_err(|e| format!("Failed to find tree: {}", e))?;
    
    // Get parent commit
    let parent = repo.head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());
    
    // Create commit
    let commit_id = if let Some(parent) = parent {
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &message,
            &tree,
            &[&parent],
        )
    } else {
        // Initial commit
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &message,
            &tree,
            &[],
        )
    }.map_err(|e| format!("Failed to create commit: {}", e))?;
    
    Ok(commit_id.to_string())
}

// Helper function to find git root
fn find_git_root(path: &str) -> Result<String> {
    let mut current = PathBuf::from(path);
    
    loop {
        if current.join(".git").exists() {
            return Ok(current.to_string_lossy().to_string());
        }
        
        if !current.pop() {
            return Err(format!("Not in a git repository: {}", path));
        }
    }
}