# Agent Instructions — Shimmy (shimmy-private)

## Two-Repo Architecture and Deployment Model

```
shimmy-private/              ← THIS REPO — public-facing CLI/server product
                                airframe = { path = "../", optional = true }
../airframe/                 ← PRIVATE sibling repo — GPU engine (NOT nested here)
```

**Shimmy is the product. Airframe is the private engine.**

- `shimmy-private` (`https://github.com/Michael-A-Kuykendall/shimmy-private.git`) is the public-facing CLI/server product.
- `airframe` (`https://github.com/Michael-A-Kuykendall/airframe.git`) is the private GPU engine. It is an OPTIONAL Cargo dependency.
- Production/CI: CI clones `../airframe` via `AIRFRAME_ACCESS_TOKEN` before running `cargo build --features airframe`.
- `cargo build` (default, no flags) works for anyone cloning shimmy — airframe never required.
- **Cloning shimmy does NOT expose airframe source.** The path dep fails silently if `../airframe` is absent.
- Do NOT add airframe as a nested submodule inside this repo. It is a sibling path dep only.

## Repository Push Policy

- **Submodule context** (`shimmy_integration/` inside airframe workspace): ONE remote `private` → `https://github.com/Michael-A-Kuykendall/shimmy-private.git`. NO `origin` remote. Use `git push private <branch>`.
- **Standalone context** (`C:/Users/micha/repos/shimmy-private`): remote `origin` → same URL. Use `git push origin <branch>`.
- Do not push unless explicitly requested by the user.
- Do not push to any public shimmy repo without explicit user approval.

## Test Failures

**Zero tolerance. No exceptions.**

`cargo test` must finish with 0 failures before any task is considered done.
There is no such thing as a "pre-existing" failure. Fix it before moving on.

## Architecture (v2.0)

- **Engine**: wgpu/WebGPU WGSL pipeline via Airframe (private). Replaces llama.cpp entirely.
- **Server**: OpenAI-compatible (`/v1/chat/completions`, `/v1/completions`), Ollama-compat (`/api/generate`, `/api/tags`), LM Studio discovery.
- **No Python in default path.** Default build is `huggingface` feature (Python bridge for HuggingFace models).
- **WGSL quant coverage**: F32, F16, Q4_0, Q8_0, Q4_K(M/S), Q5_K(M/S), Q6_K.
- **wgpu 2 GB buffer cap**: Known limit for models with tensors >2 GB. Deferred to v2.1.

## Feature Flags

```toml
default = ["huggingface"]        # crates.io-safe — no airframe required
airframe = ["dep:airframe"]      # optional; requires ../airframe path dep (private)
gpu = ["airframe", "huggingface"]
full = ["airframe", "huggingface", "mlx"]
fast / coverage = ["huggingface"]  # CI-safe, no path deps
# Deprecated stubs (llama.cpp removed in v2.0):
llama = []  llama-cuda = []  llama-vulkan = []  llama-opencl = []
```

## CI Wire

Confirmed wired in `.github/workflows/release.yml` and `ci.yml`:

```yaml
- name: Checkout Airframe engine (private path dep)
  run: git clone https://x-access-token:${{ secrets.AIRFRAME_ACCESS_TOKEN }}@github.com/Michael-A-Kuykendall/airframe.git ../airframe
```

`AIRFRAME_ACCESS_TOKEN` must be set in the repo's GitHub Secrets. Do not print or expose this token.

## Current Branch State (as of 2026-05-25)

- `main` @ `e70ed39` — HEAD, 4 commits ahead of `private/main` (unpushed)
- `private/main` @ `961cbf8` — last pushed state
- The 4 unpushed commits cover: `ci.yml`, `release.yml`, `CHANGELOG.md`, `README.md`, docs (ARCHITECTURE, CHAT_TEMPLATES, zh-CN/zh-TW docs), `src/discovery.rs`, `src/engine/adapter.rs`, `src/engine/airframe.rs`, `src/openai_compat/`, `src/server.rs`.

## Scope Control

- Console (`crates/console/`) is scaffolded but unimplemented. Keep isolated from runtime release changes.
- Vision work is deferred. Keep on dedicated branches.
- Launch scope is architecture/runtime path only.

## What NOT To Do

- Do NOT add an `airframe/` submodule inside this repo (was done by a previous AI session — removed).
- Do NOT add an `origin` remote to the submodule checkout context.
- Do NOT push without explicit user request.
- Do NOT mix vision or console feature work into launch-critical runtime changes.
