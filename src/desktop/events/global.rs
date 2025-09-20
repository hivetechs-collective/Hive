//! Global Event Bus Instance
//!
//! Provides a singleton event bus instance for application-wide event handling

use super::bus::EventBus;
use once_cell::sync::Lazy;
use std::sync::Arc;

/// Global event bus instance
pub static GLOBAL_EVENT_BUS: Lazy<Arc<EventBus>> = Lazy::new(|| {
    tracing::info!("Initializing global event bus");
    Arc::new(EventBus::new())
});

/// Get a reference to the global event bus
pub fn event_bus() -> Arc<EventBus> {
    GLOBAL_EVENT_BUS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::events::{Event, EventType};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_global_event_bus() {
        let bus = event_bus();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(EventType::FileChanged, move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await;

        // Get another reference to the global bus
        let bus2 = event_bus();
        let event = Event::empty(EventType::FileChanged);

        // Publish through the second reference
        bus2.publish(event).await.unwrap();

        // Should have been received by the subscriber
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
