#!/bin/bash
set -e

# Hive Desktop App Upload Script for Cloudflare R2
# Usage: ./upload-to-r2.sh [stable|beta]

CHANNEL=${1:-stable}
BUCKET_NAME="releases-hivetechs"
BASE_URL="https://releases.hivetechs.io"

echo "🚀 Uploading Hive Desktop App to R2 ($CHANNEL channel)..."

# Check if AWS CLI is configured for R2
if ! command -v aws &> /dev/null; then
    echo "❌ AWS CLI not found. Please install it first:"
    echo "   brew install awscli"
    echo "   Then configure for Cloudflare R2:"
    echo "   aws configure set default.s3.signature_version s3v4"
    echo "   aws configure set default.s3.endpoint_url https://your-account-id.r2.cloudflarestorage.com"
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

# Upload macOS ARM64 binary
echo "  📱 Uploading macOS ARM64 binary..."
aws s3 cp dist/hive-macos-arm64 s3://$BUCKET_NAME/$CHANNEL/hive-macos-arm64 \
    --content-type application/octet-stream \
    --cache-control "public, max-age=3600"

# Create a simple .dmg-like download for macOS
echo "  📱 Creating macOS app bundle archive..."
cd dist
tar -czf hive-macos-arm64.app.tar.gz Hive.app
cd ..

aws s3 cp dist/hive-macos-arm64.app.tar.gz s3://$BUCKET_NAME/$CHANNEL/hive-macos-arm64.app.tar.gz \
    --content-type application/gzip \
    --cache-control "public, max-age=3600"

# Upload releases metadata
echo "  📋 Uploading releases metadata..."
aws s3 cp dist/releases.json s3://$BUCKET_NAME/releases.json \
    --content-type application/json \
    --cache-control "public, max-age=300"

# Set public read permissions
echo "  🔓 Setting public read permissions..."
aws s3api put-object-acl --bucket $BUCKET_NAME --key $CHANNEL/hive-macos-arm64 --acl public-read
aws s3api put-object-acl --bucket $BUCKET_NAME --key $CHANNEL/hive-macos-arm64.app.tar.gz --acl public-read
aws s3api put-object-acl --bucket $BUCKET_NAME --key releases.json --acl public-read

echo "✅ Upload complete!"
echo ""
echo "📥 Download URLs:"
echo "  macOS Binary: $BASE_URL/$CHANNEL/hive-macos-arm64"
echo "  macOS App:    $BASE_URL/$CHANNEL/hive-macos-arm64.app.tar.gz"
echo "  Metadata:     $BASE_URL/releases.json"
echo ""
echo "🧪 Test the update checker:"
echo "  cargo run --bin hive-consensus"
echo "  Go to Help → Check for Updates"

# Optional: Invalidate Cloudflare cache
if command -v curl &> /dev/null; then
    echo ""
    echo "🔄 To invalidate Cloudflare cache (if needed):"
    echo "  curl -X POST 'https://api.cloudflare.com/client/v4/zones/YOUR_ZONE_ID/purge_cache' \\"
    echo "    -H 'Authorization: Bearer YOUR_API_TOKEN' \\"
    echo "    -H 'Content-Type: application/json' \\"
    echo "    --data '{\"files\":[\"$BASE_URL/releases.json\"]}'"
fi