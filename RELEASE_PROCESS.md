# Shimmy Release Process — v2.0

This document describes the release process for Shimmy v2.0+ (Airframe GPU engine, wgpu/WebGPU backend).

---

## Gate Overview

7 mandatory gates run in the `preflight` job of `release.yml`. All must pass or the release stops.

| Gate | What it validates |
|------|-------------------|
| 1/7 — Core Build | `cargo build --features airframe,huggingface` succeeds with the Airframe submodule |
| ~~2~~ | **Removed** — CUDA gate is gone. Airframe uses wgpu; no CUDA dependency exists. |
| 3/7 — Template Packaging | Docker templates are included in the crates.io package (Issue #60 regression guard) |
| 4/7 — Binary Size | `shimmy` binary stays under 20 MB constitutional limit |
| 5/7 — Test Suite | `cargo test --lib --no-default-features --features huggingface` passes |
| 5.1/7 — Airframe Compile | `cargo check --features airframe` — GPU engine integration compiles cleanly |
| 5.5/7 — Issue Regression | `cargo test --test core --test handlers --test compile_checks` all pass (see `docs/REGRESSION_TESTING.md` for issue-to-test mapping) |
| 6/7 — Documentation | `cargo doc` builds without errors |
| 7/7 — crates.io Dry-Run | `cargo publish --dry-run` validates the crates.io-safe feature set |

---

## Recommended Workflow

### 1. Validate locally first (fastest)

```bash
# Core build with Airframe engine
cd shimmy_integration
cargo build --release --features airframe,huggingface

# Non-GPU regression tests
cargo test --no-default-features --features huggingface

# Airframe compile check
cargo check --features airframe

# Regression tests (consolidated suite)
cargo test --test core --test handlers --test compile_checks --features airframe,huggingface
```

### 2. Dry-run on the test branch (exact CI environment)

```bash
git checkout -b test-release-vX.Y.Z
git push private test-release-vX.Y.Z
# Wait for release.yml to run on GitHub Actions
# Inspect gate results before tagging
```

> **Note:** Always push to `private`, never to `origin` (public repo).

### 3. Tag and release

```bash
git checkout main
git tag vX.Y.Z
git push private vX.Y.Z

# Clean up test branch
git push private --delete test-release-vX.Y.Z
git branch -d test-release-vX.Y.Z
```

---

## Troubleshooting

### Gate 1 (Core Build) fails
- Ensure the `airframe` submodule is checked out: `git submodule update --init --recursive`
- Verify `AIRFRAME_ACCESS_TOKEN` secret is set in the GitHub repo settings

### Gate 3 (Template Packaging) fails
- Check that `templates/docker/Dockerfile` exists and is committed
- The system handles `Cargo.lock` changes automatically via `--allow-dirty`

### Gate 4 (Binary Size) fails
- Binary exceeded 20 MB — review recently added dependencies
- `cargo bloat --release` to identify the largest contributors

### Gate 5.1 (Airframe Compile) fails
- The Airframe submodule at `airframe/` has a compilation error
- `cargo check --features airframe` locally to see the error

### Gate 5.5 (Regression Tests) fails
- A specific regression test failed; check `docs/REGRESSION_TESTING.md` for the issue mapping
- Fix the regression before releasing

### Gate 7 (crates.io Dry-Run) fails
- If `path` dependency error: ensure publish uses `--no-default-features --features huggingface` (no `airframe` path dep)
- If version conflict: bump the version in `Cargo.toml`

---

## Feature Flag Reference

| Flag | crates.io safe | Included in published binary |
|------|---------------|------------------------------|
| `huggingface` | ✅ | ✅ crates.io install |
| `airframe` | ❌ (path dep) | ✅ GitHub Release binary |
| `full` / `apple` | ❌ | ✅ platform builds |

The crates.io version (`cargo install shimmy`) ships with `huggingface` only.  
GPU-accelerated Airframe binaries are distributed via GitHub Releases.

---

## Files in This System

- `scripts/dry-run-release.sh` — Local gate emulation
- `.github/workflows/release.yml` — Real release gates (7 gates)
- `.github/workflows/ci.yml` — PR/push quality gates
- `RELEASE_GATES_CHECKLIST.md` — Live gate status reference
- `RELEASE_PROCESS.md` — This document


This document describes the **bulletproof release process** that eliminates public CI failures through complete dry-run testing.

## The Problem We Solved

- ❌ Release gates always blow up publicly
- ❌ Complex 6-gate system fails unpredictably  
- ❌ No way to test the exact release environment privately
- ❌ Red CI badges everywhere because everyone's CI breaks

## The Solution: Complete Release Emulation

We now have **3 ways** to test releases privately before going public:

### 1. Local Dry Run (Fastest)

Run the exact same 6 gates locally:

```bash
# Make executable
chmod +x scripts/dry-run-release.sh

# Run complete local emulation
./scripts/dry-run-release.sh
```

**Pros**: Instant feedback, no GitHub Actions minutes used
**Cons**: Your local environment might differ slightly from GitHub Actions

### 2. Private GitHub Actions Dry Run (Most Accurate)

Test in the exact same environment as the real release:

```bash
# Option A: Manual trigger
# Go to GitHub Actions → "Release Dry Run" → "Run workflow"

# Option B: Push to test branch
git checkout -b test-release-v1.7.2
git push origin test-release-v1.7.2
```

**Pros**: 100% identical to real release environment
**Cons**: Uses GitHub Actions minutes, takes 5-10 minutes

### 3. Real Release (When Confident)

Only after dry runs pass:

```bash
git tag v1.7.2
git push origin v1.7.2
```

## Release Gate Overview

All approaches test these 6 mandatory gates:

1. **Gate 1**: Core Build (`cargo build --features huggingface`)
2. **Gate 2**: CUDA Build (with CPU fallback if no CUDA Toolkit)
3. **Gate 3**: Template Packaging (with `--allow-dirty` for Cargo.lock)
4. **Gate 4**: Binary Size (20MB constitutional limit)
5. **Gate 5**: Test Suite (`cargo test --all-features`)
6. **Gate 6**: Documentation (`cargo doc --all-features`)

## Recommended Workflow

```bash
# 1. Quick local check
./scripts/dry-run-release.sh

# 2. If local passes, test in exact GitHub environment
git checkout -b test-release-v1.7.2
git push origin test-release-v1.7.2

# 3. If GitHub dry run passes, create real release
git checkout main
git tag v1.7.2
git push origin v1.7.2

# 4. Clean up test branch
git push origin --delete test-release-v1.7.2
git branch -d test-release-v1.7.2
```

## Troubleshooting

### Gate 2 (CUDA) Fails
- **Locally**: Install CUDA Toolkit or accept CPU-only fallback
- **GitHub**: Automatic fallback to CPU-only validation

### Gate 3 (Templates) Fails  
- Check that `templates/docker/Dockerfile` exists
- Commit any outstanding changes
- The system handles Cargo.lock changes automatically

### Gate 4 (Binary Size) Fails
- Binary exceeded 20MB constitutional limit
- Review dependencies and features
- Consider excluding debug symbols

### Gate 5 (Tests) Fails
- Fix failing tests before release
- All tests must pass with `--all-features`

### Gate 6 (Documentation) Fails
- Fix documentation compilation errors
- Ensure all public APIs are documented

## Emergency Release (Skip Some Gates)

**Only for critical security fixes:**

```bash
# Create release workflow that skips specific gates
git tag v1.7.2-emergency
```

(Requires modifying the release workflow)

## Files In This System

- `scripts/dry-run-release.sh` - Local complete emulation
- `.github/workflows/release-dry-run.yml` - Private GitHub testing  
- `.github/workflows/release.yml` - Real release gates
- `RELEASE_PROCESS.md` - This documentation

## Why This Works

1. **Identical Commands**: Dry runs use the exact same cargo commands as release
2. **Environment Parity**: GitHub dry run uses same ubuntu-latest as release
3. **Systematic Issues Fixed**: Cargo.lock and CUDA issues handled automatically
4. **Private Testing**: No more public failures during development
5. **Confidence**: Only release when you know it will work

## Success Metrics

- ✅ Zero public release failures
- ✅ Predictable release process  
- ✅ Fast feedback loop
- ✅ Same gates, multiple testing environments
- ✅ Green CI badges

---

**Remember**: Always run dry tests before public releases. Your future self will thank you.