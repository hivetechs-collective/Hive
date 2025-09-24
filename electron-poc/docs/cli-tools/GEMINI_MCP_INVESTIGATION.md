# Gemini MCP Investigation - Why It's Not Working

## Current Situation
- Gemini shows "Using: - 1 MCP server" ✅
- But commands are NOT invoking the MCP server ❌
- Gemini is answering from local context instead

## Commands That DON'T Work (Tested)
- `use tools: query_memory_with_context "what have we worked on"` - Just reads files
- `use tools: @memory "what have we worked on"` - Interprets as file path
- `hive what have we been working on?` - Just uses conversation context

## What to Try Next

### 1. Check Available Commands
```bash
/help
```
Look for any MCP-related commands

### 2. List MCP Tools Explicitly
```bash
/tools
# or
/mcp list
# or
/servers
```

### 3. Direct MCP Invocation Attempts
```bash
# Try with forward slash:
/query_memory_with_context "test"

# Try with exclamation:
!query_memory_with_context "test"

# Try with mcp prefix:
/mcp:query_memory_with_context "test"

# Try with server name:
/hive-memory-service:query_memory_with_context "test"
```

### 4. Check Gemini's MCP Settings
Look at `~/.gemini/settings.json` for:
- `mcpAutoInvoke` setting
- `mcpPrefix` setting
- `toolInvocationSyntax`

## The Core Problem

Gemini CLI might:
1. **Require a specific prefix** we haven't discovered yet
2. **Need MCP auto-invoke enabled** in settings
3. **Have a different syntax** than documented
4. **Need explicit server selection** before tool use

## Verification Method

When MCP actually works:
1. Memory Service Dashboard at http://localhost:37352 will show:
   - Query count increases
   - The query appears in the list
   - Tool identifier shows "gemini-cli"

2. Gemini should show some indication like:
   - "Invoking MCP tool..."
   - "Calling external service..."
   - A loading indicator

## Possible Configuration Issue

Check if `~/.gemini/mcp-config.json` exists and contains:
```json
{
  "servers": {
    "hive-memory-service": {
      "command": "node",
      "args": ["{full-path-to-wrapper}"],
      "description": "Hive Memory Service"
    }
  }
}
```

## Alternative: Manual MCP Testing

Try to explicitly tell Gemini to use the MCP server:
```bash
# Step 1: Select the server
/server hive-memory-service

# Step 2: Call the tool
query_memory_with_context "test"

# Or try:
@hive-memory-service query_memory_with_context "test"
```

## If Nothing Works

The issue might be that Gemini CLI:
1. Doesn't support automatic MCP invocation
2. Requires manual configuration we haven't done
3. Has a bug in MCP integration
4. Uses a completely different syntax than expected

Try pressing **Ctrl+T** to see the MCP server details and look for any hints about how to invoke the tools.