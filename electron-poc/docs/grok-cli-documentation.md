# Grok CLI - Complete Documentation

## Overview
Grok CLI is an open-source AI agent that brings the power of Grok (from xAI) directly into your terminal. It provides conversational command-line AI assistance for development tasks, similar to Claude Code CLI or Gemini CLI.

**Current Version**: 0.0.23  
**License**: MIT  
**NPM Package**: `@vibe-kit/grok-cli`  
**GitHub Repository**: https://github.com/superagent-ai/grok-cli  
**Documentation URL**: https://github.com/superagent-ai/grok-cli  

## Key Features
- **Conversational AI CLI** powered by Grok-3 and Grok-4 models
- **Smart File Operations** - Read, write, create, and modify files
- **Bash Integration** - Execute shell commands directly
- **Automatic Tool Selection** - AI determines which tools to use
- **Morph Fast Apply** - 4,500+ tokens/sec with 98% accuracy for code editing
- **MCP Support** - Model Context Protocol for extensibility
- **Interactive Terminal UI** - Beautiful interface built with Ink
- **Multi-Provider Support** - Works with OpenAI-compatible APIs
- **Global Installation** - Available everywhere via npm

## Installation

### Prerequisites
- Node.js 16 or higher
- npm or yarn package manager
- Grok API key from X.AI (https://console.x.ai/team/default/api-keys)

### Global Installation
```bash
npm install -g @vibe-kit/grok-cli
```

### Verify Installation
```bash
grok --version
```

## Configuration

### 1. API Key Setup (Required)

#### Option A: Environment Variable
```bash
export GROK_API_KEY="your_api_key_here"
```

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) for persistence:
```bash
echo 'export GROK_API_KEY="your_api_key_here"' >> ~/.zshrc
source ~/.zshrc
```

#### Option B: User Settings File
Create `~/.grok/user-settings.json`:
```json
{
  "apiKey": "your_api_key_here",
  "baseURL": "https://api.x.ai/v1",
  "defaultModel": "grok-4-latest",
  "temperature": 0.7,
  "maxTokens": 4096
}
```

#### Option C: .env File (Project-Specific)
Create `.env` in your project directory:
```env
GROK_API_KEY=your_api_key_here
GROK_BASE_URL=https://api.x.ai/v1
GROK_MODEL=grok-4-latest
```

#### Option D: Command Line Flag
```bash
grok --api-key your_api_key_here
```

### 2. Model Configuration

Available models:
- `grok-3` - Balanced performance
- `grok-4-latest` - Most advanced (default)
- `grok-4-turbo` - Faster responses
- Custom OpenAI-compatible models

Set default model in user settings or use command flag:
```bash
grok --model grok-4-latest
```

### 3. Advanced Configuration

#### Custom Base URL (for OpenAI-compatible APIs)
```json
{
  "baseURL": "https://your-api-endpoint.com/v1",
  "apiKey": "your_api_key",
  "defaultModel": "custom-model"
}
```

#### Tool Execution Limits
```json
{
  "maxToolRounds": 400,
  "toolTimeout": 30000,
  "enableMorph": true
}
```

## Usage Modes

### 1. Interactive Mode (Default)
Launch interactive terminal UI:
```bash
grok
```

Features in interactive mode:
- Continuous conversation
- Visual feedback for tool usage
- File tree navigation
- Command history (arrow keys)
- Multi-line input support

### 2. Headless Mode (Single Command)
Execute single task without UI:
```bash
grok --prompt "show me the package.json file"
grok -p "create a new Express server in app.js"
grok --prompt "explain this error" < error.log
```

### 3. Piped Input Mode
```bash
cat README.md | grok --prompt "summarize this file"
git diff | grok -p "explain these changes"
```

## Command Reference

### Basic Commands
```bash
# Help and version
grok --help
grok --version

# Interactive mode
grok

# Single prompt
grok --prompt "your task here"
grok -p "your task here"
```

### File Operations
```bash
# Read files
grok -p "show me the contents of app.js"
grok -p "what does config.json contain?"

# Write/Create files
grok -p "create a new file called server.js with an Express server"
grok -p "add error handling to main.py"

# Modify files
grok -p "refactor the login function in auth.js"
grok -p "fix the bug in calculate.py line 42"
```

### Code Generation
```bash
# Generate new code
grok -p "create a React component for a todo list"
grok -p "write a Python script to parse CSV files"

# Convert/Transform code
grok -p "convert this JavaScript to TypeScript"
grok -p "refactor this function to use async/await"
```

### Project Analysis
```bash
# Analyze codebase
grok -p "explain the architecture of this project"
grok -p "find all API endpoints in this Express app"

# Documentation
grok -p "generate README for this project"
grok -p "add JSDoc comments to all functions"
```

### Shell Integration
```bash
# Execute commands
grok -p "show me running processes"
grok -p "install dependencies and run tests"

# Git operations
grok -p "show recent commits"
grok -p "create a commit message for these changes"
```

### Advanced Options
```bash
# Specify working directory
grok --directory /path/to/project -p "analyze this codebase"
grok -d ./src -p "refactor all components"

# Model selection
grok --model grok-3 -p "quick task"
grok -m grok-4-turbo -p "complex analysis"

# API key override
grok --api-key different_key -p "task"
grok -k test_key -p "experiment"

# Tool execution control
grok --max-tool-rounds 10 -p "limited task"
grok --no-tools -p "just conversation"

# Output control
grok --json -p "task" > output.json
grok --quiet -p "task"
grok --verbose -p "debugging task"
```

## MCP (Model Context Protocol) Support

### What is MCP?
MCP allows extending Grok CLI with custom tools and servers, enabling:
- Database connections
- API integrations
- Custom file operations
- External service interactions

### Adding MCP Servers

#### 1. Stdio-based MCP Server
```bash
# Add a custom server
grok mcp add my-server \
  --transport stdio \
  --command "node" \
  --args "path/to/server.js"

# With environment variables
grok mcp add db-server \
  --transport stdio \
  --command "python" \
  --args "db_server.py" \
  --env "DB_HOST=localhost" \
  --env "DB_PORT=5432"
```

#### 2. HTTP-based MCP Server
```bash
grok mcp add api-server \
  --transport http \
  --url "http://localhost:8080/mcp"
```

### Managing MCP Servers
```bash
# List servers
grok mcp list

# Remove server
grok mcp remove my-server

# Test server
grok mcp test my-server
```

### MCP Configuration File
Create `~/.grok/mcp-config.json`:
```json
{
  "servers": {
    "github": {
      "transport": "stdio",
      "command": "github-mcp",
      "args": ["--token", "${GITHUB_TOKEN}"]
    },
    "linear": {
      "transport": "http",
      "url": "http://localhost:3000/linear-mcp"
    }
  }
}
```

## Morph Fast Apply Feature

### What is Morph?
High-speed code editing with:
- 4,500+ tokens/second processing
- 98% accuracy
- Complex multi-file transformations
- Parallel file processing

### Enabling Morph
1. Get Morph API key from provider
2. Add to configuration:
```json
{
  "morphApiKey": "your_morph_key",
  "enableMorph": true,
  "morphSettings": {
    "maxParallel": 5,
    "chunkSize": 1000
  }
}
```

### Using Morph
```bash
# Auto-enables for large operations
grok -p "refactor entire codebase to use TypeScript"

# Force Morph usage
grok --use-morph -p "update all imports"

# Disable Morph
grok --no-morph -p "simple edit"
```

## Environment Variables

Complete list of supported environment variables:

```bash
# Core Configuration
GROK_API_KEY=your_api_key
GROK_BASE_URL=https://api.x.ai/v1
GROK_MODEL=grok-4-latest

# Optional Settings
GROK_TEMPERATURE=0.7
GROK_MAX_TOKENS=4096
GROK_MAX_TOOL_ROUNDS=400
GROK_TIMEOUT=30000

# Feature Flags
GROK_ENABLE_MORPH=true
GROK_MORPH_API_KEY=morph_key
GROK_ENABLE_MCP=true
GROK_DEBUG=true
GROK_QUIET=false

# Directories
GROK_CONFIG_DIR=~/.grok
GROK_CACHE_DIR=~/.grok/cache
GROK_LOG_DIR=~/.grok/logs
```

## Troubleshooting

### Common Issues

#### 1. API Key Not Found
```bash
# Check current configuration
grok config show

# Set API key
export GROK_API_KEY="your_key"
# or
grok config set apiKey your_key
```

#### 2. Connection Errors
```bash
# Test connection
grok test-connection

# Use different endpoint
grok --base-url https://api.x.ai/v1 test-connection
```

#### 3. Model Not Available
```bash
# List available models
grok models list

# Use default model
grok --model grok-3 -p "task"
```

#### 4. Permission Errors
```bash
# Fix permissions
chmod +x $(which grok)

# Reinstall globally with sudo (if needed)
sudo npm install -g @vibe-kit/grok-cli
```

### Debug Mode
```bash
# Enable verbose logging
grok --debug -p "task"

# Check logs
cat ~/.grok/logs/debug.log
```

## Integration with Hive

### Terminal Spawning
Grok CLI can be spawned in Hive's terminal orchestrator:

```typescript
// Add to CLI_TOOLS_REGISTRY
'grok': {
  id: 'grok',
  name: 'Grok CLI',
  description: 'xAI Grok-powered terminal agent with MCP support',
  command: 'grok',
  installCommand: 'npm install -g @vibe-kit/grok-cli',
  versionCommand: 'grok --version',
  versionRegex: /(\d+\.\d+\.\d+)/,
  docsUrl: 'https://github.com/superagent-ai/grok-cli',
  icon: '🤖',
  requiresNode: true
}
```

### Launch Command
For interactive mode in terminal:
```bash
grok
```

### Health Check
```bash
grok --version
```

## Best Practices

### 1. Project-Specific Configuration
Create `.grok/config.json` in project root:
```json
{
  "model": "grok-4-latest",
  "maxToolRounds": 100,
  "includePaths": ["src", "lib"],
  "excludePaths": ["node_modules", "dist"]
}
```

### 2. Security
- Never commit API keys to version control
- Use environment variables in CI/CD
- Rotate API keys regularly
- Use project-specific keys when possible

### 3. Performance
- Use `grok-4-turbo` for faster responses
- Limit tool rounds for simple tasks
- Enable Morph for large refactoring
- Cache responses when appropriate

### 4. Workflow Integration
```bash
# Git hooks (.git/hooks/prepare-commit-msg)
#!/bin/bash
grok -p "generate commit message" > .git/COMMIT_MSG

# CI/CD pipeline
grok -p "review code changes" --json > review.json

# Pre-push validation
grok -p "check for security issues" || exit 1
```

## Comparison with Other CLI Tools

| Feature | Grok CLI | Claude Code | Gemini CLI | Cline |
|---------|----------|-------------|------------|-------|
| Speed | 4,500 tokens/sec (Morph) | Standard | Standard | Standard |
| MCP Support | ✅ | ❌ | ❌ | ❌ |
| Open Source | ✅ | ❌ | ✅ | ✅ |
| Free Tier | ❌ | ❌ | ✅ (generous) | Depends on provider |
| Multi-Provider | ✅ | ❌ | ❌ | ✅ |
| Terminal UI | ✅ (Ink) | ✅ | ✅ | ❌ |
| GitHub Stars | Growing | 31k | 55k | 47k |

## Updates and Versions

### Current Version: 0.0.23
- Released: 18 days ago
- Status: Active development
- Breaking changes: Check CHANGELOG.md

### Updating
```bash
# Update to latest
npm update -g @vibe-kit/grok-cli

# Check for updates
npm outdated -g @vibe-kit/grok-cli

# Specific version
npm install -g @vibe-kit/grok-cli@0.0.23
```

## Resources

- **GitHub Repository**: https://github.com/superagent-ai/grok-cli
- **NPM Package**: https://www.npmjs.com/package/@vibe-kit/grok-cli
- **X.AI API Keys**: https://console.x.ai/team/default/api-keys
- **Issue Tracker**: https://github.com/superagent-ai/grok-cli/issues
- **Discord Community**: Check GitHub for link
- **VibeKit Ecosystem**: https://github.com/superagent-ai/vibekit

## License
MIT License - Open source and free to use