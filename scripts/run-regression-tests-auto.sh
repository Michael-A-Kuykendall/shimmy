#!/bin/bash
# Consolidated Regression Test Runner (v2.2+)
# Runs the three consolidated test files:
#   tests/core.rs, tests/handlers.rs, tests/compile_checks.rs
#
# See docs/REGRESSION_TESTING.md for the issue-to-test mapping.

set -e  # Exit on first failure

echo "🧪 Shimmy Regression Test Suite (Consolidated v2.2+)"
echo "==================================================="
echo ""

# Track results
PASSED=0
FAILED=0
FAILED_TESTS=()

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

RUN_CORE=true
RUN_HANDLERS=true
RUN_COMPILE=true
FEATURES="--features airframe,huggingface"

for arg in "$@"; do
    case $arg in
        --no-core) RUN_CORE=false ;;
        --no-handlers) RUN_HANDLERS=false ;;
        --no-compile) RUN_COMPILE=false ;;
        --features) FEATURES="--features $2"; shift ;;
    esac
done

run_suite() {
    local name="$1"
    local target="$2"
    local features="$3"
    local logfile="${target}-output.log"

    echo "🔬 $name"
    if cargo test --test "$target" $features &> "$logfile"; then
        echo -e "   ${GREEN}✅ PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "   ${RED}❌ FAIL${NC} — see $logfile"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("$target")
    fi
    echo ""
}

if $RUN_CORE; then
    run_suite "tests/core.rs — CLI, registry, templates, serde, SSE, discovery" "core" "$FEATURES"
fi

if $RUN_HANDLERS; then
    run_suite "tests/handlers.rs — HTTP endpoints (health, models, chat, tags, concurrency)" "handlers" "$FEATURES"
fi

if $RUN_COMPILE; then
    run_suite "tests/compile_checks.rs — template file inclusion" "compile_checks" ""
fi

# Summary
echo "========================================"
echo "📊 Regression Test Results Summary"
echo "========================================"
echo -e "${GREEN}✅ Passed: $PASSED${NC}"
echo -e "${RED}❌ Failed: $FAILED${NC}"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed Suites:${NC}"
    for failed in "${FAILED_TESTS[@]}"; do
        echo "  ❌ $failed"
    done
    echo ""
    echo "🔧 Fix failing regression tests before proceeding"
    exit 1
else
    echo -e "${GREEN}🎉 ALL REGRESSION TESTS PASSED${NC}"
    echo "✅ No regressions detected — safe to proceed"
    exit 0
fi
