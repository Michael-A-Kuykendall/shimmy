# Shimmy v2.x Roadmap

**Last Updated:** 2026-06-07  
**Branch:** `release/v2.2-cleanup`  
**Current version:** 2.1.0 → 2.2.0

---

## Philosophy

Shimmy is a **shim** — thin, fast, and in the middle. It presents an OpenAI-compatible
API surface and routes to the best available inference backend. The product promise:
users point their AI tools at shimmy and they just work. Locally, privately, free.

The v2 engine is **airframe** (WebGPU, pure Rust). llama.cpp served its purpose and is
historically parked at `archive/llama-cpp-era-v1.9.0`.

---

## Roadmap Items

### 🟢 P0 — Done

**Wire airframe as default inference engine**  
*Status:* ✅ Complete — `InferenceEngineAdapter` wired to airframe GGUF loading and token generation. Server starts, models load, and generation works.  
*Points:* 5

---

### 🟢 P1 — Done

**Strip llama.cpp + HuggingFace Python bridge**  
*Status:* ✅ Complete — dead engine files removed, `adapter.rs` cleaned to airframe-only path, `main.rs` stripped of MoE config and llama diagnostics. `cargo check` passes 0 warnings, 0 errors.  
*Work completed (8 points total):*
- Removed `shimmy-llama-cpp-2` dep and all `#[cfg(feature="llama")]` blocks (5pt)
- Removed HuggingFace Python subprocess bridge (3pt)
- Removed `src/engine/llama.rs`, `universal.rs`, `huggingface.rs`
- Cleaned `adapter.rs` down to airframe-only path
- Cleaned `main.rs` of MoE config, GPU backend selection, llama diagnostics
- Updated CHANGELOG, README, wiki to reflect v2 engine

*Note:* Users who need llama.cpp have `archive/llama-cpp-era-v1.9.0` on origin.

---

### 🟡 P2 — Soon

**SafeTensors native inference**  
*Status:* Not started — `safetensors_native.rs` module exists with a stub `generate()` method  
*Work:* Replace stub `generate()` with real inference via airframe. This is the inference path for `.safetensors` files (HF format, no GGUF conversion required).  
*Design:*
- `safetensors_native.rs` stub wired into `InferenceEngineAdapter` routing
- `generate()` delegates to airframe with SafeTensors tensor layout
- Adds `.safetensors` to the model discovery file filter
*Points:* 5

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
*Status:* Commands wired, tool loop implemented, blocked on P0 (inference)*  
*Work remaining:*
- End-to-end test once airframe inference is working
- Session persistence (shimmy-session-store)
- Workspace context injection (file tree, git log)

---

### 🟢 P3 — Documentation Sprint

**Full docs update (wiki + READMEs + Chinese translations)**  
*Scope:*
- `shimmy-wiki-content/` — all EN pages updated for v2 engine
- `shimmy-wiki-content/*-zh-CN.md` — Simplified Chinese updated
- `shimmy-wiki-content/*-zh-TW.md` — Traditional Chinese updated
- `docs/zh-CN/README.md` + `docs/zh-TW/README.md` — docs center updated
- `docs/USER_MANUAL.zh-CN.md` + `docs/USER_MANUAL.zh-TW.md` — user manuals updated
- Remove all llama.cpp installation instructions
- Update feature flags table
- Add airframe engine documentation
- Update Quick Start, Installation, Configuration pages

*Points:* 8 (including Chinese translations)  
*Note:* This is separate from the code strip work. Total combined = 8+8 = 13.

---

## Archive Reference

| Tag | Commit | Contents |
|-----|--------|----------|
| `archive/llama-cpp-era-v1.9.0` | `6fe98ea` | Last shimmy with llama.cpp as default |
| `archive/pre-v2.0.0-history` | `6fe98ea` | Full pre-v2 history |

Users on the historical llama.cpp path: check out `archive/llama-cpp-era-v1.9.0`.

---

## Build Matrix (Current State)

| Command | What you get |
|---------|-------------|
| `cargo build` | shimmy with Airframe GPU engine (default features: `airframe`) |
| `cargo build --no-default-features` | CPU-only build (no Airframe GPU) |
| `cargo build --features console` | shimmy + local AI dev console |
| `cargo build --features full` | shimmy + Airframe + MLX (Apple Silicon) |
