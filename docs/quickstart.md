# Quick Start: Shimmy in 30 Seconds

Shimmy is a **single-binary** OpenAI-compatible local LLM server. One binary per
platform — the Airframe GPU engine (pure-Rust WGSL inference) is compiled in, so
no CUDA, no Vulkan SDK, no C++ toolchain, no compilation is required.

## 1. Download Pre-Built Binary

Pick your platform — all binaries use automatic WebGPU (wgpu) GPU detection:

```bash
# Windows x64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# Linux x86_64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy

# macOS Intel
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel -o shimmy && chmod +x shimmy

# Linux ARM64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64 -o shimmy && chmod +x shimmy
```

GPU detection is automatic — on first launch Airframe selects the best adapter
(discrete GPU preferred over integrated, integrated over CPU fallback).

### Build from Source / `cargo install`

```bash
# Install from crates.io (huggingface engine)
cargo install shimmy

# Build the full GPU build from source
git clone https://github.com/Michael-A-Kuykendall/shimmy
cd shimmy
cargo build --release
```

The Airframe GPU engine is a public crate on [crates.io](https://crates.io/crates/airframe)
and builds from source automatically.

## 2. Get a Model

Place any `.gguf` file in one of these locations:
- `./models/your-model.gguf`
- Set `SHIMMY_BASE_GGUF=/path/to/your-model.gguf`
- Or just drop it in the HuggingFace / Ollama / LM Studio caches — Shimmy auto-discovers

```bash
# Example: pull a small certified model
huggingface-cli download TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF \
  --include "tinyllama-1.1b-chat-v1.0.Q4_0.gguf" --local-dir ./models/
```

## 3. Start Shimmy

```bash
./shimmy serve
```

That's it! Shimmy is now running on `http://localhost:11435` (auto-allocates a
port if that one is taken).

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
placeholder API key.

## 5. Test It

```bash
# List available models
./shimmy list

# Test generation
./shimmy generate --name your-model --prompt "Hello!" --max-tokens 10

# Or use curl
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 10
  }'
```

## GPU Acceleration

- No backend flags — WebGPU (wgpu) is selected automatically: D3D12 on Windows,
  Vulkan on Linux, Metal on macOS.
- `shimmy gpu-info` shows the selected adapter.
- `shimmy serve --gpu-backend cpu` forces CPU mode (slow, but works everywhere).

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
