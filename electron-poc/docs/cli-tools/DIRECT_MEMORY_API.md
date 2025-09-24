# Direct Memory API Integration

## Overview
Simple, fast, direct API access to Hive's Memory+Context system for all CLI tools.

**No MCP complexity - just HTTP calls to existing endpoints.**

## Architecture
```
CLI Tool → hive-memory command → Dynamic port discovery → Memory Service API → Memory+Context pipeline
```

## Universal Commands (All Tools)

### Basic Memory Query
```bash
hive-memory query "What have we worked on?"
hive-memory query "Show me Python examples"
```

### Enhanced Memory+Context Query (Recommended)  
```bash
hive-memory context "What have we worked on?"
hive-memory context "Show me recent Python work"
```

### Contribute Learning
```bash
hive-memory contribute solution python "Fixed import issue with relative paths"
hive-memory contribute insight debugging "Always check console first for React errors"
```

## Tool-Specific Shortcuts

### Claude Code
```bash
hive-memory-claude-code context "What have we worked on?"
```

### Gemini CLI
```bash
hive-memory-gemini-cli context "What have we worked on?"
```

### Grok
```bash
hive-memory-grok context "What have we worked on?"
```

### Qwen Code
```bash
hive-memory-qwen-code context "What have we worked on?"
```

### OpenAI Codex
```bash
hive-memory-openai-codex context "What have we worked on?"
```

### Cline
```bash
hive-memory-cline context "What have we worked on?"
```

## How It Works

1. **Dynamic Port Discovery**: Commands scan the Memory Service port range (3457-3560) to find the active service
2. **Token Authentication**: Uses existing tokens from `~/.hive/cli-tools-config.json`
3. **Direct API Calls**: HTTP POST to `/api/v1/memory/query-with-context` endpoint
4. **Memory+Context Pipeline**: Leverages the same system used by Consensus engine
   - **Memory retrieval**: Recent, today, week, and semantic layers
   - **Context building**: Pattern extraction, topic identification, summary generation
   - **Enhanced responses**: Context-aware results with relevant history

## Benefits vs MCP

✅ **No external dependencies** - uses only Node.js built-ins  
✅ **No wrapper scripts** - direct HTTP API calls  
✅ **No connection issues** - simple port scanning  
✅ **Leverages existing architecture** - uses your ProcessManager/PortManager  
✅ **Uses Memory+Context pipeline** - same quality as Consensus engine  
✅ **Dynamic port discovery** - no hardcoded ports  
✅ **Simple debugging** - just HTTP requests  
✅ **Fast and reliable** - eliminates MCP protocol overhead

## Memory Service Dashboard

All queries will now appear in the Memory Service dashboard with proper tool identification and real-time activity tracking.

## Installation

These commands are automatically available when tools are installed via Hive's AI CLI Tools section. No manual configuration required.