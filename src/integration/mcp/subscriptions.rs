//! MCP resource subscriptions
//!
//! Provides real-time updates for resources and analysis changes

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, debug, error, warn};
use notify::{Watcher, RecommendedWatcher, RecursiveMode, EventKind};
use std::path::{Path, PathBuf};
use tokio::time::{interval, Duration};

/// Subscription manager for MCP resources
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    clients: Arc<RwLock<HashMap<String, ClientSubscriptions>>>,
    file_watcher: Option<RecommendedWatcher>,
    event_sender: broadcast::Sender<SubscriptionEvent>,
    _event_receiver: broadcast::Receiver<SubscriptionEvent>,
}

/// Individual subscription
#[derive(Clone, Debug)]
pub struct Subscription {
    pub id: String,
    pub client_id: String,
    pub resource_type: ResourceType,
    pub resource_path: String,
    pub filters: SubscriptionFilters,
    pub created_at: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub update_count: u64,
    pub status: SubscriptionStatus,
}

/// Client subscription tracking
#[derive(Clone, Debug)]
pub struct ClientSubscriptions {
    pub client_id: String,
    pub subscription_ids: HashSet<String>,
    pub event_sender: mpsc::UnboundedSender<SubscriptionEvent>,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Resource type for subscriptions
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    File,
    Directory,
    Analysis,
    Memory,
    Configuration,
    Repository,
    Custom(String),
}

/// Subscription filters
#[derive(Clone, Debug, Default)]
pub struct SubscriptionFilters {
    pub file_extensions: Option<HashSet<String>>,
    pub path_patterns: Option<Vec<String>>,
    pub event_types: Option<HashSet<SubscriptionEventType>>,
    pub min_file_size: Option<u64>,
    pub max_file_size: Option<u64>,
    pub debounce_ms: Option<u64>,
}

/// Subscription status
#[derive(Clone, Debug, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Error(String),
}

/// Subscription event types
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriptionEventType {
    FileCreated,
    FileModified,
    FileDeleted,
    FileRenamed,
    DirectoryCreated,
    DirectoryDeleted,
    AnalysisUpdated,
    MemoryUpdated,
    ConfigurationChanged,
    RepositoryChanged,
    Custom(String),
}

/// Subscription event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionEvent {
    pub id: String,
    pub subscription_id: String,
    pub resource_type: ResourceType,
    pub event_type: SubscriptionEventType,
    pub resource_path: String,
    pub timestamp: DateTime<Utc>,
    pub data: Option<serde_json::Value>,
    pub metadata: SubscriptionEventMetadata,
}

/// Event metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionEventMetadata {
    pub file_size: Option<u64>,
    pub file_extension: Option<String>,
    pub checksum: Option<String>,
    pub previous_checksum: Option<String>,
    pub change_type: Option<String>,
    pub affected_lines: Option<Vec<u32>>,
}

/// Subscription request
#[derive(Clone, Debug, Deserialize)]
pub struct SubscriptionRequest {
    pub resource_type: ResourceType,
    pub resource_path: String,
    pub filters: Option<SubscriptionFilters>,
    pub client_id: String,
}

/// Subscription response
#[derive(Clone, Debug, Serialize)]
pub struct SubscriptionResponse {
    pub subscription_id: String,
    pub status: String,
    pub message: String,
}

impl SubscriptionManager {
    /// Create new subscription manager
    pub fn new() -> Result<Self> {
        let (event_sender, event_receiver) = broadcast::channel(1000);
        
        Ok(Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            file_watcher: None,
            event_sender,
            _event_receiver: event_receiver,
        })
    }

    /// Start the subscription manager
    pub async fn start(&mut self) -> Result<()> {
        // Initialize file watcher
        self.setup_file_watcher().await?;
        
        // Start cleanup task
        self.start_cleanup_task().await;
        
        // Start health check task
        self.start_health_check_task().await;

        info!("Subscription manager started successfully");
        Ok(())
    }

    /// Setup file system watcher
    async fn setup_file_watcher(&mut self) -> Result<()> {
        let event_sender = self.event_sender.clone();
        let subscriptions = self.subscriptions.clone();

        let (tx, mut rx) = mpsc::channel(100);
        
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            match res {
                Ok(event) => {
                    if let Err(e) = tx.blocking_send(event) {
                        error!("Failed to send file system event: {}", e);
                    }
                }
                Err(e) => error!("File system watch error: {}", e),
            }
        })?;

        // Start event processing task
        let subscriptions_clone = subscriptions.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = Self::process_file_system_event(
                    event,
                    &event_sender,
                    &subscriptions_clone,
                ).await {
                    error!("Failed to process file system event: {}", e);
                }
            }
        });

        self.file_watcher = Some(watcher);
        Ok(())
    }

    /// Process file system event
    async fn process_file_system_event(
        event: notify::Event,
        event_sender: &broadcast::Sender<SubscriptionEvent>,
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
    ) -> Result<()> {
        let subscriptions_read = subscriptions.read().await;
        
        for path in &event.paths {
            for subscription in subscriptions_read.values() {
                if Self::matches_subscription(&event, path, subscription) {
                    let subscription_event = Self::create_subscription_event(
                        &event,
                        path,
                        subscription,
                    ).await?;

                    if let Err(e) = event_sender.send(subscription_event) {
                        debug!("No receivers for subscription event: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if event matches subscription
    fn matches_subscription(
        event: &notify::Event,
        path: &Path,
        subscription: &Subscription,
    ) -> bool {
        // Check if path matches subscription path
        let path_str = path.to_string_lossy();
        let subscription_path = Path::new(&subscription.resource_path);

        let path_matches = if subscription_path.is_dir() {
            path.starts_with(subscription_path)
        } else {
            path == subscription_path
        };

        if !path_matches {
            return false;
        }

        // Check file extension filter
        if let Some(ref extensions) = subscription.filters.file_extensions {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !extensions.contains(ext) {
                    return false;
                }
            }
        }

        // Check event type filter
        if let Some(ref event_types) = subscription.filters.event_types {
            let event_type = Self::map_notify_event_to_subscription_event(&event.kind);
            if !event_types.contains(&event_type) {
                return false;
            }
        }

        // Check path patterns
        if let Some(ref patterns) = subscription.filters.path_patterns {
            let path_matches_pattern = patterns.iter().any(|pattern| {
                // Simple pattern matching - in production, use a proper glob library
                path_str.contains(pattern) || 
                pattern.contains('*') && Self::matches_glob_pattern(&path_str, pattern)
            });
            
            if !path_matches_pattern {
                return false;
            }
        }

        true
    }

    /// Simple glob pattern matching
    fn matches_glob_pattern(path: &str, pattern: &str) -> bool {
        // Simplified glob matching - replace with proper implementation
        let pattern_regex = pattern
            .replace('.', r"\.")
            .replace('*', ".*")
            .replace('?', ".");
        
        regex::Regex::new(&format!("^{}$", pattern_regex))
            .map(|re| re.is_match(path))
            .unwrap_or(false)
    }

    /// Map notify event to subscription event type
    fn map_notify_event_to_subscription_event(kind: &EventKind) -> SubscriptionEventType {
        match kind {
            EventKind::Create(_) => SubscriptionEventType::FileCreated,
            EventKind::Modify(_) => SubscriptionEventType::FileModified,
            EventKind::Remove(_) => SubscriptionEventType::FileDeleted,
            _ => SubscriptionEventType::FileModified,
        }
    }

    /// Create subscription event from notify event
    async fn create_subscription_event(
        notify_event: &notify::Event,
        path: &Path,
        subscription: &Subscription,
    ) -> Result<SubscriptionEvent> {
        let event_type = Self::map_notify_event_to_subscription_event(&notify_event.kind);
        
        // Get file metadata
        let metadata = if path.exists() {
            let file_metadata = tokio::fs::metadata(path).await.ok();
            SubscriptionEventMetadata {
                file_size: file_metadata.as_ref().map(|m| m.len()),
                file_extension: path.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string()),
                checksum: Self::calculate_file_checksum(path).await.ok(),
                previous_checksum: None, // TODO: Track previous checksums
                change_type: Some(format!("{:?}", notify_event.kind)),
                affected_lines: None, // TODO: Implement line-level change detection
            }
        } else {
            SubscriptionEventMetadata {
                file_size: None,
                file_extension: path.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string()),
                checksum: None,
                previous_checksum: None,
                change_type: Some(format!("{:?}", notify_event.kind)),
                affected_lines: None,
            }
        };

        Ok(SubscriptionEvent {
            id: Uuid::new_v4().to_string(),
            subscription_id: subscription.id.clone(),
            resource_type: subscription.resource_type.clone(),
            event_type,
            resource_path: path.to_string_lossy().to_string(),
            timestamp: Utc::now(),
            data: None, // TODO: Include relevant data based on event type
            metadata,
        })
    }

    /// Calculate file checksum
    async fn calculate_file_checksum(path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Register client
    pub async fn register_client(
        &self,
        client_id: String,
    ) -> Result<mpsc::UnboundedReceiver<SubscriptionEvent>> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let client_subscriptions = ClientSubscriptions {
            client_id: client_id.clone(),
            subscription_ids: HashSet::new(),
            event_sender,
            connected_at: Utc::now(),
            last_activity: Utc::now(),
        };

        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id.clone(), client_subscriptions);
        }

        info!("Registered MCP client for subscriptions: {}", client_id);
        Ok(event_receiver)
    }

    /// Create subscription
    pub async fn create_subscription(
        &mut self,
        request: SubscriptionRequest,
    ) -> Result<SubscriptionResponse> {
        let subscription_id = Uuid::new_v4().to_string();
        
        let subscription = Subscription {
            id: subscription_id.clone(),
            client_id: request.client_id.clone(),
            resource_type: request.resource_type.clone(),
            resource_path: request.resource_path.clone(),
            filters: request.filters.unwrap_or_default(),
            created_at: Utc::now(),
            last_update: Utc::now(),
            update_count: 0,
            status: SubscriptionStatus::Active,
        };

        // Add to subscriptions
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        // Update client subscription list
        {
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(&request.client_id) {
                client.subscription_ids.insert(subscription_id.clone());
                client.last_activity = Utc::now();
            }
        }

        // Setup file system watching if needed
        if matches!(request.resource_type, ResourceType::File | ResourceType::Directory) {
            if let Some(ref mut watcher) = self.file_watcher {
                let path = Path::new(&request.resource_path);
                let mode = if path.is_dir() {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };

                if let Err(e) = watcher.watch(path, mode) {
                    warn!("Failed to watch path {}: {}", request.resource_path, e);
                }
            }
        }

        info!(
            "Created subscription {} for client {} on {:?}:{}",
            subscription_id, request.client_id, request.resource_type, request.resource_path
        );

        Ok(SubscriptionResponse {
            subscription_id,
            status: "created".to_string(),
            message: "Subscription created successfully".to_string(),
        })
    }

    /// Cancel subscription
    pub async fn cancel_subscription(
        &self,
        subscription_id: &str,
        client_id: &str,
    ) -> Result<()> {
        // Remove from subscriptions
        let subscription = {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(subscription_id)
        };

        if let Some(subscription) = subscription {
            // Verify client ownership
            if subscription.client_id != client_id {
                return Err(anyhow!("Client does not own this subscription"));
            }

            // Update client subscription list
            {
                let mut clients = self.clients.write().await;
                if let Some(client) = clients.get_mut(client_id) {
                    client.subscription_ids.remove(subscription_id);
                    client.last_activity = Utc::now();
                }
            }

            info!("Cancelled subscription {} for client {}", subscription_id, client_id);
        } else {
            warn!("Attempted to cancel non-existent subscription: {}", subscription_id);
        }

        Ok(())
    }

    /// List subscriptions for client
    pub async fn list_client_subscriptions(&self, client_id: &str) -> Vec<Subscription> {
        let subscriptions = self.subscriptions.read().await;
        
        subscriptions
            .values()
            .filter(|s| s.client_id == client_id)
            .cloned()
            .collect()
    }

    /// Trigger manual event
    pub async fn trigger_event(&self, event: SubscriptionEvent) -> Result<()> {
        if let Err(e) = self.event_sender.send(event.clone()) {
            debug!("No receivers for manual event: {}", e);
        }

        // Send directly to client if available
        {
            let clients = self.clients.read().await;
            let subscriptions = self.subscriptions.read().await;
            
            if let Some(subscription) = subscriptions.get(&event.subscription_id) {
                if let Some(client) = clients.get(&subscription.client_id) {
                    if let Err(e) = client.event_sender.send(event) {
                        debug!("Failed to send event to client: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Start cleanup task
    async fn start_cleanup_task(&self) {
        let clients = self.clients.clone();
        let subscriptions = self.subscriptions.clone();

        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(3600)); // Every hour
            
            loop {
                cleanup_interval.tick().await;
                
                if let Err(e) = Self::cleanup_inactive_clients(&clients, &subscriptions).await {
                    error!("Failed to cleanup inactive clients: {}", e);
                }
            }
        });
    }

    /// Cleanup inactive clients
    async fn cleanup_inactive_clients(
        clients: &Arc<RwLock<HashMap<String, ClientSubscriptions>>>,
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
    ) -> Result<()> {
        let inactive_threshold = Utc::now() - chrono::Duration::hours(24);
        let mut clients_to_remove = Vec::new();
        let mut subscriptions_to_remove = Vec::new();

        // Find inactive clients
        {
            let clients_read = clients.read().await;
            for (client_id, client) in clients_read.iter() {
                if client.last_activity < inactive_threshold {
                    clients_to_remove.push(client_id.clone());
                    subscriptions_to_remove.extend(client.subscription_ids.iter().cloned());
                }
            }
        }

        // Remove inactive clients and their subscriptions
        if !clients_to_remove.is_empty() {
            {
                let mut clients_write = clients.write().await;
                for client_id in &clients_to_remove {
                    clients_write.remove(client_id);
                }
            }

            {
                let mut subscriptions_write = subscriptions.write().await;
                for subscription_id in &subscriptions_to_remove {
                    subscriptions_write.remove(subscription_id);
                }
            }

            info!(
                "Cleaned up {} inactive clients and {} subscriptions",
                clients_to_remove.len(),
                subscriptions_to_remove.len()
            );
        }

        Ok(())
    }

    /// Start health check task
    async fn start_health_check_task(&self) {
        let clients = self.clients.clone();
        
        tokio::spawn(async move {
            let mut health_interval = interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                health_interval.tick().await;
                
                let client_count = clients.read().await.len();
                debug!("Subscription manager health check: {} active clients", client_count);
            }
        });
    }

    /// Get subscription statistics
    pub async fn get_statistics(&self) -> SubscriptionStatistics {
        let subscriptions = self.subscriptions.read().await;
        let clients = self.clients.read().await;

        let mut stats = SubscriptionStatistics {
            total_subscriptions: subscriptions.len(),
            active_subscriptions: 0,
            paused_subscriptions: 0,
            error_subscriptions: 0,
            total_clients: clients.len(),
            subscription_types: HashMap::new(),
            average_subscriptions_per_client: 0.0,
        };

        // Count subscription statuses and types
        for subscription in subscriptions.values() {
            match subscription.status {
                SubscriptionStatus::Active => stats.active_subscriptions += 1,
                SubscriptionStatus::Paused => stats.paused_subscriptions += 1,
                SubscriptionStatus::Error(_) => stats.error_subscriptions += 1,
            }

            let type_key = format!("{:?}", subscription.resource_type);
            *stats.subscription_types.entry(type_key).or_insert(0) += 1;
        }

        // Calculate average subscriptions per client
        if stats.total_clients > 0 {
            stats.average_subscriptions_per_client = 
                stats.total_subscriptions as f64 / stats.total_clients as f64;
        }

        stats
    }
}

/// Subscription statistics
#[derive(Debug, Serialize)]
pub struct SubscriptionStatistics {
    pub total_subscriptions: usize,
    pub active_subscriptions: usize,
    pub paused_subscriptions: usize,
    pub error_subscriptions: usize,
    pub total_clients: usize,
    pub subscription_types: HashMap<String, usize>,
    pub average_subscriptions_per_client: f64,
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default SubscriptionManager")
    }
}