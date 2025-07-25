#!/bin/bash

echo "🚀 Monitoring Hive Consensus Desktop Application"
echo "=============================================="
echo ""

# Function to check if process is running
check_process() {
    if pgrep -f "hive-consensus" > /dev/null; then
        return 0
    else
        return 1
    fi
}

# Function to get memory usage
get_memory_usage() {
    ps aux | grep "[h]ive-consensus" | awk '{print $6/1024 " MB"}'
}

# Function to get CPU usage
get_cpu_usage() {
    ps aux | grep "[h]ive-consensus" | awk '{print $3 "%"}'
}

# Wait for build to complete
echo "⏳ Waiting for build to complete..."
while pgrep -f "cargo build" > /dev/null; do
    sleep 2
done

echo "✅ Build completed"
echo ""

# Check if binary was created
BINARY_PATH="target/release/hive-consensus"
if [ -f "$BINARY_PATH" ]; then
    echo "✅ Binary created: $BINARY_PATH"
    echo "   Size: $(ls -lh $BINARY_PATH | awk '{print $5}')"
else
    echo "❌ Binary not found at $BINARY_PATH"
    exit 1
fi

# Start the application in background
echo ""
echo "🎯 Starting Hive Consensus Desktop..."
export RUST_LOG=info
nohup $BINARY_PATH > hive-consensus.log 2>&1 &
APP_PID=$!

echo "   PID: $APP_PID"
echo "   Log: hive-consensus.log"

# Monitor for 10 minutes
START_TIME=$(date +%s)
DURATION=600  # 10 minutes in seconds

echo ""
echo "📊 Monitoring for 10 minutes..."
echo "Time | Status | Memory | CPU | Log Lines"
echo "----------------------------------------"

while true; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    
    # Check if 10 minutes have passed
    if [ $ELAPSED -ge $DURATION ]; then
        break
    fi
    
    # Check if process is still running
    if check_process; then
        STATUS="✅ Running"
        MEMORY=$(get_memory_usage)
        CPU=$(get_cpu_usage)
        LOG_LINES=$(wc -l < hive-consensus.log 2>/dev/null || echo "0")
        
        printf "%3ds | %s | %8s | %6s | %d\n" \
            "$ELAPSED" "$STATUS" "$MEMORY" "$CPU" "$LOG_LINES"
    else
        echo ""
        echo "❌ Process stopped after $ELAPSED seconds"
        echo ""
        echo "Last 20 lines of log:"
        tail -20 hive-consensus.log
        exit 1
    fi
    
    sleep 30
done

echo ""
echo "✅ Application ran successfully for 10 minutes!"
echo ""
echo "📈 Final Statistics:"
echo "   Memory Usage: $(get_memory_usage)"
echo "   CPU Usage: $(get_cpu_usage)"
echo "   Log Lines: $(wc -l < hive-consensus.log)"
echo ""
echo "🔍 Sample log entries:"
tail -10 hive-consensus.log | grep -E "(INFO|WARN|ERROR)" | head -5

# Gracefully stop the application
echo ""
echo "🛑 Stopping application..."
kill $APP_PID 2>/dev/null
sleep 2

# Check if stopped
if check_process; then
    echo "⚠️  Application still running, force stopping..."
    kill -9 $APP_PID 2>/dev/null
fi

echo "✅ Application stopped successfully"
echo ""
echo "📋 Summary: The Hive Consensus Desktop application with hybrid Claude Code integration"
echo "           built successfully and ran stable for 10+ minutes!"