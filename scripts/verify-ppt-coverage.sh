#!/bin/bash
# PPT Coverage Verification Script - Release Mode for Shimmy
# Simplified for quick releases when CI/CD will handle full testing

echo "🧪 Release Readiness Check"
echo "=========================="

# For releases, just ensure code compiles with the features used by PPT tests.
# Avoid --all-features: it pulls in GPU backends (cuda, vulkan, opencl) and
# vision, which are not available on all CI runners and can fail due to stale
# CMake caches in the target/ directory.
echo "📋 Checking compilation..."
if cargo check --features llama >/dev/null 2>&1; then
    echo "✅ Code compiles successfully"
    echo "🚀 Ready for release (CI/CD will run full tests)"
    exit 0
else
    echo "❌ Compilation failed!"
    echo "🔧 Fix compilation errors before release"
    cargo check --features llama
    exit 1
fi
