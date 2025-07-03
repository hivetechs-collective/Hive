# Hive AI VS Code Extension

AI-powered code analysis and assistance using HiveTechs Consensus engine.

## Features

### 🧠 AI-Powered Analysis
- **Ask Hive AI**: Get instant answers to any coding questions using multi-model consensus
- **Code Analysis**: Comprehensive analysis of your code for quality, security, and performance
- **Code Explanation**: Understand what any piece of code does with detailed AI explanations
- **Code Improvement**: Get specific, actionable suggestions to improve your code
- **Test Generation**: Automatically generate comprehensive unit tests for your functions

### 🔌 Model Context Protocol Integration
- Direct integration with Hive AI MCP server
- Real-time streaming responses with consensus progress
- Access to repository-wide intelligence and memory

### ⚡ Real-Time Features
- Instant code analysis on file open
- Context-aware suggestions based on your codebase
- AI-powered diagnostics and error explanations

## Installation

### Prerequisites
1. Install and configure Hive AI CLI:
   ```bash
   # Install Hive AI (see main documentation)
   hive config set openrouter.api_key YOUR_API_KEY
   ```

2. Start the MCP server:
   ```bash
   hive serve --mode mcp --port 7777
   ```

### Extension Installation
1. Install from VS Code Marketplace: Search for "Hive AI"
2. Or install manually:
   ```bash
   npm install
   npm run compile
   # Install .vsix package
   ```

## Usage

### Commands
- **Ctrl+Shift+H** (Cmd+Shift+H on Mac): Ask Hive AI a question
- **Ctrl+Shift+E** (Cmd+Shift+E on Mac): Explain selected code
- **Ctrl+Shift+I** (Cmd+Shift+I on Mac): Improve selected code

### Context Menu
Right-click on any selected code to access:
- Explain Code
- Improve Code  
- Generate Tests

### Command Palette
- `Hive AI: Ask Question`
- `Hive AI: Analyze with Hive AI`
- `Hive AI: Explain Code`
- `Hive AI: Improve Code`
- `Hive AI: Generate Tests`
- `Hive AI: Show Status`

## Configuration

```json
{
  "hive.mcpServerUrl": "http://127.0.0.1:7777",
  "hive.autoAnalyze": true,
  "hive.consensusProfile": "balanced",
  "hive.enableDiagnostics": true,
  "hive.enableCompletions": true,
  "hive.supportedLanguages": [".rs", ".js", ".ts", ".py", ".java"]
}
```

## Supported Languages

- Rust
- JavaScript/TypeScript
- Python
- Java
- C/C++
- Go
- PHP
- Ruby
- And more...

## Development

### Building
```bash
npm install
npm run compile
```

### Debugging
1. Open in VS Code
2. Press F5 to launch Extension Development Host
3. Test extension features

### Package
```bash
npm install -g vsce
vsce package
```

## Architecture

The extension connects to Hive AI via the Model Context Protocol (MCP):

```
VS Code Extension → MCP Client → Hive AI MCP Server → Consensus Engine
```

This provides:
- Real-time AI assistance
- Access to repository intelligence
- Streaming consensus responses
- Secure, local processing

## Troubleshooting

### MCP Server Connection Issues
1. Ensure Hive AI MCP server is running:
   ```bash
   hive serve --mode mcp --port 7777
   ```

2. Check server URL in settings:
   ```json
   "hive.mcpServerUrl": "http://127.0.0.1:7777"
   ```

3. Verify network connectivity:
   ```bash
   curl http://127.0.0.1:7777/health
   ```

### Performance Issues
- Adjust consensus profile in settings (`speed` for faster responses)
- Disable auto-analysis for large files
- Check Hive AI server resources

## License

Proprietary - HiveTechs Collective

## Support

- Documentation: https://docs.hivetechs.com
- Issues: https://github.com/hivetechs/hive/issues
- Community: https://discord.gg/hivetechs