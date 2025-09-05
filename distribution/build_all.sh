#!/usr/bin/env bash
#
# HiveTechs Consensus Distribution Build Orchestrator
# Builds all platform binaries and installers
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION="${HIVE_VERSION:-$(cd "$PROJECT_ROOT" && cargo pkgid | sed 's/.*#//')}"
BUILD_DATE=$(date -u '+%Y-%m-%d %H:%M:%S UTC')
COMMIT_HASH="${GITHUB_SHA:-$(cd "$PROJECT_ROOT" && git rev-parse --short HEAD 2>/dev/null || echo 'unknown')}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Build configuration
PARALLEL_BUILDS=${PARALLEL_BUILDS:-true}
SKIP_TESTS=${SKIP_TESTS:-false}
SKIP_INSTALLERS=${SKIP_INSTALLERS:-false}
SKIP_CLEANUP=${SKIP_CLEANUP:-false}
BUILD_TARGETS=${BUILD_TARGETS:-"macos linux windows"}

print_banner() {
    echo
    echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
    echo -e "${BLUE}│${NC}  🐝 ${BOLD}HiveTechs Consensus Distribution${NC}   ${BLUE}│${NC}"
    echo -e "${BLUE}│${NC}     ${BOLD}Multi-Platform Build System${NC}        ${BLUE}│${NC}"
    echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
    echo
    echo -e "${BOLD}Version:${NC} $VERSION"
    echo -e "${BOLD}Commit:${NC} $COMMIT_HASH"
    echo -e "${BOLD}Build Date:${NC} $BUILD_DATE"
    echo -e "${BOLD}Targets:${NC} $BUILD_TARGETS"
    echo
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo
    echo -e "${BOLD}${BLUE}=== $1 ===${NC}"
}

check_dependencies() {
    log_step "Checking Dependencies"
    
    local missing_deps=()
    
    # Core dependencies
    for cmd in cargo git; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    # Platform-specific dependencies
    if [[ "$BUILD_TARGETS" == *"macos"* ]] && [[ "$OSTYPE" == "darwin"* ]]; then
        for cmd in lipo pkgbuild; do
            if ! command -v "$cmd" &> /dev/null; then
                log_warn "macOS tool '$cmd' not found - some features may not work"
            fi
        done
    fi
    
    if [[ "$BUILD_TARGETS" == *"linux"* ]]; then
        for cmd in fakeroot dpkg-deb; do
            if ! command -v "$cmd" &> /dev/null; then
                log_warn "Linux packaging tool '$cmd' not found - installers may not be created"
            fi
        done
    fi
    
    if [[ "$BUILD_TARGETS" == *"windows"* ]]; then
        # Check for cross-compilation support
        if ! rustup target list --installed | grep -q "x86_64-pc-windows"; then
            log_warn "Windows cross-compilation target not installed"
        fi
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    log_success "All dependencies satisfied"
}

setup_environment() {
    log_step "Setting Up Environment"
    
    cd "$PROJECT_ROOT"
    
    # Set version in environment
    export HIVE_VERSION="$VERSION"
    export HIVE_BUILD_DATE="$BUILD_DATE"
    export HIVE_COMMIT_HASH="$COMMIT_HASH"
    
    # Create distribution directory
    mkdir -p target/distribution
    
    # Clean previous builds if requested
    if [ "$SKIP_CLEANUP" = "false" ]; then
        log_info "Cleaning previous builds..."
        rm -rf target/distribution/*
        cargo clean --release
    fi
    
    log_success "Environment prepared"
}

run_tests() {
    if [ "$SKIP_TESTS" = "true" ]; then
        log_warn "Skipping tests (SKIP_TESTS=true)"
        return 0
    fi
    
    log_step "Running Tests"
    
    cd "$PROJECT_ROOT"
    
    # Run unit tests
    log_info "Running unit tests..."
    cargo test --release --all-features
    
    # Run integration tests
    log_info "Running integration tests..."
    cargo test --release --test '*'
    
    # Run benchmarks to ensure they compile
    log_info "Verifying benchmarks..."
    cargo bench --no-run
    
    log_success "All tests passed"
}

build_binary() {
    local platform=$1
    local script_name="$platform.sh"
    
    if [[ "$platform" == "windows" ]]; then
        script_name="windows.ps1"
    fi
    
    local build_script="$SCRIPT_DIR/build/$script_name"
    
    if [ ! -f "$build_script" ]; then
        log_error "Build script not found: $build_script"
        return 1
    fi
    
    log_info "Building for $platform..."
    
    # Make script executable
    chmod +x "$build_script" 2>/dev/null || true
    
    # Execute build script
    if [[ "$platform" == "windows" ]]; then
        if command -v pwsh &> /dev/null; then
            pwsh -ExecutionPolicy Bypass -File "$build_script"
        elif command -v powershell &> /dev/null; then
            powershell -ExecutionPolicy Bypass -File "$build_script"
        else
            log_warn "PowerShell not found - skipping Windows build"
            return 0
        fi
    else
        bash "$build_script"
    fi
    
    log_success "$platform build completed"
}

build_installers() {
    if [ "$SKIP_INSTALLERS" = "true" ]; then
        log_warn "Skipping installers (SKIP_INSTALLERS=true)"
        return 0
    fi
    
    log_step "Building Platform Installers"
    
    # macOS installer
    if [[ "$BUILD_TARGETS" == *"macos"* ]] && [[ "$OSTYPE" == "darwin"* ]]; then
        if [ -f "$SCRIPT_DIR/installers/macos/create_pkg.sh" ]; then
            log_info "Creating macOS installer..."
            bash "$SCRIPT_DIR/installers/macos/create_pkg.sh"
        fi
    fi
    
    # Linux installer
    if [[ "$BUILD_TARGETS" == *"linux"* ]]; then
        if [ -f "$SCRIPT_DIR/installers/linux/create_deb.sh" ]; then
            log_info "Creating Linux DEB package..."
            bash "$SCRIPT_DIR/installers/linux/create_deb.sh"
        fi
    fi
    
    # Windows installer
    if [[ "$BUILD_TARGETS" == *"windows"* ]]; then
        if [ -f "$SCRIPT_DIR/installers/windows/hive.nsi" ]; then
            log_info "Windows installer configuration ready"
            log_info "Run makensis on Windows to create installer"
        fi
    fi
    
    log_success "Installer builds completed"
}

build_all_platforms() {
    log_step "Building Platform Binaries"
    
    local build_pids=()
    
    for platform in $BUILD_TARGETS; do
        if [ "$PARALLEL_BUILDS" = "true" ]; then
            # Build in parallel
            build_binary "$platform" &
            build_pids+=($!)
        else
            # Build sequentially
            build_binary "$platform"
        fi
    done
    
    # Wait for parallel builds to complete
    if [ "$PARALLEL_BUILDS" = "true" ]; then
        log_info "Waiting for parallel builds to complete..."
        for pid in "${build_pids[@]}"; do
            if ! wait "$pid"; then
                log_error "Build process $pid failed"
                exit 1
            fi
        done
    fi
    
    log_success "All platform builds completed"
}

verify_builds() {
    log_step "Verifying Build Artifacts"
    
    local dist_dir="$PROJECT_ROOT/target/distribution"
    local issues=0
    
    # Check for platform-specific artifacts
    for platform in $BUILD_TARGETS; do
        local platform_dir="$dist_dir/$platform"
        
        if [ ! -d "$platform_dir" ]; then
            log_error "Missing distribution directory for $platform"
            ((issues++))
            continue
        fi
        
        # Check for binary
        local binary_found=false
        for ext in "" ".exe"; do
            if [ -f "$platform_dir/hive$ext" ]; then
                binary_found=true
                
                # Check binary size (should be reasonable)
                local size=$(stat -c%s "$platform_dir/hive$ext" 2>/dev/null || stat -f%z "$platform_dir/hive$ext" 2>/dev/null || echo 0)
                if [ "$size" -lt 1000000 ]; then  # Less than 1MB is suspicious
                    log_warn "$platform binary seems too small: ${size} bytes"
                fi
                if [ "$size" -gt 100000000 ]; then  # More than 100MB is suspicious
                    log_warn "$platform binary seems too large: ${size} bytes"
                fi
                
                log_info "$platform binary: $(du -h "$platform_dir/hive$ext" | cut -f1)"
                break
            fi
        done
        
        if [ "$binary_found" = "false" ]; then
            log_error "No binary found for $platform"
            ((issues++))
        fi
        
        # Check for checksums
        if ! find "$platform_dir" -name "*.sha256" | head -1 | grep -q .; then
            log_warn "No checksums found for $platform"
        fi
    done
    
    if [ "$issues" -gt 0 ]; then
        log_error "Found $issues verification issues"
        return 1
    fi
    
    log_success "All build artifacts verified"
}

generate_release_notes() {
    log_step "Generating Release Documentation"
    
    local release_dir="$PROJECT_ROOT/target/distribution/release"
    mkdir -p "$release_dir"
    
    # Generate comprehensive release notes
    cat > "$release_dir/RELEASE_NOTES.md" << EOF
# HiveTechs Consensus v$VERSION

**Release Date:** $BUILD_DATE  
**Commit:** $COMMIT_HASH

## 🚀 What's New

This release includes performance improvements, bug fixes, and new features for the AI-powered codebase intelligence platform.

## 📦 Installation

### Quick Install (Recommended)
\`\`\`bash
curl -fsSL https://hive.ai/install | sh
\`\`\`

### Platform-Specific Downloads

#### macOS
- **Universal Binary (Intel + Apple Silicon)**
  - Download: [hive-$VERSION-macos-universal.tar.gz](hive-$VERSION-macos-universal.tar.gz)
  - Installer: [HiveTechs-Consensus-$VERSION.pkg](HiveTechs-Consensus-$VERSION.pkg)

#### Linux
- **x86_64**
  - Download: [hive-$VERSION-linux-x86_64.tar.gz](hive-$VERSION-linux-x86_64.tar.gz)
  - DEB Package: [hive-ai_${VERSION}_amd64.deb](hive-ai_${VERSION}_amd64.deb)
- **ARM64**
  - Download: [hive-$VERSION-linux-aarch64.tar.gz](hive-$VERSION-linux-aarch64.tar.gz)

#### Windows
- **x64**
  - Download: [hive-$VERSION-windows-x64.zip](hive-$VERSION-windows-x64.zip)
  - Installer: [HiveTechs-Consensus-Setup.exe](HiveTechs-Consensus-Setup.exe)

## 🔍 Verification

All release artifacts include SHA256 checksums for verification:

\`\`\`bash
# Verify download integrity
sha256sum -c <artifact>.sha256
\`\`\`

## 📊 Build Information

- **Rust Version:** $(rustc --version)
- **Build Profile:** Production (optimized)
- **Features:** Full feature set enabled
- **Static Linking:** Yes (Linux)
- **Code Signing:** Platform dependent

## 🆕 Features

- Multi-model AI consensus engine
- Repository semantic understanding
- Terminal User Interface (TUI)
- Real-time code analysis
- Intelligent transformations
- Memory and context management
- IDE integrations

## 🔧 System Requirements

- **macOS:** 10.15+ (Intel or Apple Silicon)
- **Linux:** Any modern distribution (glibc 2.17+)
- **Windows:** Windows 10+ (64-bit)

## 📚 Documentation

- [User Guide](https://github.com/hivetechs/hive/blob/main/README.md)
- [API Documentation](https://github.com/hivetechs/hive/tree/main/docs)
- [Configuration Guide](https://github.com/hivetechs/hive/blob/main/docs/configuration.md)

## 🐛 Bug Reports

Report issues at: [github.com/hivetechs/hive/issues](https://github.com/hivetechs/hive/issues)

## 📞 Support

- GitHub Discussions: [github.com/hivetechs/hive/discussions](https://github.com/hivetechs/hive/discussions)
- Email: team@hivetechs.com

---

**Full Changelog:** [View on GitHub](https://github.com/hivetechs/hive/compare/v$(echo $VERSION | sed 's/\.[0-9]*$//')...v$VERSION)
EOF

    # Generate build manifest
    cat > "$release_dir/BUILD_MANIFEST.json" << EOF
{
  "version": "$VERSION",
  "build_date": "$BUILD_DATE",
  "commit_hash": "$COMMIT_HASH",
  "rust_version": "$(rustc --version)",
  "targets": [$(echo "$BUILD_TARGETS" | sed 's/ /", "/g' | sed 's/^/"/' | sed 's/$/"/')],
  "features": ["production", "static-linking", "optimized-consensus"],
  "artifacts": [
EOF

    # List all artifacts
    local first=true
    find "$PROJECT_ROOT/target/distribution" -name "*.tar.gz" -o -name "*.zip" -o -name "*.pkg" -o -name "*.deb" -o -name "*.exe" | while read -r file; do
        if [ "$first" = "true" ]; then
            first=false
        else
            echo ","
        fi
        local filename=$(basename "$file")
        local size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo 0)
        echo -n "    {\"name\": \"$filename\", \"size\": $size, \"path\": \"$(realpath --relative-to="$PROJECT_ROOT" "$file")\"}"
    done >> "$release_dir/BUILD_MANIFEST.json"
    
    echo >> "$release_dir/BUILD_MANIFEST.json"
    echo "  ]" >> "$release_dir/BUILD_MANIFEST.json"
    echo "}" >> "$release_dir/BUILD_MANIFEST.json"
    
    log_success "Release documentation generated"
}

create_universal_checksum() {
    log_step "Creating Universal Checksum File"
    
    local checksum_file="$PROJECT_ROOT/target/distribution/checksums.sha256"
    
    # Create master checksum file
    find "$PROJECT_ROOT/target/distribution" -name "*.tar.gz" -o -name "*.zip" -o -name "*.pkg" -o -name "*.deb" -o -name "*.exe" | while read -r file; do
        # Calculate checksum relative to distribution directory
        local rel_path=$(realpath --relative-to="$PROJECT_ROOT/target/distribution" "$file")
        (cd "$PROJECT_ROOT/target/distribution" && sha256sum "$rel_path")
    done > "$checksum_file"
    
    log_success "Universal checksum file created: $checksum_file"
}

show_build_summary() {
    log_step "Build Summary"
    
    local dist_dir="$PROJECT_ROOT/target/distribution"
    
    echo -e "${BOLD}Build completed successfully!${NC}"
    echo
    echo -e "${BOLD}Version:${NC} $VERSION"
    echo -e "${BOLD}Targets:${NC} $BUILD_TARGETS"
    echo -e "${BOLD}Build Time:${NC} $(date)"
    echo
    echo -e "${BOLD}Artifacts:${NC}"
    
    find "$dist_dir" -name "*.tar.gz" -o -name "*.zip" -o -name "*.pkg" -o -name "*.deb" -o -name "*.exe" | sort | while read -r file; do
        local size=$(du -h "$file" | cut -f1)
        local name=$(basename "$file")
        echo "  📦 $name ($size)"
    done
    
    echo
    echo -e "${BOLD}Distribution Directory:${NC} $dist_dir"
    echo -e "${BOLD}Total Size:${NC} $(du -sh "$dist_dir" | cut -f1)"
    echo
    echo -e "${GREEN}✅ Ready for distribution!${NC}"
    echo
    echo -e "${BOLD}Next Steps:${NC}"
    echo "1. Test installations on target platforms"
    echo "2. Upload artifacts to release server"
    echo "3. Update download links and documentation"
    echo "4. Announce the release"
}

main() {
    print_banner
    check_dependencies
    setup_environment
    run_tests
    build_all_platforms
    build_installers
    verify_builds
    generate_release_notes
    create_universal_checksum
    show_build_summary
}

# Handle command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-installers)
            SKIP_INSTALLERS=true
            shift
            ;;
        --skip-cleanup)
            SKIP_CLEANUP=true
            shift
            ;;
        --sequential)
            PARALLEL_BUILDS=false
            shift
            ;;
        --targets)
            BUILD_TARGETS="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --help)
            echo "HiveTechs Consensus Distribution Builder"
            echo
            echo "Usage: $0 [options]"
            echo
            echo "Options:"
            echo "  --skip-tests         Skip running tests"
            echo "  --skip-installers    Skip building installers"
            echo "  --skip-cleanup       Skip cleaning previous builds"
            echo "  --sequential         Build platforms sequentially"
            echo "  --targets TARGETS    Space-separated list of targets (macos linux windows)"
            echo "  --version VERSION    Override version number"
            echo "  --help              Show this help"
            echo
            echo "Environment Variables:"
            echo "  HIVE_VERSION        Override version"
            echo "  PARALLEL_BUILDS     Enable/disable parallel builds (true/false)"
            echo "  SKIP_TESTS          Skip tests (true/false)"
            echo "  SKIP_INSTALLERS     Skip installers (true/false)"
            echo "  BUILD_TARGETS       Target platforms"
            echo
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run main function
main