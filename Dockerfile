# Multi-stage build for shimmy with GPU support
FROM nvidia/cuda:12.0-devel-ubuntu22.04 as builder

# Install Rust and build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    cmake \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy source code
WORKDIR /app
COPY . .

# Build shimmy with GPU support
RUN cargo build --release --features llama

# Runtime image with CUDA runtime
FROM nvidia/cuda:12.0-runtime-ubuntu22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/shimmy /usr/local/bin/shimmy

# Create non-root user
RUN useradd -m -u 1000 shimmy
USER shimmy

# Set working directory
WORKDIR /home/shimmy

# Expose default port
EXPOSE 3000

# Default command
CMD ["shimmy", "serve", "--bind", "0.0.0.0:3000"]