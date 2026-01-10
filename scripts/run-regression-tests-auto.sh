#!/bin/bash
# Automated Regression Test Runner
# Discovers and runs all regression tests in tests/regression/
# Auto-discovers new tests - just add files, they run automatically

set -e  # Exit on first failure

echo "🧪 Shimmy Regression Test Suite (Automated)"
echo "=========================================="
echo "Auto-discovering regression tests..."
echo ""

# Track results
PASSED=0
FAILED=0
FAILED_TESTS=()

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find all regression test files
REGRESSION_DIR="tests/regression"
TEST_FILES=$(find "$REGRESSION_DIR" -name "issue_*.rs" -type f | sort)

if [ -z "$TEST_FILES" ]; then
    echo "❌ No regression test files found in $REGRESSION_DIR"
    exit 1
fi

echo "Found $(echo "$TEST_FILES" | wc -l) regression test files:"
echo "$TEST_FILES" | sed 's/^/  📄 /'
echo ""

# Function to extract issue number from filename
get_issue_number() {
    basename "$1" | sed -E 's/issue_([0-9_]+)_.*/\1/' | tr '_' '/' | sed 's|/$||'
}

# Function to run a single regression test
run_regression_test() {
    local test_file="$1"
    local test_name=$(basename "$test_file" .rs)
    local issue_num=$(get_issue_number "$test_file")
    
    echo "🔬 Testing Issue #${issue_num}: ${test_name}"
    
    # Determine cargo features based on test name
    FEATURES=""
    if [[ "$test_name" =~ mlx ]]; then
        FEATURES="--features mlx"
    elif [[ "$test_name" =~ gpu|cuda|opencl|vulkan ]]; then
        FEATURES="--features llama-opencl,llama-vulkan"
    fi
    
    # Run the specific test module from the regression test suite
    # The test target is "regression" and we filter for this specific module
    if cargo test --test regression $FEATURES "${test_name}" &> "${test_name}.log"; then
        echo "   ✅ PASS - Issue #${issue_num} regression test passed"
        PASSED=$((PASSED + 1))
    else
        echo "   ❌ FAIL - Issue #${issue_num} regression test FAILED"
        echo "      See ${test_name}.log for details"
        FAILED=$((FAILED + 1))
        EXIT_CODE=1
    fi
    echo ""
}

# Run all regression tests
for test_file in $TEST_FILES; do
    run_regression_test "$test_file"
done

# Summary
echo "========================================"
echo "📊 Regression Test Results Summary"
echo "========================================"
echo -e "${GREEN}✅ Passed: $PASSED${NC}"
echo -e "${RED}❌ Failed: $FAILED${NC}"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed Tests:${NC}"
    for failed in "${FAILED_TESTS[@]}"; do
        echo "  ❌ $failed"
    done
    echo ""
    echo "🔧 Fix failing regression tests before proceeding"
    echo "   Regression tests prevent previously fixed bugs from returning"
    echo "   ZERO TOLERANCE: All regression tests must pass"
    exit 1
else
    echo -e "${GREEN}🎉 ALL REGRESSION TESTS PASSED${NC}"
    echo "✅ No regressions detected - safe to proceed"
    exit 0
fi
