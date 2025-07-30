//! Event Types and Payloads
//!
//! Defines the event types and their associated data structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Event types that can be published through the event bus
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Git repository status has changed
    GitStatusChanged,
    
    /// Build process has started
    BuildStarted,
    
    /// Build process has completed
    BuildCompleted,
    
    /// A file has been changed (created, modified, deleted)
    FileChanged,
    
    /// The active repository has changed
    RepositoryChanged,
    
    /// Problems/diagnostics have been updated
    ProblemsUpdated,
    
    /// Terminal output has been received
    TerminalOutput,
    
    /// Consensus process state change
    ConsensusStateChanged,
    
    /// UI state change (theme, layout, etc.)
    UIStateChanged,
    
    /// Menu visibility has changed (opened/closed)
    MenuVisibilityChanged,
    
    /// Configuration has been updated
    ConfigurationChanged,
    
    /// Extension/plugin event
    ExtensionEvent,
    
    /// Branch menu has been requested (clicked in status bar)
    BranchMenuRequested,
}

/// Event payload containing data specific to each event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    /// Empty payload for events that don't need data
    Empty,
    
    /// File path for file-related events
    FilePath(PathBuf),
    
    /// Git status information
    GitStatus {
        branch: String,
        modified_files: Vec<String>,
        staged_files: Vec<String>,
    },
    
    /// Build information
    BuildInfo {
        target: String,
        profile: String,
    },
    
    /// Build result
    BuildResult {
        success: bool,
        errors: Vec<String>,
        warnings: Vec<String>,
        duration_ms: u64,
    },
    
    /// File change details
    FileChange {
        path: PathBuf,
        change_type: FileChangeType,
    },
    
    /// Repository information
    RepositoryInfo {
        path: PathBuf,
        name: String,
        remote_url: Option<String>,
    },
    
    /// Problems/diagnostics update
    ProblemsUpdate {
        added: Vec<Problem>,
        removed: Vec<Problem>,
        total_count: usize,
    },
    
    /// Terminal output
    TerminalData {
        terminal_id: String,
        data: String,
    },
    
    /// Consensus state
    ConsensusState {
        stage: ConsensusStage,
        progress: u8,
        message: Option<String>,
    },
    
    /// UI state change
    UIChange {
        component: String,
        property: String,
        value: serde_json::Value,
    },
    
    /// Menu visibility changed payload
    MenuVisibility {
        menu_id: String,
        visible: bool,
    },
    
    /// Configuration change
    ConfigChange {
        section: String,
        key: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    },
    
    /// Extension event data
    ExtensionData {
        extension_id: String,
        event_name: String,
        data: serde_json::Value,
    },
}

/// Type of file change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { old_path: PathBuf },
}

/// Problem/diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub severity: ProblemSeverity,
    pub source: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub code: Option<String>,
}

/// Problem severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProblemSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Consensus pipeline stages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusStage {
    Initializing,
    Generating,
    Refining,
    Validating,
    Curating,
    Completed,
    Failed,
}

/// Event structure that gets published through the event bus
#[derive(Debug, Clone)]
pub struct Event {
    /// The type of event
    pub event_type: EventType,
    
    /// The payload associated with this event
    pub payload: EventPayload,
}

impl Event {
    /// Create a new event with the given type and payload
    pub fn new(event_type: EventType, payload: EventPayload) -> Self {
        Self { event_type, payload }
    }
    
    /// Create an event with an empty payload
    pub fn empty(event_type: EventType) -> Self {
        Self {
            event_type,
            payload: EventPayload::Empty,
        }
    }
    
    /// Create a file changed event
    pub fn file_changed(path: PathBuf, change_type: FileChangeType) -> Self {
        Self {
            event_type: EventType::FileChanged,
            payload: EventPayload::FileChange { path, change_type },
        }
    }
    
    /// Create a git status changed event
    pub fn git_status_changed(branch: String, modified_files: Vec<String>, staged_files: Vec<String>) -> Self {
        Self {
            event_type: EventType::GitStatusChanged,
            payload: EventPayload::GitStatus {
                branch,
                modified_files,
                staged_files,
            },
        }
    }
    
    /// Create a build started event
    pub fn build_started(target: String, profile: String) -> Self {
        Self {
            event_type: EventType::BuildStarted,
            payload: EventPayload::BuildInfo { target, profile },
        }
    }
    
    /// Create a build completed event
    pub fn build_completed(success: bool, errors: Vec<String>, warnings: Vec<String>, duration_ms: u64) -> Self {
        Self {
            event_type: EventType::BuildCompleted,
            payload: EventPayload::BuildResult {
                success,
                errors,
                warnings,
                duration_ms,
            },
        }
    }
    
    /// Create a repository changed event
    pub fn repository_changed(path: PathBuf, name: String, remote_url: Option<String>) -> Self {
        Self {
            event_type: EventType::RepositoryChanged,
            payload: EventPayload::RepositoryInfo {
                path,
                name,
                remote_url,
            },
        }
    }
    
    /// Create a problems updated event
    pub fn problems_updated(added: Vec<Problem>, removed: Vec<Problem>, total_count: usize) -> Self {
        Self {
            event_type: EventType::ProblemsUpdated,
            payload: EventPayload::ProblemsUpdate {
                added,
                removed,
                total_count,
            },
        }
    }
    
    /// Create a terminal output event
    pub fn terminal_output(terminal_id: String, data: String) -> Self {
        Self {
            event_type: EventType::TerminalOutput,
            payload: EventPayload::TerminalData { terminal_id, data },
        }
    }
    
    /// Create a consensus state changed event
    pub fn consensus_state_changed(stage: ConsensusStage, progress: u8, message: Option<String>) -> Self {
        Self {
            event_type: EventType::ConsensusStateChanged,
            payload: EventPayload::ConsensusState {
                stage,
                progress,
                message,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_creation() {
        let event = Event::file_changed(
            PathBuf::from("/src/main.rs"),
            FileChangeType::Modified,
        );
        
        assert_eq!(event.event_type, EventType::FileChanged);
        match event.payload {
            EventPayload::FileChange { path, change_type } => {
                assert_eq!(path, PathBuf::from("/src/main.rs"));
                assert_eq!(change_type, FileChangeType::Modified);
            }
            _ => panic!("Wrong payload type"),
        }
    }
    
    #[test]
    fn test_empty_event() {
        let event = Event::empty(EventType::ConfigurationChanged);
        assert_eq!(event.event_type, EventType::ConfigurationChanged);
        assert!(matches!(event.payload, EventPayload::Empty));
    }
    
    #[test]
    fn test_git_status_event() {
        let event = Event::git_status_changed(
            "main".to_string(),
            vec!["src/main.rs".to_string()],
            vec!["Cargo.toml".to_string()],
        );
        
        assert_eq!(event.event_type, EventType::GitStatusChanged);
        match event.payload {
            EventPayload::GitStatus { branch, modified_files, staged_files } => {
                assert_eq!(branch, "main");
                assert_eq!(modified_files, vec!["src/main.rs"]);
                assert_eq!(staged_files, vec!["Cargo.toml"]);
            }
            _ => panic!("Wrong payload type"),
        }
    }
}