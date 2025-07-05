//! Code applier that handles the actual file modifications

use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use sha2::{Sha256, Digest};

use crate::transformation::{
    types::*,
    history::{TransformationHistory, FileBackup},
    syntax::SyntaxAwareModifier,
};
use crate::core::Language;
use crate::analysis::LanguageDetector;

/// Applies code transformations to files
pub struct CodeApplier {
    history: Arc<Mutex<TransformationHistory>>,
    syntax_modifier: SyntaxAwareModifier,
}

impl CodeApplier {
    pub fn new(history: Arc<Mutex<TransformationHistory>>, parser: Arc<Mutex<crate::analysis::TreeSitterParser>>) -> Self {
        Self {
            history,
            syntax_modifier: SyntaxAwareModifier::new(parser),
        }
    }

    /// Apply a transformation
    pub async fn apply(&self, mut transformation: Transformation) -> Result<String> {
        if transformation.applied {
            return Err(anyhow!("Transformation already applied"));
        }

        let mut backups = Vec::new();

        // Group changes by file for efficient application
        let mut changes_by_file = std::collections::HashMap::new();
        for change in transformation.changes {
            changes_by_file
                .entry(change.file_path.clone())
                .or_insert_with(Vec::new)
                .push(change);
        }

        // Apply changes to each file
        for (file_path, mut changes) in changes_by_file {
            // Sort changes by line number in reverse order
            changes.sort_by(|a, b| b.line_range.0.cmp(&a.line_range.0));

            // Create backup before modifying
            let original_content = tokio::fs::read_to_string(&file_path).await?;
            let checksum = self.calculate_checksum(&original_content);

            // Apply all changes to this file
            let new_content = self.apply_changes_to_file(&file_path, &original_content, changes).await?;

            // Write the new content
            tokio::fs::write(&file_path, &new_content).await?;

            // Store backup
            backups.push(FileBackup {
                file_path: file_path.clone(),
                original_content,
                new_content,
                checksum,
            });
        }

        // Record the transaction
        let history = self.history.lock().await;
        let transaction_id = history.record_transaction(&transformation.id, backups).await?;
        
        // Mark transformation as applied
        transformation.applied = true;
        transformation.transaction_id = Some(transaction_id.clone());
        history.mark_applied(&transformation.id, &transaction_id).await?;

        Ok(transaction_id)
    }

    /// Apply multiple changes to a single file
    async fn apply_changes_to_file(
        &self,
        file_path: &Path,
        original_content: &str,
        changes: Vec<CodeChange>,
    ) -> Result<String> {
        let detector = LanguageDetector::new();
        let language = detector.detect_from_path(file_path)?;
        let mut content = original_content.to_string();

        // Apply changes in reverse order (bottom to top) to preserve line numbers
        for change in changes {
            content = self.apply_single_change(&content, change, language.clone()).await?;
        }

        // Verify the final syntax is valid
        self.syntax_modifier.verify_syntax(&content, language).await?;

        Ok(content)
    }

    /// Apply a single change
    async fn apply_single_change(
        &self,
        content: &str,
        change: CodeChange,
        language: Language,
    ) -> Result<String> {
        // Extract the modification content from the change
        let lines: Vec<&str> = change.new_content.lines().collect();
        let (start, end) = change.line_range;
        
        // Extract the relevant portion from new_content
        if start > 0 && end <= lines.len() {
            let modification = lines[(start - 1)..end].join("\n");
            
            self.syntax_modifier.apply_modification(
                content,
                &modification,
                change.line_range,
                language,
            ).await
        } else {
            // If we can't extract specific lines, try a different approach
            // This might happen if new_content is the entire file content
            Ok(change.new_content.clone())
        }
    }

    /// Undo a transaction
    pub async fn undo(&self, transaction_id: &str) -> Result<()> {
        let history = self.history.lock().await;
        let transactions = history.get_transaction_history(self.history.lock().await.max_history).await;
        
        let transaction = transactions.iter()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        if !transaction.applied {
            return Err(anyhow!("Transaction not applied"));
        }

        // Restore files from backup
        for backup in &transaction.files_backup {
            // Verify current checksum matches what we expect
            let current_content = tokio::fs::read_to_string(&backup.file_path).await?;
            let current_checksum = self.calculate_checksum(&current_content);
            
            if current_checksum != self.calculate_checksum(&backup.new_content) {
                return Err(anyhow!(
                    "File {} has been modified since transformation was applied",
                    backup.file_path.display()
                ));
            }

            // Restore original content
            tokio::fs::write(&backup.file_path, &backup.original_content).await?;
        }

        history.mark_undone(transaction_id).await?;

        Ok(())
    }

    /// Redo a transaction
    pub async fn redo(&self, transaction_id: &str) -> Result<()> {
        let history = self.history.lock().await;
        let transactions = history.get_transaction_history(self.history.lock().await.max_history).await;
        
        let transaction = transactions.iter()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        if transaction.applied {
            return Err(anyhow!("Transaction already applied"));
        }

        // Reapply changes from backup
        for backup in &transaction.files_backup {
            // Verify current content matches original
            let current_content = tokio::fs::read_to_string(&backup.file_path).await?;
            let current_checksum = self.calculate_checksum(&current_content);
            
            if current_checksum != backup.checksum {
                return Err(anyhow!(
                    "File {} has been modified since transformation was undone",
                    backup.file_path.display()
                ));
            }

            // Apply new content
            tokio::fs::write(&backup.file_path, &backup.new_content).await?;
        }

        history.mark_redone(transaction_id).await?;

        Ok(())
    }

    /// Calculate checksum of content
    fn calculate_checksum(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Validate that a transformation can be applied
    pub async fn validate_transformation(&self, transformation: &Transformation) -> Result<()> {
        for change in &transformation.changes {
            // Check file exists
            if !change.file_path.exists() {
                return Err(anyhow!("File {} does not exist", change.file_path.display()));
            }

            // Check file is readable and writable
            let metadata = tokio::fs::metadata(&change.file_path).await?;
            if metadata.permissions().readonly() {
                return Err(anyhow!("File {} is read-only", change.file_path.display()));
            }

            // Verify syntax would be valid after change
            let detector = LanguageDetector::new();
            let language = detector.detect_from_path(&change.file_path)?;
            self.syntax_modifier.verify_syntax(&change.new_content, language).await?;
        }

        Ok(())
    }

    /// Rollback all changes since a specific point
    pub async fn rollback_to(&self, transaction_id: &str) -> Result<()> {
        let history = self.history.lock().await;
        let transactions = history.get_transaction_history(self.history.lock().await.max_history).await;

        // Find all transactions after the target
        let mut to_undo = Vec::new();
        let mut found = false;
        
        for transaction in transactions.iter() {
            if found && transaction.applied {
                to_undo.push(transaction.id.clone());
            }
            if transaction.id == transaction_id {
                found = true;
            }
        }

        if !found {
            return Err(anyhow!("Transaction {} not found", transaction_id));
        }

        // Undo transactions in reverse order
        drop(history); // Release lock before calling undo
        for id in to_undo.iter().rev() {
            self.undo(id).await?;
        }

        Ok(())
    }
}