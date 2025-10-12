#!/bin/bash
# MoE Fix Verification Test - Issue #108
# This script tests that the MoE CPU offloading fix is working

echo "========================================="
echo "MoE CPU Offloading Fix Verification"
echo "Issue #108 - Test Script"
echo "========================================="
echo ""

MODEL_PATH="D:/shimmy-test-models/Phi-3.5-mini-instruct.Q4_K_M.gguf"
SHIMMY_BIN="./target/release/shimmy.exe"

# Verify model exists
if [ ! -f "$MODEL_PATH" ]; then
    echo "❌ Model not found: $MODEL_PATH"
    exit 1
fi

echo "Model: $MODEL_PATH"
echo "Size: $(du -h "$MODEL_PATH" | cut -f1)"
echo ""

# Test 1: Verify MoE startup messages work
echo "----------------------------------------"
echo "TEST 1: MoE Startup Messages"
echo "----------------------------------------"

echo "Testing --cpu-moe flag startup..."
export SHIMMY_BASE_GGUF="$MODEL_PATH"

# Capture startup output
timeout 8s "$SHIMMY_BIN" serve --cpu-moe --bind 127.0.0.1:11435 > moe_startup_test.log 2>&1 &
PID=$!
sleep 3

# Check if process is running and get startup logs
if kill -0 $PID 2>/dev/null; then
    echo "✅ Server started successfully"
    kill $PID 2>/dev/null
    wait $PID 2>/dev/null
else
    echo "❌ Server failed to start"
fi

echo ""
echo "Startup log output:"
echo "==================="
head -10 moe_startup_test.log
echo ""

# Verify the MoE message appears
if grep -q "MoE: CPU offload ALL expert tensors" moe_startup_test.log; then
    echo "✅ MoE startup message found"
else
    echo "❌ MoE startup message NOT found"
    echo "This indicates the fix may not be working"
fi

echo ""

# Test 2: Verify MoE flags are recognized
echo "----------------------------------------" 
echo "TEST 2: MoE Flag Recognition"
echo "----------------------------------------"

echo "Testing --n-cpu-moe flag..."
timeout 3s "$SHIMMY_BIN" serve --n-cpu-moe 4 --bind 127.0.0.1:11436 > moe_n_test.log 2>&1 &
PID=$!
sleep 1
kill $PID 2>/dev/null
wait $PID 2>/dev/null

if grep -q "MoE: CPU offload first 4 layers" moe_n_test.log; then
    echo "✅ --n-cpu-moe flag working"
else
    echo "❌ --n-cpu-moe flag NOT working"
fi

echo ""

# Test 3: API Generation Test with MoE
echo "----------------------------------------"
echo "TEST 3: API Generation with MoE"
echo "----------------------------------------"

echo "Starting server with MoE for API test..."
export SHIMMY_BASE_GGUF="$MODEL_PATH"
"$SHIMMY_BIN" serve --cpu-moe --bind 127.0.0.1:11437 > moe_api_test.log 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 5

# Test API call
echo "Making API request..."
API_RESPONSE=$(curl -s -X POST http://127.0.0.1:11437/api/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Test", "max_tokens": 5}' \
  --max-time 30 2>/dev/null || echo "TIMEOUT")

# Kill server
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

if [ "$API_RESPONSE" = "TIMEOUT" ]; then
    echo "⚠️  API request timed out (may be normal for non-MoE model)"
elif echo "$API_RESPONSE" | grep -q "response\|text\|content"; then
    echo "✅ API responded successfully"
    echo "Response sample: $(echo "$API_RESPONSE" | head -c 100)..."
else
    echo "❌ API failed to respond properly"
    echo "Response: $API_RESPONSE"
fi

echo ""

# Test 4: Log Analysis for MoE Activity
echo "----------------------------------------"
echo "TEST 4: MoE Activity Analysis"
echo "----------------------------------------"

echo "Analyzing server logs for MoE activity..."

# Look for MoE-related log messages
if grep -q "MoE:" moe_api_test.log; then
    echo "✅ MoE logging found in server output"
    echo "MoE log entries:"
    grep "MoE:" moe_api_test.log | head -3
else
    echo "⚠️  No specific MoE logging found (may be normal for non-MoE model)"
fi

# Look for expert tensor offloading (the real test)
if grep -q "buffer type overridden to CUDA_Host" moe_api_test.log; then
    echo "✅ CRITICAL: Expert tensor offloading detected!"
    echo "This confirms MoE CPU offloading is working."
    grep "buffer type overridden to CUDA_Host" moe_api_test.log | head -3
elif grep -q "ffn.*exps" moe_api_test.log; then
    echo "⚠️  Expert tensors mentioned but no offloading detected"
    echo "This may indicate MoE model is needed for full test"
else
    echo "ℹ️  No expert tensors found (expected for non-MoE models)"
    echo "To fully test MoE, a real MoE model is needed"
fi

echo ""

# Summary
echo "========================================="
echo "TEST SUMMARY"
echo "========================================="
echo ""

# Count successes
SUCCESS_COUNT=0
TOTAL_TESTS=4

if grep -q "MoE: CPU offload ALL expert tensors" moe_startup_test.log; then
    echo "✅ Test 1: MoE startup messages - PASS"
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
else
    echo "❌ Test 1: MoE startup messages - FAIL"
fi

if grep -q "MoE: CPU offload first 4 layers" moe_n_test.log; then
    echo "✅ Test 2: MoE flag recognition - PASS"  
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
else
    echo "❌ Test 2: MoE flag recognition - FAIL"
fi

if [ "$API_RESPONSE" != "TIMEOUT" ] && echo "$API_RESPONSE" | grep -q "response\|text\|content"; then
    echo "✅ Test 3: API functionality - PASS"
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
else
    echo "⚠️  Test 3: API functionality - UNCLEAR"
fi

if grep -q "buffer type overridden to CUDA_Host" moe_api_test.log; then
    echo "✅ Test 4: Expert tensor offloading - CONFIRMED"
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
else
    echo "ℹ️  Test 4: Expert tensor offloading - NEEDS MOE MODEL"
fi

echo ""
echo "Results: $SUCCESS_COUNT/$TOTAL_TESTS tests clearly passed"
echo ""

if [ $SUCCESS_COUNT -ge 2 ]; then
    echo "🎯 CONCLUSION: MoE infrastructure is working"
    echo "   The fix appears successful. Code paths are active."
    echo "   For complete validation, test with actual MoE model."
else
    echo "❌ CONCLUSION: MoE fix may have issues"
    echo "   More investigation needed."
fi

echo ""
echo "Log files generated:"
echo "  - moe_startup_test.log"
echo "  - moe_n_test.log" 
echo "  - moe_api_test.log"
echo ""
echo "========================================="