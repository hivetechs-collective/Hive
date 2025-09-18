//! LazyGit Data Provider
//!
//! Extracts structured data from LazyGit for use in our GUI.
//! This allows us to use LazyGit as a backend while maintaining our custom UI.

use crate::desktop::git::{DiffHunk, DiffLine, DiffLineType, DiffResult, FileStatus, StatusType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info, warn};

/// LazyGit data provider for extracting git information
pub struct LazyGitDataProvider {
    repo_path: PathBuf,
}

/// Raw git status from git command
#[derive(Debug, Clone)]
struct RawGitStatus {
    status_code: String,
    file_path: String,
}

impl LazyGitDataProvider {
    /// Create a new LazyGit data provider for a repository
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    /// Get file statuses using git status command (same data LazyGit uses)
    pub async fn get_file_statuses(&self) -> Result<HashMap<PathBuf, FileStatus>> {
        info!("📊 Getting file statuses from git for LazyGit data");

        // Use git status --porcelain=v1 for machine-readable output
        let output = Command::new("git")
            .args(&["status", "--porcelain=v1"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to run git status")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git status failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut statuses = HashMap::new();

        // Parse porcelain output
        for line in stdout.lines() {
            if line.len() < 3 {
                continue;
            }

            let status_code = &line[0..2];
            let file_path = &line[3..];

            // Handle renamed files (format: "R  old -> new")
            let (file_path, old_path) = if file_path.contains(" -> ") {
                let parts: Vec<&str> = file_path.split(" -> ").collect();
                if parts.len() == 2 {
                    (parts[1], Some(parts[0]))
                } else {
                    (file_path, None)
                }
            } else {
                (file_path, None)
            };

            let path = PathBuf::from(file_path);
            let (status_type, is_staged, has_staged, has_unstaged) =
                self.parse_status_code(status_code);

            let file_status = FileStatus {
                path: path.clone(),
                status_type,
                is_staged,
                has_staged_changes: has_staged,
                has_unstaged_changes: has_unstaged,
            };

            statuses.insert(path, file_status);

            // Handle old path for renames
            if let Some(old) = old_path {
                let old_path = PathBuf::from(old);
                let old_status = FileStatus {
                    path: old_path.clone(),
                    status_type: StatusType::Deleted,
                    is_staged: true,
                    has_staged_changes: true,
                    has_unstaged_changes: false,
                };
                statuses.insert(old_path, old_status);
            }
        }

        debug!("Found {} changed files", statuses.len());
        Ok(statuses)
    }

    /// Parse git status code into our status type
    fn parse_status_code(&self, code: &str) -> (StatusType, bool, bool, bool) {
        let chars: Vec<char> = code.chars().collect();
        if chars.len() < 2 {
            return (StatusType::Untracked, false, false, false);
        }

        let index_status = chars[0];
        let worktree_status = chars[1];

        // Determine status type
        let status_type = match (index_status, worktree_status) {
            ('A', _) | (_, 'A') => StatusType::Added,
            ('M', _) | (_, 'M') => StatusType::Modified,
            ('D', _) | (_, 'D') => StatusType::Deleted,
            ('R', _) => StatusType::Renamed,
            ('C', _) => StatusType::Copied,
            ('?', '?') => StatusType::Untracked,
            ('!', '!') => StatusType::Ignored,
            ('U', _) | (_, 'U') => StatusType::Conflicted,
            _ => StatusType::Modified, // Default fallback
        };

        // Determine staging status
        let has_staged = index_status != ' ' && index_status != '?' && index_status != '!';
        let has_unstaged =
            worktree_status != ' ' && worktree_status != '?' && worktree_status != '!';
        let is_staged = has_staged && !has_unstaged;

        (status_type, is_staged, has_staged, has_unstaged)
    }

    /// Get diff for a specific file
    pub async fn get_file_diff(&self, file_path: &Path) -> Result<DiffResult> {
        info!("📄 Getting diff for file: {}", file_path.display());

        // Get the relative path from repo root
        let relative_path = file_path.strip_prefix(&self.repo_path).unwrap_or(file_path);

        // Get staged diff (HEAD vs index)
        let staged_output = Command::new("git")
            .args(&[
                "diff",
                "--cached",
                "--no-color",
                "--",
                relative_path.to_str().unwrap(),
            ])
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to get staged diff")?;

        // Get unstaged diff (index vs working tree)
        let unstaged_output = Command::new("git")
            .args(&["diff", "--no-color", "--", relative_path.to_str().unwrap()])
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to get unstaged diff")?;

        // If both are empty, check if it's an untracked file
        let staged_diff = String::from_utf8_lossy(&staged_output.stdout);
        let unstaged_diff = String::from_utf8_lossy(&unstaged_output.stdout);

        let diff_content = if !staged_diff.is_empty() {
            staged_diff.to_string()
        } else if !unstaged_diff.is_empty() {
            unstaged_diff.to_string()
        } else {
            // Check if untracked
            let status_output = Command::new("git")
                .args(&[
                    "status",
                    "--porcelain",
                    "--",
                    relative_path.to_str().unwrap(),
                ])
                .current_dir(&self.repo_path)
                .output()
                .await?;

            let status = String::from_utf8_lossy(&status_output.stdout);
            if status.starts_with("??") {
                // Untracked file - show as all additions
                self.get_untracked_file_diff(file_path).await?
            } else {
                // No changes
                return Ok(DiffResult {
                    original_lines: vec![],
                    modified_lines: vec![],
                    hunks: vec![],
                });
            }
        };

        // Parse the diff
        self.parse_git_diff(&diff_content)
    }

    /// Get diff for untracked file (show as all additions)
    async fn get_untracked_file_diff(&self, file_path: &Path) -> Result<String> {
        let content = tokio::fs::read_to_string(file_path)
            .await
            .context("Failed to read untracked file")?;

        // Format as a git diff
        let relative_path = file_path.strip_prefix(&self.repo_path).unwrap_or(file_path);

        let mut diff = String::new();
        diff.push_str(&format!(
            "diff --git a/{} b/{}\n",
            relative_path.display(),
            relative_path.display()
        ));
        diff.push_str("new file mode 100644\n");
        diff.push_str("--- /dev/null\n");
        diff.push_str(&format!("+++ b/{}\n", relative_path.display()));
        diff.push_str("@@ -0,0 +1,");
        diff.push_str(&content.lines().count().to_string());
        diff.push_str(" @@\n");

        for line in content.lines() {
            diff.push('+');
            diff.push_str(line);
            diff.push('\n');
        }

        Ok(diff)
    }

    /// Parse git diff output into our DiffResult structure
    fn parse_git_diff(&self, diff_content: &str) -> Result<DiffResult> {
        let lines: Vec<String> = diff_content.lines().map(|s| s.to_string()).collect();
        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut hunk_lines = Vec::new();
        let mut original_lines = Vec::new();
        let mut modified_lines = Vec::new();
        let mut line_num_old = 0;
        let mut line_num_new = 0;

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("@@") {
                // Save previous hunk if exists
                if let Some(mut hunk) = current_hunk.take() {
                    hunk.lines = hunk_lines.clone();
                    hunks.push(hunk);
                    hunk_lines.clear();
                }

                // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let old_info = parts[1].trim_start_matches('-');
                    let new_info = parts[2].trim_start_matches('+');

                    let (old_start, old_count) = self.parse_hunk_info(old_info);
                    let (new_start, new_count) = self.parse_hunk_info(new_info);

                    line_num_old = old_start;
                    line_num_new = new_start;

                    current_hunk = Some(DiffHunk {
                        original_start: old_start,
                        original_count: old_count,
                        modified_start: new_start,
                        modified_count: new_count,
                        lines: vec![],
                        hunk_id: format!("hunk_{}", i),
                        is_staged: false,
                        is_stageable: true,
                        stage_status: crate::desktop::git::HunkStageStatus::Unstaged,
                    });
                }
            } else if let Some(_) = &current_hunk {
                // Process diff lines
                let (line_type, content) = if line.starts_with('+') {
                    modified_lines.push(line[1..].to_string());
                    (DiffLineType::Added, line[1..].to_string())
                } else if line.starts_with('-') {
                    original_lines.push(line[1..].to_string());
                    (DiffLineType::Deleted, line[1..].to_string())
                } else if line.starts_with(' ') {
                    original_lines.push(line[1..].to_string());
                    modified_lines.push(line[1..].to_string());
                    (DiffLineType::Unchanged, line[1..].to_string())
                } else {
                    continue;
                };

                let diff_line = DiffLine {
                    line_type: line_type.clone(),
                    original_line_number: if matches!(
                        line_type,
                        DiffLineType::Deleted | DiffLineType::Unchanged
                    ) {
                        Some(line_num_old)
                    } else {
                        None
                    },
                    modified_line_number: if matches!(
                        line_type,
                        DiffLineType::Added | DiffLineType::Unchanged
                    ) {
                        Some(line_num_new)
                    } else {
                        None
                    },
                    content,
                    line_id: format!("line_{}_{}", i, hunk_lines.len()),
                    is_staged: false,
                    is_stageable: true,
                    hunk_id: format!("hunk_{}", hunks.len()),
                };

                // Update line numbers
                match line_type {
                    DiffLineType::Deleted => line_num_old += 1,
                    DiffLineType::Added => line_num_new += 1,
                    DiffLineType::Unchanged => {
                        line_num_old += 1;
                        line_num_new += 1;
                    }
                    _ => {}
                }

                hunk_lines.push(diff_line);
            }
        }

        // Save last hunk
        if let Some(mut hunk) = current_hunk {
            hunk.lines = hunk_lines;
            hunks.push(hunk);
        }

        Ok(DiffResult {
            original_lines,
            modified_lines,
            hunks,
        })
    }

    /// Parse hunk info like "25,10" into (start, count)
    fn parse_hunk_info(&self, info: &str) -> (usize, usize) {
        let parts: Vec<&str> = info.split(',').collect();
        let start = parts[0].parse::<usize>().unwrap_or(1);
        let count = if parts.len() > 1 {
            parts[1].parse::<usize>().unwrap_or(1)
        } else {
            1
        };
        (start, count)
    }

    /// Get current branch name
    pub async fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .context("Failed to get current branch")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get current branch"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get ahead/behind counts
    pub async fn get_ahead_behind(&self) -> Result<(usize, usize)> {
        let output = Command::new("git")
            .args(&["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(&self.repo_path)
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                let result = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = result.trim().split('\t').collect();
                if parts.len() == 2 {
                    let ahead = parts[0].parse::<usize>().unwrap_or(0);
                    let behind = parts[1].parse::<usize>().unwrap_or(0);
                    Ok((ahead, behind))
                } else {
                    Ok((0, 0))
                }
            }
            _ => Ok((0, 0)), // No upstream
        }
    }
}

/// Get file statuses using LazyGit data provider
pub async fn get_lazygit_file_statuses(repo_path: &Path) -> Result<HashMap<PathBuf, FileStatus>> {
    let provider = LazyGitDataProvider::new(repo_path);
    provider.get_file_statuses().await
}

/// Get file diff using LazyGit data provider
pub async fn get_lazygit_file_diff(repo_path: &Path, file_path: &Path) -> Result<DiffResult> {
    let provider = LazyGitDataProvider::new(repo_path);
    provider.get_file_diff(file_path).await
}
