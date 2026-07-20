# Agent Instructions — Shimmy

## Repository Architecture

```
shimmy-private/              ← THIS REPO — public-facing CLI/server product (private working copy)
public remote: shimmy.git    ← https://github.com/Michael-A-Kuykendall/shimmy.git
airframe = { version = "0.2" }  ← PUBLIC crates.io dep — Airframe is Shimmy's GPU engine library
```

- **Shimmy is the product. Airframe is Shimmy's GPU engine library.**
  All user-facing value ships through Shimmy. Airframe is a Rust library dependency
  of Shimmy and has no binary, CLI, or server. Both are MIT-licensed and public.
- `cargo build` (default features) compiles the full GPU engine — airframe is downloaded from crates.io.

## Repository Push Policy

- Two remotes exist:
  - `origin` → `https://github.com/Michael-A-Kuykendall/shimmy-private.git` (private working copy)
  - `public` → `https://github.com/Michael-A-Kuykendall/shimmy.git` (public GitHub repo users see)
- In the submodule context (`shimmy_integration/` inside airframe workspace): push with `git push private <branch>`.
- In the standalone context (`C:/Users/micha/repos/shimmy-private`): push with `git push origin <branch>` (private) or `git push public <branch>` (public).
- Do not push unless explicitly requested by the user.
- To publish to the public shimmy repo, push to the `public` remote.

## Test Failures

**Zero tolerance. No exceptions.**

`cargo test` must finish with 0 failures before any task is considered done.
There is no such thing as a "pre-existing" failure. Fix it before moving on.

## Architecture (v2.0)

- **Engine**: wgpu/WebGPU WGSL pipeline via Airframe (crates.io: `airframe = "0.1"`). Replaces llama.cpp entirely.
- **Server**: OpenAI-compatible (`/v1/chat/completions`, `/v1/completions`), Ollama-compat (`/api/generate`, `/api/tags`), LM Studio discovery.
- **No Python in default path.** Default build is `airframe` + `huggingface` features.
- **WGSL quant coverage**: F32, F16, Q4_0, Q8_0, Q4_K(M/S), Q5_K(M/S), Q6_K.
- **wgpu 2 GB buffer cap**: Known limit for models with tensors >2 GB. Deferred to v2.1.

## Feature Flags

```toml
default = ["airframe", "huggingface"]  # Full GPU build; use --no-default-features --features huggingface for CPU-only
airframe = ["dep:airframe"]            # Airframe native GPU engine (from crates.io)
gpu = ["airframe", "huggingface"]      # GPU-optimized build
full = ["airframe", "huggingface", "mlx"]
fast / coverage = ["huggingface"]      # CI-safe, no GPU hardware required
# Deprecated stubs (llama.cpp removed in v2.0):
llama = []  llama-cuda = []  llama-vulkan = []  llama-opencl = []
```

## Scope Control

- Console (`crates/console/`) is scaffolded but unimplemented. Keep isolated from runtime release changes.
- Vision work is deferred. Keep on dedicated branches.
- Launch scope is architecture/runtime path only.

## What NOT To Do

- Do NOT add an `airframe/` submodule inside this repo.
- Do NOT use a path dep for airframe — it is on crates.io as `airframe = { version = "0.1", optional = true }`.
- Do NOT push without explicit user request.
- Do NOT mix vision or console feature work into launch-critical runtime changes.
