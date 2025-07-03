#!/usr/bin/env bash
#
# Linux Build Script for HiveTechs Consensus
# Builds static binaries for maximum compatibility
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/distribution/linux"
VERSION="${HIVE_VERSION:-$(cargo pkgid | sed 's/.*#//')}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[BUILD]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Create build directory
mkdir -p "$BUILD_DIR"

cd "$PROJECT_ROOT"

print_status "Building HiveTechs Consensus v$VERSION for Linux"

# Check for required tools
if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust."
    exit 1
fi

# Add musl targets for static linking
print_status "Adding Rust targets..."
rustup target add x86_64-unknown-linux-musl || true
rustup target add aarch64-unknown-linux-musl || true

# Install musl tools if needed
install_musl_tools() {
    if command -v apt-get &> /dev/null; then
        print_status "Installing musl tools via apt..."
        sudo apt-get update && sudo apt-get install -y musl-tools musl-dev
    elif command -v yum &> /dev/null; then
        print_status "Installing musl tools via yum..."
        sudo yum install -y musl-gcc musl-libc-static
    elif command -v pacman &> /dev/null; then
        print_status "Installing musl tools via pacman..."
        sudo pacman -S --noconfirm musl
    else
        print_warning "Package manager not detected. Please install musl tools manually."
    fi
}

# Check if musl-gcc is available
if ! command -v musl-gcc &> /dev/null; then
    print_warning "musl-gcc not found. Installing musl tools..."
    install_musl_tools
fi

# Build for x86_64 Linux (static)
print_status "Building for x86_64 Linux (static)..."
RUSTFLAGS='-C link-arg=-s' cargo build \
    --profile production \
    --target x86_64-unknown-linux-musl \
    --features production \
    --bin hive

# Build for ARM64 Linux (static)
print_status "Building for ARM64 Linux (static)..."
RUSTFLAGS='-C link-arg=-s' cargo build \
    --profile production \
    --target aarch64-unknown-linux-musl \
    --features production \
    --bin hive || print_warning "ARM64 build may fail on some systems"

# Copy binaries to distribution directory
cp "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/production/hive" "$BUILD_DIR/hive-x86_64"
if [ -f "$PROJECT_ROOT/target/aarch64-unknown-linux-musl/production/hive" ]; then
    cp "$PROJECT_ROOT/target/aarch64-unknown-linux-musl/production/hive" "$BUILD_DIR/hive-aarch64"
fi

# Create architecture-specific symlink for main binary
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ln -sf hive-x86_64 "$BUILD_DIR/hive"
        ;;
    aarch64|arm64)
        if [ -f "$BUILD_DIR/hive-aarch64" ]; then
            ln -sf hive-aarch64 "$BUILD_DIR/hive"
        else
            ln -sf hive-x86_64 "$BUILD_DIR/hive"
            print_warning "Using x86_64 binary for ARM64 system"
        fi
        ;;
    *)
        ln -sf hive-x86_64 "$BUILD_DIR/hive"
        print_warning "Unknown architecture $ARCH, using x86_64 binary"
        ;;
esac

# Verify static linking
print_status "Verifying static linking..."
if ldd "$BUILD_DIR/hive" 2>&1 | grep -q "not a dynamic executable"; then
    print_success "Static binary created successfully"
else
    print_warning "Binary may have dynamic dependencies:"
    ldd "$BUILD_DIR/hive" || true
fi

# Check binary size
BINARY_SIZE=$(du -h "$BUILD_DIR/hive" | cut -f1)
print_status "Binary size: $BINARY_SIZE"

# Strip binary
print_status "Stripping binary..."
strip "$BUILD_DIR/hive" || true

# Create shell completions
print_status "Generating shell completions..."
mkdir -p "$BUILD_DIR/completions"
"$BUILD_DIR/hive" completion bash > "$BUILD_DIR/completions/hive.bash" || true
"$BUILD_DIR/hive" completion zsh > "$BUILD_DIR/completions/_hive" || true
"$BUILD_DIR/hive" completion fish > "$BUILD_DIR/completions/hive.fish" || true

# Create man page (if help2man is available)
if command -v help2man &> /dev/null; then
    print_status "Generating man page..."
    mkdir -p "$BUILD_DIR/man"
    help2man --no-info --name="HiveTechs Consensus AI" "$BUILD_DIR/hive" > "$BUILD_DIR/man/hive.1" || true
fi

# Create desktop entry
print_status "Creating desktop entry..."
mkdir -p "$BUILD_DIR/desktop"
cat > "$BUILD_DIR/desktop/hive.desktop" << EOF
[Desktop Entry]
Name=HiveTechs Consensus
Comment=AI-powered codebase intelligence platform
Exec=hive
Icon=hive
Terminal=true
Type=Application
Categories=Development;
Keywords=AI;Development;Code;Intelligence;
EOF

# Create systemd service template
print_status "Creating systemd service template..."
mkdir -p "$BUILD_DIR/systemd"
cat > "$BUILD_DIR/systemd/hive-daemon.service" << EOF
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

[Install]
WantedBy=multi-user.target
EOF

# Create DEB package structure
print_status "Preparing DEB package structure..."
DEB_DIR="$BUILD_DIR/deb"
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/local/bin"
mkdir -p "$DEB_DIR/usr/share/bash-completion/completions"
mkdir -p "$DEB_DIR/usr/share/zsh/site-functions"
mkdir -p "$DEB_DIR/usr/share/fish/completions"
mkdir -p "$DEB_DIR/usr/share/man/man1"
mkdir -p "$DEB_DIR/usr/share/applications"

# Copy files for DEB package
cp "$BUILD_DIR/hive" "$DEB_DIR/usr/local/bin/"
cp "$BUILD_DIR/completions/hive.bash" "$DEB_DIR/usr/share/bash-completion/completions/hive" || true
cp "$BUILD_DIR/completions/_hive" "$DEB_DIR/usr/share/zsh/site-functions/" || true
cp "$BUILD_DIR/completions/hive.fish" "$DEB_DIR/usr/share/fish/completions/" || true
cp "$BUILD_DIR/man/hive.1" "$DEB_DIR/usr/share/man/man1/" || true
cp "$BUILD_DIR/desktop/hive.desktop" "$DEB_DIR/usr/share/applications/" || true

# Create DEB control file
cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: hive-ai
Version: $VERSION
Section: devel
Priority: optional
Architecture: amd64
Depends: 
Maintainer: HiveTechs Collective <team@hivetechs.com>
Description: AI-powered codebase intelligence platform
 HiveTechs Consensus provides AI-powered code analysis,
 transformation, and intelligence capabilities with
 multi-model consensus for maximum accuracy.
EOF

# Create RPM spec structure
print_status "Preparing RPM spec structure..."
RPM_DIR="$BUILD_DIR/rpm"
mkdir -p "$RPM_DIR/SPECS"
mkdir -p "$RPM_DIR/SOURCES"
mkdir -p "$RPM_DIR/BUILD"
mkdir -p "$RPM_DIR/RPMS"
mkdir -p "$RPM_DIR/SRPMS"

# Create tarball for RPM
tar -czf "$RPM_DIR/SOURCES/hive-$VERSION.tar.gz" -C "$BUILD_DIR" \
    hive completions/ man/ desktop/ systemd/

# Create RPM spec file
cat > "$RPM_DIR/SPECS/hive.spec" << EOF
Name:           hive-ai
Version:        $VERSION
Release:        1%{?dist}
Summary:        AI-powered codebase intelligence platform

License:        Proprietary
URL:            https://github.com/hivetechs/hive
Source0:        hive-%{version}.tar.gz

BuildRequires:  
Requires:       

%description
HiveTechs Consensus provides AI-powered code analysis,
transformation, and intelligence capabilities with
multi-model consensus for maximum accuracy.

%prep
%setup -q

%build
# Binary is pre-built

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/local/bin
mkdir -p %{buildroot}/usr/share/bash-completion/completions
mkdir -p %{buildroot}/usr/share/zsh/site-functions
mkdir -p %{buildroot}/usr/share/fish/completions
mkdir -p %{buildroot}/usr/share/man/man1
mkdir -p %{buildroot}/usr/share/applications

install -m 755 hive %{buildroot}/usr/local/bin/
install -m 644 completions/hive.bash %{buildroot}/usr/share/bash-completion/completions/hive
install -m 644 completions/_hive %{buildroot}/usr/share/zsh/site-functions/
install -m 644 completions/hive.fish %{buildroot}/usr/share/fish/completions/
install -m 644 man/hive.1 %{buildroot}/usr/share/man/man1/
install -m 644 desktop/hive.desktop %{buildroot}/usr/share/applications/

%files
/usr/local/bin/hive
/usr/share/bash-completion/completions/hive
/usr/share/zsh/site-functions/_hive
/usr/share/fish/completions/hive.fish
/usr/share/man/man1/hive.1
/usr/share/applications/hive.desktop

%changelog
* $(date '+%a %b %d %Y') HiveTechs Collective <team@hivetechs.com> - $VERSION-1
- Initial release
EOF

# Create archives
print_status "Creating distribution archives..."
cd "$BUILD_DIR"

# Generic Linux archive
tar -czf "hive-$VERSION-linux-x86_64.tar.gz" \
    hive-x86_64 completions/ man/ desktop/ systemd/

if [ -f "hive-aarch64" ]; then
    tar -czf "hive-$VERSION-linux-aarch64.tar.gz" \
        hive-aarch64 completions/ man/ desktop/ systemd/
fi

# Calculate checksums
print_status "Calculating checksums..."
find . -name "*.tar.gz" -exec shasum -a 256 {} \; > checksums.sha256

print_success "Linux build complete!"
print_status "Distribution files:"
ls -la "$BUILD_DIR"

# Performance test
print_status "Running performance test..."
time "$BUILD_DIR/hive" --version > /dev/null 2>&1 || print_warning "Performance test failed"

print_success "Build artifacts ready at: $BUILD_DIR"