#\!/bin/bash

echo "Testing desktop mode launch..."
echo "This should launch the desktop GUI without tokio runtime conflicts"
echo ""

# Test the desktop flag
echo "1. Testing: hive --desktop"
cargo run -- --desktop &
PID=$\!
sleep 3
kill $PID 2>/dev/null

echo ""
echo "2. Testing: hive desktop"
cargo run -- desktop &
PID=$\!
sleep 3
kill $PID 2>/dev/null

echo ""
echo "Desktop mode test completed. If no runtime errors appeared, the fix is working\!"
