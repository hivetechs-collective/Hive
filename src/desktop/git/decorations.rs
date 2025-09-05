//! Git Decorations System
//!
//! VS Code-style git decorations with real-time updates
//! Provides comprehensive visual indicators for git status in file explorer

use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::desktop::state::GitFileStatus;
use serde::{Deserialize, Serialize};

/// Configuration for git decorations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitDecorationConfig {
    /// Enable/disable git decorations
    pub enabled: bool,
    /// Show status indicators (M, A, D, etc.)
    pub show_status_letters: bool,
    /// Show color coding
    pub show_colors: bool,
    /// Show folder decorations for child status
    pub show_folder_decorations: bool,
    /// Show gutter decorations in editor
    pub show_gutter_decorations: bool,
    /// Gray out ignored files
    pub gray_out_ignored: bool,
    /// Show conflict markers
    pub show_conflict_markers: bool,
    /// Decoration styles
    pub styles: DecorationStyles,
}

impl Default for GitDecorationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_status_letters: true,
            show_colors: true,
            show_folder_decorations: true,
            show_gutter_decorations: true,
            gray_out_ignored: true,
            show_conflict_markers: true,
            styles: DecorationStyles::default(),
        }
    }
}

/// Decoration style configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecorationStyles {
    /// Colors for different git statuses
    pub modified_color: String,
    pub added_color: String,
    pub deleted_color: String,
    pub untracked_color: String,
    pub ignored_color: String,
    pub conflict_color: String,
    pub renamed_color: String,
    pub copied_color: String,
    /// Decoration opacity
    pub opacity: f32,
    /// Font weight for status letters
    pub status_font_weight: String,
}

impl Default for DecorationStyles {
    fn default() -> Self {
        Self {
            modified_color: "#e2c08d".to_string(),      // VS Code orange/yellow
            added_color: "#73c991".to_string(),         // VS Code green
            deleted_color: "#f48771".to_string(),       // VS Code red
            untracked_color: "#73c991".to_string(),     // VS Code green
            ignored_color: "#6b6b6b".to_string(),       // VS Code gray
            conflict_color: "#f44747".to_string(),      // VS Code bright red
            renamed_color: "#75beff".to_string(),       // VS Code blue
            copied_color: "#73c991".to_string(),        // VS Code green
            opacity: 0.8,
            status_font_weight: "600".to_string(),
        }
    }
}

/// Comprehensive git status with detailed information
#[derive(Debug, Clone, PartialEq)]
pub struct EnhancedGitStatus {
    /// Basic git status
    pub status: GitFileStatus,
    /// Whether file has staged changes
    pub has_staged: bool,
    /// Whether file has unstaged changes
    pub has_unstaged: bool,
    /// Whether file is in conflict state
    pub is_conflicted: bool,
    /// Number of lines added
    pub lines_added: Option<usize>,
    /// Number of lines deleted
    pub lines_deleted: Option<usize>,
    /// Whether file is executable
    pub is_executable: bool,
    /// Whether file is symlink
    pub is_symlink: bool,
}

impl EnhancedGitStatus {
    /// Create new enhanced git status
    pub fn new(status: GitFileStatus) -> Self {
        Self {
            status,
            has_staged: false,
            has_unstaged: false,
            is_conflicted: false,
            lines_added: None,
            lines_deleted: None,
            is_executable: false,
            is_symlink: false,
        }
    }

    /// Get the primary decoration color
    pub fn get_color<'a>(&self, styles: &'a DecorationStyles) -> &'a str {
        if self.is_conflicted {
            return &styles.conflict_color;
        }

        match self.status {
            GitFileStatus::Modified => &styles.modified_color,
            GitFileStatus::Added => &styles.added_color,
            GitFileStatus::Deleted => &styles.deleted_color,
            GitFileStatus::Untracked => &styles.untracked_color,
            GitFileStatus::Renamed => &styles.renamed_color,
            GitFileStatus::Copied => &styles.copied_color,
            GitFileStatus::Ignored => &styles.ignored_color,
        }
    }

    /// Get the status letter
    pub fn get_status_letter(&self) -> &'static str {
        if self.is_conflicted {
            return "!";
        }

        match self.status {
            GitFileStatus::Modified => {
                match (self.has_staged, self.has_unstaged) {
                    (true, true) => "M", // Both staged and unstaged
                    (true, false) => "M", // Only staged
                    (false, true) => "M", // Only unstaged
                    (false, false) => "",
                }
            },
            GitFileStatus::Added => "A",
            GitFileStatus::Deleted => "D",
            GitFileStatus::Untracked => "U",
            GitFileStatus::Renamed => "R",
            GitFileStatus::Copied => "C",
            GitFileStatus::Ignored => "",
        }
    }

    /// Get tooltip text for status
    pub fn get_tooltip(&self) -> String {
        if self.is_conflicted {
            return "File has merge conflicts".to_string();
        }

        let base_status = match self.status {
            GitFileStatus::Modified => "Modified",
            GitFileStatus::Added => "Added",
            GitFileStatus::Deleted => "Deleted",
            GitFileStatus::Untracked => "Untracked",
            GitFileStatus::Renamed => "Renamed",
            GitFileStatus::Copied => "Copied",
            GitFileStatus::Ignored => "Ignored",
        };

        let mut tooltip = base_status.to_string();

        if let (Some(added), Some(deleted)) = (self.lines_added, self.lines_deleted) {
            tooltip.push_str(&format!(" (+{}, -{})", added, deleted));
        }

        if self.has_staged && self.has_unstaged {
            tooltip.push_str(" (staged & unstaged changes)");
        } else if self.has_staged {
            tooltip.push_str(" (staged)");
        } else if self.has_unstaged {
            tooltip.push_str(" (unstaged)");
        }

        tooltip
    }
}

/// Folder decoration information
#[derive(Debug, Clone)]
pub struct FolderDecoration {
    /// Has any modified files in subtree
    pub has_modified: bool,
    /// Has any added files in subtree
    pub has_added: bool,
    /// Has any deleted files in subtree
    pub has_deleted: bool,
    /// Has any untracked files in subtree
    pub has_untracked: bool,
    /// Has any conflicted files in subtree
    pub has_conflicts: bool,
    /// Total number of changed files in subtree
    pub total_changes: usize,
}

impl FolderDecoration {
    /// Create empty folder decoration
    pub fn new() -> Self {
        Self {
            has_modified: false,
            has_added: false,
            has_deleted: false,
            has_untracked: false,
            has_conflicts: false,
            total_changes: 0,
        }
    }

    /// Add file status to folder decoration
    pub fn add_file_status(&mut self, status: &EnhancedGitStatus) {
        if status.is_conflicted {
            self.has_conflicts = true;
        }

        match status.status {
            GitFileStatus::Modified => self.has_modified = true,
            GitFileStatus::Added => self.has_added = true,
            GitFileStatus::Deleted => self.has_deleted = true,
            GitFileStatus::Untracked => self.has_untracked = true,
            GitFileStatus::Renamed => self.has_modified = true,
            GitFileStatus::Copied => self.has_added = true,
            GitFileStatus::Ignored => {}, // Don't count ignored files
        }

        if status.status != GitFileStatus::Ignored {
            self.total_changes += 1;
        }
    }

    /// Get primary decoration color for folder
    pub fn get_folder_color<'a>(&self, styles: &'a DecorationStyles) -> Option<&'a str> {
        if self.has_conflicts {
            Some(&styles.conflict_color)
        } else if self.has_deleted {
            Some(&styles.deleted_color)
        } else if self.has_modified {
            Some(&styles.modified_color)
        } else if self.has_added {
            Some(&styles.added_color)
        } else if self.has_untracked {
            Some(&styles.untracked_color)
        } else {
            None
        }
    }

    /// Get folder decoration indicator
    pub fn get_folder_indicator(&self) -> Option<&'static str> {
        if self.has_conflicts {
            Some("!")
        } else if self.total_changes > 0 {
            Some("●") // Dot indicator for changes
        } else {
            None
        }
    }
}

/// Git decorations manager
#[derive(Clone)]
pub struct GitDecorationManager {
    /// Configuration
    pub config: Signal<GitDecorationConfig>,
    /// File status cache
    pub file_statuses: Signal<HashMap<PathBuf, EnhancedGitStatus>>,
    /// Folder decorations cache
    pub folder_decorations: Signal<HashMap<PathBuf, FolderDecoration>>,
    /// Repository root path
    pub repo_root: Signal<Option<PathBuf>>,
}

impl GitDecorationManager {
    /// Create new decoration manager
    pub fn new() -> Self {
        Self {
            config: Signal::new(GitDecorationConfig::default()),
            file_statuses: Signal::new(HashMap::new()),
            folder_decorations: Signal::new(HashMap::new()),
            repo_root: Signal::new(None),
        }
    }

    /// Update file statuses from git
    pub fn update_file_statuses(&mut self, statuses: HashMap<PathBuf, EnhancedGitStatus>) {
        self.file_statuses.set(statuses.clone());
        
        // Update folder decorations
        self.update_folder_decorations(&statuses);
    }

    /// Update folder decorations based on file statuses
    fn update_folder_decorations(&mut self, file_statuses: &HashMap<PathBuf, EnhancedGitStatus>) {
        let mut folder_decorations = HashMap::new();

        // Calculate folder decorations from file statuses
        for (file_path, status) in file_statuses {
            if let Some(parent) = file_path.parent() {
                let mut current_path = parent.to_path_buf();
                
                // Propagate status up the directory tree
                loop {
                    let decoration = folder_decorations
                        .entry(current_path.clone())
                        .or_insert_with(FolderDecoration::new);
                    
                    decoration.add_file_status(status);

                    if let Some(parent) = current_path.parent() {
                        if let Some(repo_root) = self.repo_root.read().as_ref() {
                            if parent < repo_root {
                                break;
                            }
                        }
                        current_path = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
            }
        }

        self.folder_decorations.set(folder_decorations);
    }

    /// Get decoration for file
    pub fn get_file_decoration(&self, path: &Path) -> Option<EnhancedGitStatus> {
        self.file_statuses.read().get(path).cloned()
    }

    /// Get decoration for folder
    pub fn get_folder_decoration(&self, path: &Path) -> Option<FolderDecoration> {
        self.folder_decorations.read().get(path).cloned()
    }

    /// Check if decorations are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.read().enabled
    }

    /// Update configuration
    pub fn update_config(&mut self, config: GitDecorationConfig) {
        self.config.set(config);
    }
}

/// Component for file git decoration
#[component]
pub fn FileGitDecoration(
    file_path: PathBuf,
    is_directory: bool,
    decoration_manager: GitDecorationManager,
) -> Element {
    let config = decoration_manager.config.read();
    
    if !config.enabled {
        return rsx! { span {} };
    }

    if is_directory {
        // Folder decoration
        if let Some(folder_decoration) = decoration_manager.get_folder_decoration(&file_path) {
            if let Some(color) = folder_decoration.get_folder_color(&config.styles) {
                if let Some(indicator) = folder_decoration.get_folder_indicator() {
                    return rsx! {
                        span {
                            class: "folder-git-decoration",
                            style: "color: {color}; opacity: {config.styles.opacity}; font-size: 10px; margin-left: 4px;",
                            title: "Folder contains {folder_decoration.total_changes} changed files",
                            "{indicator}"
                        }
                    };
                }
            }
        }
    } else {
        // File decoration
        if let Some(file_status) = decoration_manager.get_file_decoration(&file_path) {
            let color = file_status.get_color(&config.styles);
            let letter = file_status.get_status_letter();
            let tooltip = file_status.get_tooltip();

            if !letter.is_empty() && config.show_status_letters {
                return rsx! {
                    span {
                        class: "file-git-decoration",
                        style: "color: {color}; opacity: {config.styles.opacity}; font-size: 11px; font-weight: {config.styles.status_font_weight}; margin-left: 4px;",
                        title: "{tooltip}",
                        "{letter}"
                    }
                };
            }
        }
    }

    rsx! { span {} }
}

/// Component for gutter decorations in editor
#[component]
pub fn EditorGutterDecorations(
    file_path: PathBuf,
    line_number: usize,
    decoration_manager: GitDecorationManager,
) -> Element {
    let config = decoration_manager.config.read();
    
    if !config.enabled || !config.show_gutter_decorations {
        return rsx! { span {} };
    }

    // This would need integration with the editor's line tracking system
    // For now, just render a placeholder
    rsx! { 
        span {
            class: "gutter-decoration",
            style: "width: 4px; height: 100%; position: absolute; left: 0;",
            // Actual implementation would show line-level changes
        }
    }
}

/// Hook to use git decoration manager
pub fn use_git_decoration_manager() -> GitDecorationManager {
    use_context_provider(|| GitDecorationManager::new())
}

/// Convert basic GitFileStatus to EnhancedGitStatus
impl From<GitFileStatus> for EnhancedGitStatus {
    fn from(status: GitFileStatus) -> Self {
        EnhancedGitStatus::new(status)
    }
}

impl PartialEq for GitDecorationManager {
    fn eq(&self, other: &Self) -> bool {
        // Compare configuration values by dereferencing the GenerationalRef
        *self.config.read() == *other.config.read() &&
        // For simplicity, we'll consider managers equal if configs are equal
        // In practice, you might want more sophisticated comparison
        *self.repo_root.read() == *other.repo_root.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_git_status() {
        let status = EnhancedGitStatus::new(GitFileStatus::Modified);
        assert_eq!(status.get_status_letter(), "M");
        assert_eq!(status.get_tooltip(), "Modified");
    }

    #[test]
    fn test_folder_decoration() {
        let mut folder = FolderDecoration::new();
        let modified_status = EnhancedGitStatus::new(GitFileStatus::Modified);
        folder.add_file_status(&modified_status);
        
        assert!(folder.has_modified);
        assert_eq!(folder.total_changes, 1);
    }

    #[test]
    fn test_decoration_config() {
        let config = GitDecorationConfig::default();
        assert!(config.enabled);
        assert!(config.show_status_letters);
        assert!(config.show_colors);
    }
}