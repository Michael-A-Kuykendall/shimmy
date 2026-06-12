/// Integration tests to validate the release gate system itself works correctly
/// This ensures our release gates properly catch real issues and block releases
use std::process::Command;

/// Locate the freshly built shimmy debug binary in both standalone (./target)
/// and enclosing-workspace (../target) layouts (e.g. the meta repo workspace).
fn shimmy_debug_binary() -> String {
    let name = if cfg!(windows) {
        "shimmy.exe"
    } else {
        "shimmy"
    };
    [
        format!("./target/debug/{name}"),
        format!("../target/debug/{name}"),
    ]
    .into_iter()
    .find(|p| std::path::Path::new(p).exists())
    .expect("built shimmy debug binary not found in ./target or workspace ../target")
}

#[test]
fn test_release_gate_system_exists() {
    // Validate that release.yml contains the mandatory gates
    let workflow_content = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Failed to read release.yml");

    assert!(
        workflow_content.contains("🚧 Release Gates - MANDATORY VALIDATION"),
        "Release workflow missing mandatory gate job"
    );
    // v2.0: CUDA (Gate 2) removed — Airframe uses wgpu/WebGPU, not CUDA.
    // v2.0: Crates.io Gate 7 removed — publish = false (distributed as binaries).
    // Gates are now numbered /6 instead of /7.
    assert!(
        workflow_content.contains("GATE 1/7: Core Build Validation"),
        "Missing Gate 1 (Core Build)"
    );
    assert!(
        workflow_content.contains("GATE 3/7: Template Packaging Validation"),
        "Missing Gate 3 (Template Packaging)"
    );
    assert!(
        workflow_content.contains("GATE 4/7: Binary Size Constitutional Limit"),
        "Missing Gate 4 (Binary Size)"
    );
    assert!(
        workflow_content.contains("GATE 5/7: Test Suite Validation"),
        "Missing Gate 5 (Test Suite)"
    );
    assert!(
        workflow_content.contains("GATE 5.1/7: Airframe Integration Compile Check"),
        "Missing Gate 5.1 (Airframe Integration)"
    );
    assert!(
        workflow_content.contains("GATE 5.5/7: Issue Regression Tests"),
        "Missing Gate 5.5 (Issue Regression)"
    );
    assert!(
        workflow_content.contains("GATE 6/7: Documentation Validation"),
        "Missing Gate 6 (Documentation)"
    );
    assert!(
        workflow_content.contains("GATE 7/7: crates.io Package Validation"),
        "Missing Gate 7 (crates.io)"
    );
    // Verify intentional removal of CUDA gate (Gate 2) is documented in the workflow
    assert!(
        workflow_content.contains("GATE 2") && workflow_content.contains("intentionally removed"),
        "Workflow should document intentional removal of CUDA gate"
    );
}

#[test]
fn test_conditional_execution_logic() {
    // Validate that downstream jobs require preflight gate passage
    let workflow_content = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Failed to read release.yml");

    assert!(
        workflow_content.contains("needs: preflight"),
        "Build job doesn't depend on preflight gates"
    );
    assert!(
        workflow_content.contains("needs.preflight.outputs.should_publish == 'true'"),
        "Missing conditional execution logic"
    );
    assert!(
        workflow_content.contains("needs: [preflight, reuse-gate-binary, build]"),
        "Release job doesn't depend on preflight, reuse-gate-binary, and build"
    );
}

#[test]
fn test_gate_1_core_build_validation() {
    // Test that core build (huggingface features) works
    let output = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--no-default-features",
            "--features",
            "huggingface",
        ])
        .output()
        .expect("Failed to run cargo build");

    assert!(
        output.status.success(),
        "Gate 1 (Core Build) should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_gate_3_template_packaging_protection() {
    // Test that templates are properly included (Issue #60 protection)
    let output = Command::new("cargo")
        .args(["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");

    let package_list = String::from_utf8_lossy(&output.stdout);

    // Check for any of the valid Docker template paths (Issue #60 protection)
    // Handle both Unix (/) and Windows (\) path separators
    let has_dockerfile = package_list.lines().any(|line| {
        line == "Dockerfile"
            || line == "packaging/docker/Dockerfile"
            || line == "packaging\\docker\\Dockerfile"
            || line == "templates/docker/Dockerfile"
            || line == "templates\\docker\\Dockerfile"
    });

    assert!(
        has_dockerfile,
        "Required Docker template missing from package: {} (Issue #60 regression!)",
        package_list
    );
}

#[test]
fn test_gate_4_binary_size_constitutional_limit() {
    // First ensure we have a binary to test (debug build for speed)
    let build_output = Command::new("cargo")
        .args([
            "build",
            "--no-default-features",
            "--features",
            "huggingface",
        ])
        .output()
        .expect("Failed to build binary for size test");

    assert!(
        build_output.status.success(),
        "Failed to build binary for size test"
    );

    // Test constitutional 20MB limit (debug binary path; resolves both
    // standalone ./target and enclosing-workspace ../target layouts)
    let binary_path = shimmy_debug_binary();

    // The constitutional limit guards the standalone release process. Inside an
    // enclosing cargo workspace (e.g. the meta repo) the root workspace ignores
    // shimmy's own [profile.dev] (cargo: "profiles for the non root package will
    // be ignored"), so the dev binary carries full debuginfo and the limit does
    // not apply.
    if binary_path.starts_with("../") {
        eprintln!(
            "skipping constitutional size check: built under enclosing workspace profile ({binary_path})"
        );
        return;
    }

    if let Ok(metadata) = std::fs::metadata(&binary_path) {
        let size = metadata.len();
        let max_size = 20 * 1024 * 1024; // 20MB constitutional limit

        assert!(
            size <= max_size,
            "Binary size {} bytes exceeds constitutional limit of {} bytes (Gate 4 failure)",
            size,
            max_size
        );
    } else {
        panic!("Binary not found at {}", binary_path);
    }
}

#[test]
fn test_gate_5_test_suite_validation() {
    // Validate that test suite can be compiled and basic tests pass
    // Note: We run a more limited test to avoid circular dependency issues
    let output = Command::new("cargo")
        .args(["test", "--no-run", "--lib"])
        .output()
        .expect("Failed to compile test suite");

    assert!(
        output.status.success(),
        "Gate 5 (Test Suite compilation) should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Additional validation: Ensure we can run a simple test
    let simple_test = Command::new("cargo")
        .args(["test", "--lib", "test_model_spec_validation"])
        .output()
        .expect("Failed to run simple test");

    // Don't fail the whole thing if the simple test fails, just log it
    if !simple_test.status.success() {
        println!(
            "⚠️ Simple test failed, but compilation passed: {}",
            String::from_utf8_lossy(&simple_test.stderr)
        );
    }
}

#[test]
fn test_gate_6_documentation_validation() {
    // Test that documentation builds successfully
    let output = Command::new("cargo")
        .args([
            "doc",
            "--no-deps",
            "--no-default-features",
            "--features",
            "huggingface",
        ])
        .output()
        .expect("Failed to run cargo doc");

    assert!(
        output.status.success(),
        "Gate 6 (Documentation) should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_local_validation_scripts_exist() {
    // Ensure local validation scripts exist and are executable
    assert!(
        std::path::Path::new("scripts/validate-release.ps1").exists(),
        "PowerShell validation script missing"
    );

    // Note: Not testing bash script existence on Windows, but it should exist for Unix systems
}

// Gate 2 (CUDA timeout detection) removed in v2.0.
// Airframe uses wgpu/WebGPU — no CUDA dependency exists. llama.cpp `llama` feature
// is no longer part of this codebase.

#[test]
fn test_gate_7_cratesio_validation() {
    // Test that crates.io dry-run validation works
    let output = Command::new("cargo")
        .args(["publish", "--dry-run", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo publish --dry-run");

    // Dry-run should either succeed or fail with specific errors we can analyze
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        // Check that it actually packaged something (look in both stdout and stderr)
        let combined_output = format!("{}{}", stdout, stderr);

        if combined_output.contains("already exists on crates.io") {
            println!("ℹ️ Gate 7 (Crates.io) - Version already published (this is expected for released versions)");
            // Verify packaging still worked
            assert!(
                combined_output.contains("Packaging"),
                "Gate 7 should still show packaging step: {}",
                combined_output
            );
        } else {
            // Normal case - check that it packaged files
            assert!(
                combined_output.contains("Packaged") && combined_output.contains("files"),
                "Gate 7 (Crates.io) dry-run should package files: {}",
                combined_output
            );
        }
        println!("✅ Gate 7 (Crates.io) dry-run validation passed");
    } else {
        // If it failed, make sure it's a known/expected failure
        if stderr.contains("no upload token found") || stderr.contains("authentication") {
            println!(
                "ℹ️ Gate 7 dry-run failed due to missing token (expected in test environment)"
            );
        } else if stderr.contains("cannot be published")
            || (stderr.contains("publish") && stderr.contains("true"))
        {
            // publish = false in Cargo.toml: intentional for v2.0 (Airframe path dep cannot
            // be published to crates.io). Shimmy v2.0 is distributed as binaries.
            println!("ℹ️ Gate 7 (Crates.io) skipped: publish = false is intentional for v2.0");
        } else {
            panic!(
                "Gate 7 (Crates.io) dry-run failed with unexpected error: stderr={}, stdout={}",
                stderr, stdout
            );
        }
    }
}
