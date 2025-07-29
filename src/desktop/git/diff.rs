//! Git diff functionality
//! 
//! Implements diff algorithms and data structures for showing file changes

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

/// Result of comparing two files
#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub original_lines: Vec<String>,
    pub modified_lines: Vec<String>,
    pub hunks: Vec<DiffHunk>,
}

/// A contiguous section of changes
#[derive(Debug, Clone, PartialEq)]
pub struct DiffHunk {
    pub original_start: usize,
    pub original_count: usize,
    pub modified_start: usize,
    pub modified_count: usize,
    pub lines: Vec<DiffLine>,
}

/// Type of change for a line
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

/// A single line in a diff
#[derive(Debug, Clone, PartialEq)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub original_line_number: Option<usize>,
    pub modified_line_number: Option<usize>,
    pub content: String,
}

/// View mode for displaying diffs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffViewMode {
    SideBySide,
    Inline,
}

/// Compute diff between two texts using Myers' algorithm
pub fn compute_diff(original: &str, modified: &str) -> DiffResult {
    let original_lines: Vec<String> = original.lines().map(|s| s.to_string()).collect();
    let modified_lines: Vec<String> = modified.lines().map(|s| s.to_string()).collect();
    
    // For now, use git2's diff functionality
    // TODO: Implement Myers' O(ND) algorithm for better performance
    
    let mut hunks = Vec::new();
    let mut current_hunk: Option<DiffHunk> = None;
    let mut hunk_lines = Vec::new();
    
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
                    current_hunk = Some(DiffHunk {
                        original_start: i + 1,
                        original_count: 0,
                        modified_start: i + 1,
                        modified_count: 0,
                        lines: Vec::new(),
                    });
                }
                
                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Deleted,
                    original_line_number: Some(i + 1),
                    modified_line_number: None,
                    content: orig.clone(),
                });
                
                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    original_line_number: None,
                    modified_line_number: Some(i + 1),
                    content: modif.clone(),
                });
                
                if let Some(ref mut hunk) = current_hunk {
                    hunk.original_count += 1;
                    hunk.modified_count += 1;
                }
            }
            (Some(orig), None) => {
                // Deleted line
                if current_hunk.is_none() {
                    current_hunk = Some(DiffHunk {
                        original_start: i + 1,
                        original_count: 0,
                        modified_start: i + 1,
                        modified_count: 0,
                        lines: Vec::new(),
                    });
                }
                
                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Deleted,
                    original_line_number: Some(i + 1),
                    modified_line_number: None,
                    content: orig.clone(),
                });
                
                if let Some(ref mut hunk) = current_hunk {
                    hunk.original_count += 1;
                }
            }
            (None, Some(modif)) => {
                // Added line
                if current_hunk.is_none() {
                    current_hunk = Some(DiffHunk {
                        original_start: original_lines.len() + 1,
                        original_count: 0,
                        modified_start: i + 1,
                        modified_count: 0,
                        lines: Vec::new(),
                    });
                }
                
                hunk_lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    original_line_number: None,
                    modified_line_number: Some(i + 1),
                    content: modif.clone(),
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

/// Get the diff between a file's working tree version and its HEAD version
pub async fn get_file_diff(repo_path: &PathBuf, file_path: &PathBuf) -> Result<DiffResult> {
    use crate::desktop::git::GitRepository;
    use tokio::fs;
    
    // Get the repository
    let repo = GitRepository::open(repo_path)?;
    
    // Read current file content
    let current_content = fs::read_to_string(file_path).await?;
    
    // Get relative path for git
    let relative_path = file_path.strip_prefix(repo_path)
        .context("File is not in repository")?;
    
    // Get HEAD version of the file
    let head_content = match repo.get_file_at_head(relative_path) {
        Ok(content) => content,
        Err(_) => {
            // File might be new (not in HEAD), treat as empty
            String::new()
        }
    };
    
    Ok(compute_diff(&head_content, &current_content))
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
    }
    
    #[test]
    fn test_empty_diff() {
        let original = "line1\nline2";
        let modified = "line1\nline2";
        
        let diff = compute_diff(original, modified);
        
        assert_eq!(diff.hunks.len(), 0);
    }
}