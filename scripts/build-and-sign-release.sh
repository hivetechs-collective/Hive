#!/bin/bash
set -e

echo "🚀 Building and signing Hive release for macOS..."

# Build the release binary
echo "🔨 Building release binary..."
cargo build --release --features desktop

# Create dist directory
mkdir -p dist

# Copy binary to dist
echo "📋 Copying binary to dist..."
cp target/release/hive-consensus dist/hive-macos-arm64

# Strip debug symbols for smaller size
echo "🔧 Stripping debug symbols..."
strip dist/hive-macos-arm64

# Package the app
echo "📦 Creating app bundle..."
./scripts/package-macos.sh

# Make scripts executable
chmod +x scripts/sign-macos.sh

# Attempt to sign the app
echo "🔐 Code signing..."
./scripts/sign-macos.sh

# Create the archive
echo "🗜️ Creating distribution archive..."
cd dist
tar -czf hive-macos-arm64.app.tar.gz Hive.app
cd ..

# Calculate checksums
echo "🔐 Calculating checksums..."
cd dist
shasum -a 256 hive-macos-arm64.app.tar.gz > hive-macos-arm64.app.tar.gz.sha256
cd ..

echo "✅ Build complete!"
echo "📦 Distribution file: dist/hive-macos-arm64.app.tar.gz"
echo "🔑 Checksum file: dist/hive-macos-arm64.app.tar.gz.sha256"