#!/bin/bash
# Comprehensive Regression Testing Suite
# Validates all core functionality before releases

set -x  # Enable debug mode to see every command
echo "🧪 Shimmy Regression Testing Suite"
echo "=================================="
echo "Testing all core functionality to prevent regressions..."
echo ""
echo "[DEBUG] Script started at $(date)" | tee -a debug-regression.log

# Track overall success
REGRESSION_SUCCESS=true
RESULTS_LOG="regression-results.log"
> "$RESULTS_LOG"
echo "[DEBUG] Log file initialized" | tee -a debug-regression.log

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
echo "[DEBUG] Starting Phase 1 at $(date)" | tee -a debug-regression.log
if cargo test --lib --features huggingface > unit-test-output.log 2>&1; then
    echo "[DEBUG] Phase 1 cargo test completed successfully" | tee -a debug-regression.log
    UNIT_TESTS=$(grep -c "test result: ok" unit-test-output.log || echo "0")
    log_result "Unit Tests" "PASS" "All unit tests passed"
    echo "✅ Unit Tests: Passed"
else
    echo "[DEBUG] Phase 1 cargo test FAILED" | tee -a debug-regression.log
    log_result "Unit Tests" "FAIL" "Some unit tests failed"
    echo "❌ Unit Tests: Failed (see unit-test-output.log)"
fi
echo "[DEBUG] Phase 1 completed at $(date)" | tee -a debug-regression.log

echo ""
echo "🧪 Phase 2: Regression Test Suite"
echo "================================="
echo "[DEBUG] Starting Phase 2 at $(date)" | tee -a debug-regression.log
if cargo test --test core --test handlers --test compile_checks --features airframe,huggingface > regression-test-output.log 2>&1; then
    echo "[DEBUG] Phase 2 cargo test completed successfully" | tee -a debug-regression.log
    REGRESSION_TESTS=$(grep -c "test result: ok" regression-test-output.log || echo "0")
    log_result "Regression Tests" "PASS" "All regression tests passed"
    echo "✅ Regression Tests: Passed"
else
    echo "[DEBUG] Phase 2 cargo test FAILED" | tee -a debug-regression.log
    log_result "Regression Tests" "FAIL" "Some regression tests failed"
    echo "❌ Regression Tests: Failed (see regression-test-output.log)"
fi
echo "[DEBUG] Phase 2 completed at $(date)" | tee -a debug-regression.log

echo ""
echo "🏗️ Phase 3: Build Verification"
echo "=============================="
echo "[DEBUG] Starting Phase 3 at $(date)" | tee -a debug-regression.log
if cargo build --release --features huggingface > build-output.log 2>&1; then
    echo "[DEBUG] Phase 3 build completed successfully" | tee -a debug-regression.log
    log_result "Release Build" "PASS" "Release build succeeded"
    echo "✅ Release Build: Succeeded"
else
    echo "[DEBUG] Phase 3 build FAILED" | tee -a debug-regression.log
    log_result "Release Build" "FAIL" "Release build failed"
    echo "❌ Release Build: Failed (see build-output.log)"
fi
echo "[DEBUG] Phase 3 completed at $(date)" | tee -a debug-regression.log

echo ""
echo "🔍 Phase 4: Core & Handler Test Suite"
echo "======================================"
echo "🔄 Testing core module (CLI, registry, templates, serde, SSE, discovery)..."
if cargo test --test core --features airframe,huggingface > api-test-output.log 2>&1; then
    log_result "Core Tests" "PASS" "All core tests passed"
    echo "✅ Core Tests: Passed"
else
    log_result "Core Tests" "FAIL" "Some core tests failed"
    echo "❌ Core Tests: Failed (see api-test-output.log)"
fi

echo "🔄 Testing handler endpoints (health, models, chat, tags, concurrency)..."
if cargo test --test handlers --features airframe,huggingface >> api-test-output.log 2>&1; then
    log_result "Handler Tests" "PASS" "All handler tests passed"
    echo "✅ Handler Tests: Passed"
else
    log_result "Handler Tests" "FAIL" "Some handler tests failed"
    echo "❌ Handler Tests: Failed (see api-test-output.log)"
fi

echo ""
echo "🎯 Phase 5: Issue-Specific Regression Tests"
echo "==========================================="

echo "🔄 Testing template file compilation (Issues #64, #73, #86, #88)..."
if cargo test --test compile_checks > issue-fix-output.log 2>&1; then
    log_result "Template Compilation" "PASS" "All template files accessible"
    echo "✅ Template Compilation: Passed"
else
    log_result "Template Compilation" "FAIL" "Template files missing or broken"
    echo "❌ Template Compilation: Failed (see issue-fix-output.log)"
fi

echo "🔄 Testing Issue #13 fix (Qwen model template detection)..."
if cargo test --test core test_template_auto_detection --features airframe,huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #13 Fix" "PASS" "Qwen models use correct templates"
    echo "✅ Issue #13 (Qwen VSCode): Fixed"
else
    log_result "Issue #13 Fix" "FAIL" "Qwen template detection broken"
    echo "❌ Issue #13 (Qwen VSCode): Regression detected!"
fi

echo "🔄 Testing Issue #12 fix (Custom model directories)..."
if cargo test --test core test_custom_model_directory_env_vars --features airframe,huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #12 Fix" "PASS" "Custom directories detected"
    echo "✅ Issue #12 (Custom dirs): Fixed"
else
    log_result "Issue #12 Fix" "FAIL" "Custom directory detection broken"
    echo "❌ Issue #12 (Custom dirs): Regression detected!"
fi

echo "🔄 Testing Issue #53 fix (SSE streaming format)..."
if cargo test --test core test_sse_streaming_chunk_format --features airframe,huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #53 Fix" "PASS" "SSE streaming format correct"
    echo "✅ Issue #53 (SSE format): Fixed"
else
    log_result "Issue #53 Fix" "FAIL" "SSE streaming format broken"
    echo "❌ Issue #53 (SSE format): Regression detected!"
fi

echo "🔄 Testing Issue #65 fix (Error handling for missing models)..."
if cargo test --test core test_error_response_json_shape --features airframe,huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #65 Fix" "PASS" "Error response format correct"
    echo "✅ Issue #65 (404 error): Fixed"
else
    log_result "Issue #65 Fix" "FAIL" "Error response format broken"
    echo "❌ Issue #65 (404 error): Regression detected!"
fi

echo "🔄 Testing Issue #112 fix (SafeTensors engine)..."
if cargo test --test core test_safetensors_extension_detection --features airframe,huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #112 Fix" "PASS" "SafeTensors extension detection working"
    echo "✅ Issue #112 (SafeTensors): Fixed"
else
    log_result "Issue #112 Fix" "FAIL" "SafeTensors detection broken"
    echo "❌ Issue #112 (SafeTensors): Regression detected!"
fi

echo "🔄 Testing Issue #113 fix (OpenAI API frontend compatibility)..."
if cargo test --test core test_model_struct_completeness --features airframe,huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #113 Fix" "PASS" "OpenAI API structure complete"
    echo "✅ Issue #113 (Frontend compat): Fixed"
else
    log_result "Issue #113 Fix" "FAIL" "OpenAI API structure broken"
    echo "❌ Issue #113 (Frontend compat): Regression detected!"
fi

echo "🔄 Testing Issue #191 fix (Multi-part content arrays)..."
if cargo test --test core test_multi_part_content_array_deserialization --features airframe,huggingface >> issue-fix-output.log 2>&1; then
    log_result "Issue #191 Fix" "PASS" "Multi-part content deserialization working"
    echo "✅ Issue #191 (422 fix): Fixed"
else
    log_result "Issue #191 Fix" "FAIL" "Multi-part content deserialization broken"
    echo "❌ Issue #191 (422 fix): Regression detected!"
fi

echo ""
echo "🔒 Phase 6: Security & Error Handling"
echo "====================================="
echo "🔄 Testing error handling robustness..."
if cargo test --test core test_registry_error_handling --features airframe,huggingface > security-output.log 2>&1; then
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
