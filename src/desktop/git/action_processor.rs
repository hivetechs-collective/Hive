//! Diff action processor
//! 
//! Handles the execution of staging, unstaging, and reverting operations for hunks and lines

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use tracing::{info, warn, error, instrument};

use crate::desktop::git::{
    DiffAction, DiffActionResult, DiffActionState, DiffHunk, DiffLine, 
    DiffGitOperations, HunkStageStatus, get_file_diff_with_context
};

/// Action processor for handling diff operations with queuing and batching
#[derive(Clone)]
pub struct DiffActionProcessor {
    /// Git operations handler
    git_ops: Arc<DiffGitOperations>,
    /// Current action state
    state: Arc<RwLock<DiffActionState>>,
    /// Action queue for batching operations
    action_queue: Arc<Mutex<Vec<QueuedAction>>>,
    /// Currently processing actions (for UI feedback)
    processing: Arc<Mutex<HashMap<String, ProcessingInfo>>>,
    /// Configuration
    config: ProcessorConfig,
}

/// Queued action for batch processing
#[derive(Debug, Clone)]
struct QueuedAction {
    action: DiffAction,
    timestamp: std::time::Instant,
    priority: ActionPriority,
    context: ActionExecutionContext,
}

/// Action execution context
#[derive(Debug, Clone)]
struct ActionExecutionContext {
    file_path: PathBuf,
    hunk_id: Option<String>,
    line_ids: Vec<String>,
    user_initiated: bool,
}

/// Action priority for queue ordering
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ActionPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Processing information for UI feedback
#[derive(Debug, Clone)]
struct ProcessingInfo {
    action: DiffAction,
    started_at: std::time::Instant,
    progress: f32, // 0.0 to 1.0
    message: String,
}

/// Processor configuration
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Maximum batch size for operations
    pub max_batch_size: usize,
    /// Timeout for individual operations
    pub operation_timeout: std::time::Duration,
    /// Whether to enable automatic batching
    pub auto_batching: bool,
    /// Debounce delay for rapid operations
    pub debounce_delay: std::time::Duration,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            operation_timeout: std::time::Duration::from_secs(30),
            auto_batching: true,
            debounce_delay: std::time::Duration::from_millis(200),
        }
    }
}

/// Action execution result with detailed information
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub action: DiffAction,
    pub success: bool,
    pub message: String,
    pub duration: std::time::Duration,
    pub updated_state: Option<DiffActionState>,
    pub undo_id: Option<String>,
}

impl DiffActionProcessor {
    /// Create new action processor
    pub fn new(repo_path: &Path, file_path: PathBuf) -> Result<Self> {
        let git_ops = Arc::new(DiffGitOperations::new(repo_path)?);
        let state = Arc::new(RwLock::new(DiffActionState::new(file_path)));
        
        Ok(Self {
            git_ops,
            state,
            action_queue: Arc::new(Mutex::new(Vec::new())),
            processing: Arc::new(Mutex::new(HashMap::new())),
            config: ProcessorConfig::default(),
        })
    }
    
    /// Create new action processor with custom configuration
    pub fn with_config(repo_path: &Path, file_path: PathBuf, config: ProcessorConfig) -> Result<Self> {
        let mut processor = Self::new(repo_path, file_path)?;
        processor.config = config;
        Ok(processor)
    }
    
    /// Update processor state from diff result
    pub async fn update_state_from_diff(&self, diff: &crate::desktop::git::DiffResult) {
        let mut state = self.state.write().await;
        state.update_from_diff(diff);
    }
    
    /// Get current action state
    pub async fn get_state(&self) -> DiffActionState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// Execute action immediately (bypass queue)
    #[instrument(skip(self), fields(action = ?action))]
    pub async fn execute_immediate(&self, action: DiffAction, file_path: &Path) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let action_id = self.generate_action_id(&action);
        
        // Add to processing list
        {
            let mut processing = self.processing.lock().await;
            processing.insert(action_id.clone(), ProcessingInfo {
                action: action.clone(),
                started_at: start_time,
                progress: 0.0,
                message: "Starting operation...".to_string(),
            });
        }
        
        let result = self.execute_action_internal(action.clone(), file_path).await;
        let duration = start_time.elapsed();
        
        // Remove from processing list
        {
            let mut processing = self.processing.lock().await;
            processing.remove(&action_id);
        }
        
        // Update state if successful
        if let Ok(ref action_result) = result {
            if action_result.success {
                let mut state = self.state.write().await;
                state.apply_action(action_result);
            }
        }
        
        let execution_result = match result {
            Ok(action_result) => ExecutionResult {
                action: action.clone(),
                success: action_result.success,
                message: action_result.message,
                duration,
                updated_state: if action_result.success { 
                    Some(self.get_state().await) 
                } else { 
                    None 
                },
                undo_id: action_result.undo_id,
            },
            Err(e) => ExecutionResult {
                action: action.clone(),
                success: false,
                message: format!("Operation failed: {}", e),
                duration,
                updated_state: None,
                undo_id: None,
            },
        };
        
        info!("Executed action {:?} in {:?}: {}", 
              action, duration, execution_result.message);
        
        Ok(execution_result)
    }
    
    /// Queue action for batch processing
    pub async fn queue_action(&self, action: DiffAction, file_path: PathBuf) -> Result<()> {
        let priority = self.get_action_priority(&action);
        let hunk_id = self.extract_hunk_id(&action);
        let line_ids = self.extract_line_ids(&action);
        
        let queued_action = QueuedAction {
            action: action.clone(),
            timestamp: std::time::Instant::now(),
            priority: priority.clone(),
            context: ActionExecutionContext {
                file_path,
                hunk_id,
                line_ids,
                user_initiated: true,
            },
        };
        
        let mut queue = self.action_queue.lock().await;
        queue.push(queued_action);
        
        // Sort by priority and timestamp
        queue.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });
        
        info!("Queued action {:?} with priority {:?}", action, priority);
        
        // Process queue if auto-batching is enabled
        if self.config.auto_batching {
            self.process_queue().await?;
        }
        
        Ok(())
    }
    
    /// Process queued actions in batches
    pub async fn process_queue(&self) -> Result<Vec<ExecutionResult>> {
        let mut queue = self.action_queue.lock().await;
        if queue.is_empty() {
            return Ok(Vec::new());
        }
        
        let batch_size = self.config.max_batch_size.min(queue.len());
        let batch: Vec<QueuedAction> = queue.drain(0..batch_size).collect();
        drop(queue); // Release lock early
        
        info!("Processing batch of {} actions", batch.len());
        
        let mut results = Vec::new();
        
        for queued_action in batch {
            let result = self.execute_immediate(
                queued_action.action,
                &queued_action.context.file_path
            ).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Execute specific git operation
    async fn execute_action_internal(&self, action: DiffAction, file_path: &Path) -> Result<DiffActionResult> {
        let state = self.state.read().await;
        
        match action {
            DiffAction::StageHunk(hunk_id) => {
                if let Some(hunk) = state.hunks.get(&hunk_id) {
                    self.git_ops.stage_hunk(file_path, hunk).await
                } else {
                    Err(anyhow::anyhow!("Hunk not found: {}", hunk_id))
                }
            },
            DiffAction::UnstageHunk(hunk_id) => {
                if let Some(hunk) = state.hunks.get(&hunk_id) {
                    self.git_ops.unstage_hunk(file_path, hunk).await
                } else {
                    Err(anyhow::anyhow!("Hunk not found: {}", hunk_id))
                }
            },
            DiffAction::StageLine(line_id) => {
                if let Some(line) = state.lines.get(&line_id) {
                    self.git_ops.stage_lines(file_path, &[line_id], &state.lines).await
                } else {
                    Err(anyhow::anyhow!("Line not found: {}", line_id))
                }
            },
            DiffAction::UnstageLine(line_id) => {
                // For unstaging lines, we need to create a reverse operation
                if let Some(line) = state.lines.get(&line_id) {
                    // This is simplified - in reality we'd need more complex logic
                    self.git_ops.stage_lines(file_path, &[], &state.lines).await
                } else {
                    Err(anyhow::anyhow!("Line not found: {}", line_id))
                }
            },
            DiffAction::RevertHunk(hunk_id) => {
                if let Some(hunk) = state.hunks.get(&hunk_id) {
                    self.git_ops.revert_hunk(file_path, hunk).await
                } else {
                    Err(anyhow::anyhow!("Hunk not found: {}", hunk_id))
                }
            },
            DiffAction::RevertLine(line_id) => {
                // For reverting lines, we need to apply a reverse patch
                if let Some(_line) = state.lines.get(&line_id) {
                    // This would require more complex implementation
                    Err(anyhow::anyhow!("Line revert not implemented yet"))
                } else {
                    Err(anyhow::anyhow!("Line not found: {}", line_id))
                }
            },
            DiffAction::UndoAction(undo_id) => {
                // Find and execute undo operation
                if let Some(undo_entry) = state.undo_stack.iter().find(|u| u.undo_id == undo_id) {
                    // This would require complex state restoration
                    Err(anyhow::anyhow!("Undo operation not implemented yet"))
                } else {
                    Err(anyhow::anyhow!("Undo entry not found: {}", undo_id))
                }
            },
        }
    }
    
    /// Get processing status for UI feedback
    pub async fn get_processing_status(&self) -> HashMap<String, ProcessingInfo> {
        let processing = self.processing.lock().await;
        processing.clone()
    }
    
    /// Check if any actions are currently processing
    pub async fn is_processing(&self) -> bool {
        let processing = self.processing.lock().await;
        !processing.is_empty()
    }
    
    /// Cancel all queued actions
    pub async fn cancel_queued_actions(&self) -> usize {
        let mut queue = self.action_queue.lock().await;
        let count = queue.len();
        queue.clear();
        info!("Cancelled {} queued actions", count);
        count
    }
    
    /// Get queue status
    pub async fn get_queue_status(&self) -> (usize, usize) {
        let queue = self.action_queue.lock().await;
        let processing = self.processing.lock().await;
        (queue.len(), processing.len())
    }
    
    /// Generate unique action ID for tracking
    fn generate_action_id(&self, action: &DiffAction) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        
        match action {
            DiffAction::StageHunk(id) => format!("stage_hunk_{}_{}", id, timestamp),
            DiffAction::UnstageHunk(id) => format!("unstage_hunk_{}_{}", id, timestamp),
            DiffAction::StageLine(id) => format!("stage_line_{}_{}", id, timestamp),
            DiffAction::UnstageLine(id) => format!("unstage_line_{}_{}", id, timestamp),
            DiffAction::RevertHunk(id) => format!("revert_hunk_{}_{}", id, timestamp),
            DiffAction::RevertLine(id) => format!("revert_line_{}_{}", id, timestamp),
            DiffAction::UndoAction(id) => format!("undo_{}_{}", id, timestamp),
        }
    }
    
    /// Get action priority for queue ordering
    fn get_action_priority(&self, action: &DiffAction) -> ActionPriority {
        match action {
            DiffAction::UndoAction(_) => ActionPriority::High,
            DiffAction::RevertHunk(_) | DiffAction::RevertLine(_) => ActionPriority::High,
            DiffAction::StageHunk(_) | DiffAction::UnstageHunk(_) => ActionPriority::Normal,
            DiffAction::StageLine(_) | DiffAction::UnstageLine(_) => ActionPriority::Low,
        }
    }
    
    /// Extract hunk ID from action
    fn extract_hunk_id(&self, action: &DiffAction) -> Option<String> {
        match action {
            DiffAction::StageHunk(id) | DiffAction::UnstageHunk(id) | DiffAction::RevertHunk(id) => Some(id.clone()),
            _ => None,
        }
    }
    
    /// Extract line IDs from action
    fn extract_line_ids(&self, action: &DiffAction) -> Vec<String> {
        match action {
            DiffAction::StageLine(id) | DiffAction::UnstageLine(id) | DiffAction::RevertLine(id) => vec![id.clone()],
            _ => Vec::new(),
        }
    }
}

/// Batch processor for handling multiple similar operations efficiently
#[derive(Clone)]
pub struct BatchProcessor {
    processor: Arc<DiffActionProcessor>,
    pending_operations: Arc<Mutex<HashMap<String, Vec<DiffAction>>>>, // file_path -> actions
    debounce_timers: Arc<Mutex<HashMap<String, tokio::time::Instant>>>,
}

impl BatchProcessor {
    /// Create new batch processor
    pub fn new(processor: Arc<DiffActionProcessor>) -> Self {
        Self {
            processor,
            pending_operations: Arc::new(Mutex::new(HashMap::new())),
            debounce_timers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Add operation to batch (with debouncing)
    pub async fn add_operation(&self, file_path: String, action: DiffAction) -> Result<()> {
        let now = tokio::time::Instant::now();
        
        // Update debounce timer
        {
            let mut timers = self.debounce_timers.lock().await;
            timers.insert(file_path.clone(), now);
        }
        
        // Add to pending operations
        {
            let mut pending = self.pending_operations.lock().await;
            pending.entry(file_path.clone()).or_insert_with(Vec::new).push(action);
        }
        
        // Schedule batch processing after debounce delay
        let processor = self.processor.clone();
        let pending_ops = self.pending_operations.clone();
        let timers = self.debounce_timers.clone();
        let file_path_clone = file_path.clone();
        let debounce_delay = processor.config.debounce_delay;
        
        // Clone what we need for the async block (avoiding the processor)
        let processor_clone = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(debounce_delay).await;
            
            // Check if this is still the latest timer
            let should_process = {
                let timers = timers.lock().await;
                timers.get(&file_path_clone).map_or(false, |&timer| timer == now)
            };
            
            if should_process {
                // Process batch for this file
                let actions = {
                    let mut pending = pending_ops.lock().await;
                    pending.remove(&file_path_clone).unwrap_or_default()
                };
                
                for action in actions {
                    if let Err(e) = processor_clone.processor.queue_action(action, PathBuf::from(&file_path_clone)).await {
                        error!("Failed to queue batched action: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_action_processor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        let file_path = repo_path.join("test.txt");
        
        // This would fail in a real test since we don't have a git repo
        // but it tests the basic structure
        let result = DiffActionProcessor::new(repo_path, file_path);
        assert!(result.is_err()); // Expected to fail without git repo
    }
    
    #[test]
    fn test_action_priority_ordering() {
        let mut priorities = vec![
            ActionPriority::Low,
            ActionPriority::Critical,
            ActionPriority::Normal,
            ActionPriority::High,
        ];
        
        priorities.sort();
        
        assert_eq!(priorities, vec![
            ActionPriority::Low,
            ActionPriority::Normal,
            ActionPriority::High,
            ActionPriority::Critical,
        ]);
    }
    
    #[test]
    fn test_key_combination_matching() {
        use crate::desktop::git::keyboard_shortcuts::KeyCombination;
        
        let combo = KeyCombination {
            key: "KeyS".to_string(),
            alt: true,
            ctrl: false,
            shift: false,
            meta: false,
        };
        
        assert_eq!(combo.to_string(), "Alt+S");
    }
}