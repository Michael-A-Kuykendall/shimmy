# Multi-stage Dockerfile for Shimmy - optimized for cloud deployment
FROM rust:slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cargo still resolves the optional airframe path dependency during metadata checks,
# so create a minimal local placeholder to keep the Docker build self-contained.
RUN mkdir -p /airframe/src && \
    printf '[package]\nname = "airframe"\nversion = "0.0.1"\nedition = "2021"\n\n[lib]\npath = "src/lib.rs"\n' > /airframe/Cargo.toml && \
    printf 'pub fn _placeholder() {}\n' > /airframe/src/lib.rs

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Cargo parses bench targets from manifest and src compiles templates via include_str!.
COPY benches ./benches
COPY templates ./templates

# Copy source code
COPY src ./src

# Build release binary with minimal features for cloud deployment
RUN cargo build --release --no-default-features --features huggingface

# Runtime image
FROM debian:bookworm-slim

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
EXPOSE 11434

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:11434/health || exit 1

# Default command
CMD ["shimmy", "serve", "--bind", "0.0.0.0:11434"]
