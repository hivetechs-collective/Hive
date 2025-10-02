#!/bin/bash
# Ad-hoc signing for Homebrew distribution
# No Apple Developer ID required

set -e

APP_PATH="out/Hive Consensus-darwin-arm64/Hive Consensus.app"
DMG_PATH="out/make/Hive Consensus.dmg"

echo "🔐 Ad-hoc signing application bundle..."

# Sign with ad-hoc identity (no Developer ID needed)
codesign --sign - \
  --force \
  --deep \
  --timestamp \
  --options runtime \
  "$APP_PATH"

echo "✅ Application signed with ad-hoc signature"

# Verify signature
echo "🔍 Verifying signature..."
codesign --verify --deep --verbose "$APP_PATH"

echo "✅ Ad-hoc signing complete!"
echo "📦 DMG will be created next by electron-forge"
