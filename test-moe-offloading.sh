#!/bin/bash
# GPT-OSS MoE CPU Offloading Test Script
# Tests shimmy with and without --cpu-moe flag to demonstrate VRAM reduction

MODEL_PATH="./models/gpt-oss-20b-Q4_K_M.gguf"
SHIMMY_BIN="./target/release/shimmy.exe"

echo "========================================="
echo "GPT-OSS MoE CPU Offloading Test"
echo "========================================="
echo ""
echo "Model: gpt-oss-20b-Q4_K_M (11.6 GB)"
echo "GPU: RTX 3060 (4GB VRAM)"
echo ""

# Test 1: Try WITHOUT MoE offloading (will likely fail/OOM)
echo "----------------------------------------"
echo "TEST 1: WITHOUT MoE offloading"
echo "Expected: VRAM overflow or very slow"
echo "----------------------------------------"
echo ""
echo "Running: shimmy probe (no --cpu-moe flag)"
echo ""

SHIMMY_BASE_GGUF="$MODEL_PATH" timeout 60s "$SHIMMY_BIN" probe gpt-oss-20b 2>&1 | tee test-no-moe.log || true

echo ""
echo ""

# Test 2: WITH MoE CPU offloading
echo "----------------------------------------"
echo "TEST 2: WITH --cpu-moe flag"
echo "Expected: Experts offloaded, fits in VRAM"
echo "----------------------------------------"
echo ""
echo "Running: shimmy serve --cpu-moe"
echo ""

SHIMMY_BASE_GGUF="$MODEL_PATH" timeout 60s "$SHIMMY_BIN" serve --bind 127.0.0.1:11435 --cpu-moe 2>&1 | tee test-with-moe.log || true

echo ""
echo ""
echo "========================================="
echo "Test Complete!"
echo "========================================="
echo ""
echo "Check logs:"
echo "  - test-no-moe.log: Baseline (should show VRAM issues)"
echo "  - test-with-moe.log: With MoE offloading (should succeed)"
echo ""
echo "Look for 'MoE:' log lines in test-with-moe.log"
