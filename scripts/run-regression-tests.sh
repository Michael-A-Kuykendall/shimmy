#!/bin/bash
# Comprehensive Regression Testing Suite
# Validates all core functionality before releases

echo "🧪 Shimmy Regression Testing Suite"
echo "=================================="
echo "Testing all core functionality to prevent regressions..."
echo ""

# Track overall success
REGRESSION_SUCCESS=true
RESULTS_LOG="regression-results.log"
> "$RESULTS_LOG"

# Function to log results
log_result() {
    local test_name="$1"
    local status="$2" 
    local details="$3"
    
    echo "[$status] $test_name: $details" | tee -a "$RESULTS_LOG"
    if [ "$status" = "FAIL" ]; then
        REGRESSION_SUCCESS=false
    fi
}

echo "🔧 Phase 1: Unit & Integration Tests"
echo "===================================="
if cargo test --lib --features huggingface -- --test-threads=1 > unit-test-output.log 2>&1; then
    UNIT_TESTS=$(grep -c "test result: ok" unit-test-output.log || echo "0")
    log_result "Unit Tests" "PASS" "All unit tests passed"
    echo "✅ Unit Tests: Passed"
else
    log_result "Unit Tests" "FAIL" "Some unit tests failed"
    echo "❌ Unit Tests: Failed (see unit-test-output.log)"
fi

echo ""
echo "🧪 Phase 2: Regression Test Suite" 
echo "================================="
if cargo test --test regression_tests --features huggingface > regression-test-output.log 2>&1; then
    REGRESSION_TESTS=$(grep -c "test result: ok" regression-test-output.log || echo "0")
    log_result "Regression Tests" "PASS" "All regression tests passed"
    echo "✅ Regression Tests: Passed"
else
    log_result "Regression Tests" "FAIL" "Some regression tests failed"
    echo "❌ Regression Tests: Failed (see regression-test-output.log)"
fi

echo ""
echo "🏗️ Phase 3: Build Verification"
echo "=============================="
if cargo build --release --features huggingface > build-output.log 2>&1; then
    log_result "Release Build" "PASS" "Release build succeeded"
    echo "✅ Release Build: Succeeded"
else
    log_result "Release Build" "FAIL" "Release build failed"
    echo "❌ Release Build: Failed (see build-output.log)"
fi

echo ""
echo "🔍 Phase 4: API Compatibility Tests"
echo "==================================="
echo "🔄 Testing model discovery functionality..."
if cargo test test_model_discovery --features huggingface > api-test-output.log 2>&1; then
    log_result "Model Discovery API" "PASS" "Discovery API functional"
    echo "✅ Model Discovery API: Functional"
else
    log_result "Model Discovery API" "FAIL" "Discovery API issues"
    echo "❌ Model Discovery API: Issues (see api-test-output.log)"
fi

echo "🔄 Testing OpenAI API compatibility..."
if cargo test test_openai_api --features huggingface >> api-test-output.log 2>&1; then
    log_result "OpenAI API Compatibility" "PASS" "API responses compatible"
    echo "✅ OpenAI API: Compatible"
else
    log_result "OpenAI API Compatibility" "FAIL" "API compatibility issues"  
    echo "❌ OpenAI API: Issues (see api-test-output.log)"
fi

echo ""
echo "🎯 Phase 5: Issue-Specific Regression Tests"
echo "==========================================="

echo "🔄 Testing Issue #13 fix (Qwen model template detection)..."
if cargo test test_qwen_model_template_detection --features huggingface > issue-fix-output.log 2>&1; then
    log_result "Issue #13 Fix" "PASS" "Qwen models use correct templates"
    echo "✅ Issue #13 (Qwen VSCode): Fixed"
else
    log_result "Issue #13 Fix" "FAIL" "Qwen template detection broken"
    echo "❌ Issue #13 (Qwen VSCode): Regression detected!"
fi

echo "🔄 Testing Issue #12 fix (Custom model directories)..."
if cargo test test_custom_model_directory_environment_variables --features huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #12 Fix" "PASS" "Custom directories detected"
    echo "✅ Issue #12 (Custom dirs): Fixed"
else
    log_result "Issue #12 Fix" "FAIL" "Custom directory detection broken"
    echo "❌ Issue #12 (Custom dirs): Regression detected!"
fi

echo "🔄 Testing CLI compatibility (new --model-dirs option)..."
if cargo test test_cli_model_dirs_option_compatibility --features huggingface >> issue-fix-output.log 2>&1; then
    log_result "CLI Compatibility" "PASS" "CLI options working"
    echo "✅ CLI Options: Working"
else
    log_result "CLI Compatibility" "FAIL" "CLI parsing broken"
    echo "❌ CLI Options: Broken!"
fi

echo ""
echo "🔒 Phase 6: Security & Error Handling"
echo "====================================="
echo "🔄 Testing error handling robustness..."
if cargo test test_error_handling_robustness --features huggingface > security-output.log 2>&1; then
    log_result "Error Handling" "PASS" "Error handling robust"
    echo "✅ Error Handling: Robust"
else
    log_result "Error Handling" "FAIL" "Error handling issues"
    echo "❌ Error Handling: Issues detected!"
fi

echo ""
echo "📏 Phase 7: Code Quality Checks"
echo "==============================="
echo "🎨 Checking code formatting..."
if cargo fmt -- --check > fmt-output.log 2>&1; then
    log_result "Code Formatting" "PASS" "Code properly formatted"
    echo "✅ Code Formatting: Correct"
else
    log_result "Code Formatting" "FAIL" "Code formatting issues"
    echo "❌ Code Formatting: Issues (run 'cargo fmt')"
fi

echo "🔍 Running clippy lints..."
if cargo clippy --features huggingface -- -D warnings > clippy-output.log 2>&1; then
    log_result "Clippy Lints" "PASS" "No lint warnings"
    echo "✅ Clippy Lints: Clean"
else
    WARNINGS=$(grep -c "warning:" clippy-output.log || echo "0")
    log_result "Clippy Lints" "FAIL" "$WARNINGS warnings found"
    echo "⚠️  Clippy Lints: $WARNINGS warnings found"
fi

echo ""
echo "📊 REGRESSION TEST SUMMARY"
echo "=========================="
echo ""
echo "📋 Test Results:"
cat "$RESULTS_LOG" | while read line; do
    if [[ $line == *"[PASS]"* ]]; then
        echo "  ✅ $line"
    elif [[ $line == *"[FAIL]"* ]]; then
        echo "  ❌ $line"
    else
        echo "  ℹ️  $line"
    fi
done

echo ""
echo "📁 Generated Files:"
echo "  📊 regression-results.log - Complete results"
echo "  📋 *-output.log - Detailed test logs"

echo ""
if [ "$REGRESSION_SUCCESS" = true ]; then
    echo "🎉 REGRESSION TESTING: ALL TESTS PASSED"
    echo "✅ Safe to proceed with release!"
    echo ""
    echo "🚀 Next steps:"
    echo "  1. Update version in Cargo.toml"
    echo "  2. Update CHANGELOG.md"
    echo "  3. Create git tag and push"
    echo "  4. Trigger release workflow"
    exit 0
else
    echo "⚠️  REGRESSION TESTING: SOME TESTS FAILED"
    echo "🔧 Please fix failing tests before release"
    echo ""
    echo "🔍 Check these files for details:"
    echo "  - regression-results.log"
    echo "  - *-output.log files"
    exit 1
fi