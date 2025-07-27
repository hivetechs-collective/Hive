#!/bin/bash

echo "Testing Claude Code commands directly..."

echo -e "\n1. Testing help command:"
claude --help | head -10

echo -e "\n2. Testing version:"
claude --version

echo -e "\n3. Testing context command:"
claude context

echo -e "\n4. Testing ask with a simple question:"
echo "What is 2+2?" | claude ask -

echo -e "\n5. Testing ask with slash command (this should NOT work as expected):"
echo "/help" | claude ask -

echo -e "\nConclusion: Slash commands need to be handled specially, not passed to 'claude ask'"