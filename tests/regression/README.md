# Regression Test Directory

**Purpose**: Organized regression tests preventing user-reported bugs from recurring.

## Structure

Each issue gets its own test file following naming convention:
- `issue_NNN_<short_description>.rs` - For specific GitHub issues
- Tests must compile, run, and pass before closing related issues

## Complete Regression Test Inventory

**TOTAL: 20 test files covering 23+ user-reported issues**

### Organized Regression Tests (tests/regression/)

| Issue # | File | Description | Status |
|---------|------|-------------|--------|
| #12 | `issue_012_custom_model_dirs.rs` | Custom model directory environment variables | ✅ Active |
| #13 | `issue_013_qwen_template.rs` | Qwen model ChatML template detection | ✅ Active |
| #51 | `issue_051_lmstudio_discovery.rs` | LMStudio model auto-discovery | ✅ Active |
| #53 | `issue_053_sse_duplicate_prefix.rs` | SSE streaming duplicate 'data:' prefix | ✅ Active |
| #58, #59 | `issue_058_059_cuda_compilation.rs` | CUDA compilation errors, GPU prebuilt binaries | ✅ Active |
| #63 | `issue_063_version_mismatch.rs` | Pre-built Windows exe version reporting | ✅ Active |
| #64 | `issue_064_template_packaging.rs` | Template files missing from crates.io | ✅ Active |
| #65 | `issue_065_error_handling.rs` | Better error handling for missing models | ✅ Active |
| #68 | `issue_068_mlx_support.rs` | MLX Apple Silicon compilation support | ✅ Active |
| #72 | `issue_072_gpu_backend_flag.rs` | GPU backend flag not wired to model loading | ✅ Active |
| #87 | `issue_087_apple_gpu_info.rs` | Apple GPU info detection errors | ✅ Active |
| #101 | `issue_101_performance_fixes.rs` | Threading, streaming, OLLAMA_MODELS support | ✅ Active |
| #106 | `issue_106_windows_crash.rs` | Windows server crashes | ✅ Active |
| #108 | `issue_108_memory_allocation.rs` | Memory allocation CLI flags broken | ✅ Active |
| #109 | `issue_109_anthropic_api.rs` | Anthropic Claude API compatibility | ✅ Active |
| #110 | `issue_110_crates_io_build.rs` | Build failure on cargo install shimmy | ✅ Active |
| #111 | `issue_111_gpu_metrics.rs` | GPU metrics missing from /metrics endpoint | ✅ Active |
| #112 | `issue_112_safetensors_engine.rs` | SafeTensors engine selection | ✅ Active |
| #113 | `issue_113_openai_api.rs` | OpenAI API frontend compatibility | ✅ Active |
| #114 | `issue_114_mlx_distribution.rs` | MLX distribution pipeline support | ✅ Active |
| #127, #128 | `issue_127_128_mlx_placeholder.rs` | MLX placeholder instead of proper error | ✅ Active |
| General | `issue_012_013_model_discovery.rs` | Combined model discovery tests (#12, #13) | ✅ Active |
| General | `issue_packaging_general.rs` | General packaging regression tests | ✅ Active |
| General | `issue_version_validation.rs` | Version validation across releases | ✅ Active |

### Additional Coverage in Integration Tests

| Issue # | Location | Notes |
|---------|----------|-------|
| #60 | `tests/release_gate_integration.rs` | Template inclusion gate tests |
| #72 | `tests/gpu_backend_tests.rs`, `tests/gpu_layer_verification.rs` | Additional GPU integration tests |
| #101 | `tests/cli_integration_tests.rs` | CLI-level integration testing |

## Adding New Regression Tests

**MANDATORY when fixing any user-reported bug:**

1. **Create test file**: `tests/regression/issue_NNN_<description>.rs`
2. **Write failing test**: Reproduce the exact bug scenario
3. **Fix the bug**: Make changes to src/
4. **Verify test passes**: `cargo test --test issue_NNN_<description>`
5. **Add to this README**: Update table above
6. **Commit together**: Test file + fix in same commit

## Running Regression Tests

```bash
# Run all regression tests
cargo test --tests regression/

# Run specific issue test
cargo test --test regression/issue_072_gpu_backend_flag

# Run with specific features
cargo test --test regression/issue_127_128_mlx_placeholder --features mlx
```

## CI/CD Integration

Regression tests run automatically in:
- `.github/workflows/ci.yml` - On every PR
- `.github/workflows/release.yml` - Before every release
- Pre-commit hooks (future)

**Zero tolerance**: If ANY regression test fails, PR/release BLOCKED.

## Test Requirements

Each regression test MUST:
1. **Reference issue number** in file name and doc comment
2. **Link to GitHub issue** in module-level documentation
3. **Describe exact bug** that was fixed
4. **Test the fix** with clear assertions
5. **Be reproducible** - no flaky tests allowed
6. **Run in CI/CD** - verified automatically

## Example Test Structure

```rust
/// Regression test for Issue #XXX: Bug description
///
/// GitHub: https://github.com/Michael-A-Kuykendall/shimmy/issues/XXX
///
/// **Bug**: What was broken
/// **Fix**: What changed
/// **This test**: How we verify it stays fixed

#[cfg(test)]
mod issue_xxx_tests {
    use super::*;

    #[test]
    fn test_issue_xxx_bug_scenario() {
        // Reproduce exact bug conditions
        // Assert fix works
        // Add helpful error messages
    }

    #[test]
    fn test_issue_xxx_edge_cases() {
        // Test boundary conditions
    }
}
```

## Maintenance

- **NEVER delete** regression tests unless issue was invalid
- **NEVER skip** failing regression tests - FIX THEM
- **Update immediately** if APIs change (keep tests working)
- **Review quarterly** for obsolete tests (rare)
