#!/bin/bash

# Create a log file for the build output
BUILD_LOG="/tmp/hive-build-progress.log"
echo "Starting build..." > "$BUILD_LOG"

# Kill any existing tail processes for this log
pkill -f "tail -f $BUILD_LOG" 2>/dev/null || true

# Launch a single Terminal window to tail the log file
osascript <<EOF
tell application "Terminal"
    activate
    set newWindow to do script "echo '🏗️  Hive Consensus Build Progress' && echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━' && echo '' && tail -f $BUILD_LOG"
    set bounds of front window to {100, 100, 700, 500}
    set custom title of front window to "Build Progress"
end tell
EOF

# Small delay to ensure Terminal window is open
sleep 0.5

# Function to log to both console and file
log_message() {
    echo "$1"
    echo "$1" >> "$BUILD_LOG"
}

# Execute the build script and pipe output to log
# Note: Header is already printed by build-production-dmg.js
node scripts/build-production-dmg.js 2>&1 | while IFS= read -r line; do
    log_message "$line"
done

# Get exit code
BUILD_EXIT_CODE=${PIPESTATUS[0]}

if [ $BUILD_EXIT_CODE -eq 0 ]; then
    log_message ""
    log_message "✅ BUILD SUCCESSFUL!"
    log_message ""
    log_message "Window will close in 10 seconds..."
    sleep 10
else
    log_message ""
    log_message "❌ BUILD FAILED!"
    log_message ""
    log_message "Check the log for errors. Window will remain open."
fi

# Kill the tail process (this will close the Terminal window)
pkill -f "tail -f $BUILD_LOG"

exit $BUILD_EXIT_CODE