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

## Links
- [Getting Started Guide](https://help.openai.com/en/articles/11096431-openai-codex-cli-getting-started)
- [GitHub Repository](https://github.com/openai/codex)
- [NPM Package](https://www.npmjs.com/package/@openai/codex)
- [OpenAI Platform](https://platform.openai.com)
- [ChatGPT Subscription](https://chat.openai.com/subscription)