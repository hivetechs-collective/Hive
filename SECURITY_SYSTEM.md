# HiveTechs Consensus Security System

## Overview

HiveTechs Consensus implements a comprehensive security warning system that protects users when accessing new repositories or folders, similar to Claude Code's trust model.

## Security Warning Implementation

### Trust Dialog Display

```rust
// src/security/trust_manager.rs

use std::path::PathBuf;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TrustManager {
    trusted_paths: HashSet<PathBuf>,
    trust_cache_file: PathBuf,
}

impl TrustManager {
    pub async fn check_trust(&mut self, path: &PathBuf) -> Result<TrustDecision> {
        // Check if path is already trusted
        if self.is_path_trusted(path) {
            return Ok(TrustDecision::Trusted);
        }
        
        // Check if parent path is trusted
        if let Some(parent) = path.parent() {
            if self.is_path_trusted(&parent.to_path_buf()) {
                return Ok(TrustDecision::Trusted);
            }
        }
        
        // Show trust dialog
        let decision = self.show_trust_dialog(path).await?;
        
        if decision == TrustDecision::Trust {
            self.add_trusted_path(path).await?;
        }
        
        Ok(decision)
    }
    
    async fn show_trust_dialog(&self, path: &PathBuf) -> Result<TrustDecision> {
        use dialoguer::{theme::ColorfulTheme, Select};
        
        // Clear screen for security dialog
        print!("\x1B[2J\x1B[1;1H");
        
        println!("┌──────────────────────────────────────────────────────┐");
        println!("│     Do you trust the files in this folder?          │");
        println!("│                                                      │");
        println!("│ {}                                                   │", 
            format_path_for_display(path));
        println!("│                                                      │");
        println!("│ HiveTechs Consensus may read files in this folder.  │");
        println!("│ Reading untrusted files may lead HiveTechs          │");
        println!("│ Consensus to behave in unexpected ways.             │");
        println!("│                                                      │");
        println!("│ With your permission HiveTechs Consensus may        │");
        println!("│ execute code transformations and analyze files      │");
        println!("│ in this folder. Processing untrusted code is        │");
        println!("│ potentially unsafe.                                  │");
        println!("│                                                      │");
        println!("│ https://docs.hivetechs.com/security/trust           │");
        println!("│                                                      │");
        println!("└──────────────────────────────────────────────────────┘");
        println!();
        
        let items = vec!["Yes, proceed", "No, exit"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()?;
        
        match selection {
            0 => Ok(TrustDecision::Trust),
            _ => Ok(TrustDecision::DontTrust),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TrustDecision {
    Trusted,    // Already trusted
    Trust,      // User chose to trust
    DontTrust,  // User chose not to trust
}
```

### Integration with Main Application

```rust
// src/main.rs - Add trust check before any operations

async fn execute_in_directory(path: &PathBuf) -> Result<()> {
    let mut trust_manager = TrustManager::load().await?;
    
    match trust_manager.check_trust(path).await? {
        TrustDecision::Trusted | TrustDecision::Trust => {
            // Proceed with operation
            println!("✅ Proceeding with trusted directory");
        }
        TrustDecision::DontTrust => {
            println!("❌ Operation cancelled - directory not trusted");
            std::process::exit(1);
        }
    }
    
    Ok(())
}
```

### Trust Persistence

```rust
// src/security/trust_store.rs

impl TrustManager {
    pub async fn load() -> Result<Self> {
        let config_dir = get_hive_config_dir();
        let trust_file = config_dir.join("trusted_paths.json");
        
        if trust_file.exists() {
            let content = tokio::fs::read_to_string(&trust_file).await?;
            let trusted_paths: HashSet<PathBuf> = serde_json::from_str(&content)?;
            
            Ok(Self {
                trusted_paths,
                trust_cache_file: trust_file,
            })
        } else {
            Ok(Self {
                trusted_paths: HashSet::new(),
                trust_cache_file: trust_file,
            })
        }
    }
    
    pub async fn add_trusted_path(&mut self, path: &PathBuf) -> Result<()> {
        self.trusted_paths.insert(path.canonicalize()?);
        self.save().await
    }
    
    pub async fn remove_trusted_path(&mut self, path: &PathBuf) -> Result<()> {
        self.trusted_paths.remove(path);
        self.save().await
    }
    
    async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.trusted_paths)?;
        tokio::fs::write(&self.trust_cache_file, content).await?;
        Ok(())
    }
}
```

### Trust Management Commands

```rust
// Add to CLI commands

#[derive(Subcommand)]
enum TrustCommands {
    /// List all trusted directories
    List,
    
    /// Add a directory to trusted paths
    Add {
        /// Directory path to trust
        path: PathBuf,
    },
    
    /// Remove a directory from trusted paths
    Remove {
        /// Directory path to remove trust
        path: PathBuf,
    },
    
    /// Clear all trusted paths
    Clear {
        /// Confirm clearing all trusted paths
        #[arg(long)]
        confirm: bool,
    },
}

// Implementation
Commands::Trust { command } => {
    let mut trust_manager = TrustManager::load().await?;
    
    match command {
        TrustCommands::List => {
            println!("🔒 Trusted Directories:");
            for path in trust_manager.list_trusted_paths() {
                println!("  ✓ {}", path.display());
            }
        }
        TrustCommands::Add { path } => {
            trust_manager.add_trusted_path(&path).await?;
            println!("✅ Added {} to trusted paths", path.display());
        }
        TrustCommands::Remove { path } => {
            trust_manager.remove_trusted_path(&path).await?;
            println!("✅ Removed {} from trusted paths", path.display());
        }
        TrustCommands::Clear { confirm } => {
            if confirm {
                trust_manager.clear_all().await?;
                println!("✅ Cleared all trusted paths");
            } else {
                println!("❌ Use --confirm to clear all trusted paths");
            }
        }
    }
}
```

## Security Features

### 1. File Access Control

```rust
pub struct SecureFileAccess {
    trust_manager: Arc<TrustManager>,
    sandbox_mode: bool,
}

impl SecureFileAccess {
    pub async fn read_file(&self, path: &Path) -> Result<String> {
        // Verify trust before reading
        if !self.trust_manager.is_path_trusted(path).await? {
            return Err(SecurityError::UntrustedPath(path.to_path_buf()));
        }
        
        // Additional security checks
        self.verify_no_symlink_escape(path)?;
        self.verify_file_permissions(path)?;
        
        // Read with size limits
        let metadata = tokio::fs::metadata(path).await?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(SecurityError::FileTooLarge);
        }
        
        tokio::fs::read_to_string(path).await.map_err(Into::into)
    }
}
```

### 2. Code Execution Sandboxing

```rust
pub struct ExecutionSandbox {
    allowed_commands: HashSet<String>,
    environment_filter: EnvironmentFilter,
}

impl ExecutionSandbox {
    pub async fn execute_command(&self, command: &str) -> Result<Output> {
        // Parse and validate command
        let parsed = self.parse_command(command)?;
        
        if !self.is_command_allowed(&parsed.program) {
            return Err(SecurityError::CommandNotAllowed(parsed.program));
        }
        
        // Execute with restrictions
        Command::new(&parsed.program)
            .args(&parsed.args)
            .env_clear()
            .envs(self.environment_filter.get_safe_env())
            .current_dir(self.get_sandbox_dir()?)
            .output()
            .await
            .map_err(Into::into)
    }
}
```

### 3. Network Request Filtering

```rust
pub struct NetworkSecurityFilter {
    allowed_domains: HashSet<String>,
    blocked_domains: HashSet<String>,
}

impl NetworkSecurityFilter {
    pub fn is_url_allowed(&self, url: &str) -> bool {
        // Always allow OpenRouter API
        if url.starts_with("https://openrouter.ai/") {
            return true;
        }
        
        // Always allow Cloudflare Workers
        if url.contains(".workers.dev") {
            return true;
        }
        
        // Check against allow/block lists
        let domain = extract_domain(url);
        !self.blocked_domains.contains(&domain) && 
        (self.allowed_domains.is_empty() || self.allowed_domains.contains(&domain))
    }
}
```

## Security Best Practices

### For Users

1. **Only trust repositories you own or recognize**
2. **Review code before granting trust**
3. **Regularly audit trusted directories**
4. **Use sandbox mode for untrusted code**
5. **Keep HiveTechs Consensus updated**

### For Developers

1. **Never auto-trust directories**
2. **Implement least-privilege access**
3. **Validate all file paths**
4. **Sanitize user inputs**
5. **Log security-relevant events**

## Configuration

```toml
# ~/.hive/config.toml

[security]
# Enable trust prompts (never disable in production)
trust_prompts_enabled = true

# Sandbox mode for untrusted directories
sandbox_mode = false

# Maximum file size for reading (bytes)
max_file_size = 10485760  # 10MB

# Network security
allowed_domains = []
blocked_domains = ["example.com"]

# Command execution whitelist
allowed_commands = ["cargo", "git", "npm", "hive"]

# Audit logging
security_audit_log = true
audit_log_path = "~/.hive/security_audit.log"
```

## Audit Logging

```rust
pub struct SecurityAuditLogger {
    log_file: PathBuf,
}

impl SecurityAuditLogger {
    pub async fn log_event(&self, event: SecurityEvent) -> Result<()> {
        let entry = AuditLogEntry {
            timestamp: chrono::Utc::now(),
            event_type: event.event_type(),
            path: event.path(),
            user: std::env::var("USER").ok(),
            outcome: event.outcome(),
            details: event.details(),
        };
        
        let json = serde_json::to_string(&entry)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .await?;
            
        file.write_all(format!("{}\n", json).as_bytes()).await?;
        Ok(())
    }
}
```

## Integration with TUI

When in TUI mode, the trust dialog appears as a modal:

```
┌─────────────────────────────────────────────────────┐
│              🔒 Security Warning                    │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Do you trust the files in this folder?            │
│                                                     │
│  /Users/dev/untrusted-repo                         │
│                                                     │
│  HiveTechs Consensus needs permission to:          │
│  • Read and analyze files                          │
│  • Execute code transformations                    │
│  • Access repository metadata                      │
│                                                     │
│  ⚠️  Only trust repositories from known sources    │
│                                                     │
│  Learn more about security:                        │
│  https://docs.hivetechs.com/security/trust         │
│                                                     │
│  [ Trust This Repository ]  [ Cancel ]              │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Error Messages

```rust
pub enum SecurityError {
    UntrustedPath(PathBuf),
    SymlinkEscape,
    InsufficientPermissions,
    CommandNotAllowed(String),
    NetworkRequestBlocked(String),
    FileTooLarge,
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::UntrustedPath(path) => {
                write!(f, "❌ Security Error: Directory '{}' is not trusted.\n\
                          Run 'hive trust add {}' to trust this directory.",
                       path.display(), path.display())
            }
            SecurityError::CommandNotAllowed(cmd) => {
                write!(f, "❌ Security Error: Command '{}' is not allowed.\n\
                          Check security settings in ~/.hive/config.toml", cmd)
            }
            // ... other error messages
        }
    }
}
```

This security system provides comprehensive protection while maintaining usability, similar to Claude Code but tailored for HiveTechs Consensus's specific capabilities and risks.