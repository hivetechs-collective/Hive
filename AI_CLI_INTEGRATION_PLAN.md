# AI CLI Integration Plan

## Current System Analysis

### Existing Database Architecture ✅

sync_metadata table - Perfect foundation already in place:
```sql
CREATE TABLE sync_metadata (
    id TEXT PRIMARY KEY,
    sync_type TEXT NOT NULL,        -- Tool identifier (e.g., "lazygit_update", "claude_cli_update")
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL,           -- "pending", "completed", "failed"
    providers_synced INTEGER DEFAULT 0,
    models_synced INTEGER DEFAULT 0,
    error_message TEXT,
    next_sync_due TEXT NOT NULL,    -- When next check should happen
    rankings_last_synced TEXT,
    provider_performance_synced TEXT,
    intelligence_version TEXT DEFAULT '1.7.0',  -- Tool version installed
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Proven LazyGit Pattern ✅

Location: `/src/desktop/git/lazygit_updater_db.rs`
- 24-hour update cycle: `UPDATE_CHECK_INTERVAL_HOURS = 24`
- Database integration: Uses sync_metadata table with `sync_type = "lazygit_update"`
- Background checking: Triggers during app startup/normal usage
- GitHub API integration: Fetches latest releases automatically
- Cross-platform installation: Handles macOS, Linux, Windows binaries
- Fallback strategy: Uses system LazyGit if available, installs if needed
- Status tracking: Records success/failure, versions, error messages

## Implementation Strategy

### Phase 1: Core AI CLI Manager

Create: `src/desktop/ai_cli_updater.rs`
Pattern: Exact copy of lazygit_updater_db.rs structure

```rust
// New sync_type constants
const SYNC_TYPE_CLAUDE_CLI: &str = "claude_cli_update";
const SYNC_TYPE_GEMINI_CLI: &str = "gemini_cli_update";
const SYNC_TYPE_OPENAI_CLI: &str = "openai_cli_update";
// ... etc for all tools

pub struct AiCliUpdaterDB {
    install_dir: PathBuf,      // ~/.hive/tools/ai_cli/
    db: Arc<DatabaseManager>,  // Same database integration
    tool_configs: HashMap<String, CliToolConfig>,
}

impl AiCliUpdaterDB {
    // Mirror LazyGit's methods:
    pub async fn get_tool_path(&self, tool_id: &str) -> Result<PathBuf>
    pub async fn should_check_for_update(&self, tool_id: &str) -> Result<bool>
    pub async fn update_if_needed(&self, tool_id: &str) -> Result<()>
    pub async fn install_latest(&self, tool_id: &str) -> Result<()>
}
```

### Phase 2: Tool Registry & Definitions

Create: Tool configuration system with dependency management

```rust
#[derive(Debug, Clone)]
pub struct CliToolConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub icon: String,
    pub install_method: InstallMethod,
    pub version_check: String,
    pub install_script: InstallScript,
    pub platforms: Vec<Platform>,
    pub dependencies: Vec<ToolDependency>,
    pub auth_status: AuthStatus,
}

#[derive(Debug, Clone)]
pub struct ToolDependency {
    pub tool: String,
    pub check_command: String,
    pub install_hint: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthStatus {
    NotRequired,
    Configured,
    Required { instructions: String },
    Invalid { error: String },
}

// AI CLI Tools Registry with adjusted priority
pub fn get_ai_tools() -> Vec<CliToolConfig> {
    vec![
        // Tier 1: Primary tool
        CliToolConfig {
            id: "claude".to_string(),
            name: "Claude Code CLI".to_string(),
            command: "claude".to_string(),
            icon: "🤖".to_string(),
            install_method: InstallMethod::Npm,
            version_check: "claude --version".to_string(),
            install_script: InstallScript::Npm("@anthropic/claude-cli".to_string()),
            platforms: vec![Platform::MacOS, Platform::Linux, Platform::Windows],
            dependencies: vec![],
            auth_status: AuthStatus::Required {
                instructions: "Run 'claude login' after installation".to_string()
            },
        },
        
        // Tier 2: Popular tools
        CliToolConfig {
            id: "gh-copilot".to_string(),
            name: "GitHub Copilot CLI".to_string(),
            command: "gh copilot".to_string(),
            icon: "🐙".to_string(),
            install_method: InstallMethod::GhExtension,
            version_check: "gh copilot --version".to_string(),
            install_script: InstallScript::GhExtension("github/gh-copilot".to_string()),
            platforms: vec![Platform::MacOS, Platform::Linux, Platform::Windows],
            dependencies: vec![
                ToolDependency {
                    tool: "gh".to_string(),
                    check_command: "gh --version".to_string(),
                    install_hint: "Install GitHub CLI first: brew install gh".to_string(),
                }
            ],
            auth_status: AuthStatus::Required {
                instructions: "Run 'gh auth login' first".to_string()
            },
        },
        
        CliToolConfig {
            id: "openai".to_string(),
            name: "OpenAI CLI".to_string(),
            command: "openai".to_string(),
            icon: "🌟".to_string(),
            install_method: InstallMethod::Python,
            version_check: "openai --version".to_string(),
            install_script: InstallScript::Pipx("openai-cli".to_string()), // Using pipx for isolation
            platforms: vec![Platform::MacOS, Platform::Linux, Platform::Windows],
            dependencies: vec![],
            auth_status: AuthStatus::Required {
                instructions: "Set OPENAI_API_KEY environment variable".to_string()
            },
        },
        
        // ... additional tools in Tier 3
    ]
}
```

### Phase 3: Terminal Integration with Resource Management

Extend: `src/desktop/terminal_tabs.rs`

```rust
pub struct ResourceLimits {
    pub max_concurrent_tools: usize,  // Default: 3
    pub suspend_after_idle: Duration, // Default: 5 minutes
    pub memory_limit_mb: usize,       // Per tool limit
}

// Add AI tool tabs alongside existing tabs
pub struct TerminalTabs {
    // ... existing fields
    ai_tools: Signal<Vec<AiToolTab>>,
    ai_tools_updater: AiCliUpdaterDB,
    resource_limits: ResourceLimits,
    active_tool_count: Signal<usize>,
}

#[derive(Clone, PartialEq)]
pub struct AiToolTab {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub status: ToolStatus,
    pub terminal_id: Option<String>,
    pub auth_status: AuthStatus,
}

#[derive(Clone, PartialEq)]
pub enum ToolStatus {
    Available,      // Can be installed
    Installing,     // Currently installing
    Ready,         // Installed and ready
    Starting,      // Terminal starting
    Running,       // Terminal active
    Suspended,     // Suspended due to idle
    Error(ToolError), // Installation/run error
}

#[derive(Clone, PartialEq)]
pub enum ToolError {
    InstallationFailed { tool: String, reason: String },
    DependencyMissing { dependency: String },
    NetworkError { retry_after: Duration },
    PermissionDenied { path: PathBuf },
    AuthRequired { instructions: String },
}

impl TerminalTabs {
    fn render_ai_tool_tabs(&self) -> Element {
        rsx! {
            div { class: "ai-tools-section",
                h4 { "AI Tools" }
                for tool in self.ai_tools.read().iter() {
                    button {
                        class: format!("tab ai-tool-tab {}",
                            if tool.status == ToolStatus::Ready { "ready" } else { "pending" }
                        ),
                        onclick: move |_| self.activate_ai_tool(&tool.id),
                        span { class: "icon", "{tool.icon}" }
                        span { class: "name", "{tool.name}" }
                        match &tool.status {
                            ToolStatus::Installing => span { class: "status", "⏳" },
                            ToolStatus::Ready => span { class: "status", "✅" },
                            ToolStatus::Running => span { class: "status", "🟢" },
                            ToolStatus::Suspended => span { class: "status", "⏸️" },
                            ToolStatus::Error(_) => span { class: "status", "❌" },
                            _ => None,
                        }
                        // Show auth status if needed
                        match &tool.auth_status {
                            AuthStatus::Required { .. } => span { class: "auth-indicator", "🔐" },
                            AuthStatus::Invalid { .. } => span { class: "auth-indicator", "⚠️" },
                            _ => None,
                        }
                    }
                }
            }
        }
    }

    async fn activate_ai_tool(&mut self, tool_id: &str) {
        // Check resource limits
        if self.active_tool_count.get() >= self.resource_limits.max_concurrent_tools {
            // Show resource limit message
            return;
        }

        // Check dependencies
        if let Err(e) = self.check_tool_dependencies(tool_id).await {
            // Show dependency error
            return;
        }

        // Lazy load: ensure tool is installed
        if let Err(e) = self.ai_tools_updater.ensure_tool_ready(tool_id).await {
            // Show error status
            return;
        }

        // Check authentication
        if let Err(e) = self.check_tool_auth(tool_id).await {
            // Show auth instructions
            return;
        }

        // Spawn terminal for this tool
        let terminal_id = self.spawn_ai_tool_terminal(tool_id).await;
        self.set_active_terminal(terminal_id);
        self.active_tool_count.update(|c| c + 1);

        // Set up idle monitoring
        self.setup_idle_monitoring(terminal_id);
    }
}
```

### Phase 4: User Experience & Settings

Create: Settings integration for tool management

```rust
// Add to existing settings system
#[derive(Serialize, Deserialize)]
pub struct AiToolsSettings {
    pub enabled_tools: HashSet<String>,
    pub auto_install: bool,
    pub check_updates_startup: bool,
    pub show_installation_progress: bool,
    pub use_local_npm: bool, // For users without global npm permissions
    pub use_pipx: bool,      // For Python tool isolation
}

// Settings UI integration
impl SettingsPanel {
    fn render_ai_tools_section(&self) -> Element {
        rsx! {
            div { class: "settings-section",
                h3 { "AI CLI Tools" }

                div { class: "setting-item",
                    label { "Auto-install tools on first use" }
                    input {
                        r#type: "checkbox",
                        checked: self.ai_tools_settings.auto_install,
                        onchange: |evt| self.update_auto_install(evt.checked())
                    }
                }

                div { class: "setting-item",
                    label { "Use local npm installations (no sudo required)" }
                    input {
                        r#type: "checkbox",
                        checked: self.ai_tools_settings.use_local_npm,
                        onchange: |evt| self.update_use_local_npm(evt.checked())
                    }
                }

                div { class: "setting-item",
                    label { "Use pipx for Python tools (recommended)" }
                    input {
                        r#type: "checkbox",
                        checked: self.ai_tools_settings.use_pipx,
                        onchange: |evt| self.update_use_pipx(evt.checked())
                    }
                }

                div { class: "available-tools",
                    h4 { "Available Tools" }
                    for tool in get_ai_tools() {
                        div { class: "tool-option",
                            input {
                                r#type: "checkbox",
                                checked: self.ai_tools_settings.enabled_tools.contains(&tool.id),
                                onchange: move |evt| self.toggle_tool(&tool.id, evt.checked())
                            }
                            span { class: "tool-icon", "{tool.icon}" }
                            span { class: "tool-name", "{tool.name}" }
                            span { class: "tool-status",
                                match self.get_tool_status(&tool.id) {
                                    ToolStatus::Ready => "✅ Installed",
                                    ToolStatus::Available => "⬇️ Available",
                                    ToolStatus::Installing => "⏳ Installing...",
                                    ToolStatus::Error(e) => format!("❌ {}", e),
                                    _ => "⏸️ Disabled",
                                }
                            }
                            // Show auth status
                            match tool.auth_status {
                                AuthStatus::Required { ref instructions } => {
                                    span { class: "auth-hint", "🔐 {instructions}" }
                                }
                                AuthStatus::Invalid { ref error } => {
                                    span { class: "auth-error", "⚠️ {error}" }
                                }
                                _ => None,
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Phase 5: Installation Automation with Security

Create: Cross-platform installation scripts with verification

```rust
pub enum InstallMethod {
    Npm,           // npm install -g package (or local)
    Python,        // pip install package
    Pipx,          // pipx install package (isolated)
    Cargo,         // cargo install package
    GhExtension,   // gh extension install
    Binary,        // Download and install binary
}

pub enum InstallScript {
    Npm(String),
    Pip(String),
    Pipx(String),
    Cargo(String),
    GhExtension(String),
    Binary { 
        url: String, 
        extract: bool,
        checksum: Option<String>,
        signature: Option<String>,
    },
}

impl AiCliUpdaterDB {
    async fn install_tool(&self, tool: &CliToolConfig) -> Result<()> {
        let install_dir = self.get_tool_install_dir(&tool.id);

        // Check dependencies first
        for dep in &tool.dependencies {
            if !self.check_dependency_installed(dep).await? {
                return Err(ToolError::DependencyMissing { 
                    dependency: dep.tool.clone() 
                });
            }
        }

        match &tool.install_script {
            InstallScript::Npm(package) => {
                if self.settings.use_local_npm {
                    // Install to ~/.hive/tools/node_modules
                    let npm_dir = self.install_dir.join("node_modules");
                    self.run_command(&format!(
                        "cd {} && npm install {}",
                        npm_dir.display(),
                        package
                    )).await?;
                } else {
                    self.run_command(&format!("npm install -g {}", package)).await?;
                }
            }
            InstallScript::Pipx(package) => {
                // Ensure pipx is installed
                self.ensure_pipx_installed().await?;
                self.run_command(&format!("pipx install {}", package)).await?;
            }
            InstallScript::Pip(package) => {
                // Fallback if pipx not available
                self.run_command(&format!("pip install --user {}", package)).await?;
            }
            InstallScript::Cargo(package) => {
                self.run_command(&format!("cargo install {}", package)).await?;
            }
            InstallScript::GhExtension(extension) => {
                // Ensure gh is installed first
                self.ensure_gh_installed().await?;
                self.run_command(&format!("gh extension install {}", extension)).await?;
            }
            InstallScript::Binary { url, extract, checksum, signature } => {
                // Download with verification
                let binary_path = self.download_and_verify_binary(
                    url, 
                    extract, 
                    checksum.as_ref(),
                    signature.as_ref(),
                    &install_dir
                ).await?;
                
                // Make executable
                #[cfg(unix)]
                self.run_command(&format!("chmod +x {}", binary_path.display())).await?;
            }
        }

        // Verify installation
        self.verify_tool_installation(tool).await?;

        // Update database
        self.record_successful_installation(tool).await?;

        Ok(())
    }

    async fn download_and_verify_binary(
        &self,
        url: &str,
        extract: bool,
        checksum: Option<&String>,
        signature: Option<&String>,
        install_dir: &Path,
    ) -> Result<PathBuf> {
        // Download to temp location
        let temp_path = self.download_to_temp(url).await?;

        // Verify checksum if provided
        if let Some(expected_checksum) = checksum {
            let actual_checksum = self.calculate_checksum(&temp_path).await?;
            if actual_checksum != *expected_checksum {
                return Err(ToolError::ChecksumMismatch);
            }
        }

        // Verify signature if provided
        if let Some(sig) = signature {
            self.verify_signature(&temp_path, sig).await?;
        }

        // Extract or move to final location
        let final_path = if extract {
            self.extract_archive(&temp_path, install_dir).await?
        } else {
            let final_path = install_dir.join("binary");
            std::fs::rename(&temp_path, &final_path)?;
            final_path
        };

        Ok(final_path)
    }
}
```

## Complete AI CLI Tools List

### Tier 1: Primary Tool (Auto-install)
1. **Claude Code CLI** - `claude` - NPM - `@anthropic/claude-cli`
2. **LazyGit** - `lazygit` - Binary - Already implemented ✅

### Tier 2: Popular AI Tools (User Choice)
3. **GitHub Copilot CLI** - `gh copilot` - GH Extension - `github/gh-copilot`
4. **OpenAI CLI** - `openai` - Python - `openai-cli`

### Tier 3: Specialized Tools (Optional)
5. **Gemini CLI** - `gemini` - Python - `gemini-cli`
6. **Qwen CLI** - `qwen` - NPM - `@qwenlm/qwen-cli`
7. **Llama CLI** - `llama` - Cargo - `llama-cli`
8. **Mistral CLI** - `mistral` - Python - `mistral-cli`
9. **Cohere CLI** - `cohere` - NPM - `cohere-cli`
10. **Groq CLI** - `groq` - Python - `groq-cli`
11. **Perplexity CLI** - `perplexity` - Cargo - `perplexity-cli`

## Database Integration Details

### Sync Types for AI Tools

```sql
-- Insert new sync_type entries (no schema changes needed!)
INSERT INTO sync_metadata (id, sync_type, started_at, status, next_sync_due) VALUES
('claude_cli_init', 'claude_cli_update', datetime('now'), 'pending', datetime('now', '+1 day')),
('gh_copilot_cli_init', 'gh_copilot_cli_update', datetime('now'), 'pending', datetime('now', '+1 day')),
('openai_cli_init', 'openai_cli_update', datetime('now'), 'pending', datetime('now', '+1 day'));
-- ... etc
```

### Update Query Pattern (Same as LazyGit)

```rust
// Check if update needed (exactly like LazyGit)
let last_check: Option<String> = conn.query_row(
    "SELECT completed_at FROM sync_metadata
     WHERE sync_type = ?1 AND status = 'completed'
     ORDER BY completed_at DESC LIMIT 1",
    params![sync_type],
    |row| row.get(0),
).optional()?;

// Record installation start
conn.execute(
    "INSERT INTO sync_metadata (
        id, sync_type, started_at, status, next_sync_due, created_at
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    params![sync_id, sync_type, now, "pending", next_sync, now],
)?;

// Record success
conn.execute(
    "UPDATE sync_metadata
     SET completed_at = ?1, status = ?2, intelligence_version = ?3
     WHERE id = ?4",
    params![now, "completed", version, sync_id],
)?;
```

## File Structure Plan

### New Files to Create:

```
src/desktop/
├── ai_cli_updater.rs          # Main updater (copy LazyGit pattern)
├── ai_cli_registry.rs         # Tool definitions and configs
└── components/
    ├── ai_tools_settings.rs   # Settings panel integration
    └── tool_status_indicator.rs # Status icons and progress

src/desktop/terminal_tabs.rs   # Extend existing (add AI tool tabs)
```

### Installation Directory Structure:

```
~/.hive/tools/
├── lazygit/           # Existing ✅
│   ├── lazygit
│   └── metadata.json
├── ai_cli/            # New
│   ├── claude/
│   │   ├── installation_info.json
│   │   └── binary_or_link
│   ├── gh-copilot/
│   ├── openai/
│   └── ...
└── node_modules/      # For local npm installs
    └── .bin/
```

## Implementation Checklist

### Phase 1: Foundation (Week 1)
- [ ] Copy lazygit_updater_db.rs to ai_cli_updater.rs
- [ ] Create tool registry with Claude CLI only initially
- [ ] Test sync_metadata integration with new sync_types
- [ ] Implement basic installation for npm method
- [ ] Add dependency checking system
- [ ] Test end-to-end flow with Claude CLI

### Phase 2: Terminal Integration (Week 2)
- [ ] Extend TerminalTabs with AI tool section
- [ ] Add lazy loading for AI tool terminals
- [ ] Implement tool status indicators
- [ ] Add resource management limits
- [ ] Implement idle monitoring and suspension
- [ ] Test terminal spawning for Claude CLI

### Phase 3: User Experience (Week 2-3)
- [ ] Add AI tools section to settings panel
- [ ] Implement enable/disable tool toggles
- [ ] Add installation progress indicators
- [ ] Create tool discovery interface
- [ ] Add authentication status indicators
- [ ] Implement local npm/pipx options

### Phase 4: Expand Tool Support (Week 3)
- [ ] Add GitHub Copilot CLI support
- [ ] Add OpenAI CLI support
- [ ] Implement Python tool handling with pipx
- [ ] Add GH extension installation support
- [ ] Test cross-platform compatibility

### Phase 5: Polish & Testing (Week 4)
- [ ] Add comprehensive error recovery
- [ ] Implement retry logic with backoff
- [ ] Add binary verification (checksums)
- [ ] Test all installation methods
- [ ] Performance optimization
- [ ] Documentation and user guide

### Phase 6: Full Tool Support (Post-launch)
- [ ] Add remaining 7 AI CLI tools
- [ ] Implement tool-specific configurations
- [ ] Add tool usage analytics
- [ ] Create tool recommendation system
- [ ] Add update notifications

## Key Advantages

1. **Zero Schema Changes**: Uses existing sync_metadata table perfectly
2. **Proven Pattern**: Follows successful LazyGit implementation exactly
3. **Gradual Rollout**: Start with Claude CLI, expand based on feedback
4. **User Control**: Tools install only when user enables them
5. **Resource Management**: Prevents system overload with limits
6. **Security First**: Checksum verification, dependency validation
7. **Cross-Platform**: Supports macOS, Linux, Windows consistently
8. **Error Recovery**: Built-in retry and fallback mechanisms
9. **Authentication Aware**: Clear status and setup instructions
10. **Future-Proof**: Easy to add new CLI tools using same pattern

## Risk Mitigation

1. **NPM Permissions**: Local installation option for restricted environments
2. **Python Conflicts**: Pipx isolation prevents version conflicts
3. **Network Issues**: Retry logic and offline detection
4. **Resource Usage**: Automatic suspension and limits
5. **Authentication**: Clear instructions and status indicators

This plan leverages the existing, battle-tested infrastructure while providing a comprehensive AI CLI integration that scales naturally and maintains high performance standards.