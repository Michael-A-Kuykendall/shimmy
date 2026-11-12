# Quickstart — Shimmy 2.6.7

## Verified native downloads

The separately validated [2.6.7 native archive installation guide](BINARY_INSTALL.md) covers Linux x86-64/ARM64, Windows x86-64 and macOS ARM64, with exact checksums and retained component license materials. Native CLI tests and GPU model certification are different gates.

## Install the coordinated crate

Install Shimmy 2.6.7 from crates.io:

```bash
cargo install shimmy --version 2.6.7 --locked
shimmy --version
```

This compiles the default Airframe 0.4.6 engine and needs a Rust toolchain,
platform build prerequisites and a working wgpu driver. Transitive dependencies
may also require a native compiler/linker; the application being Rust/WGSL does
not guarantee a Rust-only dependency build. Do not disable default
features to obtain inference: the fast profile is a non-inference test build.

**Shimmy 2.6.7 is published on crates.io, GitHub Releases, GHCR, Homebrew and Scoop.** The `latest` container tag is intentionally unchanged and npm is not provided. Verify the explicit version and published checksums; those channels were independently verified after publication. Any further channel requires separate publication and
verification. Verify the installed executable with shimmy --version.

## Select and verify a model

Use an exact fixture from [SUPPORTED_MODELS](SUPPORTED_MODELS.md), not an
arbitrary GGUF carrying a familiar family/quantization label. Obtain weights
under their own license and verify SHA-256 before serving:

```bash
sha256sum /path/to/model.gguf
shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf --bind 127.0.0.1:11435
```

The path is a placeholder for your verified fixture. `--model-path` registers
the file under its filename without the extension. The explicit bind uses that
address; only `auto` requests port allocation. Use the address printed by the
server.

```bash
curl -f http://127.0.0.1:11435/health
curl -f http://127.0.0.1:11435/v1/models
```

Use a returned model ID in the request below, replacing `MODEL_ID`:

```bash
curl -f http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"MODEL_ID","messages":[{"role":"user","content":"The capital of France is"}],"temperature":0,"max_tokens":8,"stream":false}'
```

JSON is the default when `stream` is omitted. Chat `stream:true` uses SSE;
text `/v1/completions` streaming is rejected. See [API](API.md).

## Network access and API keys

Loopback may run without a key. Non-loopback binds, including the listener inside
a container, require `SHIMMY_API_KEY`: at least 16 printable ASCII characters,
without spaces. Generate a private key; do not commit it:

```bash
export SHIMMY_API_KEY="$(openssl rand -hex 32)"
shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf --bind 0.0.0.0:11435
curl -f -H "Authorization: Bearer $SHIMMY_API_KEY" http://127.0.0.1:11435/v1/models
```

Use a TLS reverse proxy and network access controls for remote traffic. Set
`SHIMMY_ALLOWED_ORIGINS` to exact allowed browser origins if needed; wildcard
origins are rejected. Origin permission never replaces credentials.

With a configured key, inference, management, metrics, diagnostics, docs and
WebSocket routes require credentials. GET /health is credential-exempt, but an
unallowed Origin header is rejected there too. See [CONFIGURATION](CONFIGURATION.md)
and [Docker instructions](../README-DOCKER.md).

## Readiness and limits

Loading is lazy by default. `SHIMMY_WARMUP=1` loads a selected model and performs
a one-token dummy request before readiness; it shifts cold work to startup,
does not remove that work, and resets request state afterward.
See [warm-up rules](CONFIGURATION.md#startup-warm-up).

The tested numerical path is F32 KV/reference kernels on exact dense fixtures.
WSL device-loss stability remains open. Apple M1 (Virtual) / Paravirtual Metal 198/198 full-logit certification passed for the released 2.6.7 / 0.4.6 source; physical M4 and other unexecuted configurations remain NOT_RUN.
See the [fixture matrix](SUPPORTED_MODELS.md) and [troubleshooting](TROUBLESHOOTING.md).
