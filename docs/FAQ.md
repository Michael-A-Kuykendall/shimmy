# Frequently Asked Questions

## General

### What is Shimmy?
Shimmy is a single-binary, OpenAI-compatible local LLM inference server. It
loads GGUF models and serves them through a 100% OpenAI-compatible HTTP API,
so existing AI tools (VSCode Copilot, Cursor, Continue.dev, OpenAI SDKs) work
with zero code changes — locally, privately, and free.

### What's the difference between Shimmy and llama.cpp / Ollama?
Shimmy is written in pure Rust with no C++ toolchain dependency. The Airframe
engine runs WGSL compute shaders compiled at startup — no pre-built binaries,
no driver version pinning. The result is faster startup (under 100 ms),
lower memory overhead, and deterministic output. See
[docs/GPU_PIPELINE.md](GPU_PIPELINE.md) for internals.

### Does Shimmy work on my GPU?
Shimmy uses WebGPU (via the Airframe engine), which runs on Vulkan, D3D12, and
Metal — covering NVIDIA, AMD, Intel, and Apple Silicon. No CUDA required.
See [docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md) if you hit adapter errors.

### Will Shimmy ever be paid?
No. Shimmy is MIT-licensed forever. There is no "free tier," no pivot to paid,
and no asterisks. If it helps you, consider
[sponsoring](https://github.com/sponsors/Michael-A-Kuykendall) — but you are
never required to.

## Models

### How do I get models?
Shimmy auto-discovers GGUF files from the HuggingFace cache, Ollama directory,
LM Studio cache (`~/.cache/lm-studio/models`), and any local `./models/`
folder. You can also point directly at a file with `SHIMMY_BASE_GGUF`. See
[docs/quickstart.md](quickstart.md) and [docs/MODEL_EXPANSION.md](MODEL_EXPANSION.md).

### Q4_K_M vs Q4_0 — which should I use?
`Q4_K_M` (K-quant) is consistently better quality than `Q4_0` for the same file
size. Use `Q4_0` only when you need maximum compatibility or the model isn't
available in K-quant. See [docs/QUANTIZATION.md](QUANTIZATION.md) for the full
analysis.

### Can I run multiple models at once?
Not currently — Shimmy loads one model per server instance. To serve multiple
models, run multiple server instances on different ports. Hot-swapping models
(reload without restart) is on the roadmap.

## Usage

### Why do I need `SHIMMY_BASE_GGUF` or `LIBSHIMMY_MODEL_PATH`?
You don't, unless you want to pin a specific file. If neither is set, Shimmy
auto-discovers models in standard directories (`~/.cache/huggingface`,
`~/.ollama`, `~/lm-studio/models`, `~/.cache/lm-studio/models`,
`~/Library/Application Support/LMStudio`). Set `SHIMMY_BASE_GGUF` to override
and point directly at a specific GGUF file.

### Why does generation stop before `max_tokens`?
The model reached a natural end-of-sequence token. For chat models this is
expected behavior — the model signals it's done. To force longer output,
increase `max_tokens` and set `temperature > 0`. If generation stops on the
wrong token, the model may be using the wrong chat template — see
[docs/CHAT_TEMPLATES.md](CHAT_TEMPLATES.md).

### Is there streaming support?
Yes. Set `"stream": true` in your request. Shimmy returns Server-Sent Events
in the standard OpenAI streaming format.

### Can I extend the context window beyond what the model was trained on?
Yes — set `SHIMMY_MAX_CTX` to any value. Airframe applies YaRN scaling
automatically when the requested context exceeds the model's native context.
Quality degrades gradually beyond 2× the native context. See
[docs/EXTENDED_CONTEXT.md](EXTENDED_CONTEXT.md).

### How much VRAM do I need?
VRAM = **weights** + **KV cache**. The KV cache is F32 (or INT4 with
[TurboShimmy](turboshimmy.md)) and scales linearly with context length:
`n_layers × n_kv_heads × head_dim × ctx × 2 × bytes_per_elem`. See
[docs/turboshimmy.md](turboshimmy.md) for VRAM tables and
[docs/EXTENDED_CONTEXT.md](EXTENDED_CONTEXT.md) for the formula.

## Troubleshooting

### I get a GPU error at startup — what do I do?
Run `shimmy gpu-info` to check the selected GPU adapter, then see
[docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md) for the GPU-specific section.

### My model generates gibberish / collapses mid-stream — what's wrong?
Run `shimmy list` to confirm the model loaded, and check that the GGUF is
standard (not a fused/oddly-split file). If prefill is finite but decode
collapses, verify the chat template — see
[docs/CHAT_TEMPLATES.md](CHAT_TEMPLATES.md) and
[docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md).

---

Still stuck? [Open an issue](https://github.com/Michael-A-Kuykendall/shimmy/issues)
or ask in [Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions).
