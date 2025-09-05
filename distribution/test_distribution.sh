#!/usr/bin/env bash
#
# Distribution System Test Script
# Tests the complete distribution pipeline without actually building
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

test_file_exists() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ]; then
        print_success "$description exists"
        return 0
    else
        print_error "$description missing: $file"
        return 1
    fi
}

test_script_executable() {
    local script="$1"
    local description="$2"
    
    if [ -x "$script" ]; then
        print_success "$description is executable"
        return 0
    else
        print_error "$description not executable: $script"
        return 1
    fi
}

test_script_syntax() {
    local script="$1"
    local description="$2"
    
    if bash -n "$script" 2>/dev/null; then
        print_success "$description syntax valid"
        return 0
    else
        print_error "$description has syntax errors"
        return 1
    fi
}

main() {
    echo
    echo -e "${BLUE}╭─────────────────────────────────────────╮${NC}"
    echo -e "${BLUE}│${NC}  🧪 Distribution System Test Suite     ${BLUE}│${NC}"
    echo -e "${BLUE}╰─────────────────────────────────────────╯${NC}"
    echo
    
    local total_tests=0
    local passed_tests=0
    
    print_status "Testing distribution system structure..."
    
    # Test directory structure
    ((total_tests++))
    if [ -d "$SCRIPT_DIR" ]; then
        print_success "Distribution directory exists"
        ((passed_tests++))
    else
        print_error "Distribution directory missing"
    fi
    
    # Test build scripts
    for platform in macos linux; do
        ((total_tests++))
        if test_file_exists "$SCRIPT_DIR/build/${platform}.sh" "Build script for $platform"; then
            ((passed_tests++))
        fi
        
        ((total_tests++))
        if test_script_executable "$SCRIPT_DIR/build/${platform}.sh" "Build script for $platform"; then
            ((passed_tests++))
        fi
        
        ((total_tests++))
        if test_script_syntax "$SCRIPT_DIR/build/${platform}.sh" "Build script for $platform"; then
            ((passed_tests++))
        fi
    done
    
    # Test Windows PowerShell script
    ((total_tests++))
    if test_file_exists "$SCRIPT_DIR/build/windows.ps1" "Windows build script"; then
        ((passed_tests++))
    fi
    
    # Test install script
    ((total_tests++))
    if test_file_exists "$SCRIPT_DIR/scripts/install.sh" "Universal install script"; then
        ((passed_tests++))
    fi
    
    ((total_tests++))
    if test_script_executable "$SCRIPT_DIR/scripts/install.sh" "Universal install script"; then
        ((passed_tests++))
    fi
    
    ((total_tests++))
    if test_script_syntax "$SCRIPT_DIR/scripts/install.sh" "Universal install script"; then
        ((passed_tests++))
    fi
    
    # Test installer configurations
    for installer in macos/create_pkg.sh linux/create_deb.sh; do
        ((total_tests++))
        if test_file_exists "$SCRIPT_DIR/installers/$installer" "Installer script for ${installer%/*}"; then
            ((passed_tests++))
        fi
        
        ((total_tests++))
        if test_script_executable "$SCRIPT_DIR/installers/$installer" "Installer script for ${installer%/*}"; then
            ((passed_tests++))
        fi
        
        ((total_tests++))
        if test_script_syntax "$SCRIPT_DIR/installers/$installer" "Installer script for ${installer%/*}"; then
            ((passed_tests++))
        fi
    done
    
    # Test Windows NSIS script
    ((total_tests++))
    if test_file_exists "$SCRIPT_DIR/installers/windows/hive.nsi" "Windows NSIS installer script"; then
        ((passed_tests++))
    fi
    
    # Test CI/CD workflow
    ((total_tests++))
    if test_file_exists "$SCRIPT_DIR/ci/release.yml" "GitHub Actions release workflow"; then
        ((passed_tests++))
    fi
    
    # Test orchestration script
    ((total_tests++))
    if test_file_exists "$SCRIPT_DIR/build_all.sh" "Build orchestration script"; then
        ((passed_tests++))
    fi
    
    ((total_tests++))
    if test_script_executable "$SCRIPT_DIR/build_all.sh" "Build orchestration script"; then
        ((passed_tests++))
    fi
    
    ((total_tests++))
    if test_script_syntax "$SCRIPT_DIR/build_all.sh" "Build orchestration script"; then
        ((passed_tests++))
    fi
    
    # Test project configuration
    ((total_tests++))
    if test_file_exists "$PROJECT_ROOT/Cargo.toml" "Project Cargo.toml"; then
        ((passed_tests++))
    fi
    
    # Test Cargo.toml for production profile
    ((total_tests++))
    if grep -q "\[profile\.production\]" "$PROJECT_ROOT/Cargo.toml"; then
        print_success "Production profile configured in Cargo.toml"
        ((passed_tests++))
    else
        print_error "Production profile missing in Cargo.toml"
    fi
    
    # Test for auto-updater
    ((total_tests++))
    if test_file_exists "$PROJECT_ROOT/src/core/updater.rs" "Auto-updater implementation"; then
        ((passed_tests++))
    fi
    
    # Test platform detection in install script
    ((total_tests++))
    if grep -q "detect_platform" "$SCRIPT_DIR/scripts/install.sh"; then
        print_success "Platform detection in install script"
        ((passed_tests++))
    else
        print_error "Platform detection missing in install script"
    fi
    
    # Test checksum verification in install script
    ((total_tests++))
    if grep -q "checksum" "$SCRIPT_DIR/scripts/install.sh"; then
        print_success "Checksum verification in install script"
        ((passed_tests++))
    else
        print_error "Checksum verification missing in install script"
    fi
    
    # Test help text in build script
    ((total_tests++))
    if grep -q "\-\-help" "$SCRIPT_DIR/build_all.sh"; then
        print_success "Help text in build orchestrator"
        ((passed_tests++))
    else
        print_error "Help text missing in build orchestrator"
    fi
    
    # Summary
    echo
    echo -e "${BLUE}Test Results:${NC}"
    echo "  Total Tests: $total_tests"
    echo "  Passed: $passed_tests"
    echo "  Failed: $((total_tests - passed_tests))"
    
    local pass_rate=$((passed_tests * 100 / total_tests))
    echo "  Pass Rate: ${pass_rate}%"
    
    if [ "$passed_tests" -eq "$total_tests" ]; then
        echo
        print_success "🎉 All tests passed! Distribution system ready."
        echo
        echo -e "${BLUE}Next steps:${NC}"
        echo "1. Test build on actual platforms: ./distribution/build_all.sh --skip-tests"
        echo "2. Verify installers work correctly"
        echo "3. Test auto-update mechanism"
        echo "4. Validate universal install script"
        echo
        return 0
    else
        echo
        print_error "❌ Some tests failed. Please fix issues before proceeding."
        return 1
    fi
}

main "$@"