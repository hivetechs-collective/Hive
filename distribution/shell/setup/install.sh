#!/bin/bash
# Hive AI Shell Integration Installer
# Professional shell setup for bash, zsh, and fish

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPLETIONS_DIR="$(dirname "$SCRIPT_DIR")/completions"

echo -e "${CYAN}🐝 Hive AI Shell Integration Installer${NC}"
echo "========================================"

# Function to detect shell
detect_shell() {
    if [ -n "${ZSH_VERSION:-}" ]; then
        echo "zsh"
    elif [ -n "${BASH_VERSION:-}" ]; then
        echo "bash"
    elif [ -n "${FISH_VERSION:-}" ]; then
        echo "fish"
    else
        echo "unknown"
    fi
}

# Function to install bash completion
install_bash_completion() {
    echo -e "${BLUE}📦 Installing Bash completions...${NC}"
    
    # Try user directory first
    if [ -d "$HOME/.bash_completion.d" ] || mkdir -p "$HOME/.bash_completion.d" 2>/dev/null; then
        cp "$COMPLETIONS_DIR/hive.bash" "$HOME/.bash_completion.d/hive"
        echo -e "${GREEN}✅ Installed to ~/.bash_completion.d/hive${NC}"
    elif [ -d "$HOME/.local/share/bash-completion/completions" ] || mkdir -p "$HOME/.local/share/bash-completion/completions" 2>/dev/null; then
        cp "$COMPLETIONS_DIR/hive.bash" "$HOME/.local/share/bash-completion/completions/hive"
        echo -e "${GREEN}✅ Installed to ~/.local/share/bash-completion/completions/hive${NC}"
    else
        echo -e "${YELLOW}⚠️  Could not find suitable bash completion directory${NC}"
        echo "Manual installation required:"
        echo "  1. Copy $COMPLETIONS_DIR/hive.bash to your bash completion directory"
        echo "  2. Source it in your ~/.bashrc"
        return 1
    fi
    
    # Add to .bashrc if not already present
    if [ -f "$HOME/.bashrc" ]; then
        if ! grep -q "hive.*completion" "$HOME/.bashrc"; then
            echo "" >> "$HOME/.bashrc"
            echo "# Hive AI Shell Integration" >> "$HOME/.bashrc"
            echo "if [ -f ~/.bash_completion.d/hive ]; then" >> "$HOME/.bashrc"
            echo "    source ~/.bash_completion.d/hive" >> "$HOME/.bashrc"
            echo "elif [ -f ~/.local/share/bash-completion/completions/hive ]; then" >> "$HOME/.bashrc"
            echo "    source ~/.local/share/bash-completion/completions/hive" >> "$HOME/.bashrc"
            echo "fi" >> "$HOME/.bashrc"
            echo -e "${GREEN}✅ Added to ~/.bashrc${NC}"
        fi
    fi
}

# Function to install zsh completion
install_zsh_completion() {
    echo -e "${BLUE}📦 Installing Zsh completions...${NC}"
    
    # Try various zsh completion directories
    if [ -d "$HOME/.zsh/completions" ] || mkdir -p "$HOME/.zsh/completions" 2>/dev/null; then
        cp "$COMPLETIONS_DIR/_hive" "$HOME/.zsh/completions/_hive"
        COMPLETION_DIR="$HOME/.zsh/completions"
    elif [ -d "$HOME/.oh-my-zsh/completions" ]; then
        cp "$COMPLETIONS_DIR/_hive" "$HOME/.oh-my-zsh/completions/_hive"
        COMPLETION_DIR="$HOME/.oh-my-zsh/completions"
    elif [ -d "$HOME/.config/zsh/completions" ] || mkdir -p "$HOME/.config/zsh/completions" 2>/dev/null; then
        cp "$COMPLETIONS_DIR/_hive" "$HOME/.config/zsh/completions/_hive"
        COMPLETION_DIR="$HOME/.config/zsh/completions"
    else
        echo -e "${YELLOW}⚠️  Could not find suitable zsh completion directory${NC}"
        echo "Manual installation required:"
        echo "  1. Copy $COMPLETIONS_DIR/_hive to your zsh completion directory"
        echo "  2. Add the directory to your fpath in ~/.zshrc"
        return 1
    fi
    
    echo -e "${GREEN}✅ Installed to $COMPLETION_DIR/_hive${NC}"
    
    # Add to .zshrc if not already present
    if [ -f "$HOME/.zshrc" ]; then
        if ! grep -q "fpath.*completions" "$HOME/.zshrc" || ! grep -q "$COMPLETION_DIR" "$HOME/.zshrc"; then
            echo "" >> "$HOME/.zshrc"
            echo "# Hive AI Shell Integration" >> "$HOME/.zshrc"
            echo "fpath=($COMPLETION_DIR \$fpath)" >> "$HOME/.zshrc"
            echo "autoload -U compinit && compinit" >> "$HOME/.zshrc"
            echo -e "${GREEN}✅ Added to ~/.zshrc${NC}"
        fi
    fi
}

# Function to install fish completion
install_fish_completion() {
    echo -e "${BLUE}📦 Installing Fish completions...${NC}"
    
    # Fish config directory
    FISH_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/fish"
    FISH_COMPLETIONS_DIR="$FISH_CONFIG_DIR/completions"
    
    if mkdir -p "$FISH_COMPLETIONS_DIR" 2>/dev/null; then
        cp "$COMPLETIONS_DIR/hive.fish" "$FISH_COMPLETIONS_DIR/hive.fish"
        echo -e "${GREEN}✅ Installed to $FISH_COMPLETIONS_DIR/hive.fish${NC}"
    else
        echo -e "${YELLOW}⚠️  Could not create fish completion directory${NC}"
        echo "Manual installation required:"
        echo "  1. Copy $COMPLETIONS_DIR/hive.fish to ~/.config/fish/completions/"
        return 1
    fi
}

# Function to verify installation
verify_installation() {
    echo -e "${BLUE}🔍 Verifying installation...${NC}"
    
    # Check if hive command is available
    if command -v hive >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Hive AI binary found in PATH${NC}"
        hive --version
    else
        echo -e "${YELLOW}⚠️  Hive AI binary not found in PATH${NC}"
        echo "You may need to:"
        echo "  1. Install Hive AI binary"
        echo "  2. Add Hive AI to your PATH"
    fi
}

# Main installation logic
main() {
    local shell_type=""
    local force_install=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --shell)
                shell_type="$2"
                shift 2
                ;;
            --force)
                force_install=true
                shift
                ;;
            --help|-h)
                echo "Hive AI Shell Integration Installer"
                echo ""
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --shell SHELL    Install for specific shell (bash, zsh, fish, all)"
                echo "  --force          Force installation even if already installed"
                echo "  --help           Show this help message"
                exit 0
                ;;
            *)
                echo -e "${RED}❌ Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done
    
    # Auto-detect shell if not specified
    if [ -z "$shell_type" ]; then
        shell_type=$(detect_shell)
        echo -e "${CYAN}🔍 Detected shell: $shell_type${NC}"
    fi
    
    # Install for specified shell(s)
    case "$shell_type" in
        bash)
            install_bash_completion
            ;;
        zsh)
            install_zsh_completion
            ;;
        fish)
            install_fish_completion
            ;;
        all)
            echo -e "${CYAN}📦 Installing for all shells...${NC}"
            install_bash_completion || true
            install_zsh_completion || true
            install_fish_completion || true
            ;;
        *)
            echo -e "${RED}❌ Unsupported shell: $shell_type${NC}"
            echo "Supported shells: bash, zsh, fish"
            exit 1
            ;;
    esac
    
    verify_installation
    
    echo ""
    echo -e "${GREEN}🎉 Installation complete!${NC}"
    echo ""
    echo -e "${CYAN}📋 Next steps:${NC}"
    echo "  1. Restart your terminal or run: source ~/.${shell_type}rc"
    echo "  2. Test completions: hive <TAB>"
    echo "  3. Try aliases: ha (analyze), hq (ask), hp (plan)"
    echo ""
    echo -e "${CYAN}💡 Quick commands:${NC}"
    echo "  hive shell status    # Check integration status"
    echo "  hive --help          # Show all commands"
    echo "  hive analyze .       # Analyze current directory"
}

# Check for required files
if [ ! -f "$COMPLETIONS_DIR/hive.bash" ] || [ ! -f "$COMPLETIONS_DIR/_hive" ] || [ ! -f "$COMPLETIONS_DIR/hive.fish" ]; then
    echo -e "${RED}❌ Completion files not found in $COMPLETIONS_DIR${NC}"
    echo "Please ensure you have the complete Hive AI distribution."
    exit 1
fi

# Run main function
main "$@"