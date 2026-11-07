# Context is a verified licensed v2.6.6 binary bundle, prepared by container_distribution.py.
FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl libvulkan1 mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 1001 shimmy && useradd -m -u 1001 -g shimmy shimmy \
    && mkdir -p /app/models /app/cache && chown -R 1001:1001 /app
COPY payload/shimmy /usr/local/bin/shimmy
COPY payload/LICENSES /usr/local/share/shimmy/LICENSES
COPY payload/THIRD-PARTY-NOTICES.md payload/DISTRIBUTION-LICENSE.txt payload/native-manifest.json /usr/local/share/shimmy/
# Debian copyright/license notices remain in /usr/share/doc; sources: https://sources.debian.org/.
USER 1001:1001
WORKDIR /app
EXPOSE 11435
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 CMD curl -fsS http://localhost:11435/health || exit 1
ENV SHIMMY_MODEL_PATHS=/app/models
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/shimmy"]
# Non-loopback binding requires SHIMMY_API_KEY (16+ printable ASCII characters, no spaces).
CMD ["serve", "--bind", "0.0.0.0:11435"]
