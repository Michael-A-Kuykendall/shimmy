# GPU Pipeline Internals

A technical deep dive into how Airframe runs transformer inference on the GPU — shader dispatch architecture, the bindless resource model, KV cache management, and the sampler chain.

This document is for contributors modifying shaders, people debugging GPU-level failures, and anyone who wants to understand what's happening beneath the surface.

---

## Architecture Overview

```
Request arrives at HTTP handler
       │
       ▼
shimmy openai_compat layer
  - Parse JSON request
  - Apply chat template → prompt string
  - Build SamplingParams (temperature, top_p, penalties, stop tokens)
       │
       ▼
airframe runtime::gpu::GpuRuntime::generate()
  - Tokenize prompt (shimmytok)
  - Prefill phase: process prompt tokens → KV cache populated
  - Decode phase: autoregressive token-by-token generation
  - Detokenize → response string
       │
       ▼
shimmy → HTTP response
```

---

## The Bindless Resource Model

Traditional WebGPU requires each buffer to be bound individually to a binding slot. Large transformer models have thousands of tensors — binding them all individually would hit WebGPU's binding limits and create enormous overhead per dispatch.

Airframe uses a **bindless design**: weight tensors for each layer are packed into a single large storage buffer, and the WGSL shader indexes into this buffer using byte offsets passed as push constants (via a uniform buffer in WebGPU terms).

```
Layer buffer layout (one per transformer layer):
┌────────────────────────────────────────────────────────────┐
│  attn_norm_weight  │  attn_q.weight  │  attn_k.weight  │  │
│  attn_v.weight  │  attn_o.weight  │  ffn_norm_weight  │  │
│  ffn_gate.weight  │  ffn_up.weight  │  ffn_down.weight  │  │
└────────────────────────────────────────────────────────────┘
        ↑
  Each tensor section stored in GGUF quantized format.
  The shader knows each section's byte offset from a metadata buffer.
```

**Why this matters for debugging:** If you see a wgpu error about buffer binding limits or bind group creation failure, it's likely not the bindless buffers themselves but rather the KV cache or activation buffers (which are still individually bound).

---

## Prefill Phase — Chunked Processing

The **prefill phase** processes the entire input prompt at once to populate the KV cache. It's the most GPU-compute-intensive part of a request.

Airframe splits long prompts into **512-token chunks** to avoid GPU command encoder timeouts:

```
prompt = 2048 tokens
           │
  ┌────────▼────────┐
  │  chunk 1: 512   │ → KV cache slots 0..511   ← forward pass, write KV
  │  chunk 2: 512   │ → KV cache slots 512..1023
  │  chunk 3: 512   │ → KV cache slots 1024..1535
  │  chunk 4: 512   │ → KV cache slots 1536..2047
  └─────────────────┘
           │
  Only the last chunk's logits are used for the first sampled token.
  The KV state from all chunks is preserved in the KV cache GPU buffers.
```

**Why 512?** WebGPU imposes a max dispatch time of a few seconds before the OS kills the GPU command. Prefilling 512 tokens at a time stays well within this limit on all tested hardware (RTX 3060 to integrated Intel). A single chunk at 512 tokens takes roughly 100-400ms depending on the model and GPU.

**Debug tracing:** Set `AIRFRAME_TRACE_PREFILL_CHUNKS=1` to log chunk boundaries and timing.

---

## Decode Phase — Autoregressive Generation

After prefill, the decoder generates one token at a time. Each decode step:

1. **Embed** the previous token via the embedding lookup table
2. **Forward pass** through all transformer layers with `seq_len = current_position + 1`
   - Attention uses the KV cache — current Q attends to all past K,V pairs
   - Only the new Q/K/V vectors for position `t` need to be computed
3. **Output projection** — apply the LM head to get logits over vocabulary
4. **Sample** — select next token from logits distribution
5. **Check stop conditions** — EOS token, extra stop tokens, max_tokens
6. Repeat from step 1 with the new token

Each decode step issues **one full forward pass** of the model. For a 22-layer model (TinyLlama), that's 22 attention computations + 22 FFN computations per token.

**Decode is memory-bandwidth bound**, not compute-bound. The bottleneck is reading weight tensors from VRAM. Larger GPUs with more VRAM bandwidth generate tokens faster.

---

## Transformer Layer Computation (WGSL Shader Anatomy)

Each transformer layer runs these operations in order:

```
Input activations (shape: [seq_len, n_embd])
  │
  ├── RMS Norm → normalized_x (stable forward pass)
  │
  ├── Q projection: normalized_x × attn_q.weight → Q [seq, n_heads, head_dim]
  ├── K projection: normalized_x × attn_k.weight → K [seq, n_kv_heads, head_dim]
  ├── V projection: normalized_x × attn_v.weight → V [seq, n_kv_heads, head_dim]
  │
  ├── RoPE: apply rotary positional encoding to Q and K
  │     θ(t, i) = (t / yarn_scale) / base^(2i / head_dim)
  │     YaRN: yarn_scale = max_ctx / native_ctx when ctx > native
  │
  ├── Write K, V to KV cache at position t
  │
  ├── Attention scores: Q × Kᵀ / √head_dim → scores [seq, n_heads, seq]
  ├── Softmax over seq dimension → weights
  ├── Weighted sum: weights × V → attn_out [seq, n_heads, head_dim]
  │
  ├── Output projection: attn_out × attn_o.weight → residual_add
  │
  ├── Residual: x = x + residual_add
  │
  ├── RMS Norm → normalized_for_ffn
  │
  ├── FFN gate: normalized_for_ffn × ffn_gate.weight → gate
  ├── FFN up:   normalized_for_ffn × ffn_up.weight   → up
  ├── SwiGLU activation: gate × sigmoid(gate) × up
  ├── FFN down: swiglu_out × ffn_down.weight → ffn_residual
  │
  └── Residual: x = x + ffn_residual → output activations
```

**Dequantization happens inline**: each matrix multiplication's WGSL shader decodes the quantized weight block on the fly and multiplies by the activation. No separate dequantize-then-multiply step.

---

## KV Cache

The KV cache stores past keys and values to avoid recomputing them on every decode step.

**Buffer layout:**
```
key_cache:   [n_layers][n_kv_heads][max_ctx][head_dim]  f32
value_cache: [n_layers][n_kv_heads][max_ctx][head_dim]  f32
```

Allocated as a flat F32 GPU buffer. Total size:
```
key_cache_bytes = n_layers × n_kv_heads × max_ctx × head_dim × 4
value_cache_bytes = same
total = key_cache_bytes × 2
```

For TinyLlama @ 2048 ctx: `22 × 4 × 2048 × 64 × 2 × 4 = ~88 MB`

**Write:** During prefill and each decode step, the new K and V vectors for the current position are written to `cache[layer][head][position]`.

**Read:** During attention, the full K and V slices up to `current_position` are read back for the dot-product attention.

**Reset:** The cache is reset between requests (all positions zeroed). Shimmy is **stateless** — there is no session-level KV cache.

---

## The Sampler Chain

After the forward pass produces logits over the vocabulary (a float vector of length `vocab_size`), sampling applies these transformations in order:

```
logits[vocab_size]
  │
  1. Repetition penalty (if repeat_penalty > 1.0):
     For each token t that appeared in the past N tokens:
       logits[t] /= repeat_penalty   (increases "cost" of repeating)
  │
  2. Temperature scaling:
     logits[i] = logits[i] / temperature
     (temperature = 0.0 → greedy; temperature → ∞ → uniform random)
  │
  3. Softmax:
     probs[i] = exp(logits[i]) / Σ exp(logits[j])
  │
  4. Top-p (nucleus) sampling:
     Sort probs descending.
     Cumulate until sum ≥ top_p threshold.
     Renormalize the kept tokens.
     Sample from this reduced distribution.
  │
  5. Sample → token_id
```

**Greedy decoding** (deterministic output): `temperature=0.0, top_p=1.0`

**Creative generation**: `temperature=0.8, top_p=0.95`

**Repetition penalty** maps from the API's `frequency_penalty` / `presence_penalty` fields:
```
raw = max(frequency_penalty, presence_penalty)
if raw > 0.0: repeat_penalty = 1.0 + raw * 0.5
```

The sampler runs on CPU (the logits are read back from GPU memory after the forward pass). For vocabulary sizes up to 32K this adds < 1ms per token.

---

## Shader Dispatch Pattern

Each matrix multiplication is a separate compute dispatch. For a single decode step on TinyLlama (22 layers × ~6 dispatches per layer):

```
Approximate dispatch count per token:
  22 × (Q_proj + K_proj + V_proj + attn_out + FFN_gate/up + FFN_down)
  = ~132 dispatches per token
  + 22 RMSNorm + 22 RoPE + 22 attention_scores + 22 softmax
  = ~220 total GPU dispatches per token
```

Each dispatch uses workgroups of `(16, 16, 1)` threads. The encoder is submitted as a single command buffer per token — not one submit per dispatch. This is critical for performance: one GPU round-trip per token rather than 220.

**Debug dispatch tracing:** Set `SHIMMY_DEBUG_RAW=1` to log raw activation values through the pipeline (very verbose — for shader debugging only).

---

## Output Head

The final layer produces activations of shape `[1, n_embd]` (for the last position). These are multiplied by the **output embedding weight** (`output.weight`, shape `[vocab_size, n_embd]`) to produce logits.

The output weight is typically the most memory-intensive allocation: for a 32K vocabulary and 4096-dim model, `output.weight` is `32768 × 4096 × sizeof(f32) = 512 MB` in full precision. In GGUF format it's stored as Q6_K, reducing it to ~210 MB.

**WebGPU 2 GB buffer limit:** The output weight must fit in a single GPU buffer. Models where the output embedding tensor exceeds ~2 GB (e.g., Gemma-2-2B with vocab_size=256128) cannot be loaded without chunked output head support. This is a known limitation — see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

---

## Profiling and Performance

**Token generation speed** depends primarily on:
1. GPU memory bandwidth (not compute FLOPS) — weight reads dominate
2. Model size — more parameters = more bytes read per token
3. Context length — attention scores scale O(n) in decode (Q × all past K)

**Rough benchmarks on RTX 3060 12GB:**

| Model | Context | Tokens/sec |
|---|---|---|
| TinyLlama 1.1B Q4_0 | 2048 | ~35-50 tok/s |
| Llama-3.2-1B Q4_K_M | 2048 | ~30-45 tok/s |
| Llama-3.2-3B Q4_K_M | 2048 | ~12-18 tok/s |
| Phi-2 2.7B Q4_K_M | 2048 | ~10-15 tok/s |

Prefill is typically 3-10x faster per token than decode because it processes multiple tokens per pass (parallelism over the sequence dimension in attention and FFN).

---

## Contributing: Adding a New Architecture

To add support for a new model architecture (e.g., a new attention variant):

1. **Add architecture detection** in `src/family/` — parse the `general.architecture` GGUF metadata key
2. **Map tensor names** — create a new `TensorMap` variant that maps GGUF tensor names to Airframe's expected slot names
3. **Handle architecture variants** — if the new arch has unique layers (QK norm, Mamba blocks, etc.), add WGSL shader variants
4. **Add `quant_verify` validation** — run the new model through `cargo run --bin quant_verify` to confirm GPU/CPU dequant agreement
5. **Add smoke test entry** — add the model to `scripts/model_smoke_test.ps1` `$VerifiedModels` array

See [MODEL_EXPANSION.md](MODEL_EXPANSION.md) for the step-by-step process.

---

## Further Reading

- [QUANTIZATION.md](QUANTIZATION.md) — deep dive on shader dequantization
- [EXTENDED_CONTEXT.md](EXTENDED_CONTEXT.md) — YaRN RoPE scaling implementation
- [ARCHITECTURE.md](ARCHITECTURE.md) — system-level architecture and component map
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — debugging GPU failures
