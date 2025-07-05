#!/bin/bash

echo "=== Testing Hive AI Consensus Engine Integration ==="
echo

# Build the project
echo "Building Hive AI..."
cargo build --release
echo

# Test 1: Basic ask command without API key
echo "Test 1: Basic ask command (demo mode)"
./target/release/hive ask "What is the capital of France?"
echo

# Test 2: Different profile
echo "Test 2: Speed profile"
./target/release/hive ask "What is 2+2?" --profile speed
echo

# Test 3: Quality profile
echo "Test 3: Quality profile"
./target/release/hive ask "Explain quantum computing" --profile quality
echo

# Test 4: Cost profile
echo "Test 4: Cost profile"
./target/release/hive ask "Hello world" --profile cost
echo

# Test 5: Invalid profile
echo "Test 5: Invalid profile (should use balanced)"
./target/release/hive ask "Test question" --profile invalid
echo

echo "=== Integration Test Complete ==="
echo
echo "To test with real OpenRouter API:"
echo "1. Set your OPENROUTER_API_KEY environment variable"
echo "2. Run: export OPENROUTER_API_KEY='sk-or-your-key-here'"
echo "3. Re-run this script"