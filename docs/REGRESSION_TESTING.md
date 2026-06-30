# Regression Testing System

## Overview

Shimmy uses an **automated regression testing system** to prevent previously fixed bugs from returning. Every user-reported bug that gets fixed MUST have a corresponding regression test.

## Test Structure (v2.2+)

As of v2.2, the old per-issue-file structure under `tests/regression/` has been retired in favor of three consolidated, architecturally-organized test files:

```
tests/
├── core.rs             # 28 tests — CLI, registry, templates, discovery,
│                       #            safetensors, OpenAI/Ollama serde, versions,
│                       #            multi-part content, SSE chunk format, error format
├── handlers.rs         #  5 tests — HTTP server health, /v1/models,
│                       #            /v1/chat/completions, /api/tags, concurrency
└── compile_checks.rs   #  1 test  — include_str template file existence
```

### Historical Issue Coverage

Every user-reported issue that previously had a dedicated regression file is now covered in the consolidated suite:

| Issue | Title | New Test(s) | Location |
|-------|-------|-------------|----------|
| #12 | Custom model directories not detected | `test_custom_model_directory_env_vars`, `test_cli_model_dirs_option` | `core.rs:208-218`, `core.rs:56-63` |
| #13 | Qwen / VSCode wrong template | `test_template_auto_detection`, `test_registry_infer_template` | `core.rs:557-579`, `core.rs:171-186` |
| #53 | SSE duplicate `data:` prefix | `test_sse_streaming_chunk_format` | `core.rs:435-486` |
| #63 | Windows exe wrong version | `test_version_is_sane` | `core.rs:123-132` |
| #64 / #73 / #86 / #88 | Missing template files | `test_template_files_exist` | `compile_checks.rs:4-44` |
| #65 | 404 error format | `test_error_response_json_shape`, `test_chat_completions_model_not_found` | `core.rs:366-381`, `handlers.rs:100-126` |
| #112 | SafeTensors wrong engine | `test_safetensors_extension_detection` | `core.rs:256-280` |
| #113 | OpenAI / Ollama frontend compat | `test_model_struct_completeness`, `test_models_response_serde`, `test_api_tags_response_structure`, and handler endpoint tests | `core.rs:347-363`, `core.rs:330-344`, `core.rs:388-433`, `handlers.rs:78-147` |
| #191 | 422 multi-part content array | `test_multi_part_content_array_deserialization` | `core.rs:439-456` |

### Retired Tests

The following user-reported issues did **not** carry forward as dedicated tests because their old tests covered code paths that no longer exist or only tested process-spawning behavior. Each is still noted here for transparency:

| Issue | Title | Reason for Retirement |
|-------|-------|-----------------------|
| #68 | MLX Apple Silicon support | MLX engine is an empty stub; no runtime behavior to lock in |
| #72 | GPU backend flag | Test was a no-op compilation check; actual GPU wiring is tested by Airframe |
| #80 | LLM-only filtering | Spawned real binary via `assert_cmd` — tested flag parsing, not core logic |
| #87 | Apple GPU info detection | MLX engine is a stub |
| #101 | High CPU / streaming perf | Spawned real binary — tested `--help`, not streaming behavior |
| #106 | Windows server crash | Spawned real binary — tested `serve --help`, not crash handling |
| #108 | Memory allocation flags | Spawned real binary |
| #109 | Anthropic API format | Anthropic translation layer removed from codebase |
| #111 | GPU metrics endpoint | No-op test (empty body) |
| #114 | MLX distribution features | MLX engine is a stub |
| #127 / #128 | MLX placeholder errors | MLX engine is a stub |

## Running Tests

```bash
# Full suite (everything)
cargo test --features airframe,huggingface

# Core unit tests only
cargo test --test core --features airframe,huggingface

# HTTP handler tests only
cargo test --test handlers --features airframe,huggingface

# Template compilation check only
cargo test --test compile_checks
```

## Adding a Regression Test for a New Issue

**MANDATORY when fixing any user-reported bug:**

1. Identify which architectural area the bug belongs to (CLI, registry, templates, handlers, serde, etc.)
2. Add the test to the appropriate file (`core.rs`, `handlers.rs`, or `compile_checks.rs`)
3. Verify the test fails before the fix, passes after
4. Add a cross-reference comment above the test in the format:

```rust
// Regression: Issue #XXX — one-line description
```

## Policy

- **Every** user-reported bug fix requires a regression test
- **Zero tolerance**: all tests must pass before merge or release
- Tests run automatically on every PR and every release via `ci.yml` and `release.yml`
