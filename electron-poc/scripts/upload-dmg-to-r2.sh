#!/bin/bash
set -euo pipefail

# Upload Hive Consensus DMG to Cloudflare R2 (S3-compatible)
# Usage: ./scripts/upload-dmg-to-r2.sh [stable|beta]

CHANNEL=${1:-stable}
BUCKET_NAME="releases-hivetechs"
DMG_PATH="out/make/Hive Consensus.dmg"
VERSION=$(node -p "require('./package.json').version")

# Auto-source local secrets if present (kept gitignored)
if [ -f "$(dirname "$0")/../.r2.env" ]; then
  set -a
  source "$(dirname "$0")/../.r2.env"
  set +a
fi

echo "🚀 Uploading Hive Consensus v$VERSION to R2 ($CHANNEL channel)..."

# Ensure AWS CLI present (we use S3-compatible API for R2 uploads)
if ! command -v aws >/dev/null 2>&1; then
  echo "❌ AWS CLI not found. Install it: brew install awscli"; exit 1;
fi

# Validate required env vars
if [ -z "${AWS_ACCESS_KEY_ID:-}" ] || [ -z "${AWS_SECRET_ACCESS_KEY:-}" ] || [ -z "${R2_ENDPOINT:-}" ]; then
  echo "❌ Missing credentials. Set AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, and R2_ENDPOINT."
  echo "   Tip: create electron-poc/.r2.env with those vars and keep it local (gitignored)."; exit 1;
fi

# Check if DMG exists
if [ ! -f "$DMG_PATH" ]; then
  echo "❌ DMG not found at $DMG_PATH"
  echo "   Run 'npm run make' (or 'npm run build:complete') first"; exit 1;
fi

# Compute file size
SIZE=$(stat -f%z "$DMG_PATH" 2>/dev/null || stat -c%s "$DMG_PATH")
SIZE_MB=$(echo "scale=2; $SIZE / 1024 / 1024" | bc)
echo "📦 Uploading DMG (${SIZE_MB} MiB) to R2 via S3-compatible API..."

# Endpoint args for AWS CLI
ENDPOINT_ARGS=(--endpoint-url "$R2_ENDPOINT")

## Upload DMG to R2 (canonical public paths)
echo "  📱 Uploading versioned DMG..."
aws s3 cp "$DMG_PATH" s3://$BUCKET_NAME/$CHANNEL/Hive-Consensus-v${VERSION}.dmg \
  --content-type application/x-apple-diskimage "${ENDPOINT_ARGS[@]}"

echo "  📱 Uploading latest DMG alias..."
aws s3 cp "$DMG_PATH" s3://$BUCKET_NAME/$CHANNEL/Hive-Consensus-latest.dmg \
  --content-type application/x-apple-diskimage "${ENDPOINT_ARGS[@]}"

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
aws s3 cp /tmp/electron-version.json s3://$BUCKET_NAME/$CHANNEL/version.json \
  --content-type application/json "${ENDPOINT_ARGS[@]}"

# Upload ZIP if it exists
ZIP_PATH="out/make/zip/darwin/arm64/Hive Consensus-darwin-arm64-${VERSION}.zip"
if [ -f "$ZIP_PATH" ]; then
  echo "  📦 Uploading ZIP archive..."
  aws s3 cp "$ZIP_PATH" s3://$BUCKET_NAME/$CHANNEL/Hive-Consensus-v${VERSION}-darwin-arm64.zip \
    --content-type application/zip "${ENDPOINT_ARGS[@]}"
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
echo "📋 To list uploaded files (via AWS CLI):"
echo "  aws s3 ls s3://$BUCKET_NAME/$CHANNEL/ --endpoint-url $R2_ENDPOINT"

# Clean up
rm -f /tmp/electron-version.json
