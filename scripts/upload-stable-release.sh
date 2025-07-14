#!/bin/bash
set -e

echo "📤 Uploading stable release to R2..."

VERSION="${HIVE_VERSION:-2.0.2}"
BUCKET="releases-hivetechs"

# Check if wrangler is available
if ! command -v wrangler &> /dev/null; then
    echo "❌ Wrangler not found. Install with: npm install -g wrangler"
    exit 1
fi

# Upload all distribution files
echo "📦 Uploading distribution files..."

# macOS files
if [ -f "dist/Hive-IDE-$VERSION.dmg" ]; then
    echo "🍎 Uploading macOS DMG..."
    wrangler r2 object put "$BUCKET/stable/Hive-IDE-$VERSION.dmg" --file="dist/Hive-IDE-$VERSION.dmg" --remote
    wrangler r2 object put "$BUCKET/stable/Hive-IDE-$VERSION.dmg.sha256" --file="dist/Hive-IDE-$VERSION.dmg.sha256" --remote
fi

if [ -f "dist/hive-macos-arm64.app.tar.gz" ]; then
    echo "🍎 Uploading macOS ARM64 archive..."
    wrangler r2 object put "$BUCKET/stable/hive-macos-arm64.app.tar.gz" --file="dist/hive-macos-arm64.app.tar.gz" --remote
    wrangler r2 object put "$BUCKET/stable/hive-macos-arm64.app.tar.gz.sha256" --file="dist/hive-macos-arm64.app.tar.gz.sha256" --remote
fi

# Windows files
if [ -f "dist/Hive-IDE-$VERSION.msi" ]; then
    echo "🪟 Uploading Windows MSI..."
    wrangler r2 object put "$BUCKET/stable/Hive-IDE-$VERSION.msi" --file="dist/Hive-IDE-$VERSION.msi" --remote
    wrangler r2 object put "$BUCKET/stable/Hive-IDE-$VERSION.msi.sha256" --file="dist/Hive-IDE-$VERSION.msi.sha256" --remote
fi

if [ -f "dist/hive-windows-x64.exe" ]; then
    echo "🪟 Uploading Windows executable..."
    wrangler r2 object put "$BUCKET/stable/hive-windows-x64.exe" --file="dist/hive-windows-x64.exe" --remote
    wrangler r2 object put "$BUCKET/stable/hive-windows-x64.exe.sha256" --file="dist/hive-windows-x64.exe.sha256" --remote
fi

# Linux files
if [ -f "dist/Hive-IDE-$VERSION-x86_64.AppImage" ]; then
    echo "🐧 Uploading Linux AppImage..."
    wrangler r2 object put "$BUCKET/stable/Hive-IDE-$VERSION-x86_64.AppImage" --file="dist/Hive-IDE-$VERSION-x86_64.AppImage" --remote
    wrangler r2 object put "$BUCKET/stable/Hive-IDE-$VERSION-x86_64.AppImage.sha256" --file="dist/Hive-IDE-$VERSION-x86_64.AppImage.sha256" --remote
fi

# Upload checksums
if [ -f "dist/checksums-$VERSION.sha256" ]; then
    echo "🔐 Uploading checksums..."
    wrangler r2 object put "$BUCKET/stable/checksums-$VERSION.sha256" --file="dist/checksums-$VERSION.sha256" --remote
fi

# Upload releases.json
if [ -f "dist/releases.json" ]; then
    echo "📄 Uploading releases.json..."
    wrangler r2 object put "$BUCKET/releases.json" --file="dist/releases.json" --remote
fi

echo "✅ Upload complete!"
echo "🌐 Files available at: https://releases.hivetechs.io/"
