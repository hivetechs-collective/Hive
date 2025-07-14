#!/bin/bash
set -e

echo "📦 Creating professional DMG for Hive IDE..."

# Configuration
APP_NAME="Hive IDE"
DMG_NAME="Hive-IDE"
VERSION="${HIVE_VERSION:-2.0.2}"
BUNDLE_NAME="Hive.app"
TEMP_DMG="temp.dmg"
FINAL_DMG="$DMG_NAME-$VERSION.dmg"

# Create directories
mkdir -p dist/dmg-staging
mkdir -p dist/dmg-background

# Clean previous DMG files
rm -f "dist/$TEMP_DMG" "dist/$FINAL_DMG"

# Ensure we have the app bundle
if [ ! -d "dist/$BUNDLE_NAME" ]; then
    echo "❌ Error: dist/$BUNDLE_NAME not found"
    echo "   Run ./scripts/package-macos.sh first"
    exit 1
fi

# Copy app bundle to staging area
echo "📋 Copying app bundle to staging area..."
cp -R "dist/$BUNDLE_NAME" "dist/dmg-staging/"

# Create Applications symlink
echo "🔗 Creating Applications symlink..."
ln -sf /Applications "dist/dmg-staging/Applications"

# Create background image (simple gradient)
echo "🎨 Creating DMG background..."
cat > "dist/dmg-background/create_background.py" << 'EOF'
from PIL import Image, ImageDraw
import sys

# Create 640x400 background with gradient
width, height = 640, 400
image = Image.new('RGB', (width, height), '#f8f9fa')

# Create subtle gradient
draw = ImageDraw.Draw(image)
for y in range(height):
    color_value = int(248 - (y / height) * 8)  # Subtle gradient
    color = (color_value, color_value + 1, color_value + 2)
    draw.line([(0, y), (width, y)], fill=color)

# Save background
image.save('dist/dmg-background/background.png')
print("Background created successfully")
EOF

# Create background if Python/PIL available, otherwise use solid color
if command -v python3 &> /dev/null && python3 -c "import PIL" 2>/dev/null; then
    python3 "dist/dmg-background/create_background.py"
else
    echo "⚠️  PIL not available, using solid background"
    # Create simple solid background
    echo "Creating simple background with sips..."
    sips -s format png --setProperty pixelsW 640 --setProperty pixelsH 400 -s formatOptions 70 /System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericDocumentIcon.icns --out dist/dmg-background/background.png 2>/dev/null || true
fi

# Create DMG with staging area
echo "🗜️ Creating temporary DMG..."
hdiutil create -volname "$APP_NAME" \
    -srcfolder "dist/dmg-staging" \
    -ov -format UDRW \
    "dist/$TEMP_DMG"

# Mount the DMG
echo "📀 Mounting DMG for customization..."
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "dist/$TEMP_DMG" | grep '/Volumes' | awk '{$1=$2=""; print $0}' | xargs)
echo "Mounted at: $MOUNT_DIR"

# Set DMG window properties
echo "🖼️ Customizing DMG appearance..."
cat > /tmp/dmg_setup.applescript << EOF
tell application "Finder"
    tell disk "$APP_NAME"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set the bounds of container window to {400, 100, 1040, 500}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 128
        if exists file "background.png" then
            set background picture of viewOptions to file "background.png"
        end if
        set position of item "Hive.app" of container window to {160, 220}
        set position of item "Applications" of container window to {480, 220}
        close
        open
        update without registering applications
        delay 2
    end tell
end tell
EOF

# Copy background if it exists
if [ -f "dist/dmg-background/background.png" ]; then
    cp "dist/dmg-background/background.png" "$MOUNT_DIR/.background.png"
    # Make it hidden
    SetFile -a V "$MOUNT_DIR/.background.png" 2>/dev/null || true
fi

# Apply DMG customization
osascript /tmp/dmg_setup.applescript 2>/dev/null || echo "⚠️  Could not customize DMG appearance"

# Hide background folder
if [ -d "$MOUNT_DIR/.background" ]; then
    SetFile -a V "$MOUNT_DIR/.background" 2>/dev/null || true
fi

# Sync and unmount
echo "💾 Syncing and unmounting..."
sync
hdiutil detach "$MOUNT_DIR"

# Convert to final read-only DMG
echo "🔒 Converting to final DMG..."
hdiutil convert "dist/$TEMP_DMG" \
    -format UDZO \
    -imagekey zlib-level=9 \
    -o "dist/$FINAL_DMG"

# Clean up
rm -f "dist/$TEMP_DMG"
rm -rf "dist/dmg-staging"
rm -rf "dist/dmg-background"
rm -f /tmp/dmg_setup.applescript

# Sign the DMG if certificate is available
if security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
    echo "🔐 Signing DMG..."
    CERT_NAME=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | sed 's/.*"\(.*\)".*/\1/')
    
    codesign --force \
        --sign "$CERT_NAME" \
        --options runtime \
        --timestamp \
        "dist/$FINAL_DMG" || echo "⚠️  DMG signing failed"
else
    echo "⚠️  No code signing certificate found - DMG not signed"
fi

# Calculate checksum
echo "🔐 Calculating checksum..."
cd dist
shasum -a 256 "$FINAL_DMG" > "$FINAL_DMG.sha256"
cd ..

# Display results
echo "✅ DMG creation complete!"
echo "📦 File: dist/$FINAL_DMG"
echo "🔑 Checksum: dist/$FINAL_DMG.sha256"
echo "📊 Size: $(du -h "dist/$FINAL_DMG" | cut -f1)"

# Verify DMG
echo "🔍 Verifying DMG..."
if hdiutil verify "dist/$FINAL_DMG" >/dev/null 2>&1; then
    echo "✅ DMG verification successful"
else
    echo "❌ DMG verification failed"
    exit 1
fi

echo "🚀 DMG ready for distribution!"