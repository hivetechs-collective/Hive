#!/bin/bash

# CLI Verification Script for Hive AI Rust Implementation
echo "🐝 Hive AI CLI Verification Script"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TOTAL_TESTS=0
PASSED_TESTS=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "\n${BLUE}Test $TOTAL_TESTS: $test_name${NC}"
    echo "Command: $test_command"
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAILED${NC}"
        echo "Error output:"
        eval "$test_command"
    fi
}

echo -e "\n${YELLOW}Testing Rust CLI Implementation...${NC}"

# Check if Cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Test 1: Project builds
run_test "Project compiles" "cargo check --quiet"

# Test 2: Tests pass
run_test "Unit tests pass" "cargo test --quiet"

# Test 3: Help command works
run_test "Help command" "cargo run --quiet -- --help"

# Test 4: Version command works  
run_test "Version command" "cargo run --quiet -- --version"

# Test 5: Config creation works
run_test "Config validation" "cargo run --quiet -- config validate"

# Test 6: Status command works
run_test "Status command" "cargo run --quiet -- status"

echo -e "\n${YELLOW}Summary:${NC}"
echo -e "Total tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$((TOTAL_TESTS - PASSED_TESTS))${NC}"

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo -e "\n${GREEN}🎉 All tests passed! CLI implementation is working.${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please check the implementation.${NC}"
    exit 1
fi