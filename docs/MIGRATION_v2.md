# Migrating from Shimmy v1.x to v2.0

This guide covers the breaking changes and new defaults introduced in Shimmy v2.0, which replaces the llama.cpp inference backend with the Airframe WebGPU engine.

---

## TL;DR

| What changed | v1.x | v2.0 |
|---|---|---|
| Default inference engine | llama.cpp | Airframe (WGSL/WebGPU) |
| GPU acceleration | CUDA / Vulkan / OpenCL / MLX | WebGPU via wgpu (auto-selected) |
| Model configuration | `--model-path` or hardcoded path | `SHIMMY_BASE_GGUF` env var or `--model-path` |
| `--gpu-backend cuda/vulkan/opencl` | Worked | Ignored (Airframe selects adapter via wgpu) |
| MoE models | Supported (default path) | Requires `--legacy` flag |
| `cargo install shimmy` | Broken (`publish = false`) | Works (installs huggingface engine) |
| Binary distribution | Pre-built binaries | Pre-built binaries (Airframe engine included) |

---

## What Is Airframe?

Airframe is a pure-Rust WebGPU inference engine. It replaces the C++ llama.cpp library with:

- WGSL compute shaders compiled at runtime by wgpu
- F32 precision throughout (no quantized-on-GPU approximations)
- YaRN RoPE scaling for extended context windows
- No C++ toolchain, no CUDA toolkit, no Vulkan SDK required

The llama.cpp code path is **historically parked** — it still works via `--legacy` but receives no new features.

---

## Breaking Changes

### 1. Default engine is now Airframe

The default inference path changed. No flag is needed for the new default.

```bash
# v1.x: implicit llama.cpp
shimmy serve --model-path /path/to/model.gguf

# v2.0: same command, now uses Airframe
shimmy serve --model-path /path/to/model.gguf

# v2.0: preferred way — set model via env var
SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve
```

### 2. `SHIMMY_BASE_GGUF` is now required (or `--model-path`)

In v1.x, some builds had a hardcoded default model path. v2.0 requires an explicit model path.

```bash
# Required: set the model path
export SHIMMY_BASE_GGUF=/path/to/TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf
shimmy serve

# Or use the flag:
shimmy serve --model-path /path/to/model.gguf
```

If neither is set, the server will fail to start with a clear error message.

### 3. `--gpu-backend cuda/vulkan/opencl` flags are ignored

In v1.x these flags selected the llama.cpp GPU backend. In v2.0, Airframe uses wgpu's adapter enumeration — the GPU is always auto-selected. The `--gpu-backend` flag is silently ignored by the Airframe engine.

```bash
# v1.x: forced CUDA
shimmy serve --gpu-backend cuda

# v2.0: all --gpu-backend values are ignored; wgpu selects the best available adapter
shimmy serve      # Airframe auto-selects GPU adapter via wgpu
```

To see which GPU adapter was selected:
```bash
shimmy gpu-info
```

### 4. MoE models require `--legacy`

Mixture-of-Experts models (e.g., Mixtral) require the llama.cpp backend. Airframe does not yet support MoE routing.

```bash
# v1.x: MoE worked by default
shimmy serve --cpu-moe --n-cpu-moe 8

# v2.0: must use --legacy for MoE
shimmy serve --legacy --cpu-moe --n-cpu-moe 8

# Or set via environment:
SHIMMY_ENGINE_BACKEND=llama shimmy serve --cpu-moe --n-cpu-moe 8
```

---

## What Stayed the Same

- **OpenAI API** (`/v1/chat/completions`, `/v1/models`, etc.) — 100% compatible, no client changes needed
- **Model format** — GGUF files work as before; Airframe reads the same GGUF metadata
- **Port and bind configuration** — `--bind`, `SHIMMY_PORT`, auto-allocation all work identically
- **Model discovery** — Hugging Face cache, Ollama directory, `./models/`, `SHIMMY_BASE_GGUF`
- **Streaming** — SSE and WebSocket streaming are unchanged
- **Template routing** — Chat templates are applied identically
- **`--legacy` flag** — Restores full llama.cpp behavior for any workflow that needs it

---

## Opting Back to v1.x Behavior

If you need the old llama.cpp engine for any reason:

```bash
# Per-invocation
shimmy serve --legacy --model-path /path/to/model.gguf

# Permanently via environment
export SHIMMY_ENGINE_BACKEND=llama
shimmy serve --model-path /path/to/model.gguf
```

The `--legacy` path supports all v1.x flags: `--gpu-backend cuda/vulkan/opencl`, `--cpu-moe`, `--n-cpu-moe`, etc.

---

## Extended Context (New in v2.0)

Airframe supports extended context via YaRN RoPE scaling. Set `SHIMMY_MAX_CTX` to the desired context length:

```bash
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=8192 shimmy serve
```

Supported values: `2048` (default), `4096`, `8192`, `16384`, `32768`. YaRN scaling activates automatically when `SHIMMY_MAX_CTX` exceeds the model's base context length.

---

## Upgrading

### Binary upgrade

Download the latest binary for your platform from [GitHub Releases](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest). Replace your existing binary.

### From cargo install

```bash
cargo install shimmy
```

> **Note**: `cargo install shimmy` installs the huggingface engine variant (no GPU). For the Airframe GPU engine, use the GitHub Releases binary.

---

## Getting Help

- [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- [Configuration Reference](CONFIGURATION.md)
- [Quickstart Guide](quickstart.md)
