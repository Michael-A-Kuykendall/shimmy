#!/bin/bash
# Coverage Analysis Script for Shimmy
# Generates comprehensive coverage reports with all feature combinations

set -e

echo "📊 Starting Shimmy Coverage Analysis..."
echo "========================================"

# Create coverage directory
mkdir -p coverage

# Clean previous runs (skip if files are locked)
echo "🧹 Cleaning previous coverage data..."
cargo clean || echo "⚠️  Some files couldn't be cleaned (may be in use)"

echo ""
echo "🧪 Running coverage analysis with all feature combinations..."
echo ""

# Generate coverage with all features (most comprehensive)
echo "📋 Coverage with ALL features (most comprehensive)..."
cargo tarpaulin \
    --all-features \
    --out html \
    --output-dir coverage \
    --timeout 300 \
    --verbose

# Generate coverage with individual feature sets for analysis
echo ""
echo "📋 Coverage with individual feature sets..."

echo "  🤖 HuggingFace features only..."
cargo tarpaulin \
    --features huggingface \
    --out xml \
    --output-dir coverage \
    --timeout 300 \
    --target-dir target-huggingface > coverage/huggingface-coverage.log 2>&1

echo "  🦙 Llama features only..."  
cargo tarpaulin \
    --features llama \
    --out xml \
    --output-dir coverage \
    --timeout 300 \
    --target-dir target-llama > coverage/llama-coverage.log 2>&1

echo ""
echo "📊 Coverage Analysis Complete!"
echo "========================================"

# Display results
if [ -f "coverage/tarpaulin-report.html" ]; then
    echo "✅ HTML Coverage Report: coverage/tarpaulin-report.html"
else
    echo "⚠️  HTML report not generated"
fi

if [ -f "coverage/cobertura.xml" ]; then
    # Extract coverage percentage from XML
    COVERAGE_PERCENT=$(grep -o 'line-rate="[^"]*"' coverage/cobertura.xml | head -1 | grep -o '[0-9.]*')
    COVERAGE_PERCENT_FORMATTED=$(echo "$COVERAGE_PERCENT * 100" | bc -l | xargs printf "%.1f")
    echo "📈 Overall Coverage: ${COVERAGE_PERCENT_FORMATTED}%"
    
    # Check if meets our 95% standard
    MEETS_STANDARD=$(echo "$COVERAGE_PERCENT >= 0.95" | bc -l)
    if [ "$MEETS_STANDARD" -eq 1 ]; then
        echo "✅ Coverage meets 95%+ professional standard!"
    else
        echo "⚠️  Coverage below 95% professional standard"
    fi
else
    echo "⚠️  XML report not generated for percentage calculation"
fi

echo ""
echo "🎯 Next Steps:"
echo "  1. Open coverage/tarpaulin-report.html in browser"
echo "  2. Review uncovered lines and add tests"
echo "  3. Run ./scripts/verify-ppt-coverage.sh for contract validation"
echo ""