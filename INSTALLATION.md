# HiveTechs Consensus - Installation Guide

> Complete installation instructions for HiveTechs Consensus across all platforms

## Quick Installation

### Recommended: Global Installation

```bash
# Install using our installer script (recommended)
curl -sSL https://install.hivetechs.com | bash

# Or using Homebrew (macOS/Linux)
brew install hivetechs/tap/hive

# Or using Cargo
cargo install hive-consensus
```

### Platform-Specific Installation

#### macOS

```bash
# Using Homebrew (recommended)
brew tap hivetechs/tap
brew install hive

# Using installer package
curl -O https://releases.hivetechs.com/latest/hive-macos.pkg
sudo installer -pkg hive-macos.pkg -target /

# Manual binary installation
curl -L https://releases.hivetechs.com/latest/hive-macos.tar.gz | tar xz
sudo mv hive /usr/local/bin/
```

#### Linux

```bash
# Using package manager (Ubuntu/Debian)
curl -fsSL https://packages.hivetechs.com/gpg | sudo apt-key add -
echo "deb https://packages.hivetechs.com/apt stable main" | sudo tee /etc/apt/sources.list.d/hivetechs.list
sudo apt update && sudo apt install hive

# Using RPM (CentOS/RHEL/Fedora)
sudo rpm --import https://packages.hivetechs.com/rpm/gpg
sudo yum-config-manager --add-repo https://packages.hivetechs.com/rpm/
sudo yum install hive

# Manual binary installation
curl -L https://releases.hivetechs.com/latest/hive-linux.tar.gz | tar xz
sudo mv hive /usr/local/bin/
```

#### Windows

```powershell
# Using Chocolatey (recommended)
choco install hive-consensus

# Using Scoop
scoop bucket add hivetechs https://github.com/hivetechs/scoop-bucket
scoop install hive

# Manual installer download
# Download hive-windows.msi from https://releases.hivetechs.com/latest/
# Run the installer with administrator privileges
```

### Build from Source

```bash
# Prerequisites: Rust 1.70+, Git
git clone https://github.com/hivetechs/hive-consensus
cd hive-consensus

# Build release binary
cargo build --release

# Install globally
cargo install --path .

# Or copy binary manually
sudo cp target/release/hive /usr/local/bin/
```

## Post-Installation Setup

### 1. Verify Installation

```bash
# Check version and basic functionality
hive --version
hive health

# Test system connectivity
hive test --integration
```

### 2. Initial Configuration

```bash
# Interactive setup wizard
hive setup

# Or manual configuration
hive config init
```

### 3. API Key Configuration

You'll need API keys for AI model providers:

#### OpenRouter (Required)
1. Visit [OpenRouter](https://openrouter.ai/) and create an account
2. Generate an API key from your dashboard
3. Configure Hive:

```bash
hive config set openrouter.api_key "sk-or-YOUR-API-KEY"
```

#### Optional Providers
- **Anthropic**: Direct Claude access
- **OpenAI**: Direct GPT access
- **Google**: Gemini models

```bash
hive config set anthropic.api_key "sk-ant-YOUR-KEY"
hive config set openai.api_key "sk-YOUR-KEY"
hive config set google.api_key "YOUR-GOOGLE-KEY"
```

### 4. License Activation

For enterprise features, activate your license:

```bash
hive license activate "YOUR-LICENSE-KEY"
```

### 5. Shell Completion Setup

#### Bash
```bash
hive completions bash >> ~/.bashrc
```

#### Zsh
```bash
hive completions zsh >> ~/.zshrc
```

#### Fish
```bash
hive completions fish > ~/.config/fish/completions/hive.fish
```

#### PowerShell
```powershell
hive completions powershell >> $PROFILE
```

## IDE Integration

### VS Code / Cursor / Windsurf

1. Install the "HiveTechs Consensus" extension from the marketplace
2. Configure your API keys in extension settings
3. Use `Cmd+Shift+H` (Mac) or `Ctrl+Shift+H` (Windows/Linux) to activate

### Vim/Neovim

```lua
-- Add to your init.lua
require('hive').setup({
  consensus_profile = 'balanced',
  auto_apply = true,
  keybindings = {
    ask = '<leader>ha',
    apply = '<leader>hp',
    analyze = '<leader>hr',
  }
})
```

### Emacs

```elisp
;; Add to your init.el
(use-package hive-mode
  :ensure t
  :config
  (setq hive-consensus-profile "balanced")
  (global-set-key (kbd "C-c h a") 'hive-ask)
  (global-set-key (kbd "C-c h p") 'hive-apply))
```

### Sublime Text

1. Install Package Control if not already installed
2. Install "HiveTechs Consensus" package
3. Configure API keys in package settings

### IntelliJ IDEA / JetBrains IDEs

1. Install "HiveTechs Consensus" plugin from marketplace
2. Configure API keys in plugin settings
3. Use `Ctrl+Alt+H` to open Hive panel

## Docker Usage

### Quick Start with Docker

```bash
# Run Hive in a container
docker run --rm -it \
  -v $(pwd):/workspace \
  -e OPENROUTER_API_KEY="your-key" \
  hivetechs/hive:latest \
  hive ask "What does this codebase do?"

# Or use docker-compose
cat > docker-compose.yml << EOF
version: '3.8'
services:
  hive:
    image: hivetechs/hive:latest
    volumes:
      - .:/workspace
    environment:
      - OPENROUTER_API_KEY=${OPENROUTER_API_KEY}
    working_dir: /workspace
EOF

docker-compose run hive ask "Analyze this code"
```

## Configuration

### Default Configuration

Hive creates a configuration file at `~/.hive/config.toml`:

```toml
[consensus]
profile = "balanced"  # speed, cost, balanced, elite
streaming = true
temperature = 0.7

[models]
generator = "anthropic/claude-3-opus"
refiner = "openai/gpt-4-turbo"
validator = "anthropic/claude-3-sonnet"
curator = "openai/gpt-4"

[providers]
[providers.openrouter]
api_key = "sk-or-your-key"
base_url = "https://openrouter.ai/api/v1"

[providers.anthropic]
api_key = ""  # Optional: direct Anthropic access

[providers.openai]
api_key = ""  # Optional: direct OpenAI access

[performance]
cache_size = "2GB"
max_workers = 8
incremental_parsing = true
memory_limit = "4GB"

[security]
trust_policy = "prompt"  # always, never, prompt
audit_logging = true
sandbox_mode = false

[integration]
lsp_port = 7777
mcp_port = 7778
rest_api_port = 7779

[analytics]
enabled = true
anonymous_metrics = true
performance_tracking = true
```

### Environment Variables

You can also use environment variables:

```bash
export HIVE_OPENROUTER_API_KEY="sk-or-your-key"
export HIVE_CONSENSUS_PROFILE="balanced"
export HIVE_LOG_LEVEL="info"
export HIVE_CACHE_SIZE="2GB"
export HIVE_MAX_WORKERS="8"
```

## System Requirements

### Minimum Requirements
- **OS**: macOS 10.15+, Linux (glibc 2.31+), Windows 10+
- **RAM**: 2GB available memory
- **Storage**: 1GB free disk space
- **Network**: Internet connection for AI model access

### Recommended Requirements
- **RAM**: 8GB+ for large codebases
- **Storage**: 10GB+ for comprehensive caching
- **CPU**: Multi-core processor for parallel processing
- **Network**: Stable broadband for streaming responses

### Large Codebase Support
- **RAM**: 16GB+ for repositories with 1M+ lines
- **Storage**: SSD recommended for symbol indexing
- **CPU**: 8+ cores for optimal performance

## Troubleshooting

### Common Installation Issues

#### "Command not found" after installation
```bash
# Check if binary is in PATH
which hive

# Add to PATH if needed (add to ~/.bashrc or ~/.zshrc)
export PATH="/usr/local/bin:$PATH"

# Reload shell
source ~/.bashrc  # or ~/.zshrc
```

#### Permission denied errors
```bash
# Fix binary permissions
sudo chmod +x /usr/local/bin/hive

# Or install to user directory
cargo install --root ~/.local hive-consensus
export PATH="$HOME/.local/bin:$PATH"
```

#### SSL/TLS certificate errors
```bash
# Update system certificates
sudo apt update && sudo apt install ca-certificates  # Ubuntu/Debian
brew install ca-certificates  # macOS

# Set certificate bundle location
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
```

### Performance Issues

#### Slow startup times
```bash
# Clear cache and rebuild
hive cache clear
hive index rebuild

# Reduce cache size if memory constrained
hive config set performance.cache_size "512MB"
```

#### High memory usage
```bash
# Monitor memory usage
hive performance monitor

# Adjust worker count
hive config set performance.max_workers 4

# Enable memory limit
hive config set performance.memory_limit "2GB"
```

### Network Issues

#### API connectivity problems
```bash
# Test connectivity
hive test --network

# Check proxy settings
export HTTPS_PROXY="http://proxy.example.com:8080"
export HTTP_PROXY="http://proxy.example.com:8080"

# Use alternative API endpoints
hive config set providers.openrouter.base_url "https://api.openrouter.ai/api/v1"
```

## Uninstallation

### Complete Removal

```bash
# Remove binary
sudo rm -f /usr/local/bin/hive

# Remove configuration and cache
rm -rf ~/.hive

# Remove shell completions
# (remove lines added to ~/.bashrc, ~/.zshrc, etc.)

# Package manager uninstall
brew uninstall hive              # Homebrew
sudo apt remove hive             # Ubuntu/Debian
sudo yum remove hive             # CentOS/RHEL
choco uninstall hive-consensus   # Chocolatey
scoop uninstall hive             # Scoop
```

### Keep Configuration

```bash
# Remove only binary, keep config
sudo rm -f /usr/local/bin/hive

# Config preserved in ~/.hive/
```

## Next Steps

After installation, see:
- [USER_GUIDE.md](USER_GUIDE.md) - Complete usage guide
- [API_REFERENCE.md](API_REFERENCE.md) - CLI command reference
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Common issues and solutions
- [ENTERPRISE_GUIDE.md](ENTERPRISE_GUIDE.md) - Team and enterprise features

## Support

- **Documentation**: https://docs.hivetechs.com
- **GitHub Issues**: https://github.com/hivetechs/hive-consensus/issues
- **Community Discord**: https://discord.gg/hivetechs
- **Enterprise Support**: enterprise@hivetechs.com

---

**Need help?** Join our community Discord or open an issue on GitHub. Our team and community are here to help you get the most out of HiveTechs Consensus.