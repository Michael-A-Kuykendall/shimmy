# Release Gates & Regression Tests — v2.2

**Date:** June 30, 2026  
**Branch:** `main`  
**Engine:** Airframe (wgpu/WebGPU — no CUDA, no llama.cpp)  
**Status:** Active

---

## Release Gate Pipeline

All 7 gates defined in `.github/workflows/release.yml` under the `preflight` job.
Every gate must pass or the entire release stops.

| Gate | Name | Covers |
|------|------|--------|
| 1/7 | Core Build | `cargo build --features airframe,huggingface` — GPU engine via submodule |
| ~~2~~| ~~CUDA Timeout~~ | **Removed v2.0** — Airframe uses wgpu; no CUDA dependency |
| 3/7 | Template Packaging | Docker templates included in crates.io package (Issue #60 regression) |
| 4/7 | Binary Size | Constitutional 20 MB limit on `shimmy` binary |
| 5/7 | Test Suite | `cargo test --lib --no-default-features --features huggingface` |
| 5.1/7 | Airframe Compile Check | `cargo check --features airframe` — integration builds without errors |
| 5.5/7 | Regression Tests | `cargo test --test core --test handlers --test compile_checks` |
| 6/7 | Documentation | `cargo doc --no-deps --features huggingface` |
| 7/7 | crates.io Dry-Run | `cargo publish --dry-run --features huggingface` validates crates.io package |

---

## Regression Test Files (v2.2+)

### Active Test Files
| File | Status | Tests |
|------|--------|-------|
| `tests/core.rs` | ✅ Active — CLI, registry, templates, serde, SSE, discovery | 28 |
| `tests/handlers.rs` | ✅ Active — HTTP endpoints (health, models, chat, tags, concurrency) | 5 |
| `tests/compile_checks.rs` | ✅ Active — template file inclusion via include_str! | 1 |

### Retired Test Files
The following files were consumed into the consolidated suite above:

| File | Absorbed By |
|------|-------------|
| `tests/regression_tests.rs` | `tests/core.rs` + `tests/handlers.rs` |
| `tests/release_gate_integration.rs` | `cargo test --lib` (CI already passes) |
| `tests/template_compilation_regression_test.rs` | `tests/compile_checks.rs` |
| `tests/compilation_regression_test.rs` | `tests/compile_checks.rs` + `cargo check` |
| `tests/apple_silicon_detection_test.rs` | — (MLX is a stub) |
| `tests/integration_tests.rs` | `tests/handlers.rs` |
| `tests/cli_integration_tests.rs` | `tests/core.rs` |
| `tests/gpu_backend_tests.rs` | — (Airframe handles GPU) |
| `tests/anthropic_api_integration_test.rs` | — (Anthropic layer removed) |
| `tests/api_error_handling_test.rs` | `tests/core.rs` + `tests/handlers.rs` |
| `tests/openai_api_real_tests.rs` | `tests/core.rs` + `tests/handlers.rs` |
| `tests/safetensors_integration.rs` | `tests/core.rs` |
| `tests/workflow_tests.rs` | — (dead_code module) |
| `tests/regression/` (all 7 files) | `tests/core.rs` |

---

## Test Execution Commands

```bash
# Gate 5 — Core lib tests
cargo test --lib --no-default-features --features huggingface -- --test-threads=1

# Gate 5.1 — Airframe compile check
cargo check --features airframe

# Gate 5.5 — Regression tests
cargo test --test core --test handlers --test compile_checks --features airframe,huggingface

# Full regression pass
cargo test --features airframe,huggingface
```

---

## Pre-Release Checklist

### Code Quality
- [ ] `cargo fmt -- --check`
- [ ] `cargo clippy --no-default-features --features huggingface` (no warnings)
- [ ] `cargo deny check` (license check)
- [ ] No compilation warnings on `--features airframe,huggingface`

### Documentation
- [ ] `CHANGELOG.md` updated with release version entry
- [ ] `README.md` reflects current feature set
- [ ] New features documented

### Version Management
- [ ] `Cargo.toml` version bumped
- [ ] Git tag created (format: `vX.Y.Z`)
- [ ] Release notes prepared

### Workflow Validation
- [ ] `release.yml` `preflight` job passes on a test branch
- [ ] `ci.yml` `test` job passes on current branch

---

## Feature Flag Reference

| Feature | Purpose | crates.io safe |
|---------|---------|----------------|
| `huggingface` | Model downloading, tokenizer — **default** | ✅ Yes |
| `airframe` | Airframe GPU engine via submodule path dep | ❌ No (path dep) |
| `full` | `huggingface + airframe` convenience alias | ❌ No |
| `apple` | Apple Silicon convenience alias | platform-only |

**crates.io publish** uses `--no-default-features --features huggingface`.  
**GitHub Release binaries** use `--features airframe,huggingface` (built in CI matrix).

---

## What's Gone (v2.0 → v2.2)

| Removed | Why |
|---------|-----|
| CUDA Gate (Gate 2) | Airframe uses wgpu; zero CUDA dependency |
| `shimmy-llama-cpp-2` dependency | Replaced by Airframe GPU engine |
| `--features llama` build path | No llama.cpp in this codebase |
| MLX-specific release artifacts | Not part of v2.0 shipping target |
| Binary size inflation from llama.cpp | Airframe is pure Rust; binary stays well under 20 MB |
| ~2900 lines of tests in 19 files | Consolidated to ~660 lines across 3 files |
| Per-issue-file regression structure | Replaced by architecturally-organized test files |
