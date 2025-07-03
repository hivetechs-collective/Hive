# HiveTechs Consensus - API Reference

> Complete reference for all CLI commands, configuration options, and integration APIs

## Table of Contents

1. [CLI Commands](#cli-commands)
2. [Configuration Reference](#configuration-reference)
3. [Hook System API](#hook-system-api)
4. [MCP Tools Reference](#mcp-tools-reference)
5. [REST API Reference](#rest-api-reference)
6. [LSP Features](#lsp-features)

## CLI Commands

### Core Commands

#### `hive ask`
Ask questions about your codebase using AI consensus.

```bash
hive ask [OPTIONS] <QUESTION>
```

**Arguments:**
- `<QUESTION>` - The question to ask about your codebase

**Options:**
- `-f, --file <FILE>` - Analyze specific file(s)
- `-d, --directory <DIR>` - Analyze specific directory
- `--files <PATTERN>` - Glob pattern for files to include
- `--exclude <PATTERN>` - Glob pattern for files to exclude
- `-c, --consensus <PROFILE>` - Consensus profile (speed|balanced|elite|cost)
- `--streaming` - Enable streaming responses (default: true)
- `--no-streaming` - Disable streaming responses
- `-o, --output <FORMAT>` - Output format (text|json|markdown)
- `--save` - Save conversation to memory
- `--continue <ID>` - Continue previous conversation
- `-v, --verbose` - Show detailed consensus process

**Examples:**
```bash
# Basic question
hive ask "What does this project do?"

# Analyze specific file
hive ask "Are there any security issues?" --file src/auth.rs

# Multiple files with pattern
hive ask "How do these components interact?" --files "src/**/*.rs"

# Use elite consensus for critical analysis
hive ask "Is this production-ready?" --consensus elite

# Save important analysis
hive ask "Document the API endpoints" --save --output markdown
```

#### `hive analyze`
Perform deep codebase analysis and generate insights.

```bash
hive analyze [OPTIONS] [PATH]
```

**Arguments:**
- `[PATH]` - Path to analyze (default: current directory)

**Options:**
- `--architecture` - Analyze system architecture
- `--dependencies` - Analyze dependencies and relationships
- `--security` - Focus on security vulnerabilities
- `--performance` - Focus on performance issues
- `--maintainability` - Focus on code maintainability
- `--debt` - Analyze technical debt
- `--metrics` - Generate quality metrics
- `--languages <LANGS>` - Specific languages to analyze
- `--min-score <SCORE>` - Minimum quality score (0-100)
- `--format <FORMAT>` - Output format (text|json|sarif|html)
- `--output <FILE>` - Save output to file
- `--exclude <PATTERN>` - Exclude files/directories
- `--include <PATTERN>` - Include only specific files
- `--exit-code` - Exit with non-zero code on issues

**Examples:**
```bash
# Analyze entire project
hive analyze

# Security-focused analysis
hive analyze --security --format sarif --output security.sarif

# Architecture overview
hive analyze --architecture --output architecture.md

# Quality gate for CI/CD
hive analyze --min-score 85 --exit-code

# Multi-language project
hive analyze --languages rust,python,javascript
```

#### `hive improve`
Generate and apply AI-powered code improvements.

```bash
hive improve [OPTIONS] <PATH>
```

**Arguments:**
- `<PATH>` - File or directory to improve

**Options:**
- `--focus <ASPECT>` - Focus area (performance|security|readability|maintainability)
- `--apply` - Apply improvements automatically
- `--preview` - Show preview of changes
- `--backup` - Create backup before applying
- `--consensus <PROFILE>` - Consensus profile to use
- `--format <FORMAT>` - Output format for suggestions
- `--interactive` - Interactive mode for review
- `--pattern <PATTERN>` - File pattern to include
- `--exclude <PATTERN>` - Files to exclude

**Examples:**
```bash
# Get improvement suggestions
hive improve src/main.rs

# Apply performance improvements
hive improve src/ --focus performance --apply --backup

# Interactive review mode
hive improve src/auth.rs --interactive --preview
```

#### `hive plan`
Create AI-powered implementation plans for features and tasks.

```bash
hive plan [OPTIONS] <DESCRIPTION>
```

**Arguments:**
- `<DESCRIPTION>` - Description of what to plan

**Options:**
- `--scope <SCOPE>` - Planning scope (file|module|project|feature)
- `--timeline` - Include timeline estimates
- `--dependencies` - Analyze dependencies
- `--risks` - Include risk analysis
- `--format <FORMAT>` - Output format (text|markdown|json)
- `--save` - Save plan to memory
- `--interactive` - Interactive planning mode

**Examples:**
```bash
# Plan a new feature
hive plan "Add user authentication with JWT tokens"

# Comprehensive project planning
hive plan "Migrate to microservices architecture" --timeline --dependencies --risks

# Quick task planning
hive plan "Add logging to database operations" --scope module
```

### Memory & Analytics Commands

#### `hive memory`
Manage conversation history and project memory.

```bash
hive memory <SUBCOMMAND>
```

**Subcommands:**

##### `hive memory list`
List conversation history.
```bash
hive memory list [OPTIONS]

Options:
  --limit <N>          Number of conversations to show (default: 20)
  --search <QUERY>     Search conversations
  --format <FORMAT>    Output format (table|json)
  --project            Show only current project conversations
```

##### `hive memory search`
Search conversation history.
```bash
hive memory search <QUERY> [OPTIONS]

Arguments:
  <QUERY>              Search query

Options:
  --limit <N>          Number of results (default: 10)
  --format <FORMAT>    Output format (table|json|markdown)
  --context            Include conversation context
```

##### `hive memory continue`
Continue a previous conversation.
```bash
hive memory continue <CONVERSATION_ID>

Arguments:
  <CONVERSATION_ID>    ID of conversation to continue
```

##### `hive memory export`
Export conversation data.
```bash
hive memory export <CONVERSATION_ID> [OPTIONS]

Arguments:
  <CONVERSATION_ID>    Conversation to export (or "all")

Options:
  --format <FORMAT>    Export format (markdown|json|html)
  --output <FILE>      Output file
  --include-metadata   Include metadata
```

##### `hive memory add`
Add project context note.
```bash
hive memory add <NOTE>

Arguments:
  <NOTE>              Context note to add
```

#### `hive analytics`
View usage analytics and performance metrics.

```bash
hive analytics <SUBCOMMAND>
```

**Subcommands:**

##### `hive analytics usage`
Show usage statistics.
```bash
hive analytics usage [OPTIONS]

Options:
  --period <PERIOD>    Time period (day|week|month|quarter|year)
  --format <FORMAT>    Output format (table|json|chart)
  --team <TEAM>        Filter by team (enterprise)
  --export <FILE>      Export to file
```

##### `hive analytics cost`
Show cost analysis.
```bash
hive analytics cost [OPTIONS]

Options:
  --period <PERIOD>    Time period
  --breakdown          Show cost breakdown by model/team
  --budget             Show budget status
  --forecast           Show cost forecast
```

##### `hive analytics performance`
Show performance metrics.
```bash
hive analytics performance [OPTIONS]

Options:
  --metrics <METRICS>  Specific metrics (latency|throughput|quality)
  --compare <PERIOD>   Compare with previous period
  --trends             Show trend analysis
```

### Configuration Commands

#### `hive config`
Manage configuration settings.

```bash
hive config <SUBCOMMAND>
```

**Subcommands:**

##### `hive config show`
Display current configuration.
```bash
hive config show [OPTIONS]

Options:
  --format <FORMAT>    Output format (toml|json|yaml)
  --section <SECTION>  Show specific section
  --redact-secrets     Hide sensitive values
```

##### `hive config set`
Set configuration value.
```bash
hive config set <KEY> <VALUE>

Arguments:
  <KEY>               Configuration key (dot notation)
  <VALUE>             Configuration value

Examples:
  hive config set consensus.profile balanced
  hive config set openrouter.api_key "sk-or-your-key"
  hive config set performance.cache_size "4GB"
```

##### `hive config get`
Get configuration value.
```bash
hive config get <KEY>

Arguments:
  <KEY>               Configuration key
```

##### `hive config reset`
Reset configuration to defaults.
```bash
hive config reset [OPTIONS]

Options:
  --section <SECTION>  Reset specific section
  --confirm            Skip confirmation prompt
```

### Advanced Commands

#### `hive index`
Manage codebase indexing and symbols.

```bash
hive index <SUBCOMMAND>
```

**Subcommands:**

##### `hive index build`
Build or rebuild symbol index.
```bash
hive index build [OPTIONS] [PATH]

Arguments:
  [PATH]              Path to index (default: current directory)

Options:
  --force             Force rebuild existing index
  --languages <LANGS> Languages to index
  --exclude <PATTERN> Exclude patterns
  --threads <N>       Number of worker threads
```

##### `hive index status`
Show indexing status.
```bash
hive index status [OPTIONS]

Options:
  --verbose           Show detailed status
  --statistics        Show index statistics
```

##### `hive index search`
Search symbol index.
```bash
hive index search <QUERY> [OPTIONS]

Arguments:
  <QUERY>             Search query

Options:
  --type <TYPE>       Symbol type (function|class|variable|etc)
  --language <LANG>   Filter by language
  --limit <N>         Maximum results
```

#### `hive hooks`
Manage automation hooks and workflows.

```bash
hive hooks <SUBCOMMAND>
```

**Subcommands:**

##### `hive hooks list`
List installed hooks.
```bash
hive hooks list [OPTIONS]

Options:
  --active            Show only active hooks
  --format <FORMAT>   Output format (table|json)
```

##### `hive hooks install`
Install predefined hook templates.
```bash
hive hooks install <TEMPLATE> [OPTIONS]

Arguments:
  <TEMPLATE>          Hook template (pre-commit|ci-cd|security-audit)

Options:
  --config <FILE>     Custom configuration file
  --dry-run           Show what would be installed
```

##### `hive hooks apply`
Apply custom hook configuration.
```bash
hive hooks apply <CONFIG> [OPTIONS]

Arguments:
  <CONFIG>            Hook configuration file or directory

Options:
  --validate          Validate configuration only
  --force             Override existing hooks
```

#### `hive security`
Security scanning and monitoring.

```bash
hive security <SUBCOMMAND>
```

**Subcommands:**

##### `hive security scan`
Perform security vulnerability scan.
```bash
hive security scan [OPTIONS] [PATH]

Arguments:
  [PATH]              Path to scan (default: current directory)

Options:
  --severity <LEVEL>  Minimum severity (low|medium|high|critical)
  --format <FORMAT>   Output format (text|json|sarif)
  --output <FILE>     Save results to file
  --exit-code         Exit with error code on findings
  --fix               Apply automatic fixes where possible
```

##### `hive security status`
Show security status overview.
```bash
hive security status [OPTIONS]

Options:
  --verbose           Show detailed status
  --compliance        Show compliance status
```

### Utility Commands

#### `hive health`
Check system health and connectivity.

```bash
hive health [OPTIONS]

Options:
  --verbose           Show detailed health information
  --check-apis        Test API connectivity
  --check-models      Verify model availability
  --format <FORMAT>   Output format (text|json)
```

#### `hive test`
Run integration tests and validation.

```bash
hive test [OPTIONS]

Options:
  --integration       Run integration tests
  --network           Test network connectivity
  --performance       Run performance tests
  --verbose           Show detailed output
  --timeout <SECONDS> Test timeout (default: 30)
```

#### `hive version`
Show version information.

```bash
hive version [OPTIONS]

Options:
  --verbose           Show detailed version info
  --check-updates     Check for available updates
  --format <FORMAT>   Output format (text|json)
```

## Configuration Reference

### Main Configuration File

Location: `~/.hive/config.toml`

```toml
[consensus]
# Consensus profile (speed|balanced|elite|cost)
profile = "balanced"

# Enable streaming responses
streaming = true

# Model temperature (0.0-2.0)
temperature = 0.7

# Maximum tokens per response
max_tokens = 4096

# Retry configuration
max_retries = 3
retry_delay = "1s"

[models]
# Model selection for each consensus stage
generator = "anthropic/claude-3-opus"
refiner = "openai/gpt-4-turbo"
validator = "anthropic/claude-3-sonnet"
curator = "openai/gpt-4"

# Fallback models
fallback_generator = "openai/gpt-4-turbo"
fallback_refiner = "anthropic/claude-3-sonnet"

[providers]
# OpenRouter configuration (required)
[providers.openrouter]
api_key = "sk-or-your-key"
base_url = "https://openrouter.ai/api/v1"
timeout = "60s"

# Direct provider access (optional)
[providers.anthropic]
api_key = ""
base_url = "https://api.anthropic.com"

[providers.openai]
api_key = ""
base_url = "https://api.openai.com/v1"

[providers.google]
api_key = ""
base_url = "https://generativelanguage.googleapis.com"

[performance]
# Cache configuration
cache_size = "2GB"
cache_ttl = "24h"

# Worker configuration
max_workers = 8
worker_timeout = "30s"

# Processing configuration
parallel_processing = true
incremental_parsing = true
memory_limit = "4GB"

# Network configuration
request_timeout = "30s"
connection_pool_size = 10

[security]
# Trust policy for new directories (always|never|prompt)
trust_policy = "prompt"

# Enable audit logging
audit_logging = true

# Sandbox mode for code execution
sandbox_mode = false

# API key encryption
encrypt_keys = true

# Session configuration
session_timeout = "4h"
require_2fa = false

[integration]
# LSP server configuration
lsp_port = 7777
lsp_enabled = true

# MCP server configuration
mcp_port = 7778
mcp_enabled = true

# REST API configuration
rest_api_port = 7779
rest_api_enabled = false

# Auto-start servers
auto_start_servers = true

[analytics]
# Enable analytics collection
enabled = true

# Anonymous usage metrics
anonymous_metrics = true

# Performance tracking
performance_tracking = true

# Detailed event tracking
detailed_tracking = false

# Cost tracking
cost_tracking = true

[logging]
# Log level (debug|info|warn|error)
level = "info"

# Log file location
file = "~/.hive/logs/hive.log"

# Log rotation
max_size = "100MB"
max_files = 5

# Structured logging
structured = false

[ui]
# Default output format
default_format = "text"

# Color output
color = "auto"  # always|never|auto

# Progress indicators
progress_indicators = true

# TUI configuration
[ui.tui]
enabled = true
theme = "dark"  # dark|light|solarized
auto_detect_size = true
min_width = 120
min_height = 30

[memory]
# Conversation retention
retention_days = 90

# Project memory size
project_memory_size = "100MB"

# Enable conversation clustering
enable_clustering = true

# Search index size
search_index_size = "50MB"

[enterprise]
# Enterprise license key
license_key = ""

# RBAC configuration
rbac_enabled = false

# Audit configuration
audit_retention_days = 2555  # 7 years

# Compliance standards
compliance_standards = ["sox", "iso27001"]

# Team management
team_management_enabled = false
```

### Environment Variables

All configuration options can be overridden with environment variables using the prefix `HIVE_`:

```bash
# Core configuration
export HIVE_CONSENSUS_PROFILE="balanced"
export HIVE_OPENROUTER_API_KEY="sk-or-your-key"
export HIVE_STREAMING="true"

# Performance tuning
export HIVE_CACHE_SIZE="4GB"
export HIVE_MAX_WORKERS="16"
export HIVE_MEMORY_LIMIT="8GB"

# Security settings
export HIVE_TRUST_POLICY="prompt"
export HIVE_AUDIT_LOGGING="true"
export HIVE_SANDBOX_MODE="true"

# Integration settings
export HIVE_LSP_PORT="7777"
export HIVE_MCP_PORT="7778"
export HIVE_REST_API_PORT="7779"

# Analytics
export HIVE_ANALYTICS_ENABLED="true"
export HIVE_ANONYMOUS_METRICS="false"

# Logging
export HIVE_LOG_LEVEL="debug"
export HIVE_LOG_FILE="./hive.log"
```

### Project-Specific Configuration

Create `.hive/config.toml` in your project root for project-specific settings:

```toml
[project]
name = "my-awesome-project"
language = "rust"
framework = "actix-web"

[consensus]
# Use faster profile for this project
profile = "speed"

[analysis]
# Custom analysis rules
exclude_patterns = ["target/", "node_modules/", "*.generated.*"]
include_languages = ["rust", "javascript", "typescript"]

# Security rules specific to this project
security_rules = [
    "no-hardcoded-secrets",
    "secure-dependencies",
    "input-validation"
]

[hooks]
# Project-specific hooks
pre_commit = [".hive/hooks/pre-commit.json"]
ci_cd = [".hive/hooks/quality-gate.json"]

[memory]
# Project context
context = [
    "This is a web API using Actix-web framework",
    "Database is PostgreSQL with Diesel ORM",
    "Authentication uses JWT tokens"
]
```

## Hook System API

### Hook Configuration Format

Hooks are configured using JSON files that define triggers, conditions, and actions:

```json
{
  "name": "quality-gate",
  "description": "Enforce code quality standards",
  "version": "1.0.0",
  "enabled": true,
  
  "trigger": {
    "type": "before-consensus",
    "stage": "generator"
  },
  
  "conditions": {
    "file_types": ["*.rs", "*.py", "*.js", "*.ts"],
    "file_size_max": "1MB",
    "complexity_max": 10,
    "min_quality_score": 85
  },
  
  "actions": [
    {
      "type": "consensus-modification",
      "config": {
        "add_context": "Focus on code quality, performance, and security",
        "temperature_adjustment": -0.1,
        "max_tokens_override": 2048
      }
    },
    {
      "type": "validation",
      "config": {
        "run_linter": true,
        "run_tests": false,
        "check_security": true
      }
    }
  ],
  
  "security": {
    "sandbox": true,
    "allowed_commands": ["rustfmt", "clippy"],
    "network_access": false
  }
}
```

### Hook Types

#### Trigger Types
- `before-consensus` - Before consensus pipeline starts
- `after-consensus` - After consensus pipeline completes
- `before-stage` - Before specific consensus stage
- `after-stage` - After specific consensus stage
- `before-apply` - Before applying code changes
- `after-apply` - After applying code changes
- `on-error` - When an error occurs
- `on-file-change` - When files are modified

#### Action Types
- `consensus-modification` - Modify consensus parameters
- `validation` - Run validation checks
- `notification` - Send notifications
- `logging` - Custom logging actions
- `external-command` - Execute external commands
- `approval-workflow` - Trigger approval process

### Example Hook Configurations

#### Pre-commit Quality Gate
```json
{
  "name": "pre-commit-quality",
  "trigger": {
    "type": "before-consensus"
  },
  "conditions": {
    "file_types": ["*.rs"],
    "git_status": "modified"
  },
  "actions": [
    {
      "type": "external-command",
      "config": {
        "command": "cargo fmt --check",
        "on_failure": "block"
      }
    },
    {
      "type": "external-command",
      "config": {
        "command": "cargo clippy -- -D warnings",
        "on_failure": "warn"
      }
    }
  ]
}
```

#### Security Audit Hook
```json
{
  "name": "security-audit",
  "trigger": {
    "type": "after-consensus",
    "stage": "validator"
  },
  "conditions": {
    "contains_keywords": ["password", "secret", "token", "api_key"],
    "security_context": true
  },
  "actions": [
    {
      "type": "validation",
      "config": {
        "security_scan": true,
        "secret_detection": true,
        "vulnerability_check": true
      }
    },
    {
      "type": "approval-workflow",
      "config": {
        "required_approvers": ["security-team"],
        "approval_timeout": "24h"
      }
    }
  ]
}
```

## MCP Tools Reference

### Available MCP Tools

#### `hive_ask`
Ask questions about the codebase using AI consensus.

**Parameters:**
- `question` (string, required): The question to ask
- `files` (array, optional): List of files to analyze
- `consensus_profile` (string, optional): Consensus profile to use
- `streaming` (boolean, optional): Enable streaming response

**Example:**
```json
{
  "question": "What does this authentication function do?",
  "files": ["src/auth.rs"],
  "consensus_profile": "balanced",
  "streaming": true
}
```

#### `hive_analyze`
Perform code analysis and generate insights.

**Parameters:**
- `path` (string, required): Path to analyze
- `focus` (string, optional): Analysis focus (security|performance|maintainability)
- `format` (string, optional): Output format (text|json|markdown)
- `exclude_patterns` (array, optional): Patterns to exclude

#### `hive_improve`
Generate code improvement suggestions.

**Parameters:**
- `path` (string, required): File or directory to improve
- `focus` (string, optional): Improvement focus
- `apply` (boolean, optional): Apply improvements automatically
- `preview` (boolean, optional): Show preview of changes

#### `hive_plan`
Create implementation plans for features and tasks.

**Parameters:**
- `description` (string, required): What to plan
- `scope` (string, optional): Planning scope
- `include_timeline` (boolean, optional): Include timeline estimates
- `include_risks` (boolean, optional): Include risk analysis

#### `hive_memory_search`
Search conversation history and project memory.

**Parameters:**
- `query` (string, required): Search query
- `limit` (integer, optional): Maximum results (default: 10)
- `include_context` (boolean, optional): Include conversation context

### MCP Resource Types

#### `conversations`
Access to conversation history.

**URI Format:** `conversations://[conversation-id]`

#### `project-memory`
Access to project context and memory.

**URI Format:** `project-memory://[project-path]`

#### `analysis-results`
Access to cached analysis results.

**URI Format:** `analysis-results://[analysis-id]`

## REST API Reference

### Authentication

All API requests require authentication via Bearer token:

```bash
curl -H "Authorization: Bearer YOUR-API-KEY" \
     http://localhost:7779/api/v1/ask
```

### Endpoints

#### POST `/api/v1/ask`
Ask questions about the codebase.

**Request Body:**
```json
{
  "question": "What does this function do?",
  "files": ["src/main.rs"],
  "consensus_profile": "balanced",
  "streaming": false
}
```

**Response:**
```json
{
  "response": "This function handles user authentication...",
  "consensus_stages": {
    "generator": "Initial response generated",
    "refiner": "Response refined for clarity",
    "validator": "Response validated for accuracy",
    "curator": "Final response curated"
  },
  "metadata": {
    "tokens_used": 1250,
    "processing_time_ms": 850,
    "models_used": ["claude-3-opus", "gpt-4-turbo"]
  }
}
```

#### POST `/api/v1/analyze`
Perform code analysis.

**Request Body:**
```json
{
  "path": "src/",
  "focus": "security",
  "format": "json",
  "exclude_patterns": ["target/", "*.test.*"]
}
```

#### POST `/api/v1/improve`
Generate code improvements.

**Request Body:**
```json
{
  "path": "src/auth.rs",
  "focus": "performance",
  "apply": false,
  "preview": true
}
```

#### GET `/api/v1/health`
Check API health status.

**Response:**
```json
{
  "status": "healthy",
  "version": "2.0.0",
  "providers": {
    "openrouter": "connected",
    "anthropic": "connected"
  },
  "performance": {
    "avg_response_time_ms": 450,
    "cache_hit_rate": 0.85
  }
}
```

## LSP Features

### Language Server Capabilities

#### Code Completion
Intelligent code completion powered by AI consensus.

**Trigger:** Type partially and press `Ctrl+Space`

#### Hover Information
Detailed information about symbols and functions.

**Trigger:** Hover over any symbol

#### Go to Definition
Navigate to symbol definitions across the codebase.

**Trigger:** `F12` or `Ctrl+Click`

#### Find References
Find all references to a symbol.

**Trigger:** `Shift+F12`

#### Code Actions
AI-powered quick fixes and refactoring suggestions.

**Trigger:** `Ctrl+.` on problematic code

#### Diagnostics
Real-time code analysis and issue detection.

**Automatic:** Updates as you type

### LSP Configuration

#### VS Code
```json
{
  "hive.lsp.enabled": true,
  "hive.lsp.port": 7777,
  "hive.lsp.features": {
    "completion": true,
    "hover": true,
    "diagnostics": true,
    "codeActions": true
  }
}
```

#### Neovim
```lua
require('lspconfig').hive.setup({
  cmd = { 'hive', 'lsp', '--port', '7777' },
  filetypes = { 'rust', 'python', 'javascript', 'typescript' },
  settings = {
    hive = {
      consensus_profile = 'balanced',
      real_time_analysis = true
    }
  }
})
```

---

**This reference covers all major API surfaces of HiveTechs Consensus. For additional examples and advanced usage, see the [USER_GUIDE.md](USER_GUIDE.md).**