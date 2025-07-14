#!/bin/bash
set -e

echo "🌐 Updating R2 distribution structure for professional releases..."

# Configuration
VERSION="${HIVE_VERSION:-2.0.2}"
BUCKET="releases-hivetechs"

echo "📋 R2 Configuration:"
echo "   Bucket: $BUCKET"
echo "   Version: $VERSION"
echo "   Domain: https://releases.hivetechs.io"
echo ""

# Create new releases.json with professional structure
echo "📄 Creating professional releases.json..."
cat > "dist/releases.json" << EOF
{
  "latest_version": "$VERSION",
  "last_updated": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "channels": {
    "stable": {
      "version": "$VERSION",
      "release_date": "$(date -u +%Y-%m-%d)",
      "platforms": {
        "darwin": {
          "arm64": {
            "dmg": "stable/Hive-IDE-$VERSION.dmg",
            "dmg_sha256": "stable/Hive-IDE-$VERSION.dmg.sha256",
            "archive": "stable/hive-macos-arm64.app.tar.gz",
            "archive_sha256": "stable/hive-macos-arm64.app.tar.gz.sha256",
            "signed": true,
            "notarized": false,
            "size_mb": 15,
            "requirements": "macOS 11.0+, Apple Silicon"
          },
          "x64": {
            "dmg": "stable/Hive-IDE-$VERSION-Intel.dmg",
            "dmg_sha256": "stable/Hive-IDE-$VERSION-Intel.dmg.sha256",
            "archive": "stable/hive-macos-intel.app.tar.gz",
            "archive_sha256": "stable/hive-macos-intel.app.tar.gz.sha256",
            "signed": true,
            "notarized": false,
            "size_mb": 15,
            "requirements": "macOS 11.0+, Intel"
          }
        },
        "windows": {
          "x64": {
            "msi": "stable/Hive-IDE-$VERSION.msi",
            "msi_sha256": "stable/Hive-IDE-$VERSION.msi.sha256",
            "exe": "stable/hive-windows-x64.exe",
            "exe_sha256": "stable/hive-windows-x64.exe.sha256",
            "signed": false,
            "size_mb": 12,
            "requirements": "Windows 10/11 64-bit",
            "install_notes": "One-time SmartScreen approval required"
          }
        },
        "linux": {
          "x64": {
            "appimage": "stable/Hive-IDE-$VERSION-x86_64.AppImage",
            "appimage_sha256": "stable/Hive-IDE-$VERSION-x86_64.AppImage.sha256",
            "archive": "stable/hive-linux-x64.tar.gz",
            "archive_sha256": "stable/hive-linux-x64.tar.gz.sha256",
            "size_mb": 18,
            "requirements": "64-bit Linux, glibc 2.17+"
          }
        }
      }
    },
    "beta": {
      "version": "$VERSION-beta",
      "release_date": "$(date -u +%Y-%m-%d)",
      "platforms": {
        "darwin": {
          "arm64": {
            "dmg": "beta/Hive-IDE-$VERSION-beta.dmg",
            "dmg_sha256": "beta/Hive-IDE-$VERSION-beta.dmg.sha256"
          }
        },
        "windows": {
          "x64": {
            "msi": "beta/Hive-IDE-$VERSION-beta.msi",
            "msi_sha256": "beta/Hive-IDE-$VERSION-beta.msi.sha256"
          }
        },
        "linux": {
          "x64": {
            "appimage": "beta/Hive-IDE-$VERSION-beta-x86_64.AppImage",
            "appimage_sha256": "beta/Hive-IDE-$VERSION-beta-x86_64.AppImage.sha256"
          }
        }
      }
    }
  },
  "checksums": {
    "stable": "stable/checksums-$VERSION.sha256",
    "beta": "beta/checksums-$VERSION-beta.sha256"
  },
  "release_notes_url": "https://github.com/hivetechs/hive/releases/tag/v$VERSION",
  "documentation_url": "https://hivetechs.io/documentation",
  "support_url": "https://hivetechs.io/support"
}
EOF

# Upload releases.json to R2
echo "📤 Uploading releases.json to R2..."
if command -v wrangler &> /dev/null; then
    wrangler r2 object put "$BUCKET/releases.json" --file="dist/releases.json" --remote || echo "⚠️  Upload failed - check wrangler setup"
else
    echo "⚠️  Wrangler not found - cannot upload to R2"
    echo "   Install with: npm install -g wrangler"
fi

# Create upload script for stable releases
echo "📜 Creating R2 upload script..."
cat > "scripts/upload-stable-release.sh" << EOF
#!/bin/bash
set -e

echo "📤 Uploading stable release to R2..."

VERSION="\${HIVE_VERSION:-$VERSION}"
BUCKET="$BUCKET"

# Check if wrangler is available
if ! command -v wrangler &> /dev/null; then
    echo "❌ Wrangler not found. Install with: npm install -g wrangler"
    exit 1
fi

# Upload all distribution files
echo "📦 Uploading distribution files..."

# macOS files
if [ -f "dist/Hive-IDE-\$VERSION.dmg" ]; then
    echo "🍎 Uploading macOS DMG..."
    wrangler r2 object put "\$BUCKET/stable/Hive-IDE-\$VERSION.dmg" --file="dist/Hive-IDE-\$VERSION.dmg" --remote
    wrangler r2 object put "\$BUCKET/stable/Hive-IDE-\$VERSION.dmg.sha256" --file="dist/Hive-IDE-\$VERSION.dmg.sha256" --remote
fi

if [ -f "dist/hive-macos-arm64.app.tar.gz" ]; then
    echo "🍎 Uploading macOS ARM64 archive..."
    wrangler r2 object put "\$BUCKET/stable/hive-macos-arm64.app.tar.gz" --file="dist/hive-macos-arm64.app.tar.gz" --remote
    wrangler r2 object put "\$BUCKET/stable/hive-macos-arm64.app.tar.gz.sha256" --file="dist/hive-macos-arm64.app.tar.gz.sha256" --remote
fi

# Windows files
if [ -f "dist/Hive-IDE-\$VERSION.msi" ]; then
    echo "🪟 Uploading Windows MSI..."
    wrangler r2 object put "\$BUCKET/stable/Hive-IDE-\$VERSION.msi" --file="dist/Hive-IDE-\$VERSION.msi" --remote
    wrangler r2 object put "\$BUCKET/stable/Hive-IDE-\$VERSION.msi.sha256" --file="dist/Hive-IDE-\$VERSION.msi.sha256" --remote
fi

if [ -f "dist/hive-windows-x64.exe" ]; then
    echo "🪟 Uploading Windows executable..."
    wrangler r2 object put "\$BUCKET/stable/hive-windows-x64.exe" --file="dist/hive-windows-x64.exe" --remote
    wrangler r2 object put "\$BUCKET/stable/hive-windows-x64.exe.sha256" --file="dist/hive-windows-x64.exe.sha256" --remote
fi

# Linux files
if [ -f "dist/Hive-IDE-\$VERSION-x86_64.AppImage" ]; then
    echo "🐧 Uploading Linux AppImage..."
    wrangler r2 object put "\$BUCKET/stable/Hive-IDE-\$VERSION-x86_64.AppImage" --file="dist/Hive-IDE-\$VERSION-x86_64.AppImage" --remote
    wrangler r2 object put "\$BUCKET/stable/Hive-IDE-\$VERSION-x86_64.AppImage.sha256" --file="dist/Hive-IDE-\$VERSION-x86_64.AppImage.sha256" --remote
fi

# Upload checksums
if [ -f "dist/checksums-\$VERSION.sha256" ]; then
    echo "🔐 Uploading checksums..."
    wrangler r2 object put "\$BUCKET/stable/checksums-\$VERSION.sha256" --file="dist/checksums-\$VERSION.sha256" --remote
fi

# Upload releases.json
if [ -f "dist/releases.json" ]; then
    echo "📄 Uploading releases.json..."
    wrangler r2 object put "\$BUCKET/releases.json" --file="dist/releases.json" --remote
fi

echo "✅ Upload complete!"
echo "🌐 Files available at: https://releases.hivetechs.io/"
EOF

chmod +x "scripts/upload-stable-release.sh"

echo "✅ R2 structure updated!"
echo ""
echo "📂 New R2 structure:"
echo "releases.hivetechs.io/"
echo "├── releases.json (metadata)"
echo "├── stable/"
echo "│   ├── Hive-IDE-$VERSION.dmg"
echo "│   ├── Hive-IDE-$VERSION.msi"
echo "│   ├── Hive-IDE-$VERSION-x86_64.AppImage"
echo "│   └── checksums-$VERSION.sha256"
echo "└── beta/"
echo "    └── (same structure)"
echo ""
echo "🎯 Next steps:"
echo "1. Build distribution files for each platform"
echo "2. Run ./scripts/upload-stable-release.sh"
echo "3. Update downloads page to use new URLs"
echo ""
echo "📱 Auto-updater will use: https://releases.hivetechs.io/releases.json"