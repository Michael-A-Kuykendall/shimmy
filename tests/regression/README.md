# Regression Test Directory

**Purpose**: Organized regression tests preventing user-reported bugs from recurring.

## Active Tests (7 files)

Each tests current shimmy v2.x functionality:

| Issue | File | What it tests |
|-------|------|---------------|
| #12 | `issue_012_custom_model_dirs.rs` | Custom model directory env vars and discovery |
| #13 | `issue_013_qwen_template.rs` | Qwen model ChatML template inference |
| #53 | `issue_053_sse_duplicate_prefix.rs` | SSE streaming `data:` prefix format |
| #63 | `issue_063_version_mismatch.rs` | CLI commands and version consistency |
| #64 | `issue_064_template_packaging.rs` | Template files included in package |
| #112 | `issue_112_safetensors_engine.rs` | SafeTensors file extension routing |
| #113 | `issue_113_openai_api.rs` | OpenAI API response structure |

## Removed Tests

The following were removed in the v2.0 airframe migration because they tested
dead or transitional functionality:

| Issue | File | Reason |
|-------|------|--------|
| #68 | `issue_068_mlx_support.rs` | MLX is empty `[]` feature stub, no code |
| #110 | `issue_110_crates_io_build.rs` | `publish = false`, binary-only dist; tests used slow `cargo` subprocesses |
| #111 | `issue_111_gpu_metrics.rs` | Two no-op tests; compile-time check enforced by rest of codebase |

## Running

```bash
cargo test --features airframe,huggingface --test regression
```
