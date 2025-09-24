# MCP Memory Service - Quick Commands Reference

## 🚀 Simplest Commands for Each Tool

### Gemini CLI
```bash
# Primary (shortest):
/use @memory what have we worked on

# Alternative:
use tools: @memory show recent work
```

### Grok CLI
```bash
# Primary (shortest):
!mcp @memory what have we worked on

# Alternative:
!tools query_memory_with_context "recent projects"
```

### Qwen Code
```bash
# Primary (shortest):
/tool @memory what have we worked on

# Alternative:
use @memory tool: show context
```

### Claude Code
```bash
# Works natively (simplest of all):
@memory what have we worked on
@context show recent changes
@recall authentication implementation
```

### OpenAI Codex
```bash
# Primary (shortest):
/mcp @memory what have we worked on

# Alternative:
invoke memory-service query "recent work"
```

### Cline
```bash
# Primary (shortest):
@tools memory what have we worked on

# Alternative:
/mcp query_memory_with_context "show context"
```

## 💡 Universal Patterns That Trigger Memory

These natural language patterns work with all tools (when using the commands above):
- "What have we worked on?"
- "What did we discuss about [topic]?"
- "Show me our recent projects"
- "Remember when we implemented [feature]?"
- "Retrieve context about [topic]"

## 🎯 One-Line Setup Test

After installing any tool through Hive IDE, test with:

```bash
# Gemini CLI
gemini "/use @memory test"

# Grok CLI
grok "!mcp @memory test"

# Qwen Code
qwen "/tool @memory test"

# Claude Code
claude "@memory test"

# OpenAI Codex
codex "/mcp @memory test"

# Cline
cline "@tools memory test"
```

## ⚡ Quick Tips

1. **Claude Code** is the only tool that works with `@memory` directly without prefixes
2. **Gemini CLI** uses `/use` as the shortest prefix
3. **Grok CLI** uses `!mcp` for quick access
4. **Qwen Code** uses `/tool` for MCP commands
5. **OpenAI Codex** uses `/mcp` similar to Qwen
6. **Cline** uses `@tools` for all tool access

## 🔧 Troubleshooting

If a command doesn't work:
1. Check Memory Service is running: `http://localhost:37352/health`
2. Verify MCP configuration exists for the tool
3. Try the alternative command format
4. Use the tool's help command to see available MCP tools

## 📝 Help Commands

Get tool-specific help:
- Gemini: `/help mcp`
- Grok: `!help tools`
- Qwen: `/help tools`
- Claude: `help @memory`
- Codex: `/help mcp`
- Cline: `@tools help`