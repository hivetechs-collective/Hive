//! Git diff functionality
//!
//! Implements diff algorithms and data structures for showing file changes

use anyhow::{Context, Result};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Result of comparing two files
#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub original_lines: Vec<String>,
    pub modified_lines: Vec<String>,
    pub hunks: Vec<DiffHunk>,
}

/// A contiguous section of changes with action support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub original_start: usize,
    pub original_count: usize,
    pub modified_start: usize,
    pub modified_count: usize,
    pub lines: Vec<DiffLine>,
    pub hunk_id: String,               // Unique identifier for this hunk
    pub is_staged: bool,               // Whether this hunk is staged
    pub is_stageable: bool,            // Whether this hunk can be staged
    pub stage_status: HunkStageStatus, // Current staging status
}

/// Type of change for a line
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffLineType {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

/// Status of hunk staging operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HunkStageStatus {
    Unstaged,        // Hunk is not staged
    Staged,          // Entire hunk is staged
    PartiallyStaged, // Some lines in hunk are staged
    Conflicted,      // Staging conflict (e.g., overlapping changes)
}

/// Diff action types for UI interactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffAction {
    StageHunk(String),   // Stage entire hunk by ID
    UnstageHunk(String), // Unstage entire hunk by ID
    StageLine(String),   // Stage individual line by ID
    UnstageLine(String), // Unstage individual line by ID
    RevertHunk(String),  // Revert hunk changes
    RevertLine(String),  // Revert individual line
    UndoAction(String),  // Undo previous action by ID
}

/// Result of a diff action operation
#[derive(Debug, Clone)]
pub struct DiffActionResult {
    pub action: DiffAction,
    pub success: bool,
    pub message: String,
    pub updated_hunks: Vec<String>, // IDs of hunks that were updated
    pub undo_id: Option<String>,    // ID for undoing this action
}

/// Undo entry for diff actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffUndoEntry {
    pub undo_id: String,
    pub action: String, // Serialized action
    pub timestamp: u64,
    pub original_state: String, // Serialized state before action
    pub description: String,
}

/// A single line in a diff with action support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub original_line_number: Option<usize>,
    pub modified_line_number: Option<usize>,
    pub content: String,
    pub line_id: String,    // Unique identifier for this line
    pub is_staged: bool,    // Whether this line is staged
    pub is_stageable: bool, // Whether this line can be staged individually
    pub hunk_id: String,    // Reference to parent hunk
}

/// View mode for displaying diffs
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DiffViewMode {
    SideBySide,
    Inline,
}

/// Enhanced diff result with action support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffActionState {
    pub file_path: PathBuf,
    pub hunks: HashMap<String, DiffHunk>, // Hunk ID -> Hunk
    pub lines: HashMap<String, DiffLine>, // Line ID -> Line
    pub undo_stack: Vec<DiffUndoEntry>,   // Undo history
    pub keyboard_shortcuts_enabled: bool,
    pub auto_stage_single_lines: bool,
    pub show_action_animations: bool,
}

/// Compute diff between two texts with action support
pub fn compute_diff(original: &str, modified: &str) -> DiffResult {
    compute_diff_with_context(original, modified, None)
}

/// Compute diff with additional context for staging support
pub fn compute_diff_with_context(
    original: &str,
    modified: &str,
    file_path: Option<&Path>,
) -> DiffResult {
    let original_lines: Vec<String> = original.lines().map(|s| s.to_string()).collect();
    let modified_lines: Vec<String> = modified.lines().map(|s| s.to_string()).collect();

    // For now, use git2's diff functionality
    // TODO: Implement Myers' O(ND) algorithm for better performance

    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut hunk_lines = Vec::new();
    let mut hunk_counter = 0;
    let mut line_counter = 0;

    // Generate unique prefix for this diff session
    let diff_prefix = format!(
        "{}_{}",
        file_path
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("diff"),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    // Simple line-by-line comparison (temporary implementation)
    let max_lines = original_lines.len().max(modified_lines.len());

    for i in 0..max_lines {
        let original_line = original_lines.get(i);
        let modified_line = modified_lines.get(i);

        match (original_line, modified_line) {
            (Some(orig), Some(modif)) if orig == modif => {
                // Unchanged line
                if !hunk_lines.is_empty() {
                    // Finish current hunk
                    if let Some(mut hunk) = current_hunk.take() {
                        hunk.lines = hunk_lines.clone();
                        hunks.push(hunk);
                        hunk_lines.clear();
                    }
                }
            }
            (Some(orig), Some(modif)) => {
                // Modified line
                if current_hunk.is_none() {
                    hunk_counter += 1;
                    current_hunk = Some(DiffHunk {
                        original_start: i + 1,
                        original_count: 0,
                        modified_start: i + 1,
                        modified_count: 0,
                        lines: Vec::new(),
                        hunk_id: format!("{}_hunk_{}", diff_prefix, hunk_counter),
                        is_staged: false,
                        is_stageable: true,
                        stage_status: HunkStageStatus::Unstaged,
                    });
                }

                line_counter += 1;
                let hunk_id = current_hunk.as_ref().unwrap().hunk_id.clone();
                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Deleted,
                    original_line_number: Some(i + 1),
                    modified_line_number: None,
                    content: orig.clone(),
                    line_id: format!("{}_line_{}", diff_prefix, line_counter),
                    is_staged: false,
                    is_stageable: true,
                    hunk_id: hunk_id.clone(),
                });

                line_counter += 1;
                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    original_line_number: None,
                    modified_line_number: Some(i + 1),
                    content: modif.clone(),
                    line_id: format!("{}_line_{}", diff_prefix, line_counter),
                    is_staged: false,
                    is_stageable: true,
                    hunk_id,
                });

                if let Some(ref mut hunk) = current_hunk {
                    hunk.original_count += 1;
                    hunk.modified_count += 1;
                }
            }
            (Some(orig), None) => {
                // Deleted line
                if current_hunk.is_none() {
                    hunk_counter += 1;
                    current_hunk = Some(DiffHunk {
                        original_start: i + 1,
                        original_count: 0,
                        modified_start: i + 1,
                        modified_count: 0,
                        lines: Vec::new(),
                        hunk_id: format!("{}_hunk_{}", diff_prefix, hunk_counter),
                        is_staged: false,
                        is_stageable: true,
                        stage_status: HunkStageStatus::Unstaged,
                    });
                }

                line_counter += 1;
                let hunk_id = current_hunk.as_ref().unwrap().hunk_id.clone();
                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Deleted,
                    original_line_number: Some(i + 1),
                    modified_line_number: None,
                    content: orig.clone(),
                    line_id: format!("{}_line_{}", diff_prefix, line_counter),
                    is_staged: false,
                    is_stageable: true,
                    hunk_id,
                });

                if let Some(ref mut hunk) = current_hunk {
                    hunk.original_count += 1;
                }
            }
            (None, Some(modif)) => {
                // Added line
                if current_hunk.is_none() {
                    hunk_counter += 1;
                    current_hunk = Some(DiffHunk {
                        original_start: original_lines.len() + 1,
                        original_count: 0,
                        modified_start: i + 1,
                        modified_count: 0,
                        lines: Vec::new(),
                        hunk_id: format!("{}_hunk_{}", diff_prefix, hunk_counter),
                        is_staged: false,
                        is_stageable: true,
                        stage_status: HunkStageStatus::Unstaged,
                    });
                }

                line_counter += 1;

                // Extract hunk_id before the mutable borrow
                let hunk_id = if let Some(ref hunk) = current_hunk {
                    hunk.hunk_id.clone()
                } else {
                    String::new() // This shouldn't happen given the check above
                };

                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    original_line_number: None,
                    modified_line_number: Some(i + 1),
                    content: modif.clone(),
                    line_id: format!("{}_line_{}", diff_prefix, line_counter),
                    is_staged: false,
                    is_stageable: true,
                    hunk_id,
                });

                if let Some(ref mut hunk) = current_hunk {
                    hunk.modified_count += 1;
                }
            }
            (None, None) => unreachable!(),
        }
    }

    // Finish last hunk if any
    if let Some(mut hunk) = current_hunk {
        hunk.lines = hunk_lines;
        hunks.push(hunk);
    }

    DiffResult {
        original_lines,
        modified_lines,
        hunks,
    }
}

/// Get the diff between a file's working tree version and its HEAD version with staging context
pub async fn get_file_diff(repo_path: &PathBuf, file_path: &PathBuf) -> Result<DiffResult> {
    get_file_diff_with_context(repo_path, file_path, None).await
}

/// Get file diff with additional context for staging operations
pub async fn get_file_diff_with_context(
    repo_path: &PathBuf,
    file_path: &PathBuf,
    existing_state: Option<&DiffActionState>,
) -> Result<DiffResult> {
    use crate::desktop::git::GitRepository;
    use tokio::fs;

    // Get the repository
    let repo = GitRepository::open(repo_path)?;

    // Check if file exists
    if !file_path.exists() {
        tracing::error!("File does not exist: {:?}", file_path);
        return Err(anyhow::anyhow!("File does not exist: {:?}", file_path));
    }

    // Read current file content
    let current_content = match fs::read_to_string(file_path).await {
        Ok(content) => content,
        Err(e) => {
            // Log the error and return a more informative error
            tracing::error!("Failed to read file {:?}: {}", file_path, e);
            return Err(anyhow::anyhow!(
                "Failed to read file {:?}: {}",
                file_path,
                e
            ));
        }
    };

    // Get relative path for git
    let relative_path = file_path
        .strip_prefix(repo_path)
        .context("File is not in repository")?;

    // Get HEAD version of the file
    let head_content = match repo.get_file_at_head(relative_path) {
        Ok(content) => content,
        Err(_) => {
            // File might be new (not in HEAD), treat as empty
            String::new()
        }
    };

    let mut diff_result =
        compute_diff_with_context(&head_content, &current_content, Some(file_path));

    // If we have existing state, preserve staging information
    if let Some(state) = existing_state {
        for hunk in &mut diff_result.hunks {
            if let Some(existing_hunk) = state.hunks.get(&hunk.hunk_id) {
                hunk.is_staged = existing_hunk.is_staged;
                hunk.stage_status = existing_hunk.stage_status.clone();

                // Update line staging status
                for line in &mut hunk.lines {
                    if let Some(existing_line) = state.lines.get(&line.line_id) {
                        line.is_staged = existing_line.is_staged;
                    }
                }
            }
        }
    }

    Ok(diff_result)
}

/// Git operations for diff actions (staging, reverting hunks and lines)
pub struct DiffGitOperations {
    repo_path: PathBuf,
    operations: super::operations::GitOperations,
}

impl DiffGitOperations {
    /// Create new diff git operations instance
    pub fn new(repo_path: &Path) -> Result<Self> {
        let operations = super::operations::GitOperations::new(repo_path)?;
        Ok(Self {
            repo_path: repo_path.to_path_buf(),
            operations,
        })
    }

    /// Stage a specific hunk by applying a patch
    pub async fn stage_hunk(&self, file_path: &Path, hunk: &DiffHunk) -> Result<DiffActionResult> {
        let start_time = std::time::Instant::now();

        // Create a patch string for this hunk
        let patch = self.create_hunk_patch(file_path, hunk)?;
        let patch_file = self.write_temp_patch(&patch).await?;

        // Apply the patch using git apply --cached
        let result = tokio::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(["apply", "--cached", &patch_file])
            .output()
            .await?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&patch_file).await;

        let success = result.status.success();
        let message = if success {
            format!(
                "Staged hunk {} successfully in {:?}",
                hunk.hunk_id,
                start_time.elapsed()
            )
        } else {
            format!(
                "Failed to stage hunk {}: {}",
                hunk.hunk_id,
                String::from_utf8_lossy(&result.stderr)
            )
        };

        Ok(DiffActionResult {
            action: DiffAction::StageHunk(hunk.hunk_id.clone()),
            success,
            message,
            updated_hunks: vec![hunk.hunk_id.clone()],
            undo_id: if success {
                Some(self.generate_undo_id())
            } else {
                None
            },
        })
    }

    /// Unstage a specific hunk by applying reverse patch
    pub async fn unstage_hunk(
        &self,
        file_path: &Path,
        hunk: &DiffHunk,
    ) -> Result<DiffActionResult> {
        let start_time = std::time::Instant::now();

        // Create a reverse patch string for this hunk
        let patch = self.create_reverse_hunk_patch(file_path, hunk)?;
        let patch_file = self.write_temp_patch(&patch).await?;

        // Apply the reverse patch using git apply --cached --reverse
        let result = tokio::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(["apply", "--cached", "--reverse", &patch_file])
            .output()
            .await?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&patch_file).await;

        let success = result.status.success();
        let message = if success {
            format!(
                "Unstaged hunk {} successfully in {:?}",
                hunk.hunk_id,
                start_time.elapsed()
            )
        } else {
            format!(
                "Failed to unstage hunk {}: {}",
                hunk.hunk_id,
                String::from_utf8_lossy(&result.stderr)
            )
        };

        Ok(DiffActionResult {
            action: DiffAction::UnstageHunk(hunk.hunk_id.clone()),
            success,
            message,
            updated_hunks: vec![hunk.hunk_id.clone()],
            undo_id: if success {
                Some(self.generate_undo_id())
            } else {
                None
            },
        })
    }

    /// Stage individual lines (more complex, requires line-level patch creation)
    pub async fn stage_lines(
        &self,
        file_path: &Path,
        line_ids: &[String],
        lines: &HashMap<String, DiffLine>,
    ) -> Result<DiffActionResult> {
        let start_time = std::time::Instant::now();

        // Group lines by hunk for efficient patching
        let mut hunks_to_patch = HashMap::new();
        for line_id in line_ids {
            if let Some(line) = lines.get(line_id) {
                hunks_to_patch
                    .entry(line.hunk_id.clone())
                    .or_insert_with(Vec::new)
                    .push(line.clone());
            }
        }

        let mut updated_hunks = Vec::new();
        let mut all_success = true;
        let mut messages = Vec::new();

        for (hunk_id, hunk_lines) in hunks_to_patch {
            // Create a partial hunk patch with only the specified lines
            let patch = self.create_line_patch(file_path, &hunk_lines)?;
            let patch_file = self.write_temp_patch(&patch).await?;

            // Apply the patch
            let result = tokio::process::Command::new("git")
                .current_dir(&self.repo_path)
                .args(["apply", "--cached", &patch_file])
                .output()
                .await?;

            // Clean up temp file
            let _ = tokio::fs::remove_file(&patch_file).await;

            if result.status.success() {
                updated_hunks.push(hunk_id.clone());
                messages.push(format!(
                    "Staged {} lines from hunk {}",
                    hunk_lines.len(),
                    hunk_id
                ));
            } else {
                all_success = false;
                messages.push(format!(
                    "Failed to stage lines from hunk {}: {}",
                    hunk_id,
                    String::from_utf8_lossy(&result.stderr)
                ));
            }
        }

        let message = if all_success {
            format!(
                "Staged {} lines successfully in {:?}: {}",
                line_ids.len(),
                start_time.elapsed(),
                messages.join("; ")
            )
        } else {
            format!("Partial staging completed: {}", messages.join("; "))
        };

        Ok(DiffActionResult {
            action: DiffAction::StageLine(line_ids.join(",")),
            success: all_success,
            message,
            updated_hunks,
            undo_id: if all_success {
                Some(self.generate_undo_id())
            } else {
                None
            },
        })
    }

    /// Revert hunk changes (discard working tree changes)
    pub async fn revert_hunk(&self, file_path: &Path, hunk: &DiffHunk) -> Result<DiffActionResult> {
        let start_time = std::time::Instant::now();

        // Create a reverse patch to revert the hunk changes
        let patch = self.create_revert_hunk_patch(file_path, hunk)?;
        let patch_file = self.write_temp_patch(&patch).await?;

        // Apply the patch to working tree
        let result = tokio::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(["apply", &patch_file])
            .output()
            .await?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&patch_file).await;

        let success = result.status.success();
        let message = if success {
            format!(
                "Reverted hunk {} successfully in {:?}",
                hunk.hunk_id,
                start_time.elapsed()
            )
        } else {
            format!(
                "Failed to revert hunk {}: {}",
                hunk.hunk_id,
                String::from_utf8_lossy(&result.stderr)
            )
        };

        Ok(DiffActionResult {
            action: DiffAction::RevertHunk(hunk.hunk_id.clone()),
            success,
            message,
            updated_hunks: vec![hunk.hunk_id.clone()],
            undo_id: None, // Reverts cannot be undone easily
        })
    }

    /// Create a git patch string for a hunk
    fn create_hunk_patch(&self, file_path: &Path, hunk: &DiffHunk) -> Result<String> {
        let relative_path = file_path
            .strip_prefix(&self.repo_path)
            .context("File is not in repository")?;

        let mut patch = String::new();
        patch.push_str(&format!(
            "diff --git a/{} b/{}\n",
            relative_path.display(),
            relative_path.display()
        ));
        patch.push_str(&format!("index 0000000..1111111 100644\n"));
        patch.push_str(&format!("--- a/{}\n", relative_path.display()));
        patch.push_str(&format!("+++ b/{}\n", relative_path.display()));
        patch.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.original_start, hunk.original_count, hunk.modified_start, hunk.modified_count
        ));

        for line in &hunk.lines {
            match line.line_type {
                DiffLineType::Added => patch.push_str(&format!("+{}\n", line.content)),
                DiffLineType::Deleted => patch.push_str(&format!("-{}\n", line.content)),
                DiffLineType::Unchanged => patch.push_str(&format!(" {}\n", line.content)),
                DiffLineType::Modified => {
                    // Modified lines are represented as delete + add
                    patch.push_str(&format!("-{}\n", line.content));
                }
            }
        }

        Ok(patch)
    }

    /// Create a reverse patch for unstaging
    fn create_reverse_hunk_patch(&self, file_path: &Path, hunk: &DiffHunk) -> Result<String> {
        let relative_path = file_path
            .strip_prefix(&self.repo_path)
            .context("File is not in repository")?;

        let mut patch = String::new();
        patch.push_str(&format!(
            "diff --git a/{} b/{}\n",
            relative_path.display(),
            relative_path.display()
        ));
        patch.push_str(&format!("index 1111111..0000000 100644\n"));
        patch.push_str(&format!("--- a/{}\n", relative_path.display()));
        patch.push_str(&format!("+++ b/{}\n", relative_path.display()));
        patch.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.modified_start, hunk.modified_count, hunk.original_start, hunk.original_count
        ));

        for line in &hunk.lines {
            match line.line_type {
                DiffLineType::Added => patch.push_str(&format!("-{}\n", line.content)),
                DiffLineType::Deleted => patch.push_str(&format!("+{}\n", line.content)),
                DiffLineType::Unchanged => patch.push_str(&format!(" {}\n", line.content)),
                DiffLineType::Modified => {
                    // For reverse, swap the order
                    patch.push_str(&format!("+{}\n", line.content));
                }
            }
        }

        Ok(patch)
    }

    /// Create a patch for specific lines within a hunk
    fn create_line_patch(&self, file_path: &Path, lines: &[DiffLine]) -> Result<String> {
        if lines.is_empty() {
            return Err(anyhow::anyhow!("No lines provided for patch"));
        }

        let relative_path = file_path
            .strip_prefix(&self.repo_path)
            .context("File is not in repository")?;

        // Calculate hunk boundaries from the lines
        let min_orig = lines
            .iter()
            .filter_map(|l| l.original_line_number)
            .min()
            .unwrap_or(1);
        let max_orig = lines
            .iter()
            .filter_map(|l| l.original_line_number)
            .max()
            .unwrap_or(1);
        let min_mod = lines
            .iter()
            .filter_map(|l| l.modified_line_number)
            .min()
            .unwrap_or(1);
        let max_mod = lines
            .iter()
            .filter_map(|l| l.modified_line_number)
            .max()
            .unwrap_or(1);

        let orig_count = max_orig - min_orig + 1;
        let mod_count = max_mod - min_mod + 1;

        let mut patch = String::new();
        patch.push_str(&format!(
            "diff --git a/{} b/{}\n",
            relative_path.display(),
            relative_path.display()
        ));
        patch.push_str(&format!("index 0000000..1111111 100644\n"));
        patch.push_str(&format!("--- a/{}\n", relative_path.display()));
        patch.push_str(&format!("+++ b/{}\n", relative_path.display()));
        patch.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            min_orig, orig_count, min_mod, mod_count
        ));

        for line in lines {
            match line.line_type {
                DiffLineType::Added => patch.push_str(&format!("+{}\n", line.content)),
                DiffLineType::Deleted => patch.push_str(&format!("-{}\n", line.content)),
                DiffLineType::Unchanged => patch.push_str(&format!(" {}\n", line.content)),
                DiffLineType::Modified => patch.push_str(&format!("-{}\n", line.content)),
            }
        }

        Ok(patch)
    }

    /// Create a revert patch for undoing working tree changes
    fn create_revert_hunk_patch(&self, file_path: &Path, hunk: &DiffHunk) -> Result<String> {
        // Revert patch reverses the working tree changes
        self.create_reverse_hunk_patch(file_path, hunk)
    }

    /// Write patch to temporary file
    async fn write_temp_patch(&self, patch: &str) -> Result<String> {
        let temp_dir = std::env::temp_dir();
        let patch_file = temp_dir.join(format!(
            "hive_patch_{}.patch",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        ));

        tokio::fs::write(&patch_file, patch).await?;
        Ok(patch_file.to_string_lossy().to_string())
    }

    /// Generate unique undo ID
    fn generate_undo_id(&self) -> String {
        format!(
            "undo_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        )
    }
}

impl DiffActionState {
    /// Create new diff action state
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            hunks: HashMap::new(),
            lines: HashMap::new(),
            undo_stack: Vec::new(),
            keyboard_shortcuts_enabled: true,
            auto_stage_single_lines: false,
            show_action_animations: true,
        }
    }

    /// Update state after applying a diff result
    pub fn update_from_diff(&mut self, diff: &DiffResult) {
        self.hunks.clear();
        self.lines.clear();

        for hunk in &diff.hunks {
            self.hunks.insert(hunk.hunk_id.clone(), hunk.clone());

            for line in &hunk.lines {
                self.lines.insert(line.line_id.clone(), line.clone());
            }
        }
    }

    /// Apply diff action and update state
    pub fn apply_action(&mut self, result: &DiffActionResult) {
        match &result.action {
            DiffAction::StageHunk(hunk_id) => {
                if let Some(hunk) = self.hunks.get_mut(hunk_id) {
                    hunk.is_staged = result.success;
                    hunk.stage_status = if result.success {
                        HunkStageStatus::Staged
                    } else {
                        HunkStageStatus::Conflicted
                    };

                    // Update all lines in the hunk
                    for line in &mut hunk.lines {
                        line.is_staged = result.success;
                        self.lines.insert(line.line_id.clone(), line.clone());
                    }
                }
            }
            DiffAction::UnstageHunk(hunk_id) => {
                if let Some(hunk) = self.hunks.get_mut(hunk_id) {
                    hunk.is_staged = false;
                    hunk.stage_status = HunkStageStatus::Unstaged;

                    // Update all lines in the hunk
                    for line in &mut hunk.lines {
                        line.is_staged = false;
                        self.lines.insert(line.line_id.clone(), line.clone());
                    }
                }
            }
            _ => {} // Handle other actions as needed
        }

        // Add to undo stack if successful and undo_id is provided
        if result.success && result.undo_id.is_some() {
            let undo_entry = DiffUndoEntry {
                undo_id: result.undo_id.as_ref().unwrap().clone(),
                action: serde_json::to_string(&result.action).unwrap_or_default(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                original_state: serde_json::to_string(&self).unwrap_or_default(),
                description: result.message.clone(),
            };

            self.undo_stack.push(undo_entry);

            // Keep undo stack size reasonable
            if self.undo_stack.len() > 50 {
                self.undo_stack.remove(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_diff() {
        let original = "line1\nline2\nline3";
        let modified = "line1\nline2 modified\nline3\nline4";

        let diff = compute_diff(original, modified);

        assert_eq!(diff.original_lines.len(), 3);
        assert_eq!(diff.modified_lines.len(), 4);
        assert!(!diff.hunks.is_empty());

        // Check that hunks have unique IDs
        for hunk in &diff.hunks {
            assert!(!hunk.hunk_id.is_empty());
            assert!(hunk.is_stageable);
            assert_eq!(hunk.stage_status, HunkStageStatus::Unstaged);
        }
    }

    #[test]
    fn test_empty_diff() {
        let original = "line1\nline2";
        let modified = "line1\nline2";

        let diff = compute_diff(original, modified);

        assert_eq!(diff.hunks.len(), 0);
    }

    #[test]
    fn test_diff_action_state() {
        let mut state = DiffActionState::new(PathBuf::from("test.txt"));
        assert!(state.hunks.is_empty());
        assert!(state.keyboard_shortcuts_enabled);

        // Test updating from diff
        let original = "line1\nline2";
        let modified = "line1\nline2 modified";
        let diff = compute_diff(original, modified);

        state.update_from_diff(&diff);
        assert_eq!(state.hunks.len(), diff.hunks.len());
        assert_eq!(
            state.lines.len(),
            diff.hunks.iter().map(|h| h.lines.len()).sum::<usize>()
        );
    }
}
