# Multi-stage build for minimal final image
FROM rust:1.70 as builder

WORKDIR /app

# Copy source code
COPY . .

# Build with release optimizations
RUN cargo build --release --features llama

# Final stage - use distroless for security and minimal size
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy the binary from builder stage
COPY --from=builder /app/target/release/shimmy /usr/local/bin/shimmy

# Use non-root user for security
USER nonroot

# Expose default port
EXPOSE 11435

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/shimmy", "list"] || exit 1

# Default command
ENTRYPOINT ["/usr/local/bin/shimmy"]
CMD ["serve", "--bind", "0.0.0.0:11435"]

# Metadata
LABEL org.opencontainers.image.title="Shimmy"
LABEL org.opencontainers.image.description="The 5MB alternative to Ollama - local AI inference server"
LABEL org.opencontainers.image.vendor="Michael A. Kuykendall"
LABEL org.opencontainers.image.source="https://github.com/Michael-A-Kuykendall/shimmy"
LABEL org.opencontainers.image.url="https://github.com/Michael-A-Kuykendall/shimmy"
LABEL org.opencontainers.image.documentation="https://github.com/Michael-A-Kuykendall/shimmy/tree/main/docs"
LABEL org.opencontainers.image.licenses="MIT"
