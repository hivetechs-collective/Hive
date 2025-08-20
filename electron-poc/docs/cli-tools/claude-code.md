# Claude Code CLI Documentation

## Overview
Claude Code is Anthropic's agentic coding tool that lives in your terminal, understands your codebase, and helps you code faster by executing routine tasks, explaining complex code, and handling git workflows - all through natural language commands.

**Latest Version**: 1.0.85 (as of August 2025)  
**NPM Package**: `@anthropic-ai/claude-code`  
**GitHub**: https://github.com/anthropics/claude-code  
**Documentation**: https://docs.anthropic.com/en/docs/claude-code

## Installation

### Prerequisites
- Node.js version ≥ 18 (preferably installed via nvm)
- Terminal or command line interface
- Active billing at console.anthropic.com

### NPM Installation
```bash
# Global installation (recommended)
npm install -g @anthropic-ai/claude-code

# DO NOT use sudo npm install -g (causes permission issues)
```

### Alternative Installation Methods
```bash
# Bash script installation (stable version)
curl -fsSL https://claude.ai/install.sh | bash

# Project-specific installation
npm i @anthropic-ai/claude-code
```

## Authentication

### Method 1: Anthropic Console (Default)
1. Run `claude` in your terminal
2. Complete OAuth process via browser
3. Requires active billing at console.anthropic.com

### Method 2: API Key
```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

## Configuration

### Initial Setup
```bash
# Navigate to your project
cd your-awesome-project

# Start Claude Code
claude

# Verify installation
claude doctor
```

### Configuration File
Location: `~/.claude/config.json`

```json
{
  "apiKey": "your-api-key",
  "model": "claude-3.7-sonnet",
  "contextWindow": 200000,
  "autoUpdate": true
}
```

## Features

### Core Capabilities
- **Codebase Understanding**: Analyzes entire project structure
- **File Operations**: Create, edit, delete files directly
- **Command Execution**: Run terminal commands
- **Git Integration**: Create commits, handle workflows
- **Web Search**: Find up-to-date information

### Usage Examples
```bash
# Basic usage
claude "Fix the TypeScript errors in this project"

# Specific file operations
claude "Add error handling to src/api.ts"

# Git operations
claude "Create a commit for these changes"

# Complex tasks
claude "Refactor the authentication system to use JWT"
```

## Memory Service Integration

### Connection Setup
Claude Code can integrate with our Memory Service for enhanced context:

```javascript
// Configuration for Memory Service
{
  "memoryService": {
    "endpoint": "http://localhost:3457",
    "token": "generated-auth-token",
    "enabled": true
  }
}
```

### API Endpoints
- `/api/v1/memory/register` - Register Claude Code
- `/api/v1/memory/query` - Query existing knowledge
- `/api/v1/memory/contribute` - Share learnings

## MCP (Model Context Protocol) Support

Claude Code supports MCP for extending functionality:

### MCP Server Configuration
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem"]
    }
  }
}
```

## Updates

### Auto-Update
Claude Code automatically checks for updates on startup and periodically while running.

### Manual Update
```bash
# Check current version
claude --version

# Update to latest
npm update -g @anthropic-ai/claude-code
```

## Pricing
- **Pro Plan**: $20/month
- **Max Plan**: $100/month
- Includes access to Claude 3.7 Sonnet model

## System Requirements
- **OS**: macOS, Linux, Windows (via WSL)
- **RAM**: Minimum 4GB recommended
- **Disk Space**: 500MB for installation
- **Network**: Internet connection required

## Troubleshooting

### Common Issues

1. **Permission Errors**
   ```bash
   # Fix npm permissions
   npm config set prefix ~/.npm
   export PATH=~/.npm/bin:$PATH
   ```

2. **Authentication Failed**
   ```bash
   # Clear credentials
   claude logout
   claude login
   ```

3. **Version Conflicts**
   ```bash
   # Clean install
   npm uninstall -g @anthropic-ai/claude-code
   npm cache clean --force
   npm install -g @anthropic-ai/claude-code
   ```

## SDK Integration

For programmatic access:

```javascript
import { ClaudeCode } from '@anthropic-ai/claude-code';

const claude = new ClaudeCode({
  apiKey: process.env.ANTHROPIC_API_KEY
});

const response = await claude.execute({
  prompt: "Analyze this codebase",
  context: "./src"
});
```

## Best Practices

1. **Context Management**: Keep prompts focused and specific
2. **File Safety**: Review changes before accepting
3. **Git Integration**: Use Claude's git features for version control
4. **Memory Efficiency**: Clear context periodically for large projects
5. **Security**: Never share API keys or sensitive data in prompts

## Links
- [Official Documentation](https://docs.anthropic.com/en/docs/claude-code)
- [GitHub Repository](https://github.com/anthropics/claude-code)
- [NPM Package](https://www.npmjs.com/package/@anthropic-ai/claude-code)
- [Best Practices Guide](https://www.anthropic.com/engineering/claude-code-best-practices)
- [SDK Documentation](https://docs.anthropic.com/en/docs/claude-code/sdk)