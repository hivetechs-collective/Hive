#!/usr/bin/env bash
#
# macOS Package Creation Script
# Creates signed .pkg installer for HiveTechs Consensus
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/distribution/macos"
PKG_DIR="$SCRIPT_DIR/package"
VERSION="${HIVE_VERSION:-$(cargo pkgid | sed 's/.*#//')}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[PKG]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Clean previous package structure
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR"

print_status "Creating macOS package structure for version $VERSION"

# Create package root structure
mkdir -p "$PKG_DIR/package_root/usr/local/bin"
mkdir -p "$PKG_DIR/package_root/usr/local/share/bash-completion/completions"
mkdir -p "$PKG_DIR/package_root/usr/local/share/zsh/site-functions"
mkdir -p "$PKG_DIR/package_root/usr/local/share/fish/vendor_completions.d"
mkdir -p "$PKG_DIR/package_root/usr/local/share/man/man1"
mkdir -p "$PKG_DIR/package_root/Library/LaunchAgents"
mkdir -p "$PKG_DIR/scripts"

# Copy binary
cp "$BUILD_DIR/hive" "$PKG_DIR/package_root/usr/local/bin/"
chmod +x "$PKG_DIR/package_root/usr/local/bin/hive"

# Copy shell completions
if [ -d "$BUILD_DIR/completions" ]; then
    cp "$BUILD_DIR/completions/hive.bash" "$PKG_DIR/package_root/usr/local/share/bash-completion/completions/hive" || true
    cp "$BUILD_DIR/completions/_hive" "$PKG_DIR/package_root/usr/local/share/zsh/site-functions/" || true
    cp "$BUILD_DIR/completions/hive.fish" "$PKG_DIR/package_root/usr/local/share/fish/vendor_completions.d/" || true
fi

# Create man page
if command -v help2man &> /dev/null; then
    help2man --no-info --name="HiveTechs Consensus AI" "$BUILD_DIR/hive" > "$PKG_DIR/package_root/usr/local/share/man/man1/hive.1" || true
fi

# Create LaunchAgent for auto-updates
cat > "$PKG_DIR/package_root/Library/LaunchAgents/com.hivetechs.hive.update.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.hivetechs.hive.update</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/hive</string>
        <string>update</string>
        <string>--check</string>
    </array>
    <key>StartInterval</key>
    <integer>86400</integer>
    <key>RunAtLoad</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/dev/null</string>
    <key>StandardErrorPath</key>
    <string>/dev/null</string>
</dict>
</plist>
EOF

# Create postinstall script
cat > "$PKG_DIR/scripts/postinstall" << 'EOF'
#!/bin/bash

# Add /usr/local/bin to PATH if not already there
USER_SHELL_RC=""
if [ -n "$USER" ] && [ "$USER" != "root" ]; then
    USER_HOME=$(eval echo "~$USER")
    
    # Determine shell RC file
    case "$SHELL" in
        */bash)
            if [ -f "$USER_HOME/.bash_profile" ]; then
                USER_SHELL_RC="$USER_HOME/.bash_profile"
            elif [ -f "$USER_HOME/.bashrc" ]; then
                USER_SHELL_RC="$USER_HOME/.bashrc"
            fi
            ;;
        */zsh)
            USER_SHELL_RC="$USER_HOME/.zshrc"
            ;;
    esac
    
    if [ -n "$USER_SHELL_RC" ] && [ -f "$USER_SHELL_RC" ]; then
        if ! grep -q "/usr/local/bin" "$USER_SHELL_RC" 2>/dev/null; then
            echo 'export PATH="/usr/local/bin:$PATH"' >> "$USER_SHELL_RC"
        fi
    fi
fi

# Initialize Hive configuration
if [ -n "$USER" ] && [ "$USER" != "root" ]; then
    sudo -u "$USER" /usr/local/bin/hive config init --silent || true
fi

# Load LaunchAgent for current user
if [ -n "$USER" ] && [ "$USER" != "root" ]; then
    USER_HOME=$(eval echo "~$USER")
    sudo -u "$USER" launchctl load "$USER_HOME/Library/LaunchAgents/com.hivetechs.hive.update.plist" 2>/dev/null || true
fi

echo "HiveTechs Consensus installed successfully!"
echo "Run 'hive --help' to get started."

exit 0
EOF

chmod +x "$PKG_DIR/scripts/postinstall"

# Create preremove script
cat > "$PKG_DIR/scripts/preremove" << 'EOF'
#!/bin/bash

# Unload LaunchAgent
if [ -n "$USER" ] && [ "$USER" != "root" ]; then
    USER_HOME=$(eval echo "~$USER")
    sudo -u "$USER" launchctl unload "$USER_HOME/Library/LaunchAgents/com.hivetechs.hive.update.plist" 2>/dev/null || true
fi

exit 0
EOF

chmod +x "$PKG_DIR/scripts/preremove"

# Create package info
cat > "$PKG_DIR/PackageInfo" << EOF
<?xml version="1.0" encoding="utf-8"?>
<pkg-info overwrite-permissions="true" relocatable="false" identifier="com.hivetechs.hive" postinstall-action="none" version="$VERSION" format-version="2" generator-version="InstallCmds-681 (20G224)" auth="root">
    <payload installKBytes="$(du -sk "$PKG_DIR/package_root" | cut -f1)" numberOfFiles="$(find "$PKG_DIR/package_root" -type f | wc -l | tr -d ' ')"/>
    <bundle path="./usr/local/bin/hive" id="com.hivetechs.hive" CFBundleVersion="$VERSION"/>
    <bundle-version>
        <bundle id="com.hivetechs.hive"/>
    </bundle-version>
    <upgrade-bundle>
        <bundle id="com.hivetechs.hive"/>
    </upgrade-bundle>
    <update-bundle>
        <bundle id="com.hivetechs.hive"/>
    </update-bundle>
    <atomic-update-bundle>
        <bundle id="com.hivetechs.hive"/>
    </atomic-update-bundle>
    <relocate>
        <bundle id="com.hivetechs.hive"/>
    </relocate>
    <scripts>
        <postinstall file="./postinstall"/>
        <preremove file="./preremove"/>
    </scripts>
</pkg-info>
EOF

# Create Distribution.xml for custom installer experience
cat > "$PKG_DIR/Distribution.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>HiveTechs Consensus</title>
    <organization>com.hivetechs</organization>
    <domains enable_anywhere="false" enable_currentUserHome="false" enable_localSystem="true"/>
    <options customize="never" require-scripts="false" rootVolumeOnly="true"/>
    
    <welcome file="welcome.html" mime-type="text/html"/>
    <license file="license.txt" mime-type="text/plain"/>
    <readme file="readme.html" mime-type="text/html"/>
    
    <pkg-ref id="com.hivetechs.hive"/>
    <options customize="never" require-scripts="false"/>
    
    <choices-outline>
        <line choice="default">
            <line choice="com.hivetechs.hive"/>
        </line>
    </choices-outline>
    
    <choice id="default"/>
    <choice id="com.hivetechs.hive" visible="false">
        <pkg-ref id="com.hivetechs.hive"/>
    </choice>
    
    <pkg-ref id="com.hivetechs.hive" version="$VERSION" onConclusion="none">hive-$VERSION.pkg</pkg-ref>
    
    <product id="com.hivetechs.hive" version="$VERSION"/>
</installer-gui-script>
EOF

# Create welcome.html
cat > "$PKG_DIR/welcome.html" << EOF
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Welcome to HiveTechs Consensus</title>
    <style type="text/css">
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; font-size: 13px; }
        h1 { color: #333; }
        .highlight { background-color: #f0f8ff; padding: 10px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>🐝 Welcome to HiveTechs Consensus</h1>
    <p>Thank you for choosing HiveTechs Consensus, the AI-powered codebase intelligence platform.</p>
    
    <div class="highlight">
        <h3>What's being installed:</h3>
        <ul>
            <li><strong>hive</strong> - The main CLI application</li>
            <li><strong>Shell completions</strong> - For bash, zsh, and fish</li>
            <li><strong>Auto-update agent</strong> - Keeps your installation current</li>
            <li><strong>Man page</strong> - Comprehensive documentation</li>
        </ul>
    </div>
    
    <h3>Getting Started:</h3>
    <ol>
        <li>Open Terminal</li>
        <li>Run <code>hive --help</code> to see available commands</li>
        <li>Configure your API keys with <code>hive config setup</code></li>
        <li>Try the TUI mode by running <code>hive</code> without arguments</li>
    </ol>
    
    <p>For more information, visit <a href="https://github.com/hivetechs/hive">github.com/hivetechs/hive</a></p>
</body>
</html>
EOF

# Create license.txt
cat > "$PKG_DIR/license.txt" << EOF
HiveTechs Consensus License Agreement

Copyright (c) 2024 HiveTechs Collective

This software is proprietary and confidential. Unauthorized copying,
distribution, or use is strictly prohibited.

By installing this software, you agree to the terms and conditions
outlined in the full license agreement available at:
https://hivetechs.com/license

For questions about licensing, contact: team@hivetechs.com
EOF

# Create readme.html
cat > "$PKG_DIR/readme.html" << EOF
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>HiveTechs Consensus - Installation Complete</title>
    <style type="text/css">
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; font-size: 13px; }
        h1 { color: #333; }
        code { background-color: #f5f5f5; padding: 2px 4px; border-radius: 3px; }
        .section { margin: 20px 0; }
    </style>
</head>
<body>
    <h1>🎉 Installation Complete!</h1>
    
    <div class="section">
        <h3>Quick Start Commands:</h3>
        <ul>
            <li><code>hive --version</code> - Check installation</li>
            <li><code>hive config setup</code> - Configure API keys</li>
            <li><code>hive ask "Hello World"</code> - Ask a question</li>
            <li><code>hive analyze .</code> - Analyze current directory</li>
            <li><code>hive</code> - Launch TUI mode</li>
        </ul>
    </div>
    
    <div class="section">
        <h3>Configuration:</h3>
        <p>Your configuration files are stored in <code>~/.hive/</code></p>
        <p>Auto-updates are enabled by default and check daily.</p>
    </div>
    
    <div class="section">
        <h3>Uninstalling:</h3>
        <p>To uninstall, run: <code>sudo pkgutil --forget com.hivetechs.hive</code></p>
        <p>Then manually remove: <code>/usr/local/bin/hive</code></p>
    </div>
    
    <div class="section">
        <h3>Support:</h3>
        <p>Documentation: <a href="https://github.com/hivetechs/hive">github.com/hivetechs/hive</a></p>
        <p>Issues: <a href="https://github.com/hivetechs/hive/issues">github.com/hivetechs/hive/issues</a></p>
    </div>
</body>
</html>
EOF

print_status "Building component package..."

# Build the component package
pkgbuild \
    --root "$PKG_DIR/package_root" \
    --scripts "$PKG_DIR/scripts" \
    --identifier "com.hivetechs.hive" \
    --version "$VERSION" \
    --install-location "/" \
    "$PKG_DIR/hive-$VERSION.pkg"

print_status "Building distribution package..."

# Build the distribution package with custom installer
productbuild \
    --distribution "$PKG_DIR/Distribution.xml" \
    --package-path "$PKG_DIR" \
    --resources "$PKG_DIR" \
    "$BUILD_DIR/HiveTechs-Consensus-$VERSION.pkg"

# Sign the package if developer certificate is available
DEVELOPER_ID=$(security find-identity -v -p codesigning | grep "Developer ID Installer" | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "")

if [ -n "$DEVELOPER_ID" ]; then
    print_status "Signing package with: $DEVELOPER_ID"
    
    productsign \
        --sign "$DEVELOPER_ID" \
        "$BUILD_DIR/HiveTechs-Consensus-$VERSION.pkg" \
        "$BUILD_DIR/HiveTechs-Consensus-$VERSION-signed.pkg"
    
    # Replace unsigned with signed version
    mv "$BUILD_DIR/HiveTechs-Consensus-$VERSION-signed.pkg" "$BUILD_DIR/HiveTechs-Consensus-$VERSION.pkg"
    
    print_success "Package signed successfully"
else
    print_status "No Developer ID Installer certificate found - package not signed"
fi

# Create checksum
cd "$BUILD_DIR"
shasum -a 256 "HiveTechs-Consensus-$VERSION.pkg" > "HiveTechs-Consensus-$VERSION.pkg.sha256"

print_success "macOS package created: $BUILD_DIR/HiveTechs-Consensus-$VERSION.pkg"

# Test package (optional)
if command -v installer &> /dev/null; then
    print_status "Testing package structure..."
    installer -pkg "$BUILD_DIR/HiveTechs-Consensus-$VERSION.pkg" -target / -dumplog 2>/dev/null || true
fi

# Notarization instructions
if [ -n "$DEVELOPER_ID" ]; then
    echo
    echo "📋 Next steps for distribution:"
    echo "1. Upload for notarization:"
    echo "   xcrun notarytool submit HiveTechs-Consensus-$VERSION.pkg --keychain-profile \"notarytool-profile\" --wait"
    echo "2. Staple the notarization:"
    echo "   xcrun stapler staple HiveTechs-Consensus-$VERSION.pkg"
    echo "3. Verify notarization:"
    echo "   xcrun stapler validate HiveTechs-Consensus-$VERSION.pkg"
fi