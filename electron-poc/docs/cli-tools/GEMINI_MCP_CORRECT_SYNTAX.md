# Correct MCP Syntax for Gemini CLI

## Issue
Gemini is interpreting `@memory` as a file path, not as an MCP tool name.

## Correct Commands for Gemini CLI

### To Query Memory:
```bash
# Use the full tool name:
use tools: query_memory_with_context "what have we worked on"

# Or if the tool is registered as @memory:
use tools: "query_memory_with_context" "what have we worked on"

# Alternative MCP invocation:
/use query_memory_with_context "what have we worked on"
```

### To Save Memory:
```bash
# Use the save_memory tool:
use tools: save_memory "content: Fixed bug, type: solution"

# Alternative:
/use save_memory "Fixed authentication bug" "bug_fix"
```

## Testing MCP Connection

1. **List available MCP tools**:
   ```bash
   # Press Ctrl+T to view available MCP servers and tools
   ```

2. **Check what tools are available**:
   The MCP server should show tools like:
   - `query_memory_with_context`
   - `save_memory`
   - `@memory` (if alias is registered)
   - `@context` (if alias is registered)

## Troubleshooting

If `@memory` is being interpreted as a file path:
1. The tool aliases might not be registered properly
2. Use the full tool name: `query_memory_with_context`
3. Check MCP server status with Ctrl+T

## Working Example

Since Gemini shows "Using: - 1 MCP server", try:
```bash
# This should work:
use tools: query_memory_with_context "what have we worked on today"

# Or try the /use prefix:
/use query_memory_with_context "show recent work"
```