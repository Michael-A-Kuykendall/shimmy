# Shimmy v2.0 Architecture

## Overview

Shimmy is a single-binary OpenAI-compatible HTTP server for local GGUF inference.
In v2.0 the default inference backend is **Airframe** — a pure-Rust WebGPU (WGSL)
transformer runtime that requires no C++ toolchain, no CUDA SDK, and no Vulkan SDK.

---

## Component Map

```
shimmy (binary)
 ├── HTTP layer (axum / tokio)
 │    ├── POST /v1/chat/completions
 │    ├── POST /v1/completions
 │    └── GET  /v1/models
 ├── Engine bridge (src/engine/airframe.rs)
 │    └── converts OpenAI request → GenOptions → SamplingParams
 └── Airframe runtime  (../airframe crate, path dep)
      ├── GGUF loader (weights + metadata)
      ├── Tokenizer (shimmytok)
      ├── GPU pipeline (wgpu + WGSL shaders)
      │    ├── Embedding lookup
      │    ├── Per-layer transformer blocks
      │    │    ├── Attention (Q/K/V matmul + RoPE + softmax)
      │    │    └── FFN (gate / up / down projections)
      │    ├── Output head (dequant + logit projection)
      │    └── KV cache (GPU buffers, reset per request)
      └── Sampler (repeat penalty → temperature → top-p)
```

---

## Airframe Engine

### WGSL / WebGPU

Airframe uses **wgpu** as the WebGPU implementation layer.  wgpu translates WGSL
shaders to the best available native API:

| Platform | Native API used by wgpu |
|---|---|
| Windows | Direct3D 12 (primary) or Vulkan |
| Linux   | Vulkan (primary) or OpenGL |
| macOS   | Metal |

No Vulkan SDK or CUDA toolkit is required — wgpu drivers ship with the binary.
WGSL shaders are compiled at startup using `wgpu::Device::create_shader_module`.

### Quantization

Weights are stored in GGUF quantized format.  The GPU dequantization shaders
decode each block on the fly during the matrix multiply:

| GGUF type | GPU shader |
|---|---|
| F32 / F16 | passthrough / fp16 unpack |
| Q4_0      | 32-element block, 4-bit signed |
| Q8_0      | 32-element block, 8-bit signed |
| Q4_K_M    | super-block with row scales |
| Q5_K_M    | super-block, 5-bit with high-bit plane |
| Q6_K      | super-block, 6-bit |

Every GPU shader result is validated against the CPU reference before a model
is considered supported (`quant_verify` tool).

### KV Cache

The KV cache is a pair of GPU buffers (keys + values) allocated at server start
with capacity `n_layers × n_kv_heads × head_dim × max_ctx × sizeof(f32)`.
The cache is **fully reset between requests** — Shimmy is stateless.

### Extended Context (YaRN)

When `SHIMMY_MAX_CTX` exceeds the model's native context window, Airframe:
1. Recomputes the RoPE theta scale using the YaRN formula
2. Reallocates the KV cache to the requested length
3. Applies the scale to all position-dependent attention computations

---

## Request Lifecycle

```
HTTP POST /v1/chat/completions
  │
  ├─ serde_json deserialization (ChatCompletionRequest)
  ├─ Input validation (empty messages, max_tokens bounds)
  ├─ Chat template application → prompt string
  ├─ Engine bridge: build GenOptions + SamplingParams
  │    ├─ stop_tokens from chat template (e.g. <|eot_id|> for Llama-3)
  │    ├─ extra_stop_tokens propagated to SamplingParams
  │    └─ penalty fields mapped: repeat_penalty = 1 + max(freq, presence) * 0.5
  ├─ Airframe generate()
  │    ├─ Tokenize prompt
  │    ├─ Chunked prefill (512-token chunks)
  │    ├─ Decode loop (one token per step)
  │    │    ├─ GPU forward pass
  │    │    ├─ Sample (penalty → temperature → top-p)
  │    │    └─ Stop on EOS | extra_stop_tokens | max_tokens
  │    └─ Detokenize
  └─ OpenAI response JSON
```

---

## Model Discovery

On startup Shimmy scans the following paths for GGUF files:

1. `SHIMMY_BASE_GGUF` (explicit env var — highest priority)
2. `SHIMMY_MODEL_PATHS` (colon-separated list)
3. `~/.cache/huggingface/hub/` (HuggingFace cache)
4. `~/.ollama/models/` (Ollama model store)
5. `~/.cache/lm-studio/models/` (LM Studio)
6. `~/models/` (generic local dir)

---

## Known Limitations (v2.0)

| Limitation | Detail |
|---|---|
| 2 GB WebGPU buffer cap | Single tensor buffer ≤ 2 GB; output heads > 2 GB (e.g. Gemma-2-2B) require chunking (not yet implemented) |
| Fused QKV tensors | Phi-3 / Phi-3.5 pack Q+K+V into one weight; Airframe expects separate tensors — server will crash |
| Stateless only | No multi-turn KV cache persistence; each request starts fresh |
| Streaming | Token streaming via SSE is on the roadmap; `stream: true` is accepted but returns full completion |
| MoE models | CPU offloading for 70B+ MoE is on the roadmap; current architecture loads entire model on GPU |
