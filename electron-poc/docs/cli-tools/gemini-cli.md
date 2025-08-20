# Gemini CLI Documentation

## Overview
Gemini CLI is Google's open-source AI agent that brings the power of Gemini directly into your terminal. It provides lightweight access to Gemini 2.5 Pro with a massive 1M token context window, built for versatile tasks from coding to research.

**Latest Version**: 0.1.18+ (as of August 2025)  
**NPM Package**: `@google/gemini-cli`  
**GitHub**: https://github.com/google-gemini/gemini-cli  
**Documentation**: https://cloud.google.com/gemini/docs/codeassist/gemini-cli

## Installation

### Prerequisites
- Node.js version ≥ 18
- Terminal or command line interface
- Google account (for free tier access)

### NPM Installation
```bash
# Global installation (recommended)
npm install -g @google/gemini-cli

# Quick test without installation
npx https://github.com/google-gemini/gemini-cli
```

## Authentication

### Method 1: Free Tier with Google Account
1. Login with personal Google account
2. Get free Gemini Code Assist license
3. Access to Gemini 2.5 Pro (1M token context)
4. **Free Limits**:
   - 60 requests per minute
   - 1,000 requests per day
   - No charge

### Method 2: API Key
```bash
# Generate key from Google AI Studio
export GOOGLE_GENAI_API_KEY="your-super-secret-api-key"
```

## Configuration

### Initial Setup
```bash
# Start Gemini CLI
gemini

# For GitHub Actions setup
gemini /setup-github
```

### Configuration File
Location: `~/.gemini/config.json`

```json
{
  "model": "gemini-2.5-pro",
  "contextWindow": 1000000,
  "apiKey": "your-api-key",
  "temperature": 0.7
}
```

## Features

### Core Capabilities
- **1M Token Context Window**: Handle massive codebases
- **Google Search Grounding**: Real-time information access
- **File Operations**: Read, write, modify files
- **Shell Commands**: Execute terminal commands
- **Web Fetching**: Retrieve and analyze web content
- **MCP Support**: Extensible with Model Context Protocol

### Built-in Tools
1. **Google Search**: Ground responses with current information
2. **File System**: Complete file manipulation
3. **Shell Execution**: Run commands safely
4. **Web Browser**: Fetch and parse web pages
5. **Code Analysis**: Deep codebase understanding

### Usage Examples
```bash
# Basic usage
gemini "Explain this codebase structure"

# Code generation
gemini "Create a REST API with Express"

# File operations
gemini "Refactor all TypeScript files to use async/await"

# Research tasks
gemini "Find the latest React best practices and update my code"

# Complex workflows
gemini "Analyze security vulnerabilities and fix them"
```

## Memory Service Integration

### Connection Configuration
```javascript
{
  "memoryService": {
    "endpoint": "http://localhost:3457",
    "token": "auth-token",
    "autoSync": true
  }
}
```

### Integration Points
- Query historical context
- Share learnings across sessions
- Collaborate with other tools

## MCP (Model Context Protocol) Support

### Adding MCP Servers
```json
{
  "mcpServers": {
    "database": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-sqlite", "path/to/db.sqlite"]
    },
    "github": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-github"]
    }
  }
}
```

## GitHub Actions Integration

### Setup
```bash
# Initialize GitHub Actions
gemini /setup-github
```

### Using in Workflows
```yaml
name: AI-Powered CI
on: [push, pull_request]

jobs:
  ai-review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: google-github-actions/run-gemini-cli@v1
        with:
          prompt: "Review this PR for security issues"
```

## Advanced Features

### Custom Tools
```javascript
// Adding custom tools via MCP
{
  "tools": {
    "customAnalyzer": {
      "description": "Custom code analyzer",
      "command": "node",
      "args": ["./tools/analyzer.js"]
    }
  }
}
```

### Prompt Templates
```bash
# Using templates
gemini --template security-audit

# Custom templates
gemini --template ./my-templates/refactor.md
```

## Free Tier Benefits

### Included in Free Tier
- **Model**: Gemini 2.5 Pro
- **Context**: 1,000,000 tokens
- **Requests**: 1,000/day, 60/minute
- **Features**: All core capabilities
- **Support**: Community support

### When to Upgrade
- Need more than 1,000 requests/day
- Require priority processing
- Want dedicated support
- Need SLA guarantees

## System Requirements
- **OS**: macOS, Linux, Windows (WSL)
- **RAM**: 2GB minimum
- **Disk**: 200MB for installation
- **Network**: Internet required

## Troubleshooting

### Common Issues

1. **Authentication Error**
   ```bash
   # Re-authenticate
   gemini auth logout
   gemini auth login
   ```

2. **Rate Limiting**
   ```bash
   # Check usage
   gemini usage
   
   # Wait or upgrade for more requests
   ```

3. **Context Overflow**
   ```bash
   # Clear context
   gemini clear
   
   # Use file filtering
   gemini --exclude "node_modules,dist"
   ```

## API Integration

### Programmatic Access
```javascript
import { GeminiCLI } from '@google/gemini-cli';

const gemini = new GeminiCLI({
  apiKey: process.env.GOOGLE_GENAI_API_KEY,
  model: 'gemini-2.5-pro'
});

const response = await gemini.prompt({
  text: "Analyze this code",
  context: "./src"
});
```

### REST API
```bash
curl -X POST https://generativelanguage.googleapis.com/v1/models/gemini-2.5-pro:generateContent \
  -H "Authorization: Bearer $GOOGLE_GENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Your prompt here"}'
```

## Best Practices

1. **Context Management**: Use the full 1M context wisely
2. **Rate Limiting**: Batch requests when possible
3. **Caching**: Enable response caching for efficiency
4. **Security**: Never commit API keys
5. **Prompt Engineering**: Be specific and structured

## Comparison with Other Tools

| Feature | Gemini CLI | Claude Code | Others |
|---------|------------|-------------|--------|
| Free Tier | ✅ 1000/day | ❌ Paid only | Varies |
| Context | 1M tokens | 200K tokens | <100K |
| Google Search | ✅ Built-in | ❌ | ❌ |
| Open Source | ✅ | ❌ | Some |
| MCP Support | ✅ | ✅ | Limited |

## MCP (Model Context Protocol) Support

### Overview
Gemini CLI has comprehensive MCP support from its initial release, allowing developers to extend capabilities through custom tools and integrations. MCP servers act as bridges between Gemini and your local environment or external services.

### MCP Server Configuration

#### Settings File Location
`~/.gemini/settings.json` or project-specific `.gemini/settings.json`

#### Configuration Structure
```json
{
  "mcpServers": {
    "database-server": {
      "command": "python",
      "args": ["-m", "database_mcp_server", "--port", "8080"],
      "cwd": "./mcp_tools/python",
      "env": {
        "DATABASE_URL": "$DB_URL_FROM_ENV",
        "API_KEY": "${env:API_KEY}"
      },
      "timeout": 15000,
      "trust": false,
      "includeTools": ["query", "update"],
      "excludeTools": ["delete", "drop"]
    },
    "github-server": {
      "command": "node",
      "args": ["./github-mcp-server.js"],
      "env": {
        "GITHUB_TOKEN": "${env:GITHUB_TOKEN}"
      }
    }
  }
}
```

### MCP Management Commands
```bash
# Add a new MCP server
gemini mcp add <server-name> --command "node server.js"

# List all configured servers
gemini mcp list

# Remove a server
gemini mcp remove <server-name>

# Check server status
/mcp

# Get detailed tool descriptions
/mcp desc
```

### Transport Types

#### STDIO (Local Servers)
```json
{
  "transport": "stdio",
  "command": "python",
  "args": ["./server.py"]
}
```

#### HTTP (Remote Servers)
```json
{
  "transport": "http",
  "url": "https://api.example.com/mcp",
  "headers": {
    "Authorization": "Bearer ${env:TOKEN}"
  }
}
```

#### SSE (Server-Sent Events)
```json
{
  "transport": "sse",
  "url": "https://api.example.com/mcp/sse"
}
```

## Extensions System

### Overview
Gemini CLI supports a powerful extensions system that bundles MCP servers with configurations and context files to create reusable toolsets.

### Extension Structure
```
.gemini/extensions/
└── my-extension/
    ├── gemini-extension.json
    ├── GEMINI.md
    ├── servers/
    │   ├── tool-server.js
    │   └── api-server.py
    └── configs/
        └── default.json
```

### Extension Configuration
```json
{
  "name": "database-extension",
  "version": "1.0.0",
  "description": "Database management tools",
  "mcpServers": {
    "postgres": {
      "command": "node",
      "args": ["./servers/postgres-mcp.js"],
      "env": {
        "PG_CONNECTION": "${env:DATABASE_URL}"
      }
    },
    "migrations": {
      "command": "python",
      "args": ["./servers/migration-tool.py"]
    }
  },
  "contextFileName": "GEMINI.md",
  "includeTools": ["query", "migrate"],
  "excludeTools": ["drop_database"],
  "settings": {
    "autoMigrate": false,
    "queryTimeout": 30000
  }
}
```

### Extension Locations
- **Project-specific**: `<workspace>/.gemini/extensions/`
- **User-global**: `~/.gemini/extensions/`
- **System-wide**: `/usr/local/share/gemini/extensions/`

### Installing Extensions
```bash
# Install from npm
npm install -g @gemini/database-extension

# Install from GitHub
gemini extension install github:org/extension-name

# Install locally
gemini extension link ./my-extension
```

## Custom Tool Development

### Creating an MCP Server

#### JavaScript/Node.js Example
```javascript
// database-mcp-server.js
const { Server } = require('@modelcontextprotocol/sdk');

const server = new Server({
  name: 'database-server',
  version: '1.0.0',
  description: 'Database operations MCP server'
});

// Add a query tool
server.addTool({
  name: 'query_database',
  description: 'Execute a SQL query',
  parameters: {
    type: 'object',
    properties: {
      query: {
        type: 'string',
        description: 'SQL query to execute'
      },
      database: {
        type: 'string',
        description: 'Database name'
      }
    },
    required: ['query']
  },
  handler: async (params) => {
    // Implementation
    const result = await executeQuery(params.query);
    return {
      type: 'text',
      text: JSON.stringify(result, null, 2)
    };
  }
});

// Start the server
server.start();
```

#### Python Example
```python
# api_mcp_server.py
from mcp import Server, Tool
import asyncio

server = Server(
    name="api-server",
    version="1.0.0",
    description="API operations MCP server"
)

@server.tool(
    description="Fetch data from API",
    parameters={
        "type": "object",
        "properties": {
            "endpoint": {"type": "string"},
            "method": {"type": "string", "enum": ["GET", "POST"]},
            "body": {"type": "object"}
        },
        "required": ["endpoint", "method"]
    }
)
async def api_request(endpoint: str, method: str, body=None):
    """Execute API request"""
    # Implementation
    result = await make_request(endpoint, method, body)
    return {"data": result}

if __name__ == "__main__":
    asyncio.run(server.run())
```

### Rich Content Support

#### Returning Multiple Content Types
```javascript
server.addTool({
  name: 'analyze_data',
  handler: async (params) => {
    return {
      content: [
        {
          type: 'text',
          text: 'Analysis complete. See visualization below:'
        },
        {
          type: 'image',
          data: base64EncodedImage,
          mimeType: 'image/png'
        },
        {
          type: 'text',
          text: 'Statistical summary:\n' + summaryText
        }
      ]
    };
  }
});
```

### Prompts as Slash Commands
```javascript
server.addPrompt({
  name: 'deploy',
  description: 'Deploy to production',
  template: 'Deploy the current branch to production with tests',
  parameters: {
    environment: 'production',
    runTests: true
  }
});

// Available in CLI as: /deploy
```

## Advanced Configuration

### Security Settings
```json
{
  "security": {
    "trustMode": "selective",  // "all", "none", "selective"
    "trustedServers": ["database-server", "api-server"],
    "requireConfirmation": ["delete_*", "drop_*"],
    "sandboxMode": true,
    "allowedHosts": ["api.company.com", "db.internal"]
  }
}
```

### Performance Optimization
```json
{
  "performance": {
    "maxConcurrentServers": 5,
    "serverTimeout": 30000,
    "cacheResults": true,
    "cacheTTL": 300,
    "batchRequests": true
  }
}
```

### OAuth 2.0 Configuration
```json
{
  "oauth": {
    "providers": {
      "google": {
        "clientId": "${env:GOOGLE_CLIENT_ID}",
        "clientSecret": "${env:GOOGLE_CLIENT_SECRET}",
        "scopes": ["drive.readonly", "sheets"]
      },
      "github": {
        "clientId": "${env:GITHUB_CLIENT_ID}",
        "clientSecret": "${env:GITHUB_CLIENT_SECRET}",
        "scopes": ["repo", "read:org"]
      }
    }
  }
}
```

## Built-in Tools

### File Operations
- `read_file`: Read file contents
- `write_file`: Write or create files
- `edit_file`: Modify existing files
- `list_files`: List directory contents

### Terminal Operations
- `run_shell_command`: Execute shell commands
- `grep`: Search file contents
- `find`: Find files by pattern

### Web Operations
- `web_search`: Search the web
- `web_fetch`: Fetch web page content

### System Operations
- `/memory`: Memory management
- `/stats`: Usage statistics
- `/tools`: List available tools
- `/mcp`: MCP server management

## Yolo Mode

### Overview
Yolo mode enables rapid, unconfirmed actions for experienced users.

### Activation
```bash
# Enable for session
gemini --yolo

# Enable in config
{
  "yoloMode": true,
  "yoloExclusions": ["delete", "drop", "rm"]
}
```

### Safety Settings
```json
{
  "yolo": {
    "enabled": false,
    "confirmDangerous": true,
    "excludePatterns": ["*delete*", "*remove*"],
    "maxFileSize": 1048576,  // 1MB
    "allowedCommands": ["ls", "cat", "grep"]
  }
}
```

## Memory Service Integration

### Configuration
```json
{
  "memoryService": {
    "enabled": true,
    "endpoint": "http://localhost:3457",
    "authToken": "${env:MEMORY_TOKEN}",
    "syncInterval": 300,
    "autoConnect": true
  }
}
```

### Memory Commands
```bash
# Check memory status
/memory status

# Clear memory
/memory clear

# Export memory
/memory export ./memory-backup.json

# Import memory
/memory import ./memory-backup.json
```

## Cloud Shell Integration

### Automatic Setup
Gemini CLI is pre-installed in Google Cloud Shell with automatic authentication.

### Cloud-Specific Features
```json
{
  "cloudIntegration": {
    "projectId": "${env:GOOGLE_CLOUD_PROJECT}",
    "region": "us-central1",
    "services": ["compute", "storage", "bigquery"],
    "autoAuth": true
  }
}
```

## React Loop Architecture

### Overview
Gemini CLI uses a Reason and Act (ReAct) loop to process tasks:

1. **Reasoning**: Analyzes the task and available tools
2. **Planning**: Creates a step-by-step approach
3. **Acting**: Executes tools and commands
4. **Observing**: Evaluates results
5. **Iterating**: Continues until task completion

### Configuration
```json
{
  "react": {
    "maxIterations": 10,
    "reasoningModel": "gemini-2-5-pro",
    "planningDepth": 3,
    "autoRetry": true,
    "verboseReasoning": false
  }
}
```

## Troubleshooting

### Debug Mode
```bash
# Enable debug output
gemini --debug

# Verbose logging
gemini --verbose

# Log to file
gemini --log-file ./gemini.log
```

### Common Issues

1. **MCP Server Connection Failed**
   ```bash
   # Check server status
   gemini mcp list --status
   
   # Test server directly
   gemini mcp test <server-name>
   
   # View server logs
   gemini mcp logs <server-name>
   ```

2. **Extension Not Loading**
   ```bash
   # Verify extension
   gemini extension validate ./my-extension
   
   # Check extension registry
   gemini extension list --verbose
   
   # Reload extensions
   gemini extension reload
   ```

3. **Tool Not Available**
   ```bash
   # List all tools
   /tools
   
   # Check tool permissions
   /mcp desc <tool-name>
   
   # Refresh tool cache
   gemini cache clear tools
   ```

## Best Practices

1. **Security First**: Never use `trust: true` for untrusted servers
2. **Tool Isolation**: Use `includeTools`/`excludeTools` for access control
3. **Environment Variables**: Use `${env:}` syntax for secrets
4. **Extension Bundling**: Create reusable extensions for team workflows
5. **Performance**: Enable caching for expensive operations
6. **Monitoring**: Use `/stats` to track usage and performance
7. **Version Control**: Commit `.gemini/settings.json` for team consistency

## Links
- [Official Documentation](https://cloud.google.com/gemini/docs/codeassist/gemini-cli)
- [GitHub Repository](https://github.com/google-gemini/gemini-cli)
- [NPM Package](https://www.npmjs.com/package/@google/gemini-cli)
- [MCP Documentation](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md)
- [Extensions Guide](https://github.com/google-gemini/gemini-cli/blob/main/docs/extensions.md)
- [Google AI Studio](https://aistudio.google.com)
- [Google Cloud Console](https://console.cloud.google.com)
- [Gemini Code Assist](https://developers.google.com/gemini-code-assist)