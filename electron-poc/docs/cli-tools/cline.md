# Cline CLI Documentation

## Overview
Cline (pronounced /klaɪn/, like "Klein") is an AI assistant that can use your CLI and Editor. Originally a VS Code extension with 2.7M+ users, Cline CLI provides command-line access to similar agentic coding capabilities powered by Claude 3.7 Sonnet and other models.

**Latest Version**: 0.1.1 (CLI version)  
**NPM Package**: `@yaegaki/cline-cli`  
**GitHub**: https://github.com/cline/cline  
**Documentation**: https://cline.bot/  
**Stars**: 47.2k+ on GitHub

## Installation

### Prerequisites
- Node.js version ≥ 16
- API keys for chosen LLM provider
- Terminal or command line interface

### NPM Installation

#### Global Installation
```bash
# Install globally
npm install -g @yaegaki/cline-cli

# Verify installation
cline-cli --version
```

#### Using npx (No Installation)
```bash
# Run without installing
npx -y @yaegaki/cline-cli init
npx -y @yaegaki/cline-cli task "your task description"
```

## Initial Setup

### Configuration
```bash
# Initialize with default settings
cline-cli init

# Initialize with custom settings path
cline-cli init --settings /path/to/your/settings.json
```

### Settings File Structure
Location: `~/.cline/settings.json`

```json
{
  "apiProvider": "anthropic",
  "apiKey": "your-api-key",
  "model": "claude-3-7-sonnet",
  "customInstructions": "",
  "autoApprove": false,
  "contextWindow": 200000,
  "temperature": 0.7
}
```

## Supported Providers

### Anthropic (Claude)
```json
{
  "apiProvider": "anthropic",
  "apiKey": "sk-ant-...",
  "model": "claude-3-7-sonnet"
}
```

### OpenRouter
```json
{
  "apiProvider": "openrouter",
  "apiKey": "sk-or-...",
  "model": "anthropic/claude-3.7-sonnet"
}
```

### OpenAI
```json
{
  "apiProvider": "openai",
  "apiKey": "sk-...",
  "model": "gpt-4o"
}
```

### AWS Bedrock
```json
{
  "apiProvider": "bedrock",
  "awsAccessKey": "...",
  "awsSecretKey": "...",
  "awsRegion": "us-east-1",
  "model": "anthropic.claude-3-sonnet"
}
```

### Other Providers
- Gemini
- Vertex AI
- Azure OpenAI
- Local models via Ollama
- Any OpenAI-compatible API

## Core Features

### Task Execution
```bash
# Basic task
cline-cli task "Fix the TypeScript errors in this project"

# With specific files
cline-cli task "Refactor user.ts to use async/await" --files src/user.ts

# With context
cline-cli task "Add error handling" --context "Use try-catch blocks"
```

### File Operations
- Create and edit files
- Navigate project structures
- Read and analyze code
- Execute file transformations

### Command Execution
**Note**: CLI version has limitations
- `browser_action` not supported
- `execute_command` not supported
- File operations fully supported

## Usage Examples

### Basic Usage
```bash
# Initialize project
cline-cli init

# Simple task
cline-cli task "Add a README file with project documentation"

# Code refactoring
cline-cli task "Convert all class components to functional components"

# Bug fixing
cline-cli task "Find and fix the memory leak in the application"

# Feature implementation
cline-cli task "Implement user authentication with JWT"
```

### Advanced Usage
```bash
# With auto-approval (use cautiously)
cline-cli task "Format all files" --auto-approve

# With custom model
cline-cli task "Complex refactoring" --model claude-3-opus

# With increased context
cline-cli task "Analyze codebase" --context-window 300000
```

## Memory Service Integration

### Configuration
```json
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
- Learn from previous interactions
- Share knowledge with other tools

## MCP (Model Context Protocol) Support

### What is MCP?
MCP allows Cline to:
- Connect to external data sources
- Use custom tools and APIs
- Extend functionality dynamically

### MCP Server Configuration
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/path"]
    },
    "postgres": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-postgres", "connection-string"]
    }
  }
}
```

## Plan Mode

### Overview
Plan Mode allows Cline to:
1. Analyze requirements
2. Create detailed plan
3. Get approval before execution
4. Execute step by step

### Usage
```bash
# Enable plan mode
cline-cli task "Build a REST API" --plan-mode

# Auto-approve plan
cline-cli task "Refactor database layer" --plan-mode --auto-approve-plan
```

## VS Code Extension vs CLI

### VS Code Extension (Primary)
- Full GUI interface
- Browser automation support
- Command execution support
- Inline diff viewing
- 2.7M+ users

### CLI Version
- Terminal-based
- Scriptable
- CI/CD integration
- Limited tool support
- Under development

## Project Status

⚠️ **Important**: The CLI version is currently under development and not recommended for production use. Features may change, and stability is not guaranteed.

### Current Limitations
1. No browser automation
2. No command execution
3. Limited UI feedback
4. Experimental status

## Best Practices

### Task Description
1. **Be Specific**: Clear, detailed descriptions work best
2. **Provide Context**: Include relevant background
3. **Set Boundaries**: Specify what should/shouldn't change
4. **Review Changes**: Always review before accepting

### Safety
```bash
# Always use without auto-approve first
cline-cli task "your task"

# Review changes
git diff

# Then commit if satisfied
git commit -am "Changes by Cline"
```

### Efficiency
- Start with small tasks
- Build context incrementally
- Use plan mode for complex tasks
- Clear context when switching projects

## Troubleshooting

### Common Issues

1. **API Key Not Working**
   ```bash
   # Re-initialize
   cline-cli init
   
   # Verify key format
   echo $ANTHROPIC_API_KEY
   ```

2. **Task Fails**
   ```bash
   # Increase verbosity
   cline-cli task "..." --verbose
   
   # Check logs
   cat ~/.cline/logs/latest.log
   ```

3. **Settings Not Found**
   ```bash
   # Check settings location
   cline-cli config --show
   
   # Re-initialize
   cline-cli init --reset
   ```

## Integration Examples

### Shell Scripts
```bash
#!/bin/bash
# Automated refactoring script

cline-cli task "Update all imports to use aliases" --auto-approve
cline-cli task "Fix linting errors" --auto-approve
cline-cli task "Update tests" --auto-approve
```

### CI/CD Pipeline
```yaml
# GitHub Actions example
name: AI Code Review
on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
      - run: npm install -g @yaegaki/cline-cli
      - run: cline-cli init --settings .cline-settings.json
      - run: cline-cli task "Review this PR for issues"
```

## Comparison with Other Tools

| Feature | Cline CLI | Claude Code | Aider |
|---------|-----------|-------------|-------|
| VS Code Integration | ✅ Full | ❌ | ❌ |
| CLI Version | ✅ Limited | ✅ Full | ✅ Full |
| Browser Automation | ❌ CLI | ❌ | ❌ |
| Open Source | ✅ | ❌ | ✅ |
| Users | 2.7M+ | N/A | 35k+ |
| MCP Support | ✅ | ✅ | ❌ |

## Future Development

### Planned Features
- Full command execution support
- Browser automation in CLI
- Improved stability
- Production readiness
- Enhanced MCP integration

### Community Contributions
```bash
# Clone repository
git clone https://github.com/cline/cline
cd cline

# Install dependencies
npm install

# Run development version
npm run dev
```

## Pricing

### Cline Itself
- **Free**: Open source software
- **No subscription**: Use your own API keys

### API Costs
- Pay per token to your chosen provider
- Anthropic, OpenAI, etc. pricing applies
- No additional Cline fees

## Resources

### Documentation
- [Official Website](https://cline.bot/)
- [GitHub Repository](https://github.com/cline/cline)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cline.cline)

### Community
- [GitHub Discussions](https://github.com/cline/cline/discussions)
- [Discord Server](https://discord.gg/cline)
- [Twitter/X](https://twitter.com/cline_ai)

## MCP (Model Context Protocol) Support

### Overview
Cline has comprehensive MCP support, allowing it to extend capabilities through custom tools and integrations. Unlike other tools that only use community servers, Cline can dynamically create and install custom tools tailored to specific workflows.

### MCP Server Configuration

#### Settings File Location
`cline_mcp_settings.json` in VS Code settings directory

#### Configuration Structure
```json
{
  "mcpServers": {
    "jira-server": {
      "command": "node",
      "args": ["/path/to/jira-mcp-server.js"],
      "env": {
        "JIRA_API_KEY": "your_api_key",
        "JIRA_HOST": "your-domain.atlassian.net"
      },
      "alwaysAllow": ["fetch_tickets", "create_ticket"],
      "disabled": false,
      "timeout": 60000
    },
    "aws-server": {
      "command": "python",
      "args": ["/path/to/aws-mcp-server.py"],
      "env": {
        "AWS_ACCESS_KEY_ID": "your_key",
        "AWS_SECRET_ACCESS_KEY": "your_secret"
      }
    }
  }
}
```

### Dynamic Tool Creation

#### Creating Custom Tools
Simply ask Cline to create tools for your specific needs:

```
"Add a tool that fetches Jira tickets"
"Add a tool that manages AWS EC2 instances"
"Add a tool that pulls PagerDuty incidents"
"Add a tool that queries our production database"
```

Cline will:
1. Create a new MCP server implementation
2. Install it into the extension
3. Configure necessary environment variables
4. Make it available for future tasks

### Transport Types

#### STDIO (Local Servers)
```json
{
  "transport": "stdio",
  "command": "node",
  "args": ["./server.js"]
}
```

#### SSE (Remote Servers)
```json
{
  "transport": "sse",
  "url": "https://api.example.com/mcp/sse"
}
```

#### HTTP (REST APIs)
```json
{
  "transport": "http",
  "url": "https://api.example.com/mcp"
}
```

### Advanced MCP Features (2025)

#### Dynamic Tool Discovery
Servers can change available tools on the fly based on:
- Current project state
- Workflow progression
- Detected frameworks
- User permissions

#### Prompts and Workflows
```json
{
  "prompts": {
    "deploy": {
      "name": "Deploy to Production",
      "description": "Complete deployment workflow",
      "parameters": {
        "environment": "production",
        "runTests": true
      }
    }
  }
}
```

Access via slash commands: `/mcp.servername.promptname`

#### Sampling Capability
MCP servers can make their own LLM requests using your model subscription:

```json
{
  "sampling": {
    "enabled": true,
    "model": "inherit",  // Use user's configured model
    "maxTokens": 1000
  }
}
```

### Managing MCP Servers

#### UI Management
1. Click "MCP Servers" icon in Cline's top navigation
2. Select "Installed" tab
3. Click "Advanced MCP Settings" for configuration

#### Server Operations
- **Add**: Install from marketplace or create custom
- **Delete**: Click trash icon or "Delete Server" button
- **Restart**: Click restart button for troubleshooting
- **Disable**: Temporarily disable without removing

#### Timeout Configuration
- Default: 1 minute
- Range: 30 seconds to 1 hour
- Configure per server in settings

### Available MCP Server Categories

#### Development Tools
- Git operations
- Code formatting
- Testing frameworks
- Build systems

#### Project Management
- Jira integration
- Linear tickets
- GitHub issues
- Azure DevOps

#### Infrastructure
- AWS management
- Docker operations
- Kubernetes control
- Database queries

#### Monitoring
- PagerDuty incidents
- Datadog metrics
- CloudWatch logs
- Prometheus queries

### Creating Custom MCP Servers

#### Basic Server Template
```javascript
// mcp-server.js
const { Server } = require('@modelcontextprotocol/sdk');

const server = new Server({
  name: 'custom-server',
  version: '1.0.0'
});

server.addTool({
  name: 'fetch_data',
  description: 'Fetch data from API',
  parameters: {
    type: 'object',
    properties: {
      endpoint: { type: 'string' }
    }
  },
  handler: async (params) => {
    // Implementation
    return { data: result };
  }
});

server.start();
```

#### Python Server Example
```python
# mcp_server.py
from mcp import Server, Tool

server = Server(
    name="custom-server",
    version="1.0.0"
)

@server.tool()
async def fetch_data(endpoint: str):
    """Fetch data from API"""
    # Implementation
    return {"data": result}

if __name__ == "__main__":
    server.run()
```

### Security Configuration

#### Permission Management
```json
{
  "permissions": {
    "alwaysAllow": ["read_*"],
    "alwaysDeny": ["delete_*"],
    "requireConfirmation": ["write_*", "execute_*"]
  }
}
```

#### Environment Variables
```json
{
  "env": {
    "API_KEY": "${env:CLINE_API_KEY}",
    "DATABASE_URL": "${secrets:database_url}"
  }
}
```

### Integration with Memory Service

#### Configuration
```json
{
  "mcpServers": {
    "memory-service": {
      "command": "node",
      "args": ["./memory-mcp-bridge.js"],
      "env": {
        "MEMORY_SERVICE_URL": "http://localhost:3457",
        "AUTH_TOKEN": "${env:MEMORY_AUTH_TOKEN}"
      }
    }
  }
}
```

### Troubleshooting MCP

#### Debug Mode
```json
{
  "debug": true,
  "logLevel": "verbose",
  "logFile": "./mcp-debug.log"
}
```

#### Common Issues
1. **Server won't start**: Check command path and permissions
2. **Timeout errors**: Increase timeout in settings
3. **Authentication failures**: Verify environment variables
4. **Tool not available**: Restart server or check configuration

### Best Practices

1. **Tool Naming**: Use descriptive, action-oriented names
2. **Error Handling**: Implement comprehensive error messages
3. **Logging**: Add detailed logging for debugging
4. **Security**: Never hardcode credentials
5. **Performance**: Implement caching for expensive operations
6. **Documentation**: Document all custom tools thoroughly

## VS Code Extension Features

### Plan Mode
When enabled, Cline creates a detailed plan before executing tasks:
1. Analyzes requirements
2. Creates step-by-step plan
3. Gets user approval
4. Executes incrementally

### Diff View
- Inline diff viewing for all changes
- Accept/reject individual changes
- Side-by-side comparison mode

### Context Management
- Automatic context optimization
- Manual context clearing
- Project-specific context storage

### Keyboard Shortcuts
- `Ctrl+Shift+C`: Open Cline panel
- `Ctrl+L`: Clear chat
- `Ctrl+Enter`: Send message
- `Escape`: Cancel operation

## Links
- [Cline Website](https://cline.bot/)
- [GitHub - Main Project](https://github.com/cline/cline)
- [NPM - CLI Package](https://www.npmjs.com/package/@yaegaki/cline-cli)
- [MCP Documentation](https://docs.cline.bot/mcp)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=cline.cline)
- [MCP Specification](https://modelcontextprotocol.com)