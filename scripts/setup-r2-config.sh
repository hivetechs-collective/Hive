#!/bin/bash
set -e

# Cloudflare R2 Configuration Helper
# This script helps configure AWS CLI to work with Cloudflare R2

echo "🔧 Configuring AWS CLI for Cloudflare R2..."
echo ""

# Check if AWS CLI is installed
if ! command -v aws &> /dev/null; then
    echo "❌ AWS CLI not found. Installing now..."
    brew install awscli
    echo "✅ AWS CLI installed successfully"
fi

echo "📋 Please gather the following from your Cloudflare Dashboard:"
echo "   1. Go to Cloudflare Dashboard → R2 Object Storage"
echo "   2. Click 'Manage R2 API tokens'"
echo "   3. Create a new API token with:"
echo "      - Permissions: Object:Edit, Object:Read"
echo "      - Bucket: releases-hivetechs"
echo "   4. Copy the Access Key ID and Secret Access Key"
echo "   5. Note your Account ID (visible in the endpoint URL)"
echo ""

# Get credentials from user
read -p "Enter your R2 Access Key ID: " access_key_id
read -s -p "Enter your R2 Secret Access Key: " secret_access_key
echo ""
read -p "Enter your Cloudflare Account ID: " account_id

echo ""
echo "🔐 Configuring AWS CLI for R2..."

# Configure AWS CLI for R2
aws configure set aws_access_key_id "$access_key_id"
aws configure set aws_secret_access_key "$secret_access_key"
aws configure set default.s3.signature_version s3v4
aws configure set default.s3.endpoint_url "https://$account_id.r2.cloudflarestorage.com"

echo "✅ AWS CLI configured for Cloudflare R2"
echo ""

# Test the configuration
echo "🧪 Testing R2 connection..."
if aws s3 ls s3://releases-hivetechs/ 2>/dev/null; then
    echo "✅ Successfully connected to R2 bucket!"
else
    echo "❌ Connection test failed. Please check:"
    echo "   - Bucket 'releases-hivetechs' exists"
    echo "   - API token has correct permissions"
    echo "   - Account ID is correct"
    exit 1
fi

echo ""
echo "🎉 R2 configuration complete! You can now run:"
echo "   ./scripts/upload-to-r2.sh stable"