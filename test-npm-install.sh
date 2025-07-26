#!/bin/bash

echo "🧪 Testing npm-based Claude Code installation..."

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install Node.js/npm first."
    exit 1
fi

# Get npm prefix
NPM_PREFIX=$(npm config get prefix)
echo "📍 npm prefix: $NPM_PREFIX"

# Check if Claude Code is already installed
if command -v claude &> /dev/null; then
    echo "✅ Claude Code is already installed:"
    claude --version
else
    echo "⚠️  Claude Code not found in PATH"
    
    # Check npm global bin directory
    if [ -f "$NPM_PREFIX/bin/claude" ]; then
        echo "✅ Found Claude Code in npm global bin: $NPM_PREFIX/bin/claude"
        "$NPM_PREFIX/bin/claude" --version
    else
        echo "❌ Claude Code not found in npm global bin"
        echo ""
        echo "To install Claude Code via npm, run:"
        echo "  npm install -g @anthropic-ai/claude-code"
    fi
fi

# Test the Rust binary detection
echo ""
echo "🦀 Testing Rust binary detection..."
cargo run --bin hive -- --version 2>&1 | grep -E "(Claude|claude)" || echo "No Claude references in output"