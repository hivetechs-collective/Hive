# Memory Commands Usage Guide

## Overview
Hive automatically configures memory commands for all CLI tools. These commands give your CLI tools access to Hive's Memory+Context intelligence.

## ✨ Automatic Setup
**No configuration needed!** When you:
1. **Install Hive** - PATH automatically configured
2. **Install CLI tools** - Memory integration automatically enabled
3. **Restart terminal** - Commands available immediately

## 📝 Command Format

### Enhanced Memory+Context (Recommended)
```bash
# Tool-specific commands (fastest)
hive-memory-gemini-cli query-with-context "What have we worked on?"
hive-memory-claude-code query-with-context "Show me Python examples"
hive-memory-grok query-with-context "What debugging techniques have we learned?"

# Universal command (works for any tool)
hive-memory context "What have we worked on?" gemini-cli
```

### Basic Memory Query
```bash
hive-memory-gemini-cli query "Show me code examples"
hive-memory query "What TypeScript patterns have we used?" gemini-cli
```

### Contribute Learning
```bash
hive-memory-gemini-cli contribute solution python "Fixed async import issue"
hive-memory contribute insight debugging "Console.log first for React errors" gemini-cli
```

## 🎯 Usage Examples

### For Gemini CLI Users
```bash
# Ask about recent work
hive-memory-gemini-cli query-with-context "What have we been working on?"

# Get language-specific help
hive-memory-gemini-cli query-with-context "Show me JavaScript patterns we've discussed"

# Share successful solutions
hive-memory-gemini-cli contribute solution react "Fixed useEffect dependency array"
```

### For Claude Code Users
```bash
hive-memory-claude-code query-with-context "What debugging approaches have we used?"
hive-memory-claude-code contribute insight typescript "Always check return types"
```

### For All Other Tools
```bash
hive-memory-grok query-with-context "What have we learned about databases?"
hive-memory-qwen-code query-with-context "Show me recent CSS techniques"
hive-memory-openai-codex query-with-context "What API patterns have we used?"
hive-memory-cline query-with-context "What testing strategies have we discussed?"
```

## 🚀 Performance
- **2-5ms response times** - Much faster than previous MCP approach
- **Dynamic port discovery** - Automatically finds Memory Service
- **Same quality** - Uses Hive's full Memory+Context pipeline

## 🔍 Verification
Commands automatically appear in Hive's Memory Service dashboard:
- **Connected Tools**: Shows all 6 tools as "Active"
- **Live Activity Stream**: Real-time query tracking
- **Query Statistics**: Tracks usage per tool

## 💡 Tips
1. **Use "query-with-context"** for best results (includes Memory+Context intelligence)
2. **Be specific in questions** - "Show me React hooks we've discussed" vs "Show me code"
3. **Contribute learnings** - Help build collective intelligence
4. **Check the Memory dashboard** - See all CLI tool activity in real-time

## ❓ Troubleshooting

### Command Not Found
```bash
# Check if PATH configured
echo $PATH | grep .hive

# If not found, restart terminal or run:
source ~/.zshrc  # or ~/.bashrc
```

### Memory Service Not Available
```bash
# Check if Hive is running
ps aux | grep "Hive Consensus"

# Commands only work when Hive IDE is running
```

### No Results
- Commands work immediately, but enhanced results come from conversation history
- Start using Hive for coding questions to build memory
- Memory+Context improves over time with usage

## 🎉 Ready to Use!
Your memory commands are ready! Try:
```bash
hive-memory-gemini-cli query-with-context "What have we been working on lately?"
```