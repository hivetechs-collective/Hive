//! Transformation history and undo/redo functionality

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::transformation::types::*;

/// Transaction representing an applied transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub transformation_id: String,
    pub timestamp: DateTime<Utc>,
    pub applied: bool,
    pub files_backup: Vec<FileBackup>,
}

/// Backup of a file before transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBackup {
    pub file_path: PathBuf,
    pub original_content: String,
    pub new_content: String,
    pub checksum: String,
}

/// Manages transformation history with undo/redo support
pub struct TransformationHistory {
    storage_path: PathBuf,
    transformations: Arc<RwLock<Vec<Transformation>>>,
    transactions: Arc<RwLock<VecDeque<Transaction>>>,
    undo_stack: Arc<RwLock<Vec<String>>>, // Transaction IDs
    redo_stack: Arc<RwLock<Vec<String>>>, // Transaction IDs
    pub max_history: usize,
}

impl TransformationHistory {
    /// Create new transformation history
    pub fn new(config_dir: &Path) -> Result<Self> {
        let storage_path = config_dir.join("transformation_history");
        std::fs::create_dir_all(&storage_path)?;

        let mut history = Self {
            storage_path,
            transformations: Arc::new(RwLock::new(Vec::new())),
            transactions: Arc::new(RwLock::new(VecDeque::new())),
            undo_stack: Arc::new(RwLock::new(Vec::new())),
            redo_stack: Arc::new(RwLock::new(Vec::new())),
            max_history: 100,
        };

        // Load existing history
        history.load_history()?;

        Ok(history)
    }

    /// Add a new transformation to history
    pub async fn add_transformation(&self, transformation: Transformation) -> Result<()> {
        let mut transformations = self.transformations.write().await;
        transformations.push(transformation.clone());

        // Persist to disk
        self.save_transformation(&transformation).await?;

        // Trim old transformations if needed
        if transformations.len() > self.max_history * 2 {
            transformations.drain(0..self.max_history);
        }

        Ok(())
    }

    /// Get a transformation by ID
    pub async fn get_transformation(&self, id: &str) -> Option<Transformation> {
        let transformations = self.transformations.read().await;
        transformations.iter().find(|t| t.id == id).cloned()
    }

    /// Record that a transformation was applied
    pub async fn record_transaction(
        &self,
        transformation_id: &str,
        backups: Vec<FileBackup>,
    ) -> Result<String> {
        let transaction = Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            transformation_id: transformation_id.to_string(),
            timestamp: Utc::now(),
            applied: true,
            files_backup: backups,
        };

        let mut transactions = self.transactions.write().await;
        transactions.push_back(transaction.clone());

        // Add to undo stack
        let mut undo_stack = self.undo_stack.write().await;
        undo_stack.push(transaction.id.clone());

        // Clear redo stack (new action invalidates redo history)
        let mut redo_stack = self.redo_stack.write().await;
        redo_stack.clear();

        // Persist transaction
        self.save_transaction(&transaction).await?;

        // Trim old transactions
        if transactions.len() > self.max_history {
            if let Some(old) = transactions.pop_front() {
                self.delete_transaction(&old.id).await?;
            }
        }

        Ok(transaction.id)
    }

    /// Mark a transformation as applied
    pub async fn mark_applied(&self, transformation_id: &str, transaction_id: &str) -> Result<()> {
        let mut transformations = self.transformations.write().await;
        if let Some(transformation) = transformations
            .iter_mut()
            .find(|t| t.id == transformation_id)
        {
            transformation.applied = true;
            transformation.transaction_id = Some(transaction_id.to_string());

            // Update on disk
            self.save_transformation(transformation).await?;
        }
        Ok(())
    }

    /// Get the last transaction
    pub async fn get_last_transaction(&self) -> Result<Transaction> {
        let undo_stack = self.undo_stack.read().await;
        let last_id = undo_stack
            .last()
            .ok_or_else(|| anyhow!("No transactions to undo"))?;

        let transactions = self.transactions.read().await;
        transactions
            .iter()
            .find(|t| &t.id == last_id)
            .cloned()
            .ok_or_else(|| anyhow!("Transaction not found"))
    }

    /// Get the last undone transaction
    pub async fn get_last_undone(&self) -> Result<Transaction> {
        let redo_stack = self.redo_stack.read().await;
        let last_id = redo_stack
            .last()
            .ok_or_else(|| anyhow!("No transactions to redo"))?;

        let transactions = self.transactions.read().await;
        transactions
            .iter()
            .find(|t| &t.id == last_id)
            .cloned()
            .ok_or_else(|| anyhow!("Transaction not found"))
    }

    /// Mark a transaction as undone
    pub async fn mark_undone(&self, transaction_id: &str) -> Result<()> {
        // Move from undo stack to redo stack
        let mut undo_stack = self.undo_stack.write().await;
        let mut redo_stack = self.redo_stack.write().await;

        if let Some(pos) = undo_stack.iter().position(|id| id == transaction_id) {
            let id = undo_stack.remove(pos);
            redo_stack.push(id);
        }

        // Update transaction status
        let mut transactions = self.transactions.write().await;
        if let Some(transaction) = transactions.iter_mut().find(|t| t.id == transaction_id) {
            transaction.applied = false;
            self.save_transaction(transaction).await?;
        }

        Ok(())
    }

    /// Mark a transaction as redone
    pub async fn mark_redone(&self, transaction_id: &str) -> Result<()> {
        // Move from redo stack to undo stack
        let mut undo_stack = self.undo_stack.write().await;
        let mut redo_stack = self.redo_stack.write().await;

        if let Some(pos) = redo_stack.iter().position(|id| id == transaction_id) {
            let id = redo_stack.remove(pos);
            undo_stack.push(id);
        }

        // Update transaction status
        let mut transactions = self.transactions.write().await;
        if let Some(transaction) = transactions.iter_mut().find(|t| t.id == transaction_id) {
            transaction.applied = true;
            self.save_transaction(transaction).await?;
        }

        Ok(())
    }

    /// Get transformation history
    pub async fn get_history(&self, limit: usize) -> Vec<Transformation> {
        let transformations = self.transformations.read().await;
        transformations.iter().rev().take(limit).cloned().collect()
    }

    /// Get transaction history
    pub async fn get_transaction_history(&self, limit: usize) -> Vec<Transaction> {
        let transactions = self.transactions.read().await;
        transactions.iter().rev().take(limit).cloned().collect()
    }

    /// Clear all history
    pub async fn clear_history(&self) -> Result<()> {
        let mut transformations = self.transformations.write().await;
        let mut transactions = self.transactions.write().await;
        let mut undo_stack = self.undo_stack.write().await;
        let mut redo_stack = self.redo_stack.write().await;

        transformations.clear();
        transactions.clear();
        undo_stack.clear();
        redo_stack.clear();

        // Clear storage
        std::fs::remove_dir_all(&self.storage_path)?;
        std::fs::create_dir_all(&self.storage_path)?;

        Ok(())
    }

    // Storage operations

    /// Save transformation to disk
    async fn save_transformation(&self, transformation: &Transformation) -> Result<()> {
        let path = self
            .storage_path
            .join(format!("transformation_{}.json", transformation.id));
        let json = serde_json::to_string_pretty(transformation)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Save transaction to disk
    async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        let path = self
            .storage_path
            .join(format!("transaction_{}.json", transaction.id));
        let json = serde_json::to_string_pretty(transaction)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Delete transaction from disk
    async fn delete_transaction(&self, id: &str) -> Result<()> {
        let path = self.storage_path.join(format!("transaction_{}.json", id));
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }

    /// Load history from disk
    fn load_history(&mut self) -> Result<()> {
        // Load transformations
        let trans_pattern = self.storage_path.join("transformation_*.json");
        if let Ok(paths) = glob::glob(trans_pattern.to_str().unwrap()) {
            for path in paths.flatten() {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(transformation) = serde_json::from_str::<Transformation>(&json) {
                        futures::executor::block_on(async {
                            self.transformations.write().await.push(transformation);
                        });
                    }
                }
            }
        }

        // Load transactions
        let tx_pattern = self.storage_path.join("transaction_*.json");
        if let Ok(paths) = glob::glob(tx_pattern.to_str().unwrap()) {
            for path in paths.flatten() {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(transaction) = serde_json::from_str::<Transaction>(&json) {
                        futures::executor::block_on(async {
                            self.transactions
                                .write()
                                .await
                                .push_back(transaction.clone());
                            if transaction.applied {
                                self.undo_stack.write().await.push(transaction.id);
                            }
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Export history to a file
    pub async fn export_history(&self, path: &Path) -> Result<()> {
        let transformations = self.transformations.read().await;
        let transactions = self.transactions.read().await;

        let export = serde_json::json!({
            "transformations": transformations.clone(),
            "transactions": transactions.clone(),
            "exported_at": Utc::now(),
            "version": "1.0"
        });

        let json = serde_json::to_string_pretty(&export)?;
        tokio::fs::write(path, json).await?;

        Ok(())
    }
}

/// Convenience function for undoing the last transformation
pub async fn undo_last_transformation(history: Arc<TransformationHistory>) -> Result<Transaction> {
    history.get_last_transaction().await
}
