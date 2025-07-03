#!/bin/bash
# Performance test for AST parsing

echo "🧪 Running AST Performance Test"
echo "================================"

# Create test directory with 1000 files
TEST_DIR="/tmp/hive_perf_test"
rm -rf $TEST_DIR
mkdir -p $TEST_DIR

echo "📝 Creating 1000 test files..."
for i in {1..1000}; do
    cat > "$TEST_DIR/test_$i.rs" << EOF
// Test file $i
fn function_$i() -> Result<(), Box<dyn std::error::Error>> {
    let data = vec![1, 2, 3, 4, 5];
    let sum: i32 = data.iter().sum();
    println!("Sum: {}", sum);
    Ok(())
}

impl MyStruct$i {
    fn new() -> Self {
        Self { value: 42 }
    }
}
EOF
done

echo "🚀 Analyzing 1000 files..."
time cargo run --release -- analyze $TEST_DIR 2>&1 | grep -E "(Files Analyzed:|Analysis Time:|Performance Target:)"

# Cleanup
rm -rf $TEST_DIR

echo "✅ Test complete!"