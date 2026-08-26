# Shimmy v2.x Roadmap

**Last Updated:** 2026-08-26  
**Branch:** `main`  
**Current version:** 2.5.0 (publishing toward 2.6.0)

---

## Philosophy

Shimmy is a **shim** — thin, fast, and in the middle. It presents an OpenAI-compatible
API surface and routes to the best available inference backend. The product promise:
users point their AI tools at shimmy and they just work. Locally, privately, free.

Shimmy runs on **Airframe** (v0.4.0), a pure-Rust WebGPU (WGSL) transformer engine.
llama.cpp, MLX, HuggingFace, and RustChain backends have all been removed — Shimmy
is now a pure Airframe product. 26 models certified across 12 families.

---

## Current Release: v2.6.0 (in progress)

### 🏆 Certified — 12 Families · 26 Model/Quant Combinations

Shimmy's certification pipeline uses a **3-box regimen** (MATH + INFERENCE +
DETERMINISM), matching the Airframe 0.4.0 release. Every certified model passes
GPU dequant audit, structural peel, plan-vs-peel reds judgment, numerical
self-consistency, a 5-prompt inference battery, and deterministic output verification.

| Family | Models | Quants |
|--------|--------|--------|
| **Llama** | Llama-3.2-1B, Llama-3.2-3B, Llama-3.1-8B, TinyLlama-1.1B | Q4_0, Q4_K_M, Q5_K_M, Q6_K |
| **Qwen3** | 0.6B, 1.7B, 4B, 4B-Thinking, 8B | Q4_K_M |
| **Qwen2** | 0.5B, 1.5B, 7B | Q4_K_M |
| **Qwen3.5** | 9B | Q4_K_M |
| **Phi** | Phi-3.5-mini, Phi-3-mini-4k, Phi-2 | Q4_0, Q4_K_M |
| **Gemma-2** | 2B | Q4_K_M |
| **Gemma-4** | E4B, 12B-coder | Q4_K_M |
| **DeepSeek-R1** | 0528-Qwen3-8B | Q4_K_M |
| **Ministral** | 3-14B-Reasoning | Q4_K_M |
| **StarCoder2** | 3B | Q4_K_M |

---

## Roadmap Items

### 🟠 P1 — Next

**Gemma-2-9B certification (tensor-scatter fix)**  
*Status:* Blocked on Airframe `plan_layer_half_windows` GPU dispatch wiring (the `dgd` epic)  
*Work:* Wire half-window dispatch into the GPU pipeline so models whose per-layer
tensor span exceeds `BLOB_BINDING_SLOTS=8` can be certified.  
*Blocks:* Full Gemma-2 family certification  
*Points:* 8

---

### 🟡 P2 — Soon

**HuggingFace Hub model sourcing (pure Rust)**  
*Status:* Not started  
*Motivation:* The shim should accept HF model IDs, not just local paths.
Users think in `microsoft/phi-4` not `/path/to/phi-4.Q4_K_M.gguf`.  
*Design:*
```
shimmy serve --model hf://microsoft/phi-4-gguf
  → hits HF Hub API (reqwest, no Python)
  → resolves to GGUF download URL
  → downloads to ~/.cache/shimmy/
  → loads into airframe
  → serves at /v1/chat/completions
```
*Features:*
- `GET /api/models/search?q=phi-4` — search HF Hub
- Auto-select quantization based on available VRAM
- Resume interrupted downloads
- `--hf-token` flag for gated models

*Dependencies:* `reqwest` (already in console crate), no new C++ deps  
*Points:* 5

---

### 🟡 P2 — Soon

**Console (local AI development platform)**  
*Status:* Commands wired, tool loop implemented, blocked on inference stability  
*Work remaining:*
- End-to-end test once airframe inference is stable across all certified models
- Session persistence (shimmy-session-store)
- Workspace context injection (file tree, git log)

---

### 🟢 P3 — Documentation Sprint

**Full docs alignment for v2.6.0**  
*Scope:*
- CHANGELOG.md — 2.6.0 entry with full feature list
- README.md — model table, version references, pipeline description
- SUPPORTED_MODELS.md — 26-model matrix, 3-box regimen
- Chinese user manuals updated
- Remove all llama.cpp installation instructions
- Update feature flags table

*Points:* 8

---

## Archive Reference

| Tag | Commit | Contents |
|-----|--------|----------|
| `archive/llama-cpp-era-v1.9.0` | `6fe98ea` | Last shimmy with llama.cpp as default |
| `archive/pre-v2.0.0-history` | `6fe98ea` | Full pre-v2 history |

Users on the historical llama.cpp path: check out `archive/llama-cpp-era-v1.9.0`.

---

## Build Matrix (Current)

| Command | What you get |
|---------|-------------|
| `cargo build` | Full GPU engine (Airframe + HuggingFace, default features) |
| `cargo test` | Run full test suite |
| `cargo build --release` | Optimized release binary |

See [CHANGELOG.md](CHANGELOG.md) for full version history.
