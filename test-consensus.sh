#!/bin/bash

echo "Starting Hive Consensus desktop app..."
echo "Please check the bottom status bar for subscription display."
echo ""
echo "The status bar should show:"
echo "- Username (email)"
echo "- Subscription tier (FREE, BASIC, PREMIUM, etc.)"
echo "- Daily conversations remaining (e.g., 10/10 daily)"
echo ""
echo "The subscription info will refresh every 30 seconds."
echo ""
echo "Press Ctrl+C to stop the app when done testing."

cargo run --release --bin hive-consensus