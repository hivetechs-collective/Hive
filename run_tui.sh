#!/bin/bash

# Run Hive TUI with standard terminal settings
export TERM=xterm-256color
export COLORTERM=truecolor

# Get the hive binary location
HIVE_BIN="./target/release/hive"

if [ ! -f "$HIVE_BIN" ]; then
    echo "❌ Hive binary not found at $HIVE_BIN"
    echo "💡 Run: cargo build --release"
    exit 1
fi

echo "🐝 Launching Hive TUI with standard terminal settings..."
echo "Terminal: $TERM"
echo ""

# Run hive tui
"$HIVE_BIN" tui