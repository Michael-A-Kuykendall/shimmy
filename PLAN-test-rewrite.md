# Shimmy Test Rewrite Plan

## Goal
Replace all existing test files (~2500 lines, 14 files) with 3 clean files (~350 lines) that test what shimmy IS today, not what it was.

## Target Files

```
tests/
  core.rs              # Fast unit tests (~200 lines)
  handlers.rs           # HTTP handler integration tests (~120 lines)
  compile_checks.rs    # Compile-time assertions (~30 lines)
```

Wipe everything else in `tests/` (except possibly `.gitkeep`).

---

## What to steal from existing tests

### → `core.rs`

| Source file | What to steal | What it tests |
|---|---|---|
| `regression_tests.rs` | `test_model_registry_basic_operations` | Registry::new/register/get/list |
| `regression_tests.rs` | `test_model_discovery_functionality` | discover_models_in_directory |
| `regression_tests.rs` | `test_template_rendering_regression` | TemplateFamily::render (ChatML, Llama3) |
| `regression_tests.rs` | `test_template_inference_regression` | Registry::infer_template name matching |
| `regression_tests.rs` | `test_openai_api_structures_serialization` | ChatCompletionRequest/Response serde |
| `regression_tests.rs` | `test_models_response_api_compatibility` | ModelsResponse serde |
| `regression_tests.rs` | `test_issue_191_multi_part_content_array_deserialization` | OAIMessage multi-part content |
| `regression_tests.rs` | `test_issue_113_ollama_api_tags_response_structure` | Ollama /api/tags JSON shape |
| `regression_tests.rs` | `test_error_handling_robustness` | Registry error edge cases |
| `regression_tests.rs` | `test_cli_model_dirs_option_compatibility` | CLI --model-dirs parsing |
| `issue_012_custom_model_dirs.rs` | env var + dir discovery tests | SHIMMY_MODELS_DIR / OLLAMA_MODELS |
| `issue_112_safetensors_engine.rs` | extension detection tests | .safetensors file routing |
| `issue_113_openai_api.rs` | Model struct + ModelsResponse | OpenAI field completeness |
| `openai_api_real_tests.rs` | `test_chat_completions_model_loading_failure` | Request struct validation |
| `openai_api_real_tests.rs` | `test_system_message_handling` | Multi-message request structure |
| `openai_api_real_tests.rs` | `test_streaming_request_processing` | stream=true in request |
| `openai_api_real_tests.rs` | `test_template_auto_detection_comprehensive` | Template name→family mapping |
| `openai_api_real_tests.rs` | `test_generation_options_parsing` | temperature/max_tokens/top_p |
| `openai_api_real_tests.rs` | `test_openai_response_serialization` | ChatCompletionResponse shape |
| `integration_tests.rs` | `test_cli_parsing` | Cli::try_parse_from for each command |
| `integration_tests.rs` | `test_template_rendering` | TemplateFamily::render with system |

### → `handlers.rs`

| Source file | What to steal | What it tests |
|---|---|---|
| `openai_api_real_tests.rs` | `test_models_endpoint_real_functionality` | GET /v1/models returns registered models |
| `openai_api_real_tests.rs` | `test_chat_completions_error_handling_real` | POST /v1/chat/completions 404 for unknown |
| `integration_tests.rs` | `test_http_api_health_check` | GET /health returns 200 |
| `integration_tests.rs` | `test_concurrent_requests` | 5 concurrent health checks pass |
| `regression_tests.rs` | Ollama tags structure (already in core) | GET /api/tags shape |

### → `compile_checks.rs`

| Source file | What to steal | What it tests |
|---|---|---|
| `issue_064_template_packaging.rs` | `test_template_files_are_included_in_package` | include_str! template paths exist |
| `issue_064_template_packaging.rs` | `test_template_content_validity` | Template content basic sanity |

---

## What NOT to keep (and why)

| Source file | Reason |
|---|---|
| `issue_013_qwen_template.rs` | Template inference covered generically in core.rs |
| `issue_053_sse_duplicate_prefix.rs` | SSE format tested end-to-end by handler; these test serde strings |
| `issue_063_version_mismatch.rs` | 7 nearly-identical tests for `env!("CARGO_PKG_VERSION")`. One line in core.rs is enough |
| `anthropic_api_integration_test.rs` | Thin translation layer over same engine. If OpenAI works, Anthropic works. Pure JSON tests test serde |
| `template_compilation_regression_test.rs` | Overlaps compile_checks; tests output file creation, not inference |
| `cli_integration_tests.rs` | Spawns real binary; tests minor flag edge cases (--llm-only, --cpu-moe) |
| `api_error_handling_test.rs` | Error JSON format tested by real handler call in handlers.rs |
| `workflow_tests.rs` | Workflow engine is `#[allow(dead_code)]` — dead stub |
| `safetensors_integration.rs` | Tests `create_test_safetensors` — a test helper, not user-facing |
| `regression_tests.rs` | Steal what we need, delete the file |
| `regression.rs` + `regression/` | Steal what we need, delete the directory and index |
| `apple_silicon_detection_test.rs` | Already deleted (all cfg(mlx) dead code) |
| `gpu_backend_tests.rs` | Already deleted (all identical no-op) |
| `gpu_layer_verification.rs` | Already deleted (all identical no-op) |
| `release_gate_integration.rs` | Already deleted (tested old gate system) |

---

## Execution Order

1. Read source files to copy-paste test bodies into the 3 new files
2. Write `tests/core.rs` (~200 lines)
3. Write `tests/handlers.rs` (~120 lines)
4. Write `tests/compile_checks.rs` (~30 lines)
5. Delete all old test files:
   - `tests/regression.rs` + `tests/regression/` directory
   - `tests/regression_tests.rs`
   - `tests/openai_api_real_tests.rs`
   - `tests/anthropic_api_integration_test.rs`
   - `tests/template_compilation_regression_test.rs`
   - `tests/cli_integration_tests.rs`
   - `tests/api_error_handling_test.rs`
   - `tests/workflow_tests.rs`
   - `tests/integration_tests.rs`
   - `tests/safetensors_integration.rs`
6. `cargo check` — fix any compile errors
7. `cargo fmt --check` — fix formatting
8. `cargo test` — all pass
9. Commit and push

---

## After Shimmy

Follow the same process for `C:\Users\micha\repos\airframe`:
- Audit what airframe IS (features, public API)
- Design minimal test suite from scratch
- Steal what's useful from existing tests
- Write fresh, delete old
- Verify locally, push
