//! Regression test for Issue #131: ARM64 CI/CD support
//!
//! Issue: https://github.com/Michael-A-Kuykendall/shimmy/issues/131
//!
//! ## Problem
//! User @Slach requested ARM64 Linux support for NVIDIA DXG Spark platform.
//! Release workflow only built x86_64 binaries, leaving ARM64 users unable to
//! download precompiled binaries.
//!
//! ## Solution
//! - Added aarch64-unknown-linux-gnu target to release workflow
//! - Used cross-rs for ARM64 cross-compilation on x86_64 runners
//! - Included ARM64 binary in release artifacts
//!
//! ## Test Coverage
//! - Release workflow includes ARM64 Linux target
//! - Cross-compilation is configured for ARM64
//! - ARM64 binaries are uploaded as release artifacts

#[test]
fn test_release_workflow_has_arm64_linux_target() {
    // Issue #131: Verify release workflow includes aarch64-unknown-linux-gnu target
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    assert!(
        workflow.contains("aarch64-unknown-linux-gnu"),
        "Release workflow must include aarch64-unknown-linux-gnu target for ARM64 Linux support"
    );
}

#[test]
fn test_release_workflow_configures_cross_compilation() {
    // Issue #131: Verify cross-compilation is set up for ARM64 builds
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // Check for cross tool installation or use-cross flag
    assert!(
        workflow.contains("cross") || workflow.contains("use-cross"),
        "Release workflow must configure cross-compilation for ARM64 Linux"
    );
}

#[test]
fn test_release_workflow_includes_arm64_artifact() {
    // Issue #131: Verify ARM64 binary is included in release artifacts
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    assert!(
        workflow.contains("shimmy-linux-aarch64"),
        "Release workflow must upload shimmy-linux-aarch64 artifact"
    );
}

#[test]
fn test_arm64_build_uses_appropriate_features() {
    // Issue #131: Verify ARM64 builds use CPU-only features
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // ARM64 Linux should use huggingface,llama features (CPU-only)
    // Look for the ARM64 target configuration section
    let arm64_section_exists = workflow.contains("aarch64-unknown-linux-gnu");
    assert!(
        arm64_section_exists,
        "ARM64 target configuration must exist in workflow"
    );

    // Verify feature configuration logic exists
    assert!(
        workflow.contains("FEATURES=") && workflow.contains("llama"),
        "Workflow must configure features based on target platform"
    );
}

#[test]
fn test_cross_tool_installation_conditional() {
    // Issue #131: Verify cross tool is only installed when needed
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // Should have conditional logic for cross installation
    assert!(
        workflow.contains("if:") && workflow.contains("cross"),
        "Cross tool installation should be conditional on use-cross flag"
    );
}

#[test]
fn test_build_command_supports_both_cargo_and_cross() {
    // Issue #131: Verify build command can use either cargo or cross
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // Should have logic to choose between cargo and cross
    let has_conditional_build = workflow.contains("if") && workflow.contains("cross build");
    assert!(
        has_conditional_build,
        "Build step must support both native cargo and cross-compilation"
    );
}

#[test]
fn test_release_artifacts_include_all_platforms() {
    // Issue #131: Verify all platform binaries are included in releases
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // Check for all expected platform artifacts
    let platforms = vec![
        "shimmy-linux-x86_64",   // x86_64 Linux
        "shimmy-linux-aarch64",  // ARM64 Linux (Issue #131)
        "shimmy-windows-x86_64", // x86_64 Windows
        "shimmy-macos-intel",    // x86_64 macOS
        "shimmy-macos-arm64",    // ARM64 macOS (Apple Silicon)
    ];

    for platform in platforms {
        assert!(
            workflow.contains(platform),
            "Release workflow must include {} artifact",
            platform
        );
    }
}

#[test]
fn test_arm64_target_triple_is_correct() {
    // Issue #131: Verify ARM64 target uses correct Rust target triple
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // ARM64 Linux should use aarch64-unknown-linux-gnu (not musl, not android)
    assert!(
        workflow.contains("aarch64-unknown-linux-gnu"),
        "ARM64 Linux must use aarch64-unknown-linux-gnu target triple"
    );

    // Should NOT use Android or musl for standard ARM64 Linux
    let arm64_android = workflow.contains("aarch64-linux-android");

    assert!(
        !arm64_android,
        "Should not use Android target for standard ARM64 Linux"
    );

    // Note: musl ARM64 could be added in future, but not for initial implementation
}

#[test]
fn test_workflow_matrix_includes_arm64() {
    // Issue #131: Verify build matrix includes ARM64 configuration
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // Check that matrix strategy includes ARM64 target
    let has_matrix = workflow.contains("matrix:") || workflow.contains("strategy:");
    assert!(
        has_matrix,
        "Workflow must use matrix strategy for multi-platform builds"
    );

    assert!(
        workflow.contains("aarch64-unknown-linux-gnu"),
        "Build matrix must include ARM64 Linux target"
    );
}

#[test]
fn test_arm64_binary_name_convention() {
    // Issue #131: Verify ARM64 binary follows naming convention
    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // ARM64 Linux binary should be named consistently
    assert!(
        workflow.contains("shimmy-linux-aarch64"),
        "ARM64 binary must follow naming convention: shimmy-linux-aarch64"
    );
}

#[test]
fn test_issue_131_user_request_fulfilled() {
    // Issue #131: Verify user's specific request is addressed
    // User wanted ARM64 support for NVIDIA DXG Spark

    let workflow = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Release workflow file should exist");

    // Verify ARM64 Linux target is present (what user requested)
    assert!(
        workflow.contains("aarch64-unknown-linux-gnu"),
        "Issue #131: ARM64 Linux support must be present for NVIDIA DXG Spark"
    );

    // Verify it will produce a downloadable binary
    assert!(
        workflow.contains("artifact-name: shimmy-linux-aarch64"),
        "Issue #131: ARM64 binary must be uploaded as release artifact"
    );
}
