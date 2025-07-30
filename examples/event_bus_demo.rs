//! Event Bus Demo
//!
//! Run with: cargo run --example event_bus_demo

use hive::desktop::events::{event_bus, Event, EventType, FileChangeType, ConsensusStage};
use std::time::Duration;
use tokio::time::sleep;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("=== Event Bus Demo ===\n");
    
    // Get the global event bus
    let bus = event_bus();
    
    // Setup status bar listener
    println!("Setting up status bar listener...");
    bus.subscribe(EventType::GitStatusChanged, |event| {
        if let hive::desktop::events::EventPayload::GitStatus { branch, modified_files, .. } = &event.payload {
            println!("📊 Status Bar: Git branch '{}' with {} modified files", branch, modified_files.len());
        }
        Ok(())
    }).await;
    
    // Setup file explorer listener
    println!("Setting up file explorer listener...");
    bus.subscribe(EventType::FileChanged, |event| {
        if let hive::desktop::events::EventPayload::FileChange { path, change_type } = &event.payload {
            let action = match change_type {
                FileChangeType::Created => "created",
                FileChangeType::Modified => "modified",
                FileChangeType::Deleted => "deleted",
                FileChangeType::Renamed { .. } => "renamed",
            };
            println!("📁 File Explorer: File {} at {:?}", action, path);
        }
        Ok(())
    }).await;
    
    // Setup consensus UI listener
    println!("Setting up consensus UI listener...");
    bus.subscribe_async(EventType::ConsensusStateChanged, |event| async move {
        if let hive::desktop::events::EventPayload::ConsensusState { stage, progress, .. } = &event.payload {
            let stage_name = match stage {
                ConsensusStage::Initializing => "Initializing",
                ConsensusStage::Generating => "Generating",
                ConsensusStage::Refining => "Refining",
                ConsensusStage::Validating => "Validating",
                ConsensusStage::Curating => "Curating",
                ConsensusStage::Completed => "Completed",
                ConsensusStage::Failed => "Failed",
            };
            println!("🤖 Consensus UI: {} - {}%", stage_name, progress);
        }
        Ok(())
    }).await;
    
    // Setup build system listener
    println!("Setting up build system listener...");
    bus.subscribe(EventType::BuildStarted, |event| {
        if let hive::desktop::events::EventPayload::BuildInfo { target, profile } = &event.payload {
            println!("🔨 Build System: Starting build - {} ({})", target, profile);
        }
        Ok(())
    }).await;
    
    bus.subscribe(EventType::BuildCompleted, |event| {
        if let hive::desktop::events::EventPayload::BuildResult { success, duration_ms, .. } = &event.payload {
            let status = if *success { "✅ succeeded" } else { "❌ failed" };
            println!("🔨 Build System: Build {} in {}ms", status, duration_ms);
        }
        Ok(())
    }).await;
    
    println!("\n--- Simulating Application Events ---\n");
    
    // Simulate git status change
    println!("1. Git repository switched to 'feature/event-bus'");
    bus.publish_async(Event::git_status_changed(
        "feature/event-bus".to_string(),
        vec!["src/events.rs".to_string(), "src/main.rs".to_string()],
        vec!["Cargo.toml".to_string()],
    )).await?;
    sleep(Duration::from_millis(100)).await;
    
    // Simulate file changes
    println!("\n2. File watcher detected changes");
    bus.publish_async(Event::file_changed(
        "/src/events/bus.rs".into(),
        FileChangeType::Created,
    )).await?;
    
    bus.publish_async(Event::file_changed(
        "/src/main.rs".into(),
        FileChangeType::Modified,
    )).await?;
    sleep(Duration::from_millis(100)).await;
    
    // Simulate build process
    println!("\n3. User triggered build");
    bus.publish_async(Event::build_started(
        "release".to_string(),
        "optimized".to_string(),
    )).await?;
    sleep(Duration::from_millis(500)).await;
    
    bus.publish_async(Event::build_completed(
        true,
        vec![],
        vec!["unused import: `std::fs`".to_string()],
        523,
    )).await?;
    sleep(Duration::from_millis(100)).await;
    
    // Simulate consensus pipeline
    println!("\n4. Consensus pipeline started");
    let stages = vec![
        (ConsensusStage::Initializing, 0),
        (ConsensusStage::Generating, 25),
        (ConsensusStage::Generating, 50),
        (ConsensusStage::Refining, 65),
        (ConsensusStage::Validating, 80),
        (ConsensusStage::Curating, 95),
        (ConsensusStage::Completed, 100),
    ];
    
    for (stage, progress) in stages {
        bus.publish_async(Event::consensus_state_changed(stage, progress, None)).await?;
        sleep(Duration::from_millis(300)).await;
    }
    
    println!("\n--- Demo Complete ---");
    println!("\nThe event bus enables loose coupling between components while maintaining type safety.");
    println!("Components can subscribe to specific event types and react accordingly.");
    
    Ok(())
}