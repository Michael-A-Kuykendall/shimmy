#!/bin/bash
# Comprehensive Development Test Suite
# Professional-grade local testing for Shimmy AI Inference Engine

set -e

echo "🚀 Shimmy Development Test Suite"
echo "================================="
echo "Running comprehensive quality checks..."
echo ""

# Track overall success
OVERALL_SUCCESS=true
RESULTS_LOG="test-results.log"
> "$RESULTS_LOG"

# Function to log results
log_result() {
    local test_name="$1"
    local status="$2"
    local details="$3"
    
    echo "[$status] $test_name: $details" | tee -a "$RESULTS_LOG"
    if [ "$status" = "FAIL" ]; then
        OVERALL_SUCCESS=false
    fi
}

echo "🧪 Phase 1: PPT Contract Tests (Critical)"
echo "=========================================="
if timeout 120s cargo test ppt_contracts --features huggingface -- --nocapture > ppt-output.log 2>&1; then
    CONTRACT_TESTS=$(grep -c "Contract test.*passed" ppt-output.log || echo "0")
    log_result "PPT Contract Tests" "PASS" "$CONTRACT_TESTS contract tests passed"
    echo "✅ PPT Contract Tests: $CONTRACT_TESTS tests passed"
else
    log_result "PPT Contract Tests" "FAIL" "Tests failed or timed out"
    echo "❌ PPT Contract Tests: Failed or timed out"
    echo "   See ppt-output.log for details"
fi
echo ""

echo "🧪 Phase 2: Property Tests"  
echo "=========================="
if timeout 60s cargo test property_tests --features huggingface -- --nocapture > property-output.log 2>&1; then
    PROPERTY_TESTS=$(grep -c "Property test.*passed" property-output.log || echo "0")
    log_result "Property Tests" "PASS" "$PROPERTY_TESTS property tests passed"
    echo "✅ Property Tests: $PROPERTY_TESTS tests passed"
else
    log_result "Property Tests" "FAIL" "Tests failed or timed out"
    echo "❌ Property Tests: Failed or timed out"
fi
echo ""

echo "🧪 Phase 3: Unit Tests"
echo "======================"
if timeout 180s cargo test --lib --features huggingface > unit-output.log 2>&1; then
    UNIT_TESTS=$(grep -c "test result: ok" unit-output.log || echo "0")
    log_result "Unit Tests" "PASS" "Library tests passed"
    echo "✅ Unit Tests: Library tests passed"
else
    log_result "Unit Tests" "FAIL" "Unit tests failed"
    echo "❌ Unit Tests: Failed"
fi
echo ""

echo "📊 Phase 4: Code Coverage Analysis"
echo "=================================="
echo "🔄 Generating coverage report (this may take a few minutes)..."
if timeout 300s cargo tarpaulin --features huggingface --out xml --output-dir coverage > coverage-output.log 2>&1; then
    if [ -f "coverage/cobertura.xml" ]; then
        COVERAGE_PERCENT=$(grep -o 'line-rate="[^"]*"' coverage/cobertura.xml | head -1 | grep -o '[0-9.]*' || echo "0")
        COVERAGE_FORMATTED=$(echo "$COVERAGE_PERCENT * 100" | bc -l 2>/dev/null | xargs printf "%.1f" 2>/dev/null || echo "0.0")
        
        MEETS_STANDARD=$(echo "$COVERAGE_PERCENT >= 0.95" | bc -l 2>/dev/null || echo "0")
        if [ "$MEETS_STANDARD" -eq 1 ]; then
            log_result "Code Coverage" "PASS" "${COVERAGE_FORMATTED}% (meets 95% standard)"
            echo "✅ Code Coverage: ${COVERAGE_FORMATTED}% (meets 95% standard)"
        else
            log_result "Code Coverage" "FAIL" "${COVERAGE_FORMATTED}% (below 95% standard)" 
            echo "⚠️  Code Coverage: ${COVERAGE_FORMATTED}% (below 95% standard)"
        fi
    else
        log_result "Code Coverage" "FAIL" "Coverage report not generated"
        echo "❌ Code Coverage: Report generation failed"
    fi
else
    log_result "Code Coverage" "FAIL" "Coverage analysis timed out"
    echo "❌ Code Coverage: Analysis timed out"
fi
echo ""

echo "🔒 Phase 5: Security Scanning" 
echo "============================="
echo "🔍 Checking for known vulnerabilities..."
if cargo audit > audit-output.log 2>&1; then
    VULNS=$(grep -c "error:" audit-output.log || echo "0")
    if [ "$VULNS" -eq 0 ]; then
        log_result "Security Audit" "PASS" "No vulnerabilities found"
        echo "✅ Security Audit: No vulnerabilities found"
    else
        log_result "Security Audit" "FAIL" "$VULNS vulnerabilities found"
        echo "❌ Security Audit: $VULNS vulnerabilities found"
        echo "   See audit-output.log for details"
    fi
else
    log_result "Security Audit" "FAIL" "Audit command failed"
    echo "❌ Security Audit: Command failed"
fi

echo "🛡️  Checking supply chain security..."
if cargo deny check > deny-output.log 2>&1; then
    log_result "Supply Chain Security" "PASS" "Supply chain checks passed"
    echo "✅ Supply Chain Security: Checks passed"
else
    log_result "Supply Chain Security" "FAIL" "Supply chain issues found"
    echo "⚠️  Supply Chain Security: Issues found (see deny-output.log)"
fi
echo ""

echo "📏 Phase 6: Code Quality"
echo "========================"
echo "🎨 Checking code formatting..."
if cargo fmt -- --check > fmt-output.log 2>&1; then
    log_result "Code Formatting" "PASS" "Code is properly formatted"
    echo "✅ Code Formatting: Properly formatted"
else
    log_result "Code Formatting" "FAIL" "Code needs formatting"
    echo "❌ Code Formatting: Needs formatting (run: cargo fmt)"
fi

echo "🔍 Running clippy lints..."
if cargo clippy --features huggingface -- -D warnings > clippy-output.log 2>&1; then
    log_result "Clippy Lints" "PASS" "No lint warnings"
    echo "✅ Clippy Lints: No warnings"
else
    WARNINGS=$(grep -c "warning:" clippy-output.log || echo "0")
    log_result "Clippy Lints" "FAIL" "$WARNINGS warnings found"
    echo "❌ Clippy Lints: $WARNINGS warnings found"
fi
echo ""

echo "🎯 Phase 7: PUNCH Analysis (Optional)"
echo "====================================="
if [ -f "./scripts/punch-analyze.sh" ]; then
    echo "🔄 Running PUNCH analysis..."
    if timeout 60s ./scripts/punch-analyze.sh > punch-output.log 2>&1; then
        log_result "PUNCH Analysis" "PASS" "Analysis completed"
        echo "✅ PUNCH Analysis: Completed (see punch-output.log)"
    else
        log_result "PUNCH Analysis" "INFO" "PUNCH not available or timed out"
        echo "ℹ️  PUNCH Analysis: Not available or timed out"
    fi
else
    log_result "PUNCH Analysis" "INFO" "PUNCH script not found"
    echo "ℹ️  PUNCH Analysis: Script not found"
fi
echo ""

echo "📈 Phase 8: Performance Benchmarks (Optional)"
echo "=============================================="
if cargo bench --version >/dev/null 2>&1; then
    echo "🏃 Running performance benchmarks..."
    if timeout 120s cargo bench > bench-output.log 2>&1; then
        log_result "Performance Benchmarks" "PASS" "Benchmarks completed"
        echo "✅ Performance Benchmarks: Completed (see bench-output.log)"
    else
        log_result "Performance Benchmarks" "FAIL" "Benchmarks failed or timed out"  
        echo "⚠️  Performance Benchmarks: Failed or timed out"
    fi
else
    log_result "Performance Benchmarks" "INFO" "Criterion not available"
    echo "ℹ️  Performance Benchmarks: Criterion not available"
fi
echo ""

# Final Results Summary
echo "🎯 Final Results Summary"
echo "========================"
echo ""
echo "📊 Test Results:"
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
echo "  📊 coverage/cobertura.xml - Coverage data"
if [ -f "coverage/tarpaulin-report.html" ]; then
    echo "  🌐 coverage/tarpaulin-report.html - Coverage report"
fi
echo "  📋 test-results.log - Detailed results"
echo "  📁 *-output.log - Individual test logs"

echo ""
if [ "$OVERALL_SUCCESS" = true ]; then
    echo "🎉 Development Test Suite: ALL CRITICAL TESTS PASSED"
    echo "🚀 Ready for commit and deployment!"
    exit 0
else
    echo "⚠️  Development Test Suite: SOME TESTS FAILED"
    echo "🔧 Please address the failed tests before committing"
    echo ""
    echo "🔍 Quick fixes:"
    echo "  - Format code: cargo fmt"
    echo "  - Fix lints: cargo clippy --fix"
    echo "  - Review test failures in *-output.log files"
    exit 1
fi