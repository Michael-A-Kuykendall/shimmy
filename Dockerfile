# Build from the repository root: docker build -f templates/docker/Dockerfile .
FROM rust:slim-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --locked --bin shimmy

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl libvulkan1 mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 1001 shimmy && useradd -m -u 1001 -g shimmy shimmy \
    && mkdir -p /app/models /app/cache && chown -R shimmy:shimmy /app
COPY --from=builder /build/target/release/shimmy /usr/local/bin/shimmy
USER shimmy
WORKDIR /app
EXPOSE 11435
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -fsS http://localhost:11435/health || exit 1
ENV SHIMMY_MODEL_PATHS=/app/models
ENV RUST_LOG=info
# A non-loopback bind refuses startup without SHIMMY_API_KEY.
CMD ["shimmy", "serve", "--bind", "0.0.0.0:11435"]
