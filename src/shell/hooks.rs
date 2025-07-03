// Shell Hooks Module for Hive AI
// Shell hooks, aliases, and convenient functions

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;

use crate::core::config::Config;
use super::{ShellType, utils};

/// Shell hooks manager for aliases and convenient functions
pub struct ShellHooks {
    config: Config,
}

impl ShellHooks {
    /// Create new shell hooks manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Install hooks and aliases for a specific shell
    pub fn install(&self, shell: ShellType) -> Result<()> {
        let shell_config = self.get_shell_config_file(shell)?;
        
        // Backup config file before modification
        if shell_config.exists() {
            utils::backup_file(&shell_config)?;
        }

        // Check if hooks are already installed
        if self.are_installed(shell)? {
            tracing::info!("Hooks already installed for {}", shell.as_str());
            return Ok(());
        }

        // Add hooks to shell config
        self.add_hooks_to_config(shell, &shell_config)?;
        
        tracing::info!("Installed hooks and aliases for {} in {}", shell.as_str(), shell_config.display());
        Ok(())
    }

    /// Check if hooks are installed for a shell
    pub fn are_installed(&self, shell: ShellType) -> Result<bool> {
        let shell_config = self.get_shell_config_file(shell)?;
        
        if !shell_config.exists() {
            return Ok(false);
        }

        let content = read_to_string(&shell_config)?;
        Ok(content.contains("# Hive AI Shell Hooks"))
    }

    /// Check if aliases are configured for a shell
    pub fn are_aliases_configured(&self, shell: ShellType) -> Result<bool> {
        let shell_config = self.get_shell_config_file(shell)?;
        
        if !shell_config.exists() {
            return Ok(false);
        }

        let content = read_to_string(&shell_config)?;
        
        // Check for common aliases
        let alias_patterns = match shell {
            ShellType::PowerShell => vec!["Set-Alias -Name ha", "Set-Alias -Name hq"],
            _ => vec!["alias ha='hive analyze'", "alias hq='hive ask'"],
        };

        for pattern in alias_patterns {
            if content.contains(pattern) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get shell configuration file path
    fn get_shell_config_file(&self, shell: ShellType) -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not found")?;
        
        let config_file = match shell {
            ShellType::Bash => {
                let bashrc = PathBuf::from(&home).join(".bashrc");
                if bashrc.exists() {
                    bashrc
                } else {
                    PathBuf::from(&home).join(".bash_profile")
                }
            }
            ShellType::Zsh => PathBuf::from(&home).join(".zshrc"),
            ShellType::Fish => {
                let config_home = std::env::var("XDG_CONFIG_HOME")
                    .unwrap_or_else(|_| format!("{}/.config", home));
                PathBuf::from(config_home).join("fish").join("config.fish")
            }
            ShellType::PowerShell => {
                if cfg!(windows) {
                    PathBuf::from(&home).join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1")
                } else {
                    PathBuf::from(&home).join(".config").join("powershell").join("Microsoft.PowerShell_profile.ps1")
                }
            }
            ShellType::Elvish => PathBuf::from(&home).join(".elvish").join("rc.elv"),
        };

        Ok(config_file)
    }

    /// Add hooks and aliases to shell config
    fn add_hooks_to_config(&self, shell: ShellType, config_file: &PathBuf) -> Result<()> {
        // Create config file if it doesn't exist
        if !config_file.exists() {
            if let Some(parent) = config_file.parent() {
                utils::create_directory_safe(&parent.to_path_buf())?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(config_file)
            .with_context(|| format!("Failed to open config file: {}", config_file.display()))?;

        let hooks_config = self.generate_hooks_config(shell);
        writeln!(file, "\n{}", hooks_config)?;
        
        Ok(())
    }

    /// Generate hooks configuration for a specific shell
    fn generate_hooks_config(&self, shell: ShellType) -> String {
        match shell {
            ShellType::Bash => self.generate_bash_hooks(),
            ShellType::Zsh => self.generate_zsh_hooks(),
            ShellType::Fish => self.generate_fish_hooks(),
            ShellType::PowerShell => self.generate_powershell_hooks(),
            ShellType::Elvish => self.generate_elvish_hooks(),
        }
    }

    /// Generate Bash hooks and aliases
    fn generate_bash_hooks(&self) -> String {
        r#"# Hive AI Shell Hooks
# Added by Hive AI Shell Integration

# Professional aliases
alias ha='hive analyze'
alias hq='hive ask'
alias hp='hive plan'
alias hs='hive search'
alias hm='hive memory search'
alias ht='hive trust check .'
alias hc='hive config show'
alias hst='hive status'
alias htui='hive tui'

# Advanced aliases for power users
alias hive-quick='hive ask --profile=speed'
alias hive-best='hive ask --profile=elite'
alias hive-cheap='hive ask --profile=cost'
alias hive-here='hive analyze . --depth=standard'
alias hive-full='hive analyze . --depth=comprehensive --dependencies --recommendations'
alias hive-check='hive trust check . && hive status'

# Smart shell functions
hive_quick_ask() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: hive_quick_ask <question>"
        echo "Example: hive_quick_ask 'How can I optimize this code?'"
        return 1
    fi
    hive ask "$*" --profile=speed --stream
}

hive_analyze_current() {
    local depth="${1:-standard}"
    echo "🔍 Analyzing current directory with depth: $depth"
    hive analyze . --depth="$depth" --recommendations --format=text
}

hive_plan_feature() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: hive_plan_feature <feature_description>"
        echo "Example: hive_plan_feature 'Add user authentication'"
        return 1
    fi
    echo "📋 Planning feature: $*"
    hive plan "$*" --template=feature --interactive
}

hive_memory_find() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: hive_memory_find <search_term>"
        echo "Example: hive_memory_find 'optimization tips'"
        return 1
    fi
    echo "🧠 Searching memory for: $*"
    hive memory search "$*" --format=table --limit=10
}

hive_project_status() {
    echo "🔍 Hive AI Project Status"
    echo "========================"
    
    # Check trust status
    echo "Trust Status:"
    hive trust check . 2>/dev/null || echo "  Not trusted"
    
    # Check if we're in a git repo
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "Git Repository: ✓"
    else
        echo "Git Repository: ✗"
    fi
    
    # Detect project type
    if [[ -f "Cargo.toml" ]]; then
        echo "Project Type: Rust"
    elif [[ -f "package.json" ]]; then
        echo "Project Type: JavaScript/Node.js"
    elif [[ -f "requirements.txt" || -f "pyproject.toml" ]]; then
        echo "Project Type: Python"
    elif [[ -f "go.mod" ]]; then
        echo "Project Type: Go"
    else
        echo "Project Type: Unknown"
    fi
    
    echo ""
    echo "💡 Suggested commands:"
    echo "  ha .          # Quick analyze"
    echo "  hive-here     # Standard analyze"
    echo "  hive-full     # Comprehensive analyze"
}

hive_context_suggest() {
    local context=""
    
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        context="git"
    fi
    
    if [[ -f "Cargo.toml" ]]; then
        context="$context rust"
        echo "🦀 Rust project detected"
        echo "💡 Try: hive analyze . --focus=performance"
        echo "💡 Ask: 'How can I optimize this Rust code?'"
    fi
    
    if [[ -f "package.json" ]]; then
        context="$context javascript"
        echo "📦 JavaScript/Node.js project detected"
        echo "💡 Try: hive analyze . --focus=security"
        echo "💡 Ask: 'Review this code for security vulnerabilities'"
    fi
    
    if [[ -f "requirements.txt" || -f "pyproject.toml" ]]; then
        context="$context python"
        echo "🐍 Python project detected"
        echo "💡 Try: hive analyze . --include-tests"
        echo "💡 Ask: 'Suggest Python best practices'"
    fi
    
    if [[ -f "go.mod" ]]; then
        context="$context go"
        echo "🐹 Go project detected"
        echo "💡 Try: hive analyze . --focus=efficiency"
        echo "💡 Ask: 'How can I improve Go performance?'"
    fi
    
    if [[ -z "$context" || "$context" == "git" ]]; then
        echo "📁 Generic project detected"
        echo "💡 Try: hive analyze . --depth=standard"
        echo "💡 Ask: 'What are potential improvements?'"
    fi
}

# Export functions for use in scripts
export -f hive_quick_ask
export -f hive_analyze_current
export -f hive_plan_feature
export -f hive_memory_find
export -f hive_project_status
export -f hive_context_suggest

# Context detection hook
hive_auto_context() {
    # Only show context suggestions occasionally (20% chance)
    if (( RANDOM % 5 == 0 )) && command -v hive >/dev/null 2>&1; then
        hive_context_suggest
    fi
}

# Add to PROMPT_COMMAND if not already present
if [[ "$PROMPT_COMMAND" != *"hive_auto_context"* ]]; then
    PROMPT_COMMAND="hive_auto_context; $PROMPT_COMMAND"
fi

# Helpful environment variables
export HIVE_SHELL_HOOKS_LOADED="true"
export HIVE_AUTO_SUGGEST="true"

# Welcome message (only show once per session)
if [[ -z "$HIVE_HOOKS_WELCOME_SHOWN" ]]; then
    echo "✅ Hive AI shell hooks loaded"
    echo "💡 New aliases: ha, hq, hp, hs, hm, ht, hc, hst, htui"
    echo "🚀 New functions: hive_quick_ask, hive_project_status, hive_context_suggest"
    export HIVE_HOOKS_WELCOME_SHOWN="true"
fi"#.to_string()
    }

    /// Generate Zsh hooks and aliases
    fn generate_zsh_hooks(&self) -> String {
        r#"# Hive AI Shell Hooks
# Added by Hive AI Shell Integration

# Professional aliases
alias ha='hive analyze'
alias hq='hive ask'
alias hp='hive plan'
alias hs='hive search'
alias hm='hive memory search'
alias ht='hive trust check .'
alias hc='hive config show'
alias hst='hive status'
alias htui='hive tui'

# Advanced aliases for power users
alias hive-quick='hive ask --profile=speed'
alias hive-best='hive ask --profile=elite'
alias hive-cheap='hive ask --profile=cost'
alias hive-here='hive analyze . --depth=standard'
alias hive-full='hive analyze . --depth=comprehensive --dependencies --recommendations'
alias hive-check='hive trust check . && hive status'

# Zsh-specific functions with enhanced features
hive_quick_ask() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: hive_quick_ask <question>"
        echo "Example: hive_quick_ask 'How can I optimize this code?'"
        return 1
    fi
    echo "🤖 Quick AI analysis..."
    hive ask "$*" --profile=speed --stream
}

hive_analyze_current() {
    local depth="${1:-standard}"
    echo "🔍 Analyzing current directory with depth: $depth"
    hive analyze . --depth="$depth" --recommendations --format=text
}

hive_plan_feature() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: hive_plan_feature <feature_description>"
        echo "Example: hive_plan_feature 'Add user authentication'"
        return 1
    fi
    echo "📋 Planning feature: $*"
    hive plan "$*" --template=feature --interactive
}

hive_memory_find() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: hive_memory_find <search_term>"
        echo "Example: hive_memory_find 'optimization tips'"
        return 1
    fi
    echo "🧠 Searching memory for: $*"
    hive memory search "$*" --format=table --limit=10
}

hive_smart_cd() {
    cd "$@"
    
    # Auto-detect context and suggest commands
    if command -v hive >/dev/null 2>&1; then
        local suggestions=""
        
        if [[ -f "Cargo.toml" ]]; then
            suggestions="🦀 Rust project | Try: ha . --focus=performance"
        elif [[ -f "package.json" ]]; then
            suggestions="📦 JS/Node project | Try: ha . --focus=security"
        elif [[ -f "requirements.txt" || -f "pyproject.toml" ]]; then
            suggestions="🐍 Python project | Try: ha . --include-tests"
        elif [[ -f "go.mod" ]]; then
            suggestions="🐹 Go project | Try: ha . --focus=efficiency"
        elif [[ -d ".git" ]]; then
            suggestions="📁 Git repository | Try: ha . --depth=standard"
        fi
        
        if [[ -n "$suggestions" ]]; then
            echo "$suggestions"
        fi
    fi
}

# Zsh hooks for enhanced UX
autoload -Uz add-zsh-hook

# Context detection on directory change
_hive_detect_context() {
    # Only show suggestions 10% of the time to avoid spam
    if (( RANDOM % 10 == 0 )) && command -v hive >/dev/null 2>&1; then
        local context=""
        
        [[ -f "Cargo.toml" ]] && context="rust"
        [[ -f "package.json" ]] && context="javascript"
        [[ -f "requirements.txt" || -f "pyproject.toml" ]] && context="python"
        [[ -f "go.mod" ]] && context="go"
        [[ -d ".git" ]] && context="${context:+$context }git"
        
        if [[ -n "$context" ]]; then
            echo "💡 Context: $context | Quick commands: ha, hq, hp"
        fi
    fi
}

add-zsh-hook chpwd _hive_detect_context

# Smart command suggestions
_hive_command_not_found() {
    local command="$1"
    
    case "$command" in
        analyze|analyse)
            echo "Command '$command' not found. Did you mean 'hive analyze' or 'ha'?"
            return 127
            ;;
        ask)
            echo "Command '$command' not found. Did you mean 'hive ask' or 'hq'?"
            return 127
            ;;
        plan)
            echo "Command '$command' not found. Did you mean 'hive plan' or 'hp'?"
            return 127
            ;;
        *)
            return 127
            ;;
    esac
}

# Override cd with smart version
alias cd='hive_smart_cd'

# Helpful environment variables
export HIVE_SHELL_HOOKS_LOADED="true"
export HIVE_AUTO_SUGGEST="true"
export HIVE_ZSH_ENHANCED="true"

# Welcome message (only show once per session)
if [[ -z "$HIVE_HOOKS_WELCOME_SHOWN" ]]; then
    echo "✅ Hive AI Zsh hooks loaded with enhanced features"
    echo "💡 Aliases: ha, hq, hp, hs, hm, ht + smart cd with context detection"
    echo "🚀 Functions: hive_quick_ask, hive_smart_cd, auto context suggestions"
    export HIVE_HOOKS_WELCOME_SHOWN="true"
fi"#.to_string()
    }

    /// Generate Fish hooks and aliases
    fn generate_fish_hooks(&self) -> String {
        r#"# Hive AI Shell Hooks
# Added by Hive AI Shell Integration

# Professional abbreviations (Fish-specific feature)
abbr -a ha 'hive analyze'
abbr -a hq 'hive ask'
abbr -a hp 'hive plan'
abbr -a hs 'hive search'
abbr -a hm 'hive memory search'
abbr -a ht 'hive trust check .'
abbr -a hc 'hive config show'
abbr -a hst 'hive status'
abbr -a htui 'hive tui'

# Advanced abbreviations for power users
abbr -a hive-quick 'hive ask --profile=speed'
abbr -a hive-best 'hive ask --profile=elite'
abbr -a hive-cheap 'hive ask --profile=cost'
abbr -a hive-here 'hive analyze . --depth=standard'
abbr -a hive-full 'hive analyze . --depth=comprehensive --dependencies --recommendations'
abbr -a hive-check 'hive trust check . && hive status'

# Fish-specific functions with rich descriptions
function hive_quick_ask -d "🚀 Quick AI question with speed profile"
    if test (count $argv) -eq 0
        echo "Usage: hive_quick_ask <question>"
        echo "Example: hive_quick_ask 'How can I optimize this code?'"
        return 1
    end
    echo "🤖 Quick AI analysis..."
    hive ask $argv --profile=speed --stream
end

function hive_analyze_current -d "🔍 Analyze current directory with specified depth"
    set -l depth standard
    if test (count $argv) -gt 0
        set depth $argv[1]
    end
    echo "🔍 Analyzing current directory with depth: $depth"
    hive analyze . --depth=$depth --recommendations --format=text
end

function hive_plan_feature -d "📋 Plan a new feature interactively"
    if test (count $argv) -eq 0
        echo "Usage: hive_plan_feature <feature_description>"
        echo "Example: hive_plan_feature 'Add user authentication'"
        return 1
    end
    echo "📋 Planning feature: $argv"
    hive plan $argv --template=feature --interactive
end

function hive_memory_find -d "🧠 Search memory with formatted output"
    if test (count $argv) -eq 0
        echo "Usage: hive_memory_find <search_term>"
        echo "Example: hive_memory_find 'optimization tips'"
        return 1
    end
    echo "🧠 Searching memory for: $argv"
    hive memory search $argv --format=table --limit=10
end

function hive_project_status -d "📊 Show comprehensive project status"
    echo "🔍 Hive AI Project Status"
    echo "========================"
    
    # Check trust status
    echo "Trust Status:"
    if hive trust check . 2>/dev/null
        echo "  ✅ Trusted"
    else
        echo "  ❌ Not trusted"
    end
    
    # Check if we're in a git repo
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1
        echo "Git Repository: ✅"
    else
        echo "Git Repository: ❌"
    end
    
    # Detect project type
    if test -f Cargo.toml
        echo "Project Type: 🦀 Rust"
    else if test -f package.json
        echo "Project Type: 📦 JavaScript/Node.js"
    else if test -f requirements.txt -o -f pyproject.toml
        echo "Project Type: 🐍 Python"
    else if test -f go.mod
        echo "Project Type: 🐹 Go"
    else
        echo "Project Type: ❓ Unknown"
    end
    
    echo ""
    echo "💡 Suggested commands:"
    echo "  ha .          # Quick analyze"
    echo "  hive-here     # Standard analyze"
    echo "  hive-full     # Comprehensive analyze"
end

function hive_context_suggest -d "💡 Get context-aware suggestions"
    if test -f Cargo.toml
        echo "🦀 Rust project detected"
        echo "💡 Try: hive analyze . --focus=performance"
        echo "💡 Ask: 'How can I optimize this Rust code?'"
    else if test -f package.json
        echo "📦 JavaScript/Node.js project detected"
        echo "💡 Try: hive analyze . --focus=security"
        echo "💡 Ask: 'Review this code for security vulnerabilities'"
    else if test -f requirements.txt -o -f pyproject.toml
        echo "🐍 Python project detected"
        echo "💡 Try: hive analyze . --include-tests"
        echo "💡 Ask: 'Suggest Python best practices'"
    else if test -f go.mod
        echo "🐹 Go project detected"
        echo "💡 Try: hive analyze . --focus=efficiency"
        echo "💡 Ask: 'How can I improve Go performance?'"
    else if test -d .git
        echo "📁 Git repository detected"
        echo "💡 Try: hive analyze . --depth=standard"
        echo "💡 Ask: 'What are potential improvements?'"
    else
        echo "📁 Directory detected"
        echo "💡 Try: hive analyze . --depth=quick"
        echo "💡 Ask: 'What does this code do?'"
    end
end

# Fish event handlers for enhanced UX
function __hive_directory_changed --on-variable PWD
    # Show context suggestions 15% of the time
    if test (random 1 7) -eq 1
        and command -v hive >/dev/null 2>&1
        hive_context_suggest
    end
end

# Smart command suggestions for typos
function __hive_command_not_found --on-event fish_command_not_found
    set -l cmd $argv[1]
    
    switch $cmd
        case 'analyse' 'analyze'
            echo "💡 Did you mean 'hive analyze' or 'ha'?"
        case 'ask'
            echo "💡 Did you mean 'hive ask' or 'hq'?"
        case 'plan'
            echo "💡 Did you mean 'hive plan' or 'hp'?"
    end
end

# Set environment variables
set -gx HIVE_SHELL_HOOKS_LOADED "true"
set -gx HIVE_AUTO_SUGGEST "true"
set -gx HIVE_FISH_ENHANCED "true"

# Welcome message (only show once per session)
if not set -q HIVE_HOOKS_WELCOME_SHOWN
    echo "✅ Hive AI Fish hooks loaded with native Fish features"
    echo "💡 Abbreviations: ha, hq, hp, hs, hm, ht + smart context detection"
    echo "🚀 Functions: hive_quick_ask, hive_project_status, hive_context_suggest"
    echo "🐟 Enhanced with Fish events and abbreviations"
    set -gx HIVE_HOOKS_WELCOME_SHOWN "true"
end"#.to_string()
    }

    /// Generate PowerShell hooks and aliases
    fn generate_powershell_hooks(&self) -> String {
        r#"# Hive AI Shell Hooks
# Added by Hive AI Shell Integration

# Professional aliases
Set-Alias -Name ha -Value 'hive analyze' -Description 'Quick analyze command'
Set-Alias -Name hq -Value 'hive ask' -Description 'Quick ask command'
Set-Alias -Name hp -Value 'hive plan' -Description 'Quick plan command'
Set-Alias -Name hs -Value 'hive search' -Description 'Quick search command'
Set-Alias -Name hm -Value 'hive memory search' -Description 'Quick memory search'
Set-Alias -Name ht -Value 'hive trust check .' -Description 'Quick trust check'
Set-Alias -Name hc -Value 'hive config show' -Description 'Quick config show'
Set-Alias -Name hst -Value 'hive status' -Description 'Quick status command'
Set-Alias -Name htui -Value 'hive tui' -Description 'Quick TUI launch'

# PowerShell functions with rich parameter support
function Invoke-HiveQuickAsk {
    <#
    .SYNOPSIS
    Ask Hive AI a quick question with speed profile
    
    .DESCRIPTION
    Wrapper function for hive ask with speed profile and streaming enabled
    
    .PARAMETER Question
    The question to ask Hive AI
    
    .EXAMPLE
    Invoke-HiveQuickAsk "How can I optimize this PowerShell script?"
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$Question
    )
    
    Write-Host "🤖 Quick AI analysis..." -ForegroundColor Cyan
    $questionString = $Question -join ' '
    hive ask $questionString --profile=speed --stream
}

function Invoke-HiveAnalyzeCurrent {
    <#
    .SYNOPSIS
    Analyze the current directory with Hive AI
    
    .DESCRIPTION
    Analyze the current directory with specified depth and generate recommendations
    
    .PARAMETER Depth
    Analysis depth level (quick, standard, comprehensive)
    
    .EXAMPLE
    Invoke-HiveAnalyzeCurrent -Depth standard
    #>
    [CmdletBinding()]
    param(
        [Parameter(Position = 0)]
        [ValidateSet('quick', 'standard', 'comprehensive')]
        [string]$Depth = 'standard'
    )
    
    Write-Host "🔍 Analyzing current directory with depth: $Depth" -ForegroundColor Yellow
    hive analyze . --depth=$Depth --recommendations --format=text
}

function Invoke-HivePlanFeature {
    <#
    .SYNOPSIS
    Plan a new feature with Hive AI
    
    .DESCRIPTION
    Create an interactive plan for a new feature using Hive AI's planning mode
    
    .PARAMETER Description
    Description of the feature to plan
    
    .EXAMPLE
    Invoke-HivePlanFeature "Add user authentication system"
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$Description
    )
    
    $descriptionString = $Description -join ' '
    Write-Host "📋 Planning feature: $descriptionString" -ForegroundColor Green
    hive plan $descriptionString --template=feature --interactive
}

function Search-HiveMemory {
    <#
    .SYNOPSIS
    Search Hive AI memory with formatted output
    
    .DESCRIPTION
    Search conversation history and knowledge base with table formatting
    
    .PARAMETER SearchTerm
    Term to search for in memory
    
    .PARAMETER Limit
    Maximum number of results to return
    
    .EXAMPLE
    Search-HiveMemory "PowerShell optimization" -Limit 5
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, Position = 0, ValueFromRemainingArguments = $true)]
        [string[]]$SearchTerm,
        
        [Parameter()]
        [int]$Limit = 10
    )
    
    $searchString = $SearchTerm -join ' '
    Write-Host "🧠 Searching memory for: $searchString" -ForegroundColor Magenta
    hive memory search $searchString --format=table --limit=$Limit
}

function Get-HiveProjectStatus {
    <#
    .SYNOPSIS
    Show comprehensive project status with Hive AI context
    
    .DESCRIPTION
    Analyze current directory for project type, trust status, and suggest relevant commands
    
    .EXAMPLE
    Get-HiveProjectStatus
    #>
    [CmdletBinding()]
    param()
    
    Write-Host "🔍 Hive AI Project Status" -ForegroundColor Cyan
    Write-Host "========================" -ForegroundColor Cyan
    
    # Check trust status
    Write-Host "Trust Status:" -ForegroundColor Yellow
    try {
        hive trust check . *>$null
        Write-Host "  ✅ Trusted" -ForegroundColor Green
    } catch {
        Write-Host "  ❌ Not trusted" -ForegroundColor Red
    }
    
    # Check if we're in a git repo
    try {
        git rev-parse --is-inside-work-tree *>$null
        Write-Host "Git Repository: ✅" -ForegroundColor Green
    } catch {
        Write-Host "Git Repository: ❌" -ForegroundColor Red
    }
    
    # Detect project type
    if (Test-Path "Cargo.toml") {
        Write-Host "Project Type: 🦀 Rust" -ForegroundColor Yellow
    } elseif (Test-Path "package.json") {
        Write-Host "Project Type: 📦 JavaScript/Node.js" -ForegroundColor Yellow
    } elseif (Test-Path "requirements.txt" -or Test-Path "pyproject.toml") {
        Write-Host "Project Type: 🐍 Python" -ForegroundColor Yellow
    } elseif (Test-Path "go.mod") {
        Write-Host "Project Type: 🐹 Go" -ForegroundColor Yellow
    } elseif (Get-ChildItem -Filter "*.ps1" -ErrorAction SilentlyContinue) {
        Write-Host "Project Type: 💙 PowerShell" -ForegroundColor Blue
    } else {
        Write-Host "Project Type: ❓ Unknown" -ForegroundColor Gray
    }
    
    Write-Host ""
    Write-Host "💡 Suggested commands:" -ForegroundColor Green
    Write-Host "  ha .          # Quick analyze" -ForegroundColor Gray
    Write-Host "  hive-here     # Standard analyze" -ForegroundColor Gray
    Write-Host "  hive-full     # Comprehensive analyze" -ForegroundColor Gray
}

function Get-HiveContextSuggestions {
    <#
    .SYNOPSIS
    Get context-aware suggestions for the current directory
    
    .DESCRIPTION
    Analyze the current directory to detect project type and suggest relevant Hive AI commands
    
    .EXAMPLE
    Get-HiveContextSuggestions
    #>
    [CmdletBinding()]
    param()
    
    if (Test-Path "Cargo.toml") {
        Write-Host "🦀 Rust project detected" -ForegroundColor Yellow
        Write-Host "💡 Try: hive analyze . --focus=performance" -ForegroundColor Cyan
        Write-Host "💡 Ask: 'How can I optimize this Rust code?'" -ForegroundColor Cyan
    } elseif (Test-Path "package.json") {
        Write-Host "📦 JavaScript/Node.js project detected" -ForegroundColor Yellow
        Write-Host "💡 Try: hive analyze . --focus=security" -ForegroundColor Cyan
        Write-Host "💡 Ask: 'Review this code for security vulnerabilities'" -ForegroundColor Cyan
    } elseif (Test-Path "requirements.txt" -or Test-Path "pyproject.toml") {
        Write-Host "🐍 Python project detected" -ForegroundColor Yellow
        Write-Host "💡 Try: hive analyze . --include-tests" -ForegroundColor Cyan
        Write-Host "💡 Ask: 'Suggest Python best practices'" -ForegroundColor Cyan
    } elseif (Test-Path "go.mod") {
        Write-Host "🐹 Go project detected" -ForegroundColor Yellow
        Write-Host "💡 Try: hive analyze . --focus=efficiency" -ForegroundColor Cyan
        Write-Host "💡 Ask: 'How can I improve Go performance?'" -ForegroundColor Cyan
    } elseif (Get-ChildItem -Filter "*.ps1" -ErrorAction SilentlyContinue) {
        Write-Host "💙 PowerShell project detected" -ForegroundColor Blue
        Write-Host "💡 Try: hive analyze . --focus=quality" -ForegroundColor Cyan
        Write-Host "💡 Ask: 'How can I improve this PowerShell code?'" -ForegroundColor Cyan
    } else {
        Write-Host "📁 Directory detected" -ForegroundColor Gray
        Write-Host "💡 Try: hive analyze . --depth=standard" -ForegroundColor Cyan
        Write-Host "💡 Ask: 'What does this code do?'" -ForegroundColor Cyan
    }
}

# Set up function aliases
Set-Alias -Name hive-quick -Value Invoke-HiveQuickAsk
Set-Alias -Name hive-here -Value Invoke-HiveAnalyzeCurrent
Set-Alias -Name hive-plan -Value Invoke-HivePlanFeature
Set-Alias -Name hive-find -Value Search-HiveMemory
Set-Alias -Name hive-status -Value Get-HiveProjectStatus
Set-Alias -Name hive-suggest -Value Get-HiveContextSuggestions

# Environment variables
$env:HIVE_SHELL_HOOKS_LOADED = "true"
$env:HIVE_AUTO_SUGGEST = "true"
$env:HIVE_POWERSHELL_ENHANCED = "true"

# PowerShell prompt customization (optional)
function prompt {
    $originalPrompt = & $function:prompt.OriginalDefinition
    
    # Add Hive context indicator if in a trusted directory
    try {
        hive trust check . *>$null
        $originalPrompt + "🐝 "
    } catch {
        $originalPrompt
    }
}

# Welcome message (only show once per session)
if (-not $env:HIVE_HOOKS_WELCOME_SHOWN) {
    Write-Host "✅ Hive AI PowerShell hooks loaded with advanced features" -ForegroundColor Green
    Write-Host "💡 Aliases: ha, hq, hp, hs, hm, ht + PowerShell-native functions" -ForegroundColor Cyan
    Write-Host "🚀 Functions: Invoke-HiveQuickAsk, Get-HiveProjectStatus, Get-HiveContextSuggestions" -ForegroundColor Magenta
    Write-Host "💙 Enhanced with PowerShell cmdlet support and rich help" -ForegroundColor Blue
    $env:HIVE_HOOKS_WELCOME_SHOWN = "true"
}"#.to_string()
    }

    /// Generate Elvish hooks and aliases
    fn generate_elvish_hooks(&self) -> String {
        r#"# Hive AI Shell Hooks
# Added by Hive AI Shell Integration

# Professional aliases
fn ha { hive analyze $@ }
fn hq { hive ask $@ }
fn hp { hive plan $@ }
fn hs { hive search $@ }
fn hm { hive memory search $@ }
fn ht { hive trust check . }
fn hc { hive config show }
fn hst { hive status }
fn htui { hive tui }

# Advanced aliases
fn hive-quick { hive ask --profile=speed $@ }
fn hive-best { hive ask --profile=elite $@ }
fn hive-cheap { hive ask --profile=cost $@ }
fn hive-here { hive analyze . --depth=standard }
fn hive-full { hive analyze . --depth=comprehensive --dependencies --recommendations }
fn hive-check { hive trust check .; and hive status }

# Elvish-specific functions
fn hive-quick-ask {|@args|
    if (== (count $args) 0) {
        echo "Usage: hive-quick-ask <question>"
        echo "Example: hive-quick-ask 'How can I optimize this code?'"
        return
    }
    echo "🤖 Quick AI analysis..."
    hive ask (str:join ' ' $args) --profile=speed --stream
}

fn hive-analyze-current {|depth|
    if (eq $depth '') {
        set depth = 'standard'
    }
    echo "🔍 Analyzing current directory with depth: "$depth
    hive analyze . --depth=$depth --recommendations --format=text
}

fn hive-context-suggest {
    if (path:is-regular Cargo.toml) {
        echo "🦀 Rust project detected"
        echo "💡 Try: hive analyze . --focus=performance"
    } elif (path:is-regular package.json) {
        echo "📦 JavaScript/Node.js project detected"
        echo "💡 Try: hive analyze . --focus=security"
    } elif (or (path:is-regular requirements.txt) (path:is-regular pyproject.toml)) {
        echo "🐍 Python project detected"
        echo "💡 Try: hive analyze . --include-tests"
    } elif (path:is-regular go.mod) {
        echo "🐹 Go project detected"
        echo "💡 Try: hive analyze . --focus=efficiency"
    } elif (path:is-dir .git) {
        echo "📁 Git repository detected"
        echo "💡 Try: hive analyze . --depth=standard"
    } else {
        echo "📁 Directory detected"
        echo "💡 Try: hive analyze . --depth=quick"
    }
}

# Environment variables
set-env HIVE_SHELL_HOOKS_LOADED true
set-env HIVE_AUTO_SUGGEST true
set-env HIVE_ELVISH_ENHANCED true

# Welcome message
if (not (has-env HIVE_HOOKS_WELCOME_SHOWN)) {
    echo "✅ Hive AI Elvish hooks loaded"
    echo "💡 Functions: ha, hq, hp, hs, hm, ht, hc, hst, htui"
    echo "🚀 Enhanced: hive-quick-ask, hive-context-suggest"
    set-env HIVE_HOOKS_WELCOME_SHOWN true
}"#.to_string()
    }

    /// Remove hooks from shell configuration
    pub fn uninstall(&self, shell: ShellType) -> Result<()> {
        let shell_config = self.get_shell_config_file(shell)?;
        
        if !shell_config.exists() {
            return Ok(());
        }

        // Backup before modification
        utils::backup_file(&shell_config)?;

        let content = read_to_string(&shell_config)?;
        let lines: Vec<&str> = content.lines().collect();
        
        // Remove Hive AI hooks block
        let mut in_hive_block = false;
        let mut filtered_lines = Vec::new();
        
        for line in lines {
            if line.contains("# Hive AI Shell Hooks") {
                in_hive_block = true;
                continue;
            }
            
            if in_hive_block {
                // End of block detection
                if line.trim().is_empty() && 
                   filtered_lines.last().map_or(false, |l: &&str| l.trim().is_empty()) {
                    in_hive_block = false;
                }
                continue;
            }
            
            filtered_lines.push(line);
        }

        // Write cleaned content back
        std::fs::write(&shell_config, filtered_lines.join("\n"))?;
        
        tracing::info!("Removed hooks for {} from {}", shell.as_str(), shell_config.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    #[test]
    fn test_hooks_generation() {
        let config = Config::default();
        let hooks = ShellHooks::new(config);
        
        let bash_hooks = hooks.generate_bash_hooks();
        assert!(bash_hooks.contains("alias ha='hive analyze'"));
        assert!(bash_hooks.contains("hive_quick_ask()"));
        assert!(bash_hooks.contains("export -f"));
        
        let fish_hooks = hooks.generate_fish_hooks();
        assert!(fish_hooks.contains("abbr -a ha"));
        assert!(fish_hooks.contains("function hive_quick_ask"));
        assert!(fish_hooks.contains("set -gx"));
        
        let ps_hooks = hooks.generate_powershell_hooks();
        assert!(ps_hooks.contains("Set-Alias -Name ha"));
        assert!(ps_hooks.contains("function Invoke-HiveQuickAsk"));
        assert!(ps_hooks.contains("[CmdletBinding()]"));
    }

    #[test]
    fn test_shell_config_file_paths() {
        let temp_dir = TempDir::new().unwrap();
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let config = Config::default();
        let hooks = ShellHooks::new(config);
        
        let bash_config = hooks.get_shell_config_file(ShellType::Bash).unwrap();
        assert!(bash_config.ends_with(".bash_profile"));
        
        let zsh_config = hooks.get_shell_config_file(ShellType::Zsh).unwrap();
        assert!(zsh_config.ends_with(".zshrc"));
        
        let fish_config = hooks.get_shell_config_file(ShellType::Fish).unwrap();
        assert!(fish_config.ends_with("config.fish"));
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn test_hook_detection() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("test_config");
        
        // Create a config file with Hive hooks
        std::fs::write(&config_file, "# Hive AI Shell Hooks\nalias ha='hive analyze'").unwrap();
        
        let content = std::fs::read_to_string(&config_file).unwrap();
        assert!(content.contains("# Hive AI Shell Hooks"));
        assert!(content.contains("alias ha="));
    }
}