#!/usr/bin/env bash
#
# macOS Build Script for HiveTechs Consensus
# Builds universal binaries for Intel and Apple Silicon
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/distribution/macos"
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

print_status "Building HiveTechs Consensus v$VERSION for macOS"

# Check for required tools
if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust."
    exit 1
fi

# Add targets if not already added
print_status "Adding Rust targets..."
rustup target add x86_64-apple-darwin || true
rustup target add aarch64-apple-darwin || true

# Build for Intel Macs
print_status "Building for Intel Macs (x86_64)..."
cargo build \
    --profile production \
    --target x86_64-apple-darwin \
    --features production \
    --bin hive

# Build for Apple Silicon
print_status "Building for Apple Silicon (ARM64)..."
cargo build \
    --profile production \
    --target aarch64-apple-darwin \
    --features production \
    --bin hive

# Create universal binary
print_status "Creating universal binary..."
lipo -create \
    "$PROJECT_ROOT/target/x86_64-apple-darwin/production/hive" \
    "$PROJECT_ROOT/target/aarch64-apple-darwin/production/hive" \
    -output "$BUILD_DIR/hive"

# Verify universal binary
print_status "Verifying universal binary..."
if file "$BUILD_DIR/hive" | grep -q "Universal binary"; then
    print_success "Universal binary created successfully"
else
    print_error "Failed to create universal binary"
    exit 1
fi

# Strip and optimize
print_status "Stripping binary..."
strip "$BUILD_DIR/hive"

# Check binary size
BINARY_SIZE=$(du -h "$BUILD_DIR/hive" | cut -f1)
print_status "Binary size: $BINARY_SIZE"

# Prepare for code signing (if certificates available)
if security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
    print_status "Code signing certificate found - preparing for signing..."
    
    # Create entitlements file
    cat > "$BUILD_DIR/entitlements.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>com.apple.security.files.downloads.read-write</key>
    <true/>
</dict>
</plist>
EOF

    # Sign the binary (uncomment when ready)
    # codesign --force --options runtime --entitlements "$BUILD_DIR/entitlements.plist" --sign "Developer ID Application" "$BUILD_DIR/hive"
    print_warning "Code signing prepared but not executed. Uncomment signing command when certificates are ready."
else
    print_warning "No code signing certificate found. Binary will not be signed."
fi

# Create shell completions
print_status "Generating shell completions..."
mkdir -p "$BUILD_DIR/completions"
"$BUILD_DIR/hive" completion bash > "$BUILD_DIR/completions/hive.bash" || true
"$BUILD_DIR/hive" completion zsh > "$BUILD_DIR/completions/_hive" || true
"$BUILD_DIR/hive" completion fish > "$BUILD_DIR/completions/hive.fish" || true

# Create archive
print_status "Creating distribution archive..."
cd "$BUILD_DIR"
tar -czf "hive-$VERSION-macos-universal.tar.gz" \
    hive \
    completions/ \
    entitlements.plist

# Calculate checksums
print_status "Calculating checksums..."
shasum -a 256 "hive-$VERSION-macos-universal.tar.gz" > "hive-$VERSION-macos-universal.tar.gz.sha256"

print_success "macOS build complete!"
print_status "Distribution files:"
ls -la "$BUILD_DIR"

# Performance test
print_status "Running performance test..."
time "$BUILD_DIR/hive" --version > /dev/null 2>&1 || print_warning "Performance test failed"

print_success "Build artifacts ready at: $BUILD_DIR"