//! Git Integration for showing changes in the editor
//!
//! Provides diff decorations and inline change indicators

use git2::{DiffOptions, Repository};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GitChange {
    pub line_start: usize,
    pub line_end: usize,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Addition,
    Deletion,
    Modification,
}

pub struct GitIntegration {
    repo: Option<Repository>,
}

impl GitIntegration {
    pub fn new(workspace_path: &Path) -> Self {
        let repo = Repository::discover(workspace_path).ok();
        Self { repo }
    }

    /// Get line-level changes for a file
    pub fn get_file_changes(&self, file_path: &Path) -> Vec<GitChange> {
        let mut changes = Vec::new();

        if let Some(repo) = &self.repo {
            if let Ok(head) = repo.head() {
                if let Ok(tree) = head.peel_to_tree() {
                    // Get the current index
                    if let Ok(index) = repo.index() {
                        if let Ok(diff) = repo.diff_tree_to_index(Some(&tree), Some(&index), None) {
                            let mut additions = HashSet::new();
                            let mut deletions = HashSet::new();
                            let modifications = HashSet::<usize>::new();

                            // Process diff
                            let _ = diff.foreach(
                                &mut |_, _| true,
                                None,
                                None,
                                Some(&mut |_, _, line| {
                                    match line.origin() {
                                        '+' => {
                                            if let Some(new_lineno) = line.new_lineno() {
                                                additions.insert(new_lineno as usize);
                                            }
                                        }
                                        '-' => {
                                            if let Some(old_lineno) = line.old_lineno() {
                                                deletions.insert(old_lineno as usize);
                                            }
                                        }
                                        _ => {}
                                    }
                                    true
                                }),
                            );

                            // Convert to changes
                            for &line in &additions {
                                changes.push(GitChange {
                                    line_start: line,
                                    line_end: line,
                                    change_type: ChangeType::Addition,
                                });
                            }

                            for &line in &deletions {
                                changes.push(GitChange {
                                    line_start: line,
                                    line_end: line,
                                    change_type: ChangeType::Deletion,
                                });
                            }
                        }
                    }
                }
            }
        }

        changes
    }

    /// Get inline decorations for added/removed text
    pub fn get_inline_decorations(&self, file_path: &Path, content: &str) -> Vec<InlineDecoration> {
        let mut decorations = Vec::new();

        if let Some(repo) = &self.repo {
            // Get the file's original content from HEAD
            if let Ok(head) = repo.head() {
                if let Ok(tree) = head.peel_to_tree() {
                    if let Ok(entry) = tree.get_path(file_path) {
                        if let Ok(blob) = entry.to_object(&repo).and_then(|obj| obj.peel_to_blob())
                        {
                            let original_content = String::from_utf8_lossy(blob.content());

                            // Perform line-by-line diff
                            let original_lines: Vec<&str> = original_content.lines().collect();
                            let current_lines: Vec<&str> = content.lines().collect();

                            // Use a simple diff algorithm
                            decorations = self.compute_inline_diff(&original_lines, &current_lines);
                        }
                    }
                }
            }
        }

        decorations
    }

    fn compute_inline_diff(&self, original: &[&str], current: &[&str]) -> Vec<InlineDecoration> {
        let mut decorations = Vec::new();

        // Simple line-by-line comparison
        let max_len = original.len().max(current.len());

        for i in 0..max_len {
            let original_line = original.get(i).copied().unwrap_or("");
            let current_line = current.get(i).copied().unwrap_or("");

            if original_line != current_line {
                if original_line.is_empty() {
                    // Line was added
                    decorations.push(InlineDecoration {
                        line: i,
                        start_col: 0,
                        end_col: current_line.len(),
                        decoration_type: InlineDecorationType::Addition,
                    });
                } else if current_line.is_empty() {
                    // Line was deleted
                    decorations.push(InlineDecoration {
                        line: i,
                        start_col: 0,
                        end_col: 0,
                        decoration_type: InlineDecorationType::Deletion,
                    });
                } else {
                    // Line was modified - find the changed parts
                    let changes = self.find_line_changes(original_line, current_line);
                    for (start, end, change_type) in changes {
                        decorations.push(InlineDecoration {
                            line: i,
                            start_col: start,
                            end_col: end,
                            decoration_type: change_type,
                        });
                    }
                }
            }
        }

        decorations
    }

    fn find_line_changes(
        &self,
        original: &str,
        current: &str,
    ) -> Vec<(usize, usize, InlineDecorationType)> {
        let mut changes = Vec::new();

        // Simple character-by-character comparison
        let original_chars: Vec<char> = original.chars().collect();
        let current_chars: Vec<char> = current.chars().collect();

        let mut i = 0;
        let mut j = 0;

        while i < original_chars.len() || j < current_chars.len() {
            if i >= original_chars.len() {
                // Rest of current is addition
                changes.push((j, current_chars.len(), InlineDecorationType::Addition));
                break;
            } else if j >= current_chars.len() {
                // Rest of original is deletion
                changes.push((j, j, InlineDecorationType::Deletion));
                break;
            } else if original_chars[i] != current_chars[j] {
                // Find the extent of the change
                let start = j;
                while j < current_chars.len()
                    && i < original_chars.len()
                    && original_chars[i] != current_chars[j]
                {
                    i += 1;
                    j += 1;
                }
                changes.push((start, j, InlineDecorationType::Modification));
            } else {
                i += 1;
                j += 1;
            }
        }

        changes
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineDecoration {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub decoration_type: InlineDecorationType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InlineDecorationType {
    Addition,
    Deletion,
    Modification,
}

impl InlineDecorationType {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Addition => "git-addition",
            Self::Deletion => "git-deletion",
            Self::Modification => "git-modification",
        }
    }

    pub fn background_color(&self) -> &'static str {
        match self {
            Self::Addition => "rgba(76, 175, 80, 0.2)",      // Green
            Self::Deletion => "rgba(244, 67, 54, 0.2)",      // Red
            Self::Modification => "rgba(33, 150, 243, 0.2)", // Blue
        }
    }
}
