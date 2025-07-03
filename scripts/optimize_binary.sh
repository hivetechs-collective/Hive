#!/bin/bash
#
# Binary Optimization Script
# Optimizes binary size and loading for maximum performance

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 Starting binary optimization..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to get binary size
get_binary_size() {
    local binary_path="$1"
    if [[ -f "$binary_path" ]]; then
        stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path"
    else
        echo "0"
    fi
}

# Function to format bytes
format_bytes() {
    local bytes=$1
    if (( bytes > 1048576 )); then
        echo "$(( bytes / 1048576 ))MB"
    elif (( bytes > 1024 )); then
        echo "$(( bytes / 1024 ))KB"
    else
        echo "${bytes}B"
    fi
}

cd "$PROJECT_ROOT"

# Clean previous builds
log_info "Cleaning previous builds..."
cargo clean

# Get baseline build
log_info "Building baseline release binary..."
cargo build --release
baseline_size=$(get_binary_size "target/release/hive")
log_info "Baseline binary size: $(format_bytes $baseline_size)"

# Optimize with production profile
log_info "Building with production profile..."
cargo build --profile production
production_size=$(get_binary_size "target/production/hive")
log_info "Production binary size: $(format_bytes $production_size)"

# Enable profile-guided optimization if supported
if command -v rustc >/dev/null 2>&1; then
    rustc_version=$(rustc --version)
    log_info "Rust version: $rustc_version"
    
    # Check if PGO is available
    if rustc --help | grep -q "profile-generate"; then
        log_info "Profile-guided optimization available, building with PGO..."
        
        # Step 1: Build with profiling
        RUSTFLAGS="-Cprofile-generate=/tmp/hive-pgo" cargo build --profile production
        
        # Step 2: Run typical workload for profiling
        log_info "Running profiling workload..."
        ./target/production/hive --version >/dev/null 2>&1 || true
        ./target/production/hive help >/dev/null 2>&1 || true
        
        # Step 3: Merge profiles and rebuild
        if command -v llvm-profdata >/dev/null 2>&1; then
            llvm-profdata merge -o /tmp/hive-pgo/merged.profdata /tmp/hive-pgo/*.profraw
            RUSTFLAGS="-Cprofile-use=/tmp/hive-pgo/merged.profdata" cargo build --profile production
            pgo_size=$(get_binary_size "target/production/hive")
            log_info "PGO binary size: $(format_bytes $pgo_size)"
        else
            log_warning "llvm-profdata not found, skipping PGO merge step"
        fi
        
        # Cleanup
        rm -rf /tmp/hive-pgo
    else
        log_warning "Profile-guided optimization not available in this Rust version"
    fi
fi

# Strip additional symbols if not already done
log_info "Stripping additional symbols..."
if command -v strip >/dev/null 2>&1; then
    cp "target/production/hive" "target/production/hive-stripped"
    strip "target/production/hive-stripped"
    stripped_size=$(get_binary_size "target/production/hive-stripped")
    log_info "Stripped binary size: $(format_bytes $stripped_size)"
else
    log_warning "strip command not found, skipping additional stripping"
fi

# Compress binary if UPX is available
if command -v upx >/dev/null 2>&1; then
    log_info "Compressing binary with UPX..."
    cp "target/production/hive" "target/production/hive-compressed"
    upx --best "target/production/hive-compressed" >/dev/null 2>&1 || {
        log_warning "UPX compression failed, binary may not be compatible"
        rm -f "target/production/hive-compressed"
    }
    
    if [[ -f "target/production/hive-compressed" ]]; then
        compressed_size=$(get_binary_size "target/production/hive-compressed")
        log_info "Compressed binary size: $(format_bytes $compressed_size)"
    fi
else
    log_warning "UPX not found, skipping compression"
fi

# Check for dynamic dependencies
log_info "Checking dynamic dependencies..."
if command -v ldd >/dev/null 2>&1; then
    ldd_output=$(ldd "target/production/hive" 2>/dev/null || echo "Static binary or ldd failed")
    echo "$ldd_output"
elif command -v otool >/dev/null 2>&1; then
    otool_output=$(otool -L "target/production/hive" 2>/dev/null || echo "Static binary or otool failed")
    echo "$otool_output"
else
    log_warning "No dependency checking tool found"
fi

# Test startup time
log_info "Testing startup time..."
startup_times=()
for i in {1..10}; do
    start_time=$(date +%s%N)
    ./target/production/hive --version >/dev/null 2>&1
    end_time=$(date +%s%N)
    startup_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    startup_times+=($startup_time)
done

# Calculate average startup time
total=0
for time in "${startup_times[@]}"; do
    total=$((total + time))
done
avg_startup=$((total / ${#startup_times[@]}))

log_info "Average startup time: ${avg_startup}ms"

# Check if we meet the target (<25ms)
if (( avg_startup < 25 )); then
    log_success "✅ Startup time target met: ${avg_startup}ms < 25ms"
else
    log_warning "⚠️  Startup time target missed: ${avg_startup}ms >= 25ms"
fi

# Memory usage test
log_info "Testing memory usage..."
if command -v /usr/bin/time >/dev/null 2>&1; then
    memory_output=$(/usr/bin/time -v ./target/production/hive --version 2>&1 | grep "Maximum resident set size" || echo "Memory measurement failed")
    log_info "Memory usage: $memory_output"
elif command -v time >/dev/null 2>&1; then
    # macOS/BSD time doesn't have -v flag, use different approach
    log_warning "Using basic memory measurement (less accurate)"
fi

# Summary
echo ""
echo "📊 OPTIMIZATION SUMMARY"
echo "======================="
echo "Baseline size:    $(format_bytes $baseline_size)"
echo "Production size:  $(format_bytes $production_size)"
if [[ -n "${pgo_size:-}" ]]; then
    echo "PGO size:         $(format_bytes $pgo_size)"
fi
if [[ -n "${stripped_size:-}" ]]; then
    echo "Stripped size:    $(format_bytes $stripped_size)"
fi
if [[ -n "${compressed_size:-}" ]]; then
    echo "Compressed size:  $(format_bytes $compressed_size)"
fi
echo "Avg startup time: ${avg_startup}ms"

# Size reduction calculation
if (( production_size < baseline_size )); then
    reduction=$(( (baseline_size - production_size) * 100 / baseline_size ))
    log_success "Binary size reduced by ${reduction}%"
else
    log_warning "No size reduction achieved"
fi

# Final recommendations
echo ""
echo "💡 RECOMMENDATIONS"
echo "=================="

if (( avg_startup >= 25 )); then
    echo "• Startup time optimization needed (target: <25ms)"
fi

if (( production_size > 10485760 )); then # 10MB
    echo "• Consider removing unused dependencies"
    echo "• Review feature flags and disable unused features"
fi

echo "• Enable LTO and codegen-units=1 in release profile ✓"
echo "• Use 'opt-level = \"z\"' for size optimization ✓"
echo "• Strip symbols in production builds ✓"

if ! command -v upx >/dev/null 2>&1; then
    echo "• Install UPX for binary compression"
fi

log_success "Binary optimization complete!"
echo ""
echo "🚀 Optimized binary available at: target/production/hive"