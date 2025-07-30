# Event Bus System

The Event Bus provides a centralized publish-subscribe system for managing events across the Hive desktop application. It enables loose coupling between components while maintaining type safety and performance.

## Architecture

The event bus consists of several key components:

- **EventBus**: The core publish-subscribe mechanism with thread-safe access
- **EventType**: Enumeration of all possible event types in the application
- **EventPayload**: Type-safe payloads for each event type
- **Event**: Combination of type and payload
- **Global Instance**: Application-wide singleton accessible via `event_bus()`

## Features

- **Thread-Safe**: Uses `Arc<RwLock>` for safe concurrent access
- **Async Support**: Both sync and async handlers are supported
- **Type Safety**: Strongly typed events and payloads
- **Performance**: Minimal overhead with efficient routing
- **Error Handling**: Graceful error handling that doesn't stop event propagation
- **Unsubscribe**: Clean subscription management with handles

## Usage

### Publishing Events

```rust
use hive::desktop::events::{event_bus, Event, FileChangeType};

// Get the global event bus
let bus = event_bus();

// Create and publish an event
let event = Event::file_changed(
    "/src/main.rs".into(),
    FileChangeType::Modified
);
bus.publish_async(event).await?;
```

### Subscribing to Events

```rust
use hive::desktop::events::{event_bus, EventType, EventPayload};

let bus = event_bus();

// Synchronous handler
let handle = bus.subscribe(EventType::FileChanged, |event| {
    if let EventPayload::FileChange { path, change_type } = &event.payload {
        println!("File {:?} was {:?}", path, change_type);
    }
    Ok(())
}).await;

// Asynchronous handler
bus.subscribe_async(EventType::BuildStarted, |event| async move {
    // Async processing...
    Ok(())
}).await;

// Later, unsubscribe if needed
bus.unsubscribe(&handle).await?;
```

## Event Types

### File System Events
- `FileChanged`: File created, modified, deleted, or renamed
- `RepositoryChanged`: Active repository switched

### Build Events
- `BuildStarted`: Build process initiated
- `BuildCompleted`: Build finished (success or failure)

### Git Events
- `GitStatusChanged`: Repository status updated

### UI Events
- `ProblemsUpdated`: Diagnostics/problems changed
- `ConsensusStateChanged`: Consensus pipeline state update
- `UIStateChanged`: Theme, layout, or other UI changes

### System Events
- `ConfigurationChanged`: Application configuration modified
- `TerminalOutput`: Terminal emitted output
- `ExtensionEvent`: Extension-specific events

## Integration Examples

### Status Bar Integration

```rust
// In status_bar.rs
use hive::desktop::events::{event_bus, EventType};

pub async fn initialize_status_bar() {
    let bus = event_bus();
    
    // Show git branch
    bus.subscribe(EventType::GitStatusChanged, |event| {
        // Update git branch display
        Ok(())
    }).await;
    
    // Show build status
    bus.subscribe(EventType::BuildStarted, |event| {
        // Show building indicator
        Ok(())
    }).await;
}
```

### File Explorer Integration

```rust
// In file_explorer.rs
pub async fn setup_file_watcher() {
    let bus = event_bus();
    
    // React to file changes
    bus.subscribe_async(EventType::FileChanged, |event| async {
        // Update file tree
        Ok(())
    }).await;
}
```

### Problems Panel Integration

```rust
// In problems_panel.rs
pub async fn setup_problems_listener() {
    let bus = event_bus();
    
    bus.subscribe_async(EventType::ProblemsUpdated, |event| async {
        if let EventPayload::ProblemsUpdate { added, removed, total_count } = event.payload {
            // Update problems list UI
        }
        Ok(())
    }).await;
}
```

## Best Practices

1. **Use Async Publishing**: Prefer `publish_async()` for better performance
2. **Handle Errors**: Event handlers should handle errors gracefully
3. **Avoid Heavy Processing**: Keep handlers lightweight or use async
4. **Unsubscribe When Done**: Clean up subscriptions to prevent memory leaks
5. **Type Safety**: Use the provided helper methods on `Event` for creating events

## Performance Considerations

- Events are cloned when distributed to handlers
- Async handlers run concurrently when using `publish_async()`
- Sync handlers block until completion when using `publish()`
- Large payloads should be wrapped in `Arc` if needed

## Testing

The event bus includes comprehensive tests. Run them with:

```bash
cargo test event_bus
```

See `tests/event_bus_integration.rs` for integration test examples.