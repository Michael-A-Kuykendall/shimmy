# Docker — Shimmy 2.6.7

The versioned image is `ghcr.io/michael-a-kuykendall/shimmy:2.6.7` for
Linux x86-64 and ARM64. The published multiarchitecture OCI index is `sha256:a85d9dc05350a971096c2b7ec17e515707262189caf4aa3f580f5e319bf31fe9`. It contains the qualified default-Airframe binaries from
[the GitHub release](https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v2.6.7),
not a fast/non-inference build. The existing `latest` tag is unchanged.

Image gates cover architecture, CLI, licensing, startup, health and API
authentication. The published index and its referenced manifest/config/layer digests were anonymously verified. They do **not** certify container GPU inference, DGX Spark,
Metal/M4 or WSL device-loss stability. Model loading is lazy: health alone does
not prove model residency or numerical correctness.

## Start a protected server

```bash
docker pull ghcr.io/michael-a-kuykendall/shimmy:2.6.7
export SHIMMY_API_KEY="$(openssl rand -hex 32)"
docker run --rm --name shimmy \
  -p 127.0.0.1:11435:11435 \
  -e SHIMMY_API_KEY \
  -v /absolute/path/to/models:/app/models:ro \
  ghcr.io/michael-a-kuykendall/shimmy:2.6.7
curl -f http://127.0.0.1:11435/health
curl -f -H "Authorization: Bearer $SHIMMY_API_KEY" http://127.0.0.1:11435/v1/models
```

The image entrypoint is `shimmy`; omit a redundant `shimmy` when overriding
arguments. For example, `docker run --rm ghcr.io/michael-a-kuykendall/shimmy:2.6.7 --version`.
The default non-loopback listener requires `SHIMMY_API_KEY` (at least 16 printable
ASCII characters, without spaces), even with a loopback host port mapping.
Protect remote access with TLS. Never bake real credentials into an image.

For an explicit fixture, append
`--ctx 512 --kv-quant f32 serve --bind 0.0.0.0:11435 --model-path /app/models/model.gguf`.
Configure vendor-specific device access separately; this is not a promise that
GPU inference works without a compatible device/runtime.

## Compose

```yaml
services:
  shimmy:
    image: ghcr.io/michael-a-kuykendall/shimmy:2.6.7
    ports:
      - "127.0.0.1:11435:11435"
    environment:
      SHIMMY_API_KEY: "${SHIMMY_API_KEY:?set a private key first}"
    volumes:
      - "/absolute/path/to/models:/app/models:ro"
```

Run `docker compose config` before starting your saved file. Old cloud/deployment
templates remain historical scaffolds; see [deployment scope](deploy/README.md).

## Licensing and reproducibility

Component licenses, the libfse 0.1.4 additional Apache-2.0 grant, third-party
notices and native source manifests are in `/usr/local/share/shimmy`.
Debian package copyright/license files remain in `/usr/share/doc`;
corresponding package sources are available from https://sources.debian.org/.
See [licensing](LICENSING.md), [supported fixtures](docs/SUPPORTED_MODELS.md)
and [security](SECURITY.md). Commercial use is permitted subject to applicable
component licenses; the original libfse crate metadata is historically unchanged.

The manual-only Docker workflow prepares checksum-verified licensed release artifacts and validates both architecture images before publishing only versioned tags. The recorded x86-64 container checks ran on Linux/WSL; ARM64 container runtime validation used QEMU and does not establish native ARM64 GPU inference. It does not run Cargo publication, move product tags, publish
crates or update `latest`. Local equivalent:

```bash
python3 .github/scripts/container_distribution.py prepare --arch amd64 --context build-context
docker build -f deploy/Dockerfile -t shimmy:2.6.7-local build-context
python3 .github/scripts/container_distribution.py test --arch amd64 --image shimmy:2.6.7-local --model /absolute/path/to/Qwen3-0.6B-Q4_K_M.gguf --output container-result.json
```

For historical 2.6.6 images, use the explicit `:2.6.6` tag. The `latest` tag has intentionally not been moved to 2.6.7; always request the versioned tag above.
