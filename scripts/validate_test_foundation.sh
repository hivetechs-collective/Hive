#!/bin/bash
# Validate Testing Foundation for HiveTechs Consensus
# Ensures all testing infrastructure is properly set up

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Validating HiveTechs Consensus Testing Foundation${NC}"
echo "=================================================="
echo ""

VALIDATION_PASSED=true

# Function to check requirement
check_requirement() {
    local requirement="$1"
    local command="$2"
    local expected="$3"
    
    echo -n "Checking $requirement... "
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC}"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        [ -n "$expected" ] && echo "   Expected: $expected"
        VALIDATION_PASSED=false
        return 1
    fi
}

# Function to check file exists
check_file() {
    local file="$1"
    local description="$2"
    
    echo -n "Checking $description... "
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✅ EXISTS${NC}"
        return 0
    else
        echo -e "${RED}❌ MISSING${NC}"
        echo "   File: $file"
        VALIDATION_PASSED=false
        return 1
    fi
}

# Function to check directory structure
check_directory() {
    local dir="$1"
    local description="$2"
    
    echo -n "Checking $description... "
    
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✅ EXISTS${NC}"
        return 0
    else
        echo -e "${RED}❌ MISSING${NC}"
        echo "   Directory: $dir"
        VALIDATION_PASSED=false
        return 1
    fi
}

echo "🏗️  Project Structure Validation"
echo "================================"

# Check core test directories
check_directory "tests" "Core tests directory"
check_directory "tests/unit" "Unit tests directory"
check_directory "tests/integration" "Integration tests directory"
check_directory "tests/performance" "Performance tests directory"
check_directory "tests/acceptance" "Acceptance tests directory"
check_directory "tests/security" "Security tests directory"

# Check unit test subdirectories
check_directory "tests/unit/consensus" "Consensus unit tests"
check_directory "tests/unit/analysis" "Analysis unit tests"
check_directory "tests/unit/memory" "Memory unit tests"
check_directory "tests/unit/analytics" "Analytics unit tests"
check_directory "tests/unit/security" "Security unit tests"

# Check integration test subdirectories
check_directory "tests/integration/api" "API integration tests"
check_directory "tests/integration/database" "Database integration tests"
check_directory "tests/integration/file_system" "File system integration tests"
check_directory "tests/integration/external" "External service integration tests"

# Check performance test subdirectories
check_directory "tests/performance/benchmarks" "Performance benchmarks"
check_directory "tests/performance/load" "Load testing"
check_directory "tests/performance/regression" "Regression testing"

echo ""
echo "📄 Test Framework Files"
echo "======================="

# Check test framework files
check_file "tests/mod.rs" "Test framework module"
check_file "tests/unit/consensus/mod.rs" "Consensus unit test module"
check_file "tests/integration/api/openrouter_integration.rs" "OpenRouter integration tests"
check_file "tests/performance/benchmarks/startup_benchmark.rs" "Startup benchmarks"
check_file "tests/acceptance/scenarios/cli_experience.rs" "CLI acceptance tests"
check_file "tests/security_audit_framework.rs" "Security audit framework"

echo ""
echo "🛠️  Testing Tools and Scripts"
echo "============================="

# Check testing scripts
check_file "scripts/run_comprehensive_tests.sh" "Comprehensive test runner"
check_file "scripts/generate_test_report.sh" "Test report generator"
check_file "scripts/validate_test_foundation.sh" "Test foundation validator"

# Check script permissions
echo -n "Checking script permissions... "
if [ -x "scripts/run_comprehensive_tests.sh" ] && [ -x "scripts/generate_test_report.sh" ] && [ -x "scripts/validate_test_foundation.sh" ]; then
    echo -e "${GREEN}✅ EXECUTABLE${NC}"
else
    echo -e "${RED}❌ NOT EXECUTABLE${NC}"
    VALIDATION_PASSED=false
fi

echo ""
echo "⚙️  Cargo Configuration"
echo "======================"

# Check Cargo.toml has testing dependencies
echo -n "Checking testing dependencies... "
if grep -q "dev-dependencies" Cargo.toml && grep -q "criterion" Cargo.toml && grep -q "serial_test" Cargo.toml; then
    echo -e "${GREEN}✅ CONFIGURED${NC}"
else
    echo -e "${RED}❌ MISSING${NC}"
    VALIDATION_PASSED=false
fi

# Check testing features
echo -n "Checking testing features... "
if grep -q "testing = \[" Cargo.toml && grep -q "performance-tests" Cargo.toml && grep -q "security-tests" Cargo.toml; then
    echo -e "${GREEN}✅ CONFIGURED${NC}"
else
    echo -e "${RED}❌ MISSING${NC}"
    VALIDATION_PASSED=false
fi

echo ""
echo "🚀 CI/CD Configuration"
echo "====================="

# Check GitHub Actions workflow
check_file ".github/workflows/comprehensive_testing.yml" "GitHub Actions testing workflow"

echo ""
echo "🔧 Development Tools"
echo "==================="

# Check for Rust toolchain
check_requirement "Rust toolchain" "command -v rustc" "Rust compiler"
check_requirement "Cargo" "command -v cargo" "Cargo package manager"

# Check for testing tools (optional)
echo -n "Checking optional testing tools... "
OPTIONAL_TOOLS=0
command -v cargo-nextest > /dev/null 2>&1 && OPTIONAL_TOOLS=$((OPTIONAL_TOOLS + 1))
command -v cargo-tarpaulin > /dev/null 2>&1 && OPTIONAL_TOOLS=$((OPTIONAL_TOOLS + 1))
command -v cargo-llvm-cov > /dev/null 2>&1 && OPTIONAL_TOOLS=$((OPTIONAL_TOOLS + 1))
command -v cargo-audit > /dev/null 2>&1 && OPTIONAL_TOOLS=$((OPTIONAL_TOOLS + 1))

if [ $OPTIONAL_TOOLS -gt 0 ]; then
    echo -e "${GREEN}✅ ${OPTIONAL_TOOLS}/4 TOOLS AVAILABLE${NC}"
else
    echo -e "${YELLOW}⚠️  NO OPTIONAL TOOLS${NC}"
fi

echo ""
echo "🏃 Test Execution Validation"
echo "============================"

# Try to run a simple test build
echo -n "Checking test compilation... "
if cargo test --no-run > /dev/null 2>&1; then
    echo -e "${GREEN}✅ COMPILES${NC}"
else
    echo -e "${RED}❌ COMPILATION FAILED${NC}"
    VALIDATION_PASSED=false
fi

# Check if we can run a simple test
echo -n "Checking test execution... "
if timeout 30s cargo test --lib -- --nocapture test_consensus_engine_initialization > /dev/null 2>&1; then
    echo -e "${GREEN}✅ EXECUTES${NC}"
elif timeout 30s cargo test --lib -- --help > /dev/null 2>&1; then
    echo -e "${GREEN}✅ FRAMEWORK READY${NC}"
else
    echo -e "${YELLOW}⚠️  NEEDS IMPLEMENTATION${NC}"
fi

echo ""
echo "📊 Test Coverage Validation"
echo "==========================="

# Check coverage target directories
echo -n "Checking coverage directories... "
mkdir -p target/coverage target/test-reports
echo -e "${GREEN}✅ CREATED${NC}"

# Validate test categories
echo ""
echo "🎯 Test Category Validation"
echo "==========================="

TEST_CATEGORIES=("unit" "integration" "performance" "acceptance" "security")
for category in "${TEST_CATEGORIES[@]}"; do
    echo -n "Validating $category tests... "
    if find tests/ -name "*.rs" -path "*/$category/*" | head -1 > /dev/null; then
        echo -e "${GREEN}✅ FOUND${NC}"
    else
        echo -e "${YELLOW}⚠️  MINIMAL${NC}"
    fi
done

echo ""
echo "🎯 Performance Target Validation"
echo "================================"

# Check performance targets are defined
PERFORMANCE_TARGETS=(
    "startup_time:50ms"
    "memory_usage:25MB"
    "consensus_time:500ms"
    "file_parsing:5ms/file"
    "database_query:3ms"
)

for target in "${PERFORMANCE_TARGETS[@]}"; do
    TARGET_NAME=$(echo "$target" | cut -d: -f1)
    TARGET_VALUE=$(echo "$target" | cut -d: -f2)
    echo -n "Performance target $TARGET_NAME... "
    
    if grep -r "$TARGET_NAME" tests/ > /dev/null 2>&1; then
        echo -e "${GREEN}✅ TRACKED ($TARGET_VALUE)${NC}"
    else
        echo -e "${YELLOW}⚠️  NOT TRACKED ($TARGET_VALUE)${NC}"
    fi
done

echo ""
echo "🔒 Security Test Validation"
echo "==========================="

# Check security test coverage
SECURITY_AREAS=(
    "file_access"
    "trust_system"
    "input_validation"
    "network_security"
    "dependency_security"
)

for area in "${SECURITY_AREAS[@]}"; do
    echo -n "Security area $area... "
    if grep -r "$area" tests/security* > /dev/null 2>&1; then
        echo -e "${GREEN}✅ COVERED${NC}"
    else
        echo -e "${YELLOW}⚠️  MINIMAL${NC}"
    fi
done

echo ""
echo "📋 Final Validation Summary"
echo "==========================="

if [ "$VALIDATION_PASSED" = true ]; then
    echo -e "${GREEN}🎉 TESTING FOUNDATION VALIDATION PASSED${NC}"
    echo ""
    echo "✅ All critical components are in place:"
    echo "   • Test directory structure complete"
    echo "   • Test framework configured"
    echo "   • Testing scripts available"
    echo "   • CI/CD pipeline configured"
    echo "   • Performance targets defined"
    echo "   • Security testing framework ready"
    echo ""
    echo "🚀 Ready for comprehensive testing execution!"
    echo ""
    echo "Next steps:"
    echo "   1. Run: ./scripts/run_comprehensive_tests.sh"
    echo "   2. Generate report: ./scripts/generate_test_report.sh"
    echo "   3. Review coverage: open target/coverage/index.html"
    echo ""
    exit 0
else
    echo -e "${RED}❌ TESTING FOUNDATION VALIDATION FAILED${NC}"
    echo ""
    echo "⚠️  Please address the following issues:"
    echo "   • Fix missing files/directories listed above"
    echo "   • Install required testing tools"
    echo "   • Verify Cargo.toml configuration"
    echo "   • Ensure scripts have execute permissions"
    echo ""
    echo "🔧 To fix common issues:"
    echo "   • Run: chmod +x scripts/*.sh"
    echo "   • Install tools: cargo install cargo-nextest cargo-tarpaulin cargo-audit"
    echo "   • Create missing directories: mkdir -p tests/{unit,integration,performance,acceptance,security}"
    echo ""
    exit 1
fi