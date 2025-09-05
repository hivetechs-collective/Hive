# Qwen Code CLI Documentation

## Overview
Qwen Code is an AI-powered command-line workflow tool for developers, specifically optimized for Qwen3-Coder models. It provides agentic coding capabilities through advanced language models, enabling developers to explore codebases, automate workflows, and enhance productivity through AI-assisted development.

**Latest Version**: Check npm for latest  
**NPM Package**: `@qwen-code/qwen-code`  
**GitHub Repository**: https://github.com/QwenLM/qwen-code  
**Model Repository**: https://github.com/QwenLM/Qwen3-Coder  
**Blog Post**: https://qwenlm.github.io/blog/qwen3-coder/

## Installation

### Prerequisites
- Node.js version 20 or higher
- Terminal or command line interface
- Authentication method (Qwen OAuth or API key)

### NPM Installation
```bash
# Global installation (recommended)
npm install -g @qwen-code/qwen-code@latest

# From source
git clone https://github.com/QwenLM/qwen-code
cd qwen-code
npm install
npm install -g .
```

## Authentication Options

### Option 1: Qwen OAuth (Recommended)
The simplest authentication method with generous free tier:
- **2,000 requests per day** with no token limits
- **60 requests per minute** rate limit
- Browser-based authentication flow
- No API key management required

To authenticate:
```bash
qwen-code auth
# Opens browser for authentication
```

### Option 2: OpenAI-Compatible API
Configure environment variables or create a `.env` file:

#### Alibaba Cloud DashScope
```bash
export OPENAI_API_KEY="your_api_key_here"
export OPENAI_BASE_URL="https://dashscope-intl.aliyuncs.com/compatible-mode/v1"
export OPENAI_MODEL="qwen3-coder-plus"
```

#### ModelScope
```bash
export OPENAI_API_KEY="your_api_key_here"
export OPENAI_BASE_URL="https://api-inference.modelscope.cn/v1"
export OPENAI_MODEL="Qwen/Qwen3-Coder"
```

#### OpenRouter
```bash
export OPENAI_API_KEY="your_api_key_here"
export OPENAI_BASE_URL="https://openrouter.ai/api/v1"
export OPENAI_MODEL="qwen/qwen3-coder"
```

#### Local Deployment
```bash
export OPENAI_BASE_URL="http://localhost:11434/v1"
export OPENAI_MODEL="qwen3-coder"
```

## Configuration

### Environment Variables
Create a `.env` file in your project root or set environment variables:

```bash
# API Configuration
OPENAI_API_KEY=your_api_key
OPENAI_BASE_URL=https://your-provider-url/v1
OPENAI_MODEL=qwen3-coder

# Optional settings
QWEN_CODE_MAX_TOKENS=100000
QWEN_CODE_TEMPERATURE=0.7
```

### Configuration File
Configuration can also be managed through the CLI:

```bash
# Set API key
qwen-code config set api-key YOUR_KEY

# Set model
qwen-code config set model qwen3-coder

# View configuration
qwen-code config show
```

## Features

### Core Capabilities
- **Codebase Exploration**: Navigate and understand large codebases
- **Code Development**: Write, refactor, and optimize code
- **Workflow Automation**: Automate repetitive development tasks
- **Debugging**: Identify and fix bugs across your codebase
- **Performance Analysis**: Analyze and improve code performance
- **Documentation Generation**: Create comprehensive documentation
- **Multi-file Operations**: Make coordinated changes across files

### Usage Examples
```bash
# Start interactive session
qwen-code

# Execute specific task
qwen-code "Analyze this Python project structure"

# Code refactoring
qwen-code "Refactor all classes to use dependency injection"

# Bug fixing
qwen-code "Find and fix all TypeScript type errors"

# Documentation
qwen-code "Generate comprehensive API documentation"

# Complex workflows
qwen-code "Implement OAuth2 authentication with refresh tokens"
```

### Session Management
```bash
# Clear session context
/clear

# Compress context to save tokens
/compress

# Show session statistics
/stats

# Exit session
/exit
```

## Memory Service Integration

### Configuration
```javascript
{
  "memoryService": {
    "endpoint": "http://localhost:3457",
    "token": "auth-token",
    "enabled": true,
    "syncInterval": 300
  }
}
```

### Integration Benefits
- Share context across sessions
- Learn from previous interactions
- Collaborate with other AI tools

## Token Management

### Token Limits
Configure token limits to manage costs and performance:

```bash
# Set maximum tokens per request
export QWEN_CODE_MAX_TOKENS=50000

# Or use command line flag
qwen-code --max-tokens 50000 "Your prompt"
```

### Best Practices
1. **Start Small**: Begin with focused file sets to minimize token usage
2. **Use Session Commands**: Leverage `/clear` and `/compress` to manage context
3. **Incremental Processing**: Break large tasks into smaller steps
4. **Monitor Usage**: Use `/stats` to track token consumption

## Advanced Features

### Command Line Options
```bash
# Specify model
qwen-code --model qwen3-coder

# Set temperature
qwen-code --temperature 0.8

# Limit tokens
qwen-code --max-tokens 50000

# Non-interactive mode
qwen-code --no-interactive "Your task"

# Verbose output
qwen-code --verbose
```

### Integration with Development Tools
Qwen Code can integrate with your existing development workflow:

```bash
# Use with git
qwen-code "Review the changes in the last commit"

# Use with testing
qwen-code "Write unit tests for the UserService class"

# Use with documentation
qwen-code "Update the README based on recent changes"
```

## Troubleshooting

### Common Issues

1. **Authentication Failed**
   ```bash
   # Re-authenticate with OAuth
   qwen-code auth
   
   # Or verify API credentials
   echo $OPENAI_API_KEY
   echo $OPENAI_BASE_URL
   ```

2. **Node.js Version Error**
   ```bash
   # Check Node.js version (must be 20+)
   node --version
   
   # Update Node.js if needed
   # Using nvm:
   nvm install 20
   nvm use 20
   ```

3. **High Token Usage**
   ```bash
   # Clear session context
   qwen-code
   > /clear
   
   # Use token limits
   qwen-code --max-tokens 30000
   ```

4. **Installation Issues**
   ```bash
   # Clear npm cache
   npm cache clean --force
   
   # Reinstall
   npm uninstall -g @qwen-code/qwen-code
   npm install -g @qwen-code/qwen-code@latest
   ```

## Platform Support

### Operating Systems
- **Windows**: Full support with Windows Terminal recommended
- **macOS**: Native support on Intel and Apple Silicon
- **Linux**: All major distributions supported

### Development Environments
- Works in any terminal or command line interface
- Compatible with VS Code integrated terminal
- Supports remote development via SSH
- Docker container compatible

## Pricing

### Qwen OAuth (Free Tier)
- **2,000 requests/day** with no token limits
- **60 requests/minute** rate limit
- No credit card required
- Ideal for individual developers

### API Provider Pricing
When using API keys, pricing depends on your provider:
- **Alibaba Cloud DashScope**: Pay-as-you-go
- **OpenRouter**: Various pricing tiers
- **ModelScope**: Free tier for research
- **Local Deployment**: No API costs

## Comparison with Other Tools

| Feature | Qwen Code | Claude Code | Aider | Cline |
|---------|-----------|-------------|-------|-------|
| Free Tier | ✅ 2000/day | ❌ | Pay-per-use | ✅ OSS |
| OAuth Auth | ✅ | ❌ | ❌ | ❌ |
| Node.js Based | ✅ | ✅ | ❌ Python | ✅ |
| Multi-file | ✅ | ✅ | ✅ | ✅ |
| Session Mgmt | ✅ | ✅ | ✅ | ✅ |

## Community & Resources

### Official Resources
- [GitHub Repository](https://github.com/QwenLM/qwen-code)
- [NPM Package](https://www.npmjs.com/package/@qwen-code/qwen-code)
- [Qwen3-Coder Models](https://github.com/QwenLM/Qwen3-Coder)
- [Blog Post](https://qwenlm.github.io/blog/qwen3-coder/)

### Model Providers
- [Alibaba Cloud Model Studio](https://modelstudio.console.aliyun.com)
- [DashScope Documentation](https://dashscope.aliyun.com)
- [ModelScope Platform](https://modelscope.cn)
- [OpenRouter](https://openrouter.ai)

### Support
- Report issues on [GitHub Issues](https://github.com/QwenLM/qwen-code/issues)
- Community discussions on GitHub
- Documentation updates via pull requests