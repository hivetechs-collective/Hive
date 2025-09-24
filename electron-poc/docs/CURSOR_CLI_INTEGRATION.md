# Cursor CLI Integration Documentation

## Overview
Cursor CLI (`cursor-agent`) is an interactive terminal AI coding assistant currently in beta. It provides both interactive and non-interactive modes for AI-powered code assistance directly from the command line.

## Installation

### Method 1: Direct Installation (Recommended)
```bash
curl https://cursor.com/install -fsS | bash
```

This script installs the `cursor-agent` binary into a user-writable location (usually `~/.local/bin`).

### Method 2: Manual Installation
Download the appropriate binary for your platform from the Cursor website and add it to your PATH.

### Updating Cursor CLI
Cursor currently ships updates through the same installer script. To update, re-run the installer and then confirm the version:

```bash
curl https://cursor.com/install -fsS | bash
cursor-agent --version
```

### Uninstalling Cursor CLI
Remove the installed binary and the optional state directory:

```bash
rm -f ~/.local/bin/cursor-agent /usr/local/bin/cursor-agent /opt/homebrew/bin/cursor-agent
rm -rf ~/.cursor-agent
```

Adjust the paths above if you placed the binary elsewhere.

### Hive Memory MCP Integration
Hive automatically manages Cursor's MCP configuration so the CLI can reach our unified memory service out of the box:

- Global config lives at `~/.cursor/mcp.json`
- We inject the `hive-memory-service` entry with the current dynamic port and auth token
- Tool aliases (`@memory`, `@context`, `@recall`) and trigger patterns are preconfigured
- Auto-invoke is enabled for `query_memory_with_context` so Cursor can pull context without extra setup

If the memory service port changes (for example after app restart), Hive rewrites the MCP config on the next install, configure, or launch action to keep the endpoint current.

## Key Features

1. **Interactive Mode**: Start an AI coding session with continuous conversation
2. **Non-Interactive Mode**: Run one-off commands and prompts
3. **Session Management**: Save and resume conversations
4. **Model Selection**: Choose different AI models for responses
5. **Multiple Output Formats**: JSON, text, markdown support
6. **Git Integration**: Review changes and commits
7. **Code Context**: Automatically understands project structure

## Command Reference

### Basic Usage

#### Start Interactive Session
```bash
cursor-agent
```

#### Start with Initial Prompt
```bash
cursor-agent "refactor the auth module to use JWT tokens"
```

### Non-Interactive Commands

#### Run Specific Prompt
```bash
cursor-agent -p "find and fix performance issues in the database module"
```

#### Review Git Changes
```bash
cursor-agent -p "review these changes for security issues" --output-format text
```

#### Use Specific Model
```bash
cursor-agent -p "generate unit tests" --model "gpt-5"
```

### Session Management

#### List All Conversations
```bash
cursor-agent ls
```

#### Resume Latest Conversation
```bash
cursor-agent resume
```

#### Resume Specific Conversation
```bash
cursor-agent --resume="chat-id-here"
```

## Integration with Hive Consensus

### Detection
The tool is detected by checking for the `cursor-agent` command in PATH:
```bash
which cursor-agent
```

### Version Check
```bash
cursor-agent --version
```

### Launch Command
```bash
cursor-agent
```

## Configuration

Currently in beta, Cursor CLI doesn't require specific configuration files. Future versions may support:
- Custom model preferences
- API key configuration
- Output format defaults
- Project-specific settings

## Use Cases

### 1. Code Refactoring
```bash
cursor-agent "refactor this function to be more efficient"
```

### 2. Bug Fixing
```bash
cursor-agent -p "find potential bugs in the auth service"
```

### 3. Code Review
```bash
git diff | cursor-agent -p "review these changes"
```

### 4. Documentation Generation
```bash
cursor-agent "generate documentation for the API endpoints"
```

### 5. Test Generation
```bash
cursor-agent "write unit tests for the UserService class"
```

## Beta Notes

- Currently in beta, actively seeking user feedback
- Features and commands may change
- Report issues through the Cursor feedback channel
- Performance and accuracy improvements ongoing
- Memory Service integration is automatically configured via MCP; no manual tokens or endpoints are required

## Comparison with Other Tools

| Feature | Cursor CLI | Claude Code | GitHub Copilot |
|---------|-----------|-------------|----------------|
| Interactive Mode | ✅ | ✅ | ✅ |
| Session Resume | ✅ | ✅ | ❌ |
| Model Selection | ✅ | ❌ | ❌ |
| Git Integration | ✅ | ✅ | ✅ |
| MCP Support | ✅ | ✅ | ❌ |
| Output Formats | Multiple | Text | Text |

## Troubleshooting

### Installation Issues
- Ensure you have proper permissions for installation
- Check that curl and bash are available
- Verify network connectivity to cursor.com

### Command Not Found
- Check PATH includes installation directory
- Run `source ~/.bashrc` or `source ~/.zshrc` after installation
- Verify installation with `which cursor-agent`

### Session Issues
- Sessions are stored locally
- Check disk space for session storage
- Use `cursor-agent ls` to verify saved sessions

## Future Enhancements

- Configuration file support
- Custom prompt templates
- IDE integration improvements
- Enhanced context awareness
- Team collaboration features

## Resources

- Official Documentation: https://docs.cursor.com/en/cli/overview
- Cursor Website: https://cursor.com
- Support: Available through Cursor platform

---

*Last Updated: December 2024*
*Status: Beta - Active Development*
