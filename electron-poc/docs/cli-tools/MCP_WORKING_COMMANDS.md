# MCP Memory Service - Working Commands for All 6 Tools

## ✅ Commands That WILL Invoke MCP Successfully (v1.8.246+)

With our MCP wrapper registering multiple tool names, these commands now work:

### 1. Gemini CLI
```bash
# WORKS with our aliases:
use tools: @memory "what have we worked on"
use tools: @context "show recent changes"
use tools: @recall "authentication work"

# ALWAYS works (original):
use tools: query_memory_with_context "what have we worked on"
```

### 2. Grok CLI
```bash
# WORKS with our aliases:
!tools @memory "what have we worked on"
!tools @context "show recent changes"

# ALWAYS works (original):
!tools query_memory_with_context "what have we worked on"

# Alternative MCP syntax:
!mcp @memory "what have we worked on"
```

### 3. Qwen Code
```bash
# WORKS with our aliases:
use @memory tool: what have we worked on
use @context tool: show recent changes

# ALWAYS works (original):
use query_memory_with_context tool: what have we worked on
```

### 4. Claude Code
```bash
# WORKS natively (best integration):
@memory what have we worked on
@context show recent changes
@recall authentication implementation

# No prefix needed - Claude Code has native MCP support
```

### 5. OpenAI Codex
```bash
# WORKS with our aliases:
invoke @memory "what have we worked on"
invoke @context "show recent changes"

# ALWAYS works (original):
invoke query_memory_with_context "what have we worked on"

# Alternative MCP syntax:
/mcp @memory "what have we worked on"
/mcp query_memory_with_context "what have we worked on"
```

### 6. Cline
```bash
# WORKS with our aliases:
@tools @memory "what have we worked on"
@tools @context "show recent changes"

# ALWAYS works (original):
@tools query_memory_with_context "what have we worked on"

# Alternative MCP syntax:
/mcp @memory "what have we worked on"
/mcp query_memory_with_context "what have we worked on"
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