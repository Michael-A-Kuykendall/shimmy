#!/bin/bash
# Install Git Hooks for Professional Development Workflow
# Sets up pre-commit hooks for fast quality feedback

echo "🪝 Installing Git Hooks for Professional Development"
echo "=================================================="

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Install pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook for Shimmy - Fast quality checks before commit
# Prevents commits that would fail CI/CD pipeline

echo "🪝 Pre-commit Quality Checks"
echo "============================"

CHECKS_FAILED=false

# 1. Code Formatting Check (Fast)
echo "🎨 Checking code formatting..."
if ! cargo fmt -- --check >/dev/null 2>&1; then
    echo "❌ Code formatting check failed"
    echo "   Fix with: cargo fmt"
    CHECKS_FAILED=true
else
    echo "✅ Code formatting: OK"
fi

# 2. Basic Compilation Check (Fast)
echo "🔨 Checking compilation..."
if ! cargo check --features huggingface >/dev/null 2>&1; then
    echo "❌ Compilation check failed"
    echo "   Fix compilation errors before committing"
    CHECKS_FAILED=true
else
    echo "✅ Compilation: OK"  
fi

# 3. Critical PPT Contract Tests (Medium)
echo "🧪 Running critical contract tests..."
if ! timeout 60s cargo test ppt_contracts --features huggingface -- --quiet >/dev/null 2>&1; then
    echo "❌ PPT contract tests failed"
    echo "   Run: cargo test ppt_contracts --features huggingface"
    CHECKS_FAILED=true
else
    echo "✅ PPT contracts: OK"
fi

# 4. Basic Clippy Lints (Medium)
echo "🔍 Running clippy lints..."
if ! cargo clippy --features huggingface -- -D warnings >/dev/null 2>&1; then
    echo "⚠️  Clippy warnings found (allowing commit)"
    echo "   Review with: cargo clippy --features huggingface"
    # Don't fail on clippy warnings, just warn
else
    echo "✅ Clippy lints: OK"
fi

echo ""
if [ "$CHECKS_FAILED" = true ]; then
    echo "❌ Pre-commit checks FAILED"
    echo "🔧 Please fix the issues above and try again"
    echo ""
    echo "💡 Tip: Run './scripts/dev-test.sh' for comprehensive testing"
    exit 1
else
    echo "✅ Pre-commit checks PASSED"
    echo "🚀 Proceeding with commit..."
fi
EOF

# Make pre-commit hook executable
chmod +x .git/hooks/pre-commit

# Install pre-push hook (more comprehensive)
cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash
# Pre-push hook for Shimmy - Comprehensive checks before push
# Ensures pushed code meets professional standards

echo "🚀 Pre-push Quality Gate"
echo "========================"

CHECKS_FAILED=false

echo "🧪 Running PPT verification..."
if ! timeout 120s ./scripts/verify-ppt-coverage.sh >/dev/null 2>&1; then
    echo "❌ PPT coverage verification failed"
    echo "   Run: ./scripts/verify-ppt-coverage.sh"
    CHECKS_FAILED=true
else
    echo "✅ PPT coverage: Verified"
fi

echo "🔒 Running security audit..."
if ! cargo audit >/dev/null 2>&1; then
    echo "❌ Security vulnerabilities found"
    echo "   Run: cargo audit"
    CHECKS_FAILED=true
else
    echo "✅ Security audit: Clean"
fi

echo "📊 Quick coverage check..."
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    if timeout 180s cargo tarpaulin --features huggingface --out xml --output-dir coverage >/dev/null 2>&1; then
        if [ -f "coverage/cobertura.xml" ]; then
            COVERAGE_PERCENT=$(grep -o 'line-rate="[^"]*"' coverage/cobertura.xml | head -1 | grep -o '[0-9.]*' || echo "0")
            MEETS_STANDARD=$(echo "$COVERAGE_PERCENT >= 0.90" | bc -l 2>/dev/null || echo "0")
            if [ "$MEETS_STANDARD" -eq 1 ]; then
                echo "✅ Coverage: Adequate for push"
            else
                echo "⚠️  Coverage below 90% (allowing push)"
            fi
        fi
    fi
else
    echo "ℹ️  Coverage tool not available"
fi

echo ""
if [ "$CHECKS_FAILED" = true ]; then
    echo "❌ Pre-push checks FAILED"
    echo "🔧 Please fix the issues above before pushing"
    echo ""
    echo "💡 Tip: Run './scripts/dev-test.sh' for full analysis"
    exit 1
else
    echo "✅ Pre-push checks PASSED"
    echo "🌐 Proceeding with push..."
fi
EOF

# Make pre-push hook executable  
chmod +x .git/hooks/pre-push

echo ""
echo "✅ Git hooks installed successfully!"
echo ""
echo "📋 Installed hooks:"
echo "  🪝 pre-commit  - Fast quality checks (formatting, compilation, critical tests)"
echo "  🚀 pre-push    - Comprehensive checks (PPT coverage, security, coverage)"
echo ""
echo "🔧 Hook behavior:"
echo "  - pre-commit: Blocks commits that would fail basic checks"
echo "  - pre-push:   Blocks pushes that would fail comprehensive tests"
echo ""
echo "💡 To skip hooks (emergency only):"
echo "  - Skip pre-commit: git commit --no-verify"
echo "  - Skip pre-push:   git push --no-verify"
echo ""
echo "🎯 Professional development workflow now active!"