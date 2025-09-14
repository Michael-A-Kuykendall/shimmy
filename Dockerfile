FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build the application
RUN cargo build --release --features huggingface

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/shimmy /usr/local/bin/shimmy

# Create models directory
RUN mkdir -p /app/models

# Expose port
EXPOSE 11434

# Set default environment
ENV SHIMMY_PORT=11434
ENV SHIMMY_HOST=0.0.0.0
ENV SHIMMY_BASE_GGUF=/app/models

# Run shimmy
CMD ["shimmy", "serve"]