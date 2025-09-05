#!/bin/bash
#
# Performance Profiling Script
# Comprehensive performance analysis and profiling

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "📊 Starting performance profiling..."

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

cd "$PROJECT_ROOT"

# Create profiling directory
PROFILE_DIR="target/profiling"
mkdir -p "$PROFILE_DIR"

# Build with profiling enabled
log_info "Building with profiling features..."
cargo build --release --features profiling

# Function to run profiling command with error handling
run_profiling() {
    local cmd="$1"
    local output_file="$2"
    local description="$3"
    
    log_info "Running $description..."
    if eval "$cmd" > "$output_file" 2>&1; then
        log_success "$description completed"
    else
        log_warning "$description failed or not available"
        echo "Command failed: $cmd" >> "$output_file"
    fi
}

# 1. CPU Profiling with perf (Linux) or Instruments (macOS)
if command -v perf >/dev/null 2>&1; then
    log_info "CPU profiling with perf..."
    
    # Record performance data
    perf record -g -o "$PROFILE_DIR/perf.data" \
        ./target/release/hive --version \
        2>/dev/null || log_warning "perf record failed"
    
    # Generate reports
    if [[ -f "$PROFILE_DIR/perf.data" ]]; then
        perf report -i "$PROFILE_DIR/perf.data" > "$PROFILE_DIR/cpu_profile.txt" 2>/dev/null || true
        perf annotate -i "$PROFILE_DIR/perf.data" > "$PROFILE_DIR/cpu_annotate.txt" 2>/dev/null || true
        log_success "CPU profiling completed"
    fi
    
elif [[ "$OSTYPE" == "darwin"* ]]; then
    log_info "CPU profiling with Instruments (macOS)..."
    
    # Use system profiler for basic info
    system_profiler SPSoftwareDataType > "$PROFILE_DIR/system_info.txt" 2>/dev/null || true
    
    # Use built-in time command for basic profiling
    /usr/bin/time -l ./target/release/hive --version > "$PROFILE_DIR/time_profile.txt" 2>&1 || true
    
else
    log_warning "No CPU profiling tool available"
fi

# 2. Memory Profiling
if command -v valgrind >/dev/null 2>&1; then
    log_info "Memory profiling with valgrind..."
    
    valgrind --tool=massif \
        --massif-out-file="$PROFILE_DIR/massif.out" \
        ./target/release/hive --version \
        >/dev/null 2>&1 || log_warning "valgrind profiling failed"
    
    if [[ -f "$PROFILE_DIR/massif.out" ]]; then
        if command -v ms_print >/dev/null 2>&1; then
            ms_print "$PROFILE_DIR/massif.out" > "$PROFILE_DIR/memory_profile.txt"
        fi
        log_success "Memory profiling completed"
    fi
    
elif command -v heaptrack >/dev/null 2>&1; then
    log_info "Memory profiling with heaptrack..."
    
    heaptrack ./target/release/hive --version 2>/dev/null || log_warning "heaptrack failed"
    
    # Move heaptrack output to profiling directory
    mv heaptrack.* "$PROFILE_DIR/" 2>/dev/null || true
    
else
    log_warning "No memory profiling tool available"
fi

# 3. I/O Profiling
if command -v iotop >/dev/null 2>&1; then
    log_info "I/O profiling with iotop..."
    
    # Run iotop in batch mode for 5 seconds during execution
    timeout 5s iotop -b -o > "$PROFILE_DIR/io_profile.txt" 2>&1 &
    IOTOP_PID=$!
    
    ./target/release/hive --version >/dev/null 2>&1
    
    wait $IOTOP_PID 2>/dev/null || true
    
elif [[ "$OSTYPE" == "darwin"* ]] && command -v fs_usage >/dev/null 2>&1; then
    log_info "I/O profiling with fs_usage (macOS)..."
    
    # Run fs_usage for the hive process
    timeout 5s sudo fs_usage -w -f filesys ./target/release/hive > "$PROFILE_DIR/io_profile.txt" 2>&1 || {
        log_warning "fs_usage requires sudo privileges"
    }
    
else
    log_warning "No I/O profiling tool available"
fi

# 4. Startup Time Profiling
log_info "Profiling startup time..."

startup_profile() {
    local iterations=50
    local times=()
    local total=0
    
    for i in $(seq 1 $iterations); do
        start_time=$(date +%s%N)
        ./target/release/hive --version >/dev/null 2>&1
        end_time=$(date +%s%N)
        
        # Convert to milliseconds
        time_ms=$(( (end_time - start_time) / 1000000 ))
        times+=($time_ms)
        total=$((total + time_ms))
        
        # Progress indicator
        if (( i % 10 == 0 )); then
            echo -n "."
        fi
    done
    echo ""
    
    # Calculate statistics
    local avg=$((total / iterations))
    
    # Calculate min and max
    local min=${times[0]}
    local max=${times[0]}
    for time in "${times[@]}"; do
        (( time < min )) && min=$time
        (( time > max )) && max=$time
    done
    
    # Calculate median (simplified)
    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}"))
    local median=${sorted[$((iterations / 2))]}
    
    # Calculate standard deviation (simplified)
    local sum_sq_diff=0
    for time in "${times[@]}"; do
        local diff=$((time - avg))
        sum_sq_diff=$((sum_sq_diff + diff * diff))
    done
    local stddev=$(echo "sqrt($sum_sq_diff / $iterations)" | bc -l 2>/dev/null || echo "N/A")
    
    cat > "$PROFILE_DIR/startup_profile.txt" << EOF
Startup Time Profiling Results
==============================
Iterations: $iterations
Average: ${avg}ms
Median: ${median}ms
Min: ${min}ms
Max: ${max}ms
Standard Deviation: ${stddev}ms

Target: <25ms
Status: $( (( avg < 25 )) && echo "✅ PASSED" || echo "❌ FAILED" )

Raw times (ms): ${times[*]}
EOF
    
    log_info "Startup profiling completed: avg=${avg}ms, target=<25ms"
}

startup_profile

# 5. Memory Usage Profiling
log_info "Profiling memory usage..."

memory_profile() {
    local binary="./target/release/hive"
    
    if command -v /usr/bin/time >/dev/null 2>&1; then
        # GNU time (Linux)
        /usr/bin/time -v "$binary" --version > "$PROFILE_DIR/memory_usage.txt" 2>&1 || true
        
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS time
        /usr/bin/time -l "$binary" --version > "$PROFILE_DIR/memory_usage.txt" 2>&1 || true
        
    else
        log_warning "No suitable time command for memory profiling"
        echo "Memory profiling not available" > "$PROFILE_DIR/memory_usage.txt"
    fi
    
    # Also test with a more complex operation if possible
    if [[ -d "src" ]]; then
        log_info "Testing memory usage with file analysis..."
        if command -v /usr/bin/time >/dev/null 2>&1; then
            /usr/bin/time -v "$binary" analyze src/ > "$PROFILE_DIR/memory_usage_analysis.txt" 2>&1 || true
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            /usr/bin/time -l "$binary" analyze src/ > "$PROFILE_DIR/memory_usage_analysis.txt" 2>&1 || true
        fi
    fi
}

memory_profile

# 6. Benchmark Suite
log_info "Running benchmark suite..."

if cargo bench --help >/dev/null 2>&1; then
    cargo bench > "$PROFILE_DIR/benchmarks.txt" 2>&1 || {
        log_warning "Benchmark execution failed"
        echo "Benchmark execution failed" > "$PROFILE_DIR/benchmarks.txt"
    }
else
    log_warning "Cargo bench not available"
    echo "Cargo bench not available" > "$PROFILE_DIR/benchmarks.txt"
fi

# 7. Binary Analysis
log_info "Analyzing binary characteristics..."

binary_analysis() {
    local binary="./target/release/hive"
    
    cat > "$PROFILE_DIR/binary_analysis.txt" << EOF
Binary Analysis
===============
File: $binary
Size: $(stat -c%s "$binary" 2>/dev/null || stat -f%z "$binary") bytes
EOF
    
    # Binary dependencies
    echo "" >> "$PROFILE_DIR/binary_analysis.txt"
    echo "Dependencies:" >> "$PROFILE_DIR/binary_analysis.txt"
    if command -v ldd >/dev/null 2>&1; then
        ldd "$binary" >> "$PROFILE_DIR/binary_analysis.txt" 2>&1 || echo "Static binary or ldd failed" >> "$PROFILE_DIR/binary_analysis.txt"
    elif command -v otool >/dev/null 2>&1; then
        otool -L "$binary" >> "$PROFILE_DIR/binary_analysis.txt" 2>&1 || echo "Static binary or otool failed" >> "$PROFILE_DIR/binary_analysis.txt"
    else
        echo "No dependency analysis tool available" >> "$PROFILE_DIR/binary_analysis.txt"
    fi
    
    # Binary sections
    echo "" >> "$PROFILE_DIR/binary_analysis.txt"
    echo "Binary sections:" >> "$PROFILE_DIR/binary_analysis.txt"
    if command -v size >/dev/null 2>&1; then
        size "$binary" >> "$PROFILE_DIR/binary_analysis.txt" 2>&1 || true
    fi
    
    if command -v nm >/dev/null 2>&1; then
        echo "" >> "$PROFILE_DIR/binary_analysis.txt"
        echo "Symbol count:" >> "$PROFILE_DIR/binary_analysis.txt"
        nm "$binary" 2>/dev/null | wc -l >> "$PROFILE_DIR/binary_analysis.txt" || echo "Symbol analysis failed" >> "$PROFILE_DIR/binary_analysis.txt"
    fi
}

binary_analysis

# 8. Generate Performance Report
log_info "Generating performance report..."

generate_report() {
    local report_file="$PROFILE_DIR/performance_report.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    cat > "$report_file" << EOF
# Performance Profiling Report

Generated: $timestamp
Binary: target/release/hive

## Executive Summary

EOF
    
    # Add startup time summary
    if [[ -f "$PROFILE_DIR/startup_profile.txt" ]]; then
        local avg_time=$(grep "Average:" "$PROFILE_DIR/startup_profile.txt" | cut -d: -f2 | tr -d ' ms')
        local status=$(grep "Status:" "$PROFILE_DIR/startup_profile.txt" | cut -d: -f2)
        
        echo "- **Startup Time**: ${avg_time}ms (target: <25ms) $status" >> "$report_file"
    fi
    
    # Add memory usage summary
    if [[ -f "$PROFILE_DIR/memory_usage.txt" ]]; then
        local max_memory="Unknown"
        if grep -q "Maximum resident set size" "$PROFILE_DIR/memory_usage.txt"; then
            max_memory=$(grep "Maximum resident set size" "$PROFILE_DIR/memory_usage.txt" | awk '{print $6}' | head -1)
            echo "- **Memory Usage**: ${max_memory}KB (target: <20MB)" >> "$report_file"
        elif grep -q "maximum resident set size" "$PROFILE_DIR/memory_usage.txt"; then
            max_memory=$(grep "maximum resident set size" "$PROFILE_DIR/memory_usage.txt" | awk '{print $1}' | head -1)
            echo "- **Memory Usage**: ${max_memory} bytes (target: <20MB)" >> "$report_file"
        fi
    fi
    
    # Add binary size summary
    if [[ -f "$PROFILE_DIR/binary_analysis.txt" ]]; then
        local binary_size=$(grep "Size:" "$PROFILE_DIR/binary_analysis.txt" | cut -d: -f2 | tr -d ' ')
        echo "- **Binary Size**: ${binary_size} bytes" >> "$report_file"
    fi
    
    cat >> "$report_file" << EOF

## Detailed Results

### Startup Time Analysis
$(cat "$PROFILE_DIR/startup_profile.txt" 2>/dev/null || echo "Not available")

### Memory Usage Analysis
$(cat "$PROFILE_DIR/memory_usage.txt" 2>/dev/null || echo "Not available")

### Binary Analysis
$(cat "$PROFILE_DIR/binary_analysis.txt" 2>/dev/null || echo "Not available")

### Benchmark Results
$(cat "$PROFILE_DIR/benchmarks.txt" 2>/dev/null || echo "Not available")

## Recommendations

1. **Startup Time Optimization**
   - Current target: <25ms
   - Consider lazy loading of modules
   - Optimize configuration loading
   - Reduce binary size

2. **Memory Optimization**
   - Current target: <20MB
   - Monitor for memory leaks
   - Optimize data structures
   - Use memory pools for frequent allocations

3. **Performance Monitoring**
   - Regular profiling in CI/CD
   - Performance regression tests
   - Real-world usage patterns

## Files Generated

- CPU Profile: cpu_profile.txt
- Memory Profile: memory_profile.txt
- I/O Profile: io_profile.txt
- Startup Profile: startup_profile.txt
- Memory Usage: memory_usage.txt
- Binary Analysis: binary_analysis.txt
- Benchmark Results: benchmarks.txt

EOF
    
    log_success "Performance report generated: $report_file"
}

generate_report

# 9. Performance Validation
log_info "Validating performance targets..."

validate_targets() {
    local failures=0
    
    echo "🎯 Performance Target Validation"
    echo "==============================="
    
    # Startup time validation
    if [[ -f "$PROFILE_DIR/startup_profile.txt" ]]; then
        local avg_time=$(grep "Average:" "$PROFILE_DIR/startup_profile.txt" | cut -d: -f2 | tr -d ' ms')
        if (( avg_time < 25 )); then
            echo "✅ Startup time: ${avg_time}ms < 25ms target"
        else
            echo "❌ Startup time: ${avg_time}ms >= 25ms target"
            failures=$((failures + 1))
        fi
    else
        echo "⚠️  Startup time: Unable to measure"
        failures=$((failures + 1))
    fi
    
    # Memory usage validation (if measurable)
    if [[ -f "$PROFILE_DIR/memory_usage.txt" ]]; then
        if grep -q "Maximum resident set size" "$PROFILE_DIR/memory_usage.txt"; then
            local max_memory_kb=$(grep "Maximum resident set size" "$PROFILE_DIR/memory_usage.txt" | awk '{print $6}' | head -1)
            local max_memory_mb=$((max_memory_kb / 1024))
            if (( max_memory_mb < 20 )); then
                echo "✅ Memory usage: ${max_memory_mb}MB < 20MB target"
            else
                echo "❌ Memory usage: ${max_memory_mb}MB >= 20MB target"
                failures=$((failures + 1))
            fi
        else
            echo "⚠️  Memory usage: Unable to parse measurement"
        fi
    else
        echo "⚠️  Memory usage: Unable to measure"
    fi
    
    # Binary size validation
    local binary_size=$(stat -c%s "./target/release/hive" 2>/dev/null || stat -f%z "./target/release/hive")
    local binary_size_mb=$((binary_size / 1048576))
    if (( binary_size_mb < 50 )); then
        echo "✅ Binary size: ${binary_size_mb}MB < 50MB target"
    else
        echo "❌ Binary size: ${binary_size_mb}MB >= 50MB target"
        failures=$((failures + 1))
    fi
    
    echo ""
    if (( failures == 0 )); then
        log_success "All performance targets met! 🎉"
    else
        log_warning "$failures performance target(s) not met"
    fi
    
    return $failures
}

validate_targets

# Summary
echo ""
echo "📊 PROFILING SUMMARY"
echo "==================="
echo "Profile data saved to: $PROFILE_DIR"
echo "Main report: $PROFILE_DIR/performance_report.md"
echo ""
echo "Available profiles:"
ls -la "$PROFILE_DIR" | grep -v "^total" | awk '{print "  " $9}' | tail -n +2

log_success "Performance profiling complete!"

echo ""
echo "🔍 To view results:"
echo "  cat $PROFILE_DIR/performance_report.md"
echo "  ls -la $PROFILE_DIR/"