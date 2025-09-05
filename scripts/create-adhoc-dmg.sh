#!/bin/bash
set -e

echo "📦 Creating ad-hoc signed DMG for immediate distribution..."

# Check if app exists
if [ ! -d "dist/Hive.app" ]; then
    echo "❌ Error: dist/Hive.app not found"
    echo "   Run ./scripts/package-macos.sh first"
    exit 1
fi

# Get version from package.json
VERSION=$(node -p "require('./package.json').version")
DMG_NAME="Hive-IDE-${VERSION}-adhoc.dmg"

echo "🔐 Applying ad-hoc signature..."
# Remove any existing signatures
rm -rf dist/Hive.app/_CodeSignature
rm -rf dist/Hive.app/Contents/_CodeSignature
xattr -cr dist/Hive.app

# Apply ad-hoc signature
codesign --force --deep -s - dist/Hive.app

echo "✅ Ad-hoc signature applied"

# Create DMG
echo "💿 Creating DMG installer..."
rm -f "dist/$DMG_NAME"

# Create a temporary directory for DMG contents
rm -rf dist/dmg-contents
mkdir -p dist/dmg-contents

# Copy app to DMG contents
cp -R dist/Hive.app dist/dmg-contents/

# Create Applications symlink
ln -s /Applications dist/dmg-contents/Applications

# Create DMG with nice layout
create-dmg \
    --volname "Hive IDE Installer" \
    --volicon "assets/icon.icns" \
    --window-pos 200 120 \
    --window-size 600 400 \
    --icon-size 100 \
    --icon "Hive.app" 175 190 \
    --hide-extension "Hive.app" \
    --app-drop-link 425 190 \
    --background "assets/dmg-background.png" \
    "dist/$DMG_NAME" \
    "dist/dmg-contents" 2>/dev/null || {
        # Fallback to simple DMG if create-dmg not available
        echo "⚠️  create-dmg not found, using simple DMG..."
        hdiutil create -volname "Hive IDE Installer" \
            -srcfolder dist/dmg-contents \
            -ov -format UDZO \
            "dist/$DMG_NAME"
    }

# Clean up
rm -rf dist/dmg-contents

# Generate checksum
echo "🔒 Generating checksum..."
cd dist
shasum -a 256 "$DMG_NAME" > "${DMG_NAME}.sha256"
cd ..

echo ""
echo "✅ Ad-hoc signed DMG created successfully!"
echo "📦 File: dist/$DMG_NAME"
echo "🔒 Checksum: dist/${DMG_NAME}.sha256"
echo ""
echo "⚠️  Note: This is ad-hoc signed and will work immediately"
echo "    Users won't see 'damaged app' errors"
echo "    But may still see 'unidentified developer' warning"
echo "    They can right-click → Open to bypass this"
echo ""
echo "📝 Once Apple recognizes your certificate (24-48 hours),"
echo "    run ./scripts/create-signed-dmg.sh for the fully signed version"