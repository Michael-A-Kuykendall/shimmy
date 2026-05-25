# Shimmy AI Agent Primer

Keep this file limited to current operating rules. Remove stale content aggressively.

## Deployment Model (Read This First)

```
shimmy-private/          ← THIS REPO — public-facing CLI/server product
                            airframe = { path = "../", optional = true }
../airframe/             ← PRIVATE sibling repo — GPU engine (NOT nested here)
```

- **Shimmy is the product.** Airframe is the private optional engine dependency.
- The `airframe` crate is OPTIONAL (`optional = true`). Default builds (`cargo build`) do not require it.
- `cargo build --features airframe` requires `../airframe` to exist (private repo, not cloned by default).
- **Cloning shimmy does NOT expose airframe source.** This is by design.
- Do NOT add `airframe` as a nested submodule inside this repo. It is a sibling path dep only.

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

## Repository Push Policy

- This repo's submodule checkout has **ONE** remote: `private` → `https://github.com/Michael-A-Kuykendall/shimmy-private.git`
- **NO `origin` remote exists in the submodule context. Do not add one.**
- The standalone checkout at `C:/Users/micha/repos/shimmy-private` has `origin` → same URL.
- Push only when explicitly requested: `git push private <branch>` (submodule) or `git push origin <branch>` (standalone).
- Do not push to public shimmy without explicit user approval.

## CI Wire — AIRFRAME_ACCESS_TOKEN

Confirmed wired in `.github/workflows/release.yml` and `ci.yml`:

```yaml
- name: Checkout Airframe engine (private path dep)
  run: git clone https://x-access-token:${{ secrets.AIRFRAME_ACCESS_TOKEN }}@github.com/Michael-A-Kuykendall/airframe.git ../airframe
```

`AIRFRAME_ACCESS_TOKEN` must be set in GitHub Secrets. **Do not print or expose this token.**

## Test Failures

**Zero tolerance. No exceptions.**

`cargo test` must finish with 0 failures before any task is considered done.
Do not declare work complete while any test is red.

## Architecture (v2.0)

- Engine: wgpu/WebGPU WGSL pipeline via Airframe (private). Replaces llama.cpp entirely.
- Server: OpenAI-compatible (`/v1/chat/completions`, `/v1/completions`), Ollama-compat (`/api/generate`, `/api/tags`).
- No Python dependencies in the default build path.
- WGSL quant coverage: F32, F16, Q4_0, Q8_0, Q4_K(M/S), Q5_K(M/S), Q6_K.

## What NOT To Do

- Do NOT add an `airframe/` submodule inside this repo (was done by a previous AI session — removed).
- Do NOT add an `origin` remote to the submodule checkout.
- Do NOT mix console (`crates/console/`) or vision feature work into runtime release changes.
- Do NOT push without explicit user request.
