#!/bin/bash
# Run the packaged Hive Consensus app with proper icon and name

APP_PATH="./out/electron-poc-darwin-arm64/electron-poc.app"

if [ -d "$APP_PATH" ]; then
    echo "Opening Hive Consensus..."
    open "$APP_PATH"
else
    echo "App not found. Please run 'npm run package' first."
fi