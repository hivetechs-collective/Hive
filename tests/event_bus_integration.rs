//! Integration tests for the Event Bus system

use hive::desktop::events::{
    event_bus, Event, EventPayload, EventType, FileChangeType, ConsensusStage,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_event_bus_file_system_integration() {
    let bus = event_bus();
    let file_event_count = Arc::new(AtomicUsize::new(0));
    let count_clone = file_event_count.clone();

    // Subscribe to file change events
    bus.subscribe(EventType::FileChanged, move |event| {
        if let EventPayload::FileChange { path, change_type } = &event.payload {
            println!("File changed: {:?} - {:?}", path, change_type);
            count_clone.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    })
    .await;

    // Simulate file system events
    let events = vec![
        Event::file_changed("/src/main.rs".into(), FileChangeType::Modified),
        Event::file_changed("/src/lib.rs".into(), FileChangeType::Created),
        Event::file_changed(
            "/src/old.rs".into(),
            FileChangeType::Renamed {
                old_path: "/src/legacy.rs".into(),
            },
        ),
    ];

    for event in events {
        bus.publish_async(event).await.unwrap();
    }

    sleep(Duration::from_millis(100)).await;
    assert_eq!(file_event_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_event_bus_build_system_integration() {
    let bus = event_bus();
    let build_states = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let states_clone = build_states.clone();

    // Subscribe to build events
    bus.subscribe_async(EventType::BuildStarted, {
        let states = states_clone.clone();
        move |event| {
            let states = states.clone();
            async move {
                if let EventPayload::BuildInfo { target, profile } = &event.payload {
                    states.lock().await.push(format!("started: {} {}", target, profile));
                }
                Ok(())
            }
        }
    })
    .await;

    bus.subscribe_async(EventType::BuildCompleted, {
        let states = states_clone.clone();
        move |event| {
            let states = states.clone();
            async move {
                if let EventPayload::BuildResult { success, .. } = &event.payload {
                    states.lock().await.push(format!("completed: {}", success));
                }
                Ok(())
            }
        }
    })
    .await;

    // Simulate build process
    bus.publish_async(Event::build_started("debug".into(), "dev".into()))
        .await
        .unwrap();

    sleep(Duration::from_millis(50)).await;

    bus.publish_async(Event::build_completed(true, vec![], vec![], 1500))
        .await
        .unwrap();

    sleep(Duration::from_millis(100)).await;

    let states = build_states.lock().await;
    assert_eq!(states.len(), 2);
    assert_eq!(states[0], "started: debug dev");
    assert_eq!(states[1], "completed: true");
}

#[tokio::test]
async fn test_event_bus_consensus_integration() {
    let bus = event_bus();
    let consensus_stages = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let stages_clone = consensus_stages.clone();

    // Subscribe to consensus state changes
    let handle = bus
        .subscribe_async(EventType::ConsensusStateChanged, move |event| {
            let stages = stages_clone.clone();
            async move {
                if let EventPayload::ConsensusState {
                    stage, progress, ..
                } = &event.payload
                {
                    stages
                        .lock()
                        .await
                        .push((stage.clone(), *progress));
                }
                Ok(())
            }
        })
        .await;

    // Simulate consensus pipeline
    let pipeline_stages = vec![
        (ConsensusStage::Initializing, 0),
        (ConsensusStage::Generating, 25),
        (ConsensusStage::Generating, 50),
        (ConsensusStage::Refining, 60),
        (ConsensusStage::Validating, 80),
        (ConsensusStage::Curating, 90),
        (ConsensusStage::Completed, 100),
    ];

    for (stage, progress) in pipeline_stages {
        let event = Event::consensus_state_changed(stage, progress, None);
        bus.publish_async(event).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }

    sleep(Duration::from_millis(100)).await;

    let stages = consensus_stages.lock().await;
    assert_eq!(stages.len(), 7);
    assert_eq!(stages[0].0, ConsensusStage::Initializing);
    assert_eq!(stages[6].0, ConsensusStage::Completed);
    assert_eq!(stages[6].1, 100);

    // Test unsubscribe
    bus.unsubscribe(&handle).await.unwrap();
    
    // This event should not be received
    bus.publish_async(Event::consensus_state_changed(
        ConsensusStage::Failed,
        0,
        Some("Test error".into()),
    ))
    .await
    .unwrap();
    
    sleep(Duration::from_millis(50)).await;
    assert_eq!(consensus_stages.lock().await.len(), 7); // Still 7, not 8
}

#[tokio::test]
async fn test_event_bus_cross_component_communication() {
    let bus = event_bus();
    let message_flow = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Component A: File watcher
    let flow_a = message_flow.clone();
    tokio::spawn(async move {
        let bus = event_bus();
        sleep(Duration::from_millis(10)).await;
        flow_a.lock().await.push("A: Detected file change");
        bus.publish_async(Event::file_changed(
            "/src/main.rs".into(),
            FileChangeType::Modified,
        ))
        .await
        .unwrap();
    });

    // Component B: Build system reacting to file changes
    let flow_b = message_flow.clone();
    bus.subscribe_async(EventType::FileChanged, move |_event| {
        let flow = flow_b.clone();
        let bus = event_bus();
        async move {
            flow.lock().await.push("B: Received file change, starting build");
            bus.publish_async(Event::build_started("debug".into(), "dev".into()))
                .await?;
            Ok(())
        }
    })
    .await;

    // Component C: Status bar showing build status
    let flow_c = message_flow.clone();
    bus.subscribe_async(EventType::BuildStarted, move |_event| {
        let flow = flow_c.clone();
        async move {
            flow.lock().await.push("C: Showing build status in status bar");
            Ok(())
        }
    })
    .await;

    // Let the events flow
    sleep(Duration::from_millis(200)).await;

    let flow = message_flow.lock().await;
    assert_eq!(flow.len(), 3);
    assert!(flow[0].contains("A:"));
    assert!(flow[1].contains("B:"));
    assert!(flow[2].contains("C:"));
}

#[tokio::test]
async fn test_event_bus_error_handling() {
    let bus = event_bus();
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    let success_clone = success_count.clone();
    let error_clone = error_count.clone();

    // Handler that sometimes fails
    bus.subscribe(EventType::FileChanged, move |event| {
        if let EventPayload::FileChange { path, .. } = &event.payload {
            if path.to_string_lossy().contains("error") {
                error_clone.fetch_add(1, Ordering::SeqCst);
                return Err(anyhow::anyhow!("Simulated error"));
            }
            success_clone.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    })
    .await;

    // Publish events - some will succeed, some will fail
    let events = vec![
        Event::file_changed("/src/good.rs".into(), FileChangeType::Modified),
        Event::file_changed("/src/error.rs".into(), FileChangeType::Modified),
        Event::file_changed("/src/another.rs".into(), FileChangeType::Created),
    ];

    for event in events {
        // The bus should handle errors gracefully and continue
        bus.publish_async(event).await.unwrap();
    }

    sleep(Duration::from_millis(100)).await;
    assert_eq!(success_count.load(Ordering::SeqCst), 2);
    assert_eq!(error_count.load(Ordering::SeqCst), 1);
}