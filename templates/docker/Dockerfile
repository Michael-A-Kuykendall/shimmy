# Multi-stage Shimmy Docker Image
# Optimized for production deployment with minimal size

# Build stage
FROM rust:1.89-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    gcc \
    git

# Set work directory
WORKDIR /build

# Copy source code
COPY . .

# Build Shimmy with all features
RUN cargo build --release --features "huggingface,llama,mlx"

# Runtime stage - minimal Alpine image
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    openssl

# Create shimmy user for security
RUN addgroup -g 1001 shimmy && \
    adduser -D -s /bin/sh -u 1001 -G shimmy shimmy

# Create directories
RUN mkdir -p /app/models /app/cache && \
    chown -R shimmy:shimmy /app

# Copy binary from builder
COPY --from=builder /build/target/release/shimmy /app/shimmy
RUN chmod +x /app/shimmy && chown shimmy:shimmy /app/shimmy

# Switch to shimmy user
USER shimmy

# Set work directory
WORKDIR /app

# Expose default port
EXPOSE 11435

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:11435/v1/models || exit 1

# Environment variables
ENV SHIMMY_HOST=0.0.0.0
ENV SHIMMY_PORT=11435
ENV RUST_LOG=info

# Default command
CMD ["./shimmy", "serve", "--bind", "0.0.0.0:11435"]
