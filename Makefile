# Shimmy Development Makefile
# Provides convenient commands for testing, building, and releasing

.PHONY: test test-cached build install clean release help

# Default target
help:
	@echo "Shimmy Development Commands:"
	@echo "  make test        - Run full test suite with CI cache integration"
	@echo "  make test-quick  - Run basic tests only"
	@echo "  make build       - Build shimmy binary"
	@echo "  make install     - Install shimmy locally"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make release     - Create release build"
	@echo "  make fmt         - Format code"
	@echo "  make lint        - Run clippy lints"

# Full test suite
test:
	@echo "🧪 Running Shimmy Test Suite"
	@echo "📋 Running PPT Contract Tests..."
	cargo test --lib --features llama ppt -- --test-threads=1 --nocapture
	@echo "📋 Running Property Tests..."
	cargo test property_tests --no-default-features --features huggingface -- --nocapture
	@echo "📋 Running Unit Tests (HuggingFace)..."
	cargo test --lib --no-default-features --features huggingface --verbose
	@echo "📋 Running Unit Tests (All Features)..."
	cargo test --lib --all-features --verbose
	@echo "✅ All tests passed locally!"

# Quick tests for development
test-quick:
	@echo "🚀 Running quick tests..."
	cargo test --lib --features huggingface

# Build commands
build:
	cargo build --release --all-features

install:
	cargo install --path . --all-features

clean:
	cargo clean
	rm -rf .test-cache

# Code quality
fmt:
	cargo fmt

lint:
	cargo clippy --all-features -- -D warnings

# Release build
release:
	@echo "🚀 Creating release build..."
	cargo build --release --all-features
	@echo "✅ Release binary: target/release/shimmy"