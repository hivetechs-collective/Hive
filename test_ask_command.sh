#!/bin/bash

# Test script for the ask command
echo "Testing ask command implementation..."

# Test 1: Help command
echo -e "\n1. Testing help command:"
./target/release/hive ask --help 2>&1 | head -10

# Test 2: Check if the command processes the question (without API key)
echo -e "\n2. Testing command without API key:"
timeout 10s ./target/release/hive ask "What is 2+2?" 2>&1 | grep -E "(Processing|OpenRouter|API key|Warning)" || echo "Command timed out or failed"

echo -e "\nTest complete."