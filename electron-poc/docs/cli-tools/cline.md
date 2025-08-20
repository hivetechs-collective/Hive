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

## Links
- [Cline Website](https://cline.bot/)
- [GitHub - Main Project](https://github.com/cline/cline)
- [NPM - CLI Package](https://www.npmjs.com/package/@yaegaki/cline-cli)
- [Blog](https://cline.bot/blog)
- [Documentation](https://cline.bot/docs)