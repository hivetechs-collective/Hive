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

## MCP (Model Context Protocol)

### Overview
MCP is an open-source standard that allows Claude Code to connect to external tools, databases, and APIs, enabling complex integrations across different platforms.

### MCP Server Types

#### 1. Local stdio Servers
```bash
# Add a local MCP server
claude mcp add airtable --env AIRTABLE_API_KEY=YOUR_KEY -- npx -y airtable-mcp-server
```
- Run as local processes
- Direct system access
- Best for development tools

#### 2. Remote SSE Servers
```bash
# Add SSE server for real-time connections
claude mcp add --transport sse linear https://mcp.linear.app/sse
```
- Server-Sent Events for streaming
- Real-time updates
- Ideal for monitoring tools

#### 3. Remote HTTP Servers
```bash
# Add HTTP server
claude mcp add --transport http notion https://mcp.notion.com/mcp
```
- Standard request/response
- OAuth 2.0 authentication
- Good for REST APIs

### MCP Server Scopes
- **Local**: Personal configurations
- **Project**: Team-shared in `.mcp.json`
- **User**: Cross-project utilities

### MCP Management Commands
```bash
claude mcp list          # List all configured servers
claude mcp get <server>  # Get server details
claude mcp remove <name> # Remove a server
/mcp                     # In-session authentication
```

### Available MCP Categories
- Development & Testing Tools
- Project Management (Jira, Linear, GitHub)
- Databases (PostgreSQL, SQLite, MongoDB)
- Payments & Commerce (Stripe)
- Design & Media (Figma)
- Infrastructure & DevOps (AWS, Docker)

## Hooks System

### Overview
Hooks are configurable scripts that run at specific points during Claude Code's operation, enabling custom workflows and integrations.

### Hook Events

#### 1. PreToolUse
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Read|Write|Edit",
        "command": "echo 'File operation detected' >> audit.log"
      }
    ]
  }
}
```

#### 2. PostToolUse
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "command": "./scripts/post-command-validation.sh"
      }
    ]
  }
}
```

#### 3. UserPromptSubmit
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "./scripts/inject-context.sh"
      }
    ]
  }
}
```

#### 4. Other Events
- **Stop**: Main agent finishes
- **SubagentStop**: Subagent completes
- **PreCompact**: Before context compaction
- **SessionStart**: New session begins
- **Notification**: Specific notifications

### Hook Configuration

#### Settings File Location
`~/.claude/settings.json`

#### Example Configuration
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "^(Edit|Write)$",
        "command": "bash -c 'if [[ \"$1\" == *\".env\"* ]]; then exit 1; fi' -- \"$(jq -r .params.file_path)\""
      }
    ],
    "UserPromptSubmit": [
      {
        "command": "echo 'Project: $CLAUDE_PROJECT_DIR'"
      }
    ]
  }
}
```

### Hook Input/Output

#### Input (via stdin)
```json
{
  "sessionId": "abc123",
  "projectDir": "/path/to/project",
  "tool": "Edit",
  "params": {
    "file_path": "src/main.js"
  }
}
```

#### Output Options
- Exit code (0 = success, non-zero = block)
- JSON response via stdout
- Error messages via stderr

### Security Considerations
- Hooks execute arbitrary shell commands
- Validate all inputs in hook scripts
- Use absolute paths for scripts
- Implement permission checks
- Never expose sensitive data

### Common Use Cases

#### 1. File Protection
```bash
#!/bin/bash
# Prevent editing sensitive files
if [[ "$1" =~ \.(env|key|pem)$ ]]; then
  echo "Cannot edit sensitive files" >&2
  exit 1
fi
```

#### 2. Audit Logging
```bash
#!/bin/bash
# Log all tool usage
echo "$(date): $TOOL_NAME used on $FILE_PATH" >> ~/.claude/audit.log
```

#### 3. Context Injection
```bash
#!/bin/bash
# Add project context to prompts
echo "Current branch: $(git branch --show-current)"
echo "Last commit: $(git log -1 --oneline)"
```

### Debugging Hooks
```bash
# Run with debug output
claude --debug

# Check hook configuration
/hooks

# Test hook manually
echo '{"tool":"Edit"}' | ./my-hook.sh
```

## Advanced Configuration

### Settings Structure
```json
{
  "apiKey": "sk-ant-...",
  "model": "claude-3-7-sonnet",
  "customInstructions": "Always follow PEP 8 for Python",
  "contextWindow": 200000,
  "temperature": 0.7,
  "hooks": { ... },
  "mcpServers": { ... },
  "autoUpdate": true,
  "telemetry": false
}
```

### Environment Variables
```bash
export CLAUDE_API_KEY="sk-ant-..."
export CLAUDE_MODEL="claude-3-7-sonnet"
export CLAUDE_PROJECT_DIR="/path/to/project"
export CLAUDE_CONTEXT_WINDOW="200000"
export CLAUDE_DEBUG="true"
```

## Integration with Memory Service

### Automatic Connection
Claude Code automatically detects and connects to the Memory Service if running on port 3457.

### Manual Configuration
```json
{
  "memoryService": {
    "endpoint": "http://localhost:3457",
    "token": "auth-token",
    "autoConnect": true
  }
}
```

### Benefits
- Persistent context across sessions
- Shared knowledge with other AI tools
- Project-specific memory banks
- Team collaboration features

## Best Practices

1. **Context Management**: Keep prompts focused and specific
2. **File Safety**: Review changes before accepting
3. **Git Integration**: Use Claude's git features for version control
4. **Memory Efficiency**: Clear context periodically for large projects
5. **Security**: Never share API keys or sensitive data in prompts
6. **Hook Safety**: Validate all inputs in hook scripts
7. **MCP Selection**: Choose appropriate transport for each integration
8. **Performance**: Use local MCP servers for latency-sensitive operations

## Troubleshooting

### Common Issues

1. **MCP Server Connection Failed**
   ```bash
   # Check server status
   claude mcp list
   
   # Re-authenticate
   /mcp
   
   # Remove and re-add
   claude mcp remove <server>
   claude mcp add ...
   ```

2. **Hook Not Executing**
   ```bash
   # Check configuration
   /hooks
   
   # Run with debug
   claude --debug
   
   # Verify script permissions
   chmod +x ./scripts/my-hook.sh
   ```

3. **Memory Service Not Connecting**
   ```bash
   # Check if service is running
   curl http://localhost:3457/health
   
   # Restart connection
   /memory reconnect
   ```

## Links
- [Official Documentation](https://docs.anthropic.com/en/docs/claude-code)
- [GitHub Repository](https://github.com/anthropics/claude-code)
- [NPM Package](https://www.npmjs.com/package/@anthropic-ai/claude-code)
- [Best Practices Guide](https://www.anthropic.com/engineering/claude-code-best-practices)
- [SDK Documentation](https://docs.anthropic.com/en/docs/claude-code/sdk)