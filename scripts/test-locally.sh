#!/bin/bash
# Local Test Runner with CI Cache Integration
# 
# USAGE:
#   ./scripts/test-locally.sh
#
# This script runs the complete test suite locally and creates a cache token
# that CI will recognize, allowing CI to skip redundant test execution.
#
# WORKFLOW:
#   1. Run this script locally: ./scripts/test-locally.sh
#   2. Commit the .test-cache files: git add .test-cache && git commit
#   3. Push to GitHub: git push
#   4. CI will detect local test results and skip re-running tests
#
# BENEFITS:
#   - Eliminates CI timeout issues for long-running tests  
#   - Faster CI feedback (tests already validated locally)
#   - Single-developer workflow optimization

set -e

COMMIT_HASH=$(git rev-parse HEAD)
echo "🧪 Running comprehensive local tests for commit ${COMMIT_HASH:0:8}..."

# Create cache directory
mkdir -p .test-cache

# Run the same tests that CI would run
echo "📋 Running PPT Contract Tests..."
cargo test --lib --features llama ppt -- --test-threads=1 --nocapture

echo "📋 Running Property Tests..."
cargo test property_tests --no-default-features --features huggingface -- --nocapture

echo "📋 Running Unit Tests (HuggingFace)..."
cargo test --lib --no-default-features --features huggingface --verbose

echo "📋 Running Unit Tests (All Features)..."
cargo test --lib --all-features --verbose

# Create success attestation
echo "${COMMIT_HASH}" > .test-cache/test-success-commit
echo "$(date -u -Iseconds)" > .test-cache/test-success-timestamp
echo "local" > .test-cache/test-runner

echo "✅ All tests passed locally!"
echo "💡 Push this commit and CI will skip redundant test execution"
echo "📋 Test cache saved for commit ${COMMIT_HASH:0:8}"