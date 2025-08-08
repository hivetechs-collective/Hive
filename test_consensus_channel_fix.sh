#!/bin/bash

echo "🧪 Testing Consensus Channel Fix"
echo "================================="

# Simple direct test bypassing GUI
echo "Testing consensus engine directly..."

# Create a simple test program
cat > /tmp/test_consensus.rs << 'EOF'
use hive_ai::consensus_runtime::ConsensusThreadWrapper;
use hive_ai::desktop::state::AppState;
use dioxus::prelude::*;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("Creating consensus wrapper...");
    
    // Create dummy app state
    let app_state = Signal::new(AppState::default());
    
    // Create wrapper
    match ConsensusThreadWrapper::new(app_state).await {
        Ok(wrapper) => {
            println!("✅ Wrapper created");
            
            // Test a simple query
            match wrapper.process_query("What is 2+2?".to_string()).await {
                Ok((result, mut event_rx)) => {
                    println!("✅ Got result: {}", result);
                    
                    // Try to receive events
                    let mut event_count = 0;
                    while let Ok(Some(event)) = event_rx.try_recv() {
                        event_count += 1;
                        println!("📡 Received event #{}", event_count);
                    }
                    
                    if event_count == 0 {
                        println!("⚠️ No events received - channel might be closed");
                    } else {
                        println!("✅ Received {} events", event_count);
                    }
                }
                Err(e) => println!("❌ Query error: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to create wrapper: {}", e),
    }
}
EOF

echo "Compiling test..."
rustc --edition 2021 -L target/debug/deps /tmp/test_consensus.rs -o /tmp/test_consensus 2>&1 | head -10

echo ""
echo "Alternative: Running hive-consensus with timeout..."
RUST_LOG=info ./target/debug/hive-consensus --help 2>&1 | head -5

echo ""
echo "✅ Test setup complete"