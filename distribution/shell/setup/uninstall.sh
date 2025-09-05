#!/bin/bash
# Hive AI Shell Integration Uninstaller
# Clean removal of shell integration with backup preservation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🗑️  Hive AI Shell Integration Uninstaller${NC}"
echo "============================================"

# Function to create backup
create_backup() {
    local file="$1"
    local backup_file="${file}.hive-backup"
    
    if [ -f "$file" ]; then
        cp "$file" "$backup_file"
        echo -e "${BLUE}📦 Created backup: $backup_file${NC}"
    fi
}

# Function to remove bash integration
remove_bash_integration() {
    echo -e "${BLUE}🗑️  Removing Bash integration...${NC}"
    
    # Remove completion files
    local completion_files=(
        "$HOME/.bash_completion.d/hive"
        "$HOME/.local/share/bash-completion/completions/hive"
    )
    
    for file in "${completion_files[@]}"; do
        if [ -f "$file" ]; then
            rm "$file"
            echo -e "${GREEN}✅ Removed $file${NC}"
        fi
    done
    
    # Remove from .bashrc
    if [ -f "$HOME/.bashrc" ]; then
        create_backup "$HOME/.bashrc"
        
        # Remove Hive AI section
        sed -i.tmp '/# Hive AI Shell Integration/,/^fi$/d' "$HOME/.bashrc" 2>/dev/null || true
        
        # Clean up temporary file
        [ -f "$HOME/.bashrc.tmp" ] && rm "$HOME/.bashrc.tmp"
        
        echo -e "${GREEN}✅ Cleaned ~/.bashrc${NC}"
    fi
}

# Function to remove zsh integration
remove_zsh_integration() {
    echo -e "${BLUE}🗑️  Removing Zsh integration...${NC}"
    
    # Remove completion files
    local completion_files=(
        "$HOME/.zsh/completions/_hive"
        "$HOME/.oh-my-zsh/completions/_hive"
        "$HOME/.config/zsh/completions/_hive"
    )
    
    for file in "${completion_files[@]}"; do
        if [ -f "$file" ]; then
            rm "$file"
            echo -e "${GREEN}✅ Removed $file${NC}"
        fi
    done
    
    # Remove from .zshrc
    if [ -f "$HOME/.zshrc" ]; then
        create_backup "$HOME/.zshrc"
        
        # Remove Hive AI section
        sed -i.tmp '/# Hive AI Shell Integration/,/autoload -U compinit && compinit/d' "$HOME/.zshrc" 2>/dev/null || true
        
        # Clean up temporary file
        [ -f "$HOME/.zshrc.tmp" ] && rm "$HOME/.zshrc.tmp"
        
        echo -e "${GREEN}✅ Cleaned ~/.zshrc${NC}"
    fi
}

# Function to remove fish integration
remove_fish_integration() {
    echo -e "${BLUE}🗑️  Removing Fish integration...${NC}"
    
    # Remove completion file
    local fish_completion="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/hive.fish"
    
    if [ -f "$fish_completion" ]; then
        rm "$fish_completion"
        echo -e "${GREEN}✅ Removed $fish_completion${NC}"
    fi
}

# Function to validate removal
validate_removal() {
    echo -e "${BLUE}🔍 Validating removal...${NC}"
    
    local issues=0
    
    # Check for remaining completion files
    local completion_files=(
        "$HOME/.bash_completion.d/hive"
        "$HOME/.local/share/bash-completion/completions/hive"
        "$HOME/.zsh/completions/_hive"
        "$HOME/.oh-my-zsh/completions/_hive"
        "$HOME/.config/zsh/completions/_hive"
        "${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/hive.fish"
    )
    
    for file in "${completion_files[@]}"; do
        if [ -f "$file" ]; then
            echo -e "${YELLOW}⚠️  Completion file still exists: $file${NC}"
            ((issues++))
        fi
    done
    
    # Check shell config files
    if [ -f "$HOME/.bashrc" ] && grep -q "Hive AI" "$HOME/.bashrc" 2>/dev/null; then
        echo -e "${YELLOW}⚠️  ~/.bashrc still contains Hive AI references${NC}"
        ((issues++))
    fi
    
    if [ -f "$HOME/.zshrc" ] && grep -q "Hive AI" "$HOME/.zshrc" 2>/dev/null; then
        echo -e "${YELLOW}⚠️  ~/.zshrc still contains Hive AI references${NC}"
        ((issues++))
    fi
    
    if [ $issues -eq 0 ]; then
        echo -e "${GREEN}✅ Shell integration successfully removed${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  $issues issues found - manual cleanup may be required${NC}"
        return 1
    fi
}

# Function to show post-removal instructions
show_instructions() {
    echo ""
    echo -e "${CYAN}📋 Post-Removal Instructions:${NC}"
    echo "  1. Restart your terminal or reload your shell configuration"
    echo "  2. Verify removal: which hive (should return nothing if binary removed)"
    echo ""
    echo -e "${CYAN}🔄 Restore from backup:${NC}"
    echo "  • ~/.bashrc.hive-backup"
    echo "  • ~/.zshrc.hive-backup"
    echo ""
    echo -e "${CYAN}♻️  Complete removal (if desired):${NC}"
    echo "  rm -rf ~/.hive/         # Remove all Hive AI data"
    echo "  # This will delete ALL Hive AI configuration and data"
}

# Main function
main() {
    local preserve_config=true
    local shells_to_remove="all"
    local force=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --shell)
                shells_to_remove="$2"
                shift 2
                ;;
            --no-preserve)
                preserve_config=false
                shift
                ;;
            --force)
                force=true
                shift
                ;;
            --help|-h)
                echo "Hive AI Shell Integration Uninstaller"
                echo ""
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --shell SHELL     Remove from specific shell (bash, zsh, fish, all)"
                echo "  --no-preserve     Don't create backup files"
                echo "  --force           Force removal without confirmation"
                echo "  --help            Show this help message"
                exit 0
                ;;
            *)
                echo -e "${RED}❌ Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done
    
    # Confirmation prompt (unless forced)
    if [ "$force" = false ]; then
        echo -e "${YELLOW}⚠️  This will remove Hive AI shell integration.${NC}"
        if [ "$preserve_config" = true ]; then
            echo "Backups will be created for modified files."
        else
            echo "No backups will be created."
        fi
        echo ""
        read -p "Continue? [y/N]: " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 0
        fi
    fi
    
    # Remove integration based on shell selection
    case "$shells_to_remove" in
        bash)
            remove_bash_integration
            ;;
        zsh)
            remove_zsh_integration
            ;;
        fish)
            remove_fish_integration
            ;;
        all)
            remove_bash_integration
            remove_zsh_integration
            remove_fish_integration
            ;;
        *)
            echo -e "${RED}❌ Unsupported shell: $shells_to_remove${NC}"
            echo "Supported shells: bash, zsh, fish, all"
            exit 1
            ;;
    esac
    
    # Validate removal
    if validate_removal; then
        echo -e "${GREEN}🎉 Shell integration successfully removed!${NC}"
    else
        echo -e "${YELLOW}⚠️  Some issues were found during removal${NC}"
    fi
    
    show_instructions
}

# Run main function
main "$@"