# Test Commands for All 6 CLI Tools - MCP Memory Service

## 🧪 Quick Test Commands (Copy & Paste)

Test each tool after installation to verify MCP is working:

### 1. Gemini CLI
```bash
# Test command:
gemini
> use tools: @memory "test connection"

# If successful, you'll see memory query results
# If failed, you'll see "tool not found" error
```

### 2. Grok CLI  
```bash
# Test command:
grok
> !tools @memory "test connection"

# Alternative if first doesn't work:
> !tools query_memory_with_context "test connection"
```

### 3. Qwen Code
```bash
# Test command:
qwen
> use @memory tool: test connection

# Alternative if first doesn't work:
> use query_memory_with_context tool: test connection
```

### 4. Claude Code
```bash
# Test command (simplest):
claude
> @memory test connection

# Should work immediately with MCP configured
```

### 5. OpenAI Codex
```bash
# Test command:
codex
> invoke @memory "test connection"

# Alternative if first doesn't work:
> /mcp @memory "test connection"
```

### 6. Cline
```bash
# Test command:
cline
> @tools @memory "test connection"

# Alternative if first doesn't work:
> /mcp @memory "test connection"
```

## ✅ Success Indicators

When MCP is working correctly, you should see:
1. **Memory Service Dashboard** shows query count increment
2. **Tool response** includes memory/context data
3. **No error messages** about missing tools or MCP servers

## ❌ Failure Indicators

If MCP is NOT working:
1. **"Tool not found"** error message
2. **"MCP server not configured"** error
3. **Tool answers without querying Memory Service**
4. **Dashboard shows 0 queries** after your test

## 🔍 Verification Steps

1. **Start Memory Service Dashboard**:
   ```bash
   # In Hive IDE, check Memory Service panel
   # Or navigate to: http://localhost:37352/dashboard
   ```

2. **Run test command** for your tool (see above)

3. **Check Dashboard** - query count should increase

4. **Try a real query**:
   ```bash
   # For any tool, use their syntax with:
   "what have we worked on today"
   "show me recent changes"
   "recall our last conversation"
   ```

## 📊 Expected Results Table

| Tool | Command | Expected Output |
|------|---------|-----------------|
| **Gemini CLI** | `use tools: @memory "test"` | Memory query results |
| **Grok CLI** | `!tools @memory "test"` | Memory query results |
| **Qwen Code** | `use @memory tool: test` | Memory query results |
| **Claude Code** | `@memory test` | Memory query results |
| **OpenAI Codex** | `invoke @memory "test"` | Memory query results |
| **Cline** | `@tools @memory "test"` | Memory query results |

## 🚀 All-In-One Test Script

```bash
#!/bin/bash
# Test all installed CLI tools for MCP connectivity

echo "Testing MCP Memory Service for all tools..."

# Test each tool if installed
which gemini > /dev/null && echo "use tools: @memory \"test\"" | gemini
which grok > /dev/null && echo "!tools @memory \"test\"" | grok  
which qwen > /dev/null && echo "use @memory tool: test" | qwen
which claude > /dev/null && echo "@memory test" | claude
which codex > /dev/null && echo "invoke @memory \"test\"" | codex
which cline > /dev/null && echo "@tools @memory \"test\"" | cline

echo "Check Memory Service Dashboard for query count"
```

## 💡 Pro Tips

1. **Claude Code** has the cleanest integration - just `@memory` works
2. **Gemini/Grok/Qwen** need their tool invocation prefix
3. **OpenAI Codex/Cline** support multiple syntaxes - try both
4. Always check the Memory Service Dashboard to confirm queries are being received
5. If simplified commands fail, fall back to `query_memory_with_context`