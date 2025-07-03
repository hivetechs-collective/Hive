# HiveTechs Consensus - Complete User Guide

> Your comprehensive guide to mastering HiveTechs Consensus, the world's most advanced AI development assistant

## Table of Contents

1. [Getting Started](#getting-started)
2. [Core Features](#core-features)
3. [Advanced Usage](#advanced-usage)
4. [Enterprise Features](#enterprise-features)
5. [Troubleshooting](#troubleshooting)

## Getting Started

### First Steps After Installation

#### 1. Initial Setup

```bash
# Run the interactive setup wizard
hive setup

# This will guide you through:
# - API key configuration
# - Consensus profile selection
# - Performance optimization
# - Security settings
```

#### 2. Verify Your Installation

```bash
# Check system health
hive health

# Test API connectivity
hive test --integration

# View current configuration
hive config show
```

#### 3. Your First Query

```bash
# Navigate to a project directory
cd /path/to/your/project

# Ask Hive about your codebase
hive ask "What does this project do?"

# Get code suggestions
hive improve src/main.rs --consensus=balanced
```

### Understanding the 4-Stage Consensus

HiveTechs Consensus uses a revolutionary 4-stage AI pipeline that eliminates hallucinations:

1. **Generator**: Creates initial response using your preferred model
2. **Refiner**: Improves and enhances the initial response
3. **Validator**: Fact-checks and validates the refined response
4. **Curator**: Finalizes and polishes the output

```bash
# Watch the consensus process in real-time
hive ask "Explain this function" --verbose --streaming

# Choose different consensus profiles
hive ask "What's wrong with this code?" --consensus=elite
```

### Consensus Profiles

| Profile | Use Case | Speed | Quality | Cost |
|---------|----------|-------|---------|------|
| **speed** | Quick answers, drafts | Fastest | Good | Lowest |
| **balanced** | General development | Fast | Great | Medium |
| **elite** | Critical code, production | Slower | Excellent | Higher |
| **cost** | Budget-conscious usage | Fast | Good | Lowest |

## Core Features

### 1. AI-Powered Code Analysis

#### Ask Questions About Your Codebase

```bash
# General questions
hive ask "How does authentication work in this app?"
hive ask "What are the main components of this system?"
hive ask "Where should I add logging?"

# Specific file analysis
hive ask "What does this function do?" --file src/auth.rs
hive ask "Are there any security issues in this code?" --file src/api/

# Multi-file analysis
hive ask "How do these components interact?" --files src/auth.rs,src/db.rs,src/api.rs
```

#### Code Quality Analysis

```bash
# Analyze entire project
hive analyze .

# Analyze specific files or directories
hive analyze src/
hive analyze --files "**/*.rs" --exclude "target/"

# Get quality metrics
hive analyze --metrics --output json

# Focus on specific aspects
hive analyze --security  # Security vulnerabilities
hive analyze --performance  # Performance issues
hive analyze --maintainability  # Code maintainability
```

### 2. Intelligent Code Transformation

#### Apply AI Suggestions

```bash
# Get improvement suggestions
hive improve src/main.rs

# Apply suggestions automatically (with confirmation)
hive improve src/main.rs --apply

# Bulk improvements
hive improve src/ --pattern "**/*.rs" --consensus=balanced

# Specific types of improvements
hive improve --focus=performance src/database.rs
hive improve --focus=security src/auth.rs
hive improve --focus=readability src/utils.rs
```

#### Safe Code Application

```bash
# Preview changes before applying
hive apply --preview "Add error handling to this function"

# Apply with backup
hive apply --backup "Refactor this class to use dependency injection"

# Apply with validation
hive apply --validate "Add unit tests for this module"
```

### 3. Project Intelligence & Planning

#### Repository Analysis

```bash
# Deep codebase analysis
hive index .

# Architectural overview
hive analyze --architecture

# Dependency analysis
hive analyze --dependencies

# Technical debt assessment
hive analyze --debt
```

#### AI-Powered Planning

```bash
# Plan a feature implementation
hive plan "Add user authentication with JWT"

# Project refactoring plan
hive plan "Migrate from REST to GraphQL"

# Performance optimization plan
hive plan "Optimize database queries for better performance"

# Enter planning mode for complex tasks
hive mode planning
# Now all queries will include implementation planning
```

### 4. Memory & Conversation Management

#### Conversation History

```bash
# View conversation history
hive memory list

# Search past conversations
hive memory search "authentication implementation"

# Continue previous conversation
hive memory continue <conversation-id>

# Export conversation
hive memory export <conversation-id> --format markdown
```

#### Project Memory

```bash
# View project context
hive memory project

# Add project notes
hive memory add "This service handles user authentication using JWT tokens"

# Update project understanding
hive memory update "The database uses PostgreSQL with connection pooling"
```

### 5. Performance & Analytics

#### Usage Analytics

```bash
# View usage statistics
hive analytics usage

# Cost tracking
hive analytics cost

# Performance metrics
hive analytics performance

# Model efficiency analysis
hive analytics models
```

#### Cost Management

```bash
# Set budget limits
hive cost budget set 50.00  # $50/month

# Get cost estimates
hive cost estimate "Analyze this entire codebase"

# View cost breakdown
hive cost breakdown --last-30-days
```

## Advanced Usage

### 1. Custom Configuration

#### Advanced Configuration

```toml
# ~/.hive/config.toml

[consensus]
profile = "balanced"
streaming = true
temperature = 0.7
max_tokens = 4096

[models]
# Custom model selection for each stage
generator = "anthropic/claude-3-opus"
refiner = "openai/gpt-4-turbo"
validator = "anthropic/claude-3-sonnet"
curator = "meta-llama/llama-3-70b"

[performance]
cache_size = "4GB"
max_workers = 16
parallel_processing = true
incremental_parsing = true

[security]
trust_policy = "prompt"  # always, never, prompt
audit_logging = true
sandbox_mode = true

[analytics]
detailed_tracking = true
anonymous_metrics = false
performance_monitoring = true
```

#### Environment-Specific Profiles

```bash
# Create development profile
hive config profile create development --consensus=speed --models=fast

# Create production profile
hive config profile create production --consensus=elite --audit=true

# Switch profiles
hive config profile use development
```

### 2. Advanced Analysis Features

#### Multi-Language Support

```bash
# Analyze polyglot projects
hive analyze --languages rust,python,javascript

# Language-specific analysis
hive analyze --rust-specific src/
hive analyze --python-specific scripts/
hive analyze --js-specific frontend/
```

#### Integration Analysis

```bash
# API integration analysis
hive analyze --apis

# Database integration analysis
hive analyze --database

# Third-party dependency analysis
hive analyze --dependencies --security-check
```

### 3. Hook System & Automation

#### Pre-defined Hooks

```bash
# Install pre-commit hooks
hive hooks install pre-commit

# Install CI/CD hooks
hive hooks install ci-cd

# Install security hooks
hive hooks install security-audit
```

#### Custom Hooks

```json
// .hive/hooks/quality-gate.json
{
  "name": "quality-gate",
  "trigger": "before-consensus",
  "conditions": {
    "file_types": ["*.rs", "*.py"],
    "min_quality_score": 85
  },
  "actions": [
    {
      "type": "consensus-modification",
      "config": {
        "add_context": "Focus on code quality and best practices",
        "validator_emphasis": "security,performance,maintainability"
      }
    }
  ]
}
```

```bash
# Apply custom hooks
hive hooks apply .hive/hooks/
```

### 4. TUI (Terminal User Interface)

#### Launching TUI Mode

```bash
# Auto-detect and launch TUI (120x30 minimum terminal)
hive

# Force TUI mode
hive tui

# TUI with specific theme
hive tui --theme dark
```

#### TUI Features

- **File Explorer**: Navigate your codebase with Git status indicators
- **Code Editor**: Syntax highlighting and inline AI suggestions
- **Consensus Panel**: Real-time view of 4-stage pipeline progress
- **Integrated Terminal**: Run commands without leaving TUI
- **Analytics Dashboard**: Live performance and usage metrics

#### TUI Keybindings

| Key | Action | Description |
|-----|--------|-------------|
| `Ctrl+P` | Quick Open | Open file by name |
| `F1` | Help | Show help and keybindings |
| `F2` | Analyze | Analyze current file |
| `F3` | Ask | Ask question about code |
| `F4` | Apply | Apply AI suggestions |
| `Ctrl+\`` | Terminal | Toggle integrated terminal |
| `Ctrl+B` | Explorer | Toggle file explorer |
| `Ctrl+J` | Consensus | Toggle consensus panel |
| `Ctrl+Q` | Quit | Exit TUI mode |

### 5. IDE Integration Deep Dive

#### VS Code / Cursor / Windsurf

```json
// settings.json
{
  "hive.consensus.profile": "balanced",
  "hive.streaming.enabled": true,
  "hive.autoApply.confirmationRequired": true,
  "hive.keybindings.ask": "cmd+shift+h",
  "hive.keybindings.apply": "cmd+shift+a",
  "hive.analytics.enabled": true
}
```

Commands available:
- `Hive: Ask Question`
- `Hive: Analyze File`
- `Hive: Apply Suggestions`
- `Hive: Start Planning Mode`
- `Hive: View Analytics`

#### Vim/Neovim Advanced Setup

```lua
-- init.lua
require('hive').setup({
  consensus_profile = 'balanced',
  auto_apply = false,
  streaming = true,
  
  -- Custom keybindings
  keybindings = {
    ask = '<leader>ha',
    analyze = '<leader>hr',
    apply = '<leader>hp',
    planning = '<leader>hP',
    memory = '<leader>hm',
    analytics = '<leader>hA',
  },
  
  -- UI configuration
  ui = {
    floating_windows = true,
    progress_indicator = true,
    syntax_highlighting = true,
  },
  
  -- Integration settings
  integration = {
    treesitter = true,
    lsp = true,
    telescope = true,
  }
})
```

## Enterprise Features

### 1. Role-Based Access Control (RBAC)

#### User Management

```bash
# Create teams
hive teams create engineering --description "Engineering team"
hive teams create security --description "Security team"

# Add users to teams
hive teams add-user engineering alice@company.com
hive teams add-user security bob@company.com

# Assign roles
hive rbac assign-role alice@company.com developer
hive rbac assign-role bob@company.com security-auditor
```

#### Permission Management

```bash
# View permissions
hive rbac list-permissions

# Create custom roles
hive rbac create-role "senior-developer" \
  --permissions read,write,analyze,apply \
  --resources "src/*,tests/*" \
  --exclude "src/security/*"

# Audit permissions
hive rbac audit --user alice@company.com
```

### 2. Enterprise Analytics

#### Team Analytics

```bash
# Team productivity metrics
hive analytics team engineering --metrics productivity,quality,velocity

# Cost allocation by team
hive analytics cost --by-team --period last-quarter

# Usage patterns
hive analytics usage --team-breakdown --export csv
```

#### Executive Dashboard

```bash
# Generate executive report
hive analytics executive --format pdf --period monthly

# ROI analysis
hive analytics roi --baseline typescript-hive --period quarterly

# Compliance reporting
hive analytics compliance --standards sox,iso27001
```

### 3. Audit & Compliance

#### Audit Logging

```bash
# View audit logs
hive audit logs --user alice@company.com --last-7-days

# Compliance reports
hive audit compliance-report --standard sox --format pdf

# Security events
hive audit security --severity high --last-30-days
```

#### Data Governance

```bash
# Data retention settings
hive governance retention set 90  # days

# Data export for compliance
hive governance export --user-data alice@company.com --format json

# Privacy controls
hive governance privacy --anonymize-metrics --opt-out-analytics
```

### 4. Advanced Security

#### Security Policies

```toml
# ~/.hive/security/policy.toml

[access_control]
require_2fa = true
session_timeout = "4h"
max_failed_attempts = 3

[code_scanning]
enabled = true
block_suspicious_patterns = true
quarantine_threshold = "high"

[data_protection]
encrypt_at_rest = true
encrypt_in_transit = true
secure_memory = true

[compliance]
standards = ["sox", "iso27001", "gdpr"]
audit_retention = "7y"
```

#### Security Monitoring

```bash
# Security status
hive security status

# Vulnerability scanning
hive security scan --full-scan

# Threat detection
hive security monitor --real-time
```

## Best Practices

### 1. Effective Prompting

#### Clear and Specific Questions

```bash
# ❌ Vague
hive ask "This code is broken"

# ✅ Specific
hive ask "This function returns a 500 error when the input string contains Unicode characters. How can I fix the encoding issue?"
```

#### Provide Context

```bash
# ❌ No context
hive ask "How do I optimize this?"

# ✅ With context
hive ask "This database query is taking 5+ seconds with 10k records. How can I optimize it for better performance?" --file src/db/queries.rs
```

### 2. Efficient Workflow

#### Use Appropriate Consensus Profiles

- **speed**: Quick iterations, brainstorming, drafts
- **balanced**: Daily development work, code reviews
- **elite**: Production code, critical features, security-sensitive code

#### Leverage Project Memory

```bash
# Document project context as you work
hive memory add "The auth service uses Redis for session storage"
hive memory add "Database migrations are in db/migrations/ and use Diesel"

# This context improves all future responses
```

### 3. Performance Optimization

#### Optimize for Your Workflow

```bash
# For large codebases, increase cache
hive config set performance.cache_size "8GB"

# For faster iteration, enable incremental parsing
hive config set performance.incremental_parsing true

# For parallel processing
hive config set performance.max_workers 16
```

#### Monitor Resource Usage

```bash
# Check resource usage
hive performance monitor

# Optimize based on usage patterns
hive performance optimize --auto-tune
```

## Integration Examples

### 1. CI/CD Integration

#### GitHub Actions

```yaml
# .github/workflows/hive-analysis.yml
name: Hive AI Code Analysis

on: [push, pull_request]

jobs:
  hive-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Hive
        run: |
          curl -sSL https://install.hivetechs.com | bash
          
      - name: Configure Hive
        env:
          OPENROUTER_API_KEY: ${{ secrets.OPENROUTER_API_KEY }}
        run: |
          hive config set openrouter.api_key "$OPENROUTER_API_KEY"
          
      - name: Analyze Code
        run: |
          hive analyze --format json --output analysis.json
          hive security scan --format sarif --output security.sarif
          
      - name: Upload Results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: security.sarif
```

### 2. Pre-commit Hooks

```bash
# Install pre-commit hooks
pip install pre-commit

# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: hive-security-check
        name: Hive Security Check
        entry: hive security scan --exit-code
        language: system
        files: '\.(rs|py|js|ts)$'
        
      - id: hive-quality-gate
        name: Hive Quality Gate
        entry: hive analyze --min-score 85 --exit-code
        language: system
        files: '\.(rs|py|js|ts)$'
```

### 3. Custom Integrations

#### REST API Usage

```python
import requests

# Hive REST API client
class HiveClient:
    def __init__(self, api_key, base_url="http://localhost:7779"):
        self.api_key = api_key
        self.base_url = base_url
        self.headers = {"Authorization": f"Bearer {api_key}"}
    
    def ask(self, question, files=None, consensus="balanced"):
        response = requests.post(
            f"{self.base_url}/api/v1/ask",
            json={
                "question": question,
                "files": files,
                "consensus_profile": consensus
            },
            headers=self.headers
        )
        return response.json()
    
    def analyze(self, path, focus=None):
        response = requests.post(
            f"{self.base_url}/api/v1/analyze",
            json={"path": path, "focus": focus},
            headers=self.headers
        )
        return response.json()

# Usage
client = HiveClient("your-api-key")
result = client.ask("What does this function do?", files=["src/main.rs"])
print(result["response"])
```

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for comprehensive problem-solving guidance.

### Quick Fixes

#### Common Issues

```bash
# Clear cache if experiencing stale results
hive cache clear

# Rebuild index if analysis is incomplete
hive index rebuild

# Reset configuration to defaults
hive config reset

# Test API connectivity
hive test --network --verbose
```

#### Performance Issues

```bash
# Reduce memory usage
hive config set performance.cache_size "1GB"
hive config set performance.max_workers 4

# Enable debug logging
hive config set logging.level debug
```

## Support & Community

- **Documentation**: https://docs.hivetechs.com
- **GitHub**: https://github.com/hivetechs/hive-consensus
- **Discord Community**: https://discord.gg/hivetechs
- **Enterprise Support**: enterprise@hivetechs.com

## What's Next?

- Explore [API_REFERENCE.md](API_REFERENCE.md) for complete command documentation
- Check out [ENTERPRISE_GUIDE.md](ENTERPRISE_GUIDE.md) for team features
- Join our Discord community to share tips and get help
- Follow us for updates on new features and improvements

---

**Happy coding with HiveTechs Consensus!** 🐝✨