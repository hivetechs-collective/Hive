#!/bin/bash
# Build DMG for Homebrew distribution
# No Apple Developer ID or R2 upload needed

set -e

echo "🍺 Building Hive Consensus for Homebrew distribution"
echo ""

# Run the complete build
npm run build:complete

echo ""
echo "✅ Build complete!"
echo ""
echo "📦 DMG location: out/make/Hive Consensus.dmg"
echo ""
echo "Next steps:"
echo "1. Create GitHub release and upload DMG"
echo "2. Calculate SHA256: shasum -a 256 'out/make/Hive Consensus.dmg'"
echo "3. Update Homebrew cask formula with version and SHA256"
echo "4. Push to homebrew-tap repository"
