#!/bin/bash
# Universal Hive AI installer script
# Usage: curl -fsSL https://hivetechs.com/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Configuration
INSTALL_VERSION=${HIVE_VERSION:-"latest"}
INSTALL_DIR=${HIVE_INSTALL_DIR:-"/usr/local/bin"}
CONFIG_DIR=""
FORCE_INSTALL=${HIVE_FORCE:-false}
QUIET=${HIVE_QUIET:-false}
DRY_RUN=${HIVE_DRY_RUN:-false}

# URLs
BASE_URL="https://releases.hivetechs.com"
API_URL="https://api.hivetechs.com"

# Logging functions
log() {
    if [ "$QUIET" != "true" ]; then
        echo -e "$1"
    fi
}

log_info() {
    log "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    log "${GREEN}✅ $1${NC}"
}

log_warning() {
    log "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}" >&2
}

log_step() {
    log "${CYAN}🔧 $1${NC}"
}

# Banner
show_banner() {
    if [ "$QUIET" != "true" ]; then
        echo -e "${PURPLE}"
        echo "  ╭─────────────────────────────────────────╮"
        echo "  │  🐝 HiveTechs Consensus Installer      │"
        echo "  ╰─────────────────────────────────────────╯"
        echo -e "${NC}"
        echo -e "${WHITE}  AI-powered development assistant${NC}"
        echo -e "${WHITE}  https://hivetechs.com${NC}"
        echo ""
    fi
}

# Platform detection
detect_platform() {
    local platform=""
    local arch=""
    
    # Detect OS
    case "$OSTYPE" in
        darwin*)
            platform="macos"
            ;;
        linux*)
            platform="linux"
            ;;
        msys*|cygwin*|win*)
            platform="windows"
            ;;
        *)
            log_error "Unsupported platform: $OSTYPE"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${platform}-${arch}"
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."
    
    # Check for required commands
    local required_commands=("curl" "tar" "chmod")
    if [ "$(detect_platform)" = "windows" ]; then
        required_commands+=("unzip")
    fi
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command '$cmd' not found"
            exit 1
        fi
    done
    
    # Check if running as root when installing to system directory
    if [[ "$INSTALL_DIR" == "/usr/local/bin" ]] && [ "$EUID" -ne 0 ] && [ "$platform" != "macos" ]; then
        log_warning "Installing to system directory may require sudo privileges"
    fi
    
    log_success "Prerequisites check passed"
}

# Get latest version
get_latest_version() {
    if [ "$INSTALL_VERSION" = "latest" ]; then
        log_step "Fetching latest version..."
        
        local version_response
        version_response=$(curl -fsSL "$API_URL/releases/latest" 2>/dev/null || echo "")
        
        if [ -n "$version_response" ]; then
            # Extract version from JSON response
            INSTALL_VERSION=$(echo "$version_response" | sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        fi
        
        # Fallback to hardcoded version if API fails
        if [ -z "$INSTALL_VERSION" ] || [ "$INSTALL_VERSION" = "latest" ]; then
            INSTALL_VERSION="2.0.0"
            log_warning "Could not fetch latest version, using default: $INSTALL_VERSION"
        else
            log_info "Latest version: $INSTALL_VERSION"
        fi
    fi
}

# Download and verify binary
download_binary() {
    local platform_arch="$1"
    local temp_dir="$2"
    
    log_step "Downloading Hive AI v$INSTALL_VERSION for $platform_arch..."
    
    # Determine file extension
    local file_ext="tar.gz"
    local binary_name="hive"
    if [[ "$platform_arch" == *"windows"* ]]; then
        file_ext="zip"
        binary_name="hive.exe"
    fi
    
    # Download URLs
    local download_url="$BASE_URL/$INSTALL_VERSION/hive-${platform_arch}.${file_ext}"
    local checksum_url="$BASE_URL/$INSTALL_VERSION/hive-${platform_arch}.${file_ext}.sha256"
    
    # Download binary
    local archive_file="$temp_dir/hive-${platform_arch}.${file_ext}"
    if ! curl -fsSL "$download_url" -o "$archive_file"; then
        log_error "Failed to download binary from $download_url"
        exit 1
    fi
    
    # Download and verify checksum
    log_step "Verifying download integrity..."
    local checksum_file="$temp_dir/checksum"
    if curl -fsSL "$checksum_url" -o "$checksum_file" 2>/dev/null; then
        local expected_checksum
        expected_checksum=$(cut -d' ' -f1 "$checksum_file")
        
        local actual_checksum
        if command -v sha256sum &> /dev/null; then
            actual_checksum=$(sha256sum "$archive_file" | cut -d' ' -f1)
        elif command -v shasum &> /dev/null; then
            actual_checksum=$(shasum -a 256 "$archive_file" | cut -d' ' -f1)
        else
            log_warning "Could not verify checksum: sha256sum or shasum not available"
        fi
        
        if [ -n "$actual_checksum" ] && [ "$actual_checksum" != "$expected_checksum" ]; then
            log_error "Checksum verification failed!"
            log_error "Expected: $expected_checksum"
            log_error "Actual:   $actual_checksum"
            exit 1
        elif [ -n "$actual_checksum" ]; then
            log_success "Checksum verified"
        fi
    else
        log_warning "Could not download checksum file for verification"
    fi
    
    # Extract binary
    log_step "Extracting binary..."
    if [[ "$platform_arch" == *"windows"* ]]; then
        unzip -q "$archive_file" -d "$temp_dir"
    else
        tar -xzf "$archive_file" -C "$temp_dir"
    fi
    
    echo "$temp_dir/$binary_name"
}

# Install binary
install_binary() {
    local binary_path="$1"
    local platform_arch="$2"
    
    log_step "Installing Hive AI to $INSTALL_DIR..."
    
    # Create install directory if it doesn't exist
    if [ "$DRY_RUN" != "true" ]; then
        if [[ "$INSTALL_DIR" == "/usr/local/bin" ]] && [ "$(id -u)" -ne 0 ]; then
            # Try to create directory with sudo
            if command -v sudo &> /dev/null; then
                sudo mkdir -p "$INSTALL_DIR"
            else
                mkdir -p "$INSTALL_DIR" 2>/dev/null || {
                    log_error "Cannot create $INSTALL_DIR (try running with sudo)"
                    exit 1
                }
            fi
        else
            mkdir -p "$INSTALL_DIR"
        fi
    fi
    
    # Determine target binary name
    local target_binary="$INSTALL_DIR/hive"
    if [[ "$platform_arch" == *"windows"* ]]; then
        target_binary="$INSTALL_DIR/hive.exe"
    fi
    
    # Backup existing installation
    if [ -f "$target_binary" ] && [ "$FORCE_INSTALL" != "true" ]; then
        log_step "Backing up existing installation..."
        local backup_name="hive.backup.$(date +%Y%m%d-%H%M%S)"
        if [ "$DRY_RUN" != "true" ]; then
            if [[ "$INSTALL_DIR" == "/usr/local/bin" ]] && [ "$(id -u)" -ne 0 ]; then
                sudo mv "$target_binary" "$INSTALL_DIR/$backup_name"
            else
                mv "$target_binary" "$INSTALL_DIR/$backup_name"
            fi
        fi
        log_info "Existing binary backed up as $backup_name"
    fi
    
    # Install new binary
    if [ "$DRY_RUN" != "true" ]; then
        if [[ "$INSTALL_DIR" == "/usr/local/bin" ]] && [ "$(id -u)" -ne 0 ]; then
            sudo cp "$binary_path" "$target_binary"
            sudo chmod +x "$target_binary"
        else
            cp "$binary_path" "$target_binary"
            chmod +x "$target_binary"
        fi
    fi
    
    log_success "Binary installed at $target_binary"
    
    # Add to PATH if necessary
    setup_path
}

# Setup PATH
setup_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_step "Setting up PATH..."
        
        # Determine shell configuration file
        local shell_config=""
        if [ -n "$BASH_VERSION" ]; then
            shell_config="$HOME/.bashrc"
            [ -f "$HOME/.bash_profile" ] && shell_config="$HOME/.bash_profile"
        elif [ -n "$ZSH_VERSION" ]; then
            shell_config="$HOME/.zshrc"
        elif [ -n "$FISH_VERSION" ]; then
            shell_config="$HOME/.config/fish/config.fish"
        fi
        
        if [ -n "$shell_config" ] && [ "$DRY_RUN" != "true" ]; then
            # Add PATH export if not already present
            if ! grep -q "export PATH.*$INSTALL_DIR" "$shell_config" 2>/dev/null; then
                echo "" >> "$shell_config"
                echo "# Added by Hive AI installer" >> "$shell_config"
                echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_config"
                log_info "Added $INSTALL_DIR to PATH in $shell_config"
                log_warning "Please restart your shell or run: source $shell_config"
            fi
        else
            log_warning "Could not automatically add to PATH"
            log_info "Please add $INSTALL_DIR to your PATH manually"
        fi
    fi
}

# Initialize configuration
init_config() {
    log_step "Initializing configuration..."
    
    # Determine config directory
    case "$(detect_platform)" in
        macos*)
            CONFIG_DIR="$HOME/.hive"
            ;;
        linux*)
            if [ -n "$XDG_CONFIG_HOME" ]; then
                CONFIG_DIR="$XDG_CONFIG_HOME/hive"
            else
                CONFIG_DIR="$HOME/.config/hive"
            fi
            ;;
        windows*)
            CONFIG_DIR="$APPDATA/HiveTechs/HiveAI"
            ;;
    esac
    
    if [ "$DRY_RUN" != "true" ]; then
        # Create config directory
        mkdir -p "$CONFIG_DIR"
        
        # Initialize configuration with hive init
        local binary_name="hive"
        if [[ "$(detect_platform)" == *"windows"* ]]; then
            binary_name="hive.exe"
        fi
        
        if [ -x "$INSTALL_DIR/$binary_name" ]; then
            "$INSTALL_DIR/$binary_name" init --global --quiet || {
                log_warning "Could not initialize configuration automatically"
                log_info "Please run 'hive init' after installation"
            }
        fi
    fi
    
    log_success "Configuration directory: $CONFIG_DIR"
}

# Generate shell completions
setup_completions() {
    log_step "Setting up shell completions..."
    
    local binary_name="hive"
    if [[ "$(detect_platform)" == *"windows"* ]]; then
        binary_name="hive.exe"
        log_info "Shell completions not supported on Windows"
        return
    fi
    
    if [ "$DRY_RUN" = "true" ] || [ ! -x "$INSTALL_DIR/$binary_name" ]; then
        log_info "Skipping shell completions setup"
        return
    fi
    
    # Generate bash completions
    if command -v bash &> /dev/null; then
        local bash_completion_dir="/etc/bash_completion.d"
        if [ -d "$bash_completion_dir" ] && [ -w "$bash_completion_dir" ]; then
            "$INSTALL_DIR/$binary_name" completion bash > "$bash_completion_dir/hive" 2>/dev/null || true
        elif [ -d "$HOME/.bash_completion.d" ]; then
            mkdir -p "$HOME/.bash_completion.d"
            "$INSTALL_DIR/$binary_name" completion bash > "$HOME/.bash_completion.d/hive" 2>/dev/null || true
        fi
    fi
    
    # Generate zsh completions
    if command -v zsh &> /dev/null; then
        local zsh_completion_dir="$HOME/.zsh/completions"
        mkdir -p "$zsh_completion_dir"
        "$INSTALL_DIR/$binary_name" completion zsh > "$zsh_completion_dir/_hive" 2>/dev/null || true
    fi
    
    # Generate fish completions
    if command -v fish &> /dev/null; then
        local fish_completion_dir="$HOME/.config/fish/completions"
        mkdir -p "$fish_completion_dir"
        "$INSTALL_DIR/$binary_name" completion fish > "$fish_completion_dir/hive.fish" 2>/dev/null || true
    fi
    
    log_success "Shell completions installed"
}

# Verify installation
verify_installation() {
    log_step "Verifying installation..."
    
    local binary_name="hive"
    if [[ "$(detect_platform)" == *"windows"* ]]; then
        binary_name="hive.exe"
    fi
    
    local binary_path="$INSTALL_DIR/$binary_name"
    
    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found at $binary_path"
        return 1
    fi
    
    if [ ! -x "$binary_path" ]; then
        log_error "Binary is not executable"
        return 1
    fi
    
    # Test version command
    local version_output
    if version_output=$("$binary_path" --version 2>&1); then
        log_success "Installation verified: $version_output"
    else
        log_error "Binary test failed"
        return 1
    fi
    
    return 0
}

# Show completion message
show_completion() {
    if [ "$QUIET" != "true" ]; then
        echo ""
        echo -e "${GREEN}🎉 Hive AI installation completed successfully!${NC}"
        echo ""
        echo -e "${WHITE}Quick start:${NC}"
        echo -e "  ${CYAN}hive --help${NC}          # Show all commands"
        echo -e "  ${CYAN}hive analyze .${NC}       # Analyze current directory"
        echo -e "  ${CYAN}hive ask \"question\"${NC}   # Ask AI about your code"
        echo ""
        echo -e "${WHITE}Configuration:${NC}"
        echo -e "  Config directory: ${CYAN}$CONFIG_DIR${NC}"
        echo -e "  Binary location:  ${CYAN}$INSTALL_DIR/hive${NC}"
        echo ""
        echo -e "${WHITE}Documentation:${NC}"
        echo -e "  ${CYAN}https://docs.hivetechs.com${NC}"
        echo ""
        
        # Show PATH warning if needed
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            echo -e "${YELLOW}⚠️  Please restart your shell or add $INSTALL_DIR to your PATH${NC}"
            echo ""
        fi
    fi
}

# Cleanup function
cleanup() {
    if [ -n "$TEMP_DIR" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}

# Main installation function
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                INSTALL_VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --force)
                FORCE_INSTALL=true
                shift
                ;;
            --quiet)
                QUIET=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help)
                echo "Hive AI Universal Installer"
                echo ""
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --version VERSION     Install specific version (default: latest)"
                echo "  --install-dir DIR     Installation directory (default: /usr/local/bin)"
                echo "  --force              Force overwrite existing installation"
                echo "  --quiet              Suppress output"
                echo "  --dry-run            Show what would be done without executing"
                echo "  --help               Show this help message"
                echo ""
                echo "Environment variables:"
                echo "  HIVE_VERSION         Version to install"
                echo "  HIVE_INSTALL_DIR     Installation directory"
                echo "  HIVE_FORCE           Force installation"
                echo "  HIVE_QUIET           Quiet mode"
                echo "  HIVE_DRY_RUN         Dry run mode"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Setup cleanup trap
    trap cleanup EXIT
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    
    # Show banner
    show_banner
    
    if [ "$DRY_RUN" = "true" ]; then
        log_warning "DRY RUN MODE - No changes will be made"
        echo ""
    fi
    
    # Run installation steps
    local platform_arch
    platform_arch=$(detect_platform)
    
    check_prerequisites
    get_latest_version
    
    local binary_path
    binary_path=$(download_binary "$platform_arch" "$TEMP_DIR")
    
    install_binary "$binary_path" "$platform_arch"
    init_config
    setup_completions
    
    if [ "$DRY_RUN" != "true" ]; then
        verify_installation
    fi
    
    show_completion
}

# Run main function with all arguments
main "$@"