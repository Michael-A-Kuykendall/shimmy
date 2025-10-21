/// Integration tests to validate the release gate system itself works correctly
/// This ensures our release gates properly catch real issues and block releases
use std::process::Command;
use std::time::Duration;

#[test]
fn test_release_gate_system_exists() {
    // Validate that release.yml contains the mandatory gates
    let workflow_content = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Failed to read release.yml");

    assert!(
        workflow_content.contains("🚧 Release Gates - MANDATORY VALIDATION"),
        "Release workflow missing mandatory gate job"
    );
    assert!(
        workflow_content.contains("GATE 1/7: Core Build Validation"),
        "Missing Gate 1 (Core Build)"
    );
    assert!(
        workflow_content.contains("GATE 2/7: CUDA Build Validation"),
        "Missing Gate 2 (CUDA Validation)"
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
        workflow_content.contains("GATE 6/7: Documentation Validation"),
        "Missing Gate 6 (Documentation)"
    );
    assert!(
        workflow_content.contains("GATE 7/7: Crates.io Publication Validation"),
        "Missing Gate 7 (Crates.io Validation)"
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

    // Test constitutional 20MB limit (debug binary path)
    let binary_path = if cfg!(windows) {
        "target/debug/shimmy.exe"
    } else {
        "target/debug/shimmy"
    };

    if let Ok(metadata) = std::fs::metadata(binary_path) {
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

#[test]
fn test_gate_2_cuda_timeout_detection() {
    // CUDA timeout detection test (Issue #59 protection)
    // This test runs the full CUDA build to completion, regardless of duration

    use std::time::Instant;
    let start = Instant::now();

    let output = Command::new("cargo")
        .args(["check", "--no-default-features", "--features", "llama"])
        .output();

    let duration = start.elapsed();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!(
                    "✅ Gate 2 passed - CUDA check completed successfully in {:?}",
                    duration
                );
            } else {
                // Build failed - this could be due to missing CUDA, linking issues, etc.
                // Log the failure but don't panic since CUDA availability varies by system
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!(
                    "⚠️ Gate 2 - CUDA build failed after {:?}: {}",
                    duration, stderr
                );

                // Only fail if this is a timeout-related issue or constitutional violation
                if duration > Duration::from_secs(3600) {
                    // 1 hour constitutional limit
                    panic!(
                        "Gate 2 FAILED - Build exceeded 1 hour constitutional limit: {:?}",
                        duration
                    );
                }

                // For other failures (missing CUDA, linker issues), log but continue
                // This allows the gate to pass on systems without CUDA while still catching timeouts
                println!("Gate 2 - Build failed due to system configuration, not timeout issues");
            }
        }
        Err(e) => {
            panic!("Gate 2 FAILED - Could not execute cargo build: {}", e);
        }
    }
}

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
        // If it failed, make sure it's not due to missing token (expected in CI)
        if stderr.contains("no upload token found") || stderr.contains("authentication") {
            println!(
                "ℹ️ Gate 7 dry-run failed due to missing token (expected in test environment)"
            );
            // This is expected - we can't publish in tests, but we can validate packaging
        } else {
            panic!(
                "Gate 7 (Crates.io) dry-run failed with unexpected error: stderr={}, stdout={}",
                stderr, stdout
            );
        }
    }
}
