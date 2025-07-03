# Hive AI Shell Integration

Professional shell integration for Hive AI with comprehensive completions, PATH setup, and convenient aliases.

## Features

- **🐚 Multi-Shell Support**: Bash, Zsh, Fish, PowerShell, Elvish
- **⚡ Fast Completions**: Context-aware completions with <100ms response time
- **🎯 Smart Suggestions**: Project-type detection and relevant command suggestions
- **🔧 Automatic Setup**: PATH configuration and environment setup
- **📦 Professional Aliases**: Convenient shortcuts for common operations
- **🛡️ Safe Installation**: Backup creation and clean uninstall process

## Quick Start

### Automatic Installation

```bash
# Install for current shell
./setup/install.sh

# Install for specific shell
./setup/install.sh --shell bash
./setup/install.sh --shell zsh
./setup/install.sh --shell fish

# Install for all shells
./setup/install.sh --shell all
```

### Using Hive AI Binary

```bash
# Install shell integration
hive shell install

# Setup PATH and environment
hive shell setup

# Check integration status
hive shell status

# Generate completion files
hive shell completions --output ./completions

# Uninstall integration
hive shell uninstall
```

## Shell-Specific Instructions

### Bash

**Installation:**
```bash
cp completions/hive.bash ~/.bash_completion.d/hive
echo 'source ~/.bash_completion.d/hive' >> ~/.bashrc
```

**Features:**
- Context-aware completions for all commands
- Smart directory and file suggestions
- Professional aliases and functions
- Project type detection

### Zsh

**Installation:**
```bash
mkdir -p ~/.zsh/completions
cp completions/_hive ~/.zsh/completions/
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc
```

**Features:**
- Rich descriptions in completion menu
- Oh-My-Zsh compatibility
- Enhanced context detection
- Smart directory completion

### Fish

**Installation:**
```bash
mkdir -p ~/.config/fish/completions
cp completions/hive.fish ~/.config/fish/completions/
```

**Features:**
- Native Fish abbreviations
- Rich emoji descriptions
- Context-aware suggestions
- Event-driven enhancements

### PowerShell

**Installation:**
```powershell
# Copy completion script
Copy-Item completions/hive.ps1 $PROFILE

# Or add to existing profile
Add-Content $PROFILE (Get-Content completions/hive.ps1)
```

**Features:**
- IntelliSense integration
- Parameter validation
- Rich help text
- PowerShell cmdlet support

## Professional Aliases

All shells include these convenient aliases:

```bash
ha          # hive analyze
hq          # hive ask
hp          # hive plan
hs          # hive search
hm          # hive memory search
ht          # hive trust check .
hc          # hive config show
hst         # hive status
htui        # hive tui
```

### Advanced Aliases

```bash
hive-quick  # hive ask --profile=speed
hive-best   # hive ask --profile=elite
hive-cheap  # hive ask --profile=cost
hive-here   # hive analyze . --depth=standard
hive-full   # hive analyze . --depth=comprehensive --dependencies --recommendations
hive-check  # hive trust check . && hive status
```

## Shell Functions

### Bash/Zsh Functions

```bash
hive_quick_ask "question"           # Quick AI question with speed profile
hive_analyze_current [depth]        # Analyze current directory
hive_plan_feature "description"     # Plan a new feature interactively
hive_memory_find "search_term"      # Search memory with formatted output
hive_project_status                 # Show comprehensive project status
hive_context_suggest                # Get context-aware suggestions
```

### Fish Functions

```fish
hive_quick_ask "question"           # Quick AI analysis
hive_analyze_current [depth]        # Analyze with specified depth
hive_plan_feature "description"     # Interactive feature planning
hive_memory_find "search_term"      # Memory search with table format
hive_project_status                 # Project status overview
hive_context_suggest                # Context-aware suggestions
```

### PowerShell Functions

```powershell
Invoke-HiveQuickAsk "question"      # Quick AI question
Invoke-HiveAnalyzeCurrent -Depth    # Analyze current directory
Invoke-HivePlanFeature "desc"       # Plan a new feature
Search-HiveMemory "term" -Limit     # Search memory
Get-HiveProjectStatus               # Project status
Get-HiveContextSuggestions          # Context suggestions
```

## Context Awareness

The shell integration automatically detects project types and provides relevant suggestions:

- **🦀 Rust Projects**: Performance-focused analysis suggestions
- **📦 JavaScript/Node.js**: Security-focused recommendations
- **🐍 Python Projects**: Best practices and testing suggestions  
- **🐹 Go Projects**: Efficiency and optimization suggestions
- **💙 PowerShell Projects**: Quality and style improvements
- **📁 Git Repositories**: History analysis and patterns

## Environment Variables

Customize behavior with these environment variables:

```bash
export HIVE_DEFAULT_PROFILE="balanced"     # Default consensus profile
export HIVE_DEFAULT_FORMAT="text"          # Default output format
export HIVE_AUTO_TRUST_CURRENT="false"     # Auto-trust current directory
export HIVE_AUTO_DETECT_CONTEXT="true"     # Enable context detection
export HIVE_AUTO_TUI="false"               # Auto-launch TUI for large terminals
```

## Troubleshooting

### Completions Not Working

1. **Check Installation**:
   ```bash
   hive shell status
   ```

2. **Reload Shell Configuration**:
   ```bash
   source ~/.bashrc    # Bash
   source ~/.zshrc     # Zsh
   exec fish           # Fish
   & $PROFILE          # PowerShell
   ```

3. **Verify PATH**:
   ```bash
   which hive
   echo $PATH
   ```

### Manual Completion Installation

If automatic installation fails, manually install completions:

```bash
# Bash
cp completions/hive.bash /etc/bash_completion.d/hive

# Zsh  
cp completions/_hive /usr/local/share/zsh/site-functions/

# Fish
cp completions/hive.fish /usr/share/fish/completions/
```

### Permission Issues

If you encounter permission errors:

```bash
# Install to user directories instead
mkdir -p ~/.local/share/bash-completion/completions
cp completions/hive.bash ~/.local/share/bash-completion/completions/hive
```

## Uninstallation

### Automatic Uninstallation

```bash
# Remove from all shells (preserves config)
./setup/uninstall.sh

# Remove from specific shell
./setup/uninstall.sh --shell bash

# Complete removal (no backups)
./setup/uninstall.sh --no-preserve
```

### Using Hive AI Binary

```bash
# Remove integration (preserves config)
hive shell uninstall

# Complete removal
hive shell uninstall --no-preserve

# Validate removal
hive shell uninstall --validate
```

### Manual Uninstallation

```bash
# Remove completion files
rm ~/.bash_completion.d/hive
rm ~/.zsh/completions/_hive
rm ~/.config/fish/completions/hive.fish

# Remove shell configuration entries
# Edit ~/.bashrc, ~/.zshrc to remove Hive AI sections
```

## Support

- **Documentation**: [https://docs.hivetechs.com/shell-integration](https://docs.hivetechs.com/shell-integration)
- **Issues**: [https://github.com/hivetechs/hive/issues](https://github.com/hivetechs/hive/issues)
- **Community**: [https://discord.gg/hivetechs](https://discord.gg/hivetechs)

## Advanced Configuration

### Custom Completion Directories

Set custom completion directories with environment variables:

```bash
export HIVE_BASH_COMPLETION_DIR="$HOME/.completions"
export HIVE_ZSH_COMPLETION_DIR="$HOME/.zsh/functions"  
export HIVE_FISH_COMPLETION_DIR="$HOME/.fish/completions"
```

### Performance Tuning

For large repositories, optimize completion performance:

```bash
export HIVE_COMPLETION_CACHE_TTL=300    # Cache completions for 5 minutes
export HIVE_MAX_COMPLETION_ITEMS=50     # Limit completion suggestions
export HIVE_COMPLETION_TIMEOUT=100      # Timeout completions after 100ms
```

### Shell-Specific Features

#### Zsh with Oh-My-Zsh

```bash
# Add to ~/.zshrc before Oh-My-Zsh loading
plugins=(... hive-ai)

# Or load manually after Oh-My-Zsh
source ~/.zsh/completions/_hive
```

#### Fish with Starship

```fish
# Add to ~/.config/fish/config.fish
set -gx STARSHIP_CONFIG ~/.config/starship-hive.toml
```

#### PowerShell with PSReadLine

```powershell
# Enhanced tab completion
Set-PSReadLineKeyHandler -Key Tab -Function MenuComplete
Set-PSReadLineOption -PredictionSource History
```

## License

This shell integration is part of Hive AI and follows the same licensing terms.

---

**🐝 HiveTechs Consensus - AI-Powered Development Intelligence**