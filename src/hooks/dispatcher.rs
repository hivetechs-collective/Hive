//! Event Dispatcher - Advanced event routing and processing with priority queues

use super::{EventHandler, EventType, HookEvent, HookPriority};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

/// Event dispatcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatcherConfig {
    /// Maximum queue size before dropping events
    pub max_queue_size: usize,
    /// Number of worker threads for processing
    pub worker_threads: usize,
    /// Queue processing interval in milliseconds
    pub processing_interval_ms: u64,
    /// Enable event batching
    pub enable_batching: bool,
    /// Batch size for processing
    pub batch_size: usize,
    /// Event TTL in seconds
    pub event_ttl_seconds: u64,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            worker_threads: 4,
            processing_interval_ms: 100,
            enable_batching: true,
            batch_size: 50,
            event_ttl_seconds: 300,
        }
    }
}

/// Event routing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Event types to match
    pub event_types: Vec<EventType>,
    /// Priority override
    pub priority_override: Option<HookPriority>,
    /// Target queue name
    pub target_queue: Option<String>,
    /// Rate limit per minute
    pub rate_limit: Option<u32>,
}

/// Event dispatcher with advanced routing
pub struct EventDispatcher {
    config: DispatcherConfig,
    event_handler: Arc<EventHandler>,
    queues: Arc<RwLock<HashMap<String, Arc<Mutex<BinaryHeap<PrioritizedEvent>>>>>>,
    routing_rules: Arc<RwLock<Vec<RoutingRule>>>,
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
    stats: Arc<RwLock<DispatcherStats>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

#[derive(Debug, Clone)]
struct PrioritizedEvent {
    event: HookEvent,
    priority: HookPriority,
    timestamp: chrono::DateTime<chrono::Utc>,
    ttl: chrono::DateTime<chrono::Utc>,
    queue_name: String,
}

impl Eq for PrioritizedEvent {}

impl PartialEq for PrioritizedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Ord for PrioritizedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => other.timestamp.cmp(&self.timestamp),
            ordering => ordering,
        }
    }
}

impl PartialOrd for PrioritizedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Rate limiter for events
struct RateLimiter {
    tokens: u32,
    max_tokens: u32,
    last_refill: chrono::DateTime<chrono::Utc>,
}

impl RateLimiter {
    fn new(max_tokens: u32) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            last_refill: chrono::Utc::now(),
        }
    }

    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(self.last_refill);
        if elapsed.num_seconds() >= 60 {
            self.tokens = self.max_tokens;
            self.last_refill = now;
        }
    }
}

/// Dispatcher statistics
#[derive(Debug, Default, Clone, Serialize)]
pub struct DispatcherStats {
    pub events_received: u64,
    pub events_processed: u64,
    pub events_dropped: u64,
    pub events_expired: u64,
    pub queue_sizes: HashMap<String, usize>,
    pub processing_times: HashMap<String, f64>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new(config: DispatcherConfig, event_handler: Arc<EventHandler>) -> Self {
        Self {
            config,
            event_handler,
            queues: Arc::new(RwLock::new(HashMap::new())),
            routing_rules: Arc::new(RwLock::new(Vec::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(DispatcherStats::default())),
            shutdown_tx: None,
        }
    }

    /// Start the dispatcher workers
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Create default queue
        let mut queues = self.queues.write().await;
        queues.insert(
            "default".to_string(),
            Arc::new(Mutex::new(BinaryHeap::new())),
        );
        drop(queues);

        // Spawn worker threads
        for i in 0..self.config.worker_threads {
            let queues = self.queues.clone();
            let event_handler = self.event_handler.clone();
            let stats = self.stats.clone();
            let config = self.config.clone();

            tokio::spawn(async move {
                loop {
                    // Process events from all queues
                    let queue_names: Vec<String> = {
                        let queues = queues.read().await;
                        queues.keys().cloned().collect()
                    };

                    for queue_name in queue_names {
                        if let Some(queue) = queues.read().await.get(&queue_name) {
                            let events = {
                                let mut q = queue.lock().await;
                                let mut batch = Vec::new();

                                // Collect batch of events
                                while batch.len() < config.batch_size && !q.is_empty() {
                                    if let Some(event) = q.pop() {
                                        // Check TTL
                                        if event.ttl > chrono::Utc::now() {
                                            batch.push(event);
                                        } else {
                                            let mut stats = stats.write().await;
                                            stats.events_expired += 1;
                                        }
                                    }
                                }

                                batch
                            };

                            // Process batch
                            for prioritized_event in events {
                                let start = std::time::Instant::now();

                                if let Err(e) =
                                    event_handler.handle_event(prioritized_event.event).await
                                {
                                    tracing::error!("Worker {} failed to process event: {}", i, e);
                                } else {
                                    let mut stats = stats.write().await;
                                    stats.events_processed += 1;

                                    let duration = start.elapsed().as_secs_f64();
                                    stats
                                        .processing_times
                                        .entry(queue_name.clone())
                                        .and_modify(|t| *t = (*t + duration) / 2.0)
                                        .or_insert(duration);
                                }
                            }
                        }
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        config.processing_interval_ms,
                    ))
                    .await;
                }
            });
        }

        Ok(())
    }

    /// Stop the dispatcher
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        Ok(())
    }

    /// Dispatch an event
    pub async fn dispatch(&self, event: HookEvent) -> Result<()> {
        let mut stats = self.stats.write().await;
        stats.events_received += 1;
        drop(stats);

        // Determine routing
        let (queue_name, priority) = self.route_event(&event).await?;

        // Check rate limit
        if !self.check_rate_limit(&event).await? {
            let mut stats = self.stats.write().await;
            stats.events_dropped += 1;
            return Err(anyhow!("Rate limit exceeded for event type"));
        }

        // Get or create queue
        let queue = {
            let mut queues = self.queues.write().await;
            queues
                .entry(queue_name.clone())
                .or_insert_with(|| Arc::new(Mutex::new(BinaryHeap::new())))
                .clone()
        };

        // Add to queue
        let mut q = queue.lock().await;

        // Check queue size
        if q.len() >= self.config.max_queue_size {
            let mut stats = self.stats.write().await;
            stats.events_dropped += 1;
            return Err(anyhow!("Queue {} is full", queue_name));
        }

        let prioritized_event = PrioritizedEvent {
            event,
            priority,
            timestamp: chrono::Utc::now(),
            ttl: chrono::Utc::now()
                + chrono::Duration::seconds(self.config.event_ttl_seconds as i64),
            queue_name: queue_name.clone(),
        };

        q.push(prioritized_event);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.queue_sizes.insert(queue_name, q.len());

        Ok(())
    }

    /// Route event based on rules
    async fn route_event(&self, event: &HookEvent) -> Result<(String, HookPriority)> {
        let rules = self.routing_rules.read().await;

        for rule in rules.iter() {
            if rule.event_types.contains(&event.event_type) {
                let queue = rule
                    .target_queue
                    .clone()
                    .unwrap_or_else(|| "default".to_string());
                let priority = rule.priority_override.unwrap_or(HookPriority::Normal);
                return Ok((queue, priority));
            }
        }

        // Default routing
        Ok(("default".to_string(), HookPriority::Normal))
    }

    /// Check rate limit for event
    async fn check_rate_limit(&self, event: &HookEvent) -> Result<bool> {
        let rules = self.routing_rules.read().await;

        for rule in rules.iter() {
            if rule.event_types.contains(&event.event_type) {
                if let Some(limit) = rule.rate_limit {
                    let key = format!("{:?}", event.event_type);
                    let mut limiters = self.rate_limiters.write().await;

                    let limiter = limiters
                        .entry(key)
                        .or_insert_with(|| RateLimiter::new(limit));

                    return Ok(limiter.try_consume());
                }
            }
        }

        Ok(true)
    }

    /// Add a routing rule
    pub async fn add_routing_rule(&self, rule: RoutingRule) -> Result<()> {
        let mut rules = self.routing_rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// Get dispatcher statistics
    pub async fn get_stats(&self) -> DispatcherStats {
        self.stats.read().await.clone()
    }

    /// Clear all queues
    pub async fn clear_queues(&self) -> Result<()> {
        let queues = self.queues.read().await;
        for queue in queues.values() {
            let mut q = queue.lock().await;
            q.clear();
        }

        let mut stats = self.stats.write().await;
        stats.queue_sizes.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_dispatcher() {
        // Test will be implemented
    }
}
