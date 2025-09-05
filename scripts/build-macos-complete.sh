#!/bin/bash
set -e

echo "🚀 Building complete macOS distribution for Hive IDE..."

# Build the release binary
echo "🔨 Building release binary..."
cargo build --release --bin hive-consensus

# Create dist directory
mkdir -p dist

# Copy binary to dist
echo "📋 Copying binary to dist..."
cp target/release/hive-consensus dist/hive-macos-arm64

# Strip debug symbols for smaller size
echo "🔧 Stripping debug symbols..."
strip dist/hive-macos-arm64

# Package the app bundle
echo "📦 Creating app bundle..."
./scripts/package-macos.sh

# Sign the app (will auto-detect certificates or use ad-hoc)
echo "🔐 Signing app..."
./scripts/sign-macos.sh

# Create professional DMG
echo "💿 Creating DMG..."
./scripts/create-dmg.sh

# Generate checksums for all distribution files
echo "🔐 Generating checksums..."
cd dist
shasum -a 256 *.dmg *.tar.gz > checksums.sha256 2>/dev/null || true
cd ..

echo "✅ macOS build complete!"
echo ""
echo "📦 Distribution files created:"
ls -la dist/*.dmg dist/*.tar.gz 2>/dev/null || true
echo ""
echo "🔑 Checksums:"
cat dist/checksums.sha256 2>/dev/null || echo "No checksum file created"

echo ""
echo "🎯 Next steps:"
echo "1. Test the DMG on a clean Mac"
echo "2. Upload to R2 when Apple certificates are ready"
echo "3. Update downloads page"