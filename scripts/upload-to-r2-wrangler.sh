#!/bin/bash
set -e

# Hive Desktop App Upload Script using Wrangler
# Usage: ./upload-to-r2-wrangler.sh [stable|beta]

CHANNEL=${1:-stable}
BUCKET_NAME="releases-hivetechs"

echo "🚀 Uploading Hive Desktop App to R2 ($CHANNEL channel) using Wrangler..."

# Check if wrangler is available
if ! command -v wrangler &> /dev/null; then
    echo "❌ Wrangler CLI not found. Please install it:"
    echo "   npm install -g wrangler"
    exit 1
fi

# Verify files exist
if [ ! -f "dist/hive-macos-arm64" ]; then
    echo "❌ dist/hive-macos-arm64 not found. Run build first."
    exit 1
fi

if [ ! -f "dist/Hive.app/Contents/MacOS/hive" ]; then
    echo "❌ Hive.app not found. Run ./scripts/package-macos.sh first."
    exit 1
fi

if [ ! -f "dist/releases.json" ]; then
    echo "❌ releases.json not found."
    exit 1
fi

echo "📦 Uploading binaries to $CHANNEL channel..."

# Create macOS app bundle archive
echo "  📱 Creating macOS app bundle archive..."
cd dist
tar -czf hive-macos-arm64.app.tar.gz Hive.app
cd ..

# Upload files using wrangler with --remote flag
echo "  📱 Uploading macOS ARM64 app bundle..."
wrangler r2 object put $BUCKET_NAME/$CHANNEL/hive-macos-arm64.app.tar.gz --file=dist/hive-macos-arm64.app.tar.gz --content-type="application/gzip" --remote

echo "  📱 Uploading raw macOS binary..."
wrangler r2 object put $BUCKET_NAME/$CHANNEL/hive-macos-arm64 --file=dist/hive-macos-arm64 --content-type="application/octet-stream" --remote

echo "  📋 Uploading releases metadata..."
wrangler r2 object put $BUCKET_NAME/releases.json --file=dist/releases.json --content-type="application/json" --remote

echo "  📄 Uploading downloads page..."
wrangler r2 object put $BUCKET_NAME/downloads.html --file=dist/downloads-page.html --content-type="text/html" --remote

echo "✅ Upload complete!"
echo ""
echo "📥 Download URLs:"
echo "  macOS App:    https://pub-[BUCKET_ID].r2.dev/$CHANNEL/hive-macos-arm64.app.tar.gz"
echo "  macOS Binary: https://pub-[BUCKET_ID].r2.dev/$CHANNEL/hive-macos-arm64"
echo "  Metadata:     https://pub-[BUCKET_ID].r2.dev/releases.json"
echo "  Downloads:    https://pub-[BUCKET_ID].r2.dev/downloads.html"
echo ""
echo "🔧 Next steps:"
echo "  1. Enable public access in Cloudflare Dashboard"
echo "  2. Set up custom domain: releases.hivetechs.io"
echo "  3. Test downloads work properly"

# List uploaded files
echo ""
echo "📋 Uploaded files:"
wrangler r2 object list $BUCKET_NAME --prefix=$CHANNEL/
echo ""
echo "📋 Metadata files:"
wrangler r2 object list $BUCKET_NAME --prefix=releases.json