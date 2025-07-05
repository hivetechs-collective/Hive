#!/bin/bash

echo "🐝 Preparing to publish HiveTechs Consensus v2.1.0 to NPM..."

# Ensure we have the latest binary
if [ ! -f "../target/release/hive" ]; then
    echo "❌ Release binary not found. Please run 'cargo build --release' first."
    exit 1
fi

# Ensure binary is copied
if [ ! -f "bin/hive" ]; then
    echo "📦 Copying release binary..."
    mkdir -p bin
    cp ../target/release/hive bin/hive
    chmod 755 bin/hive
fi

# Verify package
echo "📋 Package contents:"
ls -la bin/
echo ""

# Check package.json version
VERSION=$(node -p "require('./package.json').version")
echo "📦 Package version: $VERSION"
echo ""

# Dry run to check what will be published
echo "🔍 Running npm publish dry-run..."
npm publish --dry-run

echo ""
echo "✅ Package prepared for publishing"
echo ""
echo "To publish to NPM:"
echo "  1. Ensure you're logged in: npm login"
echo "  2. Publish: npm publish --access public"
echo ""
echo "After publishing:"
echo "  - Package will be available at: https://www.npmjs.com/package/@hivetechs/hive"
echo "  - Users can install with: npm install -g @hivetechs/hive"