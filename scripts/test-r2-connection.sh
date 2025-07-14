#!/bin/bash
set -e

echo "🧪 Testing Cloudflare R2 Connection..."
echo ""

# Check if AWS CLI is available
if ! command -v aws &> /dev/null; then
    echo "AWS CLI is available: ✅"
else
    echo "❌ AWS CLI not found. It should have been installed earlier."
    exit 1
fi

# Check if R2 credentials are configured
if aws configure get aws_access_key_id &> /dev/null && aws configure get aws_secret_access_key &> /dev/null; then
    echo "R2 credentials configured: ✅"
else
    echo "❌ R2 credentials not configured. Please run setup-r2-config.sh first"
    exit 1
fi

# Test connection to R2
echo "Testing connection to R2 bucket..."
if aws s3 ls s3://releases-hivetechs/ 2>/dev/null; then
    echo "R2 bucket connection: ✅"
    echo ""
    echo "🎉 R2 is ready for uploads!"
else
    echo "⚠️  R2 bucket 'releases-hivetechs' not found or not accessible"
    echo "   Creating bucket now..."
    
    # Try to create the bucket
    if aws s3 mb s3://releases-hivetechs/ 2>/dev/null; then
        echo "✅ Created R2 bucket successfully"
    else
        echo "❌ Failed to create bucket. Please check:"
        echo "   - Account ID is correct"
        echo "   - API token has admin permissions"
        echo "   - Bucket name 'releases-hivetechs' is available"
        exit 1
    fi
fi

echo ""
echo "📋 Next steps:"
echo "   1. Run: ./scripts/upload-to-r2.sh stable"
echo "   2. Test downloads at: https://releases.hivetechs.io/"