#!/bin/bash

echo "Starting hive-consensus for testing..."

# Start the process in background and capture output
RUST_LOG=info ./target/debug/hive-consensus 2>&1 | tee consensus.log &
PID=$!

echo "Waiting for consensus to initialize..."
sleep 5

# Check if consensus wrapper was created
echo "=== Checking consensus wrapper status ==="
grep -E "consensus wrapper|ERROR|Verified" consensus.log | tail -10

# Wait for user to test
echo ""
echo "GUI should be running. Try typing a query in the consensus tab."
echo "Press Ctrl+C to stop when done testing."

# Wait for interrupt
wait $PID