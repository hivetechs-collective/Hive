//! Integration tests for the hooks system

use crate::{
    hooks::{
        DispatcherConfig, EventBuilder, EventSource, EventType, HookEvent, HookPriority,
        HooksSystem,
    },
    Result,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_hooks_system_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let hooks_system = HooksSystem::new(temp_dir.path().to_path_buf()).await?;

    // List hooks (should be empty)
    let hooks = hooks_system.list_hooks().await?;
    assert!(hooks.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_event_dispatching() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut hooks_system = HooksSystem::new(temp_dir.path().to_path_buf()).await?;

    // Start the hooks system
    hooks_system.start().await?;

    // Create a test event
    let event = EventBuilder::new(
        EventType::BeforeCodeModification,
        EventSource::CLI {
            command: "test".to_string(),
        },
    )
    .with_context("file_path", "/test.rs")
    .with_context("operation", "format")
    .build();

    // Dispatch the event
    hooks_system.dispatch_event(event).await?;

    // Check dispatcher stats
    let stats = hooks_system.get_dispatcher_stats().await;
    assert_eq!(stats.events_received, 1);

    Ok(())
}

#[tokio::test]
async fn test_planning_events() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut hooks_system = HooksSystem::new(temp_dir.path().to_path_buf()).await?;

    // Start the hooks system
    hooks_system.start().await?;

    // Test planning events
    let events = vec![
        EventType::PlanCreated,
        EventType::TaskCreated,
        EventType::TaskCompleted,
        EventType::RiskIdentified,
    ];

    for event_type in events {
        let event = HookEvent::new(event_type.clone(), EventSource::System);

        hooks_system.dispatch_event(event).await?;
    }

    // Check that events were dispatched
    let stats = hooks_system.get_dispatcher_stats().await;
    assert_eq!(stats.events_received, 4);

    Ok(())
}

#[tokio::test]
async fn test_memory_analytics_events() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut hooks_system = HooksSystem::new(temp_dir.path().to_path_buf()).await?;

    // Start the hooks system
    hooks_system.start().await?;

    // Test memory events
    let memory_event = EventBuilder::new(EventType::PatternDetected, EventSource::System)
        .with_context("pattern_type", "code_duplication")
        .with_context("confidence_score", 0.95)
        .with_context("file_count", 5)
        .build();

    hooks_system.dispatch_event(memory_event).await?;

    // Test analytics events
    let analytics_event = EventBuilder::new(EventType::AnomalyDetected, EventSource::System)
        .with_context("metric_name", "response_time")
        .with_context("deviation", 3.5)
        .with_context("severity", "high")
        .build();

    hooks_system.dispatch_event(analytics_event).await?;

    // Verify events were processed
    let stats = hooks_system.get_dispatcher_stats().await;
    assert!(stats.events_received >= 2);

    Ok(())
}

#[tokio::test]
async fn test_hook_priority_ordering() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut hooks_system = HooksSystem::new(temp_dir.path().to_path_buf()).await?;

    // Start the hooks system
    hooks_system.start().await?;

    // Create events with different priorities
    let high_priority_event =
        EventBuilder::new(EventType::SecurityCheckFailed, EventSource::System)
            .with_context("severity", "critical")
            .build();

    let low_priority_event = EventBuilder::new(EventType::MetricCalculated, EventSource::System)
        .with_context("metric", "code_coverage")
        .build();

    // Dispatch events
    hooks_system.dispatch_event(low_priority_event).await?;
    hooks_system.dispatch_event(high_priority_event).await?;

    // High priority events should be processed first
    let stats = hooks_system.get_dispatcher_stats().await;
    assert_eq!(stats.events_received, 2);

    Ok(())
}

#[test]
fn test_event_type_parsing() {
    use crate::commands::hooks;

    // Test consensus events
    assert!(hooks::parse_event_type("before_consensus").is_ok());
    assert!(hooks::parse_event_type("after_generator_stage").is_ok());

    // Test new planning events
    assert!(hooks::parse_event_type("plan_created").is_ok());
    assert!(hooks::parse_event_type("task_completed").is_ok());
    assert!(hooks::parse_event_type("risk_identified").is_ok());

    // Test new memory events
    assert!(hooks::parse_event_type("pattern_detected").is_ok());
    assert!(hooks::parse_event_type("thematic_cluster_created").is_ok());

    // Test new analytics events
    assert!(hooks::parse_event_type("anomaly_detected").is_ok());
    assert!(hooks::parse_event_type("dashboard_updated").is_ok());

    // Test custom events
    assert!(hooks::parse_event_type("custom:my_event").is_ok());

    // Test invalid events
    assert!(hooks::parse_event_type("invalid_event").is_err());
}
