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

## Links
- [Official Documentation](https://cloud.google.com/gemini/docs/codeassist/gemini-cli)
- [GitHub Repository](https://github.com/google-gemini/gemini-cli)
- [NPM Package](https://www.npmjs.com/package/@google/gemini-cli)
- [Google AI Studio](https://aistudio.google.com)
- [Gemini Code Assist](https://developers.google.com/gemini-code-assist)