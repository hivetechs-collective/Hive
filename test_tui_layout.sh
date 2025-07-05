#!/bin/bash
# Test the TUI layout fixes

echo "Testing Hive TUI layout improvements..."
echo "Messages should appear ABOVE the input box (like Claude Code)"
echo "Consensus progress should be inline with messages"
echo "Use Page Up/Down to scroll through messages"
echo ""
echo "Press Enter to launch TUI test..."
read

# Launch the TUI
./target/release/hive tui