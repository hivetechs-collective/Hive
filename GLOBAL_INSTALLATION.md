# Global Installation Strategy: Like Claude Code

## Installation Model

The Rust Hive AI will install **exactly like Claude Code** - globally available in any terminal, IDE, or environment with a single command.

## Installation Methods

### Method 1: Direct Binary Installation (Recommended)
```bash
# Similar to Claude Code installation
curl -fsSL https://hivetechs.com/install.sh | sh

# Or via Homebrew (macOS/Linux)
brew install hivetechs/tap/hive

# Or via npm (global)
npm install -g @hivetechs/hive-ai

# Or via cargo
cargo install hive-ai
```

### Method 2: Platform-Specific Installers
```bash
# macOS
curl -O https://releases.hivetechs.com/hive-macos.pkg
sudo installer -pkg hive-macos.pkg -target /

# Windows
winget install HiveTechs.HiveAI
# or download from https://releases.hivetechs.com/hive-windows.msi

# Linux (various package managers)
# Debian/Ubuntu
wget https://releases.hivetechs.com/hive_2.0.0_amd64.deb
sudo dpkg -i hive_2.0.0_amd64.deb

# RHEL/CentOS/Fedora
sudo rpm -i https://releases.hivetechs.com/hive-2.0.0.x86_64.rpm

# Arch Linux
yay -S hive-ai
```

## Global Binary Placement

### Installation Paths (Same as Claude Code)
```rust
impl GlobalInstaller {
    fn get_install_path() -> PathBuf {
        match std::env::consts::OS {
            "macos" | "linux" => PathBuf::from("/usr/local/bin/hive"),
            "windows" => PathBuf::from("C:\\Program Files\\HiveTechs\\HiveAI\\bin\\hive.exe"),
            _ => panic!("Unsupported platform"),
        }
    }
    
    pub async fn install_globally() -> Result<()> {
        let binary_path = self.get_install_path();
        let current_exe = std::env::current_exe()?;
        
        // Create directory if needed
        if let Some(parent) = binary_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Copy binary to global location
        tokio::fs::copy(current_exe, &binary_path).await?;
        
        // Make executable (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&binary_path).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&binary_path, perms).await?;
        }
        
        // Add to PATH (Windows)
        #[cfg(windows)]
        {
            self.add_to_windows_path(&binary_path).await?;
        }
        
        println!("✅ Hive AI installed globally at: {}", binary_path.display());
        println!("🚀 You can now run 'hive' from anywhere!");
        
        Ok(())
    }
}
```

## Shell Integration

### Shell Completions (Like Claude Code)
```rust
// Generate shell completions during installation
impl ShellIntegration {
    pub fn generate_completions() -> Result<()> {
        let shells = ["bash", "zsh", "fish", "powershell"];
        
        for shell in shells {
            let completion_script = self.generate_completion_script(shell)?;
            self.install_completion_script(shell, &completion_script)?;
        }
        
        Ok(())
    }
    
    fn install_completion_script(&self, shell: &str, script: &str) -> Result<()> {
        let completion_path = match shell {
            "bash" => {
                // macOS
                if cfg!(target_os = "macos") {
                    "/usr/local/etc/bash_completion.d/hive"
                } else {
                    "/etc/bash_completion.d/hive"
                }
            }
            "zsh" => {
                // Install to user's zsh completions
                let home = std::env::var("HOME")?;
                &format!("{}/.zsh/completions/_hive", home)
            }
            "fish" => {
                let home = std::env::var("HOME")?;
                &format!("{}/.config/fish/completions/hive.fish", home)
            }
            "powershell" => {
                // Windows PowerShell profile
                "%USERPROFILE%\\Documents\\PowerShell\\Microsoft.PowerShell_profile.ps1"
            }
            _ => return Err(anyhow!("Unsupported shell: {}", shell)),
        };
        
        std::fs::write(completion_path, script)?;
        Ok(())
    }
}
```

### Example Shell Completions
```bash
# Bash completion for hive
_hive_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    opts="init analyze ask plan execute improve analytics memory tool serve index config interactive --help --version"
    
    case "${prev}" in
        analyze)
            COMPREPLY=( $(compgen -d -- ${cur}) )
            return 0
            ;;
        --format)
            COMPREPLY=( $(compgen -W "text json html markdown" -- ${cur}) )
            return 0
            ;;
        --profile)
            COMPREPLY=( $(compgen -W "speed balanced cost elite" -- ${cur}) )
            return 0
            ;;
        *)
            ;;
    esac
    
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    return 0
}

complete -F _hive_completion hive
```

## Configuration Management

### Global Configuration Directory
```rust
impl ConfigManager {
    pub fn get_global_config_dir() -> PathBuf {
        match std::env::consts::OS {
            "macos" => {
                let home = std::env::var("HOME").unwrap();
                PathBuf::from(format!("{}/.hive", home))
            }
            "linux" => {
                if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
                    PathBuf::from(format!("{}/hive", xdg_config))
                } else {
                    let home = std::env::var("HOME").unwrap();
                    PathBuf::from(format!("{}/.config/hive", home))
                }
            }
            "windows" => {
                let appdata = std::env::var("APPDATA").unwrap();
                PathBuf::from(format!("{}\\HiveTechs\\HiveAI", appdata))
            }
            _ => panic!("Unsupported platform"),
        }
    }
    
    pub async fn initialize_global_config() -> Result<()> {
        let config_dir = self.get_global_config_dir();
        tokio::fs::create_dir_all(&config_dir).await?;
        
        // Create default config if not exists
        let config_file = config_dir.join("config.toml");
        if !config_file.exists() {
            let default_config = include_str!("../templates/default_config.toml");
            tokio::fs::write(config_file, default_config).await?;
        }
        
        // Initialize database
        let db_file = config_dir.join("conversations.db");
        if !db_file.exists() {
            Database::initialize(&db_file).await?;
        }
        
        println!("✅ Global configuration initialized at: {}", config_dir.display());
        Ok(())
    }
}
```

## IDE Integration Paths

### VS Code Extension Discovery
```json
{
  "name": "hive-ai",
  "displayName": "Hive AI",
  "description": "AI-powered codebase intelligence",
  "version": "2.0.0",
  "engines": {
    "vscode": "^1.80.0"
  },
  "activationEvents": [
    "onStartupFinished"
  ],
  "main": "./out/extension.js",
  "contributes": {
    "configuration": {
      "title": "Hive AI",
      "properties": {
        "hive.binaryPath": {
          "type": "string",
          "default": "hive",
          "description": "Path to Hive AI binary (auto-detected if globally installed)"
        }
      }
    },
    "commands": [
      {
        "command": "hive.analyze",
        "title": "Analyze Repository",
        "category": "Hive AI"
      },
      {
        "command": "hive.ask",
        "title": "Ask Question",
        "category": "Hive AI"
      },
      {
        "command": "hive.plan",
        "title": "Create Plan",
        "category": "Hive AI"
      }
    ]
  }
}
```

### Extension Binary Detection
```typescript
// VS Code extension automatically finds global binary
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export class HiveBinaryManager {
    private binaryPath: string | null = null;
    
    async findHiveBinary(): Promise<string> {
        if (this.binaryPath) {
            return this.binaryPath;
        }
        
        // Check if globally installed
        try {
            const { stdout } = await execAsync('which hive');
            this.binaryPath = stdout.trim();
            return this.binaryPath;
        } catch {
            // Try Windows
            try {
                const { stdout } = await execAsync('where hive');
                this.binaryPath = stdout.trim().split('\n')[0];
                return this.binaryPath;
            } catch {
                throw new Error('Hive AI binary not found. Please install globally with: npm install -g @hivetechs/hive-ai');
            }
        }
    }
    
    async executeHive(args: string[]): Promise<string> {
        const binary = await this.findHiveBinary();
        const { stdout } = await execAsync(`${binary} ${args.join(' ')}`);
        return stdout;
    }
}
```

## Auto-Update Mechanism

### Self-Updating Binary (Like Claude Code)
```rust
impl AutoUpdater {
    pub async fn check_for_updates() -> Result<Option<UpdateInfo>> {
        let current_version = env!("CARGO_PKG_VERSION");
        let client = reqwest::Client::new();
        
        let response = client
            .get("https://api.hivetechs.com/releases/latest")
            .header("User-Agent", format!("hive-ai/{}", current_version))
            .send()
            .await?;
            
        let latest: ReleaseInfo = response.json().await?;
        
        if semver::Version::parse(&latest.version)? > semver::Version::parse(current_version)? {
            Ok(Some(UpdateInfo {
                version: latest.version,
                download_url: latest.download_url,
                changelog: latest.changelog,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn self_update() -> Result<()> {
        if let Some(update) = self.check_for_updates().await? {
            println!("🔄 Updating Hive AI to version {}...", update.version);
            
            // Download new binary
            let response = reqwest::get(&update.download_url).await?;
            let bytes = response.bytes().await?;
            
            // Replace current binary
            let current_exe = std::env::current_exe()?;
            let temp_path = current_exe.with_extension("new");
            
            tokio::fs::write(&temp_path, bytes).await?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = tokio::fs::metadata(&temp_path).await?.permissions();
                perms.set_mode(0o755);
                tokio::fs::set_permissions(&temp_path, perms).await?;
            }
            
            // Atomic replace
            tokio::fs::rename(temp_path, current_exe).await?;
            
            println!("✅ Updated to version {}!", update.version);
            println!("📝 Changelog: {}", update.changelog);
        } else {
            println!("✅ Hive AI is up to date!");
        }
        
        Ok(())
    }
}
```

## Installation Script

### Universal Install Script (Like Claude Code)
```bash
#!/bin/bash
# install.sh - Universal Hive AI installer

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect platform
detect_platform() {
    local platform=""
    local arch=""
    
    case "$OSTYPE" in
        darwin*)
            platform="darwin"
            ;;
        linux*)
            platform="linux"
            ;;
        msys*|cygwin*|win*)
            platform="windows"
            ;;
        *)
            echo -e "${RED}Error: Unsupported platform $OSTYPE${NC}"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture $(uname -m)${NC}"
            exit 1
            ;;
    esac
    
    echo "${platform}-${arch}"
}

# Download and install
install_hive() {
    local platform_arch=$(detect_platform)
    local version="latest"
    local base_url="https://releases.hivetechs.com"
    
    echo -e "${YELLOW}🐝 Installing Hive AI...${NC}"
    echo "Platform: $platform_arch"
    
    # Determine binary name and install path
    local binary_name="hive"
    local install_path="/usr/local/bin"
    
    if [[ "$platform_arch" == *"windows"* ]]; then
        binary_name="hive.exe"
        install_path="$APPDATA/HiveTechs/HiveAI/bin"
        mkdir -p "$install_path"
    fi
    
    # Download binary
    local download_url="${base_url}/${version}/hive-${platform_arch}"
    local temp_file="/tmp/hive-${platform_arch}"
    
    echo "Downloading from: $download_url"
    curl -fsSL "$download_url" -o "$temp_file"
    
    # Install binary
    if [[ "$platform_arch" != *"windows"* ]]; then
        sudo mv "$temp_file" "$install_path/$binary_name"
        sudo chmod +x "$install_path/$binary_name"
    else
        mv "$temp_file" "$install_path/$binary_name"
    fi
    
    echo -e "${GREEN}✅ Hive AI installed successfully!${NC}"
    
    # Initialize configuration
    "$install_path/$binary_name" init --global
    
    # Generate shell completions
    "$install_path/$binary_name" completion bash > /tmp/hive_completion
    if [ -w "/etc/bash_completion.d" ]; then
        sudo mv /tmp/hive_completion /etc/bash_completion.d/hive
    fi
    
    echo -e "${GREEN}🚀 You can now run 'hive' from anywhere!${NC}"
    echo -e "${YELLOW}💡 Try: hive --help${NC}"
    echo -e "${YELLOW}💡 Or: hive analyze .${NC}"
}

# Run installation
install_hive
```

## Verification Commands

### Post-Installation Verification
```bash
# Verify global installation
which hive
# Output: /usr/local/bin/hive

hive --version
# Output: hive 2.0.0

hive status
# Output: ✅ Hive AI ready (config: ~/.hive/config.toml)

# Test basic functionality
hive ask "What is the weather like?"
# Output: 🤔 Processing your question...

# Test repository analysis
cd /path/to/any/project
hive analyze .
# Output: 🔍 Analyzing repository...
```

## Summary

The Rust Hive AI will install **exactly like Claude Code**:

- ✅ **Global binary** available in any terminal (`/usr/local/bin/hive`)
- ✅ **Shell completions** for bash, zsh, fish, PowerShell
- ✅ **Auto-discovery** by IDEs and extensions
- ✅ **Self-updating** mechanism with `hive update`
- ✅ **Cross-platform** installers (brew, winget, apt, rpm)
- ✅ **Universal install script** with `curl | sh`
- ✅ **Global configuration** in `~/.hive/`

Once installed, developers can use `hive` from anywhere - terminal, IDE, or any development environment - just like Claude Code provides universal availability.