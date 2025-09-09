# Unified MCP Memory Service Commands

## Overview
Hive Consensus v1.8.244+ includes enhanced MCP (Model Context Protocol) integration that provides **unified, user-friendly commands** that work consistently across all 6 CLI tools.

## Supported Tools
- Claude Code
- Gemini CLI  
- Qwen Code
- OpenAI Codex
- Cline
- Grok CLI

## Universal Trigger Commands

These commands work **identically** in all supported CLI tools:

### Primary Commands

#### `@memory <question>`
Query your memory for specific information.
```bash
@memory what authentication method did we implement?
@memory show me the database schema we created
```

#### `@context`
Get context about your current work and project.
```bash
@context
@context show me today's progress
```

#### `@recall <topic>`
Recall past discussions about a specific topic.
```bash
@recall Memory Service architecture
@recall MCP integration
```

### Natural Language Triggers

These phrases **automatically trigger** memory queries without special syntax:

#### "What have we..."
```bash
What have we worked on today?
What have we discussed about authentication?
What have we built so far?
```

#### "What did we..."
```bash
What did we decide about the database?
What did we discuss yesterday?
What did we implement for the API?
```

#### "Show me our..."
```bash
Show me our progress on the Memory Service
Show me our recent commits
Show me our architecture decisions
```

#### "Remind me about..."
```bash
Remind me about the MCP configuration
Remind me about our security implementation
```

## How It Works

### Automatic Context Enhancement
When you use any trigger command, the Memory Service:
1. **Detects** the trigger pattern
2. **Removes** the prefix for cleaner processing
3. **Enriches** the query with relevant context
4. **Returns** comprehensive results from your memory

### Intelligent Query Processing
- Queries are automatically enhanced with project context
- Related memories are included by default
- Work history is considered for better results
- Tool-specific context is preserved

## Examples

### Basic Memory Query
```bash
# In any CLI tool:
> @memory how do we handle authentication?

# Returns:
- Previous authentication discussions
- Code implementations
- Architecture decisions
- Related security considerations
```

### Context-Aware Query
```bash
# In any CLI tool:
> What have we worked on with MCP integration?

# Returns:
- MCP configuration history
- Integration decisions
- Code changes
- Testing results
```

### Project Status
```bash
# In any CLI tool:
> @context

# Returns:
- Current project status
- Recent activities
- Open tasks
- Latest changes
```

## Advanced Features

### Include Related Context
All queries automatically include related context to provide comprehensive answers.

### Cross-Tool Memory
Information learned in one tool is immediately available in all other tools.

### Persistent Learning
All successful solutions are automatically saved for future reference.

## Tips for Best Results

1. **Be Specific**: More specific queries return more relevant results
   ```bash
   ❌ @memory code
   ✅ @memory authentication code for user login
   ```

2. **Use Natural Language**: The system understands conversational queries
   ```bash
   ✅ What did we discuss about error handling yesterday?
   ✅ Show me our database migration strategy
   ```

3. **Combine with Actions**: Use memory to inform your next steps
   ```bash
   @memory how did we structure the API routes?
   # Then use that information to continue development
   ```

## Configuration

The unified MCP commands are **automatically configured** when you install any CLI tool through Hive Consensus. No manual setup required!

### Automatic Features
- ✅ Dynamic port detection
- ✅ Automatic token management
- ✅ Tool-specific identification
- ✅ User-agnostic paths
- ✅ Cross-platform compatibility

## Troubleshooting

### Commands Not Working?

1. **Check Memory Service Status**: 
   - Open Hive Consensus
   - Navigate to Memory Service section
   - Verify it shows "Running"

2. **Verify Tool Configuration**:
   - The tool should be installed via Hive Consensus
   - Check for the tool in Connected Tools list

3. **Try Explicit Tool Invocation**:
   ```bash
   # If auto-trigger isn't working, try:
   Use query_memory_with_context tool: your question
   ```

### Getting Help

In any CLI tool, type:
```bash
/memory_help
```

This will show:
- Available commands
- Usage examples
- Tool-specific tips

## Version Requirements

- **Hive Consensus**: v1.8.244 or later
- **CLI Tools**: Latest versions (auto-updated)
- **Memory Service**: v2.0.0 (included)

## Benefits

### Consistency
Same commands work across all tools - learn once, use everywhere.

### Simplicity
Natural language triggers mean you don't need to remember special syntax.

### Intelligence
Automatic context enhancement provides better, more relevant results.

### Integration
Seamless integration means Memory Service "just works" without configuration.

## Coming Soon

- Voice triggers for hands-free memory queries
- Visual memory search with screenshots
- Team memory sharing capabilities
- Advanced filtering and search operators

---

*Last Updated: Version 1.8.244*