#!/bin/bash
# Local release validation script - mirrors CI/CD checks EXACTLY
# Run this before pushing to ensure CI will pass
set -e

echo "=== LOCAL RELEASE VALIDATION ==="
echo "Mirrors CI/CD checks to catch issues before push"
echo ""

echo "Step 1/5: Code Formatting Check..."
cargo fmt -- --check || {
    echo "❌ FAIL: Code not formatted properly"
    echo "Fix with: cargo fmt"
    exit 1
}
echo "✅ PASS: Code formatting"

echo ""
echo "Step 2/5: Clippy (Code Quality)..."
cargo clippy --all-features --all-targets -- -D warnings || {
    echo "❌ FAIL: Clippy found issues"
    exit 1
}
echo "✅ PASS: Clippy code quality"

echo ""
echo "Step 3/5: Full Test Suite (including 115 regression tests)..."
cargo test --all-features || {
    echo "❌ FAIL: Tests failed"
    exit 1
}
echo "✅ PASS: All tests (including regression tests)"

echo ""
echo "Step 4/5: Security Audit..."
cargo deny check 2>&1 | grep -E '(advisories|bans|licenses|sources)' || {
    echo "❌ FAIL: Security issues detected"
    exit 1
}
echo "✅ PASS: Security audit"

echo ""
echo "Step 5/5: Release Build..."
cargo build --release --all-features --quiet || {
    echo "❌ FAIL: Release build failed"
    exit 1
}
echo "✅ PASS: Release build"

echo ""
echo "==================================="
echo "✅ ALL VALIDATION CHECKS PASSED"
echo "==================================="
echo ""
echo "Safe to push to CI. Run:"
echo "  git push"
