#!/bin/bash
# Security System Demo for HiveTechs Consensus

echo "🔒 HiveTechs Consensus Security System Demo"
echo "==========================================="
echo ""
echo "This demo shows the Claude Code-style trust system in action."
echo ""

# Build the project if needed
echo "📦 Building Hive AI..."
cargo build --release 2>/dev/null || cargo build

echo ""
echo "🚀 Running security demo..."
echo ""

# Run the security demo
cargo run --example security_demo

echo ""
echo "✨ Demo complete!"
echo ""
echo "The security system is now integrated into all file operations."
echo "Try running 'cargo run -- analyze /some/untrusted/path' to see it in action!"