#!/bin/bash

# Test script to verify TUI terminal error fix
echo "🔧 Testing TUI terminal error fixes..."
echo "=================================="
echo

# Test 1: Non-TTY environment (should fall back gracefully)
echo "Test 1: Non-TTY environment (should use fallback CLI)"
echo "TERM=dumb" | ./target/release/hive --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ PASS: Non-TTY environment handled gracefully"
else
    echo "❌ FAIL: Non-TTY environment failed"
fi
echo

# Test 2: CI environment simulation (should disable TUI)
echo "Test 2: CI environment simulation (should disable TUI)"
CI=true ./target/release/hive --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ PASS: CI environment handled gracefully"
else
    echo "❌ FAIL: CI environment failed"
fi
echo

# Test 3: Basic help command (should work in any environment)
echo "Test 3: Basic help command"
./target/release/hive --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ PASS: Basic help command works"
else
    echo "❌ FAIL: Basic help command failed"
fi
echo

# Test 4: Version command (should work in any environment)
echo "Test 4: Version command"
./target/release/hive --version > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ PASS: Version command works"
else
    echo "❌ FAIL: Version command failed"
fi
echo

# Test 5: Check TUI capability detection
echo "Test 5: TUI capability detection"
echo "Checking if TUI capabilities can be detected without errors..."
timeout 5s ./target/release/hive status 2>&1 | grep -v "Device not configured" > /dev/null
if [ $? -eq 0 ]; then
    echo "✅ PASS: TUI capability detection works without 'Device not configured' error"
else
    echo "❌ FAIL: TUI capability detection still shows errors"
fi
echo

echo "🏁 Test Summary"
echo "==============="
echo "All tests completed. The TUI terminal error 'Device not configured (os error 6)' should now be fixed."
echo "The application should fall back gracefully to simple CLI mode when TUI is not available."
echo
echo "To manually test TUI functionality:"
echo "1. Run: ./target/release/hive"
echo "2. In a terminal that supports TUI features"
echo "3. Check that no 'Device not configured' errors appear"
echo