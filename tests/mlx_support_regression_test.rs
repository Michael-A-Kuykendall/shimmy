/// MLX support regression test for Issue #68
///
/// This test ensures that MLX support is properly compiled into macOS builds
/// and prevents regression of the issue where macOS binaries were missing MLX features.

use std::process::Command;

#[test]
fn test_mlx_feature_compilation() {
    // Test that MLX feature can be compiled
    let output = Command::new("cargo")
        .args(&["check", "--no-default-features", "--features", "mlx"])
        .output()
        .expect("Failed to run cargo check with mlx feature");

    assert!(output.status.success(),
           "MLX feature should compile successfully: {}",
           String::from_utf8_lossy(&output.stderr));

    println!("✅ MLX feature compilation test passed");
}

#[test]
fn test_apple_feature_set_compilation() {
    // Test that the 'apple' feature set (which includes MLX) compiles
    let output = Command::new("cargo")
        .args(&["check", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to run cargo check with apple feature set");

    assert!(output.status.success(),
           "Apple feature set should compile successfully: {}",
           String::from_utf8_lossy(&output.stderr));

    println!("✅ Apple feature set compilation test passed");
}

#[test]
fn test_gpu_info_with_mlx_compiled() {
    // Build with apple features and test gpu-info output
    let build_output = Command::new("cargo")
        .args(&["build", "--release", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to build with apple features");

    assert!(build_output.status.success(),
           "Build with apple features should succeed: {}",
           String::from_utf8_lossy(&build_output.stderr));

    // Test gpu-info command
    let gpu_info_output = Command::new("./target/release/shimmy")
        .arg("gpu-info")
        .output()
        .expect("Failed to run shimmy gpu-info");

    assert!(gpu_info_output.status.success(),
           "gpu-info command should succeed: {}",
           String::from_utf8_lossy(&gpu_info_output.stderr));

    let output_text = String::from_utf8_lossy(&gpu_info_output.stdout);

    // MLX should be compiled in (not showing "Disabled (compile with --features mlx)")
    assert!(!output_text.contains("MLX Backend: Disabled"),
           "MLX should not show as disabled when compiled with apple features");

    // Should show either "Not available (requires Apple Silicon)" or actual MLX info
    assert!(output_text.contains("MLX Backend:"),
           "MLX Backend information should be present in gpu-info output");

    // Should NOT show the old error message
    assert!(!output_text.contains("compile with --features mlx"),
           "Should not suggest compiling with MLX when it's already compiled in");

    println!("✅ GPU info with MLX test passed");
    println!("MLX output: {}", output_text);
}

#[test]
fn test_mlx_feature_availability() {
    // Test that MLX code is available when feature is enabled
    #[cfg(feature = "mlx")]
    {
        // This code should only compile when MLX feature is enabled
        use shimmy::engine::mlx::MLXEngine;

        // Test that MLX engine type exists and can be referenced
        let _engine_check = std::marker::PhantomData::<MLXEngine>;

        println!("✅ MLX engine code is available when feature is enabled");
    }

    #[cfg(not(feature = "mlx"))]
    {
        println!("ℹ️  MLX feature not enabled in this test build");
    }
}

#[test]
fn test_regression_issue_68_scenarios() {
    // Test the specific scenarios reported in Issue #68

    // Scenario 1: cargo install with MLX should work (template compilation)
    // We test compilation rather than full install for speed
    let mlx_compile_test = Command::new("cargo")
        .args(&["check", "--features", "mlx"])
        .output()
        .expect("Failed to check MLX compilation");

    assert!(mlx_compile_test.status.success(),
           "Issue #68 regression: MLX compilation should work (template files must be included)");

    // Scenario 2: macOS binaries should include MLX support
    // Test that apple feature set includes MLX
    let apple_features_test = Command::new("cargo")
        .args(&["check", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to check apple features");

    assert!(apple_features_test.status.success(),
           "Issue #68 regression: Apple feature set should compile with MLX support");

    println!("✅ Issue #68 regression tests passed");
}

#[test]
fn test_ci_build_matrix_features() {
    // Test the feature combinations used in the CI build matrix

    // Test Linux features
    let linux_test = Command::new("cargo")
        .args(&["check", "--no-default-features", "--features", "huggingface,llama"])
        .output()
        .expect("Failed to check Linux features");

    assert!(linux_test.status.success(), "Linux feature set should compile");

    // Test Windows features
    let windows_test = Command::new("cargo")
        .args(&["check", "--no-default-features", "--features", "gpu"])
        .output()
        .expect("Failed to check Windows features");

    assert!(windows_test.status.success(), "Windows feature set should compile");

    // Test macOS features (the fix for Issue #68)
    let macos_test = Command::new("cargo")
        .args(&["check", "--no-default-features", "--features", "apple"])
        .output()
        .expect("Failed to check macOS features");

    assert!(macos_test.status.success(), "macOS feature set should compile");

    println!("✅ CI build matrix feature combinations test passed");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_apple_feature_build_and_run() {
        // Full integration test: build and run with apple features
        let build_result = Command::new("cargo")
            .args(&["build", "--release", "--no-default-features", "--features", "apple"])
            .output()
            .expect("Failed to build with apple features");

        assert!(build_result.status.success(),
               "Full apple feature build should succeed");

        // Test that the binary works
        let version_result = Command::new("./target/release/shimmy")
            .arg("--version")
            .output()
            .expect("Failed to run shimmy --version");

        assert!(version_result.status.success(),
               "shimmy --version should work with apple features");

        let version_output = String::from_utf8_lossy(&version_result.stdout);
        assert!(version_output.contains("shimmy"),
               "Version output should contain 'shimmy': {}", version_output);

        println!("✅ Full apple feature integration test passed");
    }
}