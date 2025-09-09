# MCP Memory Service - ACTUAL Working Commands

## ✅ Commands That WILL Invoke MCP Successfully

### Gemini CLI
```bash
# This WORKS:
use tools: query_memory_with_context "what have we worked on"

# This does NOT work yet:
/use @memory what have we worked on  # (would need alias configuration)
```

### Grok CLI
```bash
# This WORKS:
!tools query_memory_with_context "what have we worked on"

# This MAY work (depends on Grok's MCP implementation):
!mcp query_memory_with_context "what have we worked on"
```

### Qwen Code
```bash
# This WORKS:
use query_memory_with_context tool: what have we worked on

# This does NOT work yet:
/tool @memory what have we worked on  # (would need alias configuration)
```

### Claude Code
```bash
# This WORKS (natively):
@memory what have we worked on
```

### OpenAI Codex
```bash
# This SHOULD work:
invoke memory-service query_memory_with_context "what have we worked on"

# Simplified version needs testing:
/mcp query_memory_with_context "what have we worked on"
```

### Cline
```bash
# This SHOULD work:
/mcp query_memory_with_context "what have we worked on"

# This needs testing:
@tools query_memory_with_context "what have we worked on"
```

## 🔧 What We Need to Fix

To make the simplified `@memory` commands work, we need to:

### Option 1: Tool Alias Configuration
Configure each tool to recognize `@memory` as an alias for `query_memory_with_context`:

```json
// Example for Gemini CLI
{
  "mcpServers": {
    "hive-memory-service": {
      "toolAliases": {
        "@memory": "query_memory_with_context",
        "@context": "query_memory_with_context",
        "@recall": "query_memory_with_context"
      }
    }
  }
}
```

### Option 2: Wrapper Tool Names
Modify our MCP wrapper to register tools with multiple names:
- `query_memory_with_context` (for compatibility)
- `@memory` (for simplified access)
- `memory` (as fallback)

### Option 3: Command Preprocessor
Add a preprocessor that converts simplified commands to full MCP invocations before they reach the tool.

## 📝 Current Working Solution

For now, users should use these **guaranteed working** commands:

1. **Gemini**: `use tools: query_memory_with_context "your question"`
2. **Grok**: `!tools query_memory_with_context "your question"`
3. **Qwen**: `use query_memory_with_context tool: your question`
4. **Claude**: `@memory your question` (works as-is)
5. **Codex**: `invoke memory-service query_memory_with_context "your question"`
6. **Cline**: `/mcp query_memory_with_context "your question"`

These are verbose but **will definitely trigger the MCP Memory Service**.

## 🚀 Next Steps

1. Test each tool's MCP implementation to verify exact syntax
2. Implement tool aliases or multiple tool name registration
3. Create command preprocessors for simplified syntax
4. Update documentation with verified working commands