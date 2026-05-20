# Airframe MoE (Mixture of Experts) — Engineering Roadmap

**Status**: Planned  
**Estimate**: 21 story points (Fibonacci)  
**Prerequisite**: Airframe v1.x stable (dense transformer fully validated)  
**Tracking**: See [ROADMAP.md](../ROADMAP.md) for priority placement

---

## Why MoE Matters

Mixture of Experts (MoE) is the architecture behind the highest-performing open-weight models at the
consumer hardware boundary: **Mixtral 8x7B**, **Mixtral 8x22B**, **DeepSeek-MoE**, and the DeepSeek
v2/v3/R1 model family. Without MoE support in the Airframe engine, Shimmy users who want to run these
models must fall back to `--legacy` (llama.cpp), which defeats the purpose of the GPU pipeline.

The `--legacy` path will remain forever, but the right long-term answer is native Airframe MoE so the
full WebGPU pipeline handles routing, expert dispatch, and re-aggregation on the GPU.

---

## Current State of Airframe

Airframe is a **dense transformer only**. The inference pipeline assumes one monolithic feedforward
network per layer with three weight matrices:

```
ffn_gate  (n_embed × ff_dim)
ffn_up    (n_embed × ff_dim)
ffn_down  (ff_dim  × n_embed)
```

`ModelSpec` carries a single `ff_dim: u32` scalar. The pipeline allocates one set of intermediate
buffers per layer. There is zero routing infrastructure — no expert count, no router weight, no
top-K selection, no per-expert tensor loading.

Loading a Mixtral GGUF today would silently use only the first expert's tensors and produce garbage.

---

## What Needs to Be Built

### 1. GGUF Loader Changes (3 pts)

MoE GGUFs encode `llm.expert_count` and `llm.expert_used_count` in their metadata. Each expert has
its own weight triple: `blk.{L}.ffn_gate_exps`, `blk.{L}.ffn_up_exps`, `blk.{L}.ffn_down_exps`
(shape: `[n_experts, dim_out, dim_in]`).

Changes needed:
- Read `llm.expert_count` (N) and `llm.expert_used_count` (K) from metadata
- Load all N × 3 expert weight tensors per layer instead of 1 × 3
- Extend `ModelSpec` to carry `n_experts: Option<u32>` and `n_experts_used: Option<u32>`

### 2. Router Weight Tensors (2 pts)

MoE layers have a router: `blk.{L}.ffn_gate_inp` — shape `[n_experts, n_embed]`, the linear
projection that scores each token against each expert.

Changes needed:
- Load `ffn_gate_inp` alongside the expert weight tensors
- Allocate a GPU buffer per layer for the router weights

### 3. WGSL Router Shader (5 pts)

The router runs a linear projection over `[n_embed]` input → `[n_experts]` logits, then softmax.

```wgsl
// Rough pseudocode
let router_logits = matmul(hidden_state, router_weights); // [n_experts]
let router_probs  = softmax(router_logits);
```

This is new territory — the current shader set has no dynamic routing. The shader must:
- Accept a variable `n_experts` via push constant or uniform
- Produce `n_experts` logit values per token
- Apply softmax

### 4. Top-K Expert Selection (3 pts)

After softmax, select the top-K expert indices and their normalized weights (sum-to-1 renorm after top-K).

Options:
- Compute top-K in a WGSL shader (hard — no sort primitive; requires bitonic sort or partial sort)
- Compute top-K on the CPU side (simple — readback the `n_experts` floats, cheap for K≤8)

**Recommendation**: CPU top-K for the first implementation. K is typically 2–8; readback of
`n_experts * 4` bytes per token is acceptable.

### 5. Per-Expert Dispatch (5 pts)

Once top-K experts are selected, execute each chosen expert's feedforward pass and accumulate
weighted outputs.

This is the hard part. Options:
- **Sequential dispatch** (simplest): loop over K chosen experts, dispatch gate/up/down shaders
  with the selected expert's weight buffer. No sparse dispatch needed.
- **Batched sparse dispatch** (optimal): pack all K dispatches into a single wgpu submission.

**Recommendation**: Sequential dispatch for first implementation. For K=2 and typical layer
counts (32–64 layers), this adds 2× the ffn dispatch count — negligible on GPU.

### 6. Expert Combine (2 pts)

After dispatching all K experts, combine their outputs weighted by router probabilities:

```
output = Σ (weight_k * expert_k_output)
```

Requires a WGSL accumulation shader or an in-place weighted-add pass.

### 7. Buffer Management (3 pts)

For N experts per layer, buffer allocation changes significantly:
- Currently: 1 set of `{ffn_gate, ffn_up, ffn_down}` buffers per layer
- Required: N sets of weight buffers per layer

Mixtral 8x7B has 32 layers × 8 experts × 3 weights = 768 weight tensors (vs. 96 for dense).
Memory footprint and buffer allocation logic in `GpuRuntime::load()` will need rework.

---

## Fibonacci Estimate Breakdown

| Work Item | Points |
|-----------|--------|
| GGUF loader: metadata + N×3 tensor loading | 3 |
| Router weight tensor loading | 2 |
| WGSL router shader (linear + softmax) | 5 |
| Top-K selection (CPU path) | 3 |
| Per-expert sequential dispatch loop | 5 |
| Expert output combine shader | 2 |
| Buffer management for N-expert sets | 3 |
| **Total** | **21** |

---

## MoE Models That Would Benefit

| Model | N Experts | K Used | Parameters |
|-------|-----------|--------|------------|
| Mixtral 8x7B | 8 | 2 | ~12B active / 47B total |
| Mixtral 8x22B | 8 | 2 | ~39B active / 141B total |
| DeepSeek-MoE | 64 | 6 | varies |
| DeepSeek-V2 | 160 | 6 | ~21B active / 236B total |
| Qwen MoE | 60 | 4 | ~14B active / 57B total |

All of these ship in GGUF format and are already runnable via `--legacy`. Airframe MoE would bring
them onto the WebGPU pipeline — no CUDA required, cross-platform.

---

## Not in Scope for v1 MoE

- Shared expert routing (DeepSeek-V2 "shared" + "routed" expert split)
- Expert parallelism across multiple GPUs
- Dynamic expert loading (streaming weights from disk)
- Fine-grained quantization of MoE weight tensors (Q4_K_M etc. — depends on Airframe quantization
  support landing first)

---

## Recommended Sequencing

1. Dense transformer hardening + extended context validation (current — v2.x)
2. Quantization support in Airframe (Q4_K_M, Q8_0 inference) — unlocks smaller dense models
3. MoE v1 (this document) — sequential dispatch, CPU top-K
4. MoE v2 — batched dispatch, GPU top-K, DeepSeek shared-expert variant

---

*Generated from engineering session on 2026-05-20. See CHANGELOG [2.0.0] for release context.*
