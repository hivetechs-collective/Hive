//! Git branch management
//! 
//! Types and operations for git branches

use git2::{Branch, BranchType as Git2BranchType};
use chrono::{DateTime, Utc};

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