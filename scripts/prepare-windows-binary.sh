#!/bin/bash
set -e

echo "🪟 Preparing Windows binary for distribution..."

# For now, we'll create a README for Windows users
cat > dist/WINDOWS_BUILD_INSTRUCTIONS.txt << 'EOF'
Hive IDE - Windows Build Instructions
=====================================

To build Hive IDE on Windows:

1. Install Rust:
   - Download from https://rustup.rs/
   - Run the installer (rustup-init.exe)
   - Follow the prompts

2. Install Visual Studio Build Tools:
   - Download from https://visualstudio.microsoft.com/downloads/
   - Install "Desktop development with C++" workload

3. Clone and build Hive:
   git clone https://github.com/hivetechs/hive.git
   cd hive
   cargo build --bin hive-consensus --release

4. The binary will be at:
   target\release\hive-consensus.exe

5. Rename to hive.exe and place in your PATH

For support: support@hivetechs.io
EOF

echo "✅ Windows build instructions created at dist/WINDOWS_BUILD_INSTRUCTIONS.txt"
echo ""
echo "📝 To build on Windows:"
echo "   1. Install Rust and Visual Studio Build Tools"
echo "   2. Run: cargo build --bin hive-consensus --release"
echo "   3. Binary will be at target/release/hive-consensus.exe"