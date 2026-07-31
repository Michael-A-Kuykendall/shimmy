# TurboShimmy INT4 KV Cache

**One flag. ~7× less KV VRAM. Same output quality.**

TurboShimmy is Shimmy's on-GPU INT4 KV-cache compression system. It squeezes
the KV cache from 32-bit floats down to per-head-vector 4-bit integers —
entirely in WGSL compute shaders with no CPU roundtrips — delivering ~7× less
KV VRAM with no measurable quality loss at normal context lengths.

## Enable It

```bash
# Enable TurboShimmy on any GGUF model
./shimmy serve --kv-quant int4

# Or via environment variable (docker-compose, systemd, etc.)
SHIMMY_KV_QUANT=int4 ./shimmy serve

# Windows GPU + long prompts: reduce per-dispatch work to prevent TDR resets
./shimmy serve --kv-quant int4 --prefill-chunk 8
```

## Why It Matters

TurboShimmy changes what fits on your GPU:

| GPU VRAM | Without TurboShimmy | With TurboShimmy (`--kv-quant int4`) |
|---|---|---|
| 3 GB | Llama-3.2-1B only | **Llama-3.2-3B fits** |
| 4 GB | Llama-3.2-3B, ctx=2048 (tight) | **Llama-3.2-3B at ctx=8192** |
| 6 GB | 3B models, short context | **7B models with reasonable context** |

**VRAM comparison (Llama-3.2-3B, ctx=2048):**

| Mode | KV cache | Total VRAM | Min GPU needed |
|---|---|---|---|
| Default (f32) | ~512 MB | ~2.4 GB | 3 GB (tight) |
| TurboShimmy (int4) | **~72 MB** | **~2.0 GB** | **2.5 GB** |

**VRAM comparison (TinyLlama 1.1B, ctx=2048):**

| Mode | KV cache | Total VRAM |
|---|---|---|
| Default (f32) | 88 MB | ~700 MB |
| TurboShimmy (int4) | **~13 MB** | **~650 MB** |

## When to Use It

| Situation | Recommendation |
|---|---|
| 3B model on a 4 GB GPU | `--kv-quant int4` — enables models that wouldn't fit otherwise |
| 7B model at ctx=4096+ | `--kv-quant int4` — cuts KV from ~512 MB → ~72 MB |
| Short chat sessions (ctx ≤ 2048) | `--kv-quant int4` — safe, no quality tradeoff |
| Long-form generation (ctx > 8192) | Default `f32` — keep maximum quality |
| Windows GPU + TDR crashes on long prompts | `--kv-quant int4 --prefill-chunk 8` |

## How It Works

Each K/V head vector is independently quantized to 4-bit integers with a
per-vector F32 scale factor (`max_abs / 7.0`), packed as nibbles into U32s by
the `sh_kv_pack_int4.wgsl` WGSL compute shader. Dequantization happens
on-the-fly when computing attention scores via `sh_kv_unpack_int4.wgsl` — also
on GPU. The Airframe engine's helical context-shift operates directly on the
packed INT4 representation; no decompression needed. Zero CPU roundtrips at any
step.

## Quality Validation

Needle-in-a-haystack benchmarks on Llama-3.2-3B show **zero retrieval
degradation vs F32 at ctx≤2048** across all tested insertion depths (15%, 50%,
85%). Full benchmark data and setup guide:
[TurboShimmy on the wiki](https://github.com/Michael-A-Kuykendall/shimmy/wiki/TurboShimmy).

## Windows Stability

Airframe ships a `device.on_uncaptured_error` handler so GPU validation errors
surface as clean HTTP 500 responses instead of crashes. Use `--prefill-chunk 8`
to prevent Windows TDR resets during long prefills on older GPUs (GTX 10xx/16xx
series). TDR transport with GPU timestamp pools provides accurate dispatch
timing, fixing TDR watchdog crashes during long prefill sequences.

---

**Engine internals:** see [docs/GPU_PIPELINE.md](GPU_PIPELINE.md) for the
bindless GPU architecture and [docs/QUANTIZATION.md](QUANTIZATION.md) for
bit-level quant format details.
