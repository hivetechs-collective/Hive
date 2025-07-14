#!/bin/bash
set -e

echo "🪣 Creating R2 bucket for desktop app distribution..."
echo ""

# Check if wrangler is available and authenticated
if ! command -v wrangler &> /dev/null; then
    echo "❌ Wrangler CLI not found. Please install it:"
    echo "   npm install -g wrangler"
    exit 1
fi

# Check authentication
if ! wrangler whoami &> /dev/null; then
    echo "❌ Not authenticated with Cloudflare. Please run:"
    echo "   wrangler login"
    exit 1
fi

echo "✅ Wrangler is authenticated"

# Create the R2 bucket
echo "Creating bucket 'releases-hivetechs'..."
if wrangler r2 bucket create releases-hivetechs; then
    echo "✅ R2 bucket created successfully"
else
    echo "⚠️  Bucket creation failed. This might be because:"
    echo "   - R2 is not enabled on your account (enable via dashboard)"
    echo "   - Bucket already exists"
    echo "   - Account permissions issue"
    echo ""
    echo "Please enable R2 in your Cloudflare Dashboard first:"
    echo "   1. Go to dash.cloudflare.com"
    echo "   2. Look for 'R2 Object Storage' in sidebar"
    echo "   3. Click 'Get Started' and accept free tier"
    exit 1
fi

echo ""
echo "🎯 Next steps:"
echo "   1. Enable public access for the bucket"
echo "   2. Upload files with: ./scripts/upload-to-r2-wrangler.sh"