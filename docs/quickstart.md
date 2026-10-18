# Quick Start: Shimmy in 30 Seconds

Shimmy is a **single-binary** OpenAI-compatible local LLM server. One binary per
platform — the Airframe GPU engine (pure-Rust WGSL inference) is compiled in, so
no CUDA, no Vulkan SDK, no C++ toolchain, no compilation is required.

## 1. Download Pre-Built Binary

Pick your platform — all binaries use automatic WebGPU (wgpu) GPU detection:

```bash
# Windows x64
curl.exe -fL --retry 3 https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# Linux x86_64
curl -fL --retry 3 https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy

# macOS ARM64 (Apple Silicon)
curl -fL --retry 3 https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy

```

GPU detection is automatic — on first launch Airframe selects the best adapter
(discrete GPU preferred over integrated, integrated over CPU fallback).

> **Windows / PowerShell:** the command above uses `curl.exe` (the real curl that
> ships with Windows 10+). In PowerShell, bare `curl` is an alias for
> `Invoke-WebRequest`, which has no `-L` flag and will fail with
> *"A parameter cannot be found that matches parameter name 'L'."* — always call
> `curl.exe` explicitly.
>
> **Available binaries:** the release workflow builds `shimmy-linux-x86_64`,
> `shimmy-windows-x86_64.exe`, and `shimmy-macos-arm64`. If a platform asset is
> missing from a release, install from crates.io with `cargo install shimmy` or
> build from source using the instructions below.

### Build from Source / `cargo install`

```bash
# Install from crates.io (Airframe GPU engine)
cargo install shimmy --locked

# Build the full GPU build from source
git clone https://github.com/Michael-A-Kuykendall/shimmy
cd shimmy
cargo build --release --locked
```

The Airframe GPU engine is a public crate on [crates.io](https://crates.io/crates/airframe)
and builds from source automatically.

## 2. Get a Model

Place any `.gguf` file in one of these locations:
- `./models/your-model.gguf`
- Set `SHIMMY_BASE_GGUF=/path/to/your-model.gguf`
- Or just drop it in the HuggingFace / Ollama / LM Studio caches — Shimmy auto-discovers

```bash
# Download TinyLlama Q4_0 (about 609 MiB); no Python tooling required.
mkdir -p ~/models
curl -fL --retry 3 -C - \
  -o ~/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf \
  https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_0.gguf
```

## 3. Start Shimmy

```bash
./shimmy serve --model-path ~/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf --bind 127.0.0.1:11435
```

That's it! Shimmy is now running on `http://localhost:11435` (auto-allocates a
port if that one is taken). Use the address printed at startup.

## Network access and API keys

Local use on `127.0.0.1` or `::1` works without configuration. To listen on a
LAN/container address, set a strong key first; Shimmy refuses a non-loopback bind
without one.

```bash
export SHIMMY_API_KEY="$(openssl rand -hex 32)"
./shimmy serve --bind 0.0.0.0:11435 --model-path ~/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf
curl -f -H "Authorization: Bearer $SHIMMY_API_KEY" http://127.0.0.1:11435/v1/models
```

When a key is configured, all inference, model-management, diagnostic and
WebSocket routes require it. `GET /health` stays available for health probes.
OpenAI clients use that key instead of a placeholder; Anthropic clients may use
the `x-api-key` header. WebSocket clients must be able to send an authorization
header. Use TLS at your reverse proxy for remote access.

Browser access is disabled by default. Set `SHIMMY_ALLOWED_ORIGINS` to a
comma-separated list of exact browser origins, for example
`http://localhost:3000,https://chat.example.com`. Wildcards are rejected.
Allowed browser origins still need the configured API key.

For Docker Compose, export `SHIMMY_API_KEY` before
`docker compose -f templates/docker/docker-compose.yml up --build`.
The example publishes its host port on loopback and mounts
`templates/docker/models/` at `/app/models`; place your GGUF files there first.
With `docker run`, bind-mount your existing model directory at `/app/models`.
For Kubernetes, create the
credential before applying the manifests:

```bash
kubectl create secret generic shimmy-api --from-literal=api-key="$SHIMMY_API_KEY"
kubectl apply -f templates/kubernetes/
```

The Kubernetes example uses a ClusterIP service. Add an authenticated TLS ingress
deliberately if external access is needed.

## 4. Connect Your Tools

**VSCode Copilot** (`settings.json`):
```json
{ "github.copilot.advanced": { "serverUrl": "http://localhost:11435" } }
```

**Continue.dev**:
```json
{
  "models": [{
    "title": "Local Shimmy",
    "provider": "openai",
    "model": "your-model-name",
    "apiBase": "http://localhost:11435/v1"
  }]
}
```

**Cursor**: set custom endpoint to `http://localhost:11435`

**Any OpenAI SDK**: point `baseURL` at `http://127.0.0.1:11435/v1`, use any
placeholder API key for unauthenticated loopback use, or the configured
`SHIMMY_API_KEY` when authentication is enabled.

## 5. Test It

```bash
# List available models
./shimmy list

# Test generation
./shimmy --model-dirs ~/models list --short
# Use the exact discovered name from list --short (normalized to lowercase).
./shimmy --model-dirs ~/models generate tinyllama-1.1b-chat-v1.0.q4-0 --prompt "What is the capital of France?" --max-tokens 32

# Or use curl
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tinyllama-1.1b-chat-v1.0.Q4_0",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 10
  }'
```

### Repeatable model-backed smoke test

From a source checkout, after `cargo build --release --locked`:

```bash
python3 scripts/smoke_text.py --binary target/release/shimmy \
  --model ~/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf \
  --log /tmp/shimmy-smoke-server.log
```

This optional development script starts an isolated server, checks model discovery,
JSON chat, SSE chat, text completions, native generation and structured errors,
then stops the server. It checks the response content and exits nonzero on failure.

## GPU Acceleration

- No backend flags — WebGPU (wgpu) is selected automatically: D3D12 on Windows,
  Vulkan on Linux, Metal on macOS.
- `shimmy gpu-info` shows whether Airframe was compiled in; the actual selected
  adapter is logged when a model loads.
- GGUF inference requires the Airframe engine and a compatible GPU driver.
  A build without default features is useful for CPU-only protocol tests; it
  does not provide a GGUF CPU inference backend.

## Extended Context

`SHIMMY_MAX_CTX` overrides the context window. When set above the model's native
window, Airframe automatically engages YaRN RoPE scaling and resizes the KV
cache:

```bash
# 4096-token context with YaRN (2× native for TinyLlama)
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=4096 shimmy serve

# 8192 tokens (4× native, higher RoPE compression)
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=8192 shimmy serve
```

Accepted range is 512–131072. Values outside that range are silently ignored.

## VRAM Sizing Reference

Airframe allocates VRAM at load time: **weights** + **KV cache**. The KV cache
is F32 (or INT4 with [TurboShimmy](turboshimmy.md)) and scales linearly with
context length:

```
KV cache = n_layers × n_kv_heads × head_dim × ctx × 2 × bytes_per_elem
```

**TinyLlama 1.1B Q4_0:**

| Context (`SHIMMY_MAX_CTX`) | KV cache | Weights | Total | Min VRAM |
|---|---|---|---|---|
| 2048 (default) | ~88 MB | ~638 MB | ~726 MB | ~800 MB |
| 4096 | ~176 MB | ~638 MB | ~814 MB | ~900 MB |
| 8192 | ~352 MB | ~638 MB | ~990 MB | ~1.1 GB |
| 16384 | ~704 MB | ~638 MB | ~1.3 GB | ~1.5 GB |

**Larger models:**

| Model | Quant | Weights | KV @ 2048 ctx | Min VRAM |
|---|---|---|---|---|
| Llama 3.2 1B | Q4_0 | ~636 MB | ~128 MB | ~900 MB |
| Llama 3.2 3B | Q4_0 | ~1.9 GB | ~448 MB | ~2.5 GB |
| Mistral 7B | Q4_K_M | ~4.1 GB | ~512 MB | ~5 GB |
| Llama 3 8B | Q4_K_M | ~4.7 GB | ~512 MB | ~5.5 GB |

Integrated graphics (Intel Iris, Apple M-series unified memory, AMD Vega) running
at 2048 context sits around ~800 MB — comfortably inside the 2 GB allocation
most integrated GPUs share with system RAM.

## Troubleshooting

**No models found?**
- Make sure your `.gguf` file is in `./models/` or set `SHIMMY_BASE_GGUF`
- Run `./shimmy discover` to see what Shimmy can find

**Port already in use?**
```bash
./shimmy serve --bind 127.0.0.1:11436
```

**Need help?**
- [Open an issue](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- Check existing [discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)

---

**Next**: [integrations](integrations.md) for more examples · [API reference](API.md) ·
[supported models](SUPPORTED_MODELS.md) · [configuration](CONFIGURATION.md)
