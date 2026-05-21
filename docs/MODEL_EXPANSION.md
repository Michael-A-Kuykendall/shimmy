# Airframe Model Expansion Guide

## How We Got Here: The Lineage

**libshimmy** (`../libshimmy`) is where this engine was born. The founding constraint from its
`SPEC.md` was deliberate and explicit:

> *"v0 targets one pinned model artifact and fails fast otherwise."*
>
> *"Preferred target model file: `TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf`"*

That choice drove the entire design. The engine was built FP32-first (correctness before
optimization), with a single pinned model so there was no ambiguity in what "correct" meant.
Once the CPU reference was provably correct against fixtures, the GPU bindless pipeline was added
in `libshimmy/crates/airframe/`, proven to match the CPU reference, and then promoted into the
standalone `airframe` crate that powers Shimmy v2.0.

The 0.06 mean absolute logit error between the GPU engine and the CPU reference is the documented
tolerance. It is lower precision than llama.cpp's Q8_0-input path — and that is intentional.
libshimmy is higher precision; the difference comes from llama.cpp voluntarily adding quantization
noise to its activations for AVX2 efficiency.

---

## What the Engine Can Load Today

### Supported Quantization Types

All of these are implemented in `airframe/src/core/dequant/` with both a CPU reference and a validated WGSL GPU dequant shader. GPU vs CPU agreement is confirmed per-type via `quant_verify`.

| Type | GGML ID | Block size | Bytes/block | quant_verify | Notes |
|------|---------|-----------|-------------|-------------|-------|
| `F32` | 0 | 1 | 4 | ✅ | Raw floats — maximum precision |
| `F16` | 1 | 1 | 2 | ✅ | Half-precision floats |
| `Q4_0` | 2 | 32 | 18 | ✅ | Original 4-bit format. Used by TinyLlama |
| `Q8_0` | 8 | 32 | 34 | ✅ | 8-bit activations / high-quality quant |
| `Q4_K` | 12 | 256 (superblock) | 144 | ✅ | K-quant 4-bit. Main weights in Q4_K_M GGUFs |
| `Q5_K` | 13 | 256 (superblock) | 176 | ✅ | K-quant 5-bit. Mixed-precision layers in Q4_K_M/Q5_K_M |
| `Q6_K` | 14 | 256 (superblock) | 210 | ✅ | K-quant 6-bit. Output head and embeddings |

**What this means in practice:** Any standard Q4_K_M or Q5_K_M GGUF will load and run — those formats use Q4_K, Q5_K, and Q6_K across their layers, all of which are fully supported.

### Supported Model Architectures

Defined in `airframe/src/core/spec.rs`. Context window is read from each model's GGUF `{arch}.context_length` key — no hardcoded limits.

| Architecture | GGUF `general.architecture` | Representative model | quant_verify | Status |
|---|---|---|---|---|
| Llama | `"llama"` | TinyLlama 1.1B, Llama-3.2-1B/3B | ✅ | Fully validated — deterministic inference confirmed |
| Mistral | `"mistral"` | Mistral-7B | ✅ (same path as Llama) | Declared; same tensor layout as Llama |
| Phi-2 | `"phi2"` | phi-2 | ✅ | GPU math verified |
| Phi-3 | `"phi3"` | phi3-mini-4k | ✅ | GPU math verified; fused QKV (`attn_qkv.weight`) requires code change before full inference |
| Gemma-2 | `"gemma2"` | gemma-2-2b-it | ✅ | GPU math verified |
| StarCoder2 | `"starcoder2"` | starcoder2-3b | ✅ | GPU math verified |
| GPT-2 | `"gpt2"` | gpt2 | ✅ | GPU math verified |
| Qwen2 | `"qwen2"` | qwen2-7b-instruct | pending remote GPU | >2 GB GGUF; needs ≥8 GB VRAM |
| Other | any | — | — | Loads if weight names follow `blk.N.*` convention |

**GPU math verified** = `quant_verify` confirmed GPU dequant matches CPU reference for all tensor types in the model.

**Fused QKV note (Phi-3/Phi-3.5):** These models store `blk.N.attn_qkv.weight` as a single fused tensor instead of separate `attn_q`, `attn_k`, `attn_v`. Airframe's current metadata loader expects the split form. Full Phi-3 inference support requires a one-pass QKV split during load — tracked on the roadmap.

All supported architectures share the GGUF weight tensor naming convention (`blk.N.*`). The architecture enum controls how hyperparameters are read from metadata — not the forward pass shader logic.

---

## Using a Different Model Today

If you have a GGUF that uses a supported architecture and any of the seven quantization types above,
it will load without any code changes.

```bash
# Any Llama-family Q4_0 GGUF
SHIMMY_BASE_GGUF=/path/to/Llama-3.2-1B-Instruct-Q4_0.gguf ./shimmy serve

# Any Q4_K_M GGUF (Q4_K_M means most layers are Q4_K, output head is Q6_K)
SHIMMY_BASE_GGUF=/path/to/Mistral-7B-Instruct-v0.2-Q4_K_M.gguf ./shimmy serve

# Extended context (if the model supports it)
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=4096 ./shimmy serve
```

Auto-discovery also finds GGUFs in `~/.cache/huggingface/`, `~/.ollama/models/`, and any path
listed in `SHIMMY_MODEL_PATHS`. Only TinyLlama 1.1B has been exhaustively validated against the
Airframe engine; others load on the documented quant/arch support.

**Selecting a model for chat:** Use the model name reported by `shimmy list` as the `model` field
in your OpenAI API request.

---

## Adding a New Quantization Type

The process is mechanical once the GGML spec for the quant type is known.

### Step 1: Add the type to the enum

In `airframe/src/core/ggml_types.rs`:

```rust
pub enum GgmlType {
    F32 = 0,
    Q4_0 = 2,
    Q4_K = 12,
    Q6_K = 14,
    Q8_0 = 8,   // ← add here
}
```

Add the `from_u32` match arm, the display name, and the `bytes_for_elements` calculation.

For **Q8_0** specifically:
- Block size: 32 elements
- Layout: 32 × `i8` values + 2-byte `f16` scale = 34 bytes per block
- Dequant: `w[i] = i8_val[i] * scale`  (simpler than Q4_0 — no nibble unpacking)

### Step 2: Write the dequant shader

Add a new file `airframe/src/core/dequant/q8_0.rs` following the pattern of `q4_0.rs`.
The shader reads the packed block, extracts the scale, and writes dequantized F32 values
to the output buffer.

### Step 3: Wire into the dispatch

In `airframe/src/core/dequant/mod.rs`, add a match arm for the new type so the loader
routes it to the correct shader.

### Step 4: Validate

Run the conformance harness against a fixture produced by the CPU reference implementation.
The acceptance criterion from the libshimmy expansion strategy:
- Q8_0: expected max logit error ~0.1 vs FP32 reference (near-FP16 quality)
- Q5_0: expected ~5-10% perplexity improvement over Q4_0

**Estimated effort per quant type:** 3–6 hours based on libshimmy's own engineering estimates.

### Priority order for next quants

1. **Q8_0** — simplest dequant (no nibble packing), best quality jump, wide model availability
2. **Q5_0** — 5-bit, midpoint between Q4_0 and Q8_0 quality
3. **Q5_K** — K-quant 5-bit superblock, needed for Q5_K_M GGUFs
4. **Q2_K** — aggressive compression, useful for very large models on limited VRAM

---

## Adding a New Model Architecture

The engine's forward pass is architecture-agnostic at the shader level — it executes matrix
multiplies against weights it finds by name. Adding a new architecture means teaching the engine
how to find those weights and what hyperparameters to read.

### Step 1: Add the variant

In `airframe/src/core/spec.rs`, add to `ModelArch` and its `From<&str>` match:

```rust
pub enum ModelArch {
    Llama,
    Mistral,
    Phi,
    Gemma,
    Qwen2,   // ← new
    Other(String),
}
```

### Step 2: Map the hyperparameter keys

Different architectures use different GGUF metadata key prefixes. In `spec.rs`'s `from_gguf()`
method, ensure the prefix is correctly extracted. Most Llama-family derivatives use identical
keys with their architecture name as the prefix (`qwen2.embedding_length`, etc.).

### Step 3: Map the weight tensor names

If the new architecture uses non-standard GGUF tensor names (not `blk.N.*`), update the
`layer_prefix()` method. Llama, Mistral, Phi, and Gemma all use `blk` so this is only needed
for unusual architectures.

### Step 4: Handle architecture-specific ops

Some architectures differ in their attention variant (sliding window in Mistral, QKNorm in
Gemma 2, etc.). These require additions to `airframe/src/family/`. For architectures that
are strict Llama-derivatives (Llama 3.x, Mistral, most Qwen2 models), the existing forward
pass handles them without modification.

---

## Scaling Up: The VRAM Math

The engine allocates VRAM at load time based on model weights + KV cache. Planning numbers:

| Model | Quant | Approx GGUF size | KV cache at 2048 ctx | Min VRAM |
|---|---|---|---|---|
| TinyLlama 1.1B | Q4_0 | ~638 MB | ~88 MB | ~800 MB |
| TinyLlama 1.1B | Q8_0 | ~1.1 GB | ~88 MB | ~1.3 GB |
| Llama 3.2 1B | Q4_0 | ~636 MB | ~128 MB | ~900 MB |
| Llama 3.2 3B | Q4_0 | ~1.9 GB | ~448 MB | ~2.5 GB |
| Mistral 7B | Q4_K_M | ~4.1 GB | ~512 MB | ~5 GB |
| Llama 3 8B | Q4_K_M | ~4.7 GB | ~512 MB | ~5.5 GB |

KV cache scales linearly with `SHIMMY_MAX_CTX`. At 4096 tokens it doubles; at 8192 it quadruples.
At 7B+ with 4096 context, 8 GB VRAM is the practical floor.

---

## The Recommended Expansion Sequence

Based on the libshimmy `EXPANSION_STRATEGY.md` and current engine state:

```
Phase 1 (same arch, new quants)     — ~1-2 weeks
  └─ Q8_0 dequant shader
  └─ Q5_0 dequant shader
  └─ Validate on TinyLlama Q8_0 GGUF

Phase 2 (same quants, larger models) — days, mostly testing
  └─ Llama 3.2 3B Q4_0 validation
  └─ Mistral 7B Q4_K_M validation
  └─ Update README validated model matrix

Phase 3 (quantization depth)        — ~2-3 weeks
  └─ Q5_K / Q5_K_M support
  └─ Q2_K support (enables very large models at high compression)

Phase 4 (architecture breadth)      — depends on arch
  └─ Qwen2 explicit support
  └─ DeepSeek-specific ops (MLA attention)
  └─ MoE dispatch (see docs/AIRFRAME_MOE_ROADMAP.md)
```

Each phase is independent. Phase 1 (Q8_0) is the highest leverage step — it unlocks better
quality on the same models without any architecture work, and the dequant code is simpler to
write than Q4_0 was.
