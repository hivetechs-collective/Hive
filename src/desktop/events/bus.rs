//! Event Bus Implementation
//!
//! Thread-safe publish-subscribe event system for desktop application

use super::types::{Event, EventType};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, trace};

/// Event handler function type
pub type EventHandler = Arc<dyn Fn(Event) -> Result<()> + Send + Sync>;

/// Async event handler function type
pub type AsyncEventHandler = Arc<dyn Fn(Event) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync>;

/// Event subscription handle for unsubscribing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionHandle(usize);

/// Event Bus for managing application events
pub struct EventBus {
    /// Synchronous event handlers
    subscribers: Arc<RwLock<HashMap<EventType, Vec<(SubscriptionHandle, EventHandler)>>>>,
    /// Asynchronous event handlers
    async_subscribers: Arc<RwLock<HashMap<EventType, Vec<(SubscriptionHandle, AsyncEventHandler)>>>>,
    /// Next subscription ID
    next_id: Arc<Mutex<usize>>,
}

impl EventBus {
    /// Create a new EventBus instance
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            async_subscribers: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Subscribe to an event type with a synchronous handler
    pub async fn subscribe<F>(&self, event_type: EventType, handler: F) -> SubscriptionHandle
    where
        F: Fn(Event) -> Result<()> + Send + Sync + 'static,
    {
        let mut id = self.next_id.lock().await;
        let handle = SubscriptionHandle(*id);
        *id += 1;

        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(event_type.clone())
            .or_insert_with(Vec::new)
            .push((handle.clone(), Arc::new(handler)));

        debug!("Subscribed handler {} to event {:?}", handle.0, event_type);
        handle
    }

    /// Subscribe to an event type with an asynchronous handler
    pub async fn subscribe_async<F, Fut>(&self, event_type: EventType, handler: F) -> SubscriptionHandle
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let mut id = self.next_id.lock().await;
        let handle = SubscriptionHandle(*id);
        *id += 1;

        let handler: AsyncEventHandler = Arc::new(move |event| Box::pin(handler(event)));

        let mut subscribers = self.async_subscribers.write().await;
        subscribers
            .entry(event_type.clone())
            .or_insert_with(Vec::new)
            .push((handle.clone(), handler));

        debug!("Subscribed async handler {} to event {:?}", handle.0, event_type);
        handle
    }

    /// Unsubscribe a handler
    pub async fn unsubscribe(&self, handle: &SubscriptionHandle) -> Result<()> {
        let mut removed = false;

        // Remove from sync subscribers
        {
            let mut subscribers = self.subscribers.write().await;
            for handlers in subscribers.values_mut() {
                handlers.retain(|(h, _)| {
                    if h == handle {
                        removed = true;
                        false
                    } else {
                        true
                    }
                });
            }
        }

        // Remove from async subscribers
        {
            let mut subscribers = self.async_subscribers.write().await;
            for handlers in subscribers.values_mut() {
                handlers.retain(|(h, _)| {
                    if h == handle {
                        removed = true;
                        false
                    } else {
                        true
                    }
                });
            }
        }

        if removed {
            debug!("Unsubscribed handler {}", handle.0);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Subscription handle not found"))
        }
    }

    /// Publish an event synchronously
    pub async fn publish(&self, event: Event) -> Result<()> {
        trace!("Publishing event: {:?}", event.event_type);

        // Call synchronous handlers
        {
            let subscribers = self.subscribers.read().await;
            if let Some(handlers) = subscribers.get(&event.event_type) {
                for (_, handler) in handlers {
                    if let Err(e) = handler(event.clone()) {
                        error!("Error in event handler: {}", e);
                    }
                }
            }
        }

        // Call async handlers synchronously (blocking)
        {
            let subscribers = self.async_subscribers.read().await;
            if let Some(handlers) = subscribers.get(&event.event_type) {
                for (_, handler) in handlers {
                    let future = handler(event.clone());
                    if let Err(e) = tokio::runtime::Handle::current().block_on(future) {
                        error!("Error in async event handler: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Publish an event asynchronously
    pub async fn publish_async(&self, event: Event) -> Result<()> {
        trace!("Publishing event async: {:?}", event.event_type);

        // Clone Arc references for concurrent execution
        let sync_subs = self.subscribers.clone();
        let async_subs = self.async_subscribers.clone();
        let event_clone = event.clone();

        // Spawn task for sync handlers
        let sync_handle = tokio::spawn(async move {
            let subscribers = sync_subs.read().await;
            if let Some(handlers) = subscribers.get(&event_clone.event_type) {
                for (_, handler) in handlers {
                    if let Err(e) = handler(event_clone.clone()) {
                        error!("Error in event handler: {}", e);
                    }
                }
            }
        });

        // Execute async handlers concurrently
        let async_handle = tokio::spawn(async move {
            let subscribers = async_subs.read().await;
            if let Some(handlers) = subscribers.get(&event.event_type) {
                let mut tasks = Vec::new();
                
                for (_, handler) in handlers {
                    let event_clone = event.clone();
                    let future = handler(event_clone);
                    tasks.push(tokio::spawn(async move {
                        if let Err(e) = future.await {
                            error!("Error in async event handler: {}", e);
                        }
                    }));
                }

                // Wait for all async handlers to complete
                for task in tasks {
                    let _ = task.await;
                }
            }
        });

        // Wait for both sync and async handlers to complete
        let _ = sync_handle.await;
        let _ = async_handle.await;

        Ok(())
    }

    /// Get the number of subscribers for a specific event type
    pub async fn subscriber_count(&self, event_type: &EventType) -> usize {
        let sync_count = self.subscribers.read().await
            .get(event_type)
            .map(|v| v.len())
            .unwrap_or(0);

        let async_count = self.async_subscribers.read().await
            .get(event_type)
            .map(|v| v.len())
            .unwrap_or(0);

        sync_count + async_count
    }

    /// Clear all subscriptions
    pub async fn clear(&self) {
        self.subscribers.write().await.clear();
        self.async_subscribers.write().await.clear();
        debug!("Cleared all event subscriptions");
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_sync_subscription() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(EventType::FileChanged, move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await;

        let event = Event {
            event_type: EventType::FileChanged,
            payload: EventPayload::FilePath("/test/file.rs".into()),
        };

        bus.publish(event).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_async_subscription() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe_async(EventType::BuildStarted, move |_event| {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .await;

        let event = Event {
            event_type: EventType::BuildStarted,
            payload: EventPayload::BuildInfo {
                target: "debug".to_string(),
                profile: "dev".to_string(),
            },
        };

        bus.publish_async(event).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let handle = bus
            .subscribe(EventType::FileChanged, move |_event| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await;

        let event = Event {
            event_type: EventType::FileChanged,
            payload: EventPayload::FilePath("/test/file.rs".into()),
        };

        bus.publish(event.clone()).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        bus.unsubscribe(&handle).await.unwrap();
        bus.publish(event).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Should still be 1
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let counter1_clone = counter1.clone();
        bus.subscribe(EventType::GitStatusChanged, move |_event| {
            counter1_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await;

        let counter2_clone = counter2.clone();
        bus.subscribe(EventType::GitStatusChanged, move |_event| {
            counter2_clone.fetch_add(2, Ordering::SeqCst);
            Ok(())
        })
        .await;

        let event = Event {
            event_type: EventType::GitStatusChanged,
            payload: EventPayload::GitStatus {
                branch: "main".to_string(),
                modified_files: vec!["/src/main.rs".to_string()],
                staged_files: vec![],
            },
        };

        bus.publish(event).await.unwrap();
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 2);
    }
}