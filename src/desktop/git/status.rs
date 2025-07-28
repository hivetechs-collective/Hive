//! Git file status tracking
//! 
//! Types and operations for tracking file changes

use git2::{Status, StatusShow};
use std::path::PathBuf;

/// File status in git
#[derive(Debug, Clone, PartialEq)]
pub struct FileStatus {
    pub path: PathBuf,
    pub status_type: StatusType,
    pub is_staged: bool,
}

/// Type of file status
#[derive(Debug, Clone, PartialEq)]
pub enum StatusType {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
}

impl From<Status> for StatusType {
    fn from(status: Status) -> Self {
        if status.contains(Status::CONFLICTED) {
            StatusType::Conflicted
        } else if status.contains(Status::WT_NEW) || status.contains(Status::INDEX_NEW) {
            StatusType::Added
        } else if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED) {
            StatusType::Deleted
        } else if status.contains(Status::WT_RENAMED) || status.contains(Status::INDEX_RENAMED) {
            StatusType::Renamed
        } else if status.contains(Status::WT_MODIFIED) || status.contains(Status::INDEX_MODIFIED) {
            StatusType::Modified
        } else if status.contains(Status::IGNORED) {
            StatusType::Ignored
        } else {
            StatusType::Untracked
        }
    }
}

impl StatusType {
    /// Get the icon for this status type (VS Code style)
    pub fn icon(&self) -> &'static str {
        match self {
            StatusType::Modified => "M",
            StatusType::Added => "A",
            StatusType::Deleted => "D",
            StatusType::Renamed => "R",
            StatusType::Copied => "C",
            StatusType::Untracked => "U",
            StatusType::Ignored => "!",
            StatusType::Conflicted => "⚠",
        }
    }
    
    /// Get the color for this status type
    pub fn color(&self) -> &'static str {
        match self {
            StatusType::Modified => "#e2c08d", // Yellow
            StatusType::Added => "#73c991",    // Green
            StatusType::Deleted => "#f44747",  // Red
            StatusType::Renamed => "#c586c0",  // Purple
            StatusType::Copied => "#c586c0",   // Purple
            StatusType::Untracked => "#73c991", // Green
            StatusType::Ignored => "#6b737c",  // Gray
            StatusType::Conflicted => "#f44747", // Red
        }
    }
}

/// Sync status with upstream
#[derive(Debug, Clone, PartialEq)]
pub struct SyncStatus {
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
    pub is_syncing: bool,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            ahead: 0,
            behind: 0,
            has_upstream: false,
            is_syncing: false,
        }
    }
}

impl SyncStatus {
    /// Format sync status for display (VS Code style)
    pub fn format(&self) -> String {
        if !self.has_upstream {
            return String::new();
        }
        
        if self.is_syncing {
            return "⟳".to_string();
        }
        
        format!("↓{} ↑{}", self.behind, self.ahead)
    }
}