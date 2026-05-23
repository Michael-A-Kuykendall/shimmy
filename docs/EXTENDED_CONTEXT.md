# Extended Context and YaRN RoPE Scaling

Everything about running Shimmy beyond the model's native context window — how YaRN works, what it costs, and what's actually safe to run.

---

## What Is "Context"?

A language model's **context window** is the maximum number of tokens it can attend to at once. Everything in the current conversation — system prompt, history, and the new message — must fit within this limit.

Exceeding the context limit causes silent truncation or garbled output. It does **not** produce an error.

**Token count reference:**
- 1 token ≈ 0.75 words in English
- A typical system prompt is 50–200 tokens
- A long user turn might be 500–2000 tokens
- A 2048-token window fits roughly a page and a half of text total

---

## How RoPE Works (Brief)

RoPE (Rotary Positional Embeddings) encodes position information directly into the query and key vectors of the attention mechanism. Instead of adding positional embeddings at the input layer, RoPE rotates Q and K vectors by an angle proportional to their position in the sequence.

For position `t` in a dimension pair `(2i, 2i+1)`:
```
θ(t, i) = t / base^(2i / d_head)
```
where `base` is typically 10000.

The rotation means attention scores between tokens automatically reflect their relative positions. The problem: when you ask the model to attend to tokens at positions beyond what it was trained on, the rotation angles are in a regime the model has never seen, causing attention to degrade and output quality to collapse.

---

## YaRN: Scaling RoPE Beyond Training Length

**YaRN** (Yet another RoPE extensioN) addresses this by computing a **scale factor** `s` that compresses the position indices so they stay within the trained range:

```
θ_scaled(t, i) = (t / s) / base^(2i / d_head)
```

Where `s = max_ctx / native_ctx`. This stretches the position space — position 4096 in a model with native 2048 context is mapped to position 2048 (the model's maximum trained position), keeping attention in a well-behaved regime.

Airframe computes this automatically when `SHIMMY_MAX_CTX > native_ctx`:

```rust
let yarn_scale = max_ctx as f32 / native_ctx as f32;
// Passed to RoPE shader as a push constant
```

The WGSL RoPE shader divides the position index by `yarn_scale` before computing rotation angles.

---

## When YaRN Engages

| `SHIMMY_MAX_CTX` vs native context | What happens |
|---|---|
| `SHIMMY_MAX_CTX` not set | Native context used, no scaling, maximum quality |
| `SHIMMY_MAX_CTX ≤ native_ctx` | Native context clamped to `SHIMMY_MAX_CTX`, no scaling |
| `SHIMMY_MAX_CTX > native_ctx` | YaRN scale = ctx / native, KV cache resized |

Llama-3.2 models have a native context of 131072 — setting `SHIMMY_MAX_CTX=8192` on a Llama-3.2 model actually **reduces** context and disables YaRN (you're staying within the trained range). YaRN only activates for TinyLlama (native: 2048), Phi-2 (native: 2048), and similar models with small native windows.

---

## VRAM Cost of Extended Context

The KV cache stores keys and values for every layer at every context position. Its size grows linearly with context length.

**Formula:**
```
KV_cache_bytes = n_layers × n_kv_heads × head_dim × max_ctx × 2 × sizeof(f32)
                                                              ↑ key + value
```

For **TinyLlama 1.1B** (22 layers, 4 KV heads, head_dim=64):
```
KV @ 2048  = 22 × 4 × 64 × 2048 × 2 × 4 = ~88 MB
KV @ 4096  = 22 × 4 × 64 × 4096 × 2 × 4 = ~176 MB
KV @ 8192  = 22 × 4 × 64 × 8192 × 2 × 4 = ~352 MB
KV @ 16384 = 22 × 4 × 64 × 16384 × 2 × 4 = ~704 MB
```

For **Llama-3.2-1B** (16 layers, 8 KV heads, head_dim=64):
```
KV @ 8192  = 16 × 8 × 64 × 8192 × 2 × 4 = ~512 MB
KV @ 32768 = 16 × 8 × 64 × 32768 × 2 × 4 = ~2.0 GB
```

For **Llama-3.2-3B** (28 layers, 8 KV heads, head_dim=128):
```
KV @ 8192  = 28 × 8 × 128 × 8192 × 2 × 4 = ~1.8 GB
KV @ 16384 = 28 × 8 × 128 × 16384 × 2 × 4 = ~3.6 GB  ← exceeds total VRAM on 3-4 GB cards
```

**Total VRAM = weights + KV cache + small activation buffers (~50 MB)**

---

## Hardware Context Size Guide

How far you can practically push context on consumer hardware:

### 6 GB VRAM (GTX 1060, RTX 3060, RX 6600)

| Model | Max safe ctx | Notes |
|---|---|---|
| TinyLlama 1.1B Q4_0 | 32768 | Weights 638 MB + KV 2.8 GB leaves ~2.5 GB headroom |
| Llama-3.2-1B Q4_K_M | 32768 | Weights 770 MB + KV 2 GB = ~2.8 GB total |
| Llama-3.2-3B Q4_K_M | 4096 | Weights 1.9 GB + KV 0.9 GB = ~2.8 GB |
| Phi-2 Q4_K_M | 8192 | Weights 1.7 GB + KV ~1.3 GB = ~3 GB |

### 8 GB VRAM (RTX 3070, RTX 4060)

| Model | Max safe ctx | Notes |
|---|---|---|
| TinyLlama 1.1B Q4_0 | 65536 | KV ~5.6 GB — tight, leaves little for activations |
| Llama-3.2-1B Q4_K_M | 65536 | KV ~4 GB total |
| Llama-3.2-3B Q4_K_M | 8192 | KV ~1.8 GB, total ~3.7 GB, comfortable |
| Phi-2 Q4_K_M | 16384 | KV ~2.6 GB, total ~4.3 GB |

### 12 GB VRAM (RTX 3060 12GB, RTX 4080 12GB)

| Model | Max safe ctx | Notes |
|---|---|---|
| Llama-3.2-3B Q4_K_M | 16384 | KV ~3.6 GB, total ~5.5 GB |
| Llama-3.2-3B Q4_K_M | 32768 | KV ~7.2 GB, total ~9.1 GB — feasible |

### Integrated GPU (4-8 GB shared RAM)

| Model | Max safe ctx | Notes |
|---|---|---|
| TinyLlama 1.1B Q4_0 | 4096 | KV 176 MB, total ~814 MB — fits comfortably |
| GPT-2 117M Q4_K_M | 1024 | Tiny model, good for testing |

---

## YaRN Quality Tradeoffs

YaRN is a reasonable approximation but not perfect. Quality degradation at extended context:

| Scale factor (ctx / native) | Expected quality | Notes |
|---|---|---|
| 1x | Full quality | Training distribution |
| 2x | Excellent | Well-tested range |
| 4x | Very good | Tested in needle bench (2K–8K on TinyLlama) |
| 8x | Good for most tasks | Noticeable degradation on complex long-range reasoning |
| 16x+ | Acceptable for retrieval, poor for generation | Use with caution |

**Needle bench results** (TinyLlama, retrieving a unique phrase from a long document):

| Context | Depth 15% | Depth 50% | Depth 85% |
|---|---|---|---|
| 2048 (native) | Pass | Pass | Pass |
| 4096 (2x YaRN) | Pass | Pass | Pass |
| 8192 (4x YaRN) | Pass | Pass | Marginal |

The "depth" is how deep in the context the needle is placed. Deeper placement is harder because the model must attend further back.

---

## Configuration

```bash
# Set context window at server start
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=8192 shimmy serve

# Override RoPE scale manually (advanced — normally computed automatically)
SHIMMY_ROPE_SCALE=4.0 SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve
```

**Valid range for SHIMMY_MAX_CTX:** 512 to 131072. Values outside this range are ignored and the model's native context is used.

`SHIMMY_ROPE_SCALE` overrides the automatically computed scale factor. Useful if you want to use a different scaling strategy than the default linear YaRN. Set to `1.0` to disable YaRN even when ctx > native.

---

## Per-Request Context Control

The maximum tokens for a request is controlled via `max_tokens` in the API:

```json
{
  "model": "tinyllama-1.1b",
  "messages": [...],
  "max_tokens": 512
}
```

`max_tokens` caps the *output* length, not the *input* length. The input (prompt + history) counts against the context window separately. If your total token count (input + max_tokens) approaches `SHIMMY_MAX_CTX`, the model will truncate.

---

## Further Reading

- [ARCHITECTURE.md](ARCHITECTURE.md) — how KV cache buffers are allocated and reset
- [QUANTIZATION.md](QUANTIZATION.md) — VRAM budget for weights
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — diagnosing OOM errors at extended context
