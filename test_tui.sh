#!/bin/bash
# Test script to verify TUI input handling

echo "Testing TUI input handler..."

# Build the project
echo "Building project..."
cargo build --release

# Create a test input file
cat > test_input.txt << EOF
help
What is Rust?
exit
EOF

echo ""
echo "Running TUI with test input..."
echo "(This will show the TUI interface and process 'help', then a question, then exit)"
echo ""

# Run the TUI with input from file
cargo run --release -- tui < test_input.txt

echo ""
echo "TUI test completed."