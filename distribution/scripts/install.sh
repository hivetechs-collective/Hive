#!/usr/bin/env bash
#
# HiveTechs Consensus Universal Installer
# Like Claude Code: curl -fsSL https://hive.ai/install | sh
#

set -euo pipefail

# Configuration
GITHUB_REPO="hivetechs/hive"
INSTALL_DIR="${HIVE_INSTALL_DIR:-/usr/local/bin}"
CONFIG_DIR="${HOME}/.hive"
DOWNLOAD_BASE="https://github.com/${GITHUB_REPO}/releases/download"
TEMP_DIR=$(mktemp -d)
VERSION="${HIVE_VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Cleanup on exit
trap 'rm -rf "$TEMP_DIR"' EXIT

print_banner() {
    echo
    echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
    echo -e "${BLUE}│${NC}  🐝 ${BOLD}HiveTechs Consensus Installer${NC}     ${BLUE}│${NC}"
    echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
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

check_dependencies() {
    local missing_deps=()
    
    for cmd in curl tar; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Please install the missing dependencies and try again."
        exit 1
    fi
}

detect_platform() {
    local os arch
    
    # Detect OS
    case "$(uname -s)" in
        Darwin)
            os="macos"
            ;;
        Linux)
            os="linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="windows"
            ;;
        *)
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        armv7l)
            arch="armv7"
            ;;
        i386|i686)
            arch="i386"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    # Construct platform string
    if [ "$os" = "macos" ]; then
        PLATFORM="macos-universal"
    elif [ "$os" = "windows" ]; then
        PLATFORM="windows-${arch}"
    else
        PLATFORM="linux-${arch}"
    fi
    
    log_info "Detected platform: $PLATFORM"
}

get_latest_version() {
    if [ "$VERSION" = "latest" ]; then
        log_info "Fetching latest version..."
        VERSION=$(curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | 
                  grep '"tag_name":' | 
                  sed -E 's/.*"tag_name": "v?([^"]+)".*/\1/')
        
        if [ -z "$VERSION" ]; then
            log_error "Failed to fetch latest version"
            exit 1
        fi
    fi
    
    log_info "Installing version: $VERSION"
}

download_and_extract() {
    local download_url archive_name
    
    # Construct download URL
    case "$PLATFORM" in
        macos-universal)
            archive_name="hive-${VERSION}-macos-universal.tar.gz"
            ;;
        linux-*)
            archive_name="hive-${VERSION}-${PLATFORM}.tar.gz"
            ;;
        windows-*)
            archive_name="hive-${VERSION}-${PLATFORM}.zip"
            ;;
    esac
    
    download_url="${DOWNLOAD_BASE}/v${VERSION}/${archive_name}"
    
    log_info "Downloading from: $download_url"
    
    cd "$TEMP_DIR"
    
    # Download with progress bar
    if ! curl -fsSL --progress-bar "$download_url" -o "$archive_name"; then
        log_error "Failed to download $archive_name"
        log_info "Please check if version $VERSION exists and supports platform $PLATFORM"
        exit 1
    fi
    
    # Verify checksum if available
    local checksum_url="${DOWNLOAD_BASE}/v${VERSION}/${archive_name}.sha256"
    if curl -fsSL "$checksum_url" -o "${archive_name}.sha256" 2>/dev/null; then
        log_info "Verifying checksum..."
        if command -v sha256sum &> /dev/null; then
            sha256sum -c "${archive_name}.sha256" || {
                log_error "Checksum verification failed"
                exit 1
            }
        elif command -v shasum &> /dev/null; then
            shasum -a 256 -c "${archive_name}.sha256" || {
                log_error "Checksum verification failed"
                exit 1
            }
        else
            log_warn "No checksum utility found, skipping verification"
        fi
    else
        log_warn "Checksum not available, skipping verification"
    fi
    
    # Extract archive
    log_info "Extracting archive..."
    case "$archive_name" in
        *.tar.gz)
            tar -xzf "$archive_name"
            ;;
        *.zip)
            if command -v unzip &> /dev/null; then
                unzip -q "$archive_name"
            else
                log_error "unzip command required for Windows archives"
                exit 1
            fi
            ;;
    esac
}

install_binary() {
    local binary_name="hive"
    
    # Find the binary
    local binary_path
    if [ -f "$binary_name" ]; then
        binary_path="./$binary_name"
    elif [ -f "hive.exe" ]; then
        binary_path="./hive.exe"
        binary_name="hive.exe"
    else
        # Look in subdirectories
        binary_path=$(find . -name "hive" -o -name "hive.exe" | head -1)
        if [ -z "$binary_path" ]; then
            log_error "Binary not found in archive"
            exit 1
        fi
    fi
    
    # Make sure install directory exists
    if [ "$INSTALL_DIR" != "/usr/local/bin" ] || [ ! -w "/usr/local/bin" ]; then
        # Try to create user-local bin directory
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
        log_info "Installing to user directory: $INSTALL_DIR"
    else
        log_info "Installing to system directory: $INSTALL_DIR"
    fi
    
    # Check if we need sudo
    local use_sudo=""
    if [ ! -w "$INSTALL_DIR" ]; then
        if command -v sudo &> /dev/null; then
            use_sudo="sudo"
            log_warn "Using sudo for installation to $INSTALL_DIR"
        else
            log_error "No write permission to $INSTALL_DIR and sudo not available"
            exit 1
        fi
    fi
    
    # Install binary
    log_info "Installing binary..."
    $use_sudo cp "$binary_path" "$INSTALL_DIR/hive"
    $use_sudo chmod +x "$INSTALL_DIR/hive"
    
    # Verify installation
    if [ -x "$INSTALL_DIR/hive" ]; then
        log_success "Binary installed successfully"
    else
        log_error "Binary installation failed"
        exit 1
    fi
}

install_completions() {
    log_info "Installing shell completions..."
    
    # Create completions directory if it doesn't exist
    local bash_completion_dir="${HOME}/.local/share/bash-completion/completions"
    local zsh_completion_dir="${HOME}/.local/share/zsh/site-functions"
    local fish_completion_dir="${HOME}/.config/fish/completions"
    
    mkdir -p "$bash_completion_dir" "$zsh_completion_dir" "$fish_completion_dir"
    
    # Generate and install completions
    if [ -d "completions" ]; then
        # Use pre-generated completions
        [ -f "completions/hive.bash" ] && cp "completions/hive.bash" "$bash_completion_dir/hive"
        [ -f "completions/_hive" ] && cp "completions/_hive" "$zsh_completion_dir/"
        [ -f "completions/hive.fish" ] && cp "completions/hive.fish" "$fish_completion_dir/"
    else
        # Generate completions on-the-fly
        "$INSTALL_DIR/hive" completion bash > "$bash_completion_dir/hive" 2>/dev/null || log_warn "Failed to generate bash completions"
        "$INSTALL_DIR/hive" completion zsh > "$zsh_completion_dir/_hive" 2>/dev/null || log_warn "Failed to generate zsh completions"
        "$INSTALL_DIR/hive" completion fish > "$fish_completion_dir/hive.fish" 2>/dev/null || log_warn "Failed to generate fish completions"
    fi
    
    log_success "Shell completions installed"
}

setup_config() {
    log_info "Setting up configuration..."
    
    # Create config directory
    mkdir -p "$CONFIG_DIR"
    
    # Create default config if it doesn't exist
    local config_file="$CONFIG_DIR/config.toml"
    if [ ! -f "$config_file" ]; then
        "$INSTALL_DIR/hive" config init || {
            log_warn "Failed to initialize config, creating minimal config"
            cat > "$config_file" << 'EOF'
[general]
auto_update = true
telemetry = false

[consensus]
default_profile = "Consensus_Balanced"

[ui]
enable_tui = true
theme = "default"
EOF
        }
        log_success "Default configuration created"
    else
        log_info "Configuration already exists"
    fi
}

update_path() {
    log_info "Updating PATH..."
    
    # Check if directory is already in PATH
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        log_info "Directory already in PATH"
        return 0
    fi
    
    # Determine which shell RC file to update
    local shell_rc=""
    case "${SHELL##*/}" in
        bash)
            if [ -f "${HOME}/.bashrc" ]; then
                shell_rc="${HOME}/.bashrc"
            elif [ -f "${HOME}/.bash_profile" ]; then
                shell_rc="${HOME}/.bash_profile"
            fi
            ;;
        zsh)
            shell_rc="${HOME}/.zshrc"
            ;;
        fish)
            shell_rc="${HOME}/.config/fish/config.fish"
            mkdir -p "$(dirname "$shell_rc")"
            ;;
    esac
    
    if [ -n "$shell_rc" ]; then
        # Add to shell RC file
        local path_export=""
        case "${SHELL##*/}" in
            fish)
                path_export="set -gx PATH $INSTALL_DIR \$PATH"
                ;;
            *)
                path_export="export PATH=\"$INSTALL_DIR:\$PATH\""
                ;;
        esac
        
        if ! grep -q "$INSTALL_DIR" "$shell_rc" 2>/dev/null; then
            echo "" >> "$shell_rc"
            echo "# Added by HiveTechs Consensus installer" >> "$shell_rc"
            echo "$path_export" >> "$shell_rc"
            log_success "Added $INSTALL_DIR to PATH in $shell_rc"
        else
            log_info "PATH already configured in $shell_rc"
        fi
    else
        log_warn "Could not determine shell RC file to update"
        log_info "Please manually add $INSTALL_DIR to your PATH"
    fi
}

run_post_install_checks() {
    log_info "Running post-installation checks..."
    
    # Test binary execution
    if "$INSTALL_DIR/hive" --version &>/dev/null; then
        local version
        version=$("$INSTALL_DIR/hive" --version | head -1)
        log_success "Installation verified: $version"
    else
        log_error "Binary execution test failed"
        exit 1
    fi
    
    # Check for updates capability
    if "$INSTALL_DIR/hive" update --check &>/dev/null; then
        log_success "Auto-update system working"
    else
        log_warn "Auto-update check failed (may be expected for development builds)"
    fi
}

show_completion_message() {
    echo
    log_success "🎉 HiveTechs Consensus installed successfully!"
    echo
    echo -e "${BOLD}Quick Start:${NC}"
    echo "  hive --help                   # Show help"
    echo "  hive ask 'Hello World'        # Ask a question"
    echo "  hive analyze .                # Analyze current directory"
    echo "  hive config show              # Show configuration"
    echo
    echo -e "${BOLD}Next Steps:${NC}"
    echo "  1. Restart your shell or run: source ~/.bashrc"
    echo "  2. Configure your API keys: hive config setup"
    echo "  3. Try the TUI mode: hive (without arguments)"
    echo
    echo -e "${BOLD}Documentation:${NC}"
    echo "  https://github.com/${GITHUB_REPO}"
    echo
    echo -e "${BLUE}Happy coding! 🚀${NC}"
    echo
}

main() {
    print_banner
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --help)
                echo "HiveTechs Consensus Installer"
                echo
                echo "Usage: $0 [options]"
                echo
                echo "Options:"
                echo "  --version VERSION    Install specific version (default: latest)"
                echo "  --install-dir DIR    Install directory (default: /usr/local/bin)"
                echo "  --help              Show this help"
                echo
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    check_dependencies
    detect_platform
    get_latest_version
    download_and_extract
    install_binary
    install_completions
    setup_config
    update_path
    run_post_install_checks
    show_completion_message
}

# Check if script is being piped from curl
if [ -t 0 ]; then
    # Interactive mode
    main "$@"
else
    # Piped mode - run with default options
    main
fi