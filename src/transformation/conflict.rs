//! Conflict detection and resolution for code transformations

use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use crate::transformation::types::*;

/// Handles conflict detection and resolution for transformations
pub struct ConflictResolver {
    pending_transformations: HashMap<String, Transformation>,
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            pending_transformations: HashMap::new(),
        }
    }

    /// Check for conflicts in a set of changes
    pub fn check_conflicts(&self, changes: &[CodeChange]) -> Result<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();

        // Group changes by file
        let mut changes_by_file: HashMap<PathBuf, Vec<&CodeChange>> = HashMap::new();
        for change in changes {
            changes_by_file
                .entry(change.file_path.clone())
                .or_insert_with(Vec::new)
                .push(change);
        }

        // Check for overlapping changes within same file
        for (file_path, file_changes) in changes_by_file {
            if let Some(overlap_conflicts) = self.check_overlapping_changes(&file_changes) {
                for conflict in overlap_conflicts {
                    conflicts.push(ConflictInfo {
                        file_path: file_path.clone(),
                        conflict_type: ConflictType::OverlappingChanges,
                        description: conflict,
                        resolution_options: self.get_overlap_resolution_options(),
                    });
                }
            }
        }

        // Check for conflicts with pending transformations
        for change in changes {
            if let Some(pending_conflicts) = self.check_pending_conflicts(change) {
                conflicts.extend(pending_conflicts);
            }
        }

        Ok(conflicts)
    }

    /// Check for overlapping changes within the same file
    fn check_overlapping_changes(&self, changes: &[&CodeChange]) -> Option<Vec<String>> {
        let mut conflicts = Vec::new();

        for i in 0..changes.len() {
            for j in (i + 1)..changes.len() {
                let (start1, end1) = changes[i].line_range;
                let (start2, end2) = changes[j].line_range;

                // Check if ranges overlap
                if (start1 <= start2 && end1 >= start2) || (start2 <= start1 && end2 >= start1) {
                    conflicts.push(format!(
                        "Changes to lines {}-{} overlap with changes to lines {}-{}",
                        start1, end1, start2, end2
                    ));
                }
            }
        }

        if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        }
    }

    /// Check for conflicts with pending transformations
    fn check_pending_conflicts(&self, change: &CodeChange) -> Option<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();

        for (id, pending) in &self.pending_transformations {
            for pending_change in &pending.changes {
                if pending_change.file_path == change.file_path {
                    let (start1, end1) = change.line_range;
                    let (start2, end2) = pending_change.line_range;

                    if (start1 <= start2 && end1 >= start2) || (start2 <= start1 && end2 >= start1) {
                        conflicts.push(ConflictInfo {
                            file_path: change.file_path.clone(),
                            conflict_type: ConflictType::PendingConflict,
                            description: format!(
                                "Conflicts with pending transformation '{}'",
                                pending.description
                            ),
                            resolution_options: self.get_pending_resolution_options(id),
                        });
                    }
                }
            }
        }

        if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        }
    }

    /// Get resolution options for overlapping changes
    fn get_overlap_resolution_options(&self) -> Vec<ResolutionOption> {
        vec![
            ResolutionOption {
                name: "Skip".to_string(),
                description: "Skip the conflicting change".to_string(),
                action: ResolutionAction::Skip,
            },
            ResolutionOption {
                name: "Force".to_string(),
                description: "Apply anyway (may override other changes)".to_string(),
                action: ResolutionAction::Force,
            },
            ResolutionOption {
                name: "Reanalyze".to_string(),
                description: "Reanalyze the file with all changes considered".to_string(),
                action: ResolutionAction::Reanalyze,
            },
        ]
    }

    /// Get resolution options for pending conflicts
    fn get_pending_resolution_options(&self, pending_id: &str) -> Vec<ResolutionOption> {
        vec![
            ResolutionOption {
                name: "Skip".to_string(),
                description: "Skip this change".to_string(),
                action: ResolutionAction::Skip,
            },
            ResolutionOption {
                name: "Cancel Pending".to_string(),
                description: format!("Cancel the pending transformation '{}'", pending_id),
                action: ResolutionAction::Skip, // Would actually cancel the pending
            },
            ResolutionOption {
                name: "Apply Both".to_string(),
                description: "Try to apply both transformations".to_string(),
                action: ResolutionAction::Reanalyze,
            },
        ]
    }

    /// Add a transformation to pending list
    pub fn add_pending(&mut self, transformation: Transformation) {
        self.pending_transformations.insert(
            transformation.id.clone(),
            transformation,
        );
    }

    /// Remove a transformation from pending list
    pub fn remove_pending(&mut self, id: &str) {
        self.pending_transformations.remove(id);
    }

    /// Resolve conflicts based on user selection
    pub async fn resolve_conflicts(
        &self,
        conflicts: Vec<ConflictInfo>,
        resolutions: HashMap<String, ResolutionAction>,
    ) -> Result<Vec<CodeChange>> {
        // This would implement the actual resolution logic
        // For now, return empty vec
        Ok(Vec::new())
    }

    /// Check if a file has been modified since a given timestamp
    pub async fn check_file_modified(
        &self,
        file_path: &Path,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool> {
        let metadata = tokio::fs::metadata(file_path).await?;
        let modified = metadata.modified()?;
        
        // Convert system time to chrono
        let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
        
        Ok(modified_time > since)
    }

    /// Detect if changes would break compilation
    pub async fn detect_syntax_breaking_changes(
        &self,
        changes: &[CodeChange],
    ) -> Result<Vec<ConflictInfo>> {
        // This would use the parser to check if changes break syntax
        // For now, return empty vec
        Ok(Vec::new())
    }

    /// Merge multiple changes affecting the same region
    pub fn merge_changes(
        &self,
        changes: Vec<CodeChange>,
    ) -> Result<Vec<CodeChange>> {
        // Group by file
        let mut by_file: HashMap<PathBuf, Vec<CodeChange>> = HashMap::new();
        for change in changes {
            by_file
                .entry(change.file_path.clone())
                .or_insert_with(Vec::new)
                .push(change);
        }

        let mut merged = Vec::new();

        for (file_path, mut file_changes) in by_file {
            // Sort by line range
            file_changes.sort_by_key(|c| c.line_range.0);

            // Merge overlapping or adjacent changes
            let mut current: Option<CodeChange> = None;

            for change in file_changes {
                match current {
                    None => current = Some(change),
                    Some(ref mut curr) => {
                        // Check if changes are adjacent or overlapping
                        if curr.line_range.1 >= change.line_range.0 - 1 {
                            // Merge
                            curr.line_range.1 = curr.line_range.1.max(change.line_range.1);
                            curr.description = format!("{}\n{}", curr.description, change.description);
                            curr.confidence = curr.confidence.min(change.confidence);
                        } else {
                            // Not adjacent, save current and start new
                            merged.push(curr.clone());
                            current = Some(change);
                        }
                    }
                }
            }

            if let Some(curr) = current {
                merged.push(curr);
            }
        }

        Ok(merged)
    }
}