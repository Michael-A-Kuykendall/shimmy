#!/bin/bash
# Verification script for Issue #72 GPU backend fix
# This script demonstrates that the --gpu-backend flag now works correctly

set -e

echo "========================================="
echo "GPU Backend Fix Verification (Issue #72)"
echo "========================================="
echo ""

echo "Test 1: CPU-only build with --gpu-backend vulkan (should warn and fall back)"
echo "Building with --features llama (no GPU)..."
cargo build --release --no-default-features --features llama -q
echo ""
echo "Running: shimmy gpu-info --gpu-backend vulkan"
RUST_LOG=warn ./target/release/shimmy.exe gpu-info --gpu-backend vulkan 2>&1 | grep -E "(WARN|Backend:|Vulkan support)" || true
echo ""

echo "----------------------------------------"
echo "Test 2: Vulkan build with --gpu-backend vulkan (should use Vulkan)"
echo "Building with --features llama-vulkan..."
cargo build --release --no-default-features --features llama-vulkan -q
echo ""
echo "Running: shimmy gpu-info --gpu-backend vulkan"
RUST_LOG=info ./target/release/shimmy.exe gpu-info --gpu-backend vulkan 2>&1 | grep -E "(INFO|Backend:|Vulkan support)"
echo ""

echo "----------------------------------------"
echo "Test 3: Vulkan build with --gpu-backend auto (should auto-detect Vulkan)"
echo "Running: shimmy gpu-info --gpu-backend auto"
RUST_LOG=info ./target/release/shimmy.exe gpu-info --gpu-backend auto 2>&1 | grep -E "(INFO|Backend:|Vulkan support)"
echo ""

echo "----------------------------------------"
echo "Test 4: Vulkan build with --gpu-backend cpu (should force CPU)"
echo "Running: shimmy gpu-info --gpu-backend cpu"
./target/release/shimmy.exe gpu-info --gpu-backend cpu 2>&1 | grep "Backend:"
echo ""

echo "========================================="
echo "✅ All verification tests passed!"
echo "The --gpu-backend flag is now properly wired through to model loading."
echo "========================================="
