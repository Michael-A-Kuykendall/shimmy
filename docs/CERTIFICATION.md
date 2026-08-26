# Certification Regimen

## How Shimmy Mathematically Proves Its Models

Every model Shimmy ships has passed a **5-gate certification pipeline** that proves the GPU inference path produces numerically correct output — not just "it generates something that looks right," but mathematically verified against a spec-derived reference. No other local LLM runner does this.

This document explains what we do, why we do it, and how the testing regime works.

---

## The Core Idea

GPU inference is a pipeline of dozens of operations (dequantization, attention, FFN, RMSNorm, RoPE, lm_head). A bug in any single operation produces silently wrong output — the model generates fluent text that is simply incorrect. The only way to catch these is to compare GPU output against an independent reference.

Our approach: **derive the exact expected output from the GGUF file itself** (the spec), then run the GPU and compare. If they match within tolerance, the model is certified. If they don't, we have a precise RED (Regression Error Document) that pinpoints the layer and operation.

---

## The Five Gates

### Gate 1 — Dequant (`quant_verify`)

GGUF models store weights in quantized formats (Q4_0, Q4_K_M, Q5_K_M, Q6_K, etc.). Before any computation, the GPU must dequantize these back to F32. The dequantization shader must produce results matching the mathematical definition in the GGUF spec.

**How:** `quant_verify` reads 512 elements per quantization type per model, computes the dequant on GPU, and compares against the CPU reference implementation derived from the spec formula (`airframe_observe::quant_formula`). Every quantization type must PASS.

**Spec reference:** `airframe_observe::quant_formula` (the GGUF/GGML spec registry). We never hand-roll the dequant math — we validate our shader against the spec element-by-element.

### Gate 2 — Structural Peel (layer-by-layer audit)

The transformer has N layers, each with Q, K, V, post-attention FFN, and final RMSNorm. The GPU must produce the correct output for every layer.

**How:** `stack_dump_gpu` captures per-layer residual and stage output (Q, K, V, post-attention, FFN, final_norm). The **PLAN** is derived from the GGUF metadata (n_layer, n_embd, head_dim, n_kv_head, qk_norm, rope_base). The **PEEL** is the GPU dump. A RED is any layer where:
- A required stage is missing
- The count doesn't match the plan
- Any value is NaN or non-finite
- Residual RMS is unexpected

### Gate 3 — Numerical (dual-peel self-consistency)

Even if each layer looks structurally correct, numerical drift can accumulate across layers.

**How:** Run `stack_dump_gpu` twice on the same model with the same prompt. Compare the two peel outputs layer-by-layer. Max delta must be ≤ 1e-2 for residuals and ≤ 1e-2 for final logits. Any deviation is a RED.

### Gate 4 — Decode≡Prefill Equivalence

A common class of bugs: the model produces correct output during prefill (short prompt) but collapses during decode (token-by-token generation). These bugs are invisible in short-prompt testing.

**How:** Run `decode_gate` which compares prefill output (full prompt processed at once) against decode output (token-by-token with KV cache). The decode logits must match the prefill logits for the same tokens. Max delta ≤ 1e-2, argmax must match.

### Gate 5 — Logits (final output against vault oracle)

The final output logits must be correct. This is the last line of defense.

**How:** The final logits from `generate_isf` are compared against a golden-vault oracle (a known-correct reference). The oracle was computed via the CPU reference path and validated element-by-element. The GPU logits must match within tolerance.

---

## The Testing Regime

### Predictive Property-Based Testing (PPT)

Our testing follows the [PPT Invariant Guide](https://github.com/Michael-A-Kuykendall/airframe/blob/main/docs/ppt-invariant-testing.md) (also in `~/Downloads/ppt_invariant_guide.md`). The system uses three layers:

| Layer | Description | Enforced With |
|---|---|---|
| **E-Test** | Exploration (temporary) | `explore_test()` or free tests |
| **P-Test** | Property test (generic input, stable behavior) | `property_test()` + invariants |
| **C-Test** | Contract (permanent, must-pass) | `contract_test()` + tracking |

### Invariant Cage (B1–B3)

Shimmy's PPT invariant cage verifies per-layer RMS/checksum against vault oracles for all populated models. The cage has three bonds:

- **B1** — LayerOutput/FinalLogits vs VaultOracle → CertPass/Fail
- **B2** — GGUF facts → TensorFact control-plane assertions
- **B3** — TensorFact → DispatchFact fabric rule (retires WGSL if/else dispatch ladder)

Run with: `cargo test --test test_invariants -- --test-threads=1`

### Contract Tests

Per-manifest contract tests verify that the runtime conforms to the model spec. Run with: `cargo test -p airframe --features isf --test test_contracts -- --test-threads=1`

Current: **9/9 passing**.

### End-to-End Testing

The certification pipeline itself is end-to-end: it runs the full GPU forward pass (prefill + decode) and compares against the CPU reference. This is the ultimate end-to-end test — it exercises every shader, every memory allocation, every dispatch.

### Property-Based Testing for Math

The `quant_verify` tool is inherently property-based: it tests 512 elements per quantization type per model, covering the full range of possible input values. The `decode_gate` is also property-based — it compares prefill vs decode for the same input, verifying that the mathematical operations are consistent regardless of execution path.

---

## The Certification Command

One command runs the full pipeline:

```bat
scripts\certify_math.bat <family-id> <path-to.gguf> ["multi-token prompt"]
```

Example:
```bat
scripts\certify_math.bat qwen3-4b-q4-k-m D:\models\Qwen3-4B-Q4_K_M.gguf "The capital of France is a beautiful"
```

This produces:
- `cert/packages/<family-id>/plan.json` — the derived plan (layer counts, dims, rope config)
- `cert/packages/<family-id>/peel.json` — the GPU stack dump
- `cert/packages/<family-id>/quant_verify.log` — dequant verification log
- `cert/packages/<family-id>/reds.json` — any REDs found (empty = pass)
- `cert/packages/<family-id>/REPORT.md` — human-readable pass/fail table
- `cert/packages/<family-id>/chat_smoke.log` — multi-prompt generation output

Exit code 0 = certified. Non-zero = REDs found.

---

## What "Certified" Means

A model is **certified** when:
1. All 5 gates pass (zero unwaived REDs)
2. The MATH box is green (reds.json has no failures)
3. The CHAT box is green (multi-prompt generate produces coherent output)

Certified models are tracked in the workspace ledger (`cert/math_ledger.duckdb`) with per-gate pass/fail columns. The README's Supported Models table is generated from this ledger via `scripts/cert/generate_models_table.py`.

---

## The Testing Hierarchy

```
┌─────────────────────────────────────────────────┐
│  C-Test: Contract Tests (permanent, must-pass)  │
│  cargo test -p airframe --features isf          │
│  --test test_contracts -- --test-threads=1      │
│  9/9 passing                                     │
├─────────────────────────────────────────────────┤
│  P-Test: Property-Based Tests                   │
│  quant_verify (512 elements × quant types)      │
│  decode_gate (prefill ≡ decode)                 │
│  dual-peel numerical (two runs ≤ 1e-2)         │
│  invariant cage B1-B3 (per-layer vs vault)      │
├─────────────────────────────────────────────────┤
│  E-Test: Exploration Tests                      │
│  stack_dump_gpu (layer-by-layer audit)          │
│  chat_smoke.log (multi-prompt coherence)        │
│  cert_reds_test.py (unit tests for judge)      │
└─────────────────────────────────────────────────┘
```

---

## Why This Matters

Most local LLM runners have no certification pipeline. They test that a model loads and generates text, then ship it. Silent numerical bugs — a wrong dequant nibble offset, a misaligned RoPE frequency, a KV cache layout mismatch — produce fluent but incorrect output that passes casual testing.

Our pipeline catches these. Every model in the Supported Models table has been verified at the mathematical level, not just the "it generates something" level.

---

## Current Certification Status

**11 model families · 25 certified model/quant combinations** (as of this release)

| Family | Models | Quants |
|---|---|---|
| Llama | TinyLlama-1.1B, Llama-3.2-1B, Llama-3.2-3B | Q4_0 · Q4_K_M · Q5_K_M · Q6_K |
| Qwen3 | 0.6B · 1.7B · 4B · 8B · 4B-Thinking | Q4_K_M |
| Qwen2 | 0.5B · 1.5B · 7B | Q4_K_M |
| Qwen3.5 | 9B | Q4_K_M |
| Phi-3 | 3.5-mini · 3-mini-4k | Q4_K_M · Q4_0 |
| Phi-2 | 2 | Q4_K_M |
| Gemma-2 | 2B · 9B | Q4_K_M |
| Gemma-4 | E4B · 12B-coder | Q4_K_M |
| DeepSeek-R1 | 0528-Qwen3-8B | Q4_K_M |
| Ministral | 3-14B-Reasoning | Q4_K_M |
| StarCoder2 | 3B | Q4_K_M |

See [docs/SUPPORTED_MODELS.md](SUPPORTED_MODELS.md) for the full list and [CHANGELOG.md](CHANGELOG.md) for what changed.