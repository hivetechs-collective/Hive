# MCP Auto-Invoke Issue Analysis

## Problem
CLI tools are not automatically invoking MCP tools when users type `@memory` or other trigger patterns. They're treating these as regular prompts and answering from local context.

## Root Cause
Most CLI tools require **explicit MCP tool invocation** and don't automatically call MCP tools based on prompt content alone. Each tool has different requirements:

### Tool-Specific Requirements

#### Gemini CLI
- **Auto-invoke**: Not enabled by default
- **Required syntax**: 
  - `use tools: @memory what have we worked on`
  - OR configure with `autoInvoke: ['query_memory_with_context']` in settings

#### Grok CLI  
- **Auto-invoke**: Not enabled by default
- **Required syntax**:
  - `!tools query_memory_with_context "what have we worked on"`
  - OR needs explicit MCP command prefix

#### Qwen Code
- **Auto-invoke**: Not enabled by default  
- **Required syntax**:
  - `use query_memory_with_context tool: what have we worked on`

## Solution Approaches

### Option 1: Tool-Specific Configuration (Recommended)
Update each tool's configuration to enable auto-invocation:

```json
// For Gemini CLI - ~/.gemini/settings.json
{
  "mcpServers": {
    "hive-memory-service": {
      // ... existing config ...
      "autoInvoke": ["query_memory_with_context"],
      "triggerPatterns": ["@memory", "@context", "@recall", "what have we", "what did we"]
    }
  }
}
```

### Option 2: System Prompts
Configure each tool with a system prompt that instructs it to use MCP tools:

```
When user input starts with @memory, @context, @recall, or asks "what have we" or "what did we", 
automatically use the query_memory_with_context MCP tool to answer.
```

### Option 3: Wrapper Enhancement
Create an intermediary that intercepts prompts and converts them to MCP calls before they reach the tool.

## Current State
- ✅ MCP wrappers created with v2.0 features
- ✅ Tools connected to Memory Service
- ✅ Trigger patterns defined in wrappers
- ❌ Tools not auto-invoking MCP based on patterns
- ❌ Tools answering from local context instead

## Solution: Simplified Commands (v1.8.246+)

We've updated the MCP wrappers to provide tool-specific guidance. Use these simplified commands:

### Gemini CLI
```bash
# Shortest form (recommended):
/use @memory what have we worked on

# Alternative:
use tools: @memory recent work
```

### Grok CLI
```bash
# Shortest form (recommended):
!mcp @memory what have we worked on

# Alternative:
!tools query_memory_with_context "recent work"
```

### Qwen Code
```bash
# Shortest form (recommended):
/tool @memory what have we worked on

# Alternative:
use @memory tool: recent work
```

### Claude Code
```bash
# Works natively (no prefix needed):
@memory what have we worked on
```

### OpenAI Codex
```bash
# Shortest form (recommended):
/mcp @memory what have we worked on

# Alternative:
invoke memory-service query "recent work"
```

### Cline
```bash
# Shortest form (recommended):
@tools memory what have we worked on

# Alternative:
/mcp query_memory_with_context "recent work"
```

## Long-term Solution
Need to implement tool-specific configuration during installation that enables automatic MCP invocation based on our trigger patterns.