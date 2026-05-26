# Shimmy AI Agent Primer

Keep this file limited to current operating rules. Remove stale content aggressively.

## Deployment Model (Read This First)

```
shimmy-private/              ← THIS REPO — public-facing CLI/server product (private working copy)
public remote: shimmy.git    ← https://github.com/Michael-A-Kuykendall/shimmy.git
airframe = { version = "0.1" }  ← PUBLIC crates.io dependency — no local path dep
```

- **Shimmy is the product.** Airframe is the GPU engine published on crates.io as `airframe = "0.1"`.
- `airframe` is a crates.io dependency — no private path dep, no AIRFRAME_ACCESS_TOKEN, no cloning required.
- `cargo build` (default, all features) works for anyone — airframe downloads from crates.io.
- No submodules. No secrets needed for standard builds.

## Feature Flags

```toml
default = ["airframe", "huggingface"]  # Full GPU build
airframe = ["dep:airframe"]            # Airframe native GPU engine (crates.io: airframe = "0.1")
gpu = ["airframe", "huggingface"]      # GPU-optimized build via Airframe
full = ["airframe", "huggingface", "mlx"]
fast / coverage = ["huggingface"]      # CI-safe, no GPU hardware required
# Deprecated stubs (llama.cpp removed in v2.0):
llama = []  llama-cuda = []  llama-vulkan = []  llama-opencl = []
```

## Repository Push Policy

- This repo has two remotes:
  - `origin` → `https://github.com/Michael-A-Kuykendall/shimmy-private.git` (private working repo)
  - `public` → `https://github.com/Michael-A-Kuykendall/shimmy.git` (public GitHub repo)
- Push to `public` when releasing or making docs/CI changes visible to users.
- Push only when explicitly requested by the user.

## Test Failures

**Zero tolerance. No exceptions.**

`cargo test` must finish with 0 failures before any task is considered done.
Do not declare work complete while any test is red.

## Architecture (v2.0)

- Engine: wgpu/WebGPU WGSL pipeline via Airframe (crates.io: `airframe = "0.1"`). Replaces llama.cpp entirely.
- Server: OpenAI-compatible (`/v1/chat/completions`, `/v1/completions`), Ollama-compat (`/api/generate`, `/api/tags`).
- No Python dependencies in the default build path.
- WGSL quant coverage: F32, F16, Q4_0, Q8_0, Q4_K(M/S), Q5_K(M/S), Q6_K.

## What NOT To Do

- Do NOT add an `airframe/` submodule inside this repo.
- Do NOT add a local path dep to airframe — use `airframe = { version = "0.1" }` from crates.io.
- Do NOT mix console (`crates/console/`) or vision feature work into runtime release changes.
- Do NOT push without explicit user request.
