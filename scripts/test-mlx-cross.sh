#!/bin/bash
# MLX Cross-compilation Testing Script
# Tests compilation without requiring Apple hardware

set -e

echo "🧪 Testing MLX compilation with cross-rs..."

# 1. Test basic compilation
echo "📦 Testing basic MLX compilation..."
cross check --target aarch64-apple-darwin --features mlx

# 2. Test release build
echo "🚀 Testing MLX release build..." 
cross build --target aarch64-apple-darwin --features mlx --release --no-run

# 3. Test feature combinations
echo "🔧 Testing MLX feature combinations..."
cross check --target aarch64-apple-darwin --features mlx,moe
cross check --target aarch64-apple-darwin --features gpu,mlx

# 4. Test conditional compilation
echo "🎯 Testing conditional compilation..."
cross check --target aarch64-apple-darwin --features mlx --no-default-features

echo "✅ MLX cross-compilation tests passed!"
echo "🍎 Next: Test on real Apple Silicon via GitHub Actions"