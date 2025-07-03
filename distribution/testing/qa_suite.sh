#!/bin/bash
# HiveTechs Consensus - Comprehensive Quality Assurance Test Suite
# Production-ready distribution testing with security validation

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DISTRIBUTION_DIR="$PROJECT_ROOT/distribution"
RELEASE_DIR="$DISTRIBUTION_DIR/release"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Logging
TEST_LOG="$DISTRIBUTION_DIR/testing/qa_results.log"
mkdir -p "$(dirname "$TEST_LOG")"
echo "QA Test Suite Started: $(date)" > "$TEST_LOG"

print_header() {
    echo -e "${BLUE}==========================================${NC}"
    echo -e "${BLUE} HiveTechs Consensus - QA Test Suite${NC}"
    echo -e "${BLUE}==========================================${NC}"
    echo ""
}

print_section() {
    echo -e "${YELLOW}[SECTION]${NC} $1"
    echo "  $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""
}

print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
    echo "[TEST] $1" >> "$TEST_LOG"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
}

print_pass() {
    echo -e "${GREEN}  ✅ PASS${NC} $1"
    echo "  ✅ PASS $1" >> "$TEST_LOG"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

print_fail() {
    echo -e "${RED}  ❌ FAIL${NC} $1"
    echo "  ❌ FAIL $1" >> "$TEST_LOG"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

print_skip() {
    echo -e "${YELLOW}  ⏭️  SKIP${NC} $1"
    echo "  ⏭️  SKIP $1" >> "$TEST_LOG"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
}

print_info() {
    echo -e "${BLUE}  ℹ️  INFO${NC} $1"
    echo "  ℹ️  INFO $1" >> "$TEST_LOG"
}

# ============================================
# Test Functions
# ============================================

test_build_system() {
    print_section "Build System Verification"
    
    print_test "Cargo.toml production profile configuration"
    if grep -q "\\[profile.production\\]" "$PROJECT_ROOT/Cargo.toml"; then
        if grep -A 10 "\\[profile.production\\]" "$PROJECT_ROOT/Cargo.toml" | grep -q "opt-level.*=.*\"z\""; then
            print_pass "Production profile with size optimization configured"
        else
            print_fail "Production profile missing size optimization"
        fi
    else
        print_fail "Production profile not found in Cargo.toml"
    fi
    
    print_test "Build script executable permissions"
    local build_scripts=(
        "$DISTRIBUTION_DIR/build_all.sh"
        "$DISTRIBUTION_DIR/installers/macos/create_pkg.sh"
        "$DISTRIBUTION_DIR/installers/linux/create_deb.sh"
    )
    
    local all_executable=true
    for script in "${build_scripts[@]}"; do
        if [[ -x "$script" ]]; then
            print_info "✓ $script is executable"
        else
            print_info "✗ $script is not executable"
            all_executable=false
        fi
    done
    
    if $all_executable; then
        print_pass "All build scripts are executable"
    else
        print_fail "Some build scripts are not executable"
    fi
    
    print_test "Cross-compilation target support"
    if cargo --list | grep -q "build"; then
        # Check for common targets
        local targets=("x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-unknown-linux-musl" "x86_64-pc-windows-msvc")
        local targets_available=0
        
        for target in "${targets[@]}"; do
            if rustup target list --installed | grep -q "$target"; then
                targets_available=$((targets_available + 1))
                print_info "✓ Target $target is installed"
            else
                print_info "✗ Target $target not installed"
            fi
        done
        
        if [[ $targets_available -ge 2 ]]; then
            print_pass "Cross-compilation targets available ($targets_available/4)"
        else
            print_fail "Insufficient cross-compilation targets ($targets_available/4)"
        fi
    else
        print_fail "Cargo build command not available"
    fi
}

test_package_managers() {
    print_section "Package Manager Integration"
    
    print_test "Homebrew formula validation"
    local homebrew_formula="$DISTRIBUTION_DIR/packages/homebrew/hive.rb"
    if [[ -f "$homebrew_formula" ]]; then
        if grep -q "class Hive < Formula" "$homebrew_formula"; then
            print_pass "Homebrew formula structure is valid"
        else
            print_fail "Homebrew formula structure is invalid"
        fi
    else
        print_fail "Homebrew formula not found"
    fi
    
    print_test "Chocolatey package specification"
    local choco_spec="$DISTRIBUTION_DIR/packages/chocolatey/hive.nuspec"
    if [[ -f "$choco_spec" ]]; then
        if grep -q "<id>hive-ai</id>" "$choco_spec"; then
            print_pass "Chocolatey package specification is valid"
        else
            print_fail "Chocolatey package specification is invalid"
        fi
    else
        print_fail "Chocolatey package specification not found"
    fi
    
    print_test "Debian package control file"
    local debian_control="$DISTRIBUTION_DIR/packages/debian/control"
    if [[ -f "$debian_control" ]]; then
        if grep -q "Package: hive-ai" "$debian_control"; then
            print_pass "Debian control file is valid"
        else
            print_fail "Debian control file is invalid"
        fi
    else
        print_fail "Debian control file not found"
    fi
    
    print_test "Debian package scripts permissions"
    local debian_scripts=("$DISTRIBUTION_DIR/packages/debian/postinst" "$DISTRIBUTION_DIR/packages/debian/prerm" "$DISTRIBUTION_DIR/packages/debian/postrm")
    local all_executable=true
    
    for script in "${debian_scripts[@]}"; do
        if [[ -x "$script" ]]; then
            print_info "✓ $(basename "$script") is executable"
        else
            print_info "✗ $(basename "$script") is not executable"
            all_executable=false
        fi
    done
    
    if $all_executable; then
        print_pass "All Debian package scripts are executable"
    else
        print_fail "Some Debian package scripts are not executable"
    fi
}

test_auto_update_system() {
    print_section "Auto-Update System"
    
    print_test "Update server configuration"
    local update_config="$DISTRIBUTION_DIR/update_server/config.toml"
    if [[ -f "$update_config" ]]; then
        if grep -q "\\[server\\]" "$update_config" && grep -q "\\[security\\]" "$update_config"; then
            print_pass "Update server configuration is complete"
        else
            print_fail "Update server configuration is incomplete"
        fi
    else
        print_fail "Update server configuration not found"
    fi
    
    print_test "Update server implementation"
    local update_server="$DISTRIBUTION_DIR/update_server/server.rs"
    if [[ -f "$update_server" ]]; then
        if grep -q "UpdateServer" "$update_server" && grep -q "handle_rollback" "$update_server"; then
            print_pass "Update server implementation includes rollback capability"
        else
            print_fail "Update server implementation missing rollback capability"
        fi
    else
        print_fail "Update server implementation not found"
    fi
    
    print_test "Auto-update client integration"
    if [[ -f "$PROJECT_ROOT/src/core/updater.rs" ]]; then
        if grep -q "check_for_updates" "$PROJECT_ROOT/src/core/updater.rs"; then
            print_pass "Auto-update client implementation found"
        else
            print_fail "Auto-update client implementation incomplete"
        fi
    else
        print_fail "Auto-update client implementation not found"
    fi
}

test_security_measures() {
    print_section "Security Measures"
    
    print_test "macOS code signing configuration"
    local macos_entitlements="$DISTRIBUTION_DIR/installers/macos/entitlements.plist"
    if [[ -f "$macos_entitlements" ]]; then
        if grep -q "com.apple.security.network.client" "$macos_entitlements"; then
            print_pass "macOS entitlements file is configured"
        else
            print_fail "macOS entitlements file is incomplete"
        fi
    else
        print_fail "macOS entitlements file not found"
    fi
    
    print_test "macOS Info.plist configuration"
    local macos_info="$DISTRIBUTION_DIR/installers/macos/Info.plist"
    if [[ -f "$macos_info" ]]; then
        if grep -q "CFBundleIdentifier" "$macos_info" && grep -q "com.hivetechs.hive" "$macos_info"; then
            print_pass "macOS Info.plist is properly configured"
        else
            print_fail "macOS Info.plist is incomplete"
        fi
    else
        print_fail "macOS Info.plist not found"
    fi
    
    print_test "Security validation in build scripts"
    if grep -q "codesign" "$DISTRIBUTION_DIR/installers/macos/create_pkg.sh"; then
        print_pass "macOS build script includes code signing"
    else
        print_fail "macOS build script missing code signing"
    fi
    
    print_test "Checksum generation support"
    if grep -q "shasum\|sha256sum" "$DISTRIBUTION_DIR/build_all.sh" 2>/dev/null || \
       grep -q "shasum\|sha256sum" "$DISTRIBUTION_DIR/installers/macos/create_pkg.sh"; then
        print_pass "Checksum generation is implemented"
    else
        print_fail "Checksum generation not found"
    fi
}

test_universal_installer() {
    print_section "Universal Install Script"
    
    print_test "Universal installer script existence"
    local install_script="$DISTRIBUTION_DIR/scripts/install.sh"
    if [[ -f "$install_script" ]]; then
        print_pass "Universal install script found"
    else
        print_fail "Universal install script not found"
    fi
    
    print_test "Platform detection capability"
    if [[ -f "$install_script" ]]; then
        if grep -q "uname" "$install_script" && grep -q "ARCH\|PLATFORM" "$install_script"; then
            print_pass "Platform detection implemented"
        else
            print_fail "Platform detection not implemented"
        fi
    else
        print_skip "Platform detection test (no install script)"
    fi
    
    print_test "Error handling in installer"
    if [[ -f "$install_script" ]]; then
        if grep -q "set -e" "$install_script"; then
            print_pass "Error handling enabled in installer"
        else
            print_fail "Error handling not enabled in installer"
        fi
    else
        print_skip "Error handling test (no install script)"
    fi
}

test_performance_targets() {
    print_section "Performance Targets"
    
    print_test "Binary size optimization settings"
    if grep -q "opt-level.*=.*\"z\"" "$PROJECT_ROOT/Cargo.toml"; then
        print_pass "Size optimization configured"
    else
        print_fail "Size optimization not configured"
    fi
    
    print_test "Link-time optimization"
    if grep -q "lto.*=.*\"fat\"" "$PROJECT_ROOT/Cargo.toml"; then
        print_pass "Full LTO configured"
    else
        print_fail "Full LTO not configured"
    fi
    
    print_test "Symbol stripping configuration"
    if grep -q "strip.*=.*\"symbols\"" "$PROJECT_ROOT/Cargo.toml"; then
        print_pass "Symbol stripping configured"
    else
        print_fail "Symbol stripping not configured"
    fi
    
    print_test "Production build target"
    if cargo build --help | grep -q "profile"; then
        print_pass "Cargo supports custom profiles"
    else
        print_fail "Cargo custom profile support not available"
    fi
}

test_ci_cd_pipeline() {
    print_section "CI/CD Pipeline"
    
    print_test "GitHub Actions workflow"
    local workflow="$DISTRIBUTION_DIR/ci/release.yml"
    if [[ -f "$workflow" ]]; then
        if grep -q "strategy:" "$workflow" && grep -q "matrix:" "$workflow"; then
            print_pass "Multi-platform build matrix configured"
        else
            print_fail "Build matrix not properly configured"
        fi
    else
        print_fail "GitHub Actions workflow not found"
    fi
    
    print_test "Release automation"
    if [[ -f "$workflow" ]]; then
        if grep -q "upload-artifact\|release" "$workflow"; then
            print_pass "Release automation configured"
        else
            print_fail "Release automation not configured"
        fi
    else
        print_skip "Release automation test (no workflow file)"
    fi
}

test_documentation() {
    print_section "Distribution Documentation"
    
    print_test "Release directory README"
    local release_readme="$DISTRIBUTION_DIR/release/README.md"
    if [[ -f "$release_readme" ]]; then
        if grep -q "Installation Methods" "$release_readme"; then
            print_pass "Release documentation is comprehensive"
        else
            print_fail "Release documentation is incomplete"
        fi
    else
        print_fail "Release README not found"
    fi
    
    print_test "Installation instructions"
    if [[ -f "$release_readme" ]]; then
        if grep -q "curl.*install" "$release_readme"; then
            print_pass "Universal installer command documented"
        else
            print_fail "Universal installer command not documented"
        fi
    else
        print_skip "Installation instructions test (no README)"
    fi
    
    print_test "Security documentation"
    if [[ -f "$release_readme" ]]; then
        if grep -q "Security\|checksums\|signatures" "$release_readme"; then
            print_pass "Security measures documented"
        else
            print_fail "Security measures not documented"
        fi
    else
        print_skip "Security documentation test (no README)"
    fi
}

test_file_structure() {
    print_section "Distribution File Structure"
    
    print_test "Required directories exist"
    local required_dirs=(
        "$DISTRIBUTION_DIR/build"
        "$DISTRIBUTION_DIR/installers/macos"
        "$DISTRIBUTION_DIR/installers/linux"  
        "$DISTRIBUTION_DIR/installers/windows"
        "$DISTRIBUTION_DIR/packages/homebrew"
        "$DISTRIBUTION_DIR/packages/chocolatey"
        "$DISTRIBUTION_DIR/packages/debian"
        "$DISTRIBUTION_DIR/scripts"
        "$DISTRIBUTION_DIR/ci"
        "$DISTRIBUTION_DIR/release"
        "$DISTRIBUTION_DIR/update_server"
        "$DISTRIBUTION_DIR/testing"
    )
    
    local missing_dirs=0
    for dir in "${required_dirs[@]}"; do
        if [[ -d "$dir" ]]; then
            print_info "✓ $(basename "$dir") directory exists"
        else
            print_info "✗ $(basename "$dir") directory missing"
            missing_dirs=$((missing_dirs + 1))
        fi
    done
    
    if [[ $missing_dirs -eq 0 ]]; then
        print_pass "All required directories exist"
    else
        print_fail "$missing_dirs required directories are missing"
    fi
    
    print_test "Critical files present"
    local required_files=(
        "$DISTRIBUTION_DIR/build_all.sh"
        "$DISTRIBUTION_DIR/test_distribution.sh"
        "$DISTRIBUTION_DIR/scripts/install.sh"
        "$PROJECT_ROOT/Cargo.toml"
    )
    
    local missing_files=0
    for file in "${required_files[@]}"; do
        if [[ -f "$file" ]]; then
            print_info "✓ $(basename "$file") exists"
        else
            print_info "✗ $(basename "$file") missing"
            missing_files=$((missing_files + 1))
        fi
    done
    
    if [[ $missing_files -eq 0 ]]; then
        print_pass "All critical files are present"
    else
        print_fail "$missing_files critical files are missing"
    fi
}

# ============================================
# Integration Tests
# ============================================

test_integration() {
    print_section "Integration Testing"
    
    print_test "Build script syntax validation"
    local build_scripts=("$DISTRIBUTION_DIR/build_all.sh")
    local syntax_errors=0
    
    for script in "${build_scripts[@]}"; do
        if [[ -f "$script" ]]; then
            if bash -n "$script" 2>/dev/null; then
                print_info "✓ $(basename "$script") syntax is valid"
            else
                print_info "✗ $(basename "$script") has syntax errors"
                syntax_errors=$((syntax_errors + 1))
            fi
        fi
    done
    
    if [[ $syntax_errors -eq 0 ]]; then
        print_pass "All build scripts have valid syntax"
    else
        print_fail "$syntax_errors build scripts have syntax errors"
    fi
    
    print_test "Package manager file validation"
    # Test Homebrew formula Ruby syntax
    if command -v ruby >/dev/null 2>&1; then
        if ruby -c "$DISTRIBUTION_DIR/packages/homebrew/hive.rb" >/dev/null 2>&1; then
            print_pass "Homebrew formula has valid Ruby syntax"
        else
            print_fail "Homebrew formula has invalid Ruby syntax"
        fi
    else
        print_skip "Homebrew formula syntax test (Ruby not available)"
    fi
    
    # Test XML files
    if command -v xmllint >/dev/null 2>&1; then
        local xml_files=(
            "$DISTRIBUTION_DIR/packages/chocolatey/hive.nuspec"
            "$DISTRIBUTION_DIR/installers/macos/entitlements.plist"
            "$DISTRIBUTION_DIR/installers/macos/Info.plist"
        )
        
        local xml_errors=0
        for file in "${xml_files[@]}"; do
            if [[ -f "$file" ]]; then
                if xmllint --noout "$file" 2>/dev/null; then
                    print_info "✓ $(basename "$file") is valid XML"
                else
                    print_info "✗ $(basename "$file") has XML errors"
                    xml_errors=$((xml_errors + 1))
                fi
            fi
        done
        
        if [[ $xml_errors -eq 0 ]]; then
            print_pass "All XML files are valid"
        else
            print_fail "$xml_errors XML files have errors"
        fi
    else
        print_skip "XML validation test (xmllint not available)"
    fi
}

# ============================================
# Security Audit
# ============================================

test_security_audit() {
    print_section "Security Audit"
    
    print_test "No hardcoded secrets in configuration files"
    local config_files=(
        "$DISTRIBUTION_DIR/update_server/config.toml"
        "$PROJECT_ROOT/Cargo.toml"
        "$DISTRIBUTION_DIR/packages/homebrew/hive.rb"
    )
    
    local secrets_found=false
    local secret_patterns=("password" "secret" "token" "key.*=" "api.*key")
    
    for file in "${config_files[@]}"; do
        if [[ -f "$file" ]]; then
            for pattern in "${secret_patterns[@]}"; do
                if grep -i "$pattern" "$file" | grep -v "PLACEHOLDER\|example\|template\|_ENV\|\\$" >/dev/null; then
                    print_info "⚠️  Potential secret found in $(basename "$file")"
                    secrets_found=true
                fi
            done
        fi
    done
    
    if $secrets_found; then
        print_fail "Potential hardcoded secrets found"
    else
        print_pass "No hardcoded secrets detected"
    fi
    
    print_test "File permissions security"
    local sensitive_files=(
        "$DISTRIBUTION_DIR/packages/debian/postinst"
        "$DISTRIBUTION_DIR/packages/debian/prerm"
        "$DISTRIBUTION_DIR/packages/debian/postrm"
        "$DISTRIBUTION_DIR/build_all.sh"
        "$DISTRIBUTION_DIR/scripts/install.sh"
    )
    
    local permission_issues=0
    for file in "${sensitive_files[@]}"; do
        if [[ -f "$file" ]]; then
            local perms=$(stat -c "%a" "$file" 2>/dev/null || stat -f "%A" "$file" 2>/dev/null || echo "unknown")
            if [[ "$perms" =~ ^[67][0-7][0-7]$ ]]; then
                print_info "✓ $(basename "$file") has secure permissions ($perms)"
            else
                print_info "⚠️  $(basename "$file") may have insecure permissions ($perms)"
                permission_issues=$((permission_issues + 1))
            fi
        fi
    done
    
    if [[ $permission_issues -eq 0 ]]; then
        print_pass "File permissions are secure"
    else
        print_fail "$permission_issues files may have permission issues"
    fi
}

# ============================================
# Main Execution
# ============================================

main() {
    print_header
    
    # Record start time
    local start_time=$(date +%s)
    
    # Run all test suites
    test_build_system
    test_package_managers
    test_auto_update_system
    test_security_measures
    test_universal_installer
    test_performance_targets
    test_ci_cd_pipeline
    test_documentation
    test_file_structure
    test_integration
    test_security_audit
    
    # Calculate duration
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Print summary
    echo ""
    echo -e "${BLUE}==========================================${NC}"
    echo -e "${BLUE} TEST SUMMARY${NC}"
    echo -e "${BLUE}==========================================${NC}"
    echo -e "Total Tests:     ${TESTS_TOTAL}"
    echo -e "${GREEN}Passed:          ${TESTS_PASSED}${NC}"
    echo -e "${RED}Failed:          ${TESTS_FAILED}${NC}"
    echo -e "${YELLOW}Skipped:         ${TESTS_SKIPPED}${NC}"
    echo -e "Duration:        ${duration}s"
    echo -e "Success Rate:    $((TESTS_PASSED * 100 / TESTS_TOTAL))%"
    echo ""
    
    # Write summary to log
    echo "" >> "$TEST_LOG"
    echo "TEST SUMMARY" >> "$TEST_LOG"
    echo "============" >> "$TEST_LOG"
    echo "Total Tests: $TESTS_TOTAL" >> "$TEST_LOG"
    echo "Passed: $TESTS_PASSED" >> "$TEST_LOG"
    echo "Failed: $TESTS_FAILED" >> "$TEST_LOG"
    echo "Skipped: $TESTS_SKIPPED" >> "$TEST_LOG"
    echo "Duration: ${duration}s" >> "$TEST_LOG"
    echo "Success Rate: $((TESTS_PASSED * 100 / TESTS_TOTAL))%" >> "$TEST_LOG"
    echo "QA Test Suite Completed: $(date)" >> "$TEST_LOG"
    
    # Exit with appropriate code
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}🎉 All tests passed! Distribution system is ready for production.${NC}"
        echo ""
        echo "Next steps:"
        echo "1. Run actual build tests: ./distribution/build_all.sh --test"
        echo "2. Test installers on clean systems"
        echo "3. Validate package manager submissions"
        echo "4. Deploy auto-update server"
        echo "5. Create production release"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Review the issues above before proceeding.${NC}"
        echo ""
        echo "Review the test log for details: $TEST_LOG"
        exit 1
    fi
}

# Run the test suite
main "$@"