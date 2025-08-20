# OpenAI Codex CLI Documentation

## Overview
Codex CLI is OpenAI's coding agent that runs locally in your terminal. It supports multimodal inputs, rich approval workflows, and leverages GPT-5, GPT-4.1, and o-series models for advanced agentic coding capabilities.

**Latest Version**: 0.22.0 (as of August 2025)  
**NPM Package**: `@openai/codex`  
**GitHub**: https://github.com/openai/codex  
**Documentation**: https://help.openai.com/en/articles/11096431-openai-codex-cli-getting-started

## Installation

### Prerequisites
- Node.js version ≥ 22
- macOS or Linux (Windows via WSL2)
- ChatGPT Plus/Pro/Team account OR OpenAI API key

### NPM Installation
```bash
# Global installation (recommended)
npm install -g @openai/codex

# Or use the upgrade command
codex --upgrade

# Project-specific installation
npm i @openai/codex

# Quick test without installation
npx @openai/codex
```

## Authentication

### Method 1: ChatGPT Sign-in (Recommended)
```bash
# Run codex and select "Sign in with ChatGPT"
codex

# Requirements:
# - ChatGPT Plus, Pro, or Team account
# - Access to latest models (GPT-5, o4-mini, o3)
# - No extra cost beyond subscription
```

### Method 2: API Key
```bash
# Export your OpenAI API key
export OPENAI_API_KEY="sk-..."

# Or add to config file
```

## Configuration

### Configuration File
Location: `~/.codex/config.json`

```json
{
  "model": "gpt-5",
  "apiKey": "your-api-key",
  "approvalMode": "auto",
  "sandbox": true,
  "temperature": 0.7
}
```

### Model Selection
```bash
# Default is o4-mini
codex

# Use GPT-4.1
codex --model gpt-4.1

# Use GPT-5 (default for coding)
codex --model gpt-5

# Use o3 for complex reasoning
codex --model o3
```

## Available Models

### 2025 Model Lineup
1. **GPT-5**: Fine-tuned for agentic coding (default)
2. **GPT-4.1**: Latest GPT-4 variant
3. **o4-mini**: Fast, efficient model
4. **o3**: Advanced reasoning model
5. **o3-mini**: Smaller o3 variant

### Model Comparison
| Model | Speed | Capability | Cost | Best For |
|-------|-------|------------|------|----------|
| GPT-5 | Fast | Excellent | Medium | General coding |
| GPT-4.1 | Medium | Very Good | Medium | Complex tasks |
| o4-mini | Very Fast | Good | Low | Quick tasks |
| o3 | Slow | Exceptional | High | Hard problems |

## Features

### Core Capabilities
- **Multimodal Inputs**: Text, screenshots, diagrams
- **Sandboxed Execution**: Safe command execution
- **Approval Workflows**: Three distinct modes
- **Terminal-Native**: Works entirely in your shell
- **File Operations**: Create, edit, delete files
- **Command Execution**: Run terminal commands safely

### Approval Modes
```bash
# Auto mode - executes without asking
codex --approval auto

# Confirm mode - asks before execution
codex --approval confirm

# Manual mode - shows commands, you execute
codex --approval manual
```

### Usage Examples
```bash
# Basic usage
codex "Fix the failing tests"

# With screenshot
codex "Make my app look like this" --image screenshot.png

# Specific file operations
codex "Refactor authentication.js to use async/await"

# Complex tasks
codex "Set up a CI/CD pipeline with GitHub Actions"

# With model selection
codex --model gpt-5 "Optimize this algorithm for performance"
```

## Memory Service Integration

### Configuration
```javascript
{
  "memoryService": {
    "endpoint": "http://localhost:3457",
    "token": "auth-token",
    "enabled": true
  }
}
```

### Benefits
- Persist context across sessions
- Share learnings between projects
- Collaborate with other tools

## Advanced Features

### Multimodal Capabilities
```bash
# Pass images
codex "Create a component like this" --image design.png

# Pass diagrams
codex "Implement this architecture" --file architecture.svg

# Multiple inputs
codex "Match this design and flow" --image ui.png --file flow.md
```

### Sandbox Environment
```bash
# Commands run in sandbox by default
codex "Install dependencies and run tests"

# Disable sandbox (use with caution)
codex --no-sandbox "Deploy to production"
```

### Custom Commands
```json
{
  "aliases": {
    "test": "run all tests and fix failures",
    "deploy": "build and deploy to staging",
    "review": "review code for best practices"
  }
}
```

## ChatGPT Integration

### Benefits of ChatGPT Sign-in
- Access to latest models
- No API key management
- Included in subscription
- Automatic model updates
- Priority processing

### Subscription Tiers
- **Plus ($20/month)**: Access to all models
- **Pro ($30/month)**: Higher limits, priority
- **Team ($25/user/month)**: Collaboration features

## Platform Support

### macOS
Full support, tested extensively

### Linux
Full support on major distributions

### Windows
Requires WSL2:
```bash
# Install WSL2 first
wsl --install

# Then install Codex in WSL
npm install -g @openai/codex
```

## Troubleshooting

### Common Issues

1. **Authentication Error**
   ```bash
   # Re-authenticate
   codex logout
   codex login
   ```

2. **Model Not Available**
   ```bash
   # Check available models
   codex --list-models
   
   # Use default model
   codex --model default
   ```

3. **Sandbox Issues**
   ```bash
   # Reset sandbox
   codex --reset-sandbox
   
   # Run without sandbox
   codex --no-sandbox
   ```

4. **WSL2 on Windows**
   ```bash
   # Ensure WSL2 is updated
   wsl --update
   
   # Install in WSL environment
   wsl npm install -g @openai/codex
   ```

## API Integration

### Programmatic Usage
```javascript
import { Codex } from '@openai/codex';

const codex = new Codex({
  apiKey: process.env.OPENAI_API_KEY,
  model: 'gpt-5'
});

const result = await codex.execute({
  prompt: "Create a REST API",
  approvalMode: 'auto'
});
```

### REST API
```bash
curl https://api.openai.com/v1/codex/execute \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Your task here",
    "model": "gpt-5"
  }'
```

## Best Practices

1. **Model Selection**: Use GPT-5 for coding, o3 for complex reasoning
2. **Approval Mode**: Start with 'confirm' mode for safety
3. **Context Management**: Keep prompts focused
4. **Security**: Review commands before execution
5. **Efficiency**: Use o4-mini for simple tasks

## Cost Optimization

### Tips for API Users
- Use o4-mini for simple tasks
- Enable caching for repeated operations
- Batch similar requests
- Monitor usage with `codex --usage`

### ChatGPT Subscription
- Fixed monthly cost
- Unlimited* usage (*fair use policy)
- Best value for regular users

## Recent Updates (2025)

### June 2025
- Codex available to ChatGPT Plus users
- GPT-5 model launch
- Improved sandbox security

### July 2025
- o3 and o3-mini models added
- Performance improvements
- Bug fixes and stability

### August 2025
- Version 0.22.0 release
- Enhanced multimodal support
- Better error handling

## Comparison with Other Tools

| Feature | Codex CLI | Claude Code | Gemini CLI |
|---------|-----------|-------------|------------|
| Models | GPT-5, o3 | Claude 3.7 | Gemini 2.5 |
| Multimodal | ✅ | ❌ | Limited |
| Sandbox | ✅ | ❌ | ❌ |
| Free Tier | ❌ | ❌ | ✅ |
| ChatGPT Login | ✅ | ❌ | ❌ |

## MCP (Model Context Protocol) Support

### Overview
Codex CLI supports MCP servers for extending functionality and connecting to external tools and data sources. MCP integration allows Codex to interact with databases, APIs, and custom tools.

### MCP Server Configuration

#### Configuration File Location
`~/.codex/config.toml`

#### MCP Server Setup
```toml
[[mcp_servers]]
name = "database"
command = "node"
args = ["./mcp-servers/database.js"]
env = { DATABASE_URL = "${env:DATABASE_URL}" }

[[mcp_servers]]
name = "github"
command = "python"
args = ["-m", "github_mcp_server"]
env = { GITHUB_TOKEN = "${env:GITHUB_TOKEN}" }
```

## Custom Model Providers

### Provider Configuration
```toml
[model_providers.azure]
name = "Azure OpenAI"
base_url = "https://YOUR_PROJECT.openai.azure.com/openai"
env_key = "AZURE_OPENAI_API_KEY"
query_params = { api-version = "2025-04-01-preview" }
wire_api = "responses"

[model_providers.ollama]
name = "Ollama"
base_url = "http://localhost:11434"
wire_api = "openai"
no_auth = true

[model_providers.openrouter]
name = "OpenRouter"
base_url = "https://openrouter.ai/api/v1"
env_key = "OPENROUTER_API_KEY"
wire_api = "openai"
```

## Configuration Profiles

### Profile System
Profiles allow you to save and switch between different configurations easily.

```toml
# Default profile
model = "o4-mini"
sandbox = "workspace-write"
approval = "on-request"

# Azure profile
[profiles.azure]
model_provider = "azure"
model = "gpt-4.1"
sandbox = "read-only"
approval = "untrusted"

# Local development profile
[profiles.local]
model_provider = "ollama"
model = "codellama:34b"
sandbox = "danger-full-access"
approval = "never"

# High-security profile
[profiles.secure]
model = "gpt-5"
sandbox = "read-only"
approval = "untrusted"
```

### Using Profiles
```bash
# Use specific profile
codex --profile azure

# Set default profile
codex config set default_profile local
```

## AGENTS.md Context System

### Overview
Codex uses `AGENTS.md` files to provide project-specific instructions and context.

### File Locations (Priority Order)
1. Current directory: `./AGENTS.md`
2. Parent directories (searched recursively)
3. Project root: `<project>/AGENTS.md`
4. Global: `~/.codex/AGENTS.md`

### AGENTS.md Structure
```markdown
# Project Context

## Architecture
This is a Django application using PostgreSQL...

## Coding Standards
- Use PEP 8 for Python code
- Write comprehensive docstrings
- Include type hints

## Security Requirements
- Validate all inputs
- Use prepared statements
- Never log sensitive data

## Custom Commands
!test: Run pytest with coverage
!deploy: Deploy to staging environment
!lint: Run all linters and formatters
```

## Sandbox Modes

### Security Levels
```toml
# Read-only mode (safest)
sandbox = "read-only"

# Workspace write (default)
sandbox = "workspace-write"

# Full system access (dangerous)
sandbox = "danger-full-access"
```

### Approval Policies
```toml
# Always require approval
approval = "untrusted"

# Ask when needed (default)
approval = "on-request"

# Never ask (dangerous)
approval = "never"
```

## Open Source Model Support

### Local Model Configuration
```bash
# Use with Ollama
codex --oss --model codellama:34b

# Use with custom endpoint
codex --oss --base-url http://localhost:8080 --model local-model
```

### Ollama Integration
```toml
[model_providers.ollama]
name = "Ollama"
base_url = "http://localhost:11434"
wire_api = "openai"
no_auth = true

# Set as default
default_model_provider = "ollama"
```

## Advanced Configuration

### Complete Config Example
```toml
# ~/.codex/config.toml

# Authentication
preferred_auth_method = "chatgpt"  # or "apikey"
api_key = "${env:OPENAI_API_KEY}"

# Model settings
model = "o4-mini"
temperature = 0.7
max_tokens = 4096

# Security
sandbox = "workspace-write"
approval = "on-request"
unsafe_commands = ["rm -rf", "format", "del /f"]

# Context
context_window = 128000
smart_context = true
include_git_diff = true

# Performance
stream = true
cache_responses = true
parallel_tools = true

# UI
theme = "dark"
show_reasoning = true
verbose = false

# MCP Servers
[[mcp_servers]]
name = "memory"
command = "node"
args = ["./memory-mcp-bridge.js"]
env = { MEMORY_SERVICE_URL = "http://localhost:3457" }

# Profiles
[profiles.production]
model = "gpt-5"
sandbox = "read-only"
approval = "untrusted"

[profiles.development]
model = "o3"
sandbox = "danger-full-access"
approval = "never"
```

## Custom Tools and Extensions

### Tool Registration
```toml
[[custom_tools]]
name = "database_query"
command = "python"
args = ["./tools/db_query.py"]
description = "Execute database queries"
dangerous = false

[[custom_tools]]
name = "deploy"
command = "bash"
args = ["./scripts/deploy.sh"]
description = "Deploy to production"
dangerous = true
requires_approval = true
```

### Hook Scripts
```toml
# Pre/post execution hooks
[hooks]
pre_command = "./hooks/pre_command.sh"
post_command = "./hooks/post_command.sh"
on_error = "./hooks/error_handler.py"
on_approval = "./hooks/approval_logger.py"
```

## Memory Service Integration

### Configuration
```toml
[memory_service]
enabled = true
endpoint = "http://localhost:3457"
auth_token = "${env:MEMORY_TOKEN}"
auto_sync = true
sync_interval = 300
```

### Memory Commands
```bash
# Check memory status
codex memory status

# Clear project memory
codex memory clear --project

# Export memory
codex memory export ./backup.json
```

## Multimodal Capabilities

### Image Input
```bash
# Analyze screenshot
codex "What's wrong with this UI?" --image screenshot.png

# Multiple images
codex "Compare these designs" --image design1.png design2.png

# Clipboard image
codex "Debug this error" --clipboard
```

### Diagram Understanding
```bash
# Architecture diagrams
codex "Implement this architecture" --image architecture.svg

# Flowcharts
codex "Convert to code" --image flowchart.png
```

## Operating Modes

### Suggest Mode
```bash
# Get suggestions without execution
codex suggest "Optimize this function"
```

### Auto Edit Mode
```bash
# Edit files with approval
codex edit "Refactor all classes to use dependency injection"
```

### Full Auto Mode
```bash
# Autonomous operation (dangerous)
codex auto "Complete the TODO items in this project"
```

## Troubleshooting

### Debug Mode
```bash
# Enable debug output
codex --debug

# Verbose logging
codex --verbose

# Show reasoning process
codex --show-reasoning
```

### Common Issues

1. **Model Not Available**
   ```bash
   # Check available models
   codex models list
   
   # Update model list
   codex models refresh
   ```

2. **MCP Server Connection Failed**
   ```bash
   # Test MCP server
   codex mcp test <server-name>
   
   # View logs
   codex mcp logs <server-name>
   ```

3. **Authentication Issues**
   ```bash
   # Re-authenticate
   codex auth refresh
   
   # Switch auth method
   codex config set preferred_auth_method apikey
   ```

## Best Practices

1. **Security First**: Always use appropriate sandbox mode
2. **Context Management**: Keep AGENTS.md files updated
3. **Profile Usage**: Create profiles for different environments
4. **Approval Workflow**: Never disable approval in production
5. **Model Selection**: Use appropriate models for task complexity
6. **Memory Integration**: Enable for better context retention
7. **Version Control**: Commit config files for team consistency

## VS Code Extension

### Installation
Search for "andromeda-codex" in VS Code marketplace

### Features
- Inline suggestions
- Auto-edit mode
- Multimodal input support
- Integrated approval UI

## Links
- [Getting Started Guide](https://help.openai.com/en/articles/11096431-openai-codex-cli-getting-started)
- [GitHub Repository](https://github.com/openai/codex)
- [NPM Package](https://www.npmjs.com/package/@openai/codex)
- [OpenAI Platform](https://platform.openai.com)
- [ChatGPT Subscription](https://chat.openai.com/subscription)
- [VS Code Extension](https://marketplace.visualstudio.com/items?itemName=openai.andromeda-codex)
- [API Documentation](https://platform.openai.com/docs)