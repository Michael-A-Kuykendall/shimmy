# Docker — Shimmy 2.6.6

**Distribution boundary:** this coordinated release prepares crates.io packages.
It does not publish a 2.6.6 Docker/GHCR image or certify container GPU execution.
Do not assume a latest image, old Compose file or historical deployment template
contains the reviewed source.

## Optional local image after source/package publication

The tracked Dockerfiles build Rust source and expose **11435**; health probes
use curl against /health. A local source build needs the exact approved source
and its real registry dependencies to be available. Image builds, device
passthrough and native GPU/container stability are **NOT_RUN** in this release.

```bash
docker build -t shimmy-local:2.6.6 .
export SHIMMY_API_KEY="$(openssl rand -hex 32)"
docker run --rm --name shimmy-local \
  -p 127.0.0.1:11435:11435 \
  -e SHIMMY_API_KEY \
  -v /absolute/path/to/models:/models:ro \
  shimmy-local:2.6.6 \
  shimmy --ctx 512 --kv-quant f32 serve --bind 0.0.0.0:11435 --model-path /models/model.gguf
```

Replace both paths with a directory and exact verified fixture. Configure device
access separately for the actual vendor/runtime; the command above does not
promise successful GPU inference. Loading is lazy, so /health alone proves
neither model residency nor numerical correctness.

```bash
curl -f http://127.0.0.1:11435/health
curl -f -H "Authorization: Bearer $SHIMMY_API_KEY" http://127.0.0.1:11435/v1/models
```

Use a returned ID for the [chat request](docs/quickstart.md).
The container's non-loopback listener requires a key even when the host mapping
is loopback. Protect remote access with TLS. Never put real credentials in an
image, repository or Compose file.

## Current Compose example

This example is a local scaffold, not an executed GPU certification:

```yaml
services:
  shimmy:
    image: shimmy-local:2.6.6
    ports:
      - "127.0.0.1:11435:11435"
    environment:
      SHIMMY_API_KEY: "${SHIMMY_API_KEY:?set a private key first}"
    volumes:
      - "/absolute/path/to/models:/models:ro"
    command: ["shimmy", "--ctx", "512", "--kv-quant", "f32", "serve", "--bind", "0.0.0.0:11435", "--model-path", "/models/model.gguf"]
    healthcheck:
      test: ["CMD", "curl", "-f", "http://127.0.0.1:11435/health"]
```

Validate your saved Compose file using docker compose config before starting it.
Do not substitute an assumed published image.

## Historical deployment files

The root/deploy Compose and cloud/template configurations are legacy scaffolds,
**not current deployment instructions**: some retain obsolete ports, environment
keys, images or health tools. Do not deploy them unchanged. The current safe
manual example is above. Generated init templates have not gained deployment
certification in this release; see [deployment scope](deploy/README.md).

Docker does not emulate Metal/M4 or certify native Windows/macOS GPU behavior.
See [supported fixtures](docs/SUPPORTED_MODELS.md), [security](SECURITY.md)
and [configuration](docs/CONFIGURATION.md).
