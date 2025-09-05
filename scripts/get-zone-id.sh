#!/bin/bash
set -e

echo "🔍 Getting Zone ID for hivetechs.io..."

# Try to get zone ID from Cloudflare API using wrangler's auth
# We'll use a simple approach by checking your existing deployment

cd /Users/veronelazio/Developer/Private/hivetechs-website

# Get zone info from wrangler.production.toml
if grep -q "zone_name.*hivetechs.io" wrangler.production.toml; then
    echo "✅ Found hivetechs.io zone configuration"
    
    # Try deploying with --dry-run to get zone info
    echo "Getting zone details..."
    wrangler deploy --dry-run 2>&1 | grep -i zone || echo "No zone info in dry run"
    
else
    echo "❌ Zone configuration not found"
fi