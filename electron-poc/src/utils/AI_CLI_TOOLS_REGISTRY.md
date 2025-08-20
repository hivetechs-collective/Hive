# Agentic Coding CLI Tools Registry - August 2025

## Overview
This registry contains only CLI tools that function as **agentic coding assistants** - tools that can autonomously understand codebases, make multi-file changes, execute commands, and work interactively from the terminal, similar to Claude Code CLI.

## Included Agentic Coding CLIs

### 1. **Claude Code CLI** (Anthropic)
- **Package**: `@anthropic-ai/claude-code`
- **Install**: `npm install -g @anthropic-ai/claude-code`
- **Features**: Terminal-native AI agent, understands entire codebases, executes commands, makes coordinated multi-file changes
- **Model**: Claude 3.7 Sonnet
- **Pricing**: Pro $20/month, Max $100/month
- **Auth**: Requires login with paid account

### 2. **Gemini CLI** (Google)
- **Package**: `@google/gemini-cli`
- **Install**: `npm install -g @google/gemini-cli`
- **Released**: June 25, 2025
- **Features**: Open-source agentic assistant, massive 1M token context window
- **Model**: Gemini 2.5 Pro
- **Pricing**: FREE - 1000 requests/day, 60/minute
- **Auth**: Google account with free Gemini Code Assist license

### 3. **Qwen Code** (Alibaba)
- **Package**: `@qwen-code/qwen-code`
- **Install**: `npm install -g @qwen-code/qwen-code`
- **Released**: July 2025
- **Features**: Forked from Gemini CLI, optimized for Qwen3-Coder models
- **Context**: 256K native, 1M with extrapolation
- **Pricing**: Free and open-source (Apache 2.0)
- **Auth**: API key from Alibaba Cloud

### 4. **OpenAI Codex CLI**
- **Package**: `@openai/codex-cli`
- **Install**: `npm install -g @openai/codex-cli`
- **Features**: Smart terminal assistant with shell and file system access
- **Models**: GPT-4.1, o3, o4-mini
- **Pricing**: Pay-as-you-go via OpenAI API
- **Auth**: OpenAI account with credits

### 5. **Aider**
- **Package**: `aider-chat`
- **Install**: `pip install aider-chat`
- **Features**: Git-integrated inline editing, seamless version control
- **GitHub Stars**: 35.2k
- **Pricing**: Uses your API keys (OpenAI, Claude, etc.)
- **Auth**: API keys for chosen providers

### 6. **Cline**
- **Package**: `@cline/cli` (tentative)
- **Install**: `npm install -g @cline/cli`
- **Features**: Lightweight conversational file editing, multi-turn interactions
- **GitHub Stars**: 47.2k
- **Pricing**: Uses your API keys
- **Auth**: API keys for chosen providers

## Key Characteristics of Agentic Coding CLIs

These tools share common capabilities:
- ✅ **Codebase Understanding**: Can analyze and understand entire projects
- ✅ **Multi-file Operations**: Make coordinated changes across multiple files
- ✅ **Command Execution**: Run terminal commands, tests, builds
- ✅ **Interactive Sessions**: Maintain context across multiple prompts
- ✅ **File System Access**: Read, write, and modify files directly
- ✅ **Autonomous Actions**: Can work independently to complete tasks

## NOT Included (Different Tool Category)

These tools were excluded as they don't provide full agentic coding capabilities:
- **GitHub Copilot CLI**: Command suggestions, not full agentic coding
- **AIChat**: Chat interface, not autonomous coding
- **LLM Tool**: Prompt runner, not file editing
- **Qodo Gen**: Agent framework, not standalone agent
- **Offline AI CLI**: Local models interface, not coding agent

## Memory Service Integration Priority

1. **Claude Code** - PRIMARY (Direct integration planned)
2. **Gemini CLI** - HIGH (Popular, free tier, similar to Claude)
3. **Qwen Code** - HIGH (Open source, forked from Gemini)
4. **OpenAI Codex** - MEDIUM (Pay-per-use model)
5. **Aider** - LOW (Python-based, different architecture)
6. **Cline** - LOW (Different interaction model)

## Installation Requirements

### NPM-based Tools (Claude, Gemini, Qwen, OpenAI)
- Node.js 18+ required
- Global or local installation options
- Version management through npm

### Python-based Tools (Aider)
- Python 3.8+ required
- Recommend pipx for isolation
- Alternative: pip with --user flag

## Authentication Patterns

1. **Subscription-based**: Claude Code
2. **Free with Account**: Gemini CLI
3. **API Key**: Qwen Code, OpenAI Codex
4. **BYO Keys**: Aider, Cline (use your own provider keys)

## Update Strategy

- Check for updates every 24 hours
- Use package manager's version check
- Store version in sync_metadata table
- Notify user of available updates
- Optional auto-update setting

## Usage Examples

```bash
# Claude Code
claude "Fix the TypeScript errors in this project"

# Gemini CLI
gemini "Add error handling to all API endpoints"

# Qwen Code
qwen-code "Refactor this code to use async/await"

# OpenAI Codex
codex "Write tests for the authentication module"

# Aider
aider "Update the README with new API documentation"

# Cline
cline "Convert this class to use functional components"
```

All these tools provide similar agentic coding experiences, allowing developers to work with AI assistants directly from their terminal to modify code, run commands, and complete complex programming tasks autonomously.