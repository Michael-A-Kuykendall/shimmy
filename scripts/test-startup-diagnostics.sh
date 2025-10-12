#!/bin/bash
# Startup Diagnostics Test Script
# Tests all scenarios for the new startup diagnostics feature

set -e

SHIMMY="./target/debug/shimmy.exe"
TEST_RESULTS="test-startup-diagnostics-results.log"

echo "🧪 Startup Diagnostics Test Suite" | tee "$TEST_RESULTS"
echo "=================================" | tee -a "$TEST_RESULTS"
echo "" | tee -a "$TEST_RESULTS"

# Ensure shimmy is built
if [ ! -f "$SHIMMY" ]; then
    echo "❌ shimmy binary not found. Building..." | tee -a "$TEST_RESULTS"
    cargo build --features llama
fi

# Test 1: No models (should show 0, then error)
echo "Test 1: No models scenario" | tee -a "$TEST_RESULTS"
echo "---" | tee -a "$TEST_RESULTS"
unset SHIMMY_BASE_GGUF
unset SHIMMY_LORA_GGUF
timeout 2 "$SHIMMY" serve --bind 127.0.0.1:19001 2>&1 | head -20 | tee -a "$TEST_RESULTS" || true
echo "" | tee -a "$TEST_RESULTS"

# Test 2: With base model set
echo "Test 2: With SHIMMY_BASE_GGUF environment variable" | tee -a "$TEST_RESULTS"
echo "---" | tee -a "$TEST_RESULTS"
export SHIMMY_BASE_GGUF="./test.gguf"
timeout 2 "$SHIMMY" serve --bind 127.0.0.1:19002 2>&1 | head -20 | tee -a "$TEST_RESULTS" || true
echo "" | tee -a "$TEST_RESULTS"

# Test 3: CPU backend explicit
echo "Test 3: Explicit CPU backend" | tee -a "$TEST_RESULTS"
echo "---" | tee -a "$TEST_RESULTS"
timeout 2 "$SHIMMY" serve --bind 127.0.0.1:19003 --gpu-backend cpu 2>&1 | head -20 | tee -a "$TEST_RESULTS" || true
echo "" | tee -a "$TEST_RESULTS"

# Test 4: Auto backend (default)
echo "Test 4: Auto backend (default)" | tee -a "$TEST_RESULTS"
echo "---" | tee -a "$TEST_RESULTS"
timeout 2 "$SHIMMY" serve --bind 127.0.0.1:19004 --gpu-backend auto 2>&1 | head -20 | tee -a "$TEST_RESULTS" || true
echo "" | tee -a "$TEST_RESULTS"

# Test 5: Invalid bind address (diagnostics should still appear)
echo "Test 5: Invalid bind address" | tee -a "$TEST_RESULTS"
echo "---" | tee -a "$TEST_RESULTS"
timeout 2 "$SHIMMY" serve --bind "invalid:address" 2>&1 | head -20 | tee -a "$TEST_RESULTS" || true
echo "" | tee -a "$TEST_RESULTS"

# Summary
echo "=================================" | tee -a "$TEST_RESULTS"
echo "✅ Test suite complete!" | tee -a "$TEST_RESULTS"
echo "Results saved to: $TEST_RESULTS" | tee -a "$TEST_RESULTS"
echo "" | tee -a "$TEST_RESULTS"

# Verification checklist
echo "Manual Verification Checklist:" | tee -a "$TEST_RESULTS"
echo "- [ ] All tests show 🎯 Shimmy v1.6.0" | tee -a "$TEST_RESULTS"
echo "- [ ] Backend info displays correctly" | tee -a "$TEST_RESULTS"
echo "- [ ] Model counts display (0 initially, then actual)" | tee -a "$TEST_RESULTS"
echo "- [ ] Ready message shows with endpoints" | tee -a "$TEST_RESULTS"
echo "- [ ] Invalid inputs still show diagnostics before erroring" | tee -a "$TEST_RESULTS"
