#!/bin/bash
set -e

echo "🚀 Building Hive IDE for all platforms..."

# Configuration
VERSION="${HIVE_VERSION:-2.0.2}"

echo "📋 Build configuration:"
echo "   Version: $VERSION"
echo "   Date: $(date)"
echo ""

# Create dist directory
mkdir -p dist

# Build macOS (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 Building macOS distribution..."
    ./scripts/build-macos-complete.sh
else
    echo "⚠️  Skipping macOS build (not on macOS)"
fi

# Prepare Windows MSI configuration (requires Windows to actually build)
echo "🪟 Preparing Windows MSI configuration..."
./scripts/build-windows-msi.sh

# Prepare Linux AppImage configuration (requires Linux to actually build)
echo "🐧 Preparing Linux AppImage configuration..."
echo "ℹ️  Linux AppImage builder configured (requires Linux to build)"

# Generate comprehensive checksums
echo "🔐 Generating comprehensive checksums..."
cd dist

# Create checksums for all distribution files
CHECKSUM_FILE="checksums-$VERSION.sha256"
rm -f "$CHECKSUM_FILE"

echo "# Hive IDE $VERSION - SHA256 Checksums" > "$CHECKSUM_FILE"
echo "# Generated on $(date)" >> "$CHECKSUM_FILE"
echo "# Verify with: shasum -c $CHECKSUM_FILE" >> "$CHECKSUM_FILE"
echo "" >> "$CHECKSUM_FILE"

# Add checksums for existing files
for file in *.dmg *.tar.gz *.msi *.AppImage *.exe; do
    if [ -f "$file" ]; then
        echo "Adding checksum for: $file"
        shasum -a 256 "$file" >> "$CHECKSUM_FILE"
    fi
done

cd ..

# Create release info JSON
echo "📄 Creating release metadata..."
cat > "dist/release-$VERSION.json" << EOF
{
  "version": "$VERSION",
  "build_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "macos": {
      "dmg": "Hive-IDE-$VERSION.dmg",
      "archive": "hive-macos-arm64.app.tar.gz",
      "signed": true,
      "notarized": false,
      "requirements": "macOS 11.0 or later, Apple Silicon or Intel"
    },
    "windows": {
      "installer": "Hive-IDE-$VERSION.msi",
      "binary": "hive-windows-x64.exe",
      "signed": false,
      "requirements": "Windows 10/11 64-bit, One-time SmartScreen approval"
    },
    "linux": {
      "appimage": "Hive-IDE-$VERSION-x86_64.AppImage",
      "universal": true,
      "requirements": "Any 64-bit Linux distribution"
    }
  },
  "checksums_file": "checksums-$VERSION.sha256",
  "download_base_url": "https://releases.hivetechs.io/stable/"
}
EOF

echo "✅ Multi-platform build preparation complete!"
echo ""
echo "📦 Distribution files prepared:"
ls -la dist/ | grep -E '\.(dmg|msi|AppImage|tar\.gz|exe)$' || echo "   No distribution files found yet"
echo ""
echo "🔑 Checksums file: dist/checksums-$VERSION.sha256"
echo "📄 Release metadata: dist/release-$VERSION.json"
echo ""
echo "🎯 Next steps by platform:"
echo ""
echo "macOS:"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "   ✅ Ready to test and upload"
else
    echo "   ⏳ Build on macOS machine"
fi
echo ""
echo "Windows:"
echo "   1. Cross-compile: cargo build --target x86_64-pc-windows-gnu"
echo "   2. Run dist/windows-installer/build.bat on Windows"
echo "   3. Test MSI installer"
echo ""
echo "Linux:"
echo "   1. Cross-compile: cargo build --target x86_64-unknown-linux-musl"
echo "   2. Run build-linux-appimage.sh on Linux"
echo "   3. Test on multiple distributions"
echo ""
echo "🌐 Final step: Upload all files to R2 and update downloads page"