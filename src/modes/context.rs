//! Context Management for Mode Operations
//! 
//! Provides intelligent context preservation and transformation
//! during mode switches to ensure seamless transitions.

use crate::core::error::{HiveResult, HiveError};
use crate::planning::ModeType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Context manager for mode operations
pub struct ContextManager {
    storage: Arc<RwLock<ContextStorage>>,
    serializer: ContextSerializer,
    compressor: ContextCompressor,
    validator: ContextValidator,
}

/// Storage for context data
#[derive(Debug)]
struct ContextStorage {
    active_contexts: HashMap<ModeType, ModeContext>,
    snapshots: Vec<ContextSnapshot>,
    history: ContextHistory,
}

/// Serializes context for persistence
#[derive(Debug)]
struct ContextSerializer {
    format: SerializationFormat,
    compression: bool,
}

/// Compresses context data
#[derive(Debug)]
struct ContextCompressor {
    algorithm: CompressionAlgorithm,
    threshold: usize,
}

/// Validates context integrity
#[derive(Debug)]
struct ContextValidator {
    rules: Vec<Box<dyn ValidationRule + Send + Sync>>,
}

/// Format for serialization
#[derive(Debug, Clone, PartialEq)]
enum SerializationFormat {
    Json,
    MessagePack,
    Bincode,
}

/// Compression algorithm
#[derive(Debug, Clone, PartialEq)]
enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
}

/// Context for a specific mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeContext {
    pub mode: ModeType,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub data: ContextData,
    pub metadata: ContextMetadata,
}

/// Core context data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    pub active_tasks: Vec<ActiveTask>,
    pub user_state: UserState,
    pub workspace: WorkspaceState,
    pub cache: HashMap<String, serde_json::Value>,
}

/// Metadata about context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub version: String,
    pub source_mode: ModeType,
    pub tags: Vec<String>,
    pub importance: ImportanceLevel,
}

/// Active task in context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTask {
    pub id: String,
    pub title: String,
    pub progress: f32,
    pub priority: TaskPriority,
    pub data: HashMap<String, serde_json::Value>,
}

/// User state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub current_focus: String,
    pub recent_actions: Vec<RecentAction>,
    pub preferences: HashMap<String, String>,
    pub session_id: String,
}

/// Workspace state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub open_files: Vec<String>,
    pub cursor_positions: HashMap<String, CursorPosition>,
    pub unsaved_changes: HashMap<String, String>,
    pub terminal_history: Vec<String>,
}

/// Recent user action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentAction {
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub context: String,
}

/// Cursor position in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Importance level for context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportanceLevel {
    Essential,
    Important,
    Standard,
    Optional,
}

/// Snapshot of context at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub id: String,
    pub mode: ModeType,
    pub timestamp: DateTime<Utc>,
    pub data: ContextData,
    pub metadata: SnapshotMetadata,
    preserved_items: usize,
    transformed_items: usize,
}

/// Metadata for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub reason: String,
    pub auto_generated: bool,
    pub expiry: Option<DateTime<Utc>>,
    pub size_bytes: usize,
}

/// History of context operations
#[derive(Debug)]
struct ContextHistory {
    operations: Vec<ContextOperation>,
    max_size: usize,
}

/// Operation performed on context
#[derive(Debug, Clone)]
struct ContextOperation {
    operation_type: OperationType,
    timestamp: DateTime<Utc>,
    mode: ModeType,
    details: String,
}

/// Type of context operation
#[derive(Debug, Clone, PartialEq)]
enum OperationType {
    Create,
    Update,
    Snapshot,
    Restore,
    Clear,
}

/// Validation rule for context
trait ValidationRule: std::fmt::Debug {
    fn validate(&self, context: &ModeContext) -> ValidationResult;
}

/// Result of validation
#[derive(Debug)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl ContextManager {
    /// Create a new context manager
    pub async fn new() -> HiveResult<Self> {
        Ok(Self {
            storage: Arc::new(RwLock::new(ContextStorage::new())),
            serializer: ContextSerializer::new(),
            compressor: ContextCompressor::new(),
            validator: ContextValidator::new(),
        })
    }
    
    /// Capture a snapshot of current context
    pub async fn capture_snapshot(&self, mode: &ModeType) -> HiveResult<ContextSnapshot> {
        let storage = self.storage.read().await;
        
        let context = storage.active_contexts.get(mode)
            .ok_or_else(|| HiveError::Planning(format!("No active context for mode {:?}", mode)))?;
        
        // Validate context before snapshot
        let validation = self.validator.validate(context)?;
        if !validation.valid {
            return Err(HiveError::Planning(format!(
                "Context validation failed: {:?}", 
                validation.errors
            )));
        }
        
        // Create snapshot
        let snapshot = ContextSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            mode: mode.clone(),
            timestamp: Utc::now(),
            data: context.data.clone(),
            metadata: SnapshotMetadata {
                reason: "Mode switch".to_string(),
                auto_generated: true,
                expiry: Some(Utc::now() + chrono::Duration::hours(24)),
                size_bytes: self.estimate_size(&context.data),
            },
            preserved_items: context.data.active_tasks.len(),
            transformed_items: 0,
        };
        
        // Compress if needed
        let final_snapshot = if snapshot.metadata.size_bytes > self.compressor.threshold {
            self.compressor.compress(snapshot)?
        } else {
            snapshot
        };
        
        Ok(final_snapshot)
    }
    
    /// Restore context from snapshot
    pub async fn restore_snapshot(
        &mut self,
        mode: &ModeType,
        snapshot: &ContextSnapshot
    ) -> HiveResult<()> {
        // Decompress if needed
        let restored_data = self.compressor.decompress(&snapshot.data)?;
        
        // Create new context from snapshot
        let context = ModeContext {
            mode: mode.clone(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
            data: restored_data,
            metadata: ContextMetadata {
                version: "2.0".to_string(),
                source_mode: snapshot.mode.clone(),
                tags: vec!["restored".to_string()],
                importance: ImportanceLevel::Important,
            },
        };
        
        // Validate restored context
        let validation = self.validator.validate(&context)?;
        if !validation.valid {
            return Err(HiveError::Planning(format!(
                "Restored context validation failed: {:?}",
                validation.errors
            )));
        }
        
        // Store context
        let mut storage = self.storage.write().await;
        storage.active_contexts.insert(mode.clone(), context);
        
        // Record operation
        storage.history.add_operation(ContextOperation {
            operation_type: OperationType::Restore,
            timestamp: Utc::now(),
            mode: mode.clone(),
            details: format!("Restored from snapshot {}", snapshot.id),
        });
        
        Ok(())
    }
    
    /// Get current context for a mode
    pub async fn get_context(&self, mode: &ModeType) -> HiveResult<ModeContext> {
        let storage = self.storage.read().await;
        
        storage.active_contexts.get(mode)
            .cloned()
            .ok_or_else(|| HiveError::Planning(format!("No context for mode {:?}", mode)))
    }
    
    /// Update context for a mode
    pub async fn update_context(
        &mut self,
        mode: &ModeType,
        updater: impl FnOnce(&mut ModeContext) -> HiveResult<()>
    ) -> HiveResult<()> {
        let mut storage = self.storage.write().await;
        
        let context = storage.active_contexts.entry(mode.clone())
            .or_insert_with(|| ModeContext::new(mode.clone()));
        
        updater(context)?;
        context.last_modified = Utc::now();
        
        // Validate updated context
        let validation = self.validator.validate(context)?;
        if !validation.valid {
            return Err(HiveError::Planning(format!(
                "Context update validation failed: {:?}",
                validation.errors
            )));
        }
        
        Ok(())
    }
    
    /// Clear context for a mode
    pub async fn clear_context(&mut self) -> HiveResult<()> {
        let mut storage = self.storage.write().await;
        storage.active_contexts.clear();
        
        storage.history.add_operation(ContextOperation {
            operation_type: OperationType::Clear,
            timestamp: Utc::now(),
            mode: ModeType::Hybrid,
            details: "All contexts cleared".to_string(),
        });
        
        Ok(())
    }
    
    /// Get context statistics
    pub async fn get_statistics(&self) -> HiveResult<ContextStatistics> {
        let storage = self.storage.read().await;
        
        let total_size: usize = storage.active_contexts.values()
            .map(|ctx| self.estimate_size(&ctx.data))
            .sum();
        
        Ok(ContextStatistics {
            active_contexts: storage.active_contexts.len(),
            total_snapshots: storage.snapshots.len(),
            total_size_bytes: total_size,
            operations_count: storage.history.operations.len(),
            modes_with_context: storage.active_contexts.keys().cloned().collect(),
        })
    }
    
    // Private helper methods
    
    fn estimate_size(&self, data: &ContextData) -> usize {
        // Rough estimation of context size
        let tasks_size = data.active_tasks.len() * 200;
        let cache_size = data.cache.len() * 100;
        let workspace_size = data.workspace.open_files.len() * 50;
        
        tasks_size + cache_size + workspace_size + 1024 // Base overhead
    }
}

impl ContextSnapshot {
    /// Get count of preserved items
    pub fn preserved_count(&self) -> usize {
        self.preserved_items
    }
    
    /// Get count of transformed items
    pub fn transformed_count(&self) -> usize {
        self.transformed_items
    }
    
    /// Get total items in snapshot
    pub fn total_items(&self) -> usize {
        self.data.active_tasks.len() + 
        self.data.workspace.open_files.len() +
        self.data.cache.len()
    }
    
    /// Check if snapshot has active tasks
    pub fn has_active_tasks(&self) -> bool {
        !self.data.active_tasks.is_empty()
    }
    
    /// Check if snapshot has mode-specific data
    pub fn has_mode_specific_data(&self, mode: &ModeType) -> bool {
        self.mode == *mode && self.total_items() > 0
    }
    
    /// Mark snapshot as transformed
    pub fn mark_as_transformed(&mut self) {
        self.transformed_items = self.total_items();
    }
}

impl ModeContext {
    /// Create new context for a mode
    fn new(mode: ModeType) -> Self {
        Self {
            mode: mode.clone(), // Clone for mode field
            created_at: Utc::now(),
            last_modified: Utc::now(),
            data: ContextData {
                active_tasks: Vec::new(),
                user_state: UserState {
                    current_focus: String::new(),
                    recent_actions: Vec::new(),
                    preferences: HashMap::new(),
                    session_id: uuid::Uuid::new_v4().to_string(),
                },
                workspace: WorkspaceState {
                    open_files: Vec::new(),
                    cursor_positions: HashMap::new(),
                    unsaved_changes: HashMap::new(),
                    terminal_history: Vec::new(),
                },
                cache: HashMap::new(),
            },
            metadata: ContextMetadata {
                version: "2.0".to_string(),
                source_mode: mode,
                tags: Vec::new(),
                importance: ImportanceLevel::Standard,
            },
        }
    }
}

impl ContextStorage {
    fn new() -> Self {
        Self {
            active_contexts: HashMap::new(),
            snapshots: Vec::new(),
            history: ContextHistory::new(),
        }
    }
}

impl ContextSerializer {
    fn new() -> Self {
        Self {
            format: SerializationFormat::Json,
            compression: true,
        }
    }
}

impl ContextCompressor {
    fn new() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Gzip,
            threshold: 10 * 1024, // 10KB
        }
    }
    
    fn compress(&self, snapshot: ContextSnapshot) -> HiveResult<ContextSnapshot> {
        // In real implementation, would compress data
        Ok(snapshot)
    }
    
    fn decompress(&self, data: &ContextData) -> HiveResult<ContextData> {
        // In real implementation, would decompress data
        Ok(data.clone())
    }
}

impl ContextValidator {
    fn new() -> Self {
        Self {
            rules: vec![
                Box::new(TaskIntegrityRule),
                Box::new(WorkspaceValidityRule),
                Box::new(SizeLimitRule),
            ],
        }
    }
    
    fn validate(&self, context: &ModeContext) -> HiveResult<ValidationResult> {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut is_valid = true;
        
        for rule in &self.rules {
            let result = rule.validate(context);
            if !result.valid {
                is_valid = false;
            }
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }
        
        Ok(ValidationResult {
            valid: is_valid,
            errors: all_errors,
            warnings: all_warnings,
        })
    }
}

impl ContextHistory {
    fn new() -> Self {
        Self {
            operations: Vec::new(),
            max_size: 1000,
        }
    }
    
    fn add_operation(&mut self, operation: ContextOperation) {
        self.operations.push(operation);
        
        // Trim if exceeds max size
        if self.operations.len() > self.max_size {
            self.operations.remove(0);
        }
    }
}

// Validation rule implementations

#[derive(Debug)]
struct TaskIntegrityRule;

impl ValidationRule for TaskIntegrityRule {
    fn validate(&self, context: &ModeContext) -> ValidationResult {
        let mut errors = Vec::new();
        
        for task in &context.data.active_tasks {
            if task.id.is_empty() {
                errors.push("Task with empty ID found".to_string());
            }
            if task.progress < 0.0 || task.progress > 1.0 {
                errors.push(format!("Invalid progress {} for task {}", task.progress, task.id));
            }
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct WorkspaceValidityRule;

impl ValidationRule for WorkspaceValidityRule {
    fn validate(&self, context: &ModeContext) -> ValidationResult {
        let mut warnings = Vec::new();
        
        if context.data.workspace.unsaved_changes.len() > 10 {
            warnings.push("Large number of unsaved changes".to_string());
        }
        
        ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings,
        }
    }
}

#[derive(Debug)]
struct SizeLimitRule;

impl ValidationRule for SizeLimitRule {
    fn validate(&self, context: &ModeContext) -> ValidationResult {
        let cache_size = context.data.cache.len();
        let max_cache_size = 1000;
        
        if cache_size > max_cache_size {
            return ValidationResult {
                valid: false,
                errors: vec![format!("Cache size {} exceeds limit {}", cache_size, max_cache_size)],
                warnings: Vec::new(),
            };
        }
        
        ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Statistics about context operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStatistics {
    pub active_contexts: usize,
    pub total_snapshots: usize,
    pub total_size_bytes: usize,
    pub operations_count: usize,
    pub modes_with_context: Vec<ModeType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_context_manager_creation() {
        // Test context manager initialization
    }
    
    #[tokio::test]
    async fn test_snapshot_capture() {
        // Test capturing context snapshots
    }
    
    #[tokio::test]
    async fn test_context_restoration() {
        // Test restoring from snapshots
    }
    
    #[tokio::test]
    async fn test_context_validation() {
        // Test context validation rules
    }
}