#!/bin/bash
# PPT Contract Coverage Verification Script
# Ensures 100% critical contract testing for professional development standards

set -e

echo "🧪 PPT Contract Coverage Verification"
echo "====================================="

# Run PPT contract tests and capture detailed output
echo "📋 Running PPT contract tests..."
echo ""

# Capture test output for analysis
CONTRACT_OUTPUT=$(cargo test ppt_contracts --all-features -- --nocapture 2>&1 || true)

echo "🔍 Analyzing contract test results..."

# Count successful contract tests
CONTRACT_TESTS=$(echo "$CONTRACT_OUTPUT" | grep -c "Contract test.*passed" || echo "0")
PROPERTY_TESTS=$(echo "$CONTRACT_OUTPUT" | grep -c "Property test.*passed" || echo "0")
EXPLORATION_TESTS=$(echo "$CONTRACT_OUTPUT" | grep -c "Exploration test.*passed" || echo "0")

# Count verified invariants  
INVARIANT_CHECKS=$(echo "$CONTRACT_OUTPUT" | grep -c "✅.*verified" || echo "0")

# Count critical workflows tested
WORKFLOW_TESTS=$(echo "$CONTRACT_OUTPUT" | grep -c "full_workflow_integrity" || echo "0")

echo ""
echo "📊 PPT Coverage Summary:"
echo "========================"
echo "  📋 Contract Tests Passed: $CONTRACT_TESTS"
echo "  🧪 Property Tests Passed: $PROPERTY_TESTS" 
echo "  🔍 Exploration Tests: $EXPLORATION_TESTS"
echo "  ✅ Invariant Verifications: $INVARIANT_CHECKS"
echo "  🚀 Full Workflow Tests: $WORKFLOW_TESTS"

echo ""
echo "📈 Critical Contract Requirements:"
echo "=================================="

# Define minimum requirements for professional standards
MIN_CONTRACT_TESTS=5
MIN_WORKFLOW_TESTS=1
MIN_INVARIANT_CHECKS=25

REQUIREMENTS_MET=true

# Check contract test coverage
if [ "$CONTRACT_TESTS" -ge $MIN_CONTRACT_TESTS ]; then
    echo "  ✅ Contract Tests: $CONTRACT_TESTS/$MIN_CONTRACT_TESTS (PASS)"
else
    echo "  ❌ Contract Tests: $CONTRACT_TESTS/$MIN_CONTRACT_TESTS (FAIL)"
    REQUIREMENTS_MET=false
fi

# Check workflow test coverage  
if [ "$WORKFLOW_TESTS" -ge $MIN_WORKFLOW_TESTS ]; then
    echo "  ✅ Workflow Tests: $WORKFLOW_TESTS/$MIN_WORKFLOW_TESTS (PASS)"
else
    echo "  ❌ Workflow Tests: $WORKFLOW_TESTS/$MIN_WORKFLOW_TESTS (FAIL)"
    REQUIREMENTS_MET=false
fi

# Check invariant verification coverage
if [ "$INVARIANT_CHECKS" -ge $MIN_INVARIANT_CHECKS ]; then
    echo "  ✅ Invariant Checks: $INVARIANT_CHECKS/$MIN_INVARIANT_CHECKS (PASS)"
else
    echo "  ❌ Invariant Checks: $INVARIANT_CHECKS/$MIN_INVARIANT_CHECKS (FAIL)"
    REQUIREMENTS_MET=false
fi

echo ""

# Check for test failures
if echo "$CONTRACT_OUTPUT" | grep -q "test result: FAILED"; then
    echo "❌ Some PPT tests are failing!"
    echo ""
    echo "Failed test details:"
    echo "$CONTRACT_OUTPUT" | grep -A 5 -B 5 "FAILED\|panicked"
    REQUIREMENTS_MET=false
fi

# Final verdict
echo "🎯 PPT Coverage Verdict:"
echo "========================"

if [ "$REQUIREMENTS_MET" = true ]; then
    echo "✅ PPT coverage meets professional standards!"
    echo "🏆 All critical contracts are properly tested"
    echo "🚀 Ready for production deployment"
    
    # Additional quality indicators
    echo ""
    echo "🌟 Quality Indicators:"
    if [ "$CONTRACT_TESTS" -ge 10 ]; then
        echo "  🏅 Excellent contract coverage ($CONTRACT_TESTS tests)"
    fi
    if [ "$INVARIANT_CHECKS" -ge 50 ]; then
        echo "  🏅 Exceptional invariant coverage ($INVARIANT_CHECKS checks)"
    fi
    
    exit 0
else
    echo "❌ PPT coverage does NOT meet professional standards"
    echo "🔧 Action required:"
    echo "  1. Add more contract tests to cover critical workflows"
    echo "  2. Ensure all invariants are properly tested"  
    echo "  3. Fix any failing PPT tests"
    echo "  4. Re-run this script to verify improvements"
    echo ""
    echo "📚 See docs/ppt-invariant-testing.md for guidance"
    
    exit 1
fi