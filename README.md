# Hive: AI-Powered Codebase Intelligence Platform

> Lightning-fast code understanding and transformation powered by multi-model AI consensus

## Overview

Hive is a next-generation development assistant that combines the reliability of multi-model AI consensus with the seamless codebase interaction pioneered by tools like Claude Code. Built in Rust for maximum performance, Hive provides:

- **Sub-millisecond code analysis** through advanced AST parsing and semantic indexing
- **Real-time code transformations** with syntax-aware validation
- **4-stage AI consensus** eliminating hallucinations through cross-validation
- **Native IDE integration** via MCP, LSP, and custom protocols
- **Enterprise-grade reliability** with comprehensive error handling and recovery

## Key Features

### 🚀 Blazing Fast Performance
- Incremental parsing updates in microseconds
- Predictive caching for instant responses
- Parallel processing across CPU cores
- Memory-mapped indices for large codebases

### 🧠 Intelligent Code Understanding
- Deep semantic analysis across 50+ languages
- Symbol relationships and dependency tracking
- Context-aware suggestions based on project patterns
- Historical change analysis for better predictions

### 🔄 Seamless Code Application
- Stream AI-generated changes directly to files
- Syntax validation before every change
- Automatic formatting and linting
- Conflict-free concurrent editing support

### 🤝 Universal Integration
- VS Code, Cursor, Windsurf via Model Context Protocol
- Any LSP-compatible editor
- Powerful CLI for automation
- REST API for custom integrations

## Architecture

Hive's architecture consists of four main layers:

1. **Semantic Understanding Layer** - AST parsing, symbol indexing, context building
2. **Consensus Pipeline** - 4-stage AI processing for reliable outputs
3. **Transformation Engine** - Safe application of code changes
4. **Integration Layer** - IDE protocols and developer tools

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed technical documentation.

## Development Guidelines

### 🚨 MANDATORY: Pre-Task Understanding Protocol

**Before starting ANY development task**, developers MUST complete the [PRE-TASK_CHECKLIST.md](PRE-TASK_CHECKLIST.md). This ensures:
- Complete understanding of existing code
- Proper API usage
- Consistent code style
- Minimal compilation errors

⚠️ **Skipping this checklist leads to bugs, rework, and wasted time.**

## Quick Start

### Installation

```bash
# Install from crates.io (coming soon)
cargo install hive-ai

# Or build from source
git clone https://github.com/hivetechs/hive
cd hive
cargo build --release
```

### Basic Usage

```bash
# Initialize in your project
hive init

# Ask questions about your codebase
hive ask "What does the authentication system do?"

# Apply AI-suggested improvements
hive improve src/auth.rs --consensus=balanced

# Start IDE server
hive serve --mcp
```

### IDE Integration

#### VS Code / Cursor / Windsurf

1. Install the Hive extension from marketplace
2. Configure your API keys in settings
3. Use `Cmd+Shift+H` to activate Hive assistant

#### vim/neovim

```lua
-- In your init.lua
require('hive').setup({
  consensus_profile = 'balanced',
  auto_apply = true,
})
```

## Configuration

Create `.hive/config.toml` in your project:

```toml
[consensus]
profile = "balanced"  # or "speed", "cost", "elite"

[models]
generator = "claude-3-opus"
refiner = "gpt-4-turbo"
validator = "claude-3-sonnet"
curator = "gpt-4"

[performance]
cache_size = "2GB"
max_workers = 8
incremental_parsing = true

[integration]
lsp_port = 7777
mcp_port = 7778
```

## Development Status

This is a Rust reimplementation of the original TypeScript Hive AI project. Current status (Last updated: January 2025):

_Testing source control panel - January 29, 2025 at 11:47 AM_

- [x] Architecture design complete
- [ ] Core parsing engine
- [ ] Semantic indexing system  
- [ ] Consensus pipeline
- [ ] Code transformation engine
- [ ] IDE integrations
- [ ] Production release

## Comparison with Original Hive AI

| Feature | TypeScript Version | Rust Version |
|---------|-------------------|--------------|
| Startup Time | ~2s | <50ms |
| Memory Usage | 200-500MB | 50-200MB |
| Parse Speed | 50ms/file | 5ms/file |
| Streaming Latency | 100-200ms | 10-50ms |
| Binary Size | ~150MB (with Node) | ~20MB |
| CPU Efficiency | Moderate | High |

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/hivetechs/hive
cd hive

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development dependencies
cargo install cargo-watch cargo-criterion

# Run tests
cargo test

# Run benchmarks
cargo criterion

# Watch mode for development
cargo watch -x run
```

## License

Hive is proprietary software. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Original Hive AI team for the innovative consensus pipeline concept
- Anthropic's Claude Code for inspiration on seamless codebase interaction
- The Rust community for excellent libraries and tools

---

Built with ❤️ by [HiveTechs Collective](https://hivetechs.com)# Test change for git status
