#!/bin/bash
# Build optimized release binaries for all platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERSION=${1:-"2.0.0"}
BUILD_TYPE=${2:-"production"}

echo -e "${BLUE}🔧 Building Hive AI v${VERSION} (${BUILD_TYPE})${NC}"

# Ensure we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Must run from project root${NC}"
    exit 1
fi

# Create build output directory
mkdir -p "build/output"
rm -rf build/output/*

# Build function for each target
build_target() {
    local target=$1
    local output_name=$2
    local platform_name=$3
    
    echo -e "${YELLOW}📦 Building ${platform_name} (${target})...${NC}"
    
    # Set target-specific flags
    export RUSTFLAGS=""
    case "$target" in
        *linux*)
            export RUSTFLAGS="-C target-feature=+crt-static"
            ;;
        *darwin*)
            export MACOSX_DEPLOYMENT_TARGET="10.15"
            ;;
        *windows*)
            export RUSTFLAGS="-C target-feature=+crt-static"
            ;;
    esac
    
    # Build with production profile
    cargo build \
        --release \
        --target "$target" \
        --features "$BUILD_TYPE" \
        --profile "production" \
        --bin hive
    
    # Copy and rename binary
    local source_path="target/${target}/production"
    local binary_name="hive"
    
    if [[ "$target" == *"windows"* ]]; then
        binary_name="hive.exe"
    fi
    
    cp "${source_path}/${binary_name}" "build/output/${output_name}"
    
    # Get binary size
    local size=$(du -h "build/output/${output_name}" | cut -f1)
    echo -e "${GREEN}✅ ${platform_name}: ${size}${NC}"
}

# Install required targets
echo -e "${YELLOW}🔧 Installing Rust targets...${NC}"
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc

# Build for all platforms
echo -e "${BLUE}🚀 Starting cross-platform builds...${NC}"

# macOS Intel
build_target "x86_64-apple-darwin" "hive-macos-x64" "macOS Intel"

# macOS Apple Silicon
build_target "aarch64-apple-darwin" "hive-macos-arm64" "macOS Apple Silicon"

# Linux x64
build_target "x86_64-unknown-linux-gnu" "hive-linux-x64" "Linux x64"

# Linux ARM64
build_target "aarch64-unknown-linux-gnu" "hive-linux-arm64" "Linux ARM64"

# Windows x64
build_target "x86_64-pc-windows-msvc" "hive-windows-x64.exe" "Windows x64"

echo -e "${BLUE}📊 Build Summary:${NC}"
echo -e "${BLUE}=================${NC}"
ls -lh build/output/

# Create checksums
echo -e "${YELLOW}🔐 Generating checksums...${NC}"
cd build/output
for file in *; do
    if [ -f "$file" ]; then
        sha256sum "$file" > "${file}.sha256"
        echo -e "${GREEN}✅ ${file}: $(cat ${file}.sha256 | cut -d' ' -f1)${NC}"
    fi
done
cd ../..

# Test each binary
echo -e "${BLUE}🧪 Testing binaries...${NC}"
for binary in build/output/hive-*; do
    if [[ "$binary" == *.exe ]]; then
        # Skip Windows binaries on non-Windows systems
        echo -e "${YELLOW}⏭️  Skipping Windows binary test on $(uname)${NC}"
        continue
    elif [[ "$binary" == *.sha256 ]]; then
        continue
    fi
    
    # Test if binary can show version
    if timeout 10s "$binary" --version &>/dev/null; then
        echo -e "${GREEN}✅ ${binary}: Works${NC}"
    else
        echo -e "${RED}❌ ${binary}: Failed${NC}"
    fi
done

echo -e "${GREEN}🎉 Build complete! Binaries available in build/output/${NC}"

# Generate build metadata
cat > build/output/build-metadata.json << EOF
{
    "version": "${VERSION}",
    "build_type": "${BUILD_TYPE}",
    "build_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
    "git_branch": "$(git branch --show-current 2>/dev/null || echo 'unknown')",
    "rust_version": "$(rustc --version)",
    "targets": [
        "x86_64-apple-darwin",
        "aarch64-apple-darwin", 
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc"
    ]
}
EOF

echo -e "${BLUE}📄 Build metadata saved to build/output/build-metadata.json${NC}"