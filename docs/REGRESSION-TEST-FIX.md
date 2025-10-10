# Regression Test Fix - Detective Work Summary

## What Happened

The regression test bash script (`scripts/run-regression-tests.sh`) was calling test functions **incorrectly**, causing all tests to be filtered out (0 tests run).

## Root Cause Investigation

### Timeline:
1. **Sept 12, 2025 (commit 3bf14cc)**: Original bash script created for v1.3.2 release
   - Script called: `cargo test test_model_discovery`
   - But `tests/regression_tests.rs` **didn't exist yet**!

2. **Sept 12, 2025 (commit a752e2e)**: `tests/regression_tests.rs` file created
   - Tests were placed inside a module: `mod regression_tests { ... }`
   - Test functions existed: `test_model_discovery_functionality`, `test_openai_api_structures_serialization`, etc.

3. **Problem**: The bash script and the test file **were created separately** and never properly synchronized

### Why Tests Failed:

The bash script was calling:
```bash
cargo test test_model_discovery --features huggingface
```

But the actual test is:
- **File**: `tests/regression_tests.rs`
- **Module**: `mod regression_tests { ... }`
- **Function**: `fn test_model_discovery_functionality()`

Correct call should be:
```bash
cargo test --test regression_tests test_model_discovery_functionality --features huggingface
```

## The Fix

Updated all test calls in `scripts/run-regression-tests.sh`:

### Before (WRONG):
```bash
cargo test test_model_discovery --features huggingface
cargo test test_openai_api --features huggingface
cargo test test_qwen_model_template_detection --features huggingface
cargo test test_custom_model_directory_environment_variables --features huggingface
cargo test test_cli_model_dirs_option_compatibility --features huggingface
cargo test test_error_handling_robustness --features huggingface
```

### After (CORRECT):
```bash
cargo test --test regression_tests test_model_discovery_functionality --features huggingface
cargo test --test regression_tests test_openai_api_structures_serialization --features huggingface
cargo test --test regression_tests test_qwen_model_template_detection --features huggingface
cargo test --test regression_tests test_custom_model_directory_environment_variables --features huggingface
cargo test --test regression_tests test_cli_model_dirs_option_compatibility --features huggingface
cargo test --test regression_tests test_error_handling_robustness --features huggingface
```

## Tests That Were Already Working

- ✅ Issue #72 (GPU backend): `cargo test --no-default-features --features huggingface,llama-opencl,llama-vulkan gpu_backend`
  - This was calling tests in `tests/gpu_backend_tests.rs` correctly
  - 9 tests passed including `test_issue_72_gpu_backend_flag_respected`

- ✅ MLX Support: `tests/mlx_support_regression_test.rs`
  - 10 comprehensive MLX tests
  - All passing

- ✅ Release Gates: `tests/release_gate_integration.rs`
  - 9 gate validation tests
  - Validates CI/CD workflow

## Still Missing: MOE Tests

**Action Required**: Create `tests/moe_cpu_offload_regression_test.rs` with tests for:
- `--cpu-moe` flag functionality
- `--n-cpu-moe N` flag functionality
- MOE feature compilation
- MOE CLI flag parsing

## Verification

Run the fixed script:
```bash
bash scripts/run-regression-tests.sh
```

All phases should now pass (except MOE tests which don't exist yet).

## Lesson Learned

**Problem**: Test file and test runner script created separately, never synchronized
**Solution**: Either:
1. Use CI/CD as single source of truth (`.github/workflows/release.yml`)
2. Migrate to pure Rust tests: `cargo test --workspace`
3. If using bash scripts, validate test names match actual functions

**Recommendation**: Deprecate bash script, use GitHub Actions release gates as the authoritative test suite.
