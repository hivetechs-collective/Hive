#!/bin/bash

echo "========================================="
echo "   AI CLI Tools Integration Verification"
echo "========================================="
echo ""

# Check Claude Code installation
echo "1. Claude Code Status:"
if command -v claude &> /dev/null; then
    echo "   ✅ Installed: $(claude --version)"
else
    echo "   ❌ Not installed"
fi
echo ""

# Check Memory Service
echo "2. Memory Service Status:"
curl -s http://localhost:3457/health | jq '.' 2>/dev/null || echo "   ❌ Not running"
echo ""

# Check MCP configuration
echo "3. MCP Configuration:"
if [ -f ~/.claude/.mcp.json ]; then
    echo "   ✅ MCP config exists"
    echo "   Servers configured:"
    cat ~/.claude/.mcp.json | jq -r '.servers | keys[]' | sed 's/^/      - /'
else
    echo "   ❌ No MCP config found"
fi
echo ""

# Check CLI tools config
echo "4. CLI Tools Configuration:"
CONFIG_DIR=~/.config/hive-consensus-cli-tools
if [ -d "$CONFIG_DIR" ]; then
    echo "   ✅ Config directory exists"
    if [ -f "$CONFIG_DIR/config.json" ]; then
        echo "   Tools configured:"
        cat "$CONFIG_DIR/config.json" | jq -r 'keys[]' | sed 's/^/      - /'
    fi
else
    echo "   ⚠️  No config directory yet (will be created on first configure)"
fi
echo ""

# Check Memory Service registration
echo "5. Memory Service Tools:"
curl -s http://localhost:3457/api/v1/memory/tools | jq '.tools | length' 2>/dev/null | xargs -I {} echo "   Connected tools: {}"
echo ""

echo "========================================="
echo "To test in the app:"
echo "1. Click 'Configure' on Claude Code card"
echo "2. Wait for 'Configured' status"
echo "3. Click 'Update' to check for updates"
echo "4. Check console for messages"
echo "========================================="