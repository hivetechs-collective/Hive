#!/bin/bash
# QA Validation Script - Run after EVERY change
# Zero tolerance for compilation errors

set -e

echo "🔍 QA Validation Check..."

# Check for compilation errors
ERROR_COUNT=$(cargo check 2>&1 | grep -E "^error:" | wc -l | tr -d ' ')

if [ "$ERROR_COUNT" -ne "0" ]; then
    echo "❌ VALIDATION FAILED: $ERROR_COUNT compilation errors detected!"
    echo "⚠️  VETO: No changes allowed until errors are fixed"
    cargo check 2>&1 | grep -E "^error:" | head -10
    exit 1
fi

# Count warnings
WARNING_COUNT=$(cargo check 2>&1 | grep -E "^warning:" | wc -l | tr -d ' ')

# Quick build time check
START_TIME=$(date +%s)
cargo check --quiet
END_TIME=$(date +%s)
BUILD_TIME=$((END_TIME - START_TIME))

echo "✅ Compilation Status: PASSING (0 errors)"
echo "⚠️  Warnings: $WARNING_COUNT"
echo "⏱️  Check Time: ${BUILD_TIME}s"

if [ "$BUILD_TIME" -gt "60" ]; then
    echo "⚠️  WARNING: Build time exceeds 60s threshold"
fi

# Summary
echo ""
echo "📊 QA Summary:"
echo "  - Errors: 0 ✅"
echo "  - Warnings: $WARNING_COUNT"
echo "  - Status: CLEARED FOR COMMIT"

exit 0