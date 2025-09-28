#!/bin/bash
set -e

# Upload Hive Consensus DMG to R2
# Usage: ./scripts/upload-dmg-to-r2.sh [stable|beta]

CHANNEL=${1:-stable}
BUCKET_NAME="releases-hivetechs"
DMG_PATH="out/make/Hive Consensus.dmg"
VERSION=$(node -p "require('./package.json').version")

echo "🚀 Uploading Hive Consensus v$VERSION to R2 ($CHANNEL channel)..."

# Check if wrangler is available
if ! command -v wrangler &> /dev/null; then
    echo "❌ Wrangler CLI not found. Please install it:"
    echo "   npm install -g wrangler"
    exit 1
fi

# Check if DMG exists
if [ ! -f "$DMG_PATH" ]; then
    echo "❌ DMG not found at $DMG_PATH"
    echo "   Run 'npm run make' first"
    exit 1
fi

# Get file size
SIZE=$(stat -f%z "$DMG_PATH" 2>/dev/null || stat -c%s "$DMG_PATH")
SIZE_MB=$(echo "scale=2; $SIZE / 1024 / 1024" | bc)

echo "📦 Uploading DMG (${SIZE_MB}MB) to R2 (remote if possible)..."

# Decide upload mechanism: Wrangler (<=300MiB) or AWS S3 to R2 (>300MiB)
USE_AWS=0
if [ "$SIZE" -gt 314572800 ]; then
  USE_AWS=1
fi

# Build optional endpoint args for AWS CLI
ENDPOINT_ARGS=()
if [ -n "${R2_ENDPOINT:-}" ]; then
  ENDPOINT_ARGS+=(--endpoint-url "$R2_ENDPOINT")
fi

## Upload DMG to R2 (canonical public paths)
if [ "$USE_AWS" -eq 1 ]; then
  echo "  ⚠️  DMG exceeds 300 MiB; using AWS S3 (R2) route"
  if ! command -v aws >/dev/null 2>&1; then
    echo "❌ AWS CLI not found. Install with: brew install awscli"; exit 1;
  fi
  echo "  📱 Uploading macOS DMG via aws s3 cp..."
  aws s3 cp "$DMG_PATH" s3://$BUCKET_NAME/$CHANNEL/Hive-Consensus-v${VERSION}.dmg \
    --content-type application/x-apple-diskimage "${ENDPOINT_ARGS[@]}"
  aws s3 cp "$DMG_PATH" s3://$BUCKET_NAME/$CHANNEL/Hive-Consensus-latest.dmg \
    --content-type application/x-apple-diskimage "${ENDPOINT_ARGS[@]}"
else
  echo "  📱 Uploading macOS DMG via wrangler (remote)..."
  wrangler r2 object put "$BUCKET_NAME/$CHANNEL/Hive-Consensus-v${VERSION}.dmg" \
      --file="$DMG_PATH" \
      --content-type="application/x-apple-diskimage" \
      --remote
  wrangler r2 object put "$BUCKET_NAME/$CHANNEL/Hive-Consensus-latest.dmg" \
      --file="$DMG_PATH" \
      --content-type="application/x-apple-diskimage" \
      --remote
fi

# Create and upload version metadata
cat > /tmp/electron-version.json <<EOF
{
    "version": "$VERSION",
    "channel": "$CHANNEL",
    "platform": "darwin",
    "arch": "arm64",
    "url": "https://releases.hivetechs.io/$CHANNEL/Hive-Consensus-v${VERSION}.dmg",
    "size": $SIZE,
    "size_mb": "$SIZE_MB",
    "date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "type": "electron-app"
}
EOF

echo "  📋 Uploading version metadata..."
if [ "$USE_AWS" -eq 1 ]; then
  aws s3 cp /tmp/electron-version.json s3://$BUCKET_NAME/$CHANNEL/version.json \
    --content-type application/json "${ENDPOINT_ARGS[@]}"
else
  wrangler r2 object put "$BUCKET_NAME/$CHANNEL/version.json" \
      --file="/tmp/electron-version.json" \
      --content-type="application/json" \
      --remote
fi

# Upload ZIP if it exists
ZIP_PATH="out/make/zip/darwin/arm64/Hive Consensus-darwin-arm64-${VERSION}.zip"
if [ -f "$ZIP_PATH" ]; then
  echo "  📦 Uploading ZIP archive..."
  if [ "$USE_AWS" -eq 1 ]; then
    aws s3 cp "$ZIP_PATH" s3://$BUCKET_NAME/$CHANNEL/Hive-Consensus-v${VERSION}-darwin-arm64.zip \
      --content-type application/zip "${ENDPOINT_ARGS[@]}"
  else
    wrangler r2 object put "$BUCKET_NAME/$CHANNEL/Hive-Consensus-v${VERSION}-darwin-arm64.zip" \
        --file="$ZIP_PATH" \
        --content-type="application/zip" \
        --remote
  fi
fi

echo ""
echo "✅ Upload complete!"
echo ""
echo "📥 Download URLs:"
echo "  DMG:      https://releases.hivetechs.io/$CHANNEL/Hive-Consensus-v${VERSION}.dmg"
echo "  Latest:   https://releases.hivetechs.io/$CHANNEL/Hive-Consensus-latest.dmg"
echo "  Metadata: https://releases.hivetechs.io/$CHANNEL/version.json"
if [ -f "$ZIP_PATH" ]; then
    echo "  ZIP:      https://releases.hivetechs.io/$CHANNEL/Hive-Consensus-v${VERSION}-darwin-arm64.zip"
fi

echo ""
echo "📋 To list uploaded files:"
echo "  wrangler r2 object list $BUCKET_NAME --prefix=$CHANNEL/electron/"

# Clean up
rm -f /tmp/electron-version.json
