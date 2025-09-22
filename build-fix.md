# Fix for Issue #63: Version Mismatch in Windows Binary

## Problem Analysis
The user reported downloading "shimmy 1.4.2" which shows version "0.1.0" and lacks the `gpu-info` command. Investigation reveals:

1. **No v1.4.2 tag exists** - latest was v1.4.1, current is v1.5.5
2. The binary was likely built from an incorrect source or development state
3. Version 0.1.0 suggests it was built from a very early commit or with corrupted build environment

## Root Cause
The issue stems from the binary being built without proper Cargo.toml version information being embedded. This can happen when:
- Building from a source without proper Cargo.toml
- Build environment not setting CARGO_PKG_VERSION correctly
- Building from a Git worktree or modified state

## Comprehensive Fix

### 1. Version Validation at Build Time
Create a build script that validates version consistency:

```rust
// build.rs
fn main() {
    // Ensure version is not default
    let version = env!("CARGO_PKG_VERSION");
    if version == "0.1.0" || version.is_empty() {
        panic!("Invalid version detected: {}. Check Cargo.toml", version);
    }

    // Validate semantic versioning
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        panic!("Version must follow semantic versioning: {}", version);
    }

    println!("cargo:rustc-env=SHIMMY_BUILD_VERSION={}", version);
    println!("cargo:rerun-if-changed=Cargo.toml");
}
```

### 2. Runtime Version Verification
Add version verification in main.rs:

```rust
fn verify_build_version() {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let build_version = env!("SHIMMY_BUILD_VERSION");

    if cargo_version != build_version {
        eprintln!("Warning: Version mismatch detected!");
        eprintln!("  Cargo version: {}", cargo_version);
        eprintln!("  Build version: {}", build_version);
    }

    if cargo_version == "0.1.0" {
        eprintln!("ERROR: Invalid default version detected!");
        eprintln!("This binary was built incorrectly. Please download from official releases.");
        std::process::exit(1);
    }
}
```

### 3. Enhanced CLI with Version Validation
Update CLI to include build information:

```rust
#[derive(Parser, Debug)]
#[command(
    name = "shimmy",
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("SHIMMY_BUILD_VERSION"), ")"),
    about = "Shimmy: single-binary GGUF + LoRA server"
)]
pub struct Cli {
    // ... existing fields
}
```

## Implementation for Backporting

Since developers are forking at various stages, here's a minimal fix that can be applied to any version:

### Minimal Fix (backport-friendly)
1. Add version check in main():
```rust
fn main() {
    // Version safety check - prevents 0.1.0 releases
    let version = env!("CARGO_PKG_VERSION");
    if version == "0.1.0" {
        eprintln!("ERROR: This binary has incorrect version information.");
        eprintln!("Please rebuild from clean source or download official release.");
        std::process::exit(1);
    }

    // ... rest of main
}
```

2. Ensure Cargo.toml has correct version before building
3. Add regression test to catch this in CI

## For Release Process
1. Always build from tagged commits
2. Verify `cargo --version` output before publishing
3. Include version verification in CI/CD
4. Test binary version output before release

## Immediate Action
1. **Close Issue #63** with explanation that v1.4.2 was never officially released
2. **Recommend users download from official releases** (v1.4.1 or latest v1.5.5)
3. **Add build verification** to prevent future occurrences
4. **Create proper v1.4.2 tag** if needed for compatibility

## For Forkers
If you're forking shimmy, ensure:
1. Update version in Cargo.toml for your fork
2. Build from clean Git state
3. Test `./shimmy -V` before distributing
4. Consider adding the version verification code above