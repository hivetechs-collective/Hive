#!/bin/bash

# Upload Hive Consensus to R2 for distribution
# Usage: ./scripts/upload-to-r2.sh

set -e

# Check for required environment variables
if [ -z "$R2_ACCESS_KEY_ID" ] || [ -z "$R2_SECRET_ACCESS_KEY" ] || [ -z "$R2_BUCKET" ] || [ -z "$R2_ENDPOINT" ]; then
    echo "Error: R2 credentials not set. Please set:"
    echo "  R2_ACCESS_KEY_ID"
    echo "  R2_SECRET_ACCESS_KEY"
    echo "  R2_BUCKET"
    echo "  R2_ENDPOINT"
    exit 1
fi

VERSION=$(node -p "require('./package.json').version")
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture names
if [ "$ARCH" = "x86_64" ]; then
    ARCH="x64"
elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
    ARCH="arm64"
fi

# Map platform names
if [ "$PLATFORM" = "darwin" ]; then
    PLATFORM="mac"
    EXT="dmg"
    DEFAULT_PATH="out/make/dmg/arm64/Hive Consensus.dmg"
    if [ -f "$DEFAULT_PATH" ]; then
        MAKE_PATH="$DEFAULT_PATH"
    else
        MAKE_PATH=$(find out/make -maxdepth 1 -name '*.dmg' | head -n1 || true)
    fi
elif [ "$PLATFORM" = "linux" ]; then
    PLATFORM="linux"
    EXT="AppImage"
    MAKE_PATH="out/make/appimage/x64/Hive-Consensus-$VERSION.AppImage"
else
    echo "Unsupported platform: $PLATFORM"
    exit 1
fi

echo "Uploading Hive Consensus v$VERSION for $PLATFORM-$ARCH..."

# Check if distribution file exists
if [ -z "$MAKE_PATH" ] || [ ! -f "$MAKE_PATH" ]; then
    echo "Distribution file not found: $MAKE_PATH"
    echo "Please run 'npm run make' first"
    exit 1
fi

UPLOAD_NAME="Hive-Consensus-$VERSION-$PLATFORM-$ARCH.$EXT"

# Generate checksum
SHA_PATH="/tmp/${UPLOAD_NAME}.sha256"
shasum -a 256 "$MAKE_PATH" > "$SHA_PATH"

# Upload to R2 using AWS CLI (works with R2)
aws s3 cp "$MAKE_PATH" \
    "s3://$R2_BUCKET/releases/v$VERSION/$PLATFORM/$UPLOAD_NAME" \
    --endpoint-url "$R2_ENDPOINT" \
    --region auto

aws s3 cp "$SHA_PATH" \
    "s3://$R2_BUCKET/releases/v$VERSION/$PLATFORM/${UPLOAD_NAME}.sha256" \
    --endpoint-url "$R2_ENDPOINT" \
    --region auto

# Also upload as 'latest'
aws s3 cp "$MAKE_PATH" \
    "s3://$R2_BUCKET/releases/latest/$PLATFORM/Hive-Consensus-$PLATFORM-$ARCH.$EXT" \
    --endpoint-url "$R2_ENDPOINT" \
    --region auto

aws s3 cp "$SHA_PATH" \
    "s3://$R2_BUCKET/releases/latest/$PLATFORM/Hive-Consensus-$PLATFORM-$ARCH.$EXT.sha256" \
    --endpoint-url "$R2_ENDPOINT" \
    --region auto

# Create and upload version manifest
cat > /tmp/version.json <<EOF
{
    "version": "$VERSION",
    "platform": "$PLATFORM",
    "arch": "$ARCH",
    "url": "https://downloads.hivetechs.io/releases/v$VERSION/$PLATFORM/Hive-Consensus-$VERSION-$PLATFORM-$ARCH.$EXT",
    "size": $(stat -f%z "$MAKE_PATH" 2>/dev/null || stat -c%s "$MAKE_PATH"),
    "date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

aws s3 cp /tmp/version.json \
    "s3://$R2_BUCKET/releases/latest/$PLATFORM/version.json" \
    --endpoint-url "$R2_ENDPOINT" \
    --region auto \
    --content-type "application/json"

echo "✅ Successfully uploaded to R2"
echo "Download URL: https://downloads.hivetechs.io/releases/latest/$PLATFORM/Hive-Consensus-$PLATFORM-$ARCH.$EXT"
