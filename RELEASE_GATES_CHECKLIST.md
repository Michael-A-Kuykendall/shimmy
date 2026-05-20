# Release Gates & Regression Tests — v2.0

**Date:** May 20, 2026  
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
| 5.5/7 | Issue Regression | Per-issue regression tests (see list below) |
| 6/7 | Documentation | `cargo doc --no-deps --features huggingface` |
| 7/7 | crates.io Dry-Run | `cargo publish --dry-run --features huggingface` validates crates.io package |

---

## Regression Test Files

### Active Test Files
| File | Status |
|------|--------|
| `tests/regression_tests.rs` | ✅ Active — main issue regression suite |
| `tests/release_gate_integration.rs` | ✅ Active — validates gate system itself |
| `tests/template_compilation_regression_test.rs` | ✅ Active — template file inclusion |
| `tests/compilation_regression_test.rs` | ✅ Active — compilation sanity |
| `tests/apple_silicon_detection_test.rs` | ✅ Active — GPU detection (Issue #87) |
| `tests/integration_tests.rs` | ✅ Active — API surface integration |
| `tests/cli_integration_tests.rs` | ✅ Active — CLI surface |
| `tests/gpu_backend_tests.rs` | ✅ Active — GPU backend selection |

### Gate 5.5 Issue Regression Tests (in `regression_tests.rs`)
| Test | Issue |
|------|-------|
| `test_issue_111_gpu_metrics_endpoint` | #111 — GPU metrics endpoint |
| `test_issue_112_safetensors_engine_selection` | #112 — SafeTensors engine selection |
| `test_issue_113_openai_api_frontend_compatibility` | #113 — OAI `/v1/models` endpoint |
| `test_issue_113_ollama_api_tags_response_structure` | #113 — Ollama `/api/tags` endpoint |
| `test_issue_114_mlx_distribution_features` | #114 — MLX distribution feature flags |
| `test_issue_191_multi_part_content_array_deserialization` | #191 — Multi-part content array (422 fix) |
| `test_qwen_model_template_detection` | #13 — Qwen ChatML template |
| `test_custom_model_directory_environment_variables` | #12 — Custom model directories |

---

## Test Execution Commands

```bash
# Gate 5 — Core lib tests
cargo test --lib --no-default-features --features huggingface -- --test-threads=1

# Gate 5.1 — Airframe compile check
cargo check --features airframe

# Gate 5.5 — Issue regression tests
cargo test --test regression_tests --no-default-features --features huggingface

# Gate validation system tests
cargo test --test release_gate_integration

# Full regression pass
cargo test --no-default-features --features huggingface
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

## What's Gone (v2.0)

| Removed | Why |
|---------|-----|
| CUDA Gate (Gate 2) | Airframe uses wgpu; zero CUDA dependency |
| `shimmy-llama-cpp-2` dependency | Replaced by Airframe GPU engine |
| `--features llama` build path | No llama.cpp in this codebase |
| MLX-specific release artifacts | Not part of v2.0 shipping target |
| Binary size inflation from llama.cpp | Airframe is pure Rust; binary stays well under 20 MB |

