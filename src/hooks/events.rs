//! Hook Events - Event types and event handling system

use super::{ExecutionContext, HookExecutor, HookPriority, HookRegistry};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Types of events that can trigger hooks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    // Consensus Pipeline Events
    BeforeConsensus,
    AfterConsensus,
    BeforeGeneratorStage,
    AfterGeneratorStage,
    BeforeRefinerStage,
    AfterRefinerStage,
    BeforeValidatorStage,
    AfterValidatorStage,
    BeforeCuratorStage,
    AfterCuratorStage,
    ConsensusError,

    // Code Modification Events
    BeforeCodeModification,
    AfterCodeModification,
    BeforeFileWrite,
    AfterFileWrite,
    BeforeFileDelete,
    AfterFileDelete,

    // Analysis Events
    BeforeAnalysis,
    AfterAnalysis,
    AnalysisComplete,
    QualityGateCheck,

    // Cost Control Events
    CostThresholdReached,
    BudgetExceeded,
    CostEstimateAvailable,

    // Repository Events
    BeforeIndexing,
    AfterIndexing,
    RepositoryChanged,
    DependencyChanged,

    // Security Events
    SecurityCheckFailed,
    UntrustedPathAccess,
    PermissionDenied,

    // Planning Events
    PlanCreated,
    TaskCreated,
    TaskCompleted,
    RiskIdentified,
    TimelineUpdated,
    PlanExecutionStarted,
    PlanExecutionCompleted,

    // Memory Events
    ConversationStored,
    PatternDetected,
    MemoryEvictionOccurred,
    ThematicClusterCreated,
    ContextRetrieved,

    // Analytics Events
    ThresholdExceeded,
    AnomalyDetected,
    ReportGenerated,
    DashboardUpdated,
    MetricCalculated,

    // Custom Events
    Custom(String),
}

/// Event data that is passed to hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub event_type: EventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: EventSource,
    pub context: HashMap<String, Value>,
    pub metadata: EventMetadata,
}

impl HookEvent {
    pub fn new(event_type: EventType, source: EventSource) -> Self {
        Self {
            event_type,
            timestamp: chrono::Utc::now(),
            source,
            context: HashMap::new(),
            metadata: EventMetadata::default(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.context.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Source of an event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventSource {
    CLI { command: String },
    Consensus { stage: String },
    FileSystem { path: PathBuf },
    Analysis { target: PathBuf },
    User { id: String },
    System,
}

/// Additional metadata for events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub priority: Option<i32>,
    pub tags: Vec<String>,
}

/// Handles event dispatching to registered hooks with priority queue support
pub struct EventHandler {
    registry: Arc<RwLock<HookRegistry>>,
    executor: Arc<HookExecutor>,
    event_queue: Arc<Mutex<BinaryHeap<QueuedEvent>>>,
    processing: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
struct QueuedEvent {
    event: HookEvent,
    priority: HookPriority,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl Eq for QueuedEvent {}

impl PartialEq for QueuedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier timestamp
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.timestamp.cmp(&self.timestamp),
            ordering => ordering,
        }
    }
}

impl PartialOrd for QueuedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl EventHandler {
    pub fn new(registry: Arc<RwLock<HookRegistry>>, executor: Arc<HookExecutor>) -> Self {
        Self {
            registry,
            executor,
            event_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            processing: Arc::new(Mutex::new(false)),
        }
    }

    /// Handle an event by dispatching it to matching hooks
    pub async fn handle_event(&self, event: HookEvent) -> Result<()> {
        // Find matching hooks
        let registry = self.registry.read().await;
        let mut matching_hooks = registry.find_by_event(&event.event_type);

        // Sort hooks by priority (highest first)
        matching_hooks.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Create execution context
        let context = ExecutionContext::from_event(&event)?;

        // Execute hooks in priority order
        for hook in matching_hooks {
            match self.executor.execute_hook(&hook, context.clone()).await {
                Ok(_) => {
                    tracing::debug!("Successfully executed hook: {}", hook.name);
                }
                Err(e) => {
                    tracing::error!("Failed to execute hook {}: {}", hook.name, e);
                    // Continue with other hooks even if one fails
                }
            }
        }

        Ok(())
    }

    /// Queue an event for later processing with priority
    pub async fn queue_event(&self, event: HookEvent, priority: HookPriority) -> Result<()> {
        let mut queue = self.event_queue.lock().await;
        queue.push(QueuedEvent {
            event,
            priority,
            timestamp: chrono::Utc::now(),
        });
        Ok(())
    }

    /// Process all queued events in priority order
    pub async fn process_queue(&self) -> Result<()> {
        // Check if already processing
        let mut processing = self.processing.lock().await;
        if *processing {
            return Ok(());
        }
        *processing = true;
        drop(processing);

        loop {
            // Get next event from queue
            let queued_event = {
                let mut queue = self.event_queue.lock().await;
                queue.pop()
            };

            match queued_event {
                Some(queued) => {
                    if let Err(e) = self.handle_event(queued.event).await {
                        tracing::error!("Failed to process queued event: {}", e);
                    }
                }
                None => break,
            }
        }

        let mut processing = self.processing.lock().await;
        *processing = false;

        Ok(())
    }

    /// Process queue in background
    pub fn spawn_queue_processor(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                if let Err(e) = self.process_queue().await {
                    tracing::error!("Queue processor error: {}", e);
                }
            }
        });
    }

    /// Emit a consensus stage event
    pub async fn emit_consensus_event(&self, stage: &str, before: bool) -> Result<()> {
        let event_type = match (stage, before) {
            ("generator", true) => EventType::BeforeGeneratorStage,
            ("generator", false) => EventType::AfterGeneratorStage,
            ("refiner", true) => EventType::BeforeRefinerStage,
            ("refiner", false) => EventType::AfterRefinerStage,
            ("validator", true) => EventType::BeforeValidatorStage,
            ("validator", false) => EventType::AfterValidatorStage,
            ("curator", true) => EventType::BeforeCuratorStage,
            ("curator", false) => EventType::AfterCuratorStage,
            _ => return Ok(()),
        };

        let event = HookEvent::new(
            event_type,
            EventSource::Consensus {
                stage: stage.to_string(),
            },
        );

        self.handle_event(event).await
    }

    /// Emit a file modification event
    pub async fn emit_file_event(
        &self,
        path: PathBuf,
        operation: &str,
        before: bool,
    ) -> Result<()> {
        let event_type = match (operation, before) {
            ("write", true) => EventType::BeforeFileWrite,
            ("write", false) => EventType::AfterFileWrite,
            ("delete", true) => EventType::BeforeFileDelete,
            ("delete", false) => EventType::AfterFileDelete,
            ("modify", true) => EventType::BeforeCodeModification,
            ("modify", false) => EventType::AfterCodeModification,
            _ => return Ok(()),
        };

        let event = HookEvent::new(event_type, EventSource::FileSystem { path: path.clone() })
            .with_context("file_path", path.to_string_lossy());

        self.handle_event(event).await
    }

    /// Emit a cost control event
    pub async fn emit_cost_event(&self, cost: f64, threshold: f64, budget: f64) -> Result<()> {
        let event_type = if cost > budget {
            EventType::BudgetExceeded
        } else if cost > threshold {
            EventType::CostThresholdReached
        } else {
            EventType::CostEstimateAvailable
        };

        let event = HookEvent::new(event_type, EventSource::System)
            .with_context("estimated_cost", cost)
            .with_context("threshold", threshold)
            .with_context("budget", budget);

        self.handle_event(event).await
    }

    /// Emit a planning event
    pub async fn emit_planning_event(
        &self,
        event_type: &str,
        data: HashMap<String, Value>,
    ) -> Result<()> {
        let event_type = match event_type {
            "plan_created" => EventType::PlanCreated,
            "task_created" => EventType::TaskCreated,
            "task_completed" => EventType::TaskCompleted,
            "risk_identified" => EventType::RiskIdentified,
            "timeline_updated" => EventType::TimelineUpdated,
            "plan_execution_started" => EventType::PlanExecutionStarted,
            "plan_execution_completed" => EventType::PlanExecutionCompleted,
            _ => return Ok(()),
        };

        let mut event = HookEvent::new(event_type, EventSource::System);
        for (key, value) in data {
            event = event.with_context(key, value);
        }

        self.handle_event(event).await
    }

    /// Emit a memory event
    pub async fn emit_memory_event(
        &self,
        event_type: &str,
        data: HashMap<String, Value>,
    ) -> Result<()> {
        let event_type = match event_type {
            "conversation_stored" => EventType::ConversationStored,
            "pattern_detected" => EventType::PatternDetected,
            "memory_eviction" => EventType::MemoryEvictionOccurred,
            "thematic_cluster_created" => EventType::ThematicClusterCreated,
            "context_retrieved" => EventType::ContextRetrieved,
            _ => return Ok(()),
        };

        let mut event = HookEvent::new(event_type, EventSource::System);
        for (key, value) in data {
            event = event.with_context(key, value);
        }

        self.handle_event(event).await
    }

    /// Emit an analytics event
    pub async fn emit_analytics_event(
        &self,
        event_type: &str,
        data: HashMap<String, Value>,
    ) -> Result<()> {
        let event_type = match event_type {
            "threshold_exceeded" => EventType::ThresholdExceeded,
            "anomaly_detected" => EventType::AnomalyDetected,
            "report_generated" => EventType::ReportGenerated,
            "dashboard_updated" => EventType::DashboardUpdated,
            "metric_calculated" => EventType::MetricCalculated,
            _ => return Ok(()),
        };

        let mut event = HookEvent::new(event_type, EventSource::System);
        for (key, value) in data {
            event = event.with_context(key, value);
        }

        self.handle_event(event).await
    }
}

/// Builder for creating events
pub struct EventBuilder {
    event_type: EventType,
    source: EventSource,
    context: HashMap<String, Value>,
    metadata: EventMetadata,
}

impl EventBuilder {
    pub fn new(event_type: EventType, source: EventSource) -> Self {
        Self {
            event_type,
            source,
            context: HashMap::new(),
            metadata: EventMetadata::default(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.context.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }

    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(id.into());
        self
    }

    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.user_id = Some(id.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    pub fn build(self) -> HookEvent {
        HookEvent {
            event_type: self.event_type,
            timestamp: chrono::Utc::now(),
            source: self.source,
            context: self.context,
            metadata: self.metadata,
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let event = EventBuilder::new(
            EventType::BeforeCodeModification,
            EventSource::FileSystem {
                path: PathBuf::from("/test.rs"),
            },
        )
        .with_context("file_size", 1024)
        .with_context("language", "rust")
        .with_correlation_id("test-123")
        .build();

        assert_eq!(event.event_type, EventType::BeforeCodeModification);
        assert_eq!(
            event.context.get("file_size").unwrap(),
            &Value::Number(1024.into())
        );
        assert_eq!(event.metadata.correlation_id, Some("test-123".to_string()));
    }
}
