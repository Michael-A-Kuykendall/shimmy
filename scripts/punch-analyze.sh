#!/bin/bash
# PUNCH Analysis Integration Script
# Provides AI-powered Rust code analysis when PUNCH systems tool is available

echo "🎯 PUNCH Rust Analysis for Shimmy"
echo "=================================="

PUNCH_BINARY=".punch/punch-systems"
PUNCH_SOURCE="../punch-discovery/target/release/punch-systems"

# Check if PUNCH is available locally
if [ -f "$PUNCH_BINARY" ]; then
    echo "✅ Using local PUNCH binary: $PUNCH_BINARY"
    PUNCH_CMD="$PUNCH_BINARY"
elif [ -f "$PUNCH_SOURCE" ]; then
    echo "📦 Found PUNCH in source directory, copying locally..."
    cp "$PUNCH_SOURCE" "$PUNCH_BINARY"
    chmod +x "$PUNCH_BINARY"
    PUNCH_CMD="$PUNCH_BINARY"
elif command -v punch-systems >/dev/null 2>&1; then
    echo "🌐 Using system-installed PUNCH"
    PUNCH_CMD="punch-systems"
else
    echo "⚠️  PUNCH systems tool not found"
    echo "   Expected locations:"
    echo "   - .punch/punch-systems (local copy)"
    echo "   - ../punch-discovery/target/release/punch-systems (source build)"
    echo "   - punch-systems (system PATH)"
    echo ""
    echo "🔧 To install PUNCH:"
    echo "   1. Build punch-discovery project: cd ../punch-discovery && cargo build --release"
    echo "   2. Copy binary: cp ../punch-discovery/target/release/punch-systems .punch/"
    echo "   3. Re-run this script"
    echo ""
    echo "📊 For now, running basic Rust analysis with cargo..."
    
    # Fallback to basic cargo analysis
    echo ""
    echo "🦀 Basic Rust Analysis (Fallback):"
    echo "  📋 Checking compilation..."
    cargo check --all-features --quiet && echo "    ✅ Compilation: PASS" || echo "    ❌ Compilation: FAIL"
    
    echo "  🧪 Running tests..."
    cargo test --all-features --quiet >/dev/null 2>&1 && echo "    ✅ Tests: PASS" || echo "    ❌ Tests: FAIL"
    
    echo "  📏 Code formatting..."
    cargo fmt -- --check >/dev/null 2>&1 && echo "    ✅ Formatting: PASS" || echo "    ❌ Formatting: FAIL"
    
    echo "  🔍 Clippy lints..."
    cargo clippy --all-features -- -D warnings >/dev/null 2>&1 && echo "    ✅ Lints: PASS" || echo "    ❌ Lints: FAIL"
    
    echo ""
    echo "🎯 For advanced PUNCH analysis, install the PUNCH systems tool"
    exit 0
fi

echo "🚀 Running PUNCH Rust analysis on ./src/..."
echo ""

# Run comprehensive PUNCH analysis
echo "📊 Code Quality Analysis:"
$PUNCH_CMD rust analyze ./src/ --verbose || echo "⚠️  Analysis had issues"

echo ""
echo "🔒 Security Analysis:"
$PUNCH_CMD rust security ./src/ --report=console || echo "⚠️  Security analysis had issues"

echo ""
echo "⚡ Performance Analysis:"
$PUNCH_CMD rust performance ./src/ --suggest || echo "⚠️  Performance analysis had issues"

echo ""
echo "📋 Contract Analysis:"
$PUNCH_CMD rust contracts ./src/ --validate || echo "⚠️  Contract analysis had issues"

echo ""
echo "🎯 Overall Quality Score:"
$PUNCH_CMD rust score ./src/ || echo "⚠️  Scoring had issues"

echo ""
echo "✅ PUNCH analysis complete!"
echo "📚 See PUNCH documentation for detailed explanations"