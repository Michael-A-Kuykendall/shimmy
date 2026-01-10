#!/bin/bash
# Pre-commit hooks setup script for Shimmy
# Installs and configures quality gates that prevent bad commits

set -e

echo "🔒 Setting up Shimmy pre-commit hooks..."

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit..."
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    else
        echo "❌ Error: pip not found. Please install Python and pip first."
        exit 1
    fi
fi

# Install the pre-commit hooks
echo "⚙️ Installing pre-commit hooks..."
pre-commit install

# Run pre-commit on all files to test setup
echo "🧪 Testing pre-commit hooks on all files..."
echo "⚠️  This may take a few minutes for the first run..."

# Run with verbose output so user can see what's happening
pre-commit run --all-files --verbose

echo ""
echo "✅ Pre-commit hooks installed successfully!"
echo ""
echo "📋 What this means:"
echo "  - cargo fmt --check: Code must be formatted"
echo "  - cargo clippy --all-features: No warnings allowed"
echo "  - cargo test --all-features: All tests must pass"
echo "  - No direct commits to main branch"
echo ""
echo "🚀 You're now protected from committing bad code!"
echo "💡 Run 'cargo fmt' before committing to auto-fix formatting"