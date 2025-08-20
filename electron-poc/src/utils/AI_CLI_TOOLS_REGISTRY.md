# AI CLI Tools Registry - August 2025

## Complete List of Available AI/LLM CLI Tools

### Tier 1: Major Provider CLIs

1. **Claude Code CLI** (Anthropic)
   - Package: `@anthropic-ai/claude-code` 
   - Install: `npm install -g @anthropic-ai/claude-code`
   - Features: Terminal-native AI agent, Claude 3.7 Sonnet
   - Pricing: Pro $20/month, Max $100/month
   - Context: Standard Claude context windows
   - Auth: Requires login with paid account

2. **Gemini CLI** (Google)
   - Package: `@google/gemini-cli`
   - Install: `npm install -g @google/gemini-cli`
   - Released: June 25, 2025
   - Features: Open-source, 1M token context window
   - Pricing: FREE - 1000 requests/day, 60/minute
   - Model: Gemini 2.5 Pro
   - Auth: Google account with free Gemini Code Assist license

3. **Qwen Code** (Alibaba)
   - Package: `@qwen-code/qwen-code`
   - Install: `npm install -g @qwen-code/qwen-code`
   - Released: July 2025
   - Features: Forked from Gemini CLI, optimized for Qwen3-Coder
   - Pricing: Free and open-source (Apache 2.0)
   - Context: 256K native, 1M with extrapolation
   - Auth: API key from Alibaba Cloud

4. **OpenAI Codex CLI**
   - Package: `@openai/codex-cli`
   - Install: `npm install -g @openai/codex-cli`
   - Features: Access to GPT-4.1, o3, o4-mini models
   - Pricing: Pay-as-you-go via OpenAI API
   - Auth: OpenAI account with credits

5. **GitHub Copilot CLI**
   - Extension: `github/gh-copilot`
   - Install: `gh extension install github/gh-copilot`
   - Requires: GitHub CLI (`brew install gh`)
   - Pricing: Part of GitHub Copilot subscription
   - Auth: GitHub account with Copilot license

### Tier 2: Multi-Model CLI Tools

6. **AIChat** (All-in-One)
   - Package: `aichat`
   - Install: `cargo install aichat` or `brew install aichat`
   - Supports: 20+ providers (OpenAI, Claude, Gemini, Ollama, Groq, Mistral, Perplexity, Qwen, etc.)
   - Features: Unified interface for all models
   - Auth: Individual API keys per provider

7. **LLM Tool** (Simon Willison)
   - Package: `llm`
   - Install: `pip install llm` or `brew install llm`
   - Features: Plugin-based architecture
   - Supports: OpenAI, Anthropic, Gemini, Mistral, Ollama, GitHub Models
   - Auth: API keys via `llm keys set <provider>`

8. **Qodo Gen CLI**
   - Package: `@qodo/gen`
   - Install: `npm install -g @qodo/gen`
   - Released: June 25, 2025
   - Features: Agent framework for custom AI coding agents
   - Auth: Qodo account

### Tier 3: Specialized/Alternative CLIs

9. **Cline**
   - GitHub Stars: 47.2k
   - Install: Via GitHub releases
   - Features: Lightweight conversational file editing
   - Model Support: Multiple via API keys

10. **Aider**
    - GitHub Stars: 35.2k
    - Install: `pip install aider-chat`
    - Features: Git-integrated inline editing
    - Model Support: OpenAI, Claude, others

11. **Offline AI CLI**
    - Package: `@offline-ai/cli`
    - Install: `npm install -g @offline-ai/cli`
    - Features: Local model support
    - Models: LLaMA, Qwen, Gemma, Phi, GLM, Mistral

### Tier 4: Provider-Specific Access (via AIChat/LLM)

12. **Mistral** - Available through AIChat or LLM plugins
13. **Groq** - Available through AIChat
14. **Perplexity** - Available through AIChat
15. **Cohere** - Available through AIChat
16. **DeepSeek** - Available through AIChat
17. **XAI Grok** - Available through AIChat

## Installation Matrix

| Tool | Method | Package Manager | Requires |
|------|--------|----------------|----------|
| Claude Code | NPM | npm/yarn | Node.js 18+ |
| Gemini CLI | NPM | npm/yarn | Node.js 18+ |
| Qwen Code | NPM | npm/yarn | Node.js 18+ |
| OpenAI Codex | NPM | npm/yarn | Node.js 18+ |
| GitHub Copilot | GH Extension | gh cli | GitHub CLI |
| AIChat | Binary/Cargo | cargo/brew | Rust/Homebrew |
| LLM Tool | Python | pip/pipx | Python 3.8+ |
| Qodo Gen | NPM | npm/yarn | Node.js 18+ |
| Aider | Python | pip/pipx | Python 3.8+ |
| Offline AI | NPM | npm/yarn | Node.js 18+ |

## Memory Service Integration Priority

1. **Claude Code** - PRIMARY (Direct integration planned)
2. **Gemini CLI** - HIGH (Popular, free tier)
3. **Qwen Code** - HIGH (Open source, powerful)
4. **OpenAI Codex** - MEDIUM
5. **AIChat** - MEDIUM (Multi-model support)
6. **LLM Tool** - MEDIUM (Extensible)

## Implementation Notes

### NPM-based Tools
- Can be installed globally or locally
- Local installation to `~/.hive/tools/node_modules` for sandboxing
- Version management through npm

### Python-based Tools
- Recommend pipx for isolation
- Alternative: pip with --user flag
- Version management through pip/pipx

### Binary Tools
- Direct download and extraction
- Checksum verification recommended
- Auto-update via GitHub releases API

### Extension-based Tools
- Require base tool (e.g., gh for GitHub Copilot)
- Installation via base tool's extension manager
- Updates managed by base tool

## Authentication Patterns

1. **API Key**: OpenAI, Anthropic, Mistral, Groq
2. **OAuth**: GitHub, Google
3. **Cloud Account**: Alibaba (Qwen), Google (Gemini)
4. **Subscription**: Claude Code, GitHub Copilot
5. **Free/Open**: Offline AI, local models

## Update Strategy

- Check for updates every 24 hours
- Use package manager's version check
- Store version in sync_metadata table
- Notify user of available updates
- Optional auto-update setting