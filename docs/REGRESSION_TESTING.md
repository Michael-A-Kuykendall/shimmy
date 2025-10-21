# Regression Testing System

## Overview

Shimmy uses an **automated regression testing system** to prevent previously fixed bugs from returning. Every user-reported bug that gets fixed MUST have a corresponding regression test.

## Purpose

**Problem**: Users report bugs → We fix them → Time passes → Someone accidentally breaks the fix → User reports same bug again → Trust destroyed

**Solution**: Automated regression tests that run on every PR and release, catching regressions before they reach users.

## Directory Structure

```
tests/regression/
├── README.md (This file - regression test inventory)
├── issue_012_custom_model_dirs.rs
├── issue_013_qwen_template.rs
├── issue_053_sse_duplicate_prefix.rs
├── issue_063_version_mismatch.rs
├── issue_064_template_packaging.rs
├── issue_068_mlx_support.rs
├── issue_072_gpu_backend_flag.rs
├── issue_101_performance_fixes.rs
├── issue_108_memory_allocation.rs
└── issue_127_128_mlx_placeholder.rs
```

## Active Regression Tests

| Issue(s) | Test File | Description | Created |
|----------|-----------|-------------|---------|
| #12 | `issue_012_custom_model_dirs.rs` | Custom model directory environment variables not detected | ✅ |
| #13 | `issue_013_qwen_template.rs` | Qwen models don't use correct ChatML templates in VSCode | ✅ |
| #53 | `issue_053_sse_duplicate_prefix.rs` | SSE streaming responses contain duplicate 'data:' prefix | ✅ |
| #63 | `issue_063_version_mismatch.rs` | Pre-built Windows exe reports wrong version | ✅ |
| #64 | `issue_064_template_packaging.rs` | Template files missing from crates.io package | ✅ |
| #68 | `issue_068_mlx_support.rs` | MLX Apple Silicon support broken | ✅ |
| #72 | `issue_072_gpu_backend_flag.rs` | GPU backend flag parsed but not wired into model loading | ✅ |
| #101 | `issue_101_performance_fixes.rs` | Threading optimization, streaming, OLLAMA_MODELS support | ✅ |
| #108 | `issue_108_memory_allocation.rs` | Memory allocation CLI flags broken | ✅ |
| #127, #128 | `issue_127_128_mlx_placeholder.rs` | MLX engine returns placeholder string instead of proper error | ✅ |

## Workflow: Adding a Regression Test

**MANDATORY when fixing any user-reported bug:**

### Step 1: Create Test File

```bash
# File naming: issue_<number(s)>_<short_description>.rs
touch tests/regression/issue_XXX_bug_description.rs
```

### Step 2: Write Test That Reproduces Bug

```rust
/// Regression test for Issue #XXX: One-line bug description
///
/// GitHub: https://github.com/Michael-A-Kuykendall/shimmy/issues/XXX
///
/// **Bug**: Detailed description of what was broken
/// **Fix**: What code change fixed it (file/line references)
/// **This test**: How this test prevents regression

#[cfg(test)]
mod issue_XXX_tests {
    use shimmy::*;  // Import relevant modules

    #[test]
    fn test_issue_XXX_exact_bug_scenario() {
        // Set up exact conditions that triggered the bug
        
        // Perform the action that was broken
        
        // Assert the fix works (this should FAIL before fix, PASS after)
        assert!(/* condition that proves bug is fixed */);
        
        println!("✅ Issue #XXX regression test: Bug prevented");
    }

    #[test]
    fn test_issue_XXX_edge_cases() {
        // Test boundary conditions and variations
        
        println!("✅ Issue #XXX edge cases: Verified");
    }
}
```

### Step 3: Verify Test Fails Before Fix

```bash
# This proves your test actually catches the bug
cargo test --test regression/issue_XXX_bug_description --features <relevant>
# Expected: test result: FAILED
```

### Step 4: Apply Your Fix

Make changes to `src/` files that fix the bug.

### Step 5: Verify Test Passes After Fix

```bash
# This proves your fix works
cargo test --test regression/issue_XXX_bug_description --features <relevant>
# Expected: test result: ok
```

### Step 6: Update Documentation

Add row to the table in `tests/regression/README.md`:

```markdown
| #XXX | `issue_XXX_bug_description.rs` | Brief description of bug | ✅ |
```

### Step 7: Commit Test + Fix Together

```bash
git add tests/regression/issue_XXX_bug_description.rs
git add src/file_you_fixed.rs
git add tests/regression/README.md
git commit -m "fix: Issue #XXX - Bug description

- Fixed: Detailed explanation of what was broken
- Added: Regression test to prevent recurrence
- Test: tests/regression/issue_XXX_bug_description.rs

Closes #XXX"
```

## Running Regression Tests

### All Regression Tests (Automated)

```bash
# Discovers and runs ALL tests in tests/regression/ automatically
bash scripts/run-regression-tests-auto.sh
```

Output shows pass/fail for each issue test with color coding.

### Single Regression Test

```bash
# Run specific issue test
cargo test --test regression/issue_072_gpu_backend_flag

# With specific features
cargo test --test regression/issue_127_128_mlx_placeholder --features mlx
```

### In CI/CD (Automatic)

Regression tests run automatically on:
- **Every Pull Request** (`.github/workflows/ci.yml` - `regression-tests` job)
- **Every Release** (`.github/workflows/release.yml` - before deployment)
- **Manual Trigger** (GitHub Actions UI)

If ANY regression test fails → PR blocked, release blocked.

## Test Requirements (Mandatory)

Each regression test file MUST include:

1. **File naming**: `issue_<num(s)>_<description>.rs`
2. **Module doc comment** with:
   - Issue number(s) and description
   - GitHub issue link
   - Bug/Fix/Test explanation
3. **At least one test** reproducing exact bug scenario
4. **Clear assertions** with helpful error messages
5. **Feature flags** (if needed): `#[cfg(feature = "mlx")]`
6. **Success messages**: `println!("✅ Issue #XXX: Verified")`

## Auto-Discovery System

**How it works:**

1. `scripts/run-regression-tests-auto.sh` scans `tests/regression/` for `issue_*.rs` files
2. Extracts issue number from filename
3. Determines required features from filename keywords (mlx, gpu, etc.)
4. Runs each test with appropriate features
5. Collects results and reports pass/fail summary
6. Exits non-zero if ANY test fails (blocks CI/CD)

**Adding new tests is ZERO CONFIG:**
- Just create `tests/regression/issue_NNN_description.rs`
- Script auto-discovers and runs it
- No manual registration needed

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
jobs:
  regression-tests:
    name: Regression Tests (Zero Tolerance)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Run Automated Regression Tests
        run: |
          chmod +x ./scripts/run-regression-tests-auto.sh
          ./scripts/run-regression-tests-auto.sh
```

### Enforcement

- Regression tests run BEFORE main test suite
- If regressions fail → Entire CI/CD fails
- Pull requests cannot merge with failing regression tests
- Releases cannot deploy with failing regression tests

## Maintenance Guidelines

### NEVER Delete Regression Tests

Unless the original issue was invalid/duplicate, regression tests are **permanent**.

### Update When APIs Change

If internal APIs change and break regression tests:
- **Fix the test** to use new APIs
- **Maintain same verification logic**
- **Never delete** to make CI pass

### Review Quarterly

Every 3 months, review regression tests for:
- Obsolete tests (rare - usually keep them)
- Tests that could be combined
- Missing coverage for known issues

## Example: Real Regression Test

From `tests/regression/issue_072_gpu_backend_flag.rs`:

```rust
/// Regression test for Issue #72: GPU backend flag ignored
///
/// GitHub: https://github.com/Michael-A-Kuykendall/shimmy/issues/72
///
/// **Bug**: --gpu-backend flag was parsed but not actually wired into model loading
/// **Fix**: Properly pass GPU backend selection through to llama.cpp initialization
/// **This test**: Verifies GPU backend flag is respected in model loading path

#[cfg(test)]
mod issue_072_tests {
    use shimmy::engine::ModelSpec;
    use std::path::PathBuf;

    #[test]
    #[cfg(any(feature = "llama-opencl", feature = "llama-vulkan", feature = "llama-cuda"))]
    fn test_gpu_backend_flag_wiring() {
        let spec = ModelSpec {
            name: "test-gpu-model".to_string(),
            base_path: PathBuf::from("test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: Some(4),
        };

        // Verify model spec can be created with GPU features enabled
        assert_eq!(spec.name, "test-gpu-model");
        
        println!("✅ Issue #72 regression test: GPU backend flag compilation verified");
    }
}
```

## Benefits

1. **User Trust**: Bugs don't come back → users trust us
2. **Developer Confidence**: Refactor safely knowing tests catch regressions
3. **Automatic Enforcement**: CI/CD blocks bad code before it ships
4. **Living Documentation**: Tests show exactly what bugs existed and how they were fixed
5. **Zero Configuration**: Add test file → auto-discovered → auto-runs
6. **Clear Reporting**: See exactly which issue regressed if test fails

## Zero Tolerance Policy

**If a regression test fails:**

1. **STOP** - Do not merge PR, do not release
2. **Investigate** - Why did previously fixed bug come back?
3. **Fix** - Restore the original fix or update properly
4. **Verify** - Ensure regression test passes again
5. **Proceed** - Only then continue with PR/release

**NEVER:**
- Skip failing regression tests
- Delete regression tests to make CI pass
- Ignore regression failures as "flaky"
- Merge with `--no-verify` to bypass checks

## Questions?

- **How do I name multi-issue tests?** Use underscores: `issue_127_128_description.rs`
- **What if test needs specific features?** Auto-detected from filename (mlx, gpu) or specify manually
- **Can I group related tests?** Each issue should have its own file for clarity
- **What if fix affects multiple files?** One regression test per bug, regardless of fix complexity

## Related Documentation

- `.github/copilot-instructions.md` - RULE THREE: Regression test requirements
- `scripts/run-regression-tests-auto.sh` - Auto-discovery runner script
- `.github/workflows/ci.yml` - CI/CD integration
- `LOCAL_GITHUB_ACTIONS_GUIDE.md` - ACT testing for platform-specific tests
