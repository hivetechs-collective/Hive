#!/bin/bash
set -e

echo "🐧 Creating Linux AppImage for Hive IDE..."

# Configuration
VERSION="${HIVE_VERSION:-2.0.2}"
APP_NAME="Hive-IDE"
APPDIR="$APP_NAME.AppDir"

# Create directories
mkdir -p "dist/$APPDIR"
mkdir -p "dist/$APPDIR/usr/bin"
mkdir -p "dist/$APPDIR/usr/share/applications"
mkdir -p "dist/$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "dist/$APPDIR/usr/share/metainfo"

echo "📋 Preparing AppImage structure..."

# Check for Linux binary
if [ ! -f "dist/hive-linux-x64" ]; then
    echo "❌ Linux binary not found: dist/hive-linux-x64"
    echo "   You'll need to cross-compile for Linux first:"
    echo "   cargo build --release --target x86_64-unknown-linux-musl --features desktop"
    echo "   Or build on a Linux machine"
    exit 1
fi

# Copy binary
echo "📋 Copying binary..."
cp "dist/hive-linux-x64" "dist/$APPDIR/usr/bin/hive"
chmod +x "dist/$APPDIR/usr/bin/hive"

# Create AppRun script
echo "📝 Creating AppRun script..."
cat > "dist/$APPDIR/AppRun" << 'EOF'
#!/bin/bash
# AppRun script for Hive IDE

# Get the directory where this AppImage is located
HERE="$(dirname "$(readlink -f "${0}")")"

# Export required environment variables
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"

# Run the application
exec "${HERE}/usr/bin/hive" "$@"
EOF

chmod +x "dist/$APPDIR/AppRun"

# Create desktop file
echo "🖥️ Creating desktop file..."
cat > "dist/$APPDIR/hive-ide.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Hive IDE
Comment=AI-powered development environment with 4-stage consensus
Exec=hive
Icon=hive-ide
StartupNotify=true
Categories=Development;IDE;TextEditor;
Keywords=AI;development;consensus;programming;IDE;
Terminal=false
MimeType=text/plain;text/x-source;
EOF

# Copy desktop file to standard location
cp "dist/$APPDIR/hive-ide.desktop" "dist/$APPDIR/usr/share/applications/"

# Create icon (simple SVG)
echo "🎨 Creating application icon..."
cat > "dist/$APPDIR/usr/share/icons/hicolor/256x256/apps/hive-ide.svg" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg width="256" height="256" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
  <!-- Background circle -->
  <circle cx="128" cy="128" r="120" fill="#f39c12" stroke="#e67e22" stroke-width="4"/>
  
  <!-- Hexagon (hive pattern) -->
  <polygon points="128,40 180,70 180,130 128,160 76,130 76,70" 
           fill="#fff" stroke="#e67e22" stroke-width="3"/>
  
  <!-- Inner hexagons -->
  <polygon points="128,70 155,85 155,115 128,130 101,115 101,85" 
           fill="#f39c12" stroke="#e67e22" stroke-width="2"/>
  
  <!-- Center dot -->
  <circle cx="128" cy="100" r="8" fill="#e67e22"/>
  
  <!-- Text -->
  <text x="128" y="200" text-anchor="middle" 
        font-family="Arial, sans-serif" font-size="24" font-weight="bold" fill="#2c3e50">
    HIVE
  </text>
</svg>
EOF

# Create PNG icon from SVG (if available)
if command -v convert &> /dev/null; then
    echo "🖼️ Converting icon to PNG..."
    convert "dist/$APPDIR/usr/share/icons/hicolor/256x256/apps/hive-ide.svg" \
            "dist/$APPDIR/usr/share/icons/hicolor/256x256/apps/hive-ide.png"
else
    echo "⚠️  ImageMagick not available, keeping SVG icon"
fi

# Create symlink for icon in root
ln -sf "usr/share/icons/hicolor/256x256/apps/hive-ide.svg" "dist/$APPDIR/hive-ide.svg"

# Create AppStream metainfo
echo "📄 Creating AppStream metadata..."
cat > "dist/$APPDIR/usr/share/metainfo/io.hivetechs.hive-ide.appdata.xml" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>io.hivetechs.hive-ide</id>
  <metadata_license>MIT</metadata_license>
  <project_license>Proprietary</project_license>
  <name>Hive IDE</name>
  <summary>AI-powered development environment</summary>
  <description>
    <p>
      Hive IDE is an advanced AI-powered development environment featuring 
      revolutionary 4-stage consensus technology. Access 400+ AI models 
      from 60+ providers with intelligent selection and cost optimization.
    </p>
    <p>Features:</p>
    <ul>
      <li>4-stage AI consensus pipeline for accurate results</li>
      <li>Integration with 400+ AI models via OpenRouter</li>
      <li>Repository intelligence and code analysis</li>
      <li>Terminal user interface (TUI) mode</li>
      <li>Memory and context management</li>
    </ul>
  </description>
  <launchable type="desktop-id">hive-ide.desktop</launchable>
  <screenshots>
    <screenshot type="default">
      <caption>Hive IDE main interface</caption>
    </screenshot>
  </screenshots>
  <url type="homepage">https://hivetechs.io</url>
  <url type="bugtracker">https://github.com/hivetechs/hive/issues</url>
  <developer_name>HiveTechs Collective LLC</developer_name>
  <releases>
    <release version="$VERSION" date="$(date +%Y-%m-%d)"/>
  </releases>
  <content_rating type="oars-1.1"/>
</component>
EOF

# Download appimagetool if not available
APPIMAGETOOL="dist/appimagetool-x86_64.AppImage"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "📥 Downloading appimagetool..."
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" \
         -O "$APPIMAGETOOL"
    chmod +x "$APPIMAGETOOL"
fi

# Build AppImage
echo "🔨 Building AppImage..."
cd dist

# Set environment variable to skip desktop integration
export NO_STRIP=1

# Build the AppImage
./"$(basename "$APPIMAGETOOL")" "$APPDIR" "$APP_NAME-$VERSION-x86_64.AppImage"

# Move back to project root
cd ..

# Calculate checksum
echo "🔐 Calculating checksum..."
cd dist
shasum -a 256 "$APP_NAME-$VERSION-x86_64.AppImage" > "$APP_NAME-$VERSION-x86_64.AppImage.sha256"
cd ..

# Clean up
rm -rf "dist/$APPDIR"

echo "✅ Linux AppImage created successfully!"
echo "📦 File: dist/$APP_NAME-$VERSION-x86_64.AppImage"
echo "🔑 Checksum: dist/$APP_NAME-$VERSION-x86_64.AppImage.sha256"
echo "📊 Size: $(du -h "dist/$APP_NAME-$VERSION-x86_64.AppImage" | cut -f1)"

echo ""
echo "🎯 Usage instructions for users:"
echo "1. Download the AppImage file"
echo "2. Make it executable: chmod +x $APP_NAME-$VERSION-x86_64.AppImage"
echo "3. Run it: ./$APP_NAME-$VERSION-x86_64.AppImage"
echo ""
echo "✨ The AppImage will work on any Linux distribution!"