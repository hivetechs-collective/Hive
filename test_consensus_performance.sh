#!/bin/bash

echo "🧪 Testing Consensus Performance Fix"
echo "=====================================

This test will run consensus and monitor for CPU overload issues.
The fix prevents uncontrolled async task spawning during consensus.

Starting test in 3 seconds..."
sleep 3

# Run consensus with a test query
echo -e "\n📊 Starting consensus with test query..."
echo "Question: 'What is the capital of France?'" | RUST_LOG=info ./target/debug/hive-consensus 2>&1 &
PID=$!

# Monitor for 10 seconds
echo -e "\n🔍 Monitoring CPU usage..."
for i in {1..10}; do
    CPU=$(ps aux | grep $PID | grep -v grep | awk '{print $3}')
    echo "[$i/10] CPU: ${CPU}%"
    
    # Check if CPU is excessive (>200% would indicate multiple cores maxed out)
    if (( $(echo "$CPU > 200" | bc -l) )); then
        echo "⚠️ WARNING: High CPU usage detected!"
    fi
    
    sleep 1
done

# Kill the process
kill $PID 2>/dev/null

echo -e "\n✅ Test completed. If CPU stayed below 200%, the fix is working."
echo "Previously, CPU would spike to 400%+ and system would become unresponsive."