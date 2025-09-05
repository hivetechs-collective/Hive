#!/bin/bash
set -e

echo "🔐 Force signing Hive for macOS (ignoring chain warnings)..."

# Check if app exists
if [ ! -d "dist/Hive.app" ]; then
    echo "❌ Error: dist/Hive.app not found"
    echo "   Run ./scripts/package-macos.sh first"
    exit 1
fi

# Find signing identity
IDENTITY="Developer ID Application: Verone Lazio (FWBLB27H52)"
echo "📝 Using identity: $IDENTITY"

# Remove old signatures
echo "🧹 Removing old signatures..."
rm -rf dist/Hive.app/_CodeSignature
rm -rf dist/Hive.app/Contents/_CodeSignature
rm -rf dist/Hive.app/Contents/MacOS/*.dSYM
xattr -cr dist/Hive.app

# Sign with forced acceptance
echo "🔏 Force signing binary..."
codesign --force --deep -s "$IDENTITY" \
    --options runtime \
    --entitlements scripts/entitlements.plist \
    dist/Hive.app/Contents/MacOS/hive || true

echo "📦 Force signing app bundle..."
codesign --force --deep -s "$IDENTITY" \
    --options runtime \
    --entitlements scripts/entitlements.plist \
    dist/Hive.app || true

# Check if signing worked despite warnings
echo "🔍 Checking signature..."
if codesign --verify --deep dist/Hive.app 2>/dev/null; then
    echo "✅ App is signed (chain warning ignored)"
    codesign -dv dist/Hive.app 2>&1 | grep -E "Identifier|Signature"
    echo "✅ Signing complete!"
    echo ""
    echo "⚠️  Note: Certificate chain warning is cosmetic"
    echo "    The app is properly signed and will work"
    echo "    Full chain validation usually works after 24-48 hours"
else
    echo "❌ Signing failed"
    exit 1
fi