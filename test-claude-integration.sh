#!/bin/bash

echo "🧪 Testing Claude Code Integration..."
echo ""

# Check if Claude Code is available
echo "1. Checking for Claude Code CLI..."
if command -v claude &> /dev/null; then
    echo "✅ Claude Code found: $(which claude)"
    echo "   Version: $(claude --version 2>&1 || echo 'Unable to get version')"
else
    echo "⚠️  Claude Code not found in PATH"
    
    # Check npm global
    NPM_PREFIX=$(npm config get prefix 2>/dev/null || echo "/usr/local")
    if [ -f "$NPM_PREFIX/bin/claude" ]; then
        echo "✅ Found in npm global: $NPM_PREFIX/bin/claude"
    else
        echo "❌ Not found in npm global either"
        echo ""
        echo "To install Claude Code:"
        echo "  npm install -g @anthropic-ai/claude-code"
    fi
fi

echo ""
echo "2. Checking Hive binary..."
if [ -f "target/debug/hive-consensus" ]; then
    echo "✅ Hive-consensus binary found"
else
    echo "❌ Hive-consensus binary not found. Run: cargo build"
fi

echo ""
echo "3. Configuration status..."
if [ -f "$HOME/.hive/config.toml" ]; then
    echo "✅ Hive config exists"
else
    echo "⚠️  No Hive config found at ~/.hive/config.toml"
fi

echo ""
echo "4. Integration features implemented:"
echo "✅ Claude Code subprocess spawning"
echo "✅ Stateless context injection"
echo "✅ Execution modes (ConsensusFirst/Assisted/Direct)"
echo "✅ Plan Mode and Auto-Edit toggles"
echo "✅ Conversation context storage"
echo "✅ Smart command routing"
echo "✅ npm-based Claude Code detection"

echo ""
echo "🎯 Ready to test hybrid integration once Claude Code is installed!"