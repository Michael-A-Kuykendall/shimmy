# Multi-stage Dockerfile for Shimmy - optimized for cloud deployment
FROM rust:slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Cargo parses bench targets from manifest and src compiles templates via include_str!.
COPY benches ./benches
COPY templates ./templates

# Copy source code
COPY src ./src

# Build release binary with full GPU support (airframe engine)
RUN cargo build --release --features airframe

# Runtime image — trixie for glibc 2.39+ (matches rust:slim builder)
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false shimmy

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/shimmy /usr/local/bin/shimmy

# Create directory for models (if needed)
RUN mkdir -p /app/models && chown shimmy:shimmy /app/models

USER shimmy

# Expose port
EXPOSE 11435

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:11435/health || exit 1

# Default command
CMD ["shimmy", "serve", "--bind", "0.0.0.0:11435"]
