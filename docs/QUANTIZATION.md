# GGUF Quantization Deep Dive

Everything you need to know about how Shimmy/Airframe represents, stores, and decodes quantized weights on the GPU.

---

## Why Quantization?

A transformer model's size is almost entirely its weight tensors. A 1B-parameter model in full F32 precision needs **4 GB** of VRAM (1B × 4 bytes). Quantization compresses weights by representing them with fewer bits per value, typically by grouping values into blocks and storing a shared scale factor.

The tradeoff: you trade a small amount of numerical accuracy for a large reduction in memory, which enables:
- Running bigger models on consumer hardware
- Fitting more of the model into L2/L3 GPU cache during compute
- Faster memory bandwidth — the bottleneck for GPU transformer inference

Airframe v2.0 supports **7 quantization formats** across two families: simple block quantization (Q4_0, Q8_0) and K-quant superblocks (Q4_K, Q5_K, Q6_K).

---

## Block Quantization: Q4_0 and Q8_0

These are the original GGML quantization formats. Simple, widely supported, and still the best choice for the smallest models.

### Q4_0 Block Layout (32 values per block)

```
[ scale: f16 (2 bytes) | q0 q1 q2 ... q15 (2 values/byte × 16 bytes = 32 values) ]
= 18 bytes per 32 values = 4.5 bits per weight
```

Each block stores one F16 scale value and 32 weights packed as 4-bit signed integers (range -8 to +7).

**Dequantization formula:**
```
value[i] = scale × (q[i] - 8)   // q[i] is in range 0..15 stored unsigned
```

The GPU shader for Q4_0 looks like this conceptually:
```wgsl
let scale = unpack_f16(block.scale);
let lo = (packed >> 0u) & 0xFu;  // lower nibble
let hi = (packed >> 4u) & 0xFu;  // upper nibble
let v0 = f32(i32(lo) - 8) * f32(scale);
let v1 = f32(i32(hi) - 8) * f32(scale);
```

**Q8_0** uses 8-bit signed integers instead of 4-bit, with one I8 scale per 32 values. More accurate but twice the size of Q4_0.

### When to use Q4_0 / Q8_0

| Format | Bits/weight | Compression vs F32 | Good for |
|---|---|---|---|
| Q4_0 | 4.5 | ~7x | Small models (TinyLlama, GPT-2); max VRAM savings |
| Q8_0 | 8.5 | ~3.7x | Slightly larger models where quality matters more |

---

## K-Quant Superblocks: Q4_K, Q5_K, Q6_K

K-quants are the modern GGUF quantization family, introduced by ggerganov to improve quality at the same bit width by using multi-level scaling (superblocks containing many sub-blocks).

### Q4_K Superblock Layout (256 values per block)

```
Superblock:
  [ super_scale: f32 (4 bytes)
    super_min: f32 (4 bytes)
    scales: u8[12] (6 bits per sub-block × 8 sub-blocks, packed = 12 bytes)
    mins:   u8[12] (same)
    qs:     u8[128] (4 bits × 256 values packed = 128 bytes) ]
= 144 bytes per 256 values = 4.5 bits per weight
```

The two-level scaling improves accuracy significantly: each 32-value sub-block has its own scale derived from the superblock scale, letting the quantization adapt to local weight distributions.

**Dequantization (simplified):**
```
super_d = super_scale / 64.0
sub_scale[j] = super_d × (scales[j] & 0x3F)
sub_min[j]   = super_d × (mins[j] & 0x3F)
value[i] = sub_scale[j] × qs[i] - sub_min[j]
```

### Q5_K Superblock Layout

Same superblock structure as Q4_K but adds a 5th bit per value stored in a separate high-bit plane:

```
[ same header as Q4_K ]
qh: u8[32] (high-bit plane for 256 values)
qs: u8[128] (low 4 bits)
= 176 bytes per 256 values = 5.5 bits per weight
```

The GPU shader reconstructs the full 5-bit value by OR-ing the high bit from `qh` onto the 4-bit `qs` value.

### Q6_K Superblock Layout

The highest-quality K-quant. Used almost exclusively for the output embedding layer (`output.weight`) and the final normalization layer in mixed-precision models.

```
[ ql: u8[128] (lower 4 bits of each value, 256 values)
  qh: u8[64]  (upper 2 bits, packed 4-per-byte)
  scales: i8[16] (one per 16-value sub-block) ]
= 210 bytes per 256 values = 6.5 bits per weight
```

---

## Mixed Precision (Q4_K_M, Q5_K_M)

GGUF models are rarely purely one quantization type. The `_M` ("medium") naming convention in HuggingFace filenames indicates a specific per-layer strategy:

| Layer type | Q4_K_M | Q5_K_M |
|---|---|---|
| Attention output + FFN down | Q4_K | Q5_K |
| Attention QKV + FFN gate/up | Q4_K | Q4_K |
| Output embedding | Q6_K | Q6_K |
| Embedding + normalization | F32 | F32 |

This is why Q4_K_M GGUFs contain all three quantization types (Q4_K, Q6_K, F32/F16) — Airframe handles the per-tensor type dispatch transparently.

---

## GPU Shader Architecture

Airframe uses a **bindless resource model**: all quantized weight tensors for a given layer are bound as a single large storage buffer. The WGSL compute shader receives the tensor offset and quantization type as push constants and dispatches the appropriate dequant path.

```
Compute dispatch per matrix multiply:
  workgroup_size = (16, 16, 1)
  dispatch = (ceil(N/16), ceil(M/16), 1)

Shader entry point:
  1. Load quantization type from push_constants
  2. Jump to appropriate dequant function (Q4_0, Q8_0, Q4_K, Q5_K, Q6_K, F32, F16)
  3. Dequantize row segment → shared memory
  4. Multiply by activations
  5. Write to output buffer
```

All dequantization happens in F32 — there are no mixed-precision accumulations. This is intentional: it gives deterministic, high-quality output at the cost of slightly more compute vs F16 accumulation.

---

## quant_verify: How GPU/CPU Agreement Is Tested

Every supported model undergoes `quant_verify` before being declared validated:

1. Load the GGUF model
2. For each quantization type present in the model:
   - Pick 512 consecutive values from a representative tensor
   - Dequantize with the CPU reference implementation (pure Rust, bit-exact)
   - Dequantize with the GPU WGSL shader
   - Compare all 512 values with tolerance `max_abs_err < 0.001` (for K-quants, ≤ 0.01)
3. Report `OK` or `MISMATCH` per type

If any type returns `MISMATCH`, the model is not safe to use — the GPU and CPU will produce different results for the same weights, which means generation will be incorrect and non-reproducible.

**Note:** `quant_verify` requires the model to fit in the WebGPU buffer allocation limit (~2 GB on most adapters). Models >2 GB cannot be verified this way and must be validated differently.

---

## Choosing a Quantization Format

| Use case | Recommended | Reason |
|---|---|---|
| Maximum VRAM efficiency on tiny models | Q4_0 | Smallest size, good quality for 1-3B |
| Best quality/size on 1-7B models | Q4_K_M | Multi-level scaling significantly improves PPL vs Q4_0 |
| Minimal quality loss, more VRAM | Q5_K_M or Q6_K | 5-6 bits approaches F16 quality |
| Debugging / reference | F32 | Bit-exact, maximum precision |

**Rule of thumb**: Start with Q4_K_M. Switch to Q5_K_M if you notice quality issues. Only use Q8_0/F32 for debugging.

---

## Quantization and Perplexity

Perplexity (PPL) is the standard accuracy metric for language models. Lower is better. As a rough guide on Llama-family models (numbers approximate — vary by model and dataset):

| Format | Bits/weight | PPL vs F16 | Size vs F16 |
|---|---|---|---|
| F32 | 32 | baseline | 2x |
| F16 | 16 | baseline | 1x |
| Q8_0 | 8.5 | +0.1% | 0.53x |
| Q6_K | 6.5 | +0.2% | 0.41x |
| Q5_K_M | 5.5 | +0.4% | 0.35x |
| Q4_K_M | 4.5 | +0.8% | 0.28x |
| Q4_0 | 4.5 | +1.5% | 0.28x |

The key insight: **Q4_K_M gives ~70% memory reduction for less than 1% quality loss.** Q4_0 gives the same memory but noticeably worse quality — K-quants are almost always the better choice at the same bit width.

---

## Known Unsupported Formats

| Format | Notes |
|---|---|
| Q2_K, Q3_K | Not yet implemented in Airframe shaders |
| IQ2, IQ3, IQ4 | i-quants (importance-matrix based) — not implemented |
| BF16 | `bfloat16` — not yet; requires wgpu extension |
| F64 | Not used in practice |

If a GGUF contains an unsupported quantization type, the Airframe runtime will fail at model load with a clear error indicating which tensor and type are unrecognized.

---

## Further Reading

- [Airframe Architecture](ARCHITECTURE.md) — how the full GPU pipeline uses dequantized weights
- [GPU Pipeline Internals](GPU_PIPELINE.md) — shader dispatch architecture
- [Model Expansion Guide](MODEL_EXPANSION.md) — adding new architectures
