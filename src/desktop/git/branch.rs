//! Git branch management
//!
//! Types and operations for git branches

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Branch, BranchType as Git2BranchType};

/// Information about a git branch
#[derive(Debug, Clone, PartialEq)]
pub struct BranchInfo {
    pub name: String,
    pub branch_type: BranchType,
    pub is_current: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub last_commit: Option<CommitInfo>,
}

/// Type of git branch
#[derive(Debug, Clone, PartialEq)]
pub enum BranchType {
    Local,
    Remote,
    Tag,
}

impl From<Git2BranchType> for BranchType {
    fn from(branch_type: Git2BranchType) -> Self {
        match branch_type {
            Git2BranchType::Local => BranchType::Local,
            Git2BranchType::Remote => BranchType::Remote,
        }
    }
}

/// Basic commit information
#[derive(Debug, Clone, PartialEq)]
pub struct CommitInfo {
    pub hash: String,
    pub hash_short: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
}

impl Default for BranchInfo {
    fn default() -> Self {
        Self {
            name: "main".to_string(),
            branch_type: BranchType::Local,
            is_current: true,
            upstream: None,
            ahead: 0,
            behind: 0,
            last_commit: None,
        }
    }
}

/// Branch filtering options
#[derive(Debug, Clone, PartialEq)]
pub struct BranchFilter {
    pub search_term: String,
    pub show_local: bool,
    pub show_remote: bool,
    pub show_tags: bool,
    pub sort_by: BranchSort,
}

impl Default for BranchFilter {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            show_local: true,
            show_remote: true,
            show_tags: false,
            sort_by: BranchSort::Name,
        }
    }
}

/// Branch sorting options
#[derive(Debug, Clone, PartialEq)]
pub enum BranchSort {
    Name,
    LastCommit,
    Ahead,
    Behind,
}

impl BranchInfo {
    /// Check if branch matches filter
    pub fn matches_filter(&self, filter: &BranchFilter) -> bool {
        // Check type filter
        let type_match = match self.branch_type {
            BranchType::Local => filter.show_local,
            BranchType::Remote => filter.show_remote,
            BranchType::Tag => filter.show_tags,
        };

        if !type_match {
            return false;
        }

        // Check search term
        if !filter.search_term.is_empty() {
            let search_lower = filter.search_term.to_lowercase();
            let name_lower = self.name.to_lowercase();

            // Search in branch name and upstream name
            let name_match = name_lower.contains(&search_lower);
            let upstream_match = self
                .upstream
                .as_ref()
                .map(|u| u.to_lowercase().contains(&search_lower))
                .unwrap_or(false);

            // Search in commit message
            let commit_match = self
                .last_commit
                .as_ref()
                .map(|c| c.message.to_lowercase().contains(&search_lower))
                .unwrap_or(false);

            if !(name_match || upstream_match || commit_match) {
                return false;
            }
        }

        true
    }
}

/// Sort branches according to the specified criteria
pub fn sort_branches(branches: &mut Vec<BranchInfo>, sort_by: &BranchSort) {
    branches.sort_by(|a, b| {
        // Always put current branch first
        if a.is_current && !b.is_current {
            return std::cmp::Ordering::Less;
        }
        if !a.is_current && b.is_current {
            return std::cmp::Ordering::Greater;
        }

        match sort_by {
            BranchSort::Name => a.name.cmp(&b.name),
            BranchSort::LastCommit => match (&a.last_commit, &b.last_commit) {
                (Some(commit_a), Some(commit_b)) => commit_b.date.cmp(&commit_a.date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            },
            BranchSort::Ahead => b.ahead.cmp(&a.ahead),
            BranchSort::Behind => b.behind.cmp(&a.behind),
        }
    });
}

/// Validate branch name according to git rules
pub fn validate_branch_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow::anyhow!("Branch name cannot be empty"));
    }

    // Check for forbidden characters
    if name.contains("..") || name.contains("~") || name.contains("^") || name.contains(":") {
        return Err(anyhow::anyhow!("Branch name contains forbidden characters"));
    }

    // Check for leading/trailing special characters
    if name.starts_with('-') || name.ends_with('.') || name.ends_with(".lock") {
        return Err(anyhow::anyhow!("Branch name has invalid format"));
    }

    // Check for reserved names
    let reserved = ["HEAD", "FETCH_HEAD", "ORIG_HEAD", "MERGE_HEAD"];
    if reserved.contains(&name) {
        return Err(anyhow::anyhow!("Branch name '{}' is reserved", name));
    }

    Ok(())
}
