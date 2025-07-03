#!/usr/bin/env bash
#
# Debian Package Creation Script
# Creates .deb package for HiveTechs Consensus
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/distribution/linux"
DEB_DIR="$SCRIPT_DIR/deb-package"
VERSION="${HIVE_VERSION:-$(cargo pkgid | sed 's/.*#//')}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[DEB]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Clean previous package structure
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR"

print_status "Creating Debian package structure for version $VERSION"

# Create package directory structure
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/local/bin"
mkdir -p "$DEB_DIR/usr/share/bash-completion/completions"
mkdir -p "$DEB_DIR/usr/share/zsh/site-functions"
mkdir -p "$DEB_DIR/usr/share/fish/vendor_completions.d"
mkdir -p "$DEB_DIR/usr/share/man/man1"
mkdir -p "$DEB_DIR/usr/share/applications"
mkdir -p "$DEB_DIR/usr/share/doc/hive-ai"
mkdir -p "$DEB_DIR/etc/systemd/system"
mkdir -p "$DEB_DIR/etc/bash_completion.d"

# Copy binary
cp "$BUILD_DIR/hive" "$DEB_DIR/usr/local/bin/"
chmod +x "$DEB_DIR/usr/local/bin/hive"

# Copy shell completions if they exist
if [ -d "$BUILD_DIR/completions" ]; then
    cp "$BUILD_DIR/completions/hive.bash" "$DEB_DIR/usr/share/bash-completion/completions/hive" 2>/dev/null || true
    cp "$BUILD_DIR/completions/hive.bash" "$DEB_DIR/etc/bash_completion.d/hive" 2>/dev/null || true
    cp "$BUILD_DIR/completions/_hive" "$DEB_DIR/usr/share/zsh/site-functions/" 2>/dev/null || true
    cp "$BUILD_DIR/completions/hive.fish" "$DEB_DIR/usr/share/fish/vendor_completions.d/" 2>/dev/null || true
fi

# Copy man page if it exists
if [ -f "$BUILD_DIR/man/hive.1" ]; then
    cp "$BUILD_DIR/man/hive.1" "$DEB_DIR/usr/share/man/man1/"
    gzip -9 "$DEB_DIR/usr/share/man/man1/hive.1"
fi

# Copy desktop file if it exists
if [ -f "$BUILD_DIR/desktop/hive.desktop" ]; then
    cp "$BUILD_DIR/desktop/hive.desktop" "$DEB_DIR/usr/share/applications/"
fi

# Create control file
cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: hive-ai
Version: $VERSION
Section: devel
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.17)
Maintainer: HiveTechs Collective <team@hivetechs.com>
Description: AI-powered codebase intelligence platform
 HiveTechs Consensus provides AI-powered code analysis, transformation,
 and intelligence capabilities with multi-model consensus for maximum
 accuracy and reliability.
 .
 Features include:
  - Multi-model AI consensus for accurate code analysis
  - Repository-wide semantic understanding
  - Intelligent code transformation and refactoring
  - Advanced memory and context management
  - Terminal User Interface (TUI) for enhanced productivity
  - Integration with popular IDEs and editors
Homepage: https://github.com/hivetechs/hive
Vcs-Git: https://github.com/hivetechs/hive.git
Vcs-Browser: https://github.com/hivetechs/hive
EOF

# Create postinst script
cat > "$DEB_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash

set -e

# Update shell completion databases
if command -v update-bash-completion >/dev/null 2>&1; then
    update-bash-completion || true
fi

# Update man page database
if command -v mandb >/dev/null 2>&1; then
    mandb -q || true
fi

# Update desktop database
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi

# Add /usr/local/bin to PATH if not already there (for current user)
if [ -n "${SUDO_USER:-}" ]; then
    USER_HOME=$(eval echo "~$SUDO_USER")
    
    # Update .bashrc
    if [ -f "$USER_HOME/.bashrc" ]; then
        if ! grep -q "/usr/local/bin" "$USER_HOME/.bashrc" 2>/dev/null; then
            echo 'export PATH="/usr/local/bin:$PATH"' >> "$USER_HOME/.bashrc"
        fi
    fi
    
    # Update .zshrc
    if [ -f "$USER_HOME/.zshrc" ]; then
        if ! grep -q "/usr/local/bin" "$USER_HOME/.zshrc" 2>/dev/null; then
            echo 'export PATH="/usr/local/bin:$PATH"' >> "$USER_HOME/.zshrc"
        fi
    fi
    
    # Initialize Hive configuration
    sudo -u "$SUDO_USER" /usr/local/bin/hive config init --silent 2>/dev/null || true
fi

echo "HiveTechs Consensus installed successfully!"
echo "Run 'hive --help' to get started."
echo "You may need to restart your shell or run 'source ~/.bashrc' to update your PATH."

EOF

chmod +x "$DEB_DIR/DEBIAN/postinst"

# Create postrm script
cat > "$DEB_DIR/DEBIAN/postrm" << 'EOF'
#!/bin/bash

set -e

case "$1" in
    remove|purge)
        # Clean up completion databases
        if command -v update-bash-completion >/dev/null 2>&1; then
            update-bash-completion || true
        fi
        
        if command -v mandb >/dev/null 2>&1; then
            mandb -q || true
        fi
        
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database /usr/share/applications || true
        fi
        
        # Remove user configuration on purge
        if [ "$1" = "purge" ]; then
            # Remove system-wide configuration
            rm -rf /etc/hive 2>/dev/null || true
            
            echo "HiveTechs Consensus has been removed."
            echo "User configuration files in ~/.hive have been preserved."
            echo "Run 'rm -rf ~/.hive' to remove them manually if desired."
        fi
        ;;
esac

EOF

chmod +x "$DEB_DIR/DEBIAN/postrm"

# Create preinst script
cat > "$DEB_DIR/DEBIAN/preinst" << 'EOF'
#!/bin/bash

set -e

# Check if we're upgrading and backup current version
if [ "$1" = "upgrade" ] && [ -f "/usr/local/bin/hive" ]; then
    echo "Backing up current version..."
    cp "/usr/local/bin/hive" "/tmp/hive.backup.$(date +%s)" 2>/dev/null || true
fi

EOF

chmod +x "$DEB_DIR/DEBIAN/preinst"

# Create prerm script
cat > "$DEB_DIR/DEBIAN/prerm" << 'EOF'
#!/bin/bash

set -e

# Stop any running hive processes
pkill -f "hive daemon" 2>/dev/null || true

EOF

chmod +x "$DEB_DIR/DEBIAN/prerm"

# Create conffiles (none for now, but good practice)
touch "$DEB_DIR/DEBIAN/conffiles"

# Create copyright file
cat > "$DEB_DIR/usr/share/doc/hive-ai/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: HiveTechs Consensus
Upstream-Contact: HiveTechs Collective <team@hivetechs.com>
Source: https://github.com/hivetechs/hive

Files: *
Copyright: 2024 HiveTechs Collective
License: Proprietary
 This software is proprietary and confidential. Unauthorized copying,
 distribution, or use is strictly prohibited.
 .
 For licensing information, contact team@hivetechs.com
EOF

# Create changelog
cat > "$DEB_DIR/usr/share/doc/hive-ai/changelog.Debian" << EOF
hive-ai ($VERSION) unstable; urgency=medium

  * Initial Debian package release
  * AI-powered codebase intelligence platform
  * Multi-model consensus engine
  * Terminal User Interface (TUI)
  * Repository semantic understanding
  * Intelligent code transformation

 -- HiveTechs Collective <team@hivetechs.com>  $(date -R)
EOF

gzip -9 "$DEB_DIR/usr/share/doc/hive-ai/changelog.Debian"

# Create README.Debian
cat > "$DEB_DIR/usr/share/doc/hive-ai/README.Debian" << EOF
HiveTechs Consensus for Debian
==============================

This package provides the HiveTechs Consensus AI-powered codebase intelligence
platform for Debian-based systems.

Configuration
-------------

After installation, run 'hive config setup' to configure your API keys and
preferences. The configuration is stored in ~/.hive/config.toml

Shell Completions
-----------------

Shell completions are automatically installed for bash, zsh, and fish.
You may need to restart your shell or source the completion files manually.

For bash: source /usr/share/bash-completion/completions/hive
For zsh: The completion should be available in your function path
For fish: The completion should be automatically available

TUI Mode
--------

HiveTechs Consensus features a Terminal User Interface (TUI) that provides
a VS Code-like experience in your terminal. Launch it by running 'hive'
without any arguments.

Auto-Updates
------------

Auto-updates are enabled by default. You can configure update behavior
with 'hive config set auto_update.enabled false' to disable them.

Documentation
-------------

For comprehensive documentation, visit:
https://github.com/hivetechs/hive

Support
-------

For support and bug reports, please visit:
https://github.com/hivetechs/hive/issues

 -- HiveTechs Collective <team@hivetechs.com>
EOF

# Create systemd service for daemon mode (optional)
cat > "$DEB_DIR/etc/systemd/system/hive-daemon.service" << EOF
[Unit]
Description=HiveTechs Consensus Daemon
After=network.target

[Service]
Type=simple
User=hive
Group=hive
ExecStart=/usr/local/bin/hive daemon
Restart=always
RestartSec=5
Environment=HIVE_CONFIG_DIR=/etc/hive

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/hive

[Install]
WantedBy=multi-user.target
EOF

# Calculate package size and installed size
PACKAGE_SIZE=$(du -sk "$DEB_DIR" | cut -f1)
echo "Installed-Size: $PACKAGE_SIZE" >> "$DEB_DIR/DEBIAN/control"

print_status "Building Debian package..."

# Fix permissions
find "$DEB_DIR" -type f -exec chmod 644 {} \;
find "$DEB_DIR" -type d -exec chmod 755 {} \;
chmod +x "$DEB_DIR/usr/local/bin/hive"
chmod +x "$DEB_DIR/DEBIAN/postinst"
chmod +x "$DEB_DIR/DEBIAN/postrm"
chmod +x "$DEB_DIR/DEBIAN/preinst"
chmod +x "$DEB_DIR/DEBIAN/prerm"

# Build the package
fakeroot dpkg-deb --build "$DEB_DIR" "$BUILD_DIR/hive-ai_${VERSION}_amd64.deb"

print_success "Debian package created: $BUILD_DIR/hive-ai_${VERSION}_amd64.deb"

# Create checksum
cd "$BUILD_DIR"
sha256sum "hive-ai_${VERSION}_amd64.deb" > "hive-ai_${VERSION}_amd64.deb.sha256"

# Verify package
print_status "Verifying package..."
dpkg-deb --info "hive-ai_${VERSION}_amd64.deb"
dpkg-deb --contents "hive-ai_${VERSION}_amd64.deb"

# Lintian check (if available)
if command -v lintian >/dev/null 2>&1; then
    print_status "Running lintian checks..."
    lintian "hive-ai_${VERSION}_amd64.deb" || true
fi

print_success "Package verification complete"

# Installation instructions
echo
echo "📦 Installation Instructions:"
echo "  sudo dpkg -i hive-ai_${VERSION}_amd64.deb"
echo "  sudo apt-get install -f  # Fix any dependency issues"
echo
echo "🗑️  Removal Instructions:"
echo "  sudo apt-get remove hive-ai          # Remove package"
echo "  sudo apt-get purge hive-ai           # Remove package and config"
echo
echo "📋 Package Information:"
echo "  dpkg -l | grep hive-ai               # Check if installed"
echo "  dpkg -L hive-ai                      # List installed files"
echo "  dpkg -s hive-ai                      # Show package status"