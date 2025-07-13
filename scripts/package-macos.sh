#!/bin/bash
set -e

echo "📦 Packaging Hive for macOS..."

# Create app bundle structure
mkdir -p dist/Hive.app/Contents/MacOS
mkdir -p dist/Hive.app/Contents/Resources

# Copy binary
cp dist/hive-macos-arm64 dist/Hive.app/Contents/MacOS/hive

# Create Info.plist
cat > dist/Hive.app/Contents/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>hive</string>
    <key>CFBundleIdentifier</key>
    <string>com.hivetechs.hive</string>
    <key>CFBundleName</key>
    <string>Hive</string>
    <key>CFBundleDisplayName</key>
    <string>Hive IDE</string>
    <key>CFBundleVersion</key>
    <string>2.0.2</string>
    <key>CFBundleShortVersionString</key>
    <string>2.0.2</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>*</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>All Files</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
        </dict>
    </array>
</dict>
</plist>
EOF

# Make executable
chmod +x dist/Hive.app/Contents/MacOS/hive

echo "✅ Hive.app created successfully!"
echo "📁 Location: $(pwd)/dist/Hive.app"
echo "🚀 Ready for distribution"