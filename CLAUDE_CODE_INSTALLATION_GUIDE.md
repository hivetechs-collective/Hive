# Claude Code CLI Installation Guide

## 🚀 Quick Install for Testing

Since Claude Code CLI doesn't have public download URLs yet, here are your options:

### Option 1: Use Claude Desktop App Binary (Recommended)

If you have Claude Desktop app installed, you can often find the CLI binary inside:

```bash
# macOS - Check if Claude Desktop is installed
ls -la /Applications/Claude.app/Contents/MacOS/claude

# If found, create a symlink
sudo ln -s /Applications/Claude.app/Contents/MacOS/claude /usr/local/bin/claude

# Or copy to Hive's expected location
mkdir -p ~/.hive/claude-code/bin
cp /Applications/Claude.app/Contents/MacOS/claude ~/.hive/claude-code/bin/claude
chmod +x ~/.hive/claude-code/bin/claude
```

### Option 2: Manual Binary Placement

1. Download Claude from https://claude.ai/download
2. Extract/install the application
3. Locate the `claude` binary
4. Place it in one of these locations:
   - `~/.hive/claude-code/claude` (Hive will auto-detect)
   - `/usr/local/bin/claude` (System-wide)
   - `~/.local/bin/claude` (User-specific)

### Option 3: Build from Source (If Available)

```bash
# If Claude Code is open source (check GitHub)
git clone https://github.com/anthropics/claude-code
cd claude-code
cargo build --release
cp target/release/claude ~/.hive/claude-code/bin/
```

## 🧪 Testing the Integration

Once Claude Code CLI is installed:

```bash
# 1. Verify installation
claude --version

# 2. Test in Hive IDE
cd /Users/veronelazio/Developer/Private/hive
cargo run --bin hive-consensus

# 3. Try these commands in the chat:
# - /login (should show Claude's auth options)
# - /help (should show Claude's commands)
# - /consensus "test question" (Hive command)
# - Regular chat should go through Claude Code
```

## 🔍 Troubleshooting

### Binary Not Found
If Hive can't find Claude Code, it checks these paths:
- `claude` (in PATH)
- `claude-code` (alternative name)
- `/usr/local/bin/claude`
- `/usr/local/bin/claude-code`
- `/opt/homebrew/bin/claude`
- `/opt/homebrew/bin/claude-code`
- `~/.local/bin/claude`
- `~/.cargo/bin/claude`
- `/Applications/Claude.app/Contents/MacOS/claude`
- `C:\\Program Files\\Claude\\claude.exe` (Windows)

### Manual Override
Set the CLAUDE_CODE_BINARY environment variable:
```bash
export CLAUDE_CODE_BINARY=/path/to/claude
cargo run --bin hive-consensus
```

### Permission Issues
```bash
# Make binary executable
chmod +x /path/to/claude

# Add to PATH
export PATH="$PATH:/path/to/claude/directory"
```

## 📝 Expected Behavior Once Working

When Claude Code CLI is properly installed:

1. **Slash Command Autocomplete**: Type `/` in chat to see dropdown
2. **Native Authentication**: `/login` shows browser-based auth
3. **Hive Commands**: `/consensus`, `/memory`, `/openrouter` work
4. **Pass-through**: All other commands go to Claude Code
5. **Enhanced Responses**: Claude responses include Hive context

## 🎯 Current Status

- ✅ All integration code complete
- ✅ Automatic installer created (awaits official URLs)
- ✅ Binary detection enhanced (10+ paths)
- ✅ Settings UI shows Claude status
- ❌ Claude Code CLI not installed on test system
- ❌ Can't test full integration without CLI

## 💡 Alternative: Mock Mode for Development

If you can't get Claude Code CLI immediately, you can test with a mock:

```bash
# Create a mock claude binary for testing
cat > ~/.hive/claude-code/bin/claude << 'EOF'
#!/bin/bash
echo "Claude Code Mock v1.0.0"
if [[ "$1" == "--version" ]]; then
    echo "claude-code 1.0.0-mock"
elif [[ "$1" == "/login" ]]; then
    echo "🔐 Claude authentication (mock mode)"
    echo "1. API Key authentication"
    echo "2. Browser-based login"
else
    echo "Mock response to: $@"
fi
EOF

chmod +x ~/.hive/claude-code/bin/claude
```

This won't give you the real Claude Code experience but will let you test the integration flow.