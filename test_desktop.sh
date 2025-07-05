#!/bin/bash

# Test script for desktop GUI

echo "🖥️  Testing Hive Desktop GUI..."
echo "================================"

# First try to build the desktop test
echo "Building desktop test app..."
cargo build --bin hive-desktop-test --release

if [ $? -eq 0 ]; then
    echo "✅ Desktop test build successful"
    echo ""
    echo "Running desktop test app..."
    RUST_LOG=debug ./target/release/hive-desktop-test
else
    echo "❌ Desktop test build failed"
    echo ""
    echo "Trying main app with desktop mode..."
    RUST_LOG=debug HIVE_DESKTOP=1 cargo run --release
fi