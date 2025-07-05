#!/bin/bash

# IDE Setup Script for Hive AI Integration
# This script sets up MCP and LSP servers for IDE integration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DEFAULT_MCP_PORT=7777
DEFAULT_LSP_PORT=8080
HIVE_CONFIG_DIR="$HOME/.hive"
IDE_CONFIG_DIR="$HIVE_CONFIG_DIR/ide"

print_header() {
    echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
    echo -e "${BLUE}│  🐝 Hive AI IDE Integration Setup     │${NC}"
    echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
    echo
}

print_step() {
    echo -e "${GREEN}▶${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Check if hive is installed
check_hive_installation() {
    if ! command -v hive &> /dev/null; then
        print_error "Hive AI is not installed or not in PATH"
        echo "Please install Hive AI first: https://github.com/hivetechs/hive"
        exit 1
    fi
    print_success "Hive AI installation found"
}

# Create IDE configuration directory
setup_config_directory() {
    print_step "Setting up IDE configuration directory..."
    
    mkdir -p "$IDE_CONFIG_DIR"
    mkdir -p "$IDE_CONFIG_DIR/vscode"
    mkdir -p "$IDE_CONFIG_DIR/cursor"
    mkdir -p "$IDE_CONFIG_DIR/neovim"
    mkdir -p "$IDE_CONFIG_DIR/scripts"
    
    print_success "Configuration directory created: $IDE_CONFIG_DIR"
}

# Generate MCP configuration
generate_mcp_config() {
    print_step "Generating MCP server configuration..."
    
    cat > "$IDE_CONFIG_DIR/mcp-config.json" << EOF
{
  "mcpServers": {
    "hive-ai": {
      "command": "hive",
      "args": ["mcp", "start", "--port", "$DEFAULT_MCP_PORT"],
      "env": {
        "HIVE_MCP_MODE": "server"
      }
    }
  }
}
EOF
    
    print_success "MCP configuration generated"
}

# Generate LSP configuration
generate_lsp_config() {
    print_step "Generating LSP server configuration..."
    
    cat > "$IDE_CONFIG_DIR/lsp-config.json" << EOF
{
  "servers": {
    "hive-ai": {
      "command": "hive",
      "args": ["lsp", "start", "--stdio"],
      "filetypes": ["rust", "javascript", "typescript", "python", "java", "cpp", "c", "go", "php", "ruby"],
      "settings": {
        "hive": {
          "consensusProfile": "balanced",
          "enableCompletions": true,
          "enableDiagnostics": true,
          "enableRefactoring": true,
          "enableDocumentation": true
        }
      }
    }
  }
}
EOF
    
    print_success "LSP configuration generated"
}

# Generate VS Code settings
generate_vscode_config() {
    print_step "Generating VS Code configuration..."
    
    cat > "$IDE_CONFIG_DIR/vscode/settings.json" << EOF
{
  "hive.mcpServerUrl": "http://127.0.0.1:$DEFAULT_MCP_PORT",
  "hive.consensusProfile": "balanced",
  "hive.autoAnalyze": true,
  "hive.enableDiagnostics": true,
  "hive.enableCompletions": true,
  "hive.supportedLanguages": [".rs", ".js", ".ts", ".py", ".java", ".cpp", ".c", ".go", ".php", ".rb"]
}
EOF
    
    cat > "$IDE_CONFIG_DIR/vscode/launch.json" << EOF
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Start Hive MCP Server",
      "type": "node",
      "request": "launch",
      "program": "hive",
      "args": ["mcp", "start", "--port", "$DEFAULT_MCP_PORT"],
      "console": "integratedTerminal"
    },
    {
      "name": "Start Hive LSP Server",
      "type": "node",
      "request": "launch",
      "program": "hive",
      "args": ["lsp", "start", "--stdio"],
      "console": "integratedTerminal"
    }
  ]
}
EOF
    
    print_success "VS Code configuration generated"
}

# Generate Cursor IDE configuration
generate_cursor_config() {
    print_step "Generating Cursor IDE configuration..."
    
    cat > "$IDE_CONFIG_DIR/cursor/composer_spec.json" << EOF
{
  "early_access": true,
  "rules": [
    {
      "pattern": "**/*.{rs,js,ts,py,java,cpp,c,go,php,rb}",
      "description": "Use Hive AI for code analysis and suggestions",
      "tools": [
        {
          "name": "hive-ai",
          "endpoint": "http://127.0.0.1:$DEFAULT_MCP_PORT",
          "type": "mcp"
        }
      ]
    }
  ]
}
EOF
    
    cat > "$IDE_CONFIG_DIR/cursor/mcp_servers.json" << EOF
{
  "hive-ai": {
    "command": "hive",
    "args": ["mcp", "start", "--port", "$DEFAULT_MCP_PORT"]
  }
}
EOF
    
    print_success "Cursor IDE configuration generated"
}

# Generate Neovim configuration
generate_neovim_config() {
    print_step "Generating Neovim configuration..."
    
    cat > "$IDE_CONFIG_DIR/neovim/hive-lsp.lua" << EOF
-- Hive AI LSP Configuration for Neovim
local lspconfig = require('lspconfig')

-- Configure Hive AI LSP
local configs = require('lspconfig.configs')

if not configs.hive then
  configs.hive = {
    default_config = {
      cmd = { 'hive', 'lsp', 'start', '--stdio' },
      filetypes = { 'rust', 'javascript', 'typescript', 'python', 'java', 'cpp', 'c', 'go', 'php', 'ruby' },
      root_dir = lspconfig.util.root_pattern('.git', 'package.json', 'Cargo.toml'),
      settings = {
        hive = {
          consensusProfile = 'balanced',
          enableCompletions = true,
          enableDiagnostics = true,
          enableRefactoring = true,
          enableDocumentation = true
        }
      }
    },
  }
end

lspconfig.hive.setup {
  on_attach = function(client, bufnr)
    -- Enable completion triggered by <c-x><c-o>
    vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')
    
    -- Mappings
    local opts = { noremap=true, silent=true }
    vim.api.nvim_buf_set_keymap(bufnr, 'n', 'gD', '<cmd>lua vim.lsp.buf.declaration()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', 'gd', '<cmd>lua vim.lsp.buf.definition()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', 'K', '<cmd>lua vim.lsp.buf.hover()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', 'gi', '<cmd>lua vim.lsp.buf.implementation()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', '<C-k>', '<cmd>lua vim.lsp.buf.signature_help()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', '<space>rn', '<cmd>lua vim.lsp.buf.rename()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', '<space>ca', '<cmd>lua vim.lsp.buf.code_action()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', 'gr', '<cmd>lua vim.lsp.buf.references()<CR>', opts)
    vim.api.nvim_buf_set_keymap(bufnr, 'n', '<space>f', '<cmd>lua vim.lsp.buf.formatting()<CR>', opts)
  end,
  flags = {
    debounce_text_changes = 150,
  }
}

-- Hive AI specific commands
vim.api.nvim_create_user_command('HiveAsk', function(opts)
  vim.fn.system('hive ask "' .. opts.args .. '"')
end, { nargs = 1 })

vim.api.nvim_create_user_command('HiveAnalyze', function()
  vim.fn.system('hive analyze .')
end, {})

vim.api.nvim_create_user_command('HiveExplain', function()
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  local content = table.concat(lines, "\\n")
  vim.fn.system('hive ask "Explain this code: ' .. content .. '"')
end, {})
EOF
    
    cat > "$IDE_CONFIG_DIR/neovim/init.vim" << EOF
" Hive AI Integration for Neovim
" Add this to your init.vim or init.lua

" Load Hive AI LSP configuration
lua require('hive-lsp')

" Hive AI keybindings
nnoremap <leader>ha :HiveAsk<Space>
nnoremap <leader>hA :HiveAnalyze<CR>
nnoremap <leader>he :HiveExplain<CR>
EOF
    
    print_success "Neovim configuration generated"
}

# Generate universal setup script
generate_universal_script() {
    print_step "Generating universal IDE setup script..."
    
    cat > "$IDE_CONFIG_DIR/scripts/setup-ide.sh" << 'EOF'
#!/bin/bash

# Universal IDE Setup Script for Hive AI
# This script detects your IDE and sets up Hive AI integration

detect_ide() {
    if command -v code &> /dev/null; then
        echo "vscode"
    elif command -v cursor &> /dev/null; then
        echo "cursor"
    elif command -v nvim &> /dev/null; then
        echo "neovim"
    elif command -v vim &> /dev/null; then
        echo "vim"
    else
        echo "unknown"
    fi
}

setup_vscode() {
    echo "Setting up VS Code..."
    
    # Create VS Code settings directory if it doesn't exist
    VSCODE_DIR="$HOME/.vscode"
    mkdir -p "$VSCODE_DIR"
    
    # Install Hive AI extension (if available)
    if [ -f "$HOME/.hive/ide/vscode/hive-ai.vsix" ]; then
        code --install-extension "$HOME/.hive/ide/vscode/hive-ai.vsix"
    fi
    
    # Copy settings
    cp "$HOME/.hive/ide/vscode/settings.json" "$VSCODE_DIR/settings.json"
    
    echo "VS Code setup complete!"
}

setup_cursor() {
    echo "Setting up Cursor IDE..."
    
    # Cursor-specific setup
    CURSOR_DIR="$HOME/.cursor"
    mkdir -p "$CURSOR_DIR"
    
    # Copy MCP configuration
    cp "$HOME/.hive/ide/cursor/mcp_servers.json" "$CURSOR_DIR/mcp_servers.json"
    
    echo "Cursor IDE setup complete!"
}

setup_neovim() {
    echo "Setting up Neovim..."
    
    # Neovim configuration directory
    NVIM_DIR="$HOME/.config/nvim"
    mkdir -p "$NVIM_DIR/lua"
    
    # Copy Hive AI LSP configuration
    cp "$HOME/.hive/ide/neovim/hive-lsp.lua" "$NVIM_DIR/lua/hive-lsp.lua"
    
    # Add to init.lua if it exists
    if [ -f "$NVIM_DIR/init.lua" ]; then
        echo "require('hive-lsp')" >> "$NVIM_DIR/init.lua"
    fi
    
    echo "Neovim setup complete!"
}

# Main setup
IDE=$(detect_ide)

case $IDE in
    vscode)
        setup_vscode
        ;;
    cursor)
        setup_cursor
        ;;
    neovim)
        setup_neovim
        ;;
    *)
        echo "IDE not detected or supported. Please set up manually."
        echo "Available configurations in: $HOME/.hive/ide/"
        ;;
esac
EOF
    
    chmod +x "$IDE_CONFIG_DIR/scripts/setup-ide.sh"
    print_success "Universal setup script generated"
}

# Test MCP and LSP servers
test_servers() {
    print_step "Testing server connectivity..."
    
    # Test MCP server
    echo "Testing MCP server..."
    if hive mcp start --port $DEFAULT_MCP_PORT &
    then
        MCP_PID=$!
        sleep 2
        
        if hive mcp status --port $DEFAULT_MCP_PORT; then
            print_success "MCP server test passed"
        else
            print_warning "MCP server test failed"
        fi
        
        kill $MCP_PID 2>/dev/null
    fi
    
    # Test LSP server
    echo "Testing LSP server..."
    if timeout 5 hive lsp start --stdio < /dev/null; then
        print_success "LSP server test passed"
    else
        print_warning "LSP server test failed or timeout"
    fi
}

# Generate service files for auto-start
generate_service_files() {
    print_step "Generating service files..."
    
    # systemd service file
    cat > "$IDE_CONFIG_DIR/scripts/hive-mcp.service" << EOF
[Unit]
Description=Hive AI MCP Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME
ExecStart=$(which hive) mcp start --port $DEFAULT_MCP_PORT
Restart=always
RestartSec=10
Environment=HOME=$HOME
Environment=HIVE_MCP_MODE=service

[Install]
WantedBy=multi-user.target
EOF
    
    # macOS LaunchAgent
    cat > "$IDE_CONFIG_DIR/scripts/com.hivetechs.hive.mcp.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.hivetechs.hive.mcp</string>
    <key>ProgramArguments</key>
    <array>
        <string>$(which hive)</string>
        <string>mcp</string>
        <string>start</string>
        <string>--port</string>
        <string>$DEFAULT_MCP_PORT</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>$HOME/Library/Logs/hive-mcp.log</string>
    <key>StandardOutPath</key>
    <string>$HOME/Library/Logs/hive-mcp.log</string>
</dict>
</plist>
EOF
    
    print_success "Service files generated"
}

# Print usage instructions
print_usage() {
    echo
    echo -e "${BLUE}🎉 IDE Integration Setup Complete!${NC}"
    echo
    echo "Next steps:"
    echo "1. Run the universal setup script:"
    echo "   $IDE_CONFIG_DIR/scripts/setup-ide.sh"
    echo
    echo "2. Start the servers:"
    echo "   hive mcp start --port $DEFAULT_MCP_PORT"
    echo "   hive lsp start --stdio"
    echo
    echo "3. Configure your IDE:"
    echo "   • VS Code: Install Hive AI extension"
    echo "   • Cursor: MCP servers configured automatically"
    echo "   • Neovim: Add hive-lsp.lua to your config"
    echo
    echo "4. Available commands:"
    echo "   hive mcp tools      # List available MCP tools"
    echo "   hive mcp resources  # List available resources"
    echo "   hive lsp capabilities # Show LSP capabilities"
    echo
    echo "Configuration files are in: $IDE_CONFIG_DIR"
}

# Main execution
main() {
    print_header
    
    check_hive_installation
    setup_config_directory
    generate_mcp_config
    generate_lsp_config
    generate_vscode_config
    generate_cursor_config
    generate_neovim_config
    generate_universal_script
    test_servers
    generate_service_files
    
    print_usage
}

# Run main function
main "$@"