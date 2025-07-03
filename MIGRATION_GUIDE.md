# HiveTechs Consensus - Migration Guide

> Complete guide for migrating from TypeScript Hive AI to Rust HiveTechs Consensus

## Overview

This guide helps you migrate from the original TypeScript Hive AI to the new Rust-based HiveTechs Consensus. The migration process preserves all your data, conversations, and configuration while providing significant performance improvements.

## Migration Benefits

### Performance Improvements
| Metric | TypeScript | Rust | Improvement |
|--------|------------|------|-------------|
| **Startup Time** | ~2.1s | <50ms | **42x faster** |
| **Memory Usage** | 180MB | <25MB | **7x less** |
| **File Parsing** | 50ms/file | <5ms/file | **10x faster** |
| **Consensus Speed** | 3.2s | <500ms | **6x faster** |
| **Database Queries** | 35ms | <3ms | **12x faster** |

### New Features
- **TUI Interface**: VS Code-like terminal experience
- **Repository Intelligence**: Complete codebase understanding
- **Planning Mode**: AI-powered task decomposition
- **Enhanced Analytics**: Business intelligence and reporting
- **Enterprise Hooks**: Custom workflow automation
- **Security Enhancements**: Advanced trust and audit systems

## Pre-Migration Checklist

### 1. Backup Your Data

```bash
# Backup TypeScript Hive configuration
cp ~/.hive/config.json ~/.hive/config.json.backup

# Backup conversation database
cp ~/.hive/conversations.db ~/.hive/conversations.db.backup

# Backup custom settings
tar -czf hive-typescript-backup.tar.gz ~/.hive/
```

### 2. Document Current Setup

```bash
# Export current configuration
hive config export --format json > current-config.json

# Export conversation list
hive memory list --format json > conversations-list.json

# Export analytics data
hive analytics export --all --format json > analytics-backup.json
```

### 3. Note Custom Integrations

Document any custom integrations you have:
- IDE extensions and configurations
- CI/CD pipeline integrations
- Custom hooks or scripts
- Team configurations
- Enterprise settings

## Installation Process

### 1. Install Rust Version

```bash
# Install HiveTechs Consensus
curl -sSL https://install.hivetechs.com | bash

# Or using package manager
brew install hivetechs/tap/hive  # macOS
sudo apt install hive-consensus  # Ubuntu
```

### 2. Verify Installation

```bash
# Check version
hive --version

# Should show: hive 2.0.0 (rust-edition)

# Test basic functionality
hive health
```

## Data Migration

### Automatic Migration

The Rust version includes an automatic migration tool:

```bash
# Run automatic migration
hive migrate --from-typescript

# This will:
# 1. Detect TypeScript installation
# 2. Convert configuration format
# 3. Migrate conversation database
# 4. Preserve all custom settings
# 5. Verify data integrity
```

### Manual Migration (If Needed)

If automatic migration fails, use manual steps:

#### 1. Configuration Migration

```bash
# Convert TypeScript config to Rust format
hive migrate config \
  --input ~/.hive/config.json \
  --output ~/.hive/config.toml

# Verify configuration
hive config show --validate
```

#### 2. Database Migration

```bash
# Migrate conversation database
hive migrate database \
  --input ~/.hive/conversations.db \
  --output ~/.hive/consensus.db

# Verify conversation count
hive memory list --count
```

#### 3. Custom Settings Migration

```bash
# Migrate custom hooks
hive migrate hooks \
  --input ~/.hive/hooks/ \
  --output ~/.hive/hooks/

# Migrate team settings (if applicable)
hive migrate teams \
  --input ~/.hive/teams.json \
  --output ~/.hive/teams.toml
```

### Migration Verification

```bash
# Comprehensive migration verification
hive migrate verify

# Check specific components
hive migrate verify --conversations
hive migrate verify --configuration
hive migrate verify --analytics
```

## Configuration Changes

### Format Changes

The configuration format has changed from JSON to TOML for better readability:

#### TypeScript (JSON)
```json
{
  "consensus": {
    "profile": "balanced",
    "streaming": true
  },
  "models": {
    "generator": "claude-3-opus",
    "refiner": "gpt-4-turbo"
  },
  "providers": {
    "openrouter": {
      "apiKey": "sk-or-your-key"
    }
  }
}
```

#### Rust (TOML)
```toml
[consensus]
profile = "balanced"
streaming = true

[models]
generator = "anthropic/claude-3-opus"
refiner = "openai/gpt-4-turbo"

[providers.openrouter]
api_key = "sk-or-your-key"
```

### New Configuration Options

The Rust version adds many new configuration options:

```toml
# New performance settings
[performance]
cache_size = "2GB"
max_workers = 8
parallel_processing = true
incremental_parsing = true

# New security settings
[security]
trust_policy = "prompt"
audit_logging = true
sandbox_mode = false

# New TUI settings
[ui.tui]
enabled = true
theme = "dark"
auto_detect_size = true

# New integration settings
[integration]
lsp_port = 7777
mcp_port = 7778
auto_start_servers = true
```

## Command Changes

### Updated Commands

Most commands remain the same, but some have new options:

#### `hive ask` (Enhanced)
```bash
# TypeScript
hive ask "What does this do?"

# Rust (same + new options)
hive ask "What does this do?" --consensus elite --streaming
```

#### `hive analyze` (Enhanced)
```bash
# TypeScript
hive analyze src/

# Rust (same + new options)
hive analyze src/ --architecture --security --metrics
```

### New Commands

The Rust version adds many new commands:

```bash
# Repository intelligence
hive index build
hive index search "function name"

# Planning mode
hive plan "Add authentication"
hive mode planning

# Performance monitoring
hive performance monitor
hive analytics dashboard

# Hook system
hive hooks install pre-commit
hive hooks apply custom-workflow.json

# Security features
hive security scan
hive trust add /path/to/project
```

## IDE Integration Updates

### VS Code / Cursor / Windsurf

1. **Uninstall old extension:**
   - Remove "Hive AI" extension
   
2. **Install new extension:**
   - Install "HiveTechs Consensus" extension
   
3. **Update settings:**
   ```json
   {
     // Old setting
     "hive.apiKey": "removed",
     
     // New settings
     "hive.consensus.profile": "balanced",
     "hive.server.port": 7778,
     "hive.streaming.enabled": true
   }
   ```

### Vim/Neovim

Update your configuration:

```lua
-- Old configuration (remove)
-- require('hive-ai').setup({...})

-- New configuration
require('hive').setup({
  consensus_profile = 'balanced',
  streaming = true,
  auto_apply = false,
  
  -- New TUI integration
  tui_integration = true,
  
  -- New keybindings
  keybindings = {
    ask = '<leader>ha',
    analyze = '<leader>hr',
    plan = '<leader>hp',  -- New
    improve = '<leader>hi',  -- New
  }
})
```

## Team Migration

### Enterprise Features

If you're using team features, migrate team configurations:

```bash
# Export team data from TypeScript
hive-old teams export --format json > teams-backup.json

# Import to Rust version
hive teams import teams-backup.json

# Verify team migration
hive teams list
hive rbac list-permissions
```

### User Management

```bash
# Migrate user accounts
hive migrate users \
  --input ~/.hive/users.json \
  --output ~/.hive/users.toml

# Update role-based access control
hive rbac migrate --from-json ~/.hive/rbac.json
```

## CI/CD Integration Updates

### GitHub Actions

Update your workflow files:

```yaml
# Old workflow
- name: Install Hive AI
  run: npm install -g @hivetechs/hive-ai

# New workflow
- name: Install HiveTechs Consensus
  run: |
    curl -sSL https://install.hivetechs.com | bash
    echo "$HOME/.local/bin" >> $GITHUB_PATH

# Updated analysis step
- name: Analyze Code
  run: |
    hive analyze --format json --output analysis.json
    hive security scan --format sarif --output security.sarif
```

### Pre-commit Hooks

Update pre-commit configuration:

```yaml
# .pre-commit-config.yaml

# Remove old hook
# - repo: https://github.com/hivetechs/hive-ai-pre-commit
#   rev: v1.0.0
#   hooks:
#     - id: hive-ai-check

# Add new hook
repos:
  - repo: local
    hooks:
      - id: hive-consensus-check
        name: HiveTechs Consensus Check
        entry: hive analyze --min-score 85 --exit-code
        language: system
```

## Performance Optimization

### Initial Setup

After migration, optimize for your setup:

```bash
# For large codebases
hive config set performance.cache_size "4GB"
hive config set performance.max_workers 16

# For limited resources
hive config set performance.cache_size "1GB"
hive config set performance.max_workers 4

# Enable all performance features
hive config set performance.parallel_processing true
hive config set performance.incremental_parsing true
```

### Build Initial Index

```bash
# Build comprehensive index for faster responses
hive index build --all-languages --optimize

# Monitor index building
hive index status --verbose
```

## Testing Migration

### Verify Functionality

Test that everything works correctly:

```bash
# Test basic commands
hive ask "What does this project do?"
hive analyze src/ --metrics

# Test conversations
hive memory list
hive memory search "authentication"

# Test new features
hive plan "Add logging"
hive security scan
```

### Performance Validation

Verify performance improvements:

```bash
# Benchmark against TypeScript version
hive benchmark --compare-typescript

# Test startup time
time hive --version

# Test analysis speed
time hive analyze src/
```

### Data Integrity Check

Ensure no data was lost:

```bash
# Compare conversation counts
echo "TypeScript conversations: $(cat conversations-list.json | jq length)"
echo "Rust conversations: $(hive memory list --count)"

# Verify configuration equivalence
hive config validate --compare-with current-config.json
```

## Rollback Procedure

If you need to rollback to TypeScript version:

### 1. Stop Rust Version

```bash
# Stop all Rust services
hive stop --all

# Uninstall Rust version (optional)
sudo rm /usr/local/bin/hive
```

### 2. Restore TypeScript Version

```bash
# Reinstall TypeScript version
npm install -g @hivetechs/hive-ai

# Restore backed up data
cp ~/.hive/config.json.backup ~/.hive/config.json
cp ~/.hive/conversations.db.backup ~/.hive/conversations.db
```

### 3. Verify Rollback

```bash
# Test TypeScript version
hive-ai --version
hive-ai memory list
```

## Post-Migration Tasks

### 1. Clean Up

```bash
# Remove migration artifacts
rm -f current-config.json conversations-list.json
rm -f analytics-backup.json teams-backup.json

# Clean up old cache files (optional)
hive cache clean --old-format
```

### 2. Update Documentation

- Update internal documentation with new commands
- Update team training materials
- Share new features with team members

### 3. Optimize Settings

```bash
# Run optimization wizard
hive optimize --interactive

# Enable new features gradually
hive config set ui.tui.enabled true
hive config set integration.auto_start_servers true
```

## New Features to Explore

After migration, explore these new capabilities:

### 1. TUI Interface

```bash
# Launch TUI mode
hive

# Or force TUI
hive tui --theme dark
```

### 2. Repository Intelligence

```bash
# Deep codebase analysis
hive analyze --architecture
hive index build --full

# Symbol search
hive index search "authentication"
```

### 3. Planning Mode

```bash
# AI-powered planning
hive plan "Implement OAuth integration"
hive mode planning
```

### 4. Enhanced Analytics

```bash
# Business intelligence
hive analytics executive --format pdf
hive analytics cost --by-team
```

### 5. Hook System

```bash
# Install automation hooks
hive hooks install pre-commit
hive hooks apply quality-gate.json
```

## Common Issues

### Migration Fails

**Issue:** Migration tool reports errors

**Solution:**
```bash
# Try manual migration steps
hive migrate --manual --verbose

# Check logs for specific issues
hive logs --filter migration

# Contact support with migration report
hive migrate report --support-bundle
```

### Performance Regression

**Issue:** Rust version seems slower than TypeScript

**Solution:**
```bash
# Run performance diagnostics
hive performance diagnose

# Optimize configuration
hive optimize --auto-tune

# Check resource constraints
hive performance monitor --detailed
```

### Missing Conversations

**Issue:** Some conversations didn't migrate

**Solution:**
```bash
# Run migration verification
hive migrate verify --fix-missing

# Import from backup
hive migrate database --input conversations.db.backup --merge

# Check migration logs
hive logs --filter "conversation.*migration"
```

## Support During Migration

### Self-Service Resources

- **Migration Validation**: `hive migrate verify --comprehensive`
- **Performance Comparison**: `hive benchmark --compare-typescript`
- **Configuration Help**: `hive config validate --help-migrate`

### Community Support

- **GitHub Discussions**: Migration help and tips
- **Discord Channel**: #migration-help
- **Documentation**: https://docs.hivetechs.com/migration

### Enterprise Support

For enterprise customers:
- **Priority Migration Support**: enterprise@hivetechs.com
- **Dedicated Migration Engineer**: Available during migration window
- **Post-Migration Training**: Team training on new features

## Migration Checklist

Use this checklist to track your migration progress:

### Pre-Migration
- [ ] Backup all TypeScript data
- [ ] Document current setup
- [ ] Note custom integrations
- [ ] Schedule migration window
- [ ] Inform team members

### Migration
- [ ] Install Rust version
- [ ] Run migration tool
- [ ] Verify configuration
- [ ] Test basic functionality
- [ ] Validate data integrity

### Post-Migration
- [ ] Update IDE integrations
- [ ] Update CI/CD pipelines
- [ ] Optimize performance settings
- [ ] Train team on new features
- [ ] Clean up old files

### Validation
- [ ] All conversations migrated
- [ ] Configuration working correctly
- [ ] Performance improvements confirmed
- [ ] All integrations functional
- [ ] Team members trained

---

**Congratulations on upgrading to HiveTechs Consensus!** You now have access to the world's most advanced AI development assistant with unparalleled performance and capabilities. 🚀